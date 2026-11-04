//! Scene management for Pasta scripts.
//!
//! This module provides scene registration, lookup, and random selection
//! for scenes with the same name.

use super::scene_types::*;
use crate::error::SceneTableError;
use crate::registry::random::RandomSelector;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Bound;

/// 候補 1 つの条件を評価する述語。真なら満たす
pub type CondFn<'a> = &'a mut dyn FnMut(&SceneInfo) -> bool;

/// 順番で選べず、シャッフルへ戻った理由
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickWarning {
    /// 抽出した候補に `＆順番` の有るものと無いものが混ざっている
    OrderMixed,
    /// 抽出した候補に同じ番号が 2 つ以上ある（前方一致で別の名前から集まったとき）
    OrderDuplicate { order: i64 },
}

/// Cache key for scene resolution (module_name + search_key + exact + owning_local).
/// 変数の値（述語）は鍵に含めない。
#[derive(Debug, PartialEq, Eq, Hash)]
struct SceneCacheKey {
    /// Module name (グローバルシーン名)
    module_name: String,
    /// Search key
    search_key: String,
    /// Exact-match search (kept apart from the prefix search's cache)
    exact: bool,
    /// 持ち主のローカルの名前（そのローカルを持つ大域の順送りだけ。ほかは None）
    owning_local: Option<String>,
}

