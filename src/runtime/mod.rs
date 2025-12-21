//! Runtime module for executing Rune scripts.
//!
//! This module provides the runtime environment for executing Rune scripts,
//! including the Rune VM wrapper, variable management, and scene management.

pub mod generator;
pub mod random;
pub mod scene;
pub mod variables;
pub mod words;

pub use generator::{ScriptGenerator, ScriptGeneratorState};
pub use random::{DefaultRandomSelector, RandomSelector};
pub use scene::{SceneInfo, SceneTable};
pub use variables::{VariableManager, VariableScope, VariableValue};
pub use words::{WordCacheKey, WordTable};

#[cfg(test)]
pub use random::MockRandomSelector;
