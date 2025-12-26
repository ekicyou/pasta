//! Word table for runtime word selection.
//!
//! This module provides the runtime word lookup table that supports
//! prefix-based search and shuffle-based random selection.

use crate::error::WordTableError;

use crate::registry::{WordDefRegistry, WordEntry, random::RandomSelector};
use fast_radix_trie::RadixMap;
use std::collections::HashMap;

/// Cache key for word selection (module_name + search_key).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordCacheKey {
    /// Module name (empty string for global scope)
    pub module_name: String,
    /// Search key
    pub search_key: String,
}

impl WordCacheKey {
    /// Create a new cache key.
    pub fn new(module_name: &str, search_key: &str) -> Self {
        Self {
            module_name: module_name.to_string(),
            search_key: search_key.to_string(),
        }
    }
}

/// Cached word selection state for sequential consumption.
struct CachedWordSelection {
    /// Shuffled word list.
    words: Vec<String>,
    /// Next index to return.
    next_index: usize,
}

/// Word table for runtime word lookup and selection.
pub struct WordTable {
    /// Word entries (ID = Vec index).
    entries: Vec<WordEntry>,
    /// Prefix index (key → entry ID list).
    prefix_index: RadixMap<Vec<usize>>,
    /// Shuffle cache ((module, key) → CachedWordSelection).
    cached_selections: HashMap<WordCacheKey, CachedWordSelection>,
    /// Random selector.
    random_selector: Box<dyn RandomSelector>,
    /// Whether to shuffle candidates (default: true).
    shuffle_enabled: bool,
}

impl WordTable {
    /// Create a new word table.
    pub fn new(random_selector: Box<dyn RandomSelector>) -> Self {
        Self {
            entries: Vec::new(),
            prefix_index: RadixMap::new(),
            cached_selections: HashMap::new(),
            random_selector,
            shuffle_enabled: true,
        }
    }

    /// Build a word table from a WordDefRegistry.
    pub fn from_word_def_registry(
        registry: WordDefRegistry,
        random_selector: Box<dyn RandomSelector>,
    ) -> Self {
        let entries = registry.into_entries();
        let mut prefix_index = RadixMap::new();

        // Build prefix index
        for entry in &entries {
            let entry_list = prefix_index
                .entry(entry.key.as_bytes())
                .or_insert_with(Vec::new);
            entry_list.push(entry.id);
        }

        Self {
            entries,
            prefix_index,
            cached_selections: HashMap::new(),
            random_selector,
            shuffle_enabled: true,
        }
    }

    /// Collect all word candidates for a search key (before shuffling).
    ///
    /// This is a helper function that performs the search and merge logic:
    /// 1. Local search: `:module_name:key` prefix match
    /// 2. Global search: `key` prefix match
    /// 3. Merge all matched word entries
    ///
    /// This function is useful for testing and validating the merge logic
    /// independently from shuffling and caching.
    ///
    /// # Arguments
    /// * `module_name` - Current module name (empty for global scope)
    /// * `key` - Search key
    ///
    /// # Returns
    /// Ok(word_list) with all merged candidates, Err(WordNotFound) if no match
    pub fn collect_word_candidates(
        &self,
        module_name: &str,
        key: &str,
    ) -> Result<Vec<String>, WordTableError> {
        // Step 1: Local search (if module_name is not empty)
        let mut entry_ids: Vec<usize> = Vec::new();

        if !module_name.is_empty() {
            let local_key = format!(":{}:{}", module_name, key);
            for (_matched_key, ids) in self.prefix_index.iter_prefix(local_key.as_bytes()) {
                entry_ids.extend(ids.iter().copied());
            }
        }

        // Step 2: Global search
        // Only match keys that don't start with ':' (exclude local keys)
        for (matched_key, ids) in self.prefix_index.iter_prefix(key.as_bytes()) {
            // Skip local keys (start with ':')
            if !matched_key.starts_with(&[b':']) {
                entry_ids.extend(ids.iter().copied());
            }
        }

        // No matches found
        if entry_ids.is_empty() {
            return Err(WordTableError::WordNotFound {
                key: key.to_string(),
            });
        }

        // Step 3: Merge all words from matched entries
        let mut all_words: Vec<String> = Vec::new();
        for id in &entry_ids {
            if let Some(entry) = self.entries.get(*id) {
                all_words.extend(entry.values.iter().cloned());
            }
        }

        if all_words.is_empty() {
            return Err(WordTableError::WordNotFound {
                key: key.to_string(),
            });
        }

        Ok(all_words)
    }

