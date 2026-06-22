//! Pasta Lua - Lua integration for Pasta DSL
//!
//! This crate provides a Lua transpiler for Pasta DSL, converting Pasta AST
//! to Lua source code. It follows the same architecture as pasta_rune but
//! targets Lua 5.3+ instead of Rune VM.
//!
//! # Architecture
//!
//! - `LuaTranspiler`: Main transpiler interface
//! - `LuaCodeGenerator`: Generates Lua code from AST nodes
//! - `StringLiteralizer`: Converts strings to optimal Lua literal format
//! - `TranspileContext`: Manages state during transpilation
//! - `TranspilerConfig`: Configuration options
//! - `PastaLuaRuntime`: Lua VM host with pasta module integration
//!
//! # Example
//!
//! ```rust,ignore
//! use pasta_lua::{LuaTranspiler, TranspilerConfig, PastaLuaRuntime};
//!
//! let transpiler = LuaTranspiler::default();
//! let mut output = Vec::new();
//!
//! let context = transpiler.transpile(&actors, &scenes, &mut output)?;
//! let lua_code = String::from_utf8(output)?;
//!
//! // Create runtime with the context
//! let runtime = PastaLuaRuntime::new(context)?;
//! runtime.exec(&lua_code)?;
//! ```

#[cfg(feature = "actor-poc")]
pub mod actor_poc;
pub mod code_gen;
pub mod config;
pub mod context;
pub mod debug;
pub mod encoding;
pub mod error;
pub mod loader;
pub mod logging;
pub mod normalize;
pub mod presentation;
pub mod runtime;
pub mod sakura_script;
pub mod search;
pub mod string_literalizer;
pub mod transpiler;

// Re-export main types
pub use code_gen::LuaCodeGenerator;
pub use config::{LineEnding, TranspilerConfig};
pub use context::TranspileContext;
pub use debug::{DebugConfig, DebugError, DebugHandle, SourceMode, enable as enable_debug};
pub use encoding::{Encoder, Encoding};
pub use error::{ConfigError, TranspileError};
pub use loader::{
    CacheManager, DebugFileConfig, LoaderConfig, LoaderContext, LoaderError, LoggingConfig,
    LuaConfig, PastaConfig, PastaLoader, TalkConfig, TranspileFailure, TranspileResult,
    default_debug_port, default_libs,
};
pub use logging::{
    GlobalLoggerRegistry, LoadDirGuard, PastaLogger, get_current_load_dir,
    init_tracing_with_reload, set_current_load_dir, update_tracing_filter,
};
pub use runtime::{PastaLuaRuntime, RuntimeConfig};
pub use search::{SearchContext, SearchError};
pub use string_literalizer::StringLiteralizer;

// Re-export mlua types needed by pasta_shiori
pub use mlua;
pub use transpiler::LuaTranspiler;
