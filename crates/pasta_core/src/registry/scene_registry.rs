//! Scene registry for tracking scenes and assigning unique IDs during transpilation.
//!
//! This module implements the two-pass transpiler strategy:
//! - Pass 1: Collect all scenes from PastaFile(s) and assign unique IDs
//! - Pass 2: Generate `mod pasta {}` with ID→function path mapping

use std::collections::HashMap;

/// A single scene entry (individual scene definition).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneEntry {
    /// Unique numeric ID (starting from 1, matches Vec index + 1).
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
/// - IDs start from 1 and increment sequentially (ID = Vec index + 1)
/// - Each scene gets a unique ID even if names are the same
/// - Vec-based storage ensures consistent iteration order
pub struct SceneRegistry {
    /// All registered scenes (index + 1 = scene ID).
    scenes: Vec<SceneEntry>,

    /// Counter for tracking duplicate scene names (name → counter).
    /// P0: Always returns 1 (no duplicates expected).
    /// P1: Increments for each duplicate.
    name_counters: HashMap<String, usize>,
}

impl SceneRegistry {
    /// Create a new scene registry.
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
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
        let id = (self.scenes.len() + 1) as i64;

        let fn_name = format!("{}_{}::__start__", Self::sanitize_name(name), counter);
        let fn_path = format!("crate::{}", fn_name);

        let entry = SceneEntry {
            id,
            name: name.to_string(),
            attributes,
            fn_path,
            fn_name,
            parent: None,
        };