impl SceneCacheKey {
    fn new(module_name: &str, search_key: &str, exact: bool, owning_local: Option<&str>) -> Self {
        Self {
            module_name: module_name.to_string(),
            search_key: search_key.to_string(),
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
    /// Prefix index for forward-matching search (search key → [SceneId], registration order).
    prefix_index: BTreeMap<String, Vec<SceneId>>,
    /// Cache for sequential scene consumption ((module_name, search_key, exact, owning_local) → CachedSelection).
    cache: HashMap<SceneCacheKey, CachedSelection>,
    /// 順番で最後に選んだ番号（鍵・寿命は cache と同じ。メモリの中だけ）
    last_orders: HashMap<SceneCacheKey, i64>,
    /// 直前の選択でシャッフルへ戻った理由
    pick_warning: Option<PickWarning>,
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

        // Build BTreeMap prefix index with key conversion
        // Unified key format with WordTable:
        // - Local: `:parent:local_name`
        // - Global: `global_name`
        let mut prefix_index: BTreeMap<String, Vec<_>> = BTreeMap::new();
        for scene in &labels {
            // Convert fn_name to search key
            let search_key = Self::fn_name_to_search_key(&scene.fn_name, scene.parent.is_some());

            let entry = prefix_index.entry(search_key).or_default();

            // Allow duplicates for search key (multiple scenes with same prefix)
            // fn_name uniqueness is not validated (SceneRegistry does not check it)
            entry.push(scene.id);
        }

        Self {
            labels,
            prefix_index,
            cache: HashMap::new(),
            last_orders: HashMap::new(),
            pick_warning: None,
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
    /// 2. 条件と優先の段で 1 つ選ぶ（`select_from_cache`）
    ///
    /// # Arguments
    /// * `module_name` - Current module name (parent global scene)
    /// * `search_key` - Search prefix
    /// * `cond` - 候補 1 つの条件を評価する述語（`None` なら条件を見ない）
    pub fn resolve_scene_id_unified(
        &mut self,
        module_name: &str,
        search_key: &str,
        cond: Option<CondFn<'_>>,
    ) -> Result<SceneId, SceneTableError> {
        // Phase 1: Collect candidates from a single scope (no fallback)
        let candidate_ids = self.collect_scene_candidates(module_name, search_key)?;

        // Phase 2: 条件と優先の段で 1 つ選ぶ
        let cache_key = SceneCacheKey::new(module_name, search_key, false, None);
        self.select_from_cache(cache_key, candidate_ids, search_key, cond)
    }

    /// Resolve scene ID by exact name match within one scope.
    ///
    /// The scope is the same as `collect_scene_candidates` (global when
    /// `module_name` is empty, excluding keys starting with `:`; otherwise
    /// `:module_name:name` locals only, no fallback), but only scenes whose
    /// key equals the search key are candidates. The condition/priority lottery and the
    /// no-repeat sequential selection are shared with the prefix search; the
    /// cache is kept separate from it (`SceneCacheKey::exact`).
    pub fn resolve_scene_id_exact(
        &mut self,
        module_name: &str,
        name: &str,
        cond: Option<CondFn<'_>>,
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
        let candidate_ids = match self.prefix_index.get(&key) {
            Some(ids) if !(module_name.is_empty() && key.starts_with(':')) => ids.clone(),
            _ => Vec::new(),
        };
        if candidate_ids.is_empty() {
            return Err(SceneTableError::SceneNotFound {
                scene: name.to_string(),
            });
        }

        let cache_key = SceneCacheKey::new(module_name, name, true, None);
        self.select_from_cache(cache_key, candidate_ids, name, cond)
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
        cond: Option<CondFn<'_>>,
    ) -> Result<Option<SceneId>, SceneTableError> {
        if name.is_empty() || local.is_empty() {
            return Err(SceneTableError::InvalidScene {
                scene: if name.is_empty() { name } else { local }.to_string(),
            });
        }
        // 大域の名前は `:` で始まらない（`:` で始まるキーはローカル）
        let globals = match self.prefix_index.get(name) {
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
                    .get(&format!(":{}:{}", owner, local))
                    .is_some_and(|ids| !ids.is_empty())
            })
            .collect();
        if owners.is_empty() {
            return Ok(None);
        }

        let cache_key = SceneCacheKey::new("", name, true, Some(local));
        self.select_from_cache(cache_key, owners, name, cond)
            .map(Some)
    }

    /// 名前の全候補の並びから、述語と優先の段で 1 つ選ぶ（並び＋次の位置の順送り）。
    ///
    /// - 並びはシャッフル 1 回の後に優先の降順へ安定に並べ替える。シャッフルは作るときと始め直すときだけ
    /// - `cond` があれば全候補を 1 回だけ評価し、満たす候補の最高の優先を今の段とする。
    ///   満たす候補が無ければ次の位置を末尾にして NoMatchingScene
    /// - 次の位置から見て、満たす候補を返す。満たさない候補は見ただけで使ったものにする。
    ///   段より低い候補か末尾に来たら、1 回だけ始め直す
    /// - `cond` が無ければ段も見ず、どの候補も満たすとする（名指し・今の挙動）
    /// - 抽出した候補（満たす かつ 今の段）の全部に `＆順番` があり重ならなければ、
    ///   覚えた番号より大きい最小の番号、無ければ最小の番号を返して覚える（並びに触らない）
    fn select_from_cache(
        &mut self,
        cache_key: SceneCacheKey,
        candidate_ids: Vec<SceneId>,
        search_key: &str,
        cond: Option<CondFn<'_>>,
    ) -> Result<SceneId, SceneTableError> {
        let SceneTable {
            labels,
            cache,
            last_orders,
            pick_warning,
            random_selector,
            ..
        } = self;
        *pick_warning = None;
        let priority = |id: &SceneId| scene_priority(&labels[id.0].attributes);
        let no_match = || SceneTableError::NoMatchingScene {
            scene: search_key.to_string(),
        };

        // 1. 述語があれば全候補を 1 回だけ評価し、今の段を求める
        let (ok, tier) = match cond {
            None => (None, i64::MIN),
            Some(cond) => {
                let ok: HashSet<SceneId> = candidate_ids
                    .iter()
                    .copied()
                    .filter(|id| cond(&labels[id.0]))
                    .collect();
                let Some(tier) = ok.iter().map(priority).max() else {
                    // この巡は見終えた（並びがまだ無ければ作らない）
                    if let Some(cached) = cache.get_mut(&cache_key) {
                        cached.next_index = cached.candidates.len();
                    }
                    return Err(no_match());
                };
                (Some(ok), tier)
            }
        };

        // 順番の分かれ: 抽出した候補の番号を集める
        let mut ordered: Vec<(i64, SceneId)> = Vec::new();
        let mut missing = false;
        for &id in &candidate_ids {
            if ok
                .as_ref()
                .is_some_and(|ok| !ok.contains(&id) || priority(&id) != tier)
            {
                continue;
            }
            match scene_order(&labels[id.0].attributes) {
                Some(order) => ordered.push((order, id)),
                None => missing = true,
            }
        }
        if !ordered.is_empty() {
            ordered.sort_unstable_by_key(|&(order, _)| order);
            if missing {
                *pick_warning = Some(PickWarning::OrderMixed);
            } else if let Some(w) = ordered.windows(2).find(|w| w[0].0 == w[1].0) {
                *pick_warning = Some(PickWarning::OrderDuplicate { order: w[0].0 });
            } else {
                let last = last_orders.get(&cache_key).copied();
                let &(order, id) = last
                    .and_then(|last| ordered.iter().find(|(o, _)| *o > last))
                    .unwrap_or(&ordered[0]);
                last_orders.insert(cache_key, order);
                return Ok(id);
            }
        }

        let mut new_round = || {
            // 重み N の候補は N 個並べる（重み 1 だけなら今と同じ配列・同じ乱数の消費）
            // shortcut: 重みに上限が無いので並びのメモリは重みの合計に比例する（数億は尽きる）、実害が出たら累積重みのくじに替える
            let mut id_values: Vec<usize> = candidate_ids
                .iter()
                .flat_map(|id| std::iter::repeat_n(id.0, scene_weight(&labels[id.0].attributes)))
                .collect();
            random_selector.shuffle_usize(&mut id_values);
            let mut ids: Vec<SceneId> = id_values.into_iter().map(SceneId).collect();
            ids.sort_by_key(|id| Reverse(priority(id)));
            ids
        };

        // 2. 並びが無ければ作る
        let cached = cache.entry(cache_key).or_insert_with(|| CachedSelection {
            candidates: new_round(),
            next_index: 0,
        });

        // 3・4. 次の位置から見る。末尾か段より低い候補に来たら 1 回だけ始め直す
        for round in 0..2 {
            if round == 1 {
                cached.candidates = new_round();
                cached.next_index = 0;
            }
            while let Some(&id) = cached.candidates.get(cached.next_index) {
                if priority(&id) < tier {
                    break;
                }
                cached.next_index += 1;
                if ok.as_ref().is_none_or(|ok| ok.contains(&id)) {
                    return Ok(id);
                }
            }
        }
        // 始め直した巡には段の候補が必ずあるので来ない（空回りしない）
        Err(no_match())
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
            for (key, ids) in self
                .prefix_index
                .range::<str, _>((Bound::Included(prefix), Bound::Unbounded))
                .take_while(|(k, _)| k.starts_with(prefix))
            {
                // Skip local keys (start with ':')
                if !key.starts_with(':') {
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
        for (_key, ids) in self
            .prefix_index
            .range::<str, _>((Bound::Included(local_search_key.as_str()), Bound::Unbounded))
            .take_while(|(k, _)| k.starts_with(local_search_key.as_str()))
        {
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
    /// Keeps labels and prefix_index intact — only the selector, cache and remembered orders are reset.
    pub fn replace_selector(&mut self, selector: Box<dyn RandomSelector>) {
        self.random_selector = selector;
        self.cache.clear();
        self.last_orders.clear();
    }

    /// 直前の選択でシャッフルへ戻った理由。読むと空になる
    pub fn take_pick_warning(&mut self) -> Option<PickWarning> {
        self.pick_warning.take()
    }
}

#[cfg(test)]
#[path = "scene_table_candidate_tests.rs"]
mod scene_table_candidate_tests;

#[cfg(test)]
#[path = "scene_table_resolve_filter_tests.rs"]
mod scene_table_resolve_filter_tests;

#[cfg(test)]
#[path = "scene_table_pick_tests.rs"]
mod scene_table_pick_tests;
