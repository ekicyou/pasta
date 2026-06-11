//! SearchContext - UserData for Lua search operations.
//!
//! This module provides the SearchContext struct which manages
//! scene and word search state for each Lua runtime instance.

use super::SearchError;
use mlua::{IntoLuaMulti, MultiValue, UserData, UserDataMethods};
use pasta_core::registry::{
    DefaultRandomSelector, MockRandomSelector, RandomSelector, SceneRegistry, SceneTable,
    WordDefRegistry, WordTable,
};
use std::collections::HashMap;

/// SearchContext - manages search state for a Lua runtime instance.
///
/// Each Lua runtime has its own SearchContext with independent
/// SceneTable and WordTable state. This ensures thread safety
/// and isolation between runtime instances.
pub struct SearchContext {
    scene_table: SceneTable,
    word_table: WordTable,
}

impl SearchContext {
    /// Create a new SearchContext from registries.
    ///
    /// Converts SceneRegistry and WordDefRegistry into runtime tables
    /// with default random selectors.
    pub fn new(
        scene_registry: SceneRegistry,
        word_registry: WordDefRegistry,
    ) -> Result<Self, SearchError> {
        let scene_table = SceneTable::from_scene_registry(
            scene_registry,
            Box::new(DefaultRandomSelector::new()),
        )?;
        let word_table = WordTable::from_word_def_registry(
            word_registry,
            Box::new(DefaultRandomSelector::new()),
        );

        Ok(Self {
            scene_table,
            word_table,
        })
    }

    /// Search for a scene.
    ///
    /// When `global_scene_name` is given, the search is local-only within
    /// that parent scope (no local → global fallback). When it is `None`,
    /// only global scenes are searched.
    ///
    /// # Arguments
    /// * `name` - Search prefix
    /// * `global_scene_name` - Parent scene name (None for global only)
    ///
    /// # Returns
    /// * `Ok(Some((global_name, local_name)))` - Scene found
    /// * `Ok(None)` - No scene found
    /// * `Err(e)` - Internal error
    ///
    /// # Note
    ///
    /// The returned names match the transpiler output format:
    /// - `global_name`: e.g., "メイン1" (from fn_name before "::")
    /// - `local_name`: e.g., "選択肢_1" or "__start__" (Lua function name format)
    pub fn search_scene(
        &mut self,
        name: &str,
        global_scene_name: Option<&str>,
    ) -> Result<Option<(String, String)>, SearchError> {
        let filters = HashMap::new();

        // Determine search strategy based on global_scene_name
        if let Some(parent) = global_scene_name {
            // Local-only search within the parent scope (no global fallback)
            match self
                .scene_table
                .resolve_scene_id_unified(parent, name, &filters)
            {
                Ok(scene_id) => {
                    let scene = self.scene_table.get_scene(scene_id).ok_or_else(|| {
                        SearchError::InvalidArgument("Scene ID not found".to_string())
                    })?;

                    // Extract global_name and local_name from fn_name
                    // fn_name format: "親名_カウンタ::ローカル名" (e.g., "メイン_1::選択肢_1" or "メイン_1::__start__")
                    let (global_name, local_name) = Self::parse_fn_name(&scene.fn_name);
                    Ok(Some((global_name, local_name)))
                }
                Err(
                    pasta_core::SceneTableError::SceneNotFound { .. }
                    | pasta_core::SceneTableError::NoMatchingScene { .. }
                    | pasta_core::SceneTableError::NoMoreScenes { .. },
                ) => Ok(None),
                Err(e) => Err(SearchError::SceneTableError(e)),
            }
        } else {
            // Global search only
            match self.scene_table.resolve_scene_id(name, &filters) {
                Ok(scene_id) => {
                    let scene = self.scene_table.get_scene(scene_id).ok_or_else(|| {
                        SearchError::InvalidArgument("Scene ID not found".to_string())
                    })?;

                    // Extract global_name from fn_name
                    let (global_name, _) = Self::parse_fn_name(&scene.fn_name);
                    Ok(Some((global_name, "__start__".to_string())))
                }
                Err(
                    pasta_core::SceneTableError::SceneNotFound { .. }
                    | pasta_core::SceneTableError::NoMatchingScene { .. }
                    | pasta_core::SceneTableError::NoMoreScenes { .. },
                ) => Ok(None),
                Err(e) => Err(SearchError::SceneTableError(e)),
            }
        }
    }