        self.scenes.push(entry);
        (id, counter)
    }

    /// Register a local scene.
    ///
    /// # Arguments
    ///
    /// * `name` - Original scene name (without scope prefix)
    /// * `parent_name` - Parent global scene name
    /// * `parent_counter` - Parent's counter value
    /// * `local_index` - Local scene index within parent (1-based, matches CodeGenerator)
    /// * `attributes` - Attributes for filtering (P1 feature)
    ///
    /// # Returns
    ///
    /// The assigned ID for this scene.
    pub fn register_local(
        &mut self,
        name: &str,
        parent_name: &str,
        parent_counter: usize,
        local_index: usize,
        attributes: HashMap<String, String>,
    ) -> i64 {
        let id = (self.scenes.len() + 1) as i64;

        // Local scene function path: parent module + local function
        // Format: crate::親_番号::子_番号
        // Use local_index to match CodeGenerator's generate_local_scene
        let fn_name = format!(
            "{}_{}::{}_{}",
            Self::sanitize_name(parent_name),
            parent_counter,
            Self::sanitize_name(name),
            local_index
        );
        let fn_path = format!("crate::{}", fn_name);

        let entry = SceneEntry {
            id,
            name: name.to_string(),
            attributes,
            fn_path,
            fn_name,
            parent: Some(parent_name.to_string()),
        };

        self.scenes.push(entry);
        id
    }

    /// Register a global scene with pre-formed fn_name (for finalize).
    ///
    /// Unlike `register_global`, this method does not auto-increment counters.
    /// Used when collecting scenes from Lua runtime where the full name
    /// (with counter already embedded) is known.
    ///
    /// # Arguments
    ///
    /// * `full_name` - Full scene name with counter (e.g., "OnBoot1")
    /// * `local_names` - List of local function names (e.g., ["__start__", "__選択肢_1__"])
    /// * `attributes` - Attributes for filtering (P1 feature)
    ///
    /// # Returns
    ///
    /// The assigned ID for the global scene.
    pub fn register_global_raw(
        &mut self,
        full_name: &str,
        local_names: &[String],
        attributes: HashMap<String, String>,
    ) -> i64 {
        let id = (self.scenes.len() + 1) as i64;

        // fn_name format: "FullName::__start__"
        let fn_name = format!("{}::__start__", full_name);
        let fn_path = format!("crate::{}", fn_name);

        let entry = SceneEntry {
            id,
            name: full_name.to_string(),
            attributes: attributes.clone(),
            fn_path,
            fn_name,
            parent: None,
        };

        self.scenes.push(entry);
        let global_id = id;

        // Register local scenes (excluding __start__)
        for local_name in local_names.iter() {
            if local_name != "__start__" {
                let local_id = (self.scenes.len() + 1) as i64;
                let local_fn_name = format!("{}::{}", full_name, local_name);
                let local_fn_path = format!("crate::{}", local_fn_name);

                let local_entry = SceneEntry {
                    id: local_id,
                    name: local_name.clone(),
                    attributes: attributes.clone(),
                    fn_path: local_fn_path,
                    fn_name: local_fn_name,
                    parent: Some(full_name.to_string()),
                };

                self.scenes.push(local_entry);
            }
        }

        global_id
    }

    /// Get all registered scenes in ID order.
    pub fn all_scenes(&self) -> Vec<&SceneEntry> {
        self.scenes.iter().collect()
    }

    /// Get a scene by ID.
    pub fn get_scene(&self, id: i64) -> Option<&SceneEntry> {
        if id < 1 {
            return None;
        }
        self.scenes.get((id - 1) as usize)
    }

    /// Increment the counter for a scene name and return the new value.
    ///
    /// The counter is keyed on the **sanitized** name so that build-time counters
    /// (module_name for WORD registration, the scene-kick `join_key`, and local
    /// `parent_counter`) match the runtime, which keys its per-scene counter on the
    /// sanitized base name (`pasta/scene.lua` `get_or_increment_counter`, called with
    /// `base_name = sanitize_name(scene.name)`).
    ///
    /// For non-colliding names this is byte-identical to raw keying (a raw name and
    /// its sanitization occur in lockstep). For names that distinctly sanitize to the
    /// same base (e.g. `会話·A` and `会話_A` → `会話_A`) it prevents a counter collision
    /// that would otherwise produce duplicate `join_key`s / fn_names and a wrong-scene
    /// kick at the cursor.
    fn increment_counter(&mut self, name: &str) -> usize {
        let key = Self::sanitize_name(name);
        let counter = self.name_counters.entry(key).or_insert(0);
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

    /// Merge scenes and counters from another registry.
    ///
    /// Used by PastaLoader to combine registries from multiple files.
    /// Scene IDs are reassigned during merge to maintain uniqueness.
    pub fn merge_from(&mut self, other: SceneRegistry) {
        for mut entry in other.scenes {
            // Reassign ID based on current length
            entry.id = (self.scenes.len() + 1) as i64;
            self.scenes.push(entry);
        }

        // Merge name counters (take max value)
        for (name, counter) in other.name_counters {
            let current = self.name_counters.entry(name).or_insert(0);
            if counter > *current {
                *current = counter;
            }
        }
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
        let local_id = registry.register_local("選択肢", "会話", parent_counter, 1, HashMap::new());

        assert_eq!(parent_id, 1);
        assert_eq!(local_id, 2);

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

    #[test]
    fn test_get_scene_invalid_ids() {
        let mut registry = SceneRegistry::new();
        registry.register_global("会話", HashMap::new());

        // id < 1 must return None (IDs are 1-based)
        assert!(registry.get_scene(0).is_none());
        assert!(registry.get_scene(-1).is_none());
        // out-of-range ID must return None
        assert!(registry.get_scene(2).is_none());
        // valid ID still works
        assert!(registry.get_scene(1).is_some());
    }

    #[test]
    fn test_register_global_raw_basic() {
        let mut registry = SceneRegistry::new();

        // full_name already contains the counter; no auto-increment
        let id = registry.register_global_raw("OnBoot1", &[], HashMap::new());
        assert_eq!(id, 1);

        let scene = registry.get_scene(id).unwrap();
        assert_eq!(scene.name, "OnBoot1");
        assert_eq!(scene.fn_name, "OnBoot1::__start__");
        assert_eq!(scene.fn_path, "crate::OnBoot1::__start__");
        assert_eq!(scene.parent, None);
    }

    #[test]
    fn test_register_global_raw_with_locals_excludes_start() {
        let mut registry = SceneRegistry::new();

        let locals = vec!["__start__".to_string(), "__選択肢_1__".to_string()];
        let global_id = registry.register_global_raw("会話1", &locals, HashMap::new());

        // Returns the GLOBAL scene's ID
        assert_eq!(global_id, 1);

        // __start__ must NOT be registered as a separate local scene
        let scenes = registry.all_scenes();
        assert_eq!(scenes.len(), 2); // global + 1 local (not 3)

        let local = registry.get_scene(2).unwrap();
        assert_eq!(local.name, "__選択肢_1__");
        assert_eq!(local.fn_name, "会話1::__選択肢_1__");
        assert_eq!(local.fn_path, "crate::会話1::__選択肢_1__");
        assert_eq!(local.parent, Some("会話1".to_string()));
    }

    #[test]
    fn test_register_global_raw_propagates_attributes_to_locals() {
        let mut registry = SceneRegistry::new();

        let mut attrs = HashMap::new();
        attrs.insert("季節".to_string(), "春".to_string());
        let locals = vec!["item_1".to_string()];
        registry.register_global_raw("Scene1", &locals, attrs);

        // Both global and local scenes carry the same attributes
        assert_eq!(
            registry.get_scene(1).unwrap().attributes.get("季節"),
            Some(&"春".to_string())
        );
        assert_eq!(
            registry.get_scene(2).unwrap().attributes.get("季節"),
            Some(&"春".to_string())
        );
    }

    #[test]
    fn test_register_global_raw_does_not_touch_counters() {
        let mut registry = SceneRegistry::new();

        // raw registration must not consume a counter for "会話"
        registry.register_global_raw("会話1", &[], HashMap::new());

        // counter-based registration still starts at 1
        let (_, counter) = registry.register_global("会話", HashMap::new());
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_merge_from_reassigns_ids() {
        let mut a = SceneRegistry::new();
        a.register_global("会話", HashMap::new());

        let mut b = SceneRegistry::new();
        b.register_global("別会話", HashMap::new());
        b.register_global("第三会話", HashMap::new());

        a.merge_from(b);

        // Merged scenes get sequential IDs continuing from existing scenes
        let scenes = a.all_scenes();
        assert_eq!(scenes.len(), 3);
        assert_eq!(scenes[0].id, 1);
        assert_eq!(scenes[1].id, 2);
        assert_eq!(scenes[2].id, 3);
        assert_eq!(scenes[1].name, "別会話");
        assert_eq!(scenes[2].name, "第三会話");

        // get_scene works with reassigned IDs
        assert_eq!(a.get_scene(3).unwrap().name, "第三会話");
    }

    #[test]
    fn test_merge_from_takes_max_counter() {
        let mut a = SceneRegistry::new();
        a.register_global("会話", HashMap::new()); // counter "会話" = 1

        let mut b = SceneRegistry::new();
        b.register_global("会話", HashMap::new()); // counter "会話" = 1
        b.register_global("会話", HashMap::new()); // counter "会話" = 2

        a.merge_from(b);

        // After merge, the counter must be max(1, 2) = 2, so next is 3
        let (_, counter) = a.register_global("会話", HashMap::new());
        assert_eq!(counter, 3);
    }

    #[test]
    fn test_merge_from_keeps_higher_existing_counter() {
        let mut a = SceneRegistry::new();
        a.register_global("会話", HashMap::new()); // counter = 1
        a.register_global("会話", HashMap::new()); // counter = 2

        let mut b = SceneRegistry::new();
        b.register_global("会話", HashMap::new()); // counter = 1

        a.merge_from(b);

        // Existing counter (2) is higher; next must be 3
        let (_, counter) = a.register_global("会話", HashMap::new());
        assert_eq!(counter, 3);
    }

    #[test]
    fn test_register_local_sanitizes_names() {
        let mut registry = SceneRegistry::new();
        let (_, counter) = registry.register_global("親-シーン", HashMap::new());
        let local_id =
            registry.register_local("子-シーン", "親-シーン", counter, 1, HashMap::new());

        let local = registry.get_scene(local_id).unwrap();
        // Both parent and child names are sanitized in fn_name; original name preserved
        assert_eq!(local.fn_name, "親_シーン_1::子_シーン_1");
        assert_eq!(local.name, "子-シーン");
        assert_eq!(local.parent, Some("親-シーン".to_string()));
    }
}
