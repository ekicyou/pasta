//! Logging module for pasta_lua.
//!
//! Provides instance-specific logging with file rotation and
//! a global registry for multi-instance log routing.
//!
//! # Components
//!
//! - `PastaLogger` - Instance-specific file logger with rotation
//! - `GlobalLoggerRegistry` - Singleton registry for log routing
//! - `LoadDirGuard` - RAII guard for setting log context

mod logger;
mod registry;

pub use logger::PastaLogger;
pub use registry::{
    GlobalLoggerRegistry, LoadDirGuard, RoutingWriter, get_current_load_dir, set_current_load_dir,
};
