//! E2E Test Helpers for pasta_lua.
//!
//! This module provides common utilities for Runtime E2E testing:
//! - `create_runtime_with_finalize()` - Create Lua VM with finalize_scene registered
//! - `create_runtime_with_search()` - Create Lua VM with @pasta_search registered
//! - `transpile()` - Transpile Pasta source to Lua code
//! - `execute_scene()` - Execute a scene and collect output (skeleton)
//!
//! # Requirements Coverage
//! - Requirement 7.2: Runtime E2E test infrastructure

use mlua::{Lua, StdLib};
use pasta_core::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::context::TranspileContext;
use pasta_lua::loader::PersistenceConfig;
use std::path::PathBuf;

/// Create a minimal Lua runtime with finalize_scene capability.
///
/// This function:
/// 1. Creates a Lua VM with safe standard libraries
/// 2. Configures package.path to include pasta scripts directory
/// 3. Registers the finalize_scene Rust binding
/// 4. Registers the @pasta_persistence module for save.lua
/// 5. Registers the @pasta_search module with empty registries
///
/// # Returns
/// * `Ok(Lua)` - Configured Lua VM ready for E2E testing
/// * `Err(e)` - Initialization failed
///
/// # Example
/// ```ignore
/// use common::e2e_helpers::create_runtime_with_finalize;
///
/// let lua = create_runtime_with_finalize().unwrap();
/// lua.load("-- your code").exec().unwrap();
/// lua.load("require('pasta').finalize_scene()").exec().unwrap();
/// ```
pub fn create_runtime_with_finalize() -> mlua::Result<Lua> {
    // Create Lua VM with safe standard libraries
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Configure package.path to include pasta scripts directory
    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .to_string_lossy()
        .replace('\\', "/");

    lua.load(&format!(
        r#"
        package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path
        "#
    ))
    .exec()?;

    // Register @pasta_persistence module (required by pasta.save)
    let temp_dir = std::env::temp_dir();
    let persistence_config = PersistenceConfig::default();
    let persistence_table =
        pasta_lua::runtime::persistence::register(&lua, &persistence_config, &temp_dir)?;
    let package: mlua::Table = lua.globals().get("package")?;
    let loaded: mlua::Table = package.get("loaded")?;
    loaded.set("@pasta_persistence", persistence_table)?;

    // Register @pasta_search module with empty registries (required by pasta.scene)
    let ctx = TranspileContext::new();
    let search_context =
        pasta_lua::search::SearchContext::new(ctx.scene_registry, ctx.word_registry)
            .expect("Failed to create SearchContext");
    loaded.set("@pasta_search", lua.create_userdata(search_context)?)?;

    // Register finalize_scene binding
    pasta_lua::runtime::finalize::register_finalize_scene(&lua)?;

    Ok(lua)
}

/// Create a Lua runtime with @pasta_search module and scripts path configured.
///
/// This function:
/// 1. Creates a Lua VM with safe standard libraries
/// 2. Configures package.path to include pasta scripts directory
/// 3. Registers the @pasta_persistence module
/// 4. Registers the @pasta_search module with a custom TranspileContext
/// 5. Registers the finalize_scene Rust binding
///
/// # Arguments
/// * `ctx` - TranspileContext containing scene/word registries for search
///
/// # Returns
/// * `Ok(Lua)` - Configured Lua VM ready for search testing
/// * `Err(e)` - Initialization failed
#[allow(dead_code)]
pub fn create_runtime_with_search(ctx: TranspileContext) -> mlua::Result<Lua> {
    // Create Lua VM with safe standard libraries
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Configure package.path to include pasta scripts directory
    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .to_string_lossy()
        .replace('\\', "/");

    lua.load(&format!(
        r#"
        package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path
        "#
    ))
    .exec()?;

    // Register @pasta_persistence module (required by pasta.save)
    let temp_dir = std::env::temp_dir();
    let persistence_config = PersistenceConfig::default();
    let persistence_table =
        pasta_lua::runtime::persistence::register(&lua, &persistence_config, &temp_dir)?;
    let package: mlua::Table = lua.globals().get("package")?;
    let loaded: mlua::Table = package.get("loaded")?;
    loaded.set("@pasta_persistence", persistence_table)?;

    // Register @pasta_search module
    let search_context =
        pasta_lua::search::SearchContext::new(ctx.scene_registry, ctx.word_registry)
            .expect("Failed to create SearchContext");
    loaded.set("@pasta_search", lua.create_userdata(search_context)?)?;

    // Register finalize_scene binding
    pasta_lua::runtime::finalize::register_finalize_scene(&lua)?;

    Ok(lua)
}

/// Transpile Pasta source to Lua code.
///
/// This function:
/// 1. Parses the Pasta source string
/// 2. Transpiles the AST to Lua code
/// 3. Returns the Lua source as a String
///
/// # Arguments
/// * `source` - Pasta DSL source code
///
/// # Returns
/// * Transpiled Lua code as a String
///
/// # Panics
/// Panics if parsing or transpilation fails (test helper).
///
/// # Example
/// ```ignore
/// use common::e2e_helpers::transpile;
///
/// let lua_code = transpile("＊メイン\n  さくら：こんにちは");
/// assert!(lua_code.contains("PASTA.create_scene"));
/// ```
pub fn transpile(source: &str) -> String {
    let file = parse_str(source, "test.pasta").expect("Failed to parse Pasta source");
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler
        .transpile(&file, &mut output)
        .expect("Failed to transpile Pasta to Lua");
    String::from_utf8(output).expect("Invalid UTF-8 in transpiled Lua")
}

/// Execute a scene and collect output (skeleton implementation).
///
/// This function is a placeholder for future implementation.
/// It will:
/// 1. Call the specified scene function
/// 2. Collect all yielded output
/// 3. Return the output as a Vec<String>
///
/// # Arguments
/// * `_lua` - Lua runtime (unused in skeleton)
/// * `_scene_name` - Scene name to execute (unused in skeleton)
///
/// # Returns
/// * Empty Vec (skeleton implementation)
///
/// # TODO
/// Implement actual scene execution with output collection.
#[allow(dead_code)]
pub fn execute_scene(_lua: &Lua, _scene_name: &str) -> Vec<String> {
    // Skeleton implementation - to be implemented in future tasks
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_runtime_with_finalize_succeeds() {
        let lua = create_runtime_with_finalize();
        assert!(lua.is_ok(), "Runtime creation should succeed");
    }

    #[test]
    fn test_transpile_basic_scene() {
        // 末尾に\nが必要（action_lineはeolで終わる必要がある）
        let source = "＊挨拶\n  さくら：「こんにちは！」\n";
        let lua_code = transpile(source);
        assert!(
            lua_code.contains("create_scene"),
            "Transpiled code should contain create_scene"
        );
    }
}
