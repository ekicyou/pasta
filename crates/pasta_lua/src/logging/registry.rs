//! Global Logger Registry - Manages multiple PastaLogger instances.
//!
//! This module provides a singleton registry for PastaLogger instances,
//! allowing log routing based on the current load_dir context.
//!
//! # Usage
//!
//! Applications using pasta_lua (SHIORI DLLs, CLI tools, etc.) can register
//! their PastaLogger instances with this registry for centralized log routing.
//!
//! ```rust,ignore
//! use pasta_lua::logging::{GlobalLoggerRegistry, LoadDirGuard, PastaLogger};
//!
//! // Register a logger for a specific load_dir
//! let logger = Arc::new(PastaLogger::new(load_dir, config)?);
//! GlobalLoggerRegistry::instance().register(load_dir.clone(), logger);
//!
//! // Set context for log routing
//! let _guard = LoadDirGuard::new(load_dir);
//! tracing::info!("This goes to the registered logger");
//! ```

use super::PastaLogger;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use tracing_subscriber::fmt::MakeWriter;

/// Global singleton instance of the logger registry.
static REGISTRY: OnceLock<GlobalLoggerRegistry> = OnceLock::new();

/// Global Logger Registry - Manages multiple PastaLogger instances.
///
/// Each application instance registers its PastaLogger with load_dir as key.
/// The registry routes log output based on the current thread's load_dir context.
///
/// This is useful for:
/// - SHIORI DLLs with multiple ghost instances
/// - CLI tools managing multiple projects
/// - Any application needing instance-specific logging
#[derive(Clone)]
pub struct GlobalLoggerRegistry {
    /// Map of load_dir -> PastaLogger
    loggers: Arc<Mutex<HashMap<PathBuf, Arc<PastaLogger>>>>,
}

impl GlobalLoggerRegistry {
    /// Create a new empty registry.
    fn new() -> Self {
        Self {
            loggers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the singleton instance.
    pub fn instance() -> &'static Self {
        REGISTRY.get_or_init(GlobalLoggerRegistry::new)
    }

    /// Register a logger for the given load_dir.
    ///
    /// If a logger already exists for the load_dir, it is replaced.
    pub fn register(&self, load_dir: PathBuf, logger: Arc<PastaLogger>) {
        let mut loggers = self
            .loggers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loggers.insert(load_dir, logger);
    }

    /// Unregister the logger for the given load_dir.
    pub fn unregister(&self, load_dir: &Path) {
        let mut loggers = self
            .loggers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loggers.remove(load_dir);
    }

    /// Get the logger for the given load_dir.
    pub fn get(&self, load_dir: &Path) -> Option<Arc<PastaLogger>> {
        let loggers = self
            .loggers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loggers.get(load_dir).cloned()
    }
}

/// Writer that routes to the appropriate PastaLogger based on current context.
///
/// Uses thread-local storage to determine which logger to use.
pub struct RoutingWriter {
    /// The logger to write to, or None for no-op.
    logger: Option<Arc<PastaLogger>>,
}

impl Write for RoutingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(ref logger) = self.logger {
            logger.write(buf)
        } else {
            // No-op: silently discard
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref logger) = self.logger {
            logger.flush()
        } else {
            Ok(())
        }
    }
}

impl<'a> MakeWriter<'a> for GlobalLoggerRegistry {
    type Writer = RoutingWriter;

    fn make_writer(&'a self) -> Self::Writer {
        // Get the current load_dir from thread-local context
        let load_dir = CURRENT_LOAD_DIR.with(|cell| cell.borrow().clone());

        let logger = load_dir.and_then(|path| self.get(&path));

        RoutingWriter { logger }
    }
}

