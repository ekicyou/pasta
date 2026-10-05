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
    /// `name` is matched with the same rule as registration
    /// (`SceneRegistry::sanitize_name`), so `会話・朝` finds the scene
    /// registered from `＊会話・朝`. `global_scene_name` is a registered name
    /// and is passed through unchanged.
    ///
    /// # Arguments
    /// * `name` - Search prefix (sanitized before matching)
    /// * `global_scene_name` - Parent scene's registered name, not sanitized (None for global only)
    ///
    /// # Returns
    /// * `Ok(Some((global_name, local_name)))` - Scene found
    /// * `Ok(None)` - No scene found
    /// * `Err(e)` - Internal error
    ///
    /// # Note
    ///
    /// The returned names match the transpiler output format:
    /// - `global_name`: e.g., "メイン_1" (from fn_name before "::")
    /// - `local_name`: e.g., "選択肢_1" or "__start__" (Lua function name format)
    pub fn search_scene(
        &mut self,
        name: &str,
        global_scene_name: Option<&str>,
    ) -> Result<Option<(String, String)>, SearchError> {
        let filters = HashMap::new();
        let name = &SceneRegistry::sanitize_name(name);

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
                    // fn_name format: "グローバル名::ローカル名" (e.g. "メイン_1::選択肢_1")
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
            // Global search only (local keys starting with ':' are excluded)
            match self
                .scene_table
                .resolve_scene_id_unified("", name, &filters)
            {
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
    /// `global_scene_name` (the scope) is matched with the same rule as
    /// registration (`SceneRegistry::sanitize_name`), so the actor scope
    /// `__actor_さくら・改__` built by actor.lua finds words registered for
    /// `さくら・改`. `name` (the word key) is passed through unchanged.
    ///
    /// # Arguments
    /// * `name` - Search key, not sanitized
    /// * `global_scene_name` - Scope: parent scene's registered name or actor scope, sanitized before matching (None for global only)
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
        let module_name = global_scene_name
            .map(SceneRegistry::sanitize_name)
            .unwrap_or_default();

        match self.word_table.search_word(&module_name, name, &[]) {
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
        self.scene_table
            .replace_selector(Self::build_selector(sequence));
        Ok(())
    }

    /// Set word selector for deterministic testing.
    ///
    /// # Arguments
    /// * `sequence` - None to reset to default, Some(vec) for mock selector
    pub fn set_word_selector(&mut self, sequence: Option<Vec<usize>>) -> Result<(), SearchError> {
        self.word_table
            .replace_selector(Self::build_selector(sequence));
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
    fn test_search_scene_global_excludes_local_keys() {
        // A global search for a ':'-prefixed name must not hit the local
        // scene registered under ":メイン_1:選択肢_1".
        let mut ctx = create_test_search_context();
        let result = ctx.search_scene(":メイン_1:選択肢", None).unwrap();
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

    /// Registered name of the global scene `会話・朝` (same form before and
    /// after finalize).
    const SYMBOL_GLOBAL: &str = "会話_朝_1";

    /// Build contexts holding a global scene `会話・朝` with a local scene
    /// `選択・A`, in both registry shapes: transpile-time (`register_global` /
    /// `register_local`) and finalized (`register_global_raw`). Both shapes
    /// yield the registered name [`SYMBOL_GLOBAL`].
    fn create_symbol_name_contexts() -> Vec<SearchContext> {
        let mut transpiled = SceneRegistry::new();
        let (_, counter) = transpiled.register_global("会話・朝", HashMap::new());
        transpiled.register_local("選択・A", "会話・朝", counter, 1, HashMap::new());

        let mut finalized = SceneRegistry::new();
        finalized.register_global_raw(
            SYMBOL_GLOBAL,
            &["__start__".to_string(), "選択_A_1".to_string()],
            HashMap::new(),
        );

        [transpiled, finalized]
            .into_iter()
            .map(|r| SearchContext::new(r, WordDefRegistry::new()).unwrap())
            .collect()
    }

    #[test]
    fn test_search_scene_symbol_global_name_and_prefix() {
        let expected = Some((SYMBOL_GLOBAL.to_string(), "__start__".to_string()));
        for mut ctx in create_symbol_name_contexts() {
            assert_eq!(ctx.search_scene("会話・朝", None).unwrap(), expected);
            assert_eq!(ctx.search_scene("会話・", None).unwrap(), expected);
        }
    }

    #[test]
    fn test_search_scene_symbol_local_name_and_prefix() {
        // The parent (registered name) is passed through unchanged.
        let expected = Some((SYMBOL_GLOBAL.to_string(), "選択_A_1".to_string()));
        for mut ctx in create_symbol_name_contexts() {
            let parent = Some(SYMBOL_GLOBAL);
            assert_eq!(ctx.search_scene("選択・A", parent).unwrap(), expected);
            assert_eq!(ctx.search_scene("選択・", parent).unwrap(), expected);
        }
    }

    #[test]
    fn test_search_scene_registered_names_unchanged() {
        // Registered names are already sanitized, so results stay the same.
        for mut ctx in create_symbol_name_contexts() {
            assert_eq!(
                ctx.search_scene(SYMBOL_GLOBAL, None).unwrap(),
                Some((SYMBOL_GLOBAL.to_string(), "__start__".to_string()))
            );
            assert_eq!(
                ctx.search_scene("選択_A_1", Some(SYMBOL_GLOBAL)).unwrap(),
                Some((SYMBOL_GLOBAL.to_string(), "選択_A_1".to_string()))
            );
        }
    }

    #[test]
    fn test_search_scene_overlapping_sanitized_names_share_candidates() {
        // `会話・朝` and `会話_朝` sanitize to the same name, so either name
        // yields both scenes, each once, in sequential consumption. Both
        // registry shapes give the same registered names.
        let build = |finalized: bool| {
            let mut registry = SceneRegistry::new();
            if finalized {
                registry.register_global_raw("会話_朝_1", &[], HashMap::new());
                registry.register_global_raw("会話_朝_2", &[], HashMap::new());
            } else {
                registry.register_global("会話・朝", HashMap::new());
                registry.register_global("会話_朝", HashMap::new());
            }
            let mut ctx = SearchContext::new(registry, WordDefRegistry::new()).unwrap();
            ctx.set_scene_selector(Some(vec![0])).unwrap();
            ctx
        };

        let expected = ["会話_朝_1", "会話_朝_2"];
        for finalized in [false, true] {
            for name in ["会話・朝", "会話_朝"] {
                let mut ctx = build(finalized);
                let mut got: Vec<String> = (0..2)
                    .map(|_| ctx.search_scene(name, None).unwrap().unwrap().0)
                    .collect();
                got.sort();
                assert_eq!(got, expected, "name={name} finalized={finalized}");
            }
        }
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

    /// Build a SearchContext holding only the given actor words
    /// `(actor_name, word_name, value)`, with a mock word selector.
    fn create_actor_word_context(words: &[(&str, &str, &str)]) -> SearchContext {
        let mut word_registry = WordDefRegistry::new();
        for (actor, name, value) in words {
            word_registry.register_actor(actor, name, vec![value.to_string()]);
        }
        let mut ctx = SearchContext::new(SceneRegistry::new(), word_registry).unwrap();
        ctx.set_word_selector(Some(vec![0])).unwrap();
        ctx
    }

    #[test]
    fn test_search_word_actor_scope_with_symbol() {
        // The scope is built by actor.lua from the raw actor name.
        let mut ctx = create_actor_word_context(&[("さくら・改", "通常", "\\s[0]")]);
        let scope = Some("__actor_さくら・改__");
        assert_eq!(
            ctx.search_word("通常", scope).unwrap(),
            Some("\\s[0]".to_string())
        );
        assert_eq!(ctx.search_word("存在しない", scope).unwrap(), None);
    }

    #[test]
    fn test_search_word_actor_scope_colon_not_confused() {
        // Actor `a__:b` word `x` vs actor `a` word `b__:x`: the scope's `:`
        // becomes `_`, the word key is not sanitized.
        let mut ctx = create_actor_word_context(&[("a__:b", "x", "AB_X"), ("a", "b__:x", "A_BX")]);
        assert_eq!(
            ctx.search_word("x", Some("__actor_a__:b__")).unwrap(),
            Some("AB_X".to_string())
        );
        assert_eq!(
            ctx.search_word("b__:x", Some("__actor_a__")).unwrap(),
            Some("A_BX".to_string())
        );
    }

    #[test]
    fn test_search_word_overlapping_actor_names_share_dictionary() {
        // `さくら・改` and `さくら_改` sanitize to the same name, so either
        // actor's scope yields both words, each once, in sequential consumption.
        for actor in ["さくら・改", "さくら_改"] {
            let mut ctx = create_actor_word_context(&[
                ("さくら・改", "通常", "A"),
                ("さくら_改", "通常", "B"),
            ]);
            let scope = format!("__actor_{actor}__");
            let mut got: Vec<String> = (0..2)
                .map(|_| ctx.search_word("通常", Some(&scope)).unwrap().unwrap())
                .collect();
            got.sort();
            assert_eq!(got, ["A", "B"], "actor={actor}");
        }
    }

    #[test]
    fn test_register_then_search_by_raw_name_round_trip() {
        // Registration and search must share one matching rule: whatever
        // the raw name, registering it and searching by the same raw name
        // finds it. Fails if either side alone changes the rule.
        for sym in ["・", "·", "＿", "-", ":", " ", "！"] {
            let global = format!("会話{sym}朝");
            let local = format!("選択{sym}A");
            let actor = format!("さくら{sym}改");

            let mut scenes = SceneRegistry::new();
            let (_, counter) = scenes.register_global(&global, HashMap::new());
            scenes.register_local(&local, &global, counter, 1, HashMap::new());
            let mut words = WordDefRegistry::new();
            words.register_actor(&actor, "通常", vec!["\\s[0]".to_string()]);
            let mut ctx = SearchContext::new(scenes, words).unwrap();

            let (parent, _) = ctx
                .search_scene(&global, None)
                .unwrap()
                .unwrap_or_else(|| panic!("global not found: sym={sym:?}"));
            assert!(
                ctx.search_scene(&local, Some(&parent)).unwrap().is_some(),
                "local not found: sym={sym:?}"
            );
            assert_eq!(
                ctx.search_word("通常", Some(&format!("__actor_{actor}__")))
                    .unwrap(),
                Some("\\s[0]".to_string()),
                "actor word not found: sym={sym:?}"
            );
        }
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