    /// Search for a word using 2-stage prefix matching with caching.
    ///
    /// # Algorithm
    /// 1. Collect word candidates (local + global merge)
    /// 2. Check/build cache for shuffled selection
    /// 3. Return next word from cache
    ///
    /// # Arguments
    /// * `module_name` - Current module name (empty for global scope)
    /// * `key` - Search key
    /// * `_filters` - Attribute filters (reserved for future use)
    ///
    /// # Returns
    /// Ok(word) on success, Err(WordNotFound) if no match
    pub fn search_word(
        &mut self,
        module_name: &str,
        key: &str,
        _filters: &[String],
    ) -> Result<String, WordTableError> {
        // Build cache key
        let cache_key = WordCacheKey::new(module_name, key);

        // Check if cache exists and has remaining words
        if let Some(cached) = self.cached_selections.get_mut(&cache_key) {
            if cached.next_index < cached.words.len() {
                let word = cached.words[cached.next_index].clone();
                cached.next_index += 1;
                return Ok(word);
            }
        }

        // Collect all word candidates
        let all_words = self.collect_word_candidates(module_name, key)?;

        // Step 4: Shuffle and cache
        let mut word_indices: Vec<usize> = (0..all_words.len()).collect();
        if self.shuffle_enabled {
            self.random_selector.shuffle_usize(&mut word_indices);
        }
        let shuffled_words: Vec<String> = word_indices
            .into_iter()
            .map(|i| all_words[i].clone())
            .collect();

        // Step 5: Return first word and cache the rest
        let result = shuffled_words[0].clone();
        self.cached_selections.insert(
            cache_key,
            CachedWordSelection {
                words: shuffled_words,
                next_index: 1,
            },
        );

        Ok(result)
    }

    /// Set shuffle enabled flag (for testing).
    pub fn set_shuffle_enabled(&mut self, enabled: bool) {
        self.shuffle_enabled = enabled;
    }

    /// Get all entries (for testing/debugging).
    pub fn entries(&self) -> &[WordEntry] {
        &self.entries
    }

    /// Clear the selection cache (for testing).
    pub fn clear_cache(&mut self) {
        self.cached_selections.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::random::MockRandomSelector;

    fn create_test_registry() -> WordDefRegistry {
        let mut registry = WordDefRegistry::new();
        // Global words
        registry.register_global(
            "挨拶",
            vec!["こんにちは".to_string(), "おはよう".to_string()],
        );
        registry.register_global("場所", vec!["東京".to_string()]);
        registry.register_global("場所_日本", vec!["大阪".to_string(), "京都".to_string()]);
        // Local words
        registry.register_local("会話_1", "挨拶", vec!["やあ".to_string()]);
        registry.register_local("会話_1", "場所", vec!["ここ".to_string()]);
        registry
    }

    #[test]
    fn test_from_word_def_registry() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let table = WordTable::from_word_def_registry(registry, selector);

        assert_eq!(table.entries.len(), 5);
    }

    #[test]
    fn test_collect_word_candidates_global_exact() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = WordTable::from_word_def_registry(registry, selector);

