//! Word definition registry for tracking word entries during transpilation.
//!
//! This module collects word definitions from PastaFile and LabelDef during Pass 1
//! and assigns unique entry IDs for runtime lookup in WordTable.

use super::SceneRegistry;

/// A single word entry (unmarged individual definition).
#[derive(Debug, Clone, PartialEq)]
pub struct WordEntry {
    /// Unique entry ID (index in Vec<WordEntry>).
    pub id: usize,
    /// Search key (global: "name", local: ":module:name").
    pub key: String,
    /// Word values list.
    pub values: Vec<String>,
}

/// Word definition registry (transpiler layer).
///
/// Collects word definitions during Pass 1 and maintains entry IDs.
/// Similar pattern to SceneRegistry but for word definitions.
pub struct WordDefRegistry {
    /// All registered word entries.
    entries: Vec<WordEntry>,
}

impl WordDefRegistry {
    /// Create a new word definition registry.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a global word definition.
    ///
    /// # Arguments
    /// * `name` - Word name (e.g., "挨拶")
    /// * `values` - Word value list
    ///
    /// # Returns
    /// The assigned entry ID.
    pub fn register_global(&mut self, name: &str, values: Vec<String>) -> usize {
        let id = self.entries.len();
        let key = name.to_string();
        self.entries.push(WordEntry { id, key, values });
        id
    }

    /// Register a local word definition.
    ///
    /// # Arguments
    /// * `module_name` - Sanitized module name (e.g., "会話_1")
    /// * `name` - Word name (e.g., "挨拶")
    /// * `values` - Word value list
    ///
    /// # Returns
    /// The assigned entry ID.
    pub fn register_local(&mut self, module_name: &str, name: &str, values: Vec<String>) -> usize {
        let id = self.entries.len();
        let sanitized_module = Self::sanitize_name(module_name);
        let key = format!(":{}:{}", sanitized_module, name);
        self.entries.push(WordEntry { id, key, values });
        id
    }

    /// Register an actor word definition.
    ///
    /// Key format: `:__actor_{sanitized_actor_name}__:{word_name}`
    ///
    /// # Arguments
    /// * `actor_name` - Actor name (e.g., "さくら")
    /// * `name` - Word name (e.g., "通常")
    /// * `values` - Word value list
    ///
    /// # Returns
    /// The assigned entry ID.
    pub fn register_actor(&mut self, actor_name: &str, name: &str, values: Vec<String>) -> usize {
        let id = self.entries.len();
        let sanitized_actor = Self::sanitize_name(actor_name);
        let key = format!(":__actor_{}__:{}", sanitized_actor, name);
        self.entries.push(WordEntry { id, key, values });
        id
    }

    /// Get all registered entries.
    pub fn all_entries(&self) -> &[WordEntry] {
        &self.entries
    }

    /// Take ownership of all entries (for building WordTable).
    pub fn into_entries(self) -> Vec<WordEntry> {
        self.entries
    }

    /// Sanitize a name for use in keys (same logic as SceneRegistry).
    pub fn sanitize_name(name: &str) -> String {
        SceneRegistry::sanitize_name(name)
    }

    /// Merge entries from another registry.
    ///
    /// Used by PastaLoader to combine registries from multiple files.
    /// Entry IDs are reassigned during merge.
    pub fn merge_from(&mut self, other: WordDefRegistry) {
        for mut entry in other.entries {
            // Reassign ID based on current length
            entry.id = self.entries.len();
            self.entries.push(entry);
        }
    }
}

