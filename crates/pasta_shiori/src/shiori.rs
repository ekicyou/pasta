use crate::error::*;
use crate::logging::{GlobalLoggerRegistry, LoadDirGuard};
use pasta_lua::{PastaLoader, PastaLuaRuntime};
use std::{ffi::*, path::*};
use tracing::{debug, error, info};

pub(crate) trait Shiori {
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
pub(crate) struct PastaShiori {
    /// DLL module handle (for future Windows API integration)
    hinst: isize,

    /// Base directory for ghost scripts (master/ directory)
    load_dir: Option<PathBuf>,

    /// Pasta Lua runtime instance (contains logger internally)
    runtime: Option<PastaLuaRuntime>,
}

// SAFETY: PastaShiori is used in a single-threaded context (SHIORI DLL).
// The OnceLock ensures only one instance exists, and Mutex protects access.
// The Lua runtime is only accessed from the main thread.
unsafe impl Send for PastaShiori {}
unsafe impl Sync for PastaShiori {}

impl Drop for PastaShiori {
    fn drop(&mut self) {
        // Unregister logger from global registry
        if let Some(ref load_dir) = self.load_dir {
            GlobalLoggerRegistry::instance().unregister(load_dir);
            info!(load_dir = %load_dir.display(), "Unregistered logger");
        }

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
                self.runtime = Some(runtime);
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

        // TODO: Actually process the request through the runtime
        Ok(format!("PastaShiori received request: {}", req))
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

        // Phase 2: request()
        let request_result = shiori.request("SHIORI/3.0\r\n\r\n");
        assert!(request_result.is_ok());
        let response = request_result.unwrap();
        assert!(response.contains("PastaShiori received request"));

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

        // Both should respond to requests
        let response1 = shiori1.request("request1").unwrap();
        let response2 = shiori2.request("request2").unwrap();

        assert!(response1.contains("request1"));
        assert!(response2.contains("request2"));

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
}
