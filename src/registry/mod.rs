//! Shared registry module for Pasta transpilers.
//!
//! This module contains registry components shared between transpiler and transpiler2.
//! Both modules use the same SceneRegistry and WordDefRegistry for scene/word management.
//!
//! # Design
//!
//! - SceneRegistry: Tracks scenes and assigns unique IDs during transpilation
//! - WordDefRegistry: Tracks word definitions during transpilation
//!
//! These registries are AST-type agnostic, allowing reuse by both transpiler (parser1 AST)
//! and transpiler2 (parser2 AST).

mod scene_registry;
mod word_registry;

pub use scene_registry::{SceneEntry, SceneRegistry};
pub use word_registry::{WordDefRegistry, WordEntry};
