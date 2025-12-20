//! Word table for runtime word selection.
//!
//! This module provides the runtime word lookup table that supports
//! prefix-based search and shuffle-based random selection.

use crate::runtime::random::RandomSelector;
use crate::transpiler::{WordDefRegistry, WordEntry};
use crate::PastaError;
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

    /// Search for a word using 2-stage prefix matching.
    ///
    /// # Algorithm
    /// 1. Local search: `:module_name:key` prefix match
    /// 2. Global search: `key` prefix match
    /// 3. Merge entry IDs from both searches
    /// 4. Build/use cache for shuffled selection
    /// 5. Return next word from cache
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
    ) -> Result<String, PastaError> {
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
            return Err(PastaError::WordNotFound {
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
            return Err(PastaError::WordNotFound {
                key: key.to_string(),
            });
        }

        // Step 4: Shuffle and cache
        let mut word_indices: Vec<usize> = (0..all_words.len()).collect();
        if self.shuffle_enabled {
            self.random_selector.shuffle_usize(&mut word_indices);
        }
        let shuffled_words: Vec<String> = word_indices.into_iter().map(|i| all_words[i].clone()).collect();

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
    use crate::runtime::random::MockRandomSelector;

    fn create_test_registry() -> WordDefRegistry {
        let mut registry = WordDefRegistry::new();
        // Global words
        registry.register_global("挨拶", vec!["こんにちは".to_string(), "おはよう".to_string()]);
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
            Err(PastaError::WordNotFound { key }) => assert_eq!(key, "存在しない"),
            _ => panic!("Expected WordNotFound error"),
        }
    }

    #[test]
    fn test_search_word_cache_sequential() {
        let mut registry = WordDefRegistry::new();
        registry.register_global("test", vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        
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
}
