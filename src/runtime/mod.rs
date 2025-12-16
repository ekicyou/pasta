//! Runtime module for executing Rune scripts.
//!
//! This module provides the runtime environment for executing Rune scripts,
//! including the Rune VM wrapper, variable management, and label management.

pub mod generator;
pub mod labels;
pub mod random;
pub mod variables;

pub use generator::{ScriptGenerator, ScriptGeneratorState};
pub use labels::{LabelInfo, LabelTable};
pub use random::{DefaultRandomSelector, RandomSelector};
pub use variables::{VariableManager, VariableScope, VariableValue};

#[cfg(test)]
pub use random::MockRandomSelector;
