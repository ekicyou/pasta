//! Shared registry module for Pasta.
//!
//! SceneRegistry and WordDefRegistry are filled in two places: by the Lua
//! transpiler's single document-order pass (each item is registered and then its
//! code is generated), and at runtime by `finalize_scene`, which rebuilds them from
//! the Lua-side scene/word collections. The runtime search uses the latter.
//!
//! # Design
//!
//! - SceneRegistry: Tracks scenes and assigns unique IDs
//! - WordDefRegistry: Tracks word definitions
//! - SceneTable: Runtime lookup table for scenes (built from SceneRegistry)
//! - WordTable: Runtime lookup table for words (built from WordDefRegistry)
//! - RandomSelector: Language-agnostic random selection trait

pub mod random;
mod scene_registry;
mod scene_table;
mod scene_types;
mod word_registry;
mod word_table;

pub use random::{DefaultRandomSelector, MockRandomSelector, RandomSelector};
pub use scene_registry::{SceneEntry, SceneRegistry};
pub use scene_table::SceneTable;
pub use scene_types::{SceneId, SceneInfo, SceneScope};
pub use word_registry::{WordDefRegistry, WordEntry};
pub use word_table::{WordCacheKey, WordTable};
