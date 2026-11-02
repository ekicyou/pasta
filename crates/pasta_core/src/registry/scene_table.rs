//! Scene management for Pasta scripts.
//!
//! This module provides scene registration, lookup, and random selection
//! for scenes with the same name.

use super::scene_types::*;
use crate::error::SceneTableError;
use crate::registry::random::RandomSelector;
use fast_radix_trie::RadixMap;
use std::collections::HashMap;

/// Cache key for scene resolution (module_name + search_key + sorted filters + exact).
/// Extended to support unified scope search with module context.
#[derive(Debug, PartialEq, Eq, Hash)]
struct SceneCacheKey {
    /// Module name (グローバルシーン名)
    module_name: String,
    /// Search key
    search_key: String,
    /// Sorted filters
    filters: Vec<(String, String)>,
    /// Exact-match search (kept apart from the prefix search's cache)
    exact: bool,
    /// 持ち主のローカルの名前（そのローカルを持つ大域の順送りだけ。ほかは None）
    owning_local: Option<String>,
}

impl SceneCacheKey {
    fn new(
        module_name: &str,
        search_key: &str,
        filters: &HashMap<String, String>,
        exact: bool,
        owning_local: Option<&str>,
    ) -> Self {
        let mut filter_vec: Vec<_> = filters
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        filter_vec.sort();
        Self {
            module_name: module_name.to_string(),
            search_key: search_key.to_string(),
            filters: filter_vec,
            exact,
            owning_local: owning_local.map(str::to_string),
        }
    }
}

/// Cached selection state for sequential scene consumption.
struct CachedSelection {
    candidates: Vec<SceneId>,
    next_index: usize,
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
}

impl SceneTable {
    /// Create a scene table from a SceneRegistry.
    ///
    /// This converts the SceneRegistry (filled at transpile time or rebuilt by the
    /// runtime finalize) into a SceneTable (used for runtime search).
    ///
    /// Key format conversion for prefix_index (WordTable unified format):
    /// - Local scene: `fn_name "会話_1::選択肢_1"` → `":会話_1:選択肢"`
    /// - Global scene: `fn_name "会話_1::__start__"` → `"会話"`
    ///
    /// 通し番号は照合に使わない（`SceneRegistry::split_registered_name` で外す）。
    pub fn from_scene_registry(
        registry: crate::registry::SceneRegistry,
        random_selector: Box<dyn RandomSelector>,
    ) -> Self {
        // Build Vec storage with ID assignment
        // Note: Internal IDs are 0-based Vec indices (SceneRegistry's 1-based IDs are not carried over)
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
            // fn_name uniqueness is not validated (SceneRegistry does not check it)
            entry.push(scene.id);
        }

