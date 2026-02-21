use crate::error::*;
use crate::lua_request;
use pasta_lua::loader::LoggingConfig;
use pasta_lua::mlua::{Function, Table};
use pasta_lua::{GlobalLoggerRegistry, LoadDirGuard, PastaLoader, PastaLuaRuntime};
use std::{ffi::*, path::*};
use tracing::{debug, error, info, trace, warn};

/// Initialize global tracing subscriber with LoggingConfig.
///
/// # Filter Priority
/// 1. PASTA_LOG environment variable (highest)
/// 2. pasta.toml [logging].filter
/// 3. pasta.toml [logging].level
/// 4. Default: "debug"
///
/// # Note
/// Never fails - falls back to default filter on any error.
/// Uses try_init() so subsequent calls are safely ignored.
pub fn init_tracing_with_config(config: &LoggingConfig) {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    // Build filter with priority: PASTA_LOG > config.filter > config.level > default
    let filter = EnvFilter::try_from_env("PASTA_LOG")
        .or_else(|_| EnvFilter::try_new(config.to_filter_directive()))
        .unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to parse log filter '{}', using default: {}",
                config.to_filter_directive(),
                e
            );
            EnvFilter::new("debug")
        });

    let _ = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(GlobalLoggerRegistry::instance().clone())
                .with_ansi(false)
                .with_target(true)
                .with_level(true)
                .with_filter(filter),
        )
        .try_init();
}

pub trait Shiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool>;
    fn request<S: AsRef<str>>(&mut self, request: S) -> MyResult<String>;
}

/// PastaShiori - SHIORI implementation using pasta_lua engine.
///
/// Manages the lifecycle of the Pasta script engine, including:
/// - Runtime initialization via PastaLoader
/// - SHIORI protocol handling
///
/// Note: Logging is handled internally by PastaLuaRuntime (encapsulation).
/// PastaShiori only manages the GlobalLoggerRegistry for log routing.
#[derive(Default)]
pub struct PastaShiori {
    /// DLL module handle (for future Windows API integration)
    hinst: isize,

    /// Base directory for ghost scripts (master/ directory)
    load_dir: Option<PathBuf>,

    /// Pasta Lua runtime instance (contains logger internally)
    runtime: Option<PastaLuaRuntime>,

    /// Cached SHIORI.load function
    load_fn: Option<Function>,

    /// Cached SHIORI.request function
    request_fn: Option<Function>,

    /// Cached SHIORI.unload function
    unload_fn: Option<Function>,
}

// SAFETY: PastaShiori is used in a single-threaded context (SHIORI DLL).
// The OnceLock ensures only one instance exists, and Mutex protects access.
// The Lua runtime is only accessed from the main thread.
unsafe impl Send for PastaShiori {}
unsafe impl Sync for PastaShiori {}

impl Drop for PastaShiori {
    fn drop(&mut self) {
        // Call SHIORI.unload if available (before runtime drop)
        self.call_lua_unload();

        // Unregister logger from global registry
        if let Some(ref load_dir) = self.load_dir {
            GlobalLoggerRegistry::instance().unregister(load_dir);
            info!(load_dir = %load_dir.display(), "Unregistered logger");
        }

        // Clear cached functions before dropping runtime
        self.clear_cached_lua_functions();

        // Drop runtime (logger is dropped with it)
        self.runtime = None;
    }
}

impl Shiori for PastaShiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool> {
        // Convert load_dir to PathBuf
        let load_dir_path: PathBuf = load_dir.as_ref().into();

        // Validate load_dir exists
        if !load_dir_path.exists() {
            error!(path = %load_dir_path.display(), "Load directory not found");
            return Ok(false);
        }

        // If already loaded, cleanup previous instance
        if self.runtime.is_some() {
            info!("Releasing existing runtime for reload");
            // Clear cached functions before releasing runtime
            self.clear_cached_lua_functions();
            if let Some(ref old_load_dir) = self.load_dir {
                GlobalLoggerRegistry::instance().unregister(old_load_dir);
            }
            self.runtime = None;
        }

        // Save hinst and load_dir
        self.hinst = hinst;
        self.load_dir = Some(load_dir_path.clone());

