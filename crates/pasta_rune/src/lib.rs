//! Pasta Rune - Script engine with Rune language backend.
//!
//! This crate provides the Rune-based execution engine for the Pasta DSL.
//! It transpiles Pasta AST to Rune code and executes it using Rune VM.
//!
//! # Modules
//!
//! - `engine`: PastaEngine - integrated API for script execution
//! - `transpiler`: AST to Rune code conversion
//! - `runtime`: Rune VM execution and generators
//! - `cache`: Parse result caching
//! - `ir`: Intermediate representation (ScriptEvent)
//! - `stdlib`: Standard library functions for Rune
//! - `error`: Runtime error types
//!
//! # Example
//!
//! ```no_run
//! use pasta_rune::PastaEngine;
//!
//! // Create engine and execute script
//! // let engine = PastaEngine::new(script_root)?;
//! // let mut gen = engine.execute("挨拶")?;
//! // while let Some(event) = gen.resume() { ... }
//! ```

pub mod cache;
pub mod engine;
pub mod error;
pub mod ir;
mod loader;
pub mod runtime;
pub mod stdlib;
pub mod transpiler;

// Re-export pasta_core as core for convenience
pub use pasta_core as core;

// Re-export parser module from pasta_core for compatibility
pub use pasta_core::parser;

// Convenience re-exports
pub use cache::ParseCache;
pub use engine::PastaEngine;
pub use error::{ParseErrorInfo, PastaError, Result, Transpiler2Pass};
pub use ir::{ContentPart, ScriptEvent};
pub use loader::{DirectoryLoader, LoadedFiles};
pub use runtime::{
    ScriptGenerator, ScriptGeneratorState, VariableManager, VariableScope, VariableValue,
};

// Re-export commonly used types from pasta_core
pub use pasta_core::{
    DefaultRandomSelector, RandomSelector, SceneEntry, SceneId, SceneInfo, SceneRegistry,
    SceneScope, SceneTable, WordCacheKey, WordDefRegistry, WordEntry, WordTable,
};
