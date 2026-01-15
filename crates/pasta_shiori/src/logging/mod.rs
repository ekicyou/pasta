//! Logging infrastructure for pasta_shiori.
//!
//! Re-exports logging components from pasta_lua for convenience.
//! All logging functionality is implemented in pasta_lua to enable
//! sharing across different applications (SHIORI DLLs, CLI tools, etc.).

// Re-export all logging components from pasta_lua
pub use pasta_lua::{
    GlobalLoggerRegistry, LoadDirGuard, LoggingConfig, PastaLogger, get_current_load_dir,
    set_current_load_dir,
};
