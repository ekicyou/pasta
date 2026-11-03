//! Scene registry for tracking scenes and assigning unique IDs.
//!
//! The registry is filled by the Lua transpiler's single document-order pass
//! (each item is registered and then its code is generated), and rebuilt at
//! runtime by `finalize_scene` from the Lua-side scene collection.

use super::scene_types::SceneAttr;
use std::collections::HashMap;

/// A single scene entry (individual scene definition).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneEntry {
    /// Unique numeric ID (starting from 1, matches Vec index + 1).
    pub id: i64,

    /// Original scene name (without counter suffix).
    pub name: String,

    /// Attributes associated with this scene (used by SceneTable attribute filters;
    /// the runtime finalize carries them over from the Lua store).
    pub attributes: Vec<SceneAttr>,

    /// Scene function name in `global::local` form (e.g., "会話_1::__start__").
    pub fn_name: String,

    /// Parent scene name (for local scenes only, None for global scenes).
    pub parent: Option<String>,
}

/// Scene registry for managing scene collection and ID assignment.
///
/// # Design Notes
///
/// - `register_global` numbers duplicate names with per-name counters (`_1`, `_2`, ...)
/// - `register_global_raw` (runtime finalize) takes names that already carry the counter
/// - IDs start from 1 and increment sequentially (ID = Vec index + 1)
/// - Each scene gets a unique ID even if names are the same
/// - Vec-based storage ensures consistent iteration order
pub struct SceneRegistry {
    /// All registered scenes (index + 1 = scene ID).
    scenes: Vec<SceneEntry>,

