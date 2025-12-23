//! Scene registry for tracking scenes and assigning unique IDs during transpilation.
//!
//! This module implements the two-pass transpiler strategy:
//! - Pass 1: Collect all scenes from PastaFile(s) and assign unique IDs
//! - Pass 2: Generate `mod pasta {}` with ID→function path mapping

use std::collections::HashMap;

/// Information about a registered scene.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneInfo {
    /// Unique numeric ID (starting from 1).
    pub id: i64,

    /// Original scene name (without counter suffix).
    pub name: String,

    /// Attributes associated with this scene (for future P1 filtering).
    pub attributes: HashMap<String, String>,

    /// Full Rune function path (e.g., "crate::会話_1::__start__").
    pub fn_path: String,

    /// Module/function name without "crate::" prefix (e.g., "会話_1::__start__").
    pub fn_name: String,

    /// Parent scene name (for local scenes only, None for global scenes).
    pub parent: Option<String>,
}

/// Scene registry for managing scene collection and ID assignment.
///
/// # Design Notes
///
/// - **P0 Implementation**: No duplicate scene names, all scenes get `_1` suffix
/// - **P1 Implementation**: Handle duplicate names with sequential counters (`_1`, `_2`, ...)
/// - IDs start from 1 and increment sequentially
/// - Each scene gets a unique ID even if names are the same
pub struct SceneRegistry {
    /// All registered labels, indexed by ID.
    labels: HashMap<i64, SceneInfo>,

    /// Counter for assigning the next unique ID.
    next_id: i64,

    /// Counter for tracking duplicate scene names (name → counter).
    /// P0: Always returns 1 (no duplicates expected).
    /// P1: Increments for each duplicate.
    name_counters: HashMap<String, usize>,
}

impl SceneRegistry {
    /// Create a new scene registry.
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            next_id: 1,
            name_counters: HashMap::new(),
        }
    }

    /// Register a global scene.
    ///
    /// # Arguments
    ///
    /// * `name` - Original scene name (without scope prefix)
    /// * `attributes` - Attributes for filtering (P1 feature)
    ///
    /// # Returns
    ///
    /// The assigned ID and counter for this scene.
    pub fn register_global(
        &mut self,
        name: &str,
        attributes: HashMap<String, String>,
    ) -> (i64, usize) {
        let counter = self.increment_counter(name);
        let id = self.next_id;
        self.next_id += 1;

        let fn_name = format!("{}_{}::__start__", Self::sanitize_name(name), counter);
        let fn_path = format!("crate::{}", fn_name);

        let info = SceneInfo {
            id,
            name: name.to_string(),
            attributes,
            fn_path,
            fn_name,
            parent: None,
        };

        self.labels.insert(id, info);
        (id, counter)
    }

    /// Register a local scene.
    ///
    /// # Arguments
    ///
    /// * `name` - Original scene name (without scope prefix)
    /// * `parent_name` - Parent global scene name
    /// * `parent_counter` - Parent's counter value
    /// * `attributes` - Attributes for filtering (P1 feature)
    ///
    /// # Returns
    ///
    /// The assigned ID and counter for this scene.
    pub fn register_local(
        &mut self,
        name: &str,
        parent_name: &str,
        parent_counter: usize,
        attributes: HashMap<String, String>,
    ) -> (i64, usize) {
        // For local scenes, the full name includes parent for uniqueness
        let full_name = format!("{}::{}", parent_name, name);
        let counter = self.increment_counter(&full_name);
        let id = self.next_id;
        self.next_id += 1;

        // Local scene function path: parent module + local function
        // Format: crate::親_番号::子_番号
        let fn_name = format!(
            "{}_{}::{}_{}",
            Self::sanitize_name(parent_name),
            parent_counter,
            Self::sanitize_name(name),
            counter
        );
        let fn_path = format!("crate::{}", fn_name);

        let info = SceneInfo {
            id,
            name: name.to_string(),
            attributes,
            fn_path,
            fn_name,
            parent: Some(parent_name.to_string()),
        };

        self.labels.insert(id, info);
        (id, counter)
    }

    /// Get all registered scenes.
    pub fn all_scenes(&self) -> Vec<&SceneInfo> {
        let mut scenes: Vec<_> = self.labels.values().collect();
        scenes.sort_by_key(|s| s.id);
        scenes
    }

    /// Get a scene by ID.
    pub fn get_scene(&self, id: i64) -> Option<&SceneInfo> {
        self.labels.get(&id)
    }

    /// Iterate over all registered scenes.
    pub fn iter(&self) -> impl Iterator<Item = (&i64, &SceneInfo)> {
        self.labels.iter()
    }

    /// Increment the counter for a scene name and return the new value.
    fn increment_counter(&mut self, name: &str) -> usize {
        let counter = self.name_counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Sanitize a scene name for use in Rune identifiers.
    ///
    /// Replaces any character that is not alphanumeric or underscore with underscore.
    /// This is used by both SceneRegistry and WordDefRegistry for consistent naming.
    pub fn sanitize_name(name: &str) -> String {
        name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }
}

impl Default for SceneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_global_label() {
        let mut registry = SceneRegistry::new();

        let (id1, counter1) = registry.register_global("会話", HashMap::new());
        assert_eq!(id1, 1);
        assert_eq!(counter1, 1);

        let scene = registry.get_scene(id1).unwrap();
        assert_eq!(scene.name, "会話");
        assert_eq!(scene.fn_path, "crate::会話_1::__start__");
        assert_eq!(scene.parent, None);
    }

    #[test]
    fn test_register_multiple_global_labels() {
        let mut registry = SceneRegistry::new();

        let (id1, _) = registry.register_global("会話", HashMap::new());
        let (id2, _) = registry.register_global("別会話", HashMap::new());

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let scenes = registry.all_scenes();
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].name, "会話");
        assert_eq!(scenes[1].name, "別会話");
    }

    #[test]
    fn test_register_duplicate_global_labels() {
        let mut registry = SceneRegistry::new();

        let (id1, counter1) = registry.register_global("会話", HashMap::new());
        let (id2, counter2) = registry.register_global("会話", HashMap::new());

        assert_eq!(id1, 1);
        assert_eq!(counter1, 1);
        assert_eq!(id2, 2);
        assert_eq!(counter2, 2);

        let label1 = registry.get_scene(id1).unwrap();
        let label2 = registry.get_scene(id2).unwrap();

        assert_eq!(label1.fn_path, "crate::会話_1::__start__");
        assert_eq!(label2.fn_path, "crate::会話_2::__start__");
    }

    #[test]
    fn test_register_local_label() {
        let mut registry = SceneRegistry::new();

        // Register parent first
        let (parent_id, parent_counter) = registry.register_global("会話", HashMap::new());

        // Register local scene
        let (local_id, local_counter) =
            registry.register_local("選択肢", "会話", parent_counter, HashMap::new());

        assert_eq!(parent_id, 1);
        assert_eq!(local_id, 2);
        assert_eq!(local_counter, 1);

        let local_label = registry.get_scene(local_id).unwrap();
        assert_eq!(local_label.name, "選択肢");
        assert_eq!(local_label.parent, Some("会話".to_string()));
        // Local scene function is in parent module: crate::親_番号::子名_番号
        assert_eq!(local_label.fn_path, "crate::会話_1::選択肢_1");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(SceneRegistry::sanitize_name("hello"), "hello");
        assert_eq!(SceneRegistry::sanitize_name("hello-world"), "hello_world");
        assert_eq!(SceneRegistry::sanitize_name("会話"), "会話");
        assert_eq!(SceneRegistry::sanitize_name("＊会話"), "_会話");
    }
}
