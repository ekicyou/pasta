//! Runtime module for executing Rune scripts.
//!
//! This module provides the runtime environment for executing Rune scripts,
//! including the Rune VM wrapper, variable management, and scene management.
//!
//! Note: SceneTable, WordTable, and RandomSelector are now in pasta_core.

pub mod generator;
pub mod variables;

pub use generator::{ScriptGenerator, ScriptGeneratorState};
pub use variables::{VariableManager, VariableScope, VariableValue};

// Re-export from pasta_core for convenience
pub use pasta_core::registry::{
    DefaultRandomSelector, RandomSelector, SceneId, SceneInfo, SceneScope, SceneTable,
    WordCacheKey, WordTable,
};