    /// Counter for tracking duplicate scene names (sanitized name → counter).
    /// Incremented by `register_global` for each occurrence.
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
    /// * `attributes` - Attributes for filtering
    ///
    /// # Returns
    ///
    /// The assigned ID and counter for this scene.
    pub fn register_global(&mut self, name: &str, attributes: Vec<SceneAttr>) -> (i64, usize) {
        let counter = self.increment_counter(name);
        let id = (self.scenes.len() + 1) as i64;

        let fn_name = format!("{}::__start__", Self::registered_name(name, counter));

        let entry = SceneEntry {
            id,
            name: name.to_string(),
            attributes,
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
    /// * `local_counter` - 照合用の名前ごとの通し番号（1 から。生成器の関数名と同じ番号）
    /// * `attributes` - Attributes for filtering
    ///
    /// # Returns
    ///
    /// The assigned ID for this scene.
    pub fn register_local(
        &mut self,
        name: &str,
        parent_name: &str,
        parent_counter: usize,
        local_counter: usize,
        attributes: Vec<SceneAttr>,
    ) -> i64 {
        let id = (self.scenes.len() + 1) as i64;

        // Local scene function name: parent module + local function
        // Format: 親_番号::子_番号
        // local_counter is the same number CodeGenerator's generate_local_scene uses
        let fn_name = format!(
            "{}::{}",
            Self::registered_name(parent_name, parent_counter),
            Self::registered_name(name, local_counter)
        );

        let entry = SceneEntry {
            id,
            name: name.to_string(),
            attributes,
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
    /// * `full_name` - Full scene name with counter (e.g., "OnBoot_1")
    /// * `locals` - (local function name, its attributes) pairs (e.g., `("選択肢_1", vec![])`);
    ///   `__start__` is skipped. The parent's attributes are not copied onto locals.
    /// * `attributes` - Attributes of the global scene (its `__start__`)
    ///
    /// # Returns
    ///
    /// The assigned ID for the global scene.
    pub fn register_global_raw(
        &mut self,
        full_name: &str,
        locals: &[(String, Vec<SceneAttr>)],
        attributes: Vec<SceneAttr>,
    ) -> i64 {
        let id = (self.scenes.len() + 1) as i64;

        // fn_name format: "FullName::__start__"
        let fn_name = format!("{}::__start__", full_name);

        let entry = SceneEntry {
            id,
            name: full_name.to_string(),
            attributes,
            fn_name,
            parent: None,
        };

        self.scenes.push(entry);
        let global_id = id;

        // Register local scenes (excluding __start__)
        for (local_name, local_attrs) in locals {
            if local_name != "__start__" {
                let local_id = (self.scenes.len() + 1) as i64;
                let local_fn_name = format!("{}::{}", full_name, local_name);

                let local_entry = SceneEntry {
                    id: local_id,
                    name: local_name.clone(),
                    attributes: local_attrs.clone(),
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

    /// Increment the counter for a scene name and return the new value.
    ///
    /// The counter is keyed on the **sanitized** name so that build-time counters
    /// (module_name for WORD registration, the scene map's `join_key`, and local
    /// `parent_counter`) match the runtime, which keys its per-scene counter on the
    /// sanitized base name (`pasta/scene.lua` `get_or_increment_counter`, called with
    /// `base_name = sanitize_name(scene.name)`).
    ///
    /// For non-colliding names this is byte-identical to raw keying (a raw name and
    /// its sanitization occur in lockstep). For names that distinctly sanitize to the
    /// same base (e.g. `会話·A` and `会話_A` → `会話_A`) it prevents a counter collision
    /// that would otherwise produce duplicate `join_key`s / fn_names and `OnPastaSceneAt`
    /// playing the wrong scene.
    fn increment_counter(&mut self, name: &str) -> usize {
        let key = Self::sanitize_name(name);
        let counter = self.name_counters.entry(key).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Sanitize a scene name for use in generated Lua identifiers and registry keys.
    ///
    /// Replaces any character that is not alphanumeric or underscore with underscore.
    /// This is used by both SceneRegistry and WordDefRegistry for consistent naming.
    ///
    /// It is also the matching rule on the search side: pasta_lua's
    /// `SearchContext::search_scene` applies it to the scene name and
    /// `SearchContext::search_word` to the scope, so a raw name finds what was
    /// registered from it. Registration and search must keep sharing this one rule.
    pub fn sanitize_name(name: &str) -> String {
        name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    /// 登録名を作る: 照合用の名前（`sanitize_name`）・`_`・通し番号。
    ///
    /// 例: `registered_name("会話・朝", 1)` = `会話_朝_1`。結果に `:` は含まれない。
    pub fn registered_name(name: &str, counter: usize) -> String {
        format!("{}_{}", Self::sanitize_name(name), counter)
    }

    /// 登録名を（名前, 通し番号）に分ける。`registered_name` の逆。
    ///
    /// 最後の `_` の後ろが 1 文字以上の ASCII 数字で、前が空でないときだけ分ける。
    /// それ以外（`__start__`・`加算ループ`・`_1`・usize に収まらない数字列）は（全体, None）。
    pub fn split_registered_name(registered: &str) -> (&str, Option<usize>) {
        if let Some((name, digits)) = registered.rsplit_once('_')
            && !name.is_empty()
            && !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
            && let Ok(counter) = digits.parse()
        {
            return (name, Some(counter));
        }
        (registered, None)
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
    use crate::registry::scene_types::SceneAttrValue;

    #[test]
    fn test_register_global_label() {
        let mut registry = SceneRegistry::new();

        let (id1, counter1) = registry.register_global("会話", Vec::new());
        assert_eq!(id1, 1);
        assert_eq!(counter1, 1);

        let scene = registry.all_scenes()[id1 as usize - 1];
        assert_eq!(scene.name, "会話");
        assert_eq!(scene.fn_name, "会話_1::__start__");
        assert_eq!(scene.parent, None);
    }

    #[test]
    fn test_register_multiple_global_labels() {
        let mut registry = SceneRegistry::new();

        let (id1, _) = registry.register_global("会話", Vec::new());
        let (id2, _) = registry.register_global("別会話", Vec::new());

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

        let (id1, counter1) = registry.register_global("会話", Vec::new());
        let (id2, counter2) = registry.register_global("会話", Vec::new());

        assert_eq!(id1, 1);
        assert_eq!(counter1, 1);
        assert_eq!(id2, 2);
        assert_eq!(counter2, 2);

        let label1 = registry.all_scenes()[id1 as usize - 1];
        let label2 = registry.all_scenes()[id2 as usize - 1];

        assert_eq!(label1.fn_name, "会話_1::__start__");
        assert_eq!(label2.fn_name, "会話_2::__start__");
    }

    #[test]
    fn test_register_local_label() {
        let mut registry = SceneRegistry::new();

        // Register parent first
        let (parent_id, parent_counter) = registry.register_global("会話", Vec::new());

        // Register local scene
        let local_id = registry.register_local("選択肢", "会話", parent_counter, 1, Vec::new());

        assert_eq!(parent_id, 1);
        assert_eq!(local_id, 2);

        let local_label = registry.all_scenes()[local_id as usize - 1];
        assert_eq!(local_label.name, "選択肢");
        assert_eq!(local_label.parent, Some("会話".to_string()));
        // Local scene function is in parent module: 親_番号::子名_番号
        assert_eq!(local_label.fn_name, "会話_1::選択肢_1");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(SceneRegistry::sanitize_name("hello"), "hello");
        assert_eq!(SceneRegistry::sanitize_name("hello-world"), "hello_world");
        assert_eq!(SceneRegistry::sanitize_name("会話"), "会話");
        assert_eq!(SceneRegistry::sanitize_name("＊会話"), "_会話");
    }

    #[test]
    fn test_registered_name_round_trip() {
        // 末尾が数字・`_` と数字で終わる・記号を含む・`_` だけの名前で往復一致する
        for name in ["メイン", "章11", "章_1", "会話・朝", "_", "__", "a_"] {
            for counter in [1, 2, 11, usize::MAX] {
                let registered = SceneRegistry::registered_name(name, counter);
                assert!(!registered.contains(':'), "{registered}");
                assert_eq!(
                    SceneRegistry::split_registered_name(&registered),
                    (SceneRegistry::sanitize_name(name).as_str(), Some(counter)),
                    "{name} / {counter}"
                );
            }
        }
        assert_eq!(SceneRegistry::registered_name("会話・朝", 1), "会話_朝_1");
        // `A1` の 1 つ目と `A` の 11 個目は別の登録名
        assert_ne!(
            SceneRegistry::registered_name("A1", 1),
            SceneRegistry::registered_name("A", 11)
        );
    }

    #[test]
    fn test_split_registered_name() {
        assert_eq!(
            SceneRegistry::split_registered_name("章_1_1"),
            ("章_1", Some(1))
        );
        assert_eq!(
            SceneRegistry::split_registered_name("章_11"),
            ("章", Some(11))
        );
        // 分けられない名前は（全体, 番号なし）
        for whole in [
            "__start__",
            "加算ループ",
            "_1",
            "章_",
            "章_1a",
            "章_１",
            "章_99999999999999999999999999",
        ] {
            assert_eq!(SceneRegistry::split_registered_name(whole), (whole, None));
        }
    }

    #[test]
    fn test_register_global_raw_basic() {
        let mut registry = SceneRegistry::new();

        // full_name already contains the counter; no auto-increment
        let id = registry.register_global_raw("OnBoot_1", &[], Vec::new());
        assert_eq!(id, 1);

        let scene = registry.all_scenes()[id as usize - 1];
        assert_eq!(scene.name, "OnBoot_1");
        assert_eq!(scene.fn_name, "OnBoot_1::__start__");
        assert_eq!(scene.parent, None);
    }

    #[test]
    fn test_register_global_raw_with_locals_excludes_start() {
        let mut registry = SceneRegistry::new();

        let locals = vec![
            ("__start__".to_string(), Vec::new()),
            ("__選択肢_1__".to_string(), Vec::new()),
        ];
        let global_id = registry.register_global_raw("会話_1", &locals, Vec::new());

        // Returns the GLOBAL scene's ID
        assert_eq!(global_id, 1);

        // __start__ must NOT be registered as a separate local scene
        let scenes = registry.all_scenes();
        assert_eq!(scenes.len(), 2); // global + 1 local (not 3)

        let local = registry.all_scenes()[1];
        assert_eq!(local.name, "__選択肢_1__");
        assert_eq!(local.fn_name, "会話_1::__選択肢_1__");
        assert_eq!(local.parent, Some("会話_1".to_string()));
    }

    #[test]
    fn test_register_global_raw_does_not_copy_parent_attributes_to_locals() {
        let mut registry = SceneRegistry::new();

        let attrs = vec![SceneAttr::Decl {
            key: "季節".to_string(),
            value: SceneAttrValue::String("春".to_string()),
        }];
        let local_attrs = vec![SceneAttr::Flag {
            key: "済み".to_string(),
        }];
        let locals = vec![
            ("item_1".to_string(), Vec::new()),
            ("item_2".to_string(), local_attrs.clone()),
        ];
        registry.register_global_raw("Scene1", &locals, attrs.clone());

        // The global keeps its own; each local carries only what it was given (3.8)
        let scenes = registry.all_scenes();
        assert_eq!(scenes[0].attributes, attrs);
        assert_eq!(scenes[1].attributes, Vec::new());
        assert_eq!(scenes[2].attributes, local_attrs);
    }

    #[test]
    fn test_register_global_raw_does_not_touch_counters() {
        let mut registry = SceneRegistry::new();

        // raw registration must not consume a counter for "会話"
        registry.register_global_raw("会話_1", &[], Vec::new());

        // counter-based registration still starts at 1
        let (_, counter) = registry.register_global("会話", Vec::new());
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_register_local_sanitizes_names() {
        let mut registry = SceneRegistry::new();
        let (_, counter) = registry.register_global("親-シーン", Vec::new());
        let local_id = registry.register_local("子-シーン", "親-シーン", counter, 1, Vec::new());

        let local = registry.all_scenes()[local_id as usize - 1];
        // Both parent and child names are sanitized in fn_name; original name preserved
        assert_eq!(local.fn_name, "親_シーン_1::子_シーン_1");
        assert_eq!(local.name, "子-シーン");
        assert_eq!(local.parent, Some("親-シーン".to_string()));
    }
}
