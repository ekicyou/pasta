use crate::error::*;
use crate::lua_request;
use pasta_lua::mlua::{Function, Table};
use pasta_lua::{GlobalLoggerRegistry, LoadDirGuard, PastaLoader, PastaLuaRuntime};
use std::{ffi::*, path::*};
use tracing::{debug, error, info, warn};

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
        debug!(request_len = req.len(), "Processing SHIORI request");

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
                        debug!("SHIORI.load function cached");
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
                        debug!("SHIORI.request function cached");
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
                        debug!("SHIORI.unload function cached");
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
                debug!("SHIORI.load returned true");
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
                debug!(response_len = response.len(), "SHIORI.request completed");
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
            debug!("SHIORI.unload called successfully");
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Copy fixture to a temporary directory for testing.
    fn copy_fixture_to_temp(fixture_name: &str) -> TempDir {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("pasta_lua/tests/fixtures/loader")
            .join(fixture_name);
        let temp = TempDir::new().unwrap();
        copy_dir_recursive(&src, temp.path()).unwrap();

        // Copy scripts directory
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("pasta_lua");
        let scripts_src = crate_root.join("scripts");
        let scripts_dst = temp.path().join("scripts");
        if scripts_src.exists() {
            std::fs::create_dir_all(&scripts_dst).unwrap();
            copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
        }

        // Copy scriptlibs directory
        let scriptlibs_src = crate_root.join("scriptlibs");
        let scriptlibs_dst = temp.path().join("scriptlibs");
        if scriptlibs_src.exists() {
            std::fs::create_dir_all(&scriptlibs_dst).unwrap();
            copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
        }

        temp
    }

    fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                // Skip profile directories
                if entry.file_name() == "profile" {
                    continue;
                }
                std::fs::create_dir_all(&dest_path)?;
                copy_dir_recursive(&path, &dest_path)?;
            } else {
                std::fs::copy(&path, &dest_path)?;
            }
        }
        Ok(())
    }

    // ========================================================================
    // Task 11.3: 複数回load()テスト
    // ========================================================================

    #[test]
    fn test_multiple_load_releases_previous_runtime() {
        let temp1 = copy_fixture_to_temp("minimal");
        let temp2 = copy_fixture_to_temp("with_config");

        let mut shiori = PastaShiori::default();

        // First load
        let result1 = shiori.load(0, temp1.path().as_os_str());
        assert!(result1.is_ok());
        assert!(result1.unwrap());
        assert!(shiori.runtime.is_some());

        let first_load_dir = shiori.load_dir.clone();
        assert!(first_load_dir.is_some());

        // Second load - should release previous runtime
        let result2 = shiori.load(0, temp2.path().as_os_str());
        assert!(result2.is_ok());
        assert!(result2.unwrap());
        assert!(shiori.runtime.is_some());

        // load_dir should be updated to new path
        let second_load_dir = shiori.load_dir.clone();
        assert!(second_load_dir.is_some());
        assert_ne!(first_load_dir.unwrap(), second_load_dir.unwrap());
    }

    #[test]
    fn test_reload_same_directory() {
        let temp = copy_fixture_to_temp("minimal");

        let mut shiori = PastaShiori::default();

        // First load
        let result1 = shiori.load(0, temp.path().as_os_str());
        assert!(result1.unwrap());

        // Reload same directory - should work without error
        let result2 = shiori.load(0, temp.path().as_os_str());
        assert!(result2.unwrap());
        assert!(shiori.runtime.is_some());
    }

    // ========================================================================
    // Task 12: E2Eテスト - SHIORI load → request → unload サイクル
    // ========================================================================

    #[test]
    fn test_full_shiori_lifecycle() {
        let temp = copy_fixture_to_temp("minimal");

        let mut shiori = PastaShiori::default();

        // Phase 1: load()
        let load_result = shiori.load(42, temp.path().as_os_str());
        assert!(load_result.is_ok());
        assert!(load_result.unwrap());
        assert!(shiori.runtime.is_some());
        assert_eq!(shiori.hinst, 42);

        // Phase 2: request() - main.lua should provide SHIORI.request
        // Use valid SHIORI request format (GET method required for parsing)
        let valid_request = "GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n";
        let request_result = shiori.request(valid_request);
        assert!(request_result.is_ok());
        let response = request_result.unwrap();
        // Should return 204 No Content from main.lua (minimal fixture has no SHIORI.request)
        assert!(response.contains("SHIORI/3.0 204 No Content"));
        assert!(response.contains("Charset: UTF-8"));
        assert!(response.contains("Sender: Pasta"));

        // Phase 3: unload via drop
        drop(shiori);
        // If we get here without panic, cleanup was successful
    }

    #[test]
    fn test_request_before_load_returns_error() {
        let mut shiori = PastaShiori::default();

        // Request before load should fail
        let result = shiori.request("test");
        assert!(result.is_err());
        match result.unwrap_err() {
            MyError::NotInitialized => {}
            _ => panic!("Expected NotInitialized error"),
        }
    }

    #[test]
    fn test_load_nonexistent_directory_returns_false() {
        let mut shiori = PastaShiori::default();
        let nonexistent = PathBuf::from("/definitely/nonexistent/path");

        let result = shiori.load(0, nonexistent.as_os_str());
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false, not error
        assert!(shiori.runtime.is_none());
    }

    // ========================================================================
    // Task 13: 複数インスタンス同時ロードテスト
    // ========================================================================

    #[test]
    fn test_multiple_instances_independent() {
        let temp1 = copy_fixture_to_temp("minimal");
        let temp2 = copy_fixture_to_temp("with_config");

        let mut shiori1 = PastaShiori::default();
        let mut shiori2 = PastaShiori::default();

        // Load both instances
        assert!(shiori1.load(1, temp1.path().as_os_str()).unwrap());
        assert!(shiori2.load(2, temp2.path().as_os_str()).unwrap());

        // Both should have independent runtimes
        assert!(shiori1.runtime.is_some());
        assert!(shiori2.runtime.is_some());

        // Both should respond to requests with 204 No Content
        // Use valid SHIORI request format for parsing
        let valid_request = "GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n";
        let response1 = shiori1.request(valid_request).unwrap();
        let response2 = shiori2.request(valid_request).unwrap();

        assert!(response1.contains("SHIORI/3.0 204 No Content"));
        assert!(response2.contains("SHIORI/3.0 204 No Content"));

        // Different hinst values
        assert_eq!(shiori1.hinst, 1);
        assert_eq!(shiori2.hinst, 2);

        // Different load_dirs
        assert_ne!(shiori1.load_dir, shiori2.load_dir);
    }

    #[test]
    fn test_multiple_instances_share_global_registry() {
        let temp1 = copy_fixture_to_temp("minimal");
        let temp2 = copy_fixture_to_temp("with_config");

        let mut shiori1 = PastaShiori::default();
        let mut shiori2 = PastaShiori::default();

        // Load both instances
        shiori1.load(1, temp1.path().as_os_str()).unwrap();
        shiori2.load(2, temp2.path().as_os_str()).unwrap();

        // GlobalLoggerRegistry should have both loggers registered
        // (we can't directly test this without exposing internals,
        // but the fact that both load successfully indicates it works)

        // Cleanup - drop should unregister from GlobalLoggerRegistry
        drop(shiori1);
        drop(shiori2);
    }

    // ========================================================================
    // Task 7.1: PastaShiori::load テスト - SHIORI 関数フラグ検証
    // ========================================================================

    #[test]
    fn test_load_sets_shiori_flags_when_main_lua_exists() {
        let temp = copy_fixture_to_temp("minimal");

        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // main.lua is present in scripts/, so cached functions should be Some
        assert!(shiori.load_fn.is_some(), "load_fn should be cached");
        assert!(shiori.request_fn.is_some(), "request_fn should be cached");
    }

    #[test]
    fn test_load_flags_false_without_main_lua() {
        let temp = copy_fixture_to_temp("minimal");

        // Remove the main.lua file to simulate missing SHIORI functions
        let main_lua_path = temp.path().join("scripts/pasta/shiori/main.lua");
        if main_lua_path.exists() {
            std::fs::remove_file(&main_lua_path).unwrap();
        }

        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // main.lua doesn't exist, so cached functions should be None
        assert!(
            shiori.load_fn.is_none(),
            "load_fn should be None without main.lua"
        );
        assert!(
            shiori.request_fn.is_none(),
            "request_fn should be None without main.lua"
        );
    }

    // ========================================================================
    // Task 7.2: PastaShiori::request テスト
    // ========================================================================

    #[test]
    fn test_request_returns_204_from_lua() {
        let temp = copy_fixture_to_temp("minimal");

        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // Use valid SHIORI request format for parsing
        let valid_request = "GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n";
        let response = shiori.request(valid_request).unwrap();

        // Verify SHIORI/3.0 response format
        assert!(response.starts_with("SHIORI/3.0 204 No Content\r\n"));
        assert!(response.contains("Charset: UTF-8\r\n"));
        assert!(response.contains("Sender: Pasta\r\n"));
        assert!(response.ends_with("\r\n\r\n"));
    }

    #[test]
    fn test_request_returns_default_204_without_main_lua() {
        let temp = copy_fixture_to_temp("minimal");

        // Remove main.lua to test fallback behavior
        let main_lua_path = temp.path().join("scripts/pasta/shiori/main.lua");
        if main_lua_path.exists() {
            std::fs::remove_file(&main_lua_path).unwrap();
        }

        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // Should still get 204 response (default fallback)
        let response = shiori.request("test").unwrap();
        assert!(response.contains("SHIORI/3.0 204 No Content"));
        assert!(response.contains("Charset: UTF-8"));
        assert!(response.contains("Sender: Pasta"));
    }

    #[test]
    fn test_request_not_initialized_error() {
        let mut shiori = PastaShiori::default();

        // Request before load should return NotInitialized error
        let result = shiori.request("test");
        assert!(result.is_err());
        match result.unwrap_err() {
            MyError::NotInitialized => {}
            e => panic!("Expected NotInitialized, got {:?}", e),
        }
    }

    // ========================================================================
    // Task 7.1: unload 呼び出しの検証テスト
    // ========================================================================

    #[test]
    fn test_unload_called_on_drop() {
        let temp = copy_fixture_to_temp("minimal");

        // Create a Lua script that defines SHIORI.unload and sets a global flag
        let main_lua_path = temp.path().join("scripts/pasta/shiori/main.lua");
        std::fs::write(
            &main_lua_path,
            r#"
SHIORI = {}

-- Track if unload was called via a file marker
local unload_marker_path = nil

function SHIORI.load(hinst, load_dir)
    unload_marker_path = load_dir .. "/unload_called.marker"
    return true
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end

function SHIORI.unload()
    -- Write a marker file to indicate unload was called
    if unload_marker_path then
        local f = io.open(unload_marker_path, "w")
        if f then
            f:write("unloaded")
            f:close()
        end
    end
end
"#,
        )
        .unwrap();

        let marker_path = temp.path().join("unload_called.marker");

        // Ensure marker doesn't exist before test
        if marker_path.exists() {
            std::fs::remove_file(&marker_path).unwrap();
        }

        {
            let mut shiori = PastaShiori::default();
            assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

            // Verify unload_fn is cached
            assert!(shiori.unload_fn.is_some(), "unload_fn should be cached");

            // shiori will be dropped here
        }

        // After drop, the marker file should exist
        assert!(
            marker_path.exists(),
            "SHIORI.unload should have created the marker file on drop"
        );
    }

    // ========================================================================
    // Task 7.2: unload エラー耐性テスト
    // ========================================================================

    #[test]
    fn test_unload_error_does_not_panic() {
        let temp = copy_fixture_to_temp("minimal");

        // Create a Lua script with an unload function that always errors
        let main_lua_path = temp.path().join("scripts/pasta/shiori/main.lua");
        std::fs::write(
            &main_lua_path,
            r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return true
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end

function SHIORI.unload()
    error("Intentional unload error for testing")
end
"#,
        )
        .unwrap();

        {
            let mut shiori = PastaShiori::default();
            assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

            // Verify unload_fn is cached
            assert!(shiori.unload_fn.is_some(), "unload_fn should be cached");

            // shiori will be dropped here - should NOT panic even with error in unload
        }

        // If we reach here, the test passed (no panic occurred)
    }

    // ========================================================================
    // Task 7.3: reload 時のキャッシュクリアテスト
    // ========================================================================

    #[test]
    fn test_cached_functions_cleared_on_reload() {
        let temp1 = copy_fixture_to_temp("minimal");
        let temp2 = copy_fixture_to_temp("minimal");

        // Modify temp2's main.lua to remove SHIORI.load but keep request
        let main_lua_path2 = temp2.path().join("scripts/pasta/shiori/main.lua");
        std::fs::write(
            &main_lua_path2,
            r#"
SHIORI = {}

-- No SHIORI.load function defined

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end
"#,
        )
        .unwrap();

        let mut shiori = PastaShiori::default();

        // First load - should have both load and request functions
        assert!(shiori.load(0, temp1.path().as_os_str()).unwrap());
        assert!(
            shiori.load_fn.is_some(),
            "First load: load_fn should be cached"
        );
        assert!(
            shiori.request_fn.is_some(),
            "First load: request_fn should be cached"
        );

        // Second load (reload) - should only have request function
        assert!(shiori.load(0, temp2.path().as_os_str()).unwrap());
        assert!(
            shiori.load_fn.is_none(),
            "After reload: load_fn should be None (not defined in temp2)"
        );
        assert!(
            shiori.request_fn.is_some(),
            "After reload: request_fn should still be cached"
        );
    }

    // ========================================================================
    // Task 7.4: 複数インスタンス独立性テスト
    // ========================================================================

    #[test]
    fn test_multiple_instances_independent_caches() {
        let temp1 = copy_fixture_to_temp("minimal");
        let temp2 = copy_fixture_to_temp("minimal");

        // Modify temp1's main.lua to define all three SHIORI functions
        let main_lua_path1 = temp1.path().join("scripts/pasta/shiori/main.lua");
        std::fs::write(
            &main_lua_path1,
            r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return true
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end

function SHIORI.unload()
    -- Instance 1 unload
end
"#,
        )
        .unwrap();

        // Modify temp2's main.lua to only define request (no load/unload)
        let main_lua_path2 = temp2.path().join("scripts/pasta/shiori/main.lua");
        std::fs::write(
            &main_lua_path2,
            r#"
SHIORI = {}

-- No SHIORI.load or unload

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end
"#,
        )
        .unwrap();

        let mut shiori1 = PastaShiori::default();
        let mut shiori2 = PastaShiori::default();

        // Load both instances
        assert!(shiori1.load(1, temp1.path().as_os_str()).unwrap());
        assert!(shiori2.load(2, temp2.path().as_os_str()).unwrap());

        // Instance 1 should have all functions cached
        assert!(shiori1.load_fn.is_some(), "shiori1.load_fn should be Some");
        assert!(
            shiori1.request_fn.is_some(),
            "shiori1.request_fn should be Some"
        );
        assert!(
            shiori1.unload_fn.is_some(),
            "shiori1.unload_fn should be Some"
        );

        // Instance 2 should only have request_fn cached
        assert!(shiori2.load_fn.is_none(), "shiori2.load_fn should be None");
        assert!(
            shiori2.request_fn.is_some(),
            "shiori2.request_fn should be Some"
        );
        assert!(
            shiori2.unload_fn.is_none(),
            "shiori2.unload_fn should be None"
        );

        // Modifying one instance's cache should not affect the other
        // (This is implicitly verified by the above assertions)
    }

    // ========================================================================
    // Task 1.1: 400 Bad Requestレスポンス生成機能テスト
    // ========================================================================

    #[test]
    fn test_default_400_response_format() {
        let response = PastaShiori::default_400_response();

        // SHIORI/3.0プロトコル準拠を検証
        assert!(response.starts_with("SHIORI/3.0 400 Bad Request\r\n"));
        assert!(response.contains("Charset: UTF-8\r\n"));
        assert!(response.contains("Sender: Pasta\r\n"));
        assert!(response.ends_with("\r\n\r\n"));
    }

    // ========================================================================
    // Task 4.1: パース成功・失敗パステスト
    // ========================================================================

    #[test]
    fn test_request_with_valid_shiori_request() {
        // 有効なSHIORIリクエスト形式でのテスト
        let temp = copy_shiori_lifecycle_fixture();
        let mut shiori = PastaShiori::default();

        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // 有効なSHIORI/3.0リクエスト
        let valid_request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            Reference0: first\r\n\
            \r\n";

        let result = shiori.request(valid_request);
        assert!(result.is_ok());
        let response = result.unwrap();
        // パースが成功してLuaが呼ばれた証拠として、400 Bad Request以外を期待
        // (シーンが見つからない場合は500、見つかった場合は200が返る)
        assert!(
            !response.contains("SHIORI/3.0 400 Bad Request"),
            "Parse should have succeeded, but got 400 Bad Request: {}",
            response
        );
        // 有効なSHIORIレスポンス形式であることを確認
        assert!(
            response.starts_with("SHIORI/3.0"),
            "Expected valid SHIORI response format, got: {}",
            response
        );
    }

    #[test]
    fn test_request_with_invalid_shiori_request_returns_400() {
        // 無効なSHIORIリクエスト形式でのテスト
        let temp = copy_shiori_lifecycle_fixture();
        let mut shiori = PastaShiori::default();

        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // 完全に無効なリクエスト（パース失敗を引き起こす）
        let invalid_request = "THIS IS NOT A VALID SHIORI REQUEST";

        let result = shiori.request(invalid_request);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(
            response.contains("SHIORI/3.0 400 Bad Request"),
            "Expected 400 Bad Request, got: {}",
            response
        );
    }

    /// Test that parsed request table fields are correctly passed to Lua.
    /// This verifies Lua can actually read method, version, id, reference, dic, etc.
    #[test]
    fn test_request_parsed_table_fields_accessible_in_lua() {
        let temp = copy_shiori_lifecycle_fixture();

        // Override main.lua to echo back req table fields for verification
        let main_lua_path = temp.path().join("scripts/pasta/shiori/main.lua");
        std::fs::write(
            &main_lua_path,
            r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return true
end

--- Echo back req table fields for verification
function SHIORI.request(req)
    -- Verify req is a table
    if type(req) ~= "table" then
        return "SHIORI/3.0 500 Internal Server Error\r\nValue: req is not a table\r\n\r\n"
    end

    -- Extract all expected fields from req table
    local method = req.method or "NIL"
    local version = req.version or "NIL"
    local id = req.id or "NIL"
    local charset = req.charset or "NIL"
    local sender = req.sender or "NIL"

    -- Extract reference array
    local ref0 = "NIL"
    local ref1 = "NIL"
    if req.reference then
        ref0 = req.reference[0] or "NIL"
        ref1 = req.reference[1] or "NIL"
    end

    -- Extract dic table entry
    local dic_id = "NIL"
    if req.dic then
        dic_id = req.dic["ID"] or "NIL"
    end

    -- Return all fields in response Value header for verification
    local fields = string.format(
        "method=%s,version=%s,id=%s,charset=%s,sender=%s,ref0=%s,ref1=%s,dic_id=%s",
        tostring(method), tostring(version), tostring(id),
        tostring(charset), tostring(sender),
        tostring(ref0), tostring(ref1), tostring(dic_id)
    )

    return "SHIORI/3.0 200 OK\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Value: " .. fields .. "\r\n" ..
        "\r\n"
end

function SHIORI.unload()
end
"#,
        )
        .unwrap();

        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // Send a request with all fields populated
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            Sender: SSP\r\n\
            ID: OnBoot\r\n\
            Reference0: ref_value_0\r\n\
            Reference1: ref_value_1\r\n\
            \r\n";

        let response = shiori.request(request).unwrap();

        // Verify response is 200 OK (not 400 Bad Request)
        assert!(
            response.contains("SHIORI/3.0 200 OK"),
            "Expected 200 OK, got: {}",
            response
        );

        // Verify each field was correctly parsed and accessible in Lua
        assert!(
            response.contains("method=get"),
            "Expected method=get in response: {}",
            response
        );
        assert!(
            response.contains("version=30"),
            "Expected version=30 in response: {}",
            response
        );
        assert!(
            response.contains("id=OnBoot"),
            "Expected id=OnBoot in response: {}",
            response
        );
        assert!(
            response.contains("charset=UTF-8"),
            "Expected charset=UTF-8 in response: {}",
            response
        );
        assert!(
            response.contains("sender=SSP"),
            "Expected sender=SSP in response: {}",
            response
        );
        assert!(
            response.contains("ref0=ref_value_0"),
            "Expected ref0=ref_value_0 in response: {}",
            response
        );
        assert!(
            response.contains("ref1=ref_value_1"),
            "Expected ref1=ref_value_1 in response: {}",
            response
        );
        assert!(
            response.contains("dic_id=OnBoot"),
            "Expected dic_id=OnBoot in response: {}",
            response
        );
    }

    /// Copy shiori_lifecycle fixture to a temporary directory.
    fn copy_shiori_lifecycle_fixture() -> TempDir {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shiori_lifecycle");
        let temp = TempDir::new().unwrap();
        copy_dir_recursive(&src, temp.path()).unwrap();

        // Copy scripts directory from pasta_lua
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("pasta_lua");
        let scripts_src = crate_root.join("scripts");
        let scripts_dst = temp.path().join("scripts");
        if scripts_src.exists() {
            // Merge: copy pasta_lua scripts first, then overlay fixture scripts
            std::fs::create_dir_all(&scripts_dst).unwrap();
            copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
        }

        // Copy scriptlibs directory
        let scriptlibs_src = crate_root.join("scriptlibs");
        let scriptlibs_dst = temp.path().join("scriptlibs");
        if scriptlibs_src.exists() {
            std::fs::create_dir_all(&scriptlibs_dst).unwrap();
            copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
        }

        // Copy fixture's scripts on top (overwrite)
        let fixture_scripts = src.join("scripts");
        if fixture_scripts.exists() {
            copy_dir_recursive(&fixture_scripts, &scripts_dst).unwrap();
        }

        temp
    }
}
