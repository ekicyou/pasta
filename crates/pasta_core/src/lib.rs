//! Pasta Core - Language-independent registry and utility layer.
//!
//! This crate provides the registry functionality for the Pasta DSL.
//! DSL parsing (parser, AST, grammar) has been moved to the `pasta_dsl` crate.
//!
//! # Modules
//!
//! - `registry`: Scene and word registration (Pass 1 + Runtime tables)
//! - `error`: Registry-related error types (SceneTableError, WordTableError)
//!
//! # Example
//!
//! ```no_run
//! use pasta_core::registry::{SceneRegistry, WordDefRegistry};
//!
//! let mut scene_reg = SceneRegistry::new();
//! let mut word_reg = WordDefRegistry::new();
//! ```

pub mod error;
pub mod registry;

// Convenience re-exports
pub use error::{SceneTableError, SceneTableResult, WordTableError, WordTableResult};
pub use registry::{
    DefaultRandomSelector, MockRandomSelector, RandomSelector, SceneEntry, SceneId, SceneInfo,
    SceneRegistry, SceneScope, SceneTable, WordCacheKey, WordDefRegistry, WordEntry, WordTable,
};
