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
        let mut loggers = self.loggers.lock().unwrap();
        loggers.insert(load_dir, logger);
    }

    /// Unregister the logger for the given load_dir.
    pub fn unregister(&self, load_dir: &Path) {
        let mut loggers = self.loggers.lock().unwrap();
        loggers.remove(load_dir);
    }

    /// Get the logger for the given load_dir.
    pub fn get(&self, load_dir: &Path) -> Option<Arc<PastaLogger>> {
        let loggers = self.loggers.lock().unwrap();
        loggers.get(load_dir).cloned()
    }

    /// Get the number of registered loggers (for testing).
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.loggers.lock().unwrap().len()
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
}
