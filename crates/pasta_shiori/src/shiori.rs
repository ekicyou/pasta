use crate::error::*;
use crate::lua_request;
use pasta_lua::mlua::{Function, Table};
use pasta_lua::{GlobalLoggerRegistry, LoadDirGuard, PastaLoader, PastaLuaRuntime};
use std::{ffi::*, path::*};
use tracing::{debug, error, info, trace, warn};

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

    /// Error message from last failed load (for X-ERROR-REASON)
    last_load_error: Option<String>,

    /// Cached SHIORI.load function
    load_fn: Option<Function>,

    /// Cached SHIORI.request function
    request_fn: Option<Function>,

    /// Cached SHIORI.unload function
    unload_fn: Option<Function>,
}

// SAFETY: PastaShiori is used within a single-instance SHIORI DLL context.
// The OnceLock<RawShiori<PastaShiori>> ensures only one instance is created.
// The Arc<Mutex<Option<PastaShiori>>> provides interior mutability with lock-based
// synchronization, ensuring that all access to the Lua runtime and cached
// functions is serialized. The Lua runtime (mlua) itself is !Send+!Sync, but
// SHIORI DLL is only ever accessed from the host application's main thread.
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
            self.clear_cached_lua_functions();
            if let Some(ref old_load_dir) = self.load_dir {
                GlobalLoggerRegistry::instance().unregister(old_load_dir);
            }
            self.runtime = None;
            self.last_load_error = None;
        }

        // Save hinst and load_dir
        self.hinst = hinst;
        self.load_dir = Some(load_dir_path.clone());

        // Stage 1: Early logger initialization (before PastaLoader::load)
        // Create a default logger so all load-phase logs are captured to file
        if let Ok(logger) = pasta_lua::PastaLogger::new(&load_dir_path, None) {
            let logger = std::sync::Arc::new(logger);
            GlobalLoggerRegistry::instance().register(load_dir_path.clone(), logger);
        }
        pasta_lua::init_tracing_with_reload(&pasta_lua::LoggingConfig::default());

        // Set load_dir context for logging
        let _guard = LoadDirGuard::new(load_dir_path.clone());

        info!(
            load_dir = %load_dir_path.display(),
            hinst = hinst,
            "Starting PastaShiori load"
        );

        // Load runtime via PastaLoader (Stage 1.5 logger update happens inside)
        match PastaLoader::load(&load_dir_path) {
            Ok(runtime) => {
                // Cache SHIORI functions (load/request/unload)
                self.cache_lua_functions(&runtime);

                self.runtime = Some(runtime);
                self.last_load_error = None;

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
                self.last_load_error = Some(format!("{}", e));
                Ok(false)
            }
        }
    }

    fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
        // Check if runtime is initialized, with detailed error on load failure
        let _runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| match &self.last_load_error {
                Some(msg) => MyError::Load(msg.clone()),
                None => MyError::NotInitialized,
            })?;

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
                self.load_fn = Self::lookup_lua_fn(&table, "load", false);
                self.request_fn = Self::lookup_lua_fn(&table, "request", false);
                self.unload_fn = Self::lookup_lua_fn(&table, "unload", true);
            }
            Err(e) => {
                warn!(error = %e, "SHIORI table not found");
                self.clear_cached_lua_functions();
            }
        }
    }

    /// Look up `SHIORI.<name>` in the table, logging cache hit/miss.
    /// `optional` functions log a miss at debug level instead of warn.
    fn lookup_lua_fn(table: &Table, name: &str, optional: bool) -> Option<Function> {
        match table.get::<Function>(name) {
            Ok(f) => {
                trace!("SHIORI.{} function cached", name);
                Some(f)
            }
            Err(_) if optional => {
                debug!("SHIORI.{} function not found (optional)", name);
                None
            }
            Err(_) => {
                warn!("SHIORI.{} function not found", name);
                None
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
                info!("SHIORI.load called successfully");
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
                return Ok(e.to_shiori_400_response());
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
}

#[cfg(test)]
#[path = "shiori_tests.rs"]
mod tests;