        Self {
            labels,
            prefix_index,
            cache: HashMap::new(),
            random_selector,
        }
    }

    /// Convert fn_name to search key for prefix_index.
    ///
    /// # Key format
    /// - Local scene: `fn_name "親の登録名::ローカルの登録名"` → `":親の登録名:ローカルの名前の部分"`
    /// - Global scene: `fn_name "グローバルの登録名::__start__"` → `"グローバルの名前の部分"`
    ///
    /// 名前の部分は登録名から通し番号を除いたもの（`SceneRegistry::split_registered_name`）。
    /// 分けられない名前（Lua で直接定義した `加算ループ` 等）は全体がキーになる。
    /// 親の部分は登録名のまま（検索側が親の登録名で引くため）。
    ///
    /// This format is unified with WordTable's key format for consistency.
    fn fn_name_to_search_key(fn_name: &str, is_local: bool) -> String {
        use crate::registry::SceneRegistry;
        let (global, local) = fn_name.split_once("::").unwrap_or((fn_name, ""));
        if is_local {
            // Local scene: "会話_1::選択肢_1" → ":会話_1:選択肢"
            format!(
                ":{}:{}",
                global,
                SceneRegistry::split_registered_name(local).0
            )
        } else {
            // Global scene: "会話_1::__start__" → "会話"
            SceneRegistry::split_registered_name(global).0.to_string()
        }
    }

    /// Resolve scene ID with module context.
    ///
    /// Searches local scenes of `module_name` only (global scenes when it is empty);
    /// there is no local → global fallback.
    ///
    /// # Algorithm
    /// 1. Collect candidates (collect_scene_candidates)
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
        // Phase 1: Collect candidates from a single scope (no fallback)
        let candidate_ids = self.collect_scene_candidates(module_name, search_key)?;

        // Phase 2: Filter by attributes
        let filtered_ids = self.filter_by_attributes(candidate_ids, search_key, filters)?;

        // Phase 3-5: Cache-based sequential selection
        let cache_key = SceneCacheKey::new(module_name, search_key, filters, false, None);
        Ok(self.select_from_cache(cache_key, filtered_ids))
    }

    /// Resolve scene ID by exact name match within one scope.
    ///
    /// The scope is the same as `collect_scene_candidates` (global when
    /// `module_name` is empty, excluding keys starting with `:`; otherwise
    /// `:module_name:name` locals only, no fallback), but only scenes whose
    /// key equals the search key are candidates. Attribute filtering and the
    /// no-repeat sequential selection are shared with the prefix search; the
    /// cache is kept separate from it (`SceneCacheKey::exact`).
    pub fn resolve_scene_id_exact(
        &mut self,
        module_name: &str,
        name: &str,
        filters: &HashMap<String, String>,
    ) -> Result<SceneId, SceneTableError> {
        if name.is_empty() {
            return Err(SceneTableError::InvalidScene {
                scene: name.to_string(),
            });
        }

        let key = if module_name.is_empty() {
            name.to_string()
        } else {
            format!(":{}:{}", module_name, name)
        };
        // グローバルの範囲は `:` で始まるキー（ローカル）を含めない
        let candidate_ids = match self.prefix_index.get(key.as_bytes()) {
            Some(ids) if !(module_name.is_empty() && key.starts_with(':')) => ids.clone(),
            _ => Vec::new(),
        };
        if candidate_ids.is_empty() {
            return Err(SceneTableError::SceneNotFound {
                scene: name.to_string(),
            });
        }

        let filtered_ids = self.filter_by_attributes(candidate_ids, name, filters)?;
        let cache_key = SceneCacheKey::new(module_name, name, filters, true, None);
        Ok(self.select_from_cache(cache_key, filtered_ids))
    }

    /// `name` と等しいキーの大域シーンのうち、`:{大域の登録名}:{local}` と等しいキーの
    /// ローカルシーンを持つものだけを候補にし、重ならない順送りで 1 つ選ぶ。
    ///
    /// - Ok(Some(id)): 選んだ大域シーン
    /// - Ok(None): 大域シーンはあるが、どれも `local` を持たない
    /// - Err(SceneNotFound): `name` の大域シーンが無い
    /// - Err(InvalidScene): `name` か `local` が空
    ///
    /// 順送りの状態は `resolve_scene_id_exact("", name)` とは別（`SceneCacheKey::owning_local`）。
    pub fn resolve_global_owning_local_exact(
        &mut self,
        name: &str,
        local: &str,
        filters: &HashMap<String, String>,
    ) -> Result<Option<SceneId>, SceneTableError> {
        if name.is_empty() || local.is_empty() {
            return Err(SceneTableError::InvalidScene {
                scene: if name.is_empty() { name } else { local }.to_string(),
            });
        }
        // 大域の名前は `:` で始まらない（`:` で始まるキーはローカル）
        let globals = match self.prefix_index.get(name.as_bytes()) {
            Some(ids) if !name.starts_with(':') => ids.clone(),
            _ => Vec::new(),
        };
        if globals.is_empty() {
            return Err(SceneTableError::SceneNotFound {
                scene: name.to_string(),
            });
        }

        let owners: Vec<SceneId> = globals
            .into_iter()
            .filter(|&id| {
                let fn_name = &self.labels[id.0].fn_name;
                let owner = fn_name
                    .split_once("::")
                    .map_or(fn_name.as_str(), |(g, _)| g);
                self.prefix_index
                    .get(format!(":{}:{}", owner, local).as_bytes())
                    .is_some_and(|ids| !ids.is_empty())
            })
            .collect();
        if owners.is_empty() {
            return Ok(None);
        }

        let filtered_ids = self.filter_by_attributes(owners, name, filters)?;
        let cache_key = SceneCacheKey::new("", name, filters, true, Some(local));
        Ok(Some(self.select_from_cache(cache_key, filtered_ids)))
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
    ) -> SceneId {
        let random_selector = &mut self.random_selector;
        let mut shuffled = || {
            let mut id_values: Vec<usize> = filtered_ids.iter().map(|id| id.0).collect();
            random_selector.shuffle_usize(&mut id_values);
            id_values.into_iter().map(SceneId).collect::<Vec<_>>()
        };

        // Phase 3: Get or create cache entry
        let cached = self
            .cache
            .entry(cache_key)
            .or_insert_with(|| CachedSelection {
                candidates: shuffled(),
                next_index: 0,
            });

        // Phase 4: Reset when all candidates have been consumed
        if cached.next_index >= cached.candidates.len() {
            cached.candidates = shuffled();
            cached.next_index = 0;
        }

        // Phase 5: 一巡前なら next_index < len、一巡後は空でない filtered_ids で作り直して 0 なので添字は候補の中にある
        let selected_id = cached.candidates[cached.next_index];
        cached.next_index += 1;
        selected_id
    }

    /// Get scene info by ID.
    pub fn get_scene(&self, id: SceneId) -> Option<&SceneInfo> {
        self.labels.get(id.0)
    }

    /// Collect scene candidates for one scope (no local → global fallback).
    ///
    /// # Algorithm
    /// - `module_name` が空: `prefix` で前方一致（`:` で始まるキーを除外）
    /// - `module_name` が非空: `:module_name:prefix` で前方一致（ローカルのみ）
    /// - ローカル→グローバルの順序は呼び出し側（Lua の検索手順）が 2 回の呼び出しで組む
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
