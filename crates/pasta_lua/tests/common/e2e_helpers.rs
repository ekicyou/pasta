//! E2E test helpers: Lua VM setup and Pasta-to-Lua transpilation.

use mlua::{Lua, StdLib};
use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::context::TranspileContext;
use pasta_lua::loader::PersistenceConfig;
use std::path::PathBuf;

/// Create a minimal Lua runtime with finalize_scene capability.
/// # Example
/// ```ignore
/// use common::e2e_helpers::create_runtime_with_finalize;
///
/// let lua = create_runtime_with_finalize().unwrap();
/// lua.load("-- your code").exec().unwrap();
/// lua.load("require('pasta').finalize_scene()").exec().unwrap();
/// ```
pub fn create_runtime_with_finalize() -> mlua::Result<Lua> {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/");

    lua.load(format!(
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
        pasta_lua::search::SearchContext::new(ctx.scene_registry, ctx.word_registry);
    loaded.set("@pasta_search", lua.create_userdata(search_context)?)?;

    // Register @pasta_log module (required by pasta.shiori.sakura_builder)
    let log_table = pasta_lua::runtime::log::register(&lua)?;
    loaded.set("@pasta_log", log_table)?;

    pasta_lua::runtime::finalize::register_finalize_scene(
        &lua,
        pasta_core::registry::SceneAliasTable::empty(),
    )?;

    Ok(lua)
}

/// Create a Lua runtime with @pasta_search module and scripts path configured.
#[allow(dead_code)]
pub fn create_runtime_with_search(ctx: TranspileContext) -> mlua::Result<Lua> {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/");

    lua.load(format!(
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

    let search_context =
        pasta_lua::search::SearchContext::new(ctx.scene_registry, ctx.word_registry);
    loaded.set("@pasta_search", lua.create_userdata(search_context)?)?;

    // Register @pasta_log module (required by pasta.shiori.sakura_builder)
    let log_table = pasta_lua::runtime::log::register(&lua)?;
    loaded.set("@pasta_log", log_table)?;

    pasta_lua::runtime::finalize::register_finalize_scene(
        &lua,
        pasta_core::registry::SceneAliasTable::empty(),
    )?;

    Ok(lua)
}

/// Transpile Pasta source to Lua code.
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
///
/// # TODO
/// Implement actual scene execution with output collection.
#[allow(dead_code)]
pub fn execute_scene(_lua: &Lua, _scene_name: &str) -> Vec<String> {
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
