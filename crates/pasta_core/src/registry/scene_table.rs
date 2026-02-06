//! Scene management for Pasta scripts.
//!
//! This module provides scene registration, lookup, and random selection
//! for scenes with the same name.

use crate::error::SceneTableError;
use crate::registry::random::RandomSelector;
use fast_radix_trie::RadixMap;
use std::collections::HashMap;

/// Unique identifier for a scene (Vec index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub usize);

/// Scene scope type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneScope {
    /// Global scene (accessible from any scope).
    Global,
    /// Local scene (accessible only within parent scene).
    Local,
}

/// Information about a single scene.
#[derive(Debug, Clone)]
pub struct SceneInfo {
    /// Unique identifier for this scene.
    pub id: SceneId,
    /// Scene name.
    pub name: String,
    /// Scene scope.
    pub scope: SceneScope,
    /// Attributes for filtering.
    pub attributes: HashMap<String, String>,
    /// Generated function name in Rune code.
    pub fn_name: String,
    /// Parent scene name (for local scenes).
    pub parent: Option<String>,
}

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
            let parts: Vec<&str> = fn_name.split("::").collect();
            format!(":{}", parts.join(":"))
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

        // Phase 3: Get or create cache entry
        // Note: 旧 resolve_scene_id は module_name なしで呼び出されるため、空文字を使用
        let cache_key = SceneCacheKey::new("", search_key, filters);
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

        // Phase 4: Reset if needed (借用解放済み)
        if needs_reset {
            let cached = self.cache.get_mut(&cache_key).unwrap();
            cached.next_index = 0;
            cached.history.clear();
            if self.shuffle_enabled {
                let mut id_values: Vec<usize> = cached.candidates.iter().map(|id| id.0).collect();
                self.random_selector.shuffle_usize(&mut id_values);
                cached.candidates = id_values.into_iter().map(SceneId).collect();
            }
        }

        // Phase 5: Sequential selection
        let cached = self.cache.get_mut(&cache_key).unwrap();
        let selected_id = cached.candidates[cached.next_index];
        cached.next_index += 1;
        cached.history.push(selected_id);

        Ok(selected_id)
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

        // Phase 3: Get or create cache entry with module context
        let cache_key = SceneCacheKey::new(module_name, search_key, filters);
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

        // Phase 4: Reset if needed (借用解放済み)
        if needs_reset {
            let cached = self.cache.get_mut(&cache_key).unwrap();
            cached.next_index = 0;
            cached.history.clear();
            if self.shuffle_enabled {
                let mut id_values: Vec<usize> = cached.candidates.iter().map(|id| id.0).collect();
                self.random_selector.shuffle_usize(&mut id_values);
                cached.candidates = id_values.into_iter().map(SceneId).collect();
            }
        }

        // Phase 5: Sequential selection
        let cached = self.cache.get_mut(&cache_key).unwrap();
        let selected_id = cached.candidates[cached.next_index];
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
                if !key.starts_with(&[b':']) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::random::MockRandomSelector;

    fn create_test_scene_info(id: usize, name: &str, fn_name: &str) -> SceneInfo {
        SceneInfo {
            id: SceneId(id),
            name: name.to_string(),
            scope: SceneScope::Global,
            attributes: HashMap::new(),
            fn_name: fn_name.to_string(),
            parent: None,
        }
    }

    #[test]
    fn test_resolve_scene_id_basic() {
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable {
            labels: vec![create_test_scene_info(0, "test", "test_1::__start__")],
            prefix_index: {
                let mut map = RadixMap::new();
                map.insert(b"test_1::__start__", vec![SceneId(0)]);
                map
            },
            cache: HashMap::new(),
            random_selector: selector,
            shuffle_enabled: false,
        };

        let result = table.resolve_scene_id("test", &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SceneId(0));
    }

    // ======================================================================
    // Tests for collect_scene_candidates (Task 5.1)
    // ======================================================================

    fn create_test_local_scene_info(
        id: usize,
        name: &str,
        fn_name: &str,
        parent: &str,
    ) -> SceneInfo {
        SceneInfo {
            id: SceneId(id),
            name: name.to_string(),
            scope: SceneScope::Local,
            attributes: HashMap::new(),
            fn_name: fn_name.to_string(),
            parent: Some(parent.to_string()),
        }
    }

    #[test]
    fn test_collect_scene_candidates_local_search() {
        // Test: Local search should find scenes with :module:prefix pattern
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add local scene with :parent:local key format
        table.labels.push(create_test_local_scene_info(
            0,
            "選択肢",
            "会話_1::選択肢_1",
            "会話_1",
        ));
        table
            .prefix_index
            .insert(":会話_1:選択肢".as_bytes(), vec![SceneId(0)]);

        // Search from 会話_1 module
        let result = table.collect_scene_candidates("会話_1", "選択肢");
        assert!(result.is_ok());
        let candidates = result.unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0], SceneId(0));
    }

    #[test]
    fn test_collect_scene_candidates_global_search() {
        // Test: Global search (empty module_name) should find scenes without : prefix
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add global scene with simple key
        table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
        table
            .prefix_index
            .insert("挨拶".as_bytes(), vec![SceneId(0)]);

        // Search with empty module_name - should get global
        let result = table.collect_scene_candidates("", "挨拶");
        assert!(result.is_ok());
        let candidates = result.unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0], SceneId(0));
    }

    #[test]
    fn test_collect_scene_candidates_local_only() {
        // Test: Local search only - no global fallback
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add global scene
        table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
        table
            .prefix_index
            .insert("挨拶".as_bytes(), vec![SceneId(0)]);

        // Add local scene with same prefix
        table.labels.push(create_test_local_scene_info(
            1,
            "挨拶",
            "会話_1::挨拶_1",
            "会話_1",
        ));
        table
            .prefix_index
            .insert(":会話_1:挨拶".as_bytes(), vec![SceneId(1)]);

        // Search from 会話_1 module - should get ONLY local
        let result = table.collect_scene_candidates("会話_1", "挨拶");
        assert!(result.is_ok());
        let candidates = result.unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&SceneId(1))); // local only
        assert!(!candidates.contains(&SceneId(0))); // global NOT included
    }

    #[test]
    fn test_collect_scene_candidates_local_not_found_no_fallback() {
        // Test: No local found, should NOT fall back to global - return error
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add global scene only
        table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
        table
            .prefix_index
            .insert("挨拶".as_bytes(), vec![SceneId(0)]);

        // Search from 会話_1 module - no local, should NOT fall back to global
        let result = table.collect_scene_candidates("会話_1", "挨拶");
        assert!(result.is_err());
        match result {
            Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "挨拶"),
            _ => panic!("Expected SceneNotFound error"),
        }
    }

    #[test]
    fn test_collect_scene_candidates_global_via_empty_module_name() {
        // Test: Empty module_name should search global only
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add global scene
        table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
        table
            .prefix_index
            .insert("挨拶".as_bytes(), vec![SceneId(0)]);

        // Search with empty module_name - should get global
        let result = table.collect_scene_candidates("", "挨拶");
        assert!(result.is_ok());
        let candidates = result.unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&SceneId(0)));
    }

    #[test]
    fn test_collect_scene_candidates_prefix_match() {
        // Test: Prefix matching should work correctly
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add scenes with common prefix
        table
            .labels
            .push(create_test_scene_info(0, "挨拶_朝", "挨拶_朝"));
        table
            .labels
            .push(create_test_scene_info(1, "挨拶_昼", "挨拶_昼"));
        table
            .labels
            .push(create_test_scene_info(2, "挨拶_夜", "挨拶_夜"));

        // Insert with proper UTF-8 bytes
        table
            .prefix_index
            .insert("挨拶_朝".as_bytes(), vec![SceneId(0)]);
        table
            .prefix_index
            .insert("挨拶_昼".as_bytes(), vec![SceneId(1)]);
        table
            .prefix_index
            .insert("挨拶_夜".as_bytes(), vec![SceneId(2)]);

        // Prefix search for "挨拶" should find all three
        let result = table.collect_scene_candidates("", "挨拶");
        assert!(result.is_ok());
        let candidates = result.unwrap();
        assert_eq!(candidates.len(), 3);
    }

    #[test]
    fn test_collect_scene_candidates_not_found() {
        // Test: Should return error when no candidates found
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = SceneTable::new(selector);

        let result = table.collect_scene_candidates("会話_1", "存在しないシーン");
        assert!(result.is_err());
        match result {
            Err(SceneTableError::SceneNotFound { scene }) => {
                assert_eq!(scene, "存在しないシーン");
            }
            _ => panic!("Expected SceneNotFound error"),
        }
    }

    #[test]
    fn test_collect_scene_candidates_empty_prefix_error() {
        // Test: Empty prefix should return error
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = SceneTable::new(selector);

        let result = table.collect_scene_candidates("会話_1", "");
        assert!(result.is_err());
        match result {
            Err(SceneTableError::InvalidScene { scene }) => {
                assert_eq!(scene, "");
            }
            _ => panic!("Expected InvalidScene error"),
        }
    }

    #[test]
    fn test_collect_scene_candidates_exclude_local_from_global() {
        // Test: Global search should exclude local keys (starting with :)
        let selector = Box::new(MockRandomSelector::new(vec![]));
        let mut table = SceneTable::new(selector);

        // Add a local scene
        table.labels.push(create_test_local_scene_info(
            0,
            "選択肢",
            "他モジュール::選択肢_1",
            "他モジュール",
        ));
        table
            .prefix_index
            .insert(":他モジュール:選択肢".as_bytes(), vec![SceneId(0)]);

        // Search from different module - should NOT find the local scene of another module
        let result = table.collect_scene_candidates("会話_1", "選択肢");
        assert!(result.is_err()); // No candidates
    }

    // ======================================================================
    // Tests for fn_name_to_search_key and prefix_index conversion (Task 5.3)
    // ======================================================================

    #[test]
    fn test_fn_name_to_search_key_local_scene() {
        // Local scene: "会話_1::選択肢_1" → ":会話_1:選択肢_1"
        let result = SceneTable::fn_name_to_search_key("会話_1::選択肢_1", true);
        assert_eq!(result, ":会話_1:選択肢_1");
    }

    #[test]
    fn test_fn_name_to_search_key_global_scene() {
        // Global scene: "会話_1::__start__" → "会話_1"
        let result = SceneTable::fn_name_to_search_key("会話_1::__start__", false);
        assert_eq!(result, "会話_1");
    }

    #[test]
    fn test_fn_name_to_search_key_global_scene_no_suffix() {
        // Global scene without ::__start__: "挨拶" → "挨拶"
        let result = SceneTable::fn_name_to_search_key("挨拶", false);
        assert_eq!(result, "挨拶");
    }

    #[test]
    fn test_from_scene_registry_key_conversion() {
        use crate::registry::SceneRegistry;

        // Create a registry with global and local scenes
        let mut registry = SceneRegistry::new();
        let (_, counter) = registry.register_global("会話", HashMap::new());
        registry.register_local("選択肢", "会話", counter, 1, HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![]));
        let table = SceneTable::from_scene_registry(registry, selector).unwrap();

        // Verify search key conversion works with collect_scene_candidates
        // Global scene search
        let global_result = table.collect_scene_candidates("", "会話");
        assert!(global_result.is_ok());
        assert_eq!(global_result.unwrap().len(), 1);

        // Local scene search (from 会話_1 module)
        let local_result = table.collect_scene_candidates("会話_1", "選択肢");
        assert!(local_result.is_ok());
        assert_eq!(local_result.unwrap().len(), 1);

        // Local scene should not be found from different module
        let cross_module_result = table.collect_scene_candidates("他のモジュール", "選択肢");
        assert!(cross_module_result.is_err());
    }

    // ======================================================================
    // Tests for resolve_scene_id_unified (Task 5.2)
    // ======================================================================

    #[test]
    fn test_resolve_scene_id_unified_local_scene() {
        use crate::registry::SceneRegistry;

        let mut registry = SceneRegistry::new();
        let (_, counter) = registry.register_global("会話", HashMap::new());
        registry.register_local("選択肢", "会話", counter, 1, HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
        table.set_shuffle_enabled(false);

        // Resolve local scene from parent module
        let result = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
        assert!(result.is_ok());
        let scene_id = result.unwrap();
        let scene = table.get_scene(scene_id).unwrap();
        assert!(scene.name.contains("選択肢"));
    }

    #[test]
    fn test_resolve_scene_id_unified_global_scene() {
        use crate::registry::SceneRegistry;

        let mut registry = SceneRegistry::new();
        registry.register_global("挨拶", HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
        table.set_shuffle_enabled(false);

        // Resolve global scene - must use empty module_name for global search
        let result = table.resolve_scene_id_unified("", "挨拶", &HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_scene_id_unified_global_scene_no_fallback() {
        use crate::registry::SceneRegistry;

        let mut registry = SceneRegistry::new();
        registry.register_global("挨拶", HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
        table.set_shuffle_enabled(false);

        // Try to resolve global scene from a module - should fail (no fallback)
        let result = table.resolve_scene_id_unified("任意のモジュール", "挨拶", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_scene_id_unified_local_found() {
        use crate::registry::SceneRegistry;

        let mut registry = SceneRegistry::new();
        // Global scene "挨拶" (should not be found when searching from module)
        registry.register_global("挨拶", HashMap::new());
        // Local scene "挨拶" in module 会話_1
        let (_, counter) = registry.register_global("会話", HashMap::new());
        registry.register_local("挨拶", "会話", counter, 1, HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
        table.set_shuffle_enabled(false);

        // Resolve from 会話_1 - should find ONLY local (no fallback to global)
        let result = table.resolve_scene_id_unified("会話_1", "挨拶", &HashMap::new());
        assert!(result.is_ok());
        let scene_id = result.unwrap();
        let scene = table.get_scene(scene_id).unwrap();
        // Should be local scene (has parent)
        assert!(scene.parent.is_some());

        // Call again - should succeed due to cycling reset (1 local candidate cycles)
        let result2 = table.resolve_scene_id_unified("会話_1", "挨拶", &HashMap::new());
        assert!(result2.is_ok()); // Cycling reset returns the same scene
        let scene_id2 = result2.unwrap();
        let scene2 = table.get_scene(scene_id2).unwrap();
        assert!(scene2.parent.is_some()); // Still a local scene
    }

    // ======================================================================
    // Tests for cycling reset (Task 3.1〜3.4)
    // ======================================================================

    #[test]
    fn test_resolve_scene_id_cycling() {
        // Task 3.1: 3件のシーンを登録、4回目の呼び出しが成功することを検証
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable {
            labels: vec![
                create_test_scene_info(0, "OnTalk1", "OnTalk"),
                create_test_scene_info(1, "OnTalk2", "OnTalk"),
                create_test_scene_info(2, "OnTalk3", "OnTalk"),
            ],
            prefix_index: {
                let mut map = RadixMap::new();
                map.insert(b"OnTalk", vec![SceneId(0), SceneId(1), SceneId(2)]);
                map
            },
            cache: HashMap::new(),
            random_selector: selector,
            shuffle_enabled: false,
        };

        // 1回目〜3回目: 候補を順番に消費
        let r1 = table.resolve_scene_id("OnTalk", &HashMap::new());
        assert!(r1.is_ok());
        let r2 = table.resolve_scene_id("OnTalk", &HashMap::new());
        assert!(r2.is_ok());
        let r3 = table.resolve_scene_id("OnTalk", &HashMap::new());
        assert!(r3.is_ok());

        // 4回目: 循環リセットが発生し、成功すること
        let r4 = table.resolve_scene_id("OnTalk", &HashMap::new());
        assert!(r4.is_ok(), "4回目の呼び出しが失敗: 循環リセットが動作していない");

        // 返却されたSceneIdが候補のいずれかであること
        let id = r4.unwrap();
        assert!(
            id == SceneId(0) || id == SceneId(1) || id == SceneId(2),
            "返却されたSceneId {:?} が候補に含まれていない",
            id
        );

        // さらに続行できることも検証（10回以上）
        for i in 5..=12 {
            let r = table.resolve_scene_id("OnTalk", &HashMap::new());
            assert!(r.is_ok(), "{}回目の呼び出しが失敗", i);
        }
    }

    #[test]
    fn test_resolve_scene_id_cycling_reshuffles() {
        // Task 3.2: シャッフル有効時のリセット後再シャッフル検証
        // DefaultRandomSelectorを使用して実際のシャッフルが発生することを確認
        use crate::registry::random::DefaultRandomSelector;

        let selector = Box::new(DefaultRandomSelector::with_seed(42));
        let mut table = SceneTable {
            labels: vec![
                create_test_scene_info(0, "OnTalk1", "OnTalk"),
                create_test_scene_info(1, "OnTalk2", "OnTalk"),
                create_test_scene_info(2, "OnTalk3", "OnTalk"),
            ],
            prefix_index: {
                let mut map = RadixMap::new();
                map.insert(b"OnTalk", vec![SceneId(0), SceneId(1), SceneId(2)]);
                map
            },
            cache: HashMap::new(),
            random_selector: selector,
            shuffle_enabled: true,
        };

        // 1周目の順序を記録
        let mut first_cycle = Vec::new();
        for _ in 0..3 {
            let r = table.resolve_scene_id("OnTalk", &HashMap::new());
            first_cycle.push(r.unwrap());
        }

        // 2周目（リセット後）の順序を記録
        let mut second_cycle = Vec::new();
        for _ in 0..3 {
            let r = table.resolve_scene_id("OnTalk", &HashMap::new());
            assert!(r.is_ok());
            second_cycle.push(r.unwrap());
        }

        // 同じ候補セットだが、シャッフルにより順序が異なる可能性がある
        let first_set: std::collections::HashSet<_> = first_cycle.iter().collect();
        let second_set: std::collections::HashSet<_> = second_cycle.iter().collect();
        assert_eq!(first_set, second_set, "候補セットが変化している");
    }

    #[test]
    fn test_resolve_scene_id_cycling_preserves_candidates() {
        // Task 3.3: リセット前後で候補リスト（ID集合）が保持されることを検証
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable {
            labels: vec![
                create_test_scene_info(0, "OnTalk1", "OnTalk"),
                create_test_scene_info(1, "OnTalk2", "OnTalk"),
                create_test_scene_info(2, "OnTalk3", "OnTalk"),
            ],
            prefix_index: {
                let mut map = RadixMap::new();
                map.insert(b"OnTalk", vec![SceneId(0), SceneId(1), SceneId(2)]);
                map
            },
            cache: HashMap::new(),
            random_selector: selector,
            shuffle_enabled: false,
        };

        // 1周目: 全候補を収集
        let mut first_cycle: std::collections::HashSet<SceneId> = std::collections::HashSet::new();
        for _ in 0..3 {
            first_cycle.insert(table.resolve_scene_id("OnTalk", &HashMap::new()).unwrap());
        }
        assert_eq!(first_cycle.len(), 3);

        // 2周目: リセット後の全候補を収集
        let mut second_cycle: std::collections::HashSet<SceneId> = std::collections::HashSet::new();
        for _ in 0..3 {
            second_cycle.insert(table.resolve_scene_id("OnTalk", &HashMap::new()).unwrap());
        }
        assert_eq!(second_cycle.len(), 3);

        // 候補セットが同一であること
        assert_eq!(first_cycle, second_cycle, "リセット後に候補リストが変化している");
    }

    #[test]
    fn test_resolve_scene_id_unified_cycling() {
        // Task 3.4: unified版での循環リセット動作検証
        use crate::registry::SceneRegistry;

        let mut registry = SceneRegistry::new();
        let (_, counter) = registry.register_global("会話", HashMap::new());
        registry.register_local("選択肢A", "会話", counter, 1, HashMap::new());
        registry.register_local("選択肢B", "会話", counter, 2, HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
        table.set_shuffle_enabled(false);

        // 1周目: 2件のローカル候補を消費
        let r1 = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
        assert!(r1.is_ok());
        let r2 = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
        assert!(r2.is_ok());

        // 3回目: 循環リセットが発生し、成功すること
        let r3 = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
        assert!(r3.is_ok(), "unified版で3回目の呼び出しが失敗: 循環リセットが動作していない");

        // 返却されたSceneIdが有効なローカルシーンであること
        let scene = table.get_scene(r3.unwrap()).unwrap();
        assert!(scene.parent.is_some(), "返却されたシーンがローカルでない");
    }

    #[test]
    fn test_resolve_scene_id_unified_cache_key_includes_module() {
        use crate::registry::SceneRegistry;

        // Create registry with same-named local scenes in different modules
        let mut registry = SceneRegistry::new();
        let (_, counter1) = registry.register_global("会話A", HashMap::new());
        registry.register_local("選択肢", "会話A", counter1, 1, HashMap::new());
        let (_, counter2) = registry.register_global("会話B", HashMap::new());
        registry.register_local("選択肢", "会話B", counter2, 1, HashMap::new());

        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
        table.set_shuffle_enabled(false);

        // Resolve from 会話A_1
        let result_a = table.resolve_scene_id_unified("会話A_1", "選択肢", &HashMap::new());
        assert!(result_a.is_ok());

        // Resolve from 会話B_1 - should use different cache key
        let result_b = table.resolve_scene_id_unified("会話B_1", "選択肢", &HashMap::new());
        assert!(result_b.is_ok());

        // Both should succeed (different cache keys)
        // The scenes should be different
        let scene_a = table.get_scene(result_a.unwrap()).unwrap();
        let scene_b = table.get_scene(result_b.unwrap()).unwrap();
        assert_ne!(scene_a.fn_name, scene_b.fn_name);
    }
}
