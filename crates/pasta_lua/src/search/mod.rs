//! Pasta Search Module - Lua bindings for scene and word search.
//!
//! This module provides the `@pasta_search` Lua module that exposes
//! scene and word search functionality from pasta_core.
//!
//! # Module Registration
//!
//! Use `register()` to register the module with a Lua instance:
//!
//! ```rust,ignore
//! use pasta_lua::search;
//!
//! let lua = mlua::Lua::new();
//! search::register(&lua, scene_registry, word_registry)?;
//!
//! // Now Lua scripts can use:
//! // local SEARCH = require "@pasta_search"
//! // local global_name, local_name = SEARCH:search_scene("シーン", "親シーン")
//! ```

mod context;
mod error;

pub use context::SearchContext;
pub use error::SearchError;

use mlua::{Lua, Result as LuaResult, Table};
use pasta_core::registry::{SceneRegistry, WordDefRegistry};

/// Create the `@pasta_search` module table.
///
/// This function creates a Lua UserData containing the SearchContext.
/// The SearchContext provides all search methods as Lua methods.
///
/// # Arguments
/// * `lua` - The Lua instance
/// * `scene_registry` - SceneRegistry from transpilation
/// * `word_registry` - WordDefRegistry from transpilation
///
/// # Returns
/// A Lua UserData representing the `@pasta_search` module
pub fn loader(
    lua: &Lua,
    scene_registry: SceneRegistry,
    word_registry: WordDefRegistry,
) -> LuaResult<mlua::AnyUserData> {
    // Create SearchContext from registries
    let context = SearchContext::new(scene_registry, word_registry)?;

    // Return the SearchContext directly as UserData
    // This allows SEARCH:method() calls to work correctly
    lua.create_userdata(context)
}

/// Register the `@pasta_search` module with a Lua instance.
///
/// This function calls `loader()` and registers the result in `package.loaded`
/// so that `require "@pasta_search"` returns the module.
///
/// # Arguments
/// * `lua` - The Lua instance
/// * `scene_registry` - SceneRegistry from transpilation
/// * `word_registry` - WordDefRegistry from transpilation
///
/// # Returns
/// The registered module (UserData)
pub fn register(
    lua: &Lua,
    scene_registry: SceneRegistry,
    word_registry: WordDefRegistry,
) -> LuaResult<mlua::AnyUserData> {
    let module = loader(lua, scene_registry, word_registry)?;

    // Register in package.loaded for require "@pasta_search"
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set("@pasta_search", module.clone())?;

    Ok(module)
}