    /// Parse fn_name to extract global_name and local_name in transpiler output format.
    ///
    /// # Arguments
    /// * `fn_name` - e.g., "メイン_1::選択肢_1" or "メイン_1::__start__"
    ///
    /// # Returns
    /// * `(global_name, local_name)` - e.g., ("メイン_1", "選択肢_1") or ("メイン_1", "__start__")
    fn parse_fn_name(fn_name: &str) -> (String, String) {
        if let Some((global_part, local_part)) = fn_name.split_once("::") {
            let local_name = if local_part == "__start__" {
                "__start__".to_string()
            } else {
                // Return local_part as-is (already in Lua function name format)
                local_part.to_string()
            };
            (global_part.to_string(), local_name)
        } else {
            // Fallback: shouldn't happen with valid fn_name
            (fn_name.to_string(), "__start__".to_string())
        }
    }

    /// Search for a word.
    ///
    /// When `global_scene_name` is given, the search is local-only within
    /// that parent scope (no local → global fallback). When it is `None`,
    /// only global words are searched.
    ///
    /// # Arguments
    /// * `name` - Search key
    /// * `global_scene_name` - Parent scene name (None for global only)
    ///
    /// # Returns
    /// * `Ok(Some(word))` - Word found
    /// * `Ok(None)` - No word found
    /// * `Err(e)` - Internal error
    pub fn search_word(
        &mut self,
        name: &str,
        global_scene_name: Option<&str>,
    ) -> Result<Option<String>, SearchError> {
        let module_name = global_scene_name.unwrap_or("");

        match self.word_table.search_word(module_name, name, &[]) {
            Ok(word) => Ok(Some(word)),
            Err(pasta_core::WordTableError::WordNotFound { .. }) => Ok(None),
        }
    }

    /// Build a selector: mock for a given sequence, default otherwise.
    fn build_selector(sequence: Option<Vec<usize>>) -> Box<dyn RandomSelector> {
        match sequence {
            Some(seq) => Box::new(MockRandomSelector::new(seq)),
            None => Box::new(DefaultRandomSelector::new()),
        }
    }

    /// Set scene selector for deterministic testing.
    ///
    /// # Arguments
    /// * `sequence` - None to reset to default, Some(vec) for mock selector
    pub fn set_scene_selector(&mut self, sequence: Option<Vec<usize>>) -> Result<(), SearchError> {
        self.scene_table.replace_selector(Self::build_selector(sequence));
        Ok(())
    }

    /// Set word selector for deterministic testing.
    ///
    /// # Arguments
    /// * `sequence` - None to reset to default, Some(vec) for mock selector
    pub fn set_word_selector(&mut self, sequence: Option<Vec<usize>>) -> Result<(), SearchError> {
        self.word_table.replace_selector(Self::build_selector(sequence));
        Ok(())
    }
}

/// Parse Lua varargs into a selector sequence (all arguments must be integers).
fn parse_selector_args(args: &MultiValue) -> mlua::Result<Vec<usize>> {
    args.iter()
        .map(|v| {
            v.as_integer()
                .ok_or_else(|| mlua::Error::RuntimeError("expected integer argument".into()))
                .map(|i| i as usize)
        })
        .collect()
}

