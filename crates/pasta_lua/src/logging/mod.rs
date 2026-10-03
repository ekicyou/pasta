//! Logging module for pasta_lua.
//!
//! Provides instance-specific logging to a fixed-name file (no rotation)
//! and a global registry for multi-instance log routing.
//!
//! # Components
//!
//! - `PastaLogger` - Instance-specific file logger (fixed file name, never rotated)
//! - `GlobalLoggerRegistry` - Singleton registry for log routing
//! - `LoadDirGuard` - RAII guard for setting log context

mod logger;
mod registry;
mod tracing_init;

pub use logger::PastaLogger;
pub use registry::{
    GlobalLoggerRegistry, LoadDirGuard, RoutingWriter, get_current_load_dir, set_current_load_dir,
};
pub use tracing_init::{init_tracing_with_reload, update_tracing_filter};
