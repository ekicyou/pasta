//! Public type definitions for scene management.
//!
//! SceneId, SceneScope, SceneInfo are used across multiple crates
//! and separated from the SceneTable implementation for clarity.

use std::collections::HashMap;

/// Unique identifier for a scene (Vec index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub usize);

/// Scene scope type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneScope {
    /// Global scene (accessible from any scope).
    Global,
    /// Local scene (accessible only within parent scene).
    Local,
}

/// Information about a single scene.
#[derive(Debug, Clone)]
pub struct SceneInfo {
    /// Unique identifier for this scene.
    pub id: SceneId,
    /// Scene name.
    pub name: String,
    /// Scene scope.
    pub scope: SceneScope,
    /// Attributes for filtering.
    pub attributes: HashMap<String, String>,
    /// Generated function name in Rune code.
    pub fn_name: String,
    /// Parent scene name (for local scenes).
    pub parent: Option<String>,
}
