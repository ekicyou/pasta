//! Pasta DSL - Independent DSL parser and AST definitions.
//!
//! This crate provides a standalone parser for the Pasta DSL (Domain Specific Language),
//! enabling DSL parsing without depending on any specific backend (Lua, etc.) or runtime.
//!
//! # Features
//!
//! - **PEG-based parser**: Uses Pest 2.8 for grammar-driven parsing
//! - **Complete AST**: Full AST type definitions for all Pasta DSL constructs
//! - **Independent error types**: `ParseError` and `ParseErrorInfo` for parse diagnostics
//! - **Zero backend dependency**: No Lua, registry, or runtime dependencies
//!
//! # Example
//!
//! ```no_run
//! use pasta_dsl::parser::{parse_str, FileItem, PastaFile};
//!
//! let source = "＊挨拶\n  Alice：こんにちは\n";
//! let ast = parse_str(source, "test.pasta").unwrap();
//!
//! let scene_count = ast.items.iter()
//!     .filter(|i| matches!(i, FileItem::GlobalSceneScope(_)))
//!     .count();
//! println!("Parsed {} global scenes", scene_count);
//! ```

pub mod error;
pub mod parser;
pub mod partial;

// Convenience re-exports
pub use error::{ParseError, ParseErrorInfo, ParseResult};
pub use parser::{FileItem, PastaFile, parse_file, parse_str};
pub use partial::{PartialParseError, PartialParseResult, parse_str_partial};
