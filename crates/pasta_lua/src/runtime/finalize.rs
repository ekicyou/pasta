//! Finalize Scene Module - Collects Lua-side registries and builds SearchContext.
//!
//! This module implements the `finalize_scene()` function that:
//! 1. Collects scene information from `pasta.scene` Lua registry
//! 2. Collects word definitions from `pasta.word` Lua registry
//! 3. Builds `SceneRegistry` and `WordDefRegistry` from collected data
//! 4. Constructs `SearchContext` and registers it as `@pasta_search` module
//!
//! # Requirements Coverage
//! - Req 1: Lua側シーン情報収集
//! - Req 2: 単語辞書情報収集
//! - Req 3: SearchContext構築・登録
//! - Req 4: Rust-Lua連携メカニズム
//! - Req 6: エラーハンドリング
//! - Req 7: 将来拡張への備え

use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use pasta_core::registry::{SceneRegistry, WordDefRegistry};
use std::collections::HashMap;

/// Entry for collected word information.
#[derive(Debug)]
pub struct WordCollectionEntry {
    /// Word key
    pub key: String,
    /// Values for this word entry
    pub values: Vec<String>,
    /// Whether this is a local word
    pub is_local: bool,
    /// Scene name (for local words)
    pub scene_name: Option<String>,
    /// Actor name (for actor words)
    pub actor_name: Option<String>,
}

/// Collect all scenes from Lua `pasta.scene` registry (Requirement 1.1, 1.2).
///
/// # Arguments
/// * `lua` - Lua instance reference
///
/// # Returns
/// * `Ok(Vec<(global_name, local_name)>)` - Scene information list
/// * `Err(e)` - Collection error
pub fn collect_scenes(lua: &Lua) -> LuaResult<Vec<(String, String)>> {
    // Get pasta.scene module
    let scene_module: Table = lua.load("return require('pasta.scene')").eval()?;

    // Call get_all_scenes()
    let get_all_scenes: Function = scene_module.get("get_all_scenes")?;
    let registry: Table = get_all_scenes.call(())?;

    let mut scenes = Vec::new();

    // Iterate over registry: {global_name: {__global_name__, local_name: func}}
    for pair in registry.pairs::<String, Table>() {
        let (global_name, scene_table) = pair?;

        // Iterate over scene table entries
        for entry in scene_table.pairs::<String, Value>() {
            let (local_name, _value) = entry?;

            // Skip __global_name__ metadata field
            if local_name == "__global_name__" {
                continue;
            }

            scenes.push((global_name.clone(), local_name));
        }
    }

    // Log warning if registry is empty (Requirement 1.3)
    if scenes.is_empty() {
        tracing::warn!("Scene registry is empty");
    }

    Ok(scenes)
}

/// Collect all words from Lua `pasta.word` registry (Requirement 2.6).
///
/// # Arguments
/// * `lua` - Lua instance reference
///
/// # Returns
/// * `Ok(Vec<WordCollectionEntry>)` - Word information list
/// * `Err(e)` - Collection error
pub fn collect_words(lua: &Lua) -> LuaResult<Vec<WordCollectionEntry>> {
    // Get pasta.word module
    let word_module: Table = lua.load("return require('pasta.word')").eval()?;

    // Call get_all_words()
    let get_all_words: Function = word_module.get("get_all_words")?;
    let all_words: Table = get_all_words.call(())?;

    let mut entries = Vec::new();

    // Process global words: {key: [[values]]}
    if let Ok(global_words) = all_words.get::<Table>("global") {
        for pair in global_words.pairs::<String, Table>() {
            let (key, values_list) = pair?;

            // Each key has multiple value arrays
            for values_pair in values_list.pairs::<i64, Table>() {
                let (_idx, values_table) = values_pair?;

                let mut values = Vec::new();
                for val_pair in values_table.pairs::<i64, String>() {
                    let (_i, val) = val_pair?;
                    values.push(val);
                }

                entries.push(WordCollectionEntry {
                    key: key.clone(),
                    values,
                    is_local: false,
                    scene_name: None,
                    actor_name: None,
                });
            }
        }
    }

    // Process local words: {scene_name: {key: [[values]]}}
    if let Ok(local_words) = all_words.get::<Table>("local") {
        for scene_pair in local_words.pairs::<String, Table>() {
            let (scene_name, scene_words) = scene_pair?;

            for key_pair in scene_words.pairs::<String, Table>() {
                let (key, values_list) = key_pair?;

                for values_pair in values_list.pairs::<i64, Table>() {
                    let (_idx, values_table) = values_pair?;

                    let mut values = Vec::new();
                    for val_pair in values_table.pairs::<i64, String>() {
                        let (_i, val) = val_pair?;
                        values.push(val);
                    }

                    entries.push(WordCollectionEntry {
                        key: key.clone(),
                        values,
                        is_local: true,
                        scene_name: Some(scene_name.clone()),
                        actor_name: None,
                    });
                }
            }
        }
    }

    // Process actor words: {actor_name: {key: [[values]]}}
    if let Ok(actor_words) = all_words.get::<Table>("actor") {
        for actor_pair in actor_words.pairs::<String, Table>() {
            let (actor_name, actor_word_map) = actor_pair?;

            for key_pair in actor_word_map.pairs::<String, Table>() {
                let (key, values_list) = key_pair?;

                for values_pair in values_list.pairs::<i64, Table>() {
                    let (_idx, values_table) = values_pair?;

                    let mut values = Vec::new();
                    for val_pair in values_table.pairs::<i64, String>() {
                        let (_i, val) = val_pair?;
                        values.push(val);
                    }

                    entries.push(WordCollectionEntry {
                        key: key.clone(),
                        values,
                        is_local: false, // Actor words are not scene-local
                        scene_name: None,
                        actor_name: Some(actor_name.clone()),
                    });
                }
            }
        }
    }

    Ok(entries)
}