impl UserData for SearchContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // search_scene(name, global_scene_name?) -> (global_name, local_name) or nil
        methods.add_method_mut(
            "search_scene",
            |lua, this, (name, global_scene_name): (String, Option<String>)| match this
                .search_scene(&name, global_scene_name.as_deref())
            {
                Ok(Some((global, local))) => (global, local).into_lua_multi(lua),
                Ok(None) => Ok(MultiValue::new()),
                Err(e) => Err(mlua::Error::from(e)),
            },
        );

        // search_word(name, global_scene_name?) -> string or nil
        methods.add_method_mut(
            "search_word",
            |lua, this, (name, global_scene_name): (String, Option<String>)| match this
                .search_word(&name, global_scene_name.as_deref())
            {
                Ok(Some(word)) => word.into_lua_multi(lua),
                Ok(None) => Ok(MultiValue::new()),
                Err(e) => Err(mlua::Error::from(e)),
            },
        );

        // set_scene_selector(n1, n2, ...) or set_scene_selector() to reset
        methods.add_method_mut("set_scene_selector", |_lua, this, args: MultiValue| {
            if args.is_empty() {
                this.set_scene_selector(None).map_err(mlua::Error::from)?;
            } else {
                let sequence = parse_selector_args(&args)?;
                this.set_scene_selector(Some(sequence))
                    .map_err(mlua::Error::from)?;
            }
            Ok(())
        });

        // set_word_selector(n1, n2, ...) or set_word_selector() to reset
        methods.add_method_mut("set_word_selector", |_lua, this, args: MultiValue| {
            if args.is_empty() {
                this.set_word_selector(None).map_err(mlua::Error::from)?;
            } else {
                let sequence = parse_selector_args(&args)?;
                this.set_word_selector(Some(sequence))
                    .map_err(mlua::Error::from)?;
            }
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a SearchContext with a small fixed dataset:
    /// - Global scenes: 挨拶 (×2), メイン (with local 選択肢)
    /// - Global words: 場所 = [東京, 大阪]
    /// - Local words (メイン_1): 挨拶 = [やあ]
    fn create_test_search_context() -> SearchContext {
        let mut scene_registry = SceneRegistry::new();
        scene_registry.register_global("挨拶", HashMap::new());
        scene_registry.register_global("挨拶", HashMap::new());
        let (_, counter) = scene_registry.register_global("メイン", HashMap::new());
        scene_registry.register_local("選択肢", "メイン", counter, 1, HashMap::new());

        let mut word_registry = WordDefRegistry::new();
        word_registry.register_global("場所", vec!["東京".to_string(), "大阪".to_string()]);
        word_registry.register_local("メイン_1", "挨拶", vec!["やあ".to_string()]);

        SearchContext::new(scene_registry, word_registry).unwrap()
    }

    #[test]
    fn test_search_scene_global_found() {
        let mut ctx = create_test_search_context();
        // Deterministic order: mock selector disables shuffling.
        ctx.set_scene_selector(Some(vec![0])).unwrap();

        let result = ctx.search_scene("メイン", None).unwrap();
        assert_eq!(
            result,
            Some(("メイン_1".to_string(), "__start__".to_string()))
        );
    }

    #[test]
    fn test_search_scene_not_found_returns_none() {
        let mut ctx = create_test_search_context();
        let result = ctx.search_scene("存在しない", None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_scene_local_found() {
        let mut ctx = create_test_search_context();
        let result = ctx.search_scene("選択肢", Some("メイン_1")).unwrap();
        assert_eq!(
            result,
            Some(("メイン_1".to_string(), "選択肢_1".to_string()))
        );
    }

    #[test]
    fn test_search_scene_with_parent_is_local_only() {
        // Documents actual behavior: when a parent scope is given, the
        // search is local-only — a name that exists only globally is NOT
        // found via fallback (collect_scene_candidates has no fallback).
        let mut ctx = create_test_search_context();
        let result = ctx.search_scene("挨拶", Some("メイン_1")).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_scene_empty_name_is_error() {
        // Empty search key is InvalidScene in pasta_core, which is not a
        // not-found variant and must surface as Err.
        let mut ctx = create_test_search_context();
        let result = ctx.search_scene("", None);
        assert!(matches!(result, Err(SearchError::SceneTableError(_))));
    }

    #[test]
    fn test_search_scene_sequential_no_repeat_with_mock_selector() {
        // With a mock selector (no shuffle), candidates with the same prefix
        // are consumed sequentially without repetition until exhausted.
        let mut ctx = create_test_search_context();
        ctx.set_scene_selector(Some(vec![0])).unwrap();

        let first = ctx.search_scene("挨拶", None).unwrap().unwrap();
        let second = ctx.search_scene("挨拶", None).unwrap().unwrap();
        assert_eq!(first.0, "挨拶_1");
        assert_eq!(second.0, "挨拶_2");

        // Third call wraps around after the cache is exhausted.
        let third = ctx.search_scene("挨拶", None).unwrap().unwrap();
        assert_eq!(third.0, "挨拶_1");
    }

    #[test]
    fn test_search_word_deterministic_with_mock_selector() {
        // Mock selector disables shuffle: words come back in registration
        // order, sequentially, without repetition until exhausted.
        let mut ctx = create_test_search_context();
        ctx.set_word_selector(Some(vec![0])).unwrap();

        let first = ctx.search_word("場所", None).unwrap();
        let second = ctx.search_word("場所", None).unwrap();
        assert_eq!(first, Some("東京".to_string()));
        assert_eq!(second, Some("大阪".to_string()));
    }

    #[test]
    fn test_search_word_local_scope() {
        let mut ctx = create_test_search_context();
        let result = ctx.search_word("挨拶", Some("メイン_1")).unwrap();
        assert_eq!(result, Some("やあ".to_string()));
    }

    #[test]
    fn test_search_word_with_parent_is_local_only() {
        // Documents actual behavior: a global-only word is not visible when
        // a parent scope is specified (no local→global fallback).
        let mut ctx = create_test_search_context();
        let result = ctx.search_word("場所", Some("メイン_1")).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_word_not_found_returns_none() {
        let mut ctx = create_test_search_context();
        let result = ctx.search_word("存在しない単語", None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_selector_reset_to_default_succeeds() {
        let mut ctx = create_test_search_context();
        ctx.set_scene_selector(Some(vec![0])).unwrap();
        ctx.set_word_selector(Some(vec![0])).unwrap();

        // Reset to default selectors; subsequent searches still succeed.
        ctx.set_scene_selector(None).unwrap();
        ctx.set_word_selector(None).unwrap();

        assert!(ctx.search_scene("メイン", None).unwrap().is_some());
        assert!(ctx.search_word("場所", None).unwrap().is_some());
    }
}
