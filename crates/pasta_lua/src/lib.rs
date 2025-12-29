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
//!
//! # Example
//!
//! ```rust,ignore
//! use pasta_lua::{LuaTranspiler, TranspilerConfig};
//!
//! let transpiler = LuaTranspiler::default();
//! let mut output = Vec::new();
//!
//! transpiler.transpile(&actors, &scenes, &mut output)?;
//! let lua_code = String::from_utf8(output)?;
//! ```

pub mod code_generator;
pub mod config;
pub mod context;
pub mod error;
pub mod string_literalizer;
pub mod transpiler;

// Re-export main types
pub use code_generator::LuaCodeGenerator;
pub use config::TranspilerConfig;
pub use context::TranspileContext;
pub use error::TranspileError;
pub use string_literalizer::StringLiteralizer;
pub use transpiler::LuaTranspiler;