        // Exact match for global word
        let candidates = table.collect_word_candidates("", "挨拶").unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"こんにちは".to_string()));
        assert!(candidates.contains(&"おはよう".to_string()));
    }

    #[test]
    fn test_collect_word_candidates_global_prefix() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = WordTable::from_word_def_registry(registry, selector);

        // Prefix match: "場所" matches both "場所" and "場所_日本"
        let candidates = table.collect_word_candidates("", "場所").unwrap();
        // Should get all 3 words merged (東京, 大阪, 京都)
        assert_eq!(candidates.len(), 3);
        assert!(candidates.contains(&"東京".to_string()));
        assert!(candidates.contains(&"大阪".to_string()));
        assert!(candidates.contains(&"京都".to_string()));
    }

    #[test]
    fn test_collect_word_candidates_local() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = WordTable::from_word_def_registry(registry, selector);

        // Local search: ":会話_1:挨拶" + global "挨拶"
        let candidates = table.collect_word_candidates("会話_1", "挨拶").unwrap();
        // Should get 3 words (やあ from local, こんにちは and おはよう from global)
        assert_eq!(candidates.len(), 3);
        assert!(candidates.contains(&"やあ".to_string()));
        assert!(candidates.contains(&"こんにちは".to_string()));
        assert!(candidates.contains(&"おはよう".to_string()));
    }

    #[test]
    fn test_collect_word_candidates_duplicate_entries_merge() {
        // Test that duplicate entries are merged in collect_word_candidates
        let mut registry = WordDefRegistry::new();
        registry.register_global(
            "挨拶",
            vec!["おはよう".to_string(), "こんにちわ".to_string()],
        );
        registry.register_global(
            "挨拶",
            vec!["はろー".to_string(), "ぐっもーにん".to_string()],
        );

        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = WordTable::from_word_def_registry(registry, selector);

        // Should collect all 4 words before shuffling
        let candidates = table.collect_word_candidates("", "挨拶").unwrap();
        assert_eq!(candidates.len(), 4);
        assert!(candidates.contains(&"おはよう".to_string()));
        assert!(candidates.contains(&"こんにちわ".to_string()));
        assert!(candidates.contains(&"はろー".to_string()));
        assert!(candidates.contains(&"ぐっもーにん".to_string()));
    }

    #[test]
    fn test_collect_word_candidates_not_found() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = WordTable::from_word_def_registry(registry, selector);

        let result = table.collect_word_candidates("", "存在しない");
        assert!(result.is_err());
        match result {
            Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "存在しない"),
            _ => panic!("Expected WordNotFound error"),
        }
    }

    #[test]
    fn test_search_word_global_exact() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Exact match for global word
        let result = table.search_word("", "挨拶", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "こんにちは");
    }

    #[test]
    fn test_search_word_global_prefix() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Prefix match: "場所" matches both "場所" and "場所_日本"
        let result = table.search_word("", "場所", &[]);
        assert!(result.is_ok());
        // Should get word from merged list (東京, 大阪, 京都)
        let word = result.unwrap();
        assert!(word == "東京" || word == "大阪" || word == "京都");
    }

    #[test]
    fn test_search_word_local() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Local search: ":会話_1:挨拶" + global "挨拶"
        let result = table.search_word("会話_1", "挨拶", &[]);
        assert!(result.is_ok());
        // Should get from merged list (やあ, こんにちは, おはよう)
        let word = result.unwrap();
        assert!(word == "やあ" || word == "こんにちは" || word == "おはよう");
    }

    #[test]
    fn test_search_word_not_found() {
        let registry = create_test_registry();
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);

        let result = table.search_word("", "存在しない", &[]);
        assert!(result.is_err());
        match result {
            Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "存在しない"),
            _ => panic!("Expected WordNotFound error"),
        }
    }

    #[test]
    fn test_search_word_cache_sequential() {
        let mut registry = WordDefRegistry::new();
        registry.register_global(
            "test",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Sequential calls should return different words until exhausted
        let r1 = table.search_word("", "test", &[]).unwrap();
        let r2 = table.search_word("", "test", &[]).unwrap();
        let r3 = table.search_word("", "test", &[]).unwrap();

        assert_eq!(r1, "a");
        assert_eq!(r2, "b");
        assert_eq!(r3, "c");
    }

    #[test]
    fn test_search_word_cache_reshuffle() {
        let mut registry = WordDefRegistry::new();
        registry.register_global("test", vec!["a".to_string(), "b".to_string()]);

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Exhaust cache
        let _ = table.search_word("", "test", &[]).unwrap();
        let _ = table.search_word("", "test", &[]).unwrap();

        // Next call should reshuffle and start over
        let r3 = table.search_word("", "test", &[]).unwrap();
        assert_eq!(r3, "a"); // Back to first (no shuffle)
    }

    #[test]
    fn test_cache_key_separation() {
        let mut registry = WordDefRegistry::new();
        registry.register_global("word", vec!["global".to_string()]);
        registry.register_local("mod1", "word", vec!["local1".to_string()]);
        registry.register_local("mod2", "word", vec!["local2".to_string()]);

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Different modules should have separate caches
        let r1 = table.search_word("mod1", "word", &[]).unwrap();
        let r2 = table.search_word("mod2", "word", &[]).unwrap();

        // mod1 search: local1 + global = [local1, global]
        // mod2 search: local2 + global = [local2, global]
        assert!(r1 == "local1" || r1 == "global");
        assert!(r2 == "local2" || r2 == "global");
    }

    #[test]
    fn test_local_does_not_match_global_prefix() {
        let mut registry = WordDefRegistry::new();
        registry.register_global("abc", vec!["global_abc".to_string()]);
        registry.register_local("mod", "abc", vec!["local_abc".to_string()]);

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Global search should not include local entries
        let result = table.search_word("", "abc", &[]).unwrap();
        assert_eq!(result, "global_abc");
    }

    #[test]
    fn test_search_word_merge_duplicate_entries() {
        // Test that multiple registrations of same key are merged at search time
        let mut registry = WordDefRegistry::new();

        // Same global word name registered twice
        registry.register_global(
            "挨拶",
            vec!["おはよう".to_string(), "こんにちわ".to_string()],
        );
        registry.register_global(
            "挨拶",
            vec!["はろー".to_string(), "ぐっもーにん".to_string()],
        );

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Search for "挨拶" should find both entries and merge all 4 words
        let result = table.search_word("", "挨拶", &[]);
        assert!(result.is_ok());

        // Verify that we get one of the 4 expected words
        let word = result.unwrap();
        assert!(
            word == "おはよう"
                || word == "こんにちわ"
                || word == "はろー"
                || word == "ぐっもーにん",
            "Unexpected word: {}",
            word
        );
    }

    #[test]
    fn test_search_word_merge_duplicate_entries_all_words_reachable() {
        // Test that sequential calls reach all 4 words from duplicate entries
        let mut registry = WordDefRegistry::new();

        registry.register_global(
            "挨拶",
            vec!["おはよう".to_string(), "こんにちわ".to_string()],
        );
        registry.register_global(
            "挨拶",
            vec!["はろー".to_string(), "ぐっもーにん".to_string()],
        );

        let selector = Box::new(MockRandomSelector::new(vec![0, 1, 2, 3]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Sequential calls should exhaust all 4 words
        let r1 = table.search_word("", "挨拶", &[]).unwrap();
        let r2 = table.search_word("", "挨拶", &[]).unwrap();
        let r3 = table.search_word("", "挨拶", &[]).unwrap();
        let r4 = table.search_word("", "挨拶", &[]).unwrap();

        // Collect results
        let results = vec![r1, r2, r3, r4];

        // All 4 words should be represented
        assert!(results.contains(&"おはよう".to_string()));
        assert!(results.contains(&"こんにちわ".to_string()));
        assert!(results.contains(&"はろー".to_string()));
        assert!(results.contains(&"ぐっもーにん".to_string()));
    }

    #[test]
    fn test_search_word_merge_duplicate_local_entries() {
        // Test that duplicate local entries are merged
        let mut registry = WordDefRegistry::new();

        registry.register_local("会話", "挨拶", vec!["やあ".to_string(), "よう".to_string()]);
        registry.register_local(
            "会話",
            "挨拶",
            vec!["おす".to_string(), "ういっす".to_string()],
        );

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // Search for local word should find both entries
        let result = table.search_word("会話", "挨拶", &[]);
        assert!(result.is_ok());

        let word = result.unwrap();
        assert!(
            word == "やあ" || word == "よう" || word == "おす" || word == "ういっす",
            "Unexpected word: {}",
            word
        );
    }

    #[test]
    fn test_search_word_merge_duplicate_with_global() {
        // Test that duplicate global + global entries are all merged
        // Pattern: search for global, should get all 5 words (2 + 3)
        let mut registry = WordDefRegistry::new();

        registry.register_global("word", vec!["a".to_string(), "b".to_string()]);
        registry.register_global(
            "word",
            vec!["c".to_string(), "d".to_string(), "e".to_string()],
        );

        let selector = Box::new(MockRandomSelector::new(vec![0, 1, 2, 3, 4]));
        let mut table = WordTable::from_word_def_registry(registry, selector);
        table.set_shuffle_enabled(false);

        // All 5 calls should succeed
        let results: Vec<_> = (0..5)
            .map(|_| table.search_word("", "word", &[]).unwrap())
            .collect();

        // Check all 5 words are reachable
        assert_eq!(results.len(), 5);
        assert!(results.contains(&"a".to_string()));
        assert!(results.contains(&"b".to_string()));
        assert!(results.contains(&"c".to_string()));
        assert!(results.contains(&"d".to_string()));
        assert!(results.contains(&"e".to_string()));
    }
}
