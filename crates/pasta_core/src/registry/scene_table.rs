//! Scene management for Pasta scripts.
//!
//! This module provides scene registration, lookup, and random selection
//! for scenes with the same name.

use super::scene_types::*;
use crate::error::SceneTableError;
use crate::registry::random::RandomSelector;
use fast_radix_trie::RadixMap;
use std::collections::HashMap;

/// Cache key for scene resolution (module_name + search_key + sorted filters).
/// Extended to support unified scope search with module context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SceneCacheKey {
    // Clone is required for Phase 3/4 borrow split pattern
    /// Module name (グローバルシーン名)
    module_name: String,
    /// Search key
    search_key: String,
    /// Sorted filters
    filters: Vec<(String, String)>,
}

impl SceneCacheKey {
    fn new(module_name: &str, search_key: &str, filters: &HashMap<String, String>) -> Self {
        let mut filter_vec: Vec<_> = filters
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        filter_vec.sort();
        Self {
            module_name: module_name.to_string(),
            search_key: search_key.to_string(),
            filters: filter_vec,
        }
    }
}

/// Cached selection state for sequential scene consumption.
struct CachedSelection {
    candidates: Vec<SceneId>,
    next_index: usize,
    history: Vec<SceneId>,
}

/// scene table for managing script labels.
pub struct SceneTable {
    /// ID-based storage for scenes (index = SceneId).
    labels: Vec<SceneInfo>,
    /// Prefix index for forward-matching search (fn_name → [SceneId]).
    prefix_index: RadixMap<Vec<SceneId>>,
    /// Cache for sequential scene consumption ((module_name, search_key, filters) → CachedSelection).
    cache: HashMap<SceneCacheKey, CachedSelection>,
    /// Random selector for scene selection.
    random_selector: Box<dyn RandomSelector>,
    /// Whether to shuffle candidates (default: true, false for deterministic testing).
    shuffle_enabled: bool,
}

impl SceneTable {
    /// Create a new scene table with default random selector.
    pub fn new(random_selector: Box<dyn RandomSelector>) -> Self {
        Self {
            labels: Vec::new(),
            prefix_index: RadixMap::new(),
            cache: HashMap::new(),
            random_selector,
            shuffle_enabled: true,
        }
    }

    /// Create a scene table from a transpiler's SceneRegistry.
    ///
    /// This converts the SceneRegistry (used during transpilation) into
    /// a SceneTable (used during runtime).
    ///
    /// Key format conversion for prefix_index (WordTable unified format):
    /// - Local scene: `fn_name "会話_1::選択肢_1"` → `":会話_1:選択肢_1"`
    /// - Global scene: `fn_name "会話_1::__start__"` → `"会話_1"`
    pub fn from_scene_registry(
        registry: crate::registry::SceneRegistry,
        random_selector: Box<dyn RandomSelector>,
    ) -> Result<Self, SceneTableError> {
        // Build Vec storage with ID assignment
        // Note: Internal IDs are 0-based (Vec index), but select_label_to_id converts to 1-based
        let labels: Vec<SceneInfo> = registry
            .all_scenes()
            .into_iter()
            .enumerate()
            .map(|(idx, registry_info)| SceneInfo {
                id: SceneId(idx),
                name: registry_info.name.clone(),
                scope: if registry_info.parent.is_some() {
                    SceneScope::Local
                } else {
                    SceneScope::Global
                },
                attributes: registry_info.attributes.clone(),
                fn_name: registry_info.fn_name.clone(),
                parent: registry_info.parent.clone(),
            })
            .collect();

        // Build RadixMap prefix index with key conversion
        // Unified key format with WordTable:
        // - Local: `:parent:local_name`
        // - Global: `global_name`
        let mut prefix_index = RadixMap::new();
        for scene in &labels {
            // Convert fn_name to search key
            let search_key = Self::fn_name_to_search_key(&scene.fn_name, scene.parent.is_some());

            let entry = prefix_index
                .entry(search_key.as_bytes())
                .or_insert_with(Vec::new);

            // Allow duplicates for search key (multiple scenes with same prefix)
            // The original fn_name uniqueness is validated in SceneRegistry
            entry.push(scene.id);
        }

        Ok(Self {
            labels,
            prefix_index,
            cache: HashMap::new(),
            random_selector,
            shuffle_enabled: true,
        })
    }

    /// Convert fn_name to search key for prefix_index.
    ///
    /// # Key format
    /// - Local scene: `fn_name "親シーン::ローカル名"` → `":親シーン:ローカル名"`
    /// - Global scene: `fn_name "グローバル名::__start__"` → `"グローバル名"`
    ///
    /// This format is unified with WordTable's key format for consistency.
    fn fn_name_to_search_key(fn_name: &str, is_local: bool) -> String {
        if is_local {
            // Local scene: "会話_1::選択肢_1" → ":会話_1:選択肢_1"
            format!(":{}", fn_name.replace("::", ":"))
        } else {
            // Global scene: "会話_1::__start__" → "会話_1"
            fn_name.split("::").next().unwrap_or(fn_name).to_string()
        }
    }