        // Set load_dir context for logging
        let _guard = LoadDirGuard::new(load_dir_path.clone());

        info!(
            load_dir = %load_dir_path.display(),
            hinst = hinst,
            "Starting PastaShiori load"
        );

        // Load runtime via PastaLoader (logger is created inside)
        match PastaLoader::load(&load_dir_path) {
            Ok(runtime) => {
                // Initialize tracing subscriber with config from pasta.toml (Requirement 6)
                // Priority: PASTA_LOG env var > filter > level > default ("debug")
                let logging_config = runtime
                    .config()
                    .and_then(|c| c.logging())
                    .unwrap_or_default();
                init_tracing_with_config(&logging_config);

                // Immediately log load_dir after tracing initialization (Requirement 7)
                info!(
                    load_dir = %load_dir_path.display(),
                    "Logger initialized for ghost directory"
                );

                // Register runtime's logger with global registry for log routing
                if let Some(logger) = runtime.logger() {
                    GlobalLoggerRegistry::instance().register(load_dir_path.clone(), logger);
                    debug!(load_dir = %load_dir_path.display(), "Registered logger with GlobalLoggerRegistry");
                }

                // Cache SHIORI functions (load/request/unload)
                self.cache_lua_functions(&runtime);

                self.runtime = Some(runtime);

                // Call SHIORI.load if available (using cached function)
                if !self.call_lua_load(hinst, &load_dir_path) {
                    return Ok(false);
                }

                info!(load_dir = %load_dir_path.display(), "PastaShiori load completed");
                Ok(true)
            }
            Err(e) => {
                error!(
                    load_dir = %load_dir_path.display(),
                    error = %e,
                    "PastaShiori load failed"
                );
                // Return false on error (SHIORI convention)
                Ok(false)
            }
        }
    }

    fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
        // Check if runtime is initialized
        let _runtime = self.runtime.as_ref().ok_or(MyError::NotInitialized)?;

        // Set load_dir context for logging
        let _guard = self.load_dir.as_ref().map(|p| LoadDirGuard::new(p.clone()));

        let req = req.as_ref();
        trace!(request_len = req.len(), "Processing SHIORI request");

        // Call SHIORI.request using cached function
        self.call_lua_request(req)
    }
}

impl PastaShiori {
    /// Get a reference to the internal Lua runtime.
    /// Returns None if the runtime has not been initialized via load().
    pub fn runtime(&self) -> Option<&PastaLuaRuntime> {
        self.runtime.as_ref()
    }

    /// Cache SHIORI.load, SHIORI.request, and SHIORI.unload functions from Lua runtime.
    /// This eliminates the need for hash table lookups on each request.
    fn cache_lua_functions(&mut self, runtime: &PastaLuaRuntime) {
        let lua = runtime.lua();
        let globals = lua.globals();

        // Get SHIORI table
        let shiori_table: Result<Table, _> = globals.get("SHIORI");
        match shiori_table {
            Ok(table) => {
                // Cache SHIORI.load function
                self.load_fn = match table.get::<Function>("load") {
                    Ok(f) => {
                        trace!("SHIORI.load function cached");
                        Some(f)
                    }
                    Err(_) => {
                        warn!("SHIORI.load function not found");
                        None
                    }
                };

                // Cache SHIORI.request function
                self.request_fn = match table.get::<Function>("request") {
                    Ok(f) => {
                        trace!("SHIORI.request function cached");
                        Some(f)
                    }
                    Err(_) => {
                        warn!("SHIORI.request function not found");
                        None
                    }
                };

                // Cache SHIORI.unload function
                self.unload_fn = match table.get::<Function>("unload") {
                    Ok(f) => {
                        trace!("SHIORI.unload function cached");
                        Some(f)
                    }
                    Err(_) => {
                        debug!("SHIORI.unload function not found (optional)");
                        None
                    }
                };
            }
            Err(e) => {
                warn!(error = %e, "SHIORI table not found");
                self.load_fn = None;
                self.request_fn = None;
                self.unload_fn = None;
            }
        }
    }

