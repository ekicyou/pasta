//! Shared registry module for Pasta transpilers.
//!
//! This module contains registry components shared between transpiler and transpiler2.
//! Both modules use the same SceneRegistry and WordDefRegistry for scene/word management.
//!
//! # Design
//!
//! - SceneRegistry: Tracks scenes and assigns unique IDs during transpilation (Pass 1)
//! - WordDefRegistry: Tracks word definitions during transpilation (Pass 1)
//! - SceneTable: Runtime lookup table for scenes (built from SceneRegistry)
//! - WordTable: Runtime lookup table for words (built from WordDefRegistry)
//! - RandomSelector: Language-agnostic random selection trait

pub mod random;
mod scene_registry;
mod scene_table;
mod word_registry;
mod word_table;

pub use random::{DefaultRandomSelector, MockRandomSelector, RandomSelector};
pub use scene_registry::{SceneEntry, SceneRegistry};
pub use scene_table::{SceneId, SceneInfo, SceneScope, SceneTable};
pub use word_registry::{WordDefRegistry, WordEntry};
pub use word_table::{WordCacheKey, WordTable};