    /// Resolve scene ID by search key and filters (P1 runtime resolution).
    ///
    /// # Algorithm
    /// 1. Prefix search using RadixMap (search_key → candidate IDs)
    /// 2. Filter by attributes
    /// 3. Cache-based sequential selection (no repeat until exhausted)
    /// 4. Return selected SceneId
    pub fn resolve_scene_id(
        &mut self,
        search_key: &str,
        filters: &HashMap<String, String>,
    ) -> Result<SceneId, SceneTableError> {
        // Validate search_key
        if search_key.is_empty() {
            return Err(SceneTableError::InvalidScene {
                scene: search_key.to_string(),
            });
        }

        // Phase 1: Prefix search using RadixMap
        let mut candidate_ids: Vec<SceneId> = Vec::new();
        for (_key, ids) in self.prefix_index.iter_prefix(search_key.as_bytes()) {
            candidate_ids.extend(ids.iter().copied());
        }

        if candidate_ids.is_empty() {
            return Err(SceneTableError::SceneNotFound {
                scene: search_key.to_string(),
            });
        }

        // Phase 2: Filter by attributes
        let filtered_ids = self.filter_by_attributes(candidate_ids, search_key, filters)?;

        // Phase 3-5: Cache-based sequential selection
        let cache_key = SceneCacheKey::new("", search_key, filters);
        self.select_from_cache(cache_key, filtered_ids)
    }

    /// Resolve scene ID with unified scope search (local + global).
    ///
    /// This is the new unified method that uses module context for local scope search.
    ///
    /// # Algorithm
    /// 1. Collect candidates using 2-stage search (collect_scene_candidates)
    /// 2. Filter by attributes
    /// 3. Cache-based sequential selection (no repeat until exhausted)
    /// 4. Return selected SceneId
    ///
    /// # Arguments
    /// * `module_name` - Current module name (parent global scene)
    /// * `search_key` - Search prefix
    /// * `filters` - Attribute filters
    pub fn resolve_scene_id_unified(
        &mut self,
        module_name: &str,
        search_key: &str,
        filters: &HashMap<String, String>,
    ) -> Result<SceneId, SceneTableError> {
        // Phase 1: Collect candidates using 2-stage search
        let candidate_ids = self.collect_scene_candidates(module_name, search_key)?;

        // Phase 2: Filter by attributes
        let filtered_ids = self.filter_by_attributes(candidate_ids, search_key, filters)?;

        // Phase 3-5: Cache-based sequential selection
        let cache_key = SceneCacheKey::new(module_name, search_key, filters);
        self.select_from_cache(cache_key, filtered_ids)
    }

    /// Filter candidate IDs by attribute filters (Phase 2 of scene resolution).
    ///
    /// Returns NoMatchingScene if no candidate satisfies all filters.
    fn filter_by_attributes(
        &self,
        candidate_ids: Vec<SceneId>,
        search_key: &str,
        filters: &HashMap<String, String>,
    ) -> Result<Vec<SceneId>, SceneTableError> {
        let filtered_ids: Vec<SceneId> = candidate_ids
            .into_iter()
            .filter(|&id| {
                let scene = &self.labels[id.0];
                filters
                    .iter()
                    .all(|(key, value)| scene.attributes.get(key) == Some(value))
            })
            .collect();

        if filtered_ids.is_empty() {
            return Err(SceneTableError::NoMatchingScene {
                scene: search_key.to_string(),
                filters: filters.clone(),
            });
        }

        Ok(filtered_ids)
    }

    /// Common helper for cache-based sequential scene selection (Phase 3-5).
    ///
    /// # Algorithm
    /// - Phase 3: Get or create cache entry with shuffled candidates
    /// - Phase 4: Reset cache when all candidates have been consumed
    /// - Phase 5: Return next candidate from cache
    fn select_from_cache(
        &mut self,
        cache_key: SceneCacheKey,
        filtered_ids: Vec<SceneId>,
    ) -> Result<SceneId, SceneTableError> {
        // Phase 3: Get or create cache entry
        let needs_reset = {
            let cached = self.cache.entry(cache_key.clone()).or_insert_with(|| {
                let mut id_values: Vec<usize> = filtered_ids.iter().map(|id| id.0).collect();
                if self.shuffle_enabled {
                    self.random_selector.shuffle_usize(&mut id_values);
                }
                let ids = id_values.into_iter().map(SceneId).collect();
                CachedSelection {
                    candidates: ids,
                    next_index: 0,
                    history: Vec::new(),
                }
            });
            cached.next_index >= cached.candidates.len()
        };

        // Phase 4: Reset if needed (borrowing released above)
        if needs_reset
            && let Some(cached) = self.cache.get_mut(&cache_key)
        {
            cached.next_index = 0;
            cached.history.clear();
            if self.shuffle_enabled {
                let mut id_values: Vec<usize> =
                    cached.candidates.iter().map(|id| id.0).collect();
                self.random_selector.shuffle_usize(&mut id_values);
                cached.candidates = id_values.into_iter().map(SceneId).collect();
            }
        }

        // Phase 5: Sequential selection
        let cached = self
            .cache
            .get_mut(&cache_key)
            .ok_or(SceneTableError::RandomSelectionFailed)?;
        let selected_id = cached
            .candidates
            .get(cached.next_index)
            .copied()
            .ok_or(SceneTableError::RandomSelectionFailed)?;
        cached.next_index += 1;
        cached.history.push(selected_id);

        Ok(selected_id)
    }

