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

    /// Search for a scene with fallback strategy (local → global).
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
    /// - `local_name`: e.g., "__選択肢_1__" or "__start__" (Lua function name format)
    pub fn search_scene(
        &mut self,
        name: &str,
        global_scene_name: Option<&str>,
    ) -> Result<Option<(String, String)>, SearchError> {
        let filters = HashMap::new();

        // Determine search strategy based on global_scene_name
        if let Some(parent) = global_scene_name {
            // Try local search first (unified method handles fallback internally)
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
                Err(pasta_core::SceneTableError::SceneNotFound { .. }) => Ok(None),
                Err(pasta_core::SceneTableError::NoMatchingScene { .. }) => Ok(None),
                Err(pasta_core::SceneTableError::NoMoreScenes { .. }) => Ok(None),
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
                Err(pasta_core::SceneTableError::SceneNotFound { .. }) => Ok(None),
                Err(pasta_core::SceneTableError::NoMatchingScene { .. }) => Ok(None),
                Err(pasta_core::SceneTableError::NoMoreScenes { .. }) => Ok(None),
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
    /// * `(global_name, local_name)` - e.g., ("メイン_1", "__選択肢_1__") or ("メイン_1", "__start__")
    fn parse_fn_name(fn_name: &str) -> (String, String) {
        if let Some((global_part, local_part)) = fn_name.split_once("::") {
            let local_name = if local_part == "__start__" {
                "__start__".to_string()
            } else {
                // Convert "選択肢_1" to "__選択肢_1__"
                format!("__{}__", local_part)
            };
            (global_part.to_string(), local_name)
        } else {
            // Fallback: shouldn't happen with valid fn_name
            (fn_name.to_string(), "__start__".to_string())
        }
    }

    /// Search for a word with fallback strategy (local → global).
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

    /// Set scene selector for deterministic testing.
    ///
    /// # Arguments
    /// * `sequence` - None to reset to default, Some(vec) for mock selector
    pub fn set_scene_selector(&mut self, sequence: Option<Vec<usize>>) -> Result<(), SearchError> {
        let selector: Box<dyn RandomSelector> = match sequence {
            Some(seq) => Box::new(MockRandomSelector::new(seq)),
            None => Box::new(DefaultRandomSelector::new()),
        };

        // Recreate scene table with new selector
        // Note: This is a workaround since SceneTable doesn't have replace_selector
        // In a production implementation, we'd add that method to SceneTable
        self.scene_table = SceneTable::new(selector);
        Ok(())
    }

    /// Set word selector for deterministic testing.
    ///
    /// # Arguments
    /// * `sequence` - None to reset to default, Some(vec) for mock selector
    pub fn set_word_selector(&mut self, sequence: Option<Vec<usize>>) -> Result<(), SearchError> {
        let selector: Box<dyn RandomSelector> = match sequence {
            Some(seq) => Box::new(MockRandomSelector::new(seq)),
            None => Box::new(DefaultRandomSelector::new()),
        };

        // Recreate word table with new selector
        self.word_table = WordTable::new(selector);
        Ok(())
    }
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
                let sequence: Result<Vec<usize>, _> = args
                    .iter()
                    .map(|v| {
                        v.as_integer()
                            .ok_or_else(|| {
                                mlua::Error::RuntimeError("expected integer argument".into())
                            })
                            .map(|i| i as usize)
                    })
                    .collect();
                this.set_scene_selector(Some(sequence?))
                    .map_err(mlua::Error::from)?;
            }
            Ok(())
        });

        // set_word_selector(n1, n2, ...) or set_word_selector() to reset
        methods.add_method_mut("set_word_selector", |_lua, this, args: MultiValue| {
            if args.is_empty() {
                this.set_word_selector(None).map_err(mlua::Error::from)?;
            } else {
                let sequence: Result<Vec<usize>, _> = args
                    .iter()
                    .map(|v| {
                        v.as_integer()
                            .ok_or_else(|| {
                                mlua::Error::RuntimeError("expected integer argument".into())
                            })
                            .map(|i| i as usize)
                    })
                    .collect();
                this.set_word_selector(Some(sequence?))
                    .map_err(mlua::Error::from)?;
            }
            Ok(())
        });
    }
}