/// Build SceneRegistry from collected scene data (Requirement 1.4, 5.3).
fn build_scene_registry(scenes: &[(String, String)]) -> SceneRegistry {
    let mut registry = SceneRegistry::new();

    // Group by global scene name
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for (global_name, local_name) in scenes {
        grouped
            .entry(global_name.clone())
            .or_default()
            .push(local_name.clone());
    }

    // Register scenes
    for (global_name, local_names) in grouped {
        // Register global scene (counter is already embedded in name)
        let (_, counter) = registry.register_global(&global_name, HashMap::new());

        // Register local scenes
        for (idx, local_name) in local_names.iter().enumerate() {
            // Skip __start__ as it's the default entry point
            if local_name != "__start__" {
                registry.register_local(local_name, &global_name, counter, idx + 1, HashMap::new());
            }
        }
    }

    registry
}

/// Build WordDefRegistry from collected word data (Requirement 2.7, 5.3).
fn build_word_registry(entries: &[WordCollectionEntry]) -> WordDefRegistry {
    let mut registry = WordDefRegistry::new();

    for entry in entries {
        if let Some(ref actor_name) = entry.actor_name {
            // Actor words: register with actor scope
            registry.register_actor(actor_name, &entry.key, entry.values.clone());
        } else if entry.is_local {
            if let Some(ref scene_name) = entry.scene_name {
                registry.register_local(scene_name, &entry.key, entry.values.clone());
            }
        } else {
            registry.register_global(&entry.key, entry.values.clone());
        }
    }

    registry
}

/// Main finalize implementation (Requirement 4.2, 5.5, 6.4, 6.5).
///
/// Collects Lua-side registries and builds SearchContext.
///
/// # Arguments
/// * `lua` - Lua instance reference
///
/// # Returns
/// * `Ok(true)` - Success
/// * `Err(LuaError)` - Failure
pub fn finalize_scene_impl(lua: &Lua) -> LuaResult<bool> {
    // Collect scenes from Lua registry
    let scenes = collect_scenes(lua)?;
    tracing::debug!(
        scene_count = scenes.len(),
        "Collected scenes from Lua registry"
    );

    // Collect words from Lua registry
    let word_entries = collect_words(lua)?;
    tracing::debug!(
        word_count = word_entries.len(),
        "Collected words from Lua registry"
    );

    // Build registries
    let scene_registry = build_scene_registry(&scenes);
    let word_registry = build_word_registry(&word_entries);

    // Register @pasta_search module (Requirement 3.3, 3.4, 5.4)
    crate::search::register(lua, scene_registry, word_registry)?;

    tracing::info!(
        scenes = scenes.len(),
        words = word_entries.len(),
        "SearchContext constructed and registered as @pasta_search"
    );

    Ok(true)
}

/// Register the finalize_scene function binding (Requirement 4.1, 4.3, 4.4, 5.6).
///
/// This function overwrites the stub `PASTA.finalize_scene()` with a Rust implementation.
///
/// # Arguments
/// * `lua` - Lua instance reference
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(e)` - Registration failed
pub fn register_finalize_scene(lua: &Lua) -> LuaResult<()> {
    // Create Rust function
    let finalize_fn = lua.create_function(|lua, ()| finalize_scene_impl(lua))?;

    // Get pasta module from package.loaded
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    // Get or require pasta module
    let pasta_module: Table = if let Ok(module) = loaded.get::<Table>("pasta") {
        module
    } else {
        lua.load("return require('pasta')").eval()?
    };

    // Overwrite finalize_scene with Rust function
    pasta_module.set("finalize_scene", finalize_fn)?;

    tracing::debug!("Registered finalize_scene Rust binding");

    Ok(())
}

// Future extension point (Requirement 7.1, 7.2, 7.3)
// TODO: Add collect_actors() for actor dictionary support
// pub fn collect_actors(lua: &Lua) -> LuaResult<Vec<ActorCollectionEntry>> { ... }
