//! Pasta Core - Language-independent DSL parsing and registry layer.
//!
//! This crate provides the core parsing and registry functionality for the Pasta DSL.
//! It is designed to be language-independent, allowing different language backends
//! (Rune, Lua, etc.) to use the same parsing infrastructure.
//!
//! # Modules
//!
//! - `parser`: Pasta DSL parsing (pest-based PEG grammar)
//! - `registry`: Scene and word registration (Pass 1 + Runtime tables)
//! - `error`: Parse-related error types
//!
//! # Example
//!
//! ```no_run
//! use pasta_core::parser::{parse_str, FileItem, PastaFile};
//! use pasta_core::registry::{SceneRegistry, WordDefRegistry};
//!
//! let source = "＊挨拶\n  Alice：こんにちは\n";
//! let ast = parse_str(source, "test.pasta").unwrap();
//!
//! let scene_count = ast.items.iter().filter(|i| matches!(i, FileItem::GlobalSceneScope(_))).count();
//! println!("Parsed {} global scenes", scene_count);
//! ```

pub mod error;
pub mod parser;
pub mod registry;

// Convenience re-exports
pub use error::{
    ParseError, ParseErrorInfo, ParseResult, SceneTableError, SceneTableResult, WordTableError,
    WordTableResult,
};
pub use parser::{FileItem, PastaFile, parse_file, parse_str};
pub use registry::{
    DefaultRandomSelector, RandomSelector, SceneEntry, SceneId, SceneInfo, SceneRegistry,
    SceneScope, SceneTable, WordCacheKey, WordDefRegistry, WordEntry, WordTable,
};
