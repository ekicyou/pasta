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

    /// Collect all word candidates using fallback strategy (local → global).
    ///
    /// # Algorithm (Fallback Strategy)
    /// 1. Local search: `:module_name:key` prefix match
    ///    - 結果あり → ローカル候補のみ返す（終了）
    /// 2. Global search: `key` prefix match (exclude keys starting with ':')
    ///    - ローカル検索結果が0件の場合のみ実行
    ///
    /// # Arguments
    /// * `module_name` - Current module name (empty for global scope)
    /// * `key` - Search key
    ///
    /// # Returns
    /// Ok(word_list) with matching candidates (no fallback), Err(WordNotFound) if no match
    ///
    /// # Behavior
    /// - If `module_name` is empty: search global entries only
    /// - If `module_name` is non-empty: search local entries only (no global fallback)
    pub fn collect_word_candidates(
        &self,
        module_name: &str,
        key: &str,
    ) -> Result<Vec<String>, WordTableError> {
        // If module_name is empty, search global only
        if module_name.is_empty() {
            let mut global_entry_ids: Vec<usize> = Vec::new();
            for (matched_key, ids) in self.prefix_index.iter_prefix(key.as_bytes()) {
                // Skip local keys (start with ':')
                if !matched_key.starts_with(b":") {
                    global_entry_ids.extend(ids.iter().copied());
                }
            }

            if global_entry_ids.is_empty() {
                return Err(WordTableError::WordNotFound {
                    key: key.to_string(),
                });
            }

            let mut all_words: Vec<String> = Vec::new();
            for id in &global_entry_ids {
                if let Some(entry) = self.entries.get(*id) {
                    all_words.extend(entry.values.iter().cloned());
                }
            }

            if all_words.is_empty() {
                return Err(WordTableError::WordNotFound {
                    key: key.to_string(),
                });
            }

            return Ok(all_words);
        }

        // module_name is non-empty: search local only (no fallback)
        let local_key = format!(":{}:{}", module_name, key);
        let mut local_entry_ids: Vec<usize> = Vec::new();
        for (_matched_key, ids) in self.prefix_index.iter_prefix(local_key.as_bytes()) {
            local_entry_ids.extend(ids.iter().copied());
        }

        if local_entry_ids.is_empty() {
            return Err(WordTableError::WordNotFound {
                key: key.to_string(),
            });
        }

        let mut local_words: Vec<String> = Vec::new();
        for id in &local_entry_ids {
            if let Some(entry) = self.entries.get(*id) {
                local_words.extend(entry.values.iter().cloned());
            }
        }

        if local_words.is_empty() {
            return Err(WordTableError::WordNotFound {
                key: key.to_string(),
            });
        }

        Ok(local_words)
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
        if let Some(cached) = self.cached_selections.get_mut(&cache_key)
            && let Some(word) = cached.words.get(cached.next_index).cloned()
        {
            cached.next_index += 1;
            return Ok(word);
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
            .filter_map(|i| all_words.get(i).cloned())
            .collect();

        // Step 5: Return first word and cache the rest
        let result = shuffled_words
            .first()
            .cloned()
            .ok_or_else(|| WordTableError::WordNotFound {
                key: key.to_string(),
            })?;
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

    /// Replace the random selector and clear cached selections.
    ///
    /// Keeps entries and prefix_index intact — only the selector and cache are reset.
    pub fn replace_selector(&mut self, selector: Box<dyn RandomSelector>) {
        self.random_selector = selector;
        self.cached_selections.clear();
    }
}