    /// Clear all cached SHIORI functions.
    /// Called before reload or when runtime is released.
    fn clear_cached_lua_functions(&mut self) {
        self.load_fn = None;
        self.request_fn = None;
        self.unload_fn = None;
    }

    /// Call SHIORI.load function with hinst and load_dir using cached function.
    /// Returns true if successful or if function doesn't exist (skip).
    /// Returns false if function returns false or errors.
    fn call_lua_load(&self, hinst: isize, load_dir: &Path) -> bool {
        // Use cached load_fn directly
        let load_fn = match &self.load_fn {
            Some(f) => f,
            None => {
                debug!("SHIORI.load not available, skipping");
                return true;
            }
        };

        // Call SHIORI.load(hinst, load_dir)
        let load_dir_str = load_dir.to_string_lossy().to_string();
        match load_fn.call::<bool>((hinst, load_dir_str)) {
            Ok(true) => {
                trace!("SHIORI.load returned true");
                true
            }
            Ok(false) => {
                warn!("SHIORI.load returned false");
                false
            }
            Err(e) => {
                error!(error = %e, "SHIORI.load execution failed");
                false
            }
        }
    }

    /// Call SHIORI.request function using cached function.
    /// Parses request text and passes parsed table to Lua.
    /// Returns 204 response if function doesn't exist.
    /// Returns 400 Bad Request if request parsing fails.
    fn call_lua_request(&self, request: &str) -> MyResult<String> {
        // Use cached request_fn directly
        let request_fn = match &self.request_fn {
            Some(f) => f,
            None => {
                debug!("SHIORI.request not available, returning default 204 response");
                return Ok(Self::default_204_response());
            }
        };

        // Get runtime for Lua context
        let runtime = self.runtime.as_ref().ok_or(MyError::NotInitialized)?;
        let lua = runtime.lua();

        // Parse request text to Lua table
        let req_table = match lua_request::parse_request(lua, request) {
            Ok(table) => table,
            Err(e) => {
                error!(error = %e, "SHIORI request parsing failed");
                return Ok(Self::default_400_response());
            }
        };

        // Call SHIORI.request(req) with parsed table
        match request_fn.call::<String>(req_table) {
            Ok(response) => {
                // Log request/response at DEBUG level for non-204 responses
                if !response.starts_with("SHIORI/3.0 204 No Content") {
                    debug!(request = %request, "### SHIORI request ###\n");
                    debug!(response = %response, "### SHIORI response###\n");
                }
                trace!(response_len = response.len(), "SHIORI.request completed");
                Ok(response)
            }
            Err(e) => {
                error!(error = %e, "SHIORI.request execution failed");
                Err(MyError::from(e))
            }
        }
    }

    /// Call SHIORI.unload function using cached function.
    /// Logs warning on error but does not propagate (safe for Drop).
    fn call_lua_unload(&self) {
        // Check both unload_fn and runtime exist
        let (unload_fn, _runtime) = match (&self.unload_fn, &self.runtime) {
            (Some(f), Some(r)) => (f, r),
            _ => {
                debug!("SHIORI.unload not available, skipping");
                return;
            }
        };

        // Set load_dir context for logging
        let _guard = self.load_dir.as_ref().map(|p| LoadDirGuard::new(p.clone()));

        // Call SHIORI.unload()
        if let Err(e) = unload_fn.call::<()>(()) {
            warn!(error = %e, "SHIORI.unload failed");
        } else {
            info!("SHIORI.unload called successfully");
        }
    }

    /// Generate default 204 No Content response.
    fn default_204_response() -> String {
        "SHIORI/3.0 204 No Content\r\n\
         Charset: UTF-8\r\n\
         Sender: Pasta\r\n\
         \r\n"
            .to_string()
    }

    /// Generate default 400 Bad Request response.
    /// Used when SHIORI request parsing fails.
    fn default_400_response() -> String {
        "SHIORI/3.0 400 Bad Request\r\n\
         Charset: UTF-8\r\n\
         Sender: Pasta\r\n\
         \r\n"
            .to_string()
    }
}

#[cfg(test)]
#[path = "shiori_tests.rs"]
mod tests;