    /// Get scene info by ID.
    pub fn get_scene(&self, id: SceneId) -> Option<&SceneInfo> {
        self.labels.get(id.0)
    }

    /// Iterate over all scenes (for debugging).
    pub fn labels_iter(&self) -> impl Iterator<Item = &SceneInfo> {
        self.labels.iter()
    }

    /// Set shuffle enabled flag (for testing/debugging).
    pub fn set_shuffle_enabled(&mut self, enabled: bool) {
        self.shuffle_enabled = enabled;
    }

    /// Collect all scene candidates using fallback strategy (local → global).
    ///
    /// # Algorithm (Fallback Strategy)
    /// 1. Local search: `:module_name:prefix` で前方一致
    ///    - 結果あり → ローカル候補のみ返す（終了）
    /// 2. Global search: `prefix` で前方一致（`:` で始まるキーを除外）
    ///    - ローカル検索結果が0件の場合のみ実行
    ///
    /// # Arguments
    /// * `module_name` - グローバルシーン名
    /// * `prefix` - 検索プレフィックス
    ///
    /// # Returns
    /// Ok(Vec<SceneId>) 候補リスト（フォールバックなし）
    /// Err(SceneNotFound) 候補なし
    /// Err(InvalidScene) prefix が空の場合
    ///
    /// # Behavior
    /// - If `module_name` is empty: search global scenes only
    /// - If `module_name` is non-empty: search local scenes only (no global fallback)
    pub fn collect_scene_candidates(
        &self,
        module_name: &str,
        prefix: &str,
    ) -> Result<Vec<SceneId>, SceneTableError> {
        // Validate prefix
        if prefix.is_empty() {
            return Err(SceneTableError::InvalidScene {
                scene: prefix.to_string(),
            });
        }

        // If module_name is empty, search global only
        if module_name.is_empty() {
            let mut global_candidates = Vec::new();
            for (key, ids) in self.prefix_index.iter_prefix(prefix.as_bytes()) {
                // Skip local keys (start with ':')
                if !key.starts_with(b":") {
                    global_candidates.extend(ids.iter().copied());
                }
            }

            if global_candidates.is_empty() {
                return Err(SceneTableError::SceneNotFound {
                    scene: prefix.to_string(),
                });
            }

            return Ok(global_candidates);
        }

        // module_name is non-empty: search local only (no fallback)
        let local_search_key = format!(":{}:{}", module_name, prefix);
        let mut local_candidates = Vec::new();
        for (_key, ids) in self.prefix_index.iter_prefix(local_search_key.as_bytes()) {
            local_candidates.extend(ids.iter().copied());
        }

        if local_candidates.is_empty() {
            return Err(SceneTableError::SceneNotFound {
                scene: prefix.to_string(),
            });
        }

        Ok(local_candidates)
    }

    /// Find a scene by name, with optional attribute filters (legacy method).
    ///
    /// This is kept for backward compatibility with execute_scene().
    /// For new code, use resolve_scene_id() instead.
    pub fn find_scene(
        &mut self,
        name: &str,
        filters: &HashMap<String, String>,
    ) -> Result<String, SceneTableError> {
        // Use resolve_scene_id for the lookup
        let scene_id = self.resolve_scene_id(name, filters)?;
        let scene = self
            .get_scene(scene_id)
            .ok_or_else(|| SceneTableError::SceneNotFound {
                scene: name.to_string(),
            })?;
        Ok(scene.fn_name.clone())
    }

    /// Replace the random selector and clear cached selections.
    ///
    /// Keeps labels and prefix_index intact — only the selector and cache are reset.
    pub fn replace_selector(&mut self, selector: Box<dyn RandomSelector>) {
        self.random_selector = selector;
        self.cache.clear();
    }
}


#[cfg(test)]
#[path = "scene_table_candidate_tests.rs"]
mod scene_table_candidate_tests;

#[cfg(test)]
#[path = "scene_table_resolve_filter_tests.rs"]
mod scene_table_resolve_filter_tests;