// Thread-local storage for the current load_dir context.
//
// Set this before logging to route logs to the correct file.
thread_local! {
    static CURRENT_LOAD_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Set the current load_dir context for logging.
///
/// All log messages in this thread will be routed to the logger
/// registered for this load_dir.
pub fn set_current_load_dir(load_dir: Option<PathBuf>) {
    CURRENT_LOAD_DIR.with(|cell| {
        *cell.borrow_mut() = load_dir;
    });
}

/// Get the current load_dir context.
pub fn get_current_load_dir() -> Option<PathBuf> {
    CURRENT_LOAD_DIR.with(|cell| cell.borrow().clone())
}

/// Guard that sets the load_dir context and restores it on drop.
///
/// Use this to ensure proper context cleanup in scoped operations.
pub struct LoadDirGuard {
    previous: Option<PathBuf>,
}

impl LoadDirGuard {
    /// Create a new guard that sets the load_dir context.
    pub fn new(load_dir: PathBuf) -> Self {
        let previous = get_current_load_dir();
        set_current_load_dir(Some(load_dir));
        Self { previous }
    }
}

impl Drop for LoadDirGuard {
    fn drop(&mut self) {
        set_current_load_dir(self.previous.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_singleton() {
        let r1 = GlobalLoggerRegistry::instance();
        let r2 = GlobalLoggerRegistry::instance();
        // Same address
        assert!(std::ptr::eq(r1, r2));
    }

    #[test]
    fn test_load_dir_context() {
        // Initially None
        assert!(get_current_load_dir().is_none());

        // Set and get
        set_current_load_dir(Some(PathBuf::from("/test/path")));
        assert_eq!(get_current_load_dir(), Some(PathBuf::from("/test/path")));

        // Clear
        set_current_load_dir(None);
        assert!(get_current_load_dir().is_none());
    }

    #[test]
    fn test_load_dir_guard() {
        set_current_load_dir(Some(PathBuf::from("/original")));

        {
            let _guard = LoadDirGuard::new(PathBuf::from("/guarded"));
            assert_eq!(get_current_load_dir(), Some(PathBuf::from("/guarded")));
        }

        // Restored after guard drops
        assert_eq!(get_current_load_dir(), Some(PathBuf::from("/original")));

        // Cleanup
        set_current_load_dir(None);
    }

    #[test]
    fn test_load_dir_guard_nested() {
        // Nested guards must restore the previous context level by level.
        set_current_load_dir(None);

        {
            let _outer = LoadDirGuard::new(PathBuf::from("/outer"));
            assert_eq!(get_current_load_dir(), Some(PathBuf::from("/outer")));

            {
                let _inner = LoadDirGuard::new(PathBuf::from("/inner"));
                assert_eq!(get_current_load_dir(), Some(PathBuf::from("/inner")));
            }

            // Inner guard restores outer context
            assert_eq!(get_current_load_dir(), Some(PathBuf::from("/outer")));
        }

        // Outer guard restores the original (None) context
        assert!(get_current_load_dir().is_none());
    }

    #[test]
    fn test_registry_register_get_unregister() {
        // The registry is a process-wide singleton shared with other tests,
        // so a unique temp dir is used as the key.
        let temp_dir = tempfile::TempDir::new().unwrap();
        let load_dir = temp_dir.path().to_path_buf();
        let registry = GlobalLoggerRegistry::instance();

        // Not registered yet
        assert!(registry.get(&load_dir).is_none());

        let logger = Arc::new(PastaLogger::new(&load_dir, None).unwrap());
        registry.register(load_dir.clone(), logger.clone());

        let fetched = registry.get(&load_dir).expect("logger should be registered");
        assert_eq!(fetched.log_path(), logger.log_path());

        registry.unregister(&load_dir);
        assert!(registry.get(&load_dir).is_none());
    }

    #[test]
    fn test_registry_register_replaces_existing() {
        let temp_a = tempfile::TempDir::new().unwrap();
        let temp_b = tempfile::TempDir::new().unwrap();
        let key = temp_a.path().to_path_buf();
        let registry = GlobalLoggerRegistry::instance();

        // Two loggers with distinct log paths, registered under the same key.
        let first = Arc::new(PastaLogger::new(temp_a.path(), None).unwrap());
        let second = Arc::new(PastaLogger::new(temp_b.path(), None).unwrap());

        registry.register(key.clone(), first.clone());
        registry.register(key.clone(), second.clone());

        let fetched = registry.get(&key).expect("logger should be registered");
        assert_eq!(
            fetched.log_path(),
            second.log_path(),
            "second registration should replace the first"
        );

        registry.unregister(&key);
    }

    #[test]
    fn test_make_writer_without_context_is_noop() {
        // Without a load_dir context, the routing writer silently discards
        // output but still reports success.
        set_current_load_dir(None);
        let registry = GlobalLoggerRegistry::instance();

        let mut writer = registry.make_writer();
        let n = writer.write(b"discarded").unwrap();
        assert_eq!(n, b"discarded".len());
        writer.flush().unwrap();
    }

    #[test]
    fn test_make_writer_with_unregistered_dir_is_noop() {
        // Context is set but no logger is registered for it: no-op success.
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _guard = LoadDirGuard::new(temp_dir.path().to_path_buf());
        let registry = GlobalLoggerRegistry::instance();

        let mut writer = registry.make_writer();
        let n = writer.write(b"discarded").unwrap();
        assert_eq!(n, b"discarded".len());
        writer.flush().unwrap();
    }

    #[test]
    fn test_make_writer_routes_to_registered_logger() {
        // End-to-end: register a logger, set the context, write through the
        // MakeWriter interface, then verify the bytes reached the log file.
        let temp_dir = tempfile::TempDir::new().unwrap();
        let load_dir = temp_dir.path().to_path_buf();
        let registry = GlobalLoggerRegistry::instance();

        let logger = Arc::new(PastaLogger::new(&load_dir, None).unwrap());
        let log_path = logger.log_path().to_path_buf();
        registry.register(load_dir.clone(), logger);

        {
            let _guard = LoadDirGuard::new(load_dir.clone());
            let mut writer = registry.make_writer();
            writer.write_all(b"routed via registry\n").unwrap();
            writer.flush().unwrap();
        }

        // Drop all Arc references so the worker guard flushes on drop.
        registry.unregister(&load_dir);

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(
            content.contains("routed via registry"),
            "log file should contain the routed line, got: {:?}",
            content
        );
    }
}
