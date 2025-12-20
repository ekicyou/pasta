//! Label management for Pasta scripts.
//!
//! This module provides label registration, lookup, and random selection
//! for labels with the same name.

use crate::runtime::random::RandomSelector;
use crate::{LabelScope, PastaError};
use fast_radix_trie::RadixMap;
use std::collections::HashMap;

/// Unique identifier for a label (Vec index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelId(pub usize);

/// Information about a single label.
#[derive(Debug, Clone)]
pub struct LabelInfo {
    /// Unique identifier for this label.
    pub id: LabelId,
    /// Label name.
    pub name: String,
    /// Label scope.
    pub scope: LabelScope,
    /// Attributes for filtering.
    pub attributes: HashMap<String, String>,
    /// Generated function name in Rune code.
    pub fn_name: String,
    /// Parent label name (for local labels).
    pub parent: Option<String>,
}

/// Cache key for label resolution (search_key + sorted filters).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    search_key: String,
    filters: Vec<(String, String)>,
}

impl CacheKey {
    fn new(search_key: &str, filters: &HashMap<String, String>) -> Self {
        let mut filter_vec: Vec<_> = filters
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        filter_vec.sort();
        Self {
            search_key: search_key.to_string(),
            filters: filter_vec,
        }
    }
}

/// Cached selection state for sequential label consumption.
struct CachedSelection {
    candidates: Vec<LabelId>,
    next_index: usize,
    history: Vec<LabelId>,
}

/// Label table for managing script labels.
pub struct LabelTable {
    /// ID-based storage for labels (index = LabelId).
    labels: Vec<LabelInfo>,
    /// Prefix index for forward-matching search (fn_name → [LabelId]).
    prefix_index: RadixMap<Vec<LabelId>>,
    /// Cache for sequential label consumption ((search_key, filters) → CachedSelection).
    cache: HashMap<CacheKey, CachedSelection>,
    /// Random selector for label selection.
    random_selector: Box<dyn RandomSelector>,
    /// Whether to shuffle candidates (default: true, false for deterministic testing).
    shuffle_enabled: bool,
}

impl LabelTable {
    /// Create a new label table with default random selector.
    pub fn new(random_selector: Box<dyn RandomSelector>) -> Self {
        Self {
            labels: Vec::new(),
            prefix_index: RadixMap::new(),
            cache: HashMap::new(),
            random_selector,
            shuffle_enabled: true,
        }
    }

    /// Create a label table from a transpiler's LabelRegistry.
    ///
    /// This converts the LabelRegistry (used during transpilation) into
    /// a LabelTable (used during runtime).
    pub fn from_label_registry(
        registry: crate::transpiler::LabelRegistry,
        random_selector: Box<dyn RandomSelector>,
    ) -> Result<Self, PastaError> {
        // Build Vec storage with ID assignment
        // Note: Internal IDs are 0-based (Vec index), but select_label_to_id converts to 1-based
        let labels: Vec<LabelInfo> = registry
            .iter()
            .enumerate()
            .map(|(idx, (_, registry_info))| LabelInfo {
                id: LabelId(idx),
                name: registry_info.name.clone(),
                scope: if registry_info.parent.is_some() {
                    LabelScope::Local
                } else {
                    LabelScope::Global
                },
                attributes: registry_info.attributes.clone(),
                fn_name: registry_info.fn_name.clone(),
                parent: registry_info.parent.clone(),
            })
            .collect();

        // Build RadixMap prefix index with duplicate detection
        let mut prefix_index = RadixMap::new();
        for label in &labels {
            let entry = prefix_index
                .entry(label.fn_name.as_bytes())
                .or_insert_with(Vec::new);
            
            // Check for duplicates (defensive programming)
            if !entry.is_empty() {
                return Err(PastaError::DuplicateLabelPath {
                    fn_name: label.fn_name.clone(),
                });
            }
            
            entry.push(label.id);
        }

        Ok(Self {
            labels,
            prefix_index,
            cache: HashMap::new(),
            random_selector,
            shuffle_enabled: true,
        })
    }