impl Default for WordDefRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = WordDefRegistry::new();
        assert!(registry.all_entries().is_empty());
    }

    #[test]
    fn test_register_global_basic() {
        let mut registry = WordDefRegistry::new();
        let id = registry.register_global(
            "挨拶",
            vec!["こんにちは".to_string(), "おはよう".to_string()],
        );

        assert_eq!(id, 0);
        let entries = registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 0);
        assert_eq!(entries[0].key, "挨拶");
        assert_eq!(entries[0].values, vec!["こんにちは", "おはよう"]);
    }

    #[test]
    fn test_register_global_multiple() {
        let mut registry = WordDefRegistry::new();

        let id1 = registry.register_global("場所", vec!["東京".to_string()]);
        let id2 = registry.register_global("天気", vec!["晴れ".to_string()]);
        let id3 = registry.register_global("場所", vec!["大阪".to_string()]); // Same name, separate entry

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);

        let entries = registry.all_entries();
        assert_eq!(entries.len(), 3);
        // Same name creates separate entries (no early merge)
        assert_eq!(entries[0].key, "場所");
        assert_eq!(entries[2].key, "場所");
    }

    #[test]
    fn test_register_local_basic() {
        let mut registry = WordDefRegistry::new();
        let id = registry.register_local("会話", "挨拶", vec!["やあ".to_string()]);

        assert_eq!(id, 0);
        let entries = registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, ":会話:挨拶");
        assert_eq!(entries[0].values, vec!["やあ"]);
    }

    #[test]
    fn test_register_local_with_sanitization() {
        let mut registry = WordDefRegistry::new();
        let id = registry.register_local("会話-テスト", "挨拶", vec!["こんにちは".to_string()]);

        assert_eq!(id, 0);
        let entries = registry.all_entries();
        // Module name should be sanitized (hyphen → underscore)
        assert_eq!(entries[0].key, ":会話_テスト:挨拶");
    }

    #[test]
    fn test_global_and_local_mixed() {
        let mut registry = WordDefRegistry::new();

        let id1 = registry.register_global("挨拶", vec!["こんにちは".to_string()]);
        let id2 = registry.register_local("会話", "挨拶", vec!["やあ".to_string()]);
        let id3 = registry.register_global("挨拶", vec!["おはよう".to_string()]);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);

        let entries = registry.all_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, "挨拶"); // global
        assert_eq!(entries[1].key, ":会話:挨拶"); // local
        assert_eq!(entries[2].key, "挨拶"); // global (separate entry)
    }

    #[test]
    fn test_entry_id_sequential() {
        let mut registry = WordDefRegistry::new();

        for i in 0..10 {
            let id = registry.register_global(&format!("word{}", i), vec!["value".to_string()]);
            assert_eq!(id, i);
        }

        assert_eq!(registry.all_entries().len(), 10);
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(WordDefRegistry::sanitize_name("hello"), "hello");
        assert_eq!(WordDefRegistry::sanitize_name("hello-world"), "hello_world");
        assert_eq!(WordDefRegistry::sanitize_name("会話"), "会話");
        assert_eq!(WordDefRegistry::sanitize_name("＊会話"), "_会話");
    }

    #[test]
    fn test_into_entries() {
        let mut registry = WordDefRegistry::new();
        registry.register_global("test", vec!["value".to_string()]);

        let entries = registry.into_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "test");
    }

    #[test]
    fn test_register_actor_basic() {
        let mut registry = WordDefRegistry::new();
        let id = registry.register_actor("さくら", "通常", vec!["\\s[0]".to_string()]);

        assert_eq!(id, 0);
        let entries = registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, ":__actor_さくら__:通常");
        assert_eq!(entries[0].values, vec!["\\s[0]"]);
    }

    #[test]
    fn test_register_actor_with_sanitization() {
        let mut registry = WordDefRegistry::new();
        let id = registry.register_actor("さくら-太郎", "表情", vec!["\\s[1]".to_string()]);

        assert_eq!(id, 0);
        let entries = registry.all_entries();
        // Actor name should be sanitized (hyphen → underscore)
        assert_eq!(entries[0].key, ":__actor_さくら_太郎__:表情");
    }

    #[test]
    fn test_register_actor_multiple_values() {
        let mut registry = WordDefRegistry::new();
        let id = registry.register_actor(
            "さくら",
            "表情",
            vec![
                "\\s[0]".to_string(),
                "\\s[1]".to_string(),
                "\\s[2]".to_string(),
            ],
        );

        assert_eq!(id, 0);
        let entries = registry.all_entries();
        assert_eq!(entries[0].values.len(), 3);
    }

    #[test]
    fn test_global_local_actor_mixed() {
        let mut registry = WordDefRegistry::new();

        let id1 = registry.register_global("挨拶", vec!["こんにちは".to_string()]);
        let id2 = registry.register_local("会話", "挨拶", vec!["やあ".to_string()]);
        let id3 = registry.register_actor("さくら", "表情", vec!["\\s[0]".to_string()]);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);

        let entries = registry.all_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, "挨拶"); // global
        assert_eq!(entries[1].key, ":会話:挨拶"); // local
        assert_eq!(entries[2].key, ":__actor_さくら__:表情"); // actor
    }
}