    /// Resolve label ID by search key and filters (P1 runtime resolution).
    ///
    /// # Algorithm
    /// 1. Prefix search using RadixMap (search_key → candidate IDs)
    /// 2. Filter by attributes
    /// 3. Cache-based sequential selection (no repeat until exhausted)
    /// 4. Return selected LabelId
    pub fn resolve_label_id(
        &mut self,
        search_key: &str,
        filters: &HashMap<String, String>,
    ) -> Result<LabelId, PastaError> {
        // Validate search_key
        if search_key.is_empty() {
            return Err(PastaError::InvalidLabel {
                label: search_key.to_string(),
            });
        }

        // Phase 1: Prefix search using RadixMap
        let mut candidate_ids: Vec<LabelId> = Vec::new();
        for (_key, ids) in self.prefix_index.iter_prefix(search_key.as_bytes()) {
            candidate_ids.extend(ids.iter().copied());
        }

        if candidate_ids.is_empty() {
            return Err(PastaError::LabelNotFound {
                label: search_key.to_string(),
            });
        }

        // Phase 2: Filter by attributes
        let filtered_ids: Vec<LabelId> = candidate_ids
            .into_iter()
            .filter(|&id| {
                let label = &self.labels[id.0];
                filters
                    .iter()
                    .all(|(key, value)| label.attributes.get(key) == Some(value))
            })
            .collect();

        if filtered_ids.is_empty() {
            return Err(PastaError::NoMatchingLabel {
                label: search_key.to_string(),
                filters: filters.clone(),
            });
        }

        // Phase 3: Get or create cache entry
        let cache_key = CacheKey::new(search_key, filters);
        let cached = self.cache.entry(cache_key).or_insert_with(|| {
            let mut id_values: Vec<usize> = filtered_ids.iter().map(|id| id.0).collect();
            if self.shuffle_enabled {
                self.random_selector.shuffle_usize(&mut id_values);
            }
            let ids = id_values.into_iter().map(LabelId).collect();
            CachedSelection {
                candidates: ids,
                next_index: 0,
                history: Vec::new(),
            }
        });

        // Phase 4: Sequential selection
        if cached.next_index >= cached.candidates.len() {
            return Err(PastaError::NoMoreLabels {
                search_key: search_key.to_string(),
                filters: filters.clone(),
            });
        }

        let selected_id = cached.candidates[cached.next_index];
        cached.next_index += 1;
        cached.history.push(selected_id);

        Ok(selected_id)
    }

    /// Get label info by ID.
    pub fn get_label(&self, id: LabelId) -> Option<&LabelInfo> {
        self.labels.get(id.0)
    }

    /// Set shuffle enabled flag (for testing/debugging).
    pub fn set_shuffle_enabled(&mut self, enabled: bool) {
        self.shuffle_enabled = enabled;
    }

    /// Find a label by name, with optional attribute filters (legacy method).
    ///
    /// This is kept for backward compatibility with execute_label().
    /// For new code, use resolve_label_id() instead.
    pub fn find_label(
        &mut self,
        name: &str,
        filters: &HashMap<String, String>,
    ) -> Result<String, PastaError> {
        // Use resolve_label_id for the lookup
        let label_id = self.resolve_label_id(name, filters)?;
        let label = self
            .get_label(label_id)
            .ok_or_else(|| PastaError::LabelNotFound {
                label: name.to_string(),
            })?;
        Ok(label.fn_name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::random::MockRandomSelector;

    fn create_test_label_info(id: usize, name: &str, fn_name: &str) -> LabelInfo {
        LabelInfo {
            id: LabelId(id),
            name: name.to_string(),
            scope: LabelScope::Global,
            attributes: HashMap::new(),
            fn_name: fn_name.to_string(),
            parent: None,
        }
    }

    #[test]
    fn test_resolve_label_id_basic() {
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = LabelTable {
            labels: vec![create_test_label_info(0, "test", "test_1::__start__")],
            prefix_index: {
                let mut map = RadixMap::new();
                map.insert(b"test_1::__start__", vec![LabelId(0)]);
                map
            },
            cache: HashMap::new(),
            random_selector: selector,
            shuffle_enabled: false,
        };

        let result = table.resolve_label_id("test", &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LabelId(0));
    }
}
