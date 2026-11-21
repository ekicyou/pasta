//! Integration tests for PastaLoader lifecycle: directory prep, init order, user scripts, scene_dic.

use crate::common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::loader::PastaLoader;

// ============================================================================
// Directory Preparation Tests
// ============================================================================

#[test]
fn test_directories_created() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    assert!(base_dir.join("profile/pasta/save").exists());
    assert!(base_dir.join("profile/pasta/save/lua").exists());
    assert!(base_dir.join("profile/pasta/cache").exists());
    assert!(base_dir.join("profile/pasta/cache/lua").exists());
}

#[test]
fn test_cache_incremental_update() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let cache_dir = base_dir.join("profile/pasta/cache/lua");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Write current version to .cache_version (otherwise it will be cleared)
    let version = env!("CARGO_PKG_VERSION");
    std::fs::write(cache_dir.join(".cache_version"), version).unwrap();
    // Write the default scene alias table fingerprint too (otherwise it will be cleared)
    std::fs::write(
        cache_dir.join(".scene_alias"),
        pasta_core::SceneAliasTable::builtin_default().fingerprint(),
    )
    .unwrap();

    std::fs::create_dir_all(cache_dir.join("pasta/scene")).unwrap();

    std::fs::write(cache_dir.join("pasta/scene/orphan.lua"), "-- orphan cache").unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Orphan cache file should still exist (we don't auto-delete)
    assert!(
        cache_dir.join("pasta/scene/orphan.lua").exists(),
        "Orphan cache should be preserved"
    );

    assert!(
        cache_dir.join("pasta/scene_dic.lua").exists(),
        "scene_dic.lua should exist"
    );
}

// ============================================================================
// User Scripts Priority Tests
// ============================================================================

#[test]
fn test_scripts_priority_over_pasta_scripts() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let scripts_dir = base_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    std::fs::write(
        scripts_dir.join("main.lua"),
        r#"
-- User's custom main.lua
_G.MAIN_SOURCE = "scripts"
return { source = "scripts" }
"#,
    )
    .unwrap();

    // pasta_scripts/main.lua exists from copy_dir_recursive, but let's ensure it has different content
    let pasta_scripts_dir = base_dir.join("pasta_scripts");
    std::fs::write(
        pasta_scripts_dir.join("main.lua"),
        r#"
-- Default pasta_scripts/main.lua (should be overridden)
_G.MAIN_SOURCE = "pasta_scripts"
return { source = "pasta_scripts" }
"#,
    )
    .unwrap();

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime.exec("return _G.MAIN_SOURCE").unwrap();
    assert_eq!(
        value_as_str(&result).as_deref(),
        Some("scripts"),
        "scripts/main.lua should have priority over pasta_scripts/main.lua"
    );
}

#[test]
fn test_scripts_module_priority() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let scripts_dir = base_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    std::fs::write(
        scripts_dir.join("test_module.lua"),
        r#"return { source = "scripts", value = 42 }"#,
    )
    .unwrap();

    let pasta_scripts_dir = base_dir.join("pasta_scripts");
    std::fs::write(
        pasta_scripts_dir.join("test_module.lua"),
        r#"return { source = "pasta_scripts", value = 0 }"#,
    )
    .unwrap();

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime
        .exec(r#"return require("test_module").source"#)
        .unwrap();
    assert_eq!(
        value_as_str(&result).as_deref(),
        Some("scripts"),
        "scripts module should have priority"
    );

    let result = runtime
        .exec(r#"return require("test_module").value"#)
        .unwrap();
    assert_eq!(result.as_i64(), Some(42));
}

// ============================================================================
// Default main.lua Tests
// ============================================================================

#[test]
fn test_default_main_lua_fallback() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Don't create scripts/main.lua - should fall back to pasta_scripts/main.lua
    // The default pasta_scripts/main.lua should be copied by copy_dir_recursive

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime.exec("return 'initialized'").unwrap();
    assert_eq!(value_as_str(&result).as_deref(), Some("initialized"));
}

// ============================================================================
// Initialization Order Tests
// ============================================================================

#[test]
fn test_main_lua_executed_before_scene_dic() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let scripts_dir = base_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    std::fs::write(
        scripts_dir.join("main.lua"),
        r#"
-- main.lua executed during initialization
-- Record that we were called (and finalize_scene hasn't been called yet)
_G.MAIN_EXECUTED = true
_G.MAIN_EXECUTION_ORDER = (_G.EXECUTION_ORDER or 0) + 1
_G.EXECUTION_ORDER = _G.MAIN_EXECUTION_ORDER

-- At this point, SCENE API should be available
local SCENE = require("pasta.scene")
_G.SCENE_API_AVAILABLE = type(SCENE) == "table" and type(SCENE.register) == "function"

return {}
"#,
    )
    .unwrap();

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime.exec("return _G.MAIN_EXECUTED").unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "main.lua should have been executed"
    );

    let result = runtime.exec("return _G.SCENE_API_AVAILABLE").unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "pasta.scene API should be available during main.lua execution"
    );
}

#[test]
fn test_dictionary_registration_in_main_lua() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let scripts_dir = base_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    std::fs::write(
        scripts_dir.join("main.lua"),
        r#"
-- Register a word dictionary via pasta.word API
local WORD = require("pasta.word")

-- Register a simple global word using builder pattern
WORD.create_global("custom_greeting"):entry("カスタム挨拶1"):entry("カスタム挨拶2")

_G.DICTIONARY_REGISTERED = true
return {}
"#,
    )
    .unwrap();

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime.exec("return _G.DICTIONARY_REGISTERED").unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "Dictionary should have been registered in main.lua"
    );

    let result = runtime
        .exec(
            r#"
        local WORD = require("pasta.word")
        local all_words = WORD.get_all_words()
        -- Check that custom_greeting exists in global words
        return all_words.global and all_words.global["custom_greeting"] ~= nil
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "Registered word should be found in global words"
    );
}

// ============================================================================
// scene_dic require Tests
// ============================================================================

#[test]
fn test_scene_dic_require_resolution() {
    // Note: scene_dic.lua doesn't return a table - it loads modules and calls finalize_scene()
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime
        .exec(
            r#"
        return package.loaded["pasta.scene_dic"] ~= nil
    "#,
        )
        .unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "pasta.scene_dic should be loaded in package.loaded"
    );
}

#[test]
fn test_scene_dic_new_path() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    let new_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    assert!(
        new_path.exists(),
        "scene_dic.lua should exist at cache/lua/pasta/scene_dic.lua"
    );

    let old_path = base_dir.join("profile/pasta/cache/lua/scene_dic.lua");
    assert!(
        !old_path.exists(),
        "scene_dic.lua should NOT exist at old path cache/lua/scene_dic.lua"
    );
}

#[test]
fn test_scene_module_with_dot_in_file_name() {
    // `require` maps every `.` in a module name to a path separator, so a `.` in the
    // file stem must not survive into the module name (`dic/v1.2.pasta` → `v1_2`).
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();
    std::fs::write(
        base_dir.join("dic/v1.2.pasta"),
        "＊版数\n  ゴースト：「v1.2」\n",
    )
    .unwrap();

    let runtime = PastaLoader::load(base_dir).unwrap();

    let result = runtime
        .exec(r#"return package.loaded["pasta.scene.v1_2"] ~= nil"#)
        .unwrap();
    assert_eq!(result.as_boolean(), Some(true));
    assert!(
        base_dir
            .join("profile/pasta/cache/lua/pasta/scene/v1_2.lua")
            .exists()
    );
}

/// 読み込みが作る生成コードの行の対応は、`.pasta` をゴーストのフォルダーから見た形で持つ。
/// 絶対パスは載せない。
#[test]
fn test_generated_line_map_uses_ghost_relative_pasta_path() {
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();
    let _runtime = PastaLoader::load(base_dir).unwrap();

    let lua = std::fs::read_to_string(
        base_dir.join("profile/pasta/cache/lua/pasta/scene/test/hello.lua"),
    )
    .unwrap();
    let sep = std::path::MAIN_SEPARATOR;
    let init = lua
        .lines()
        .find(|l| l.contains("act:init_scene("))
        .expect("init_scene の行");
    assert!(
        init.contains(&format!("dic{sep}test{sep}hello.pasta|")),
        "{init}"
    );
    assert!(!init.contains(&*base_dir.to_string_lossy()), "{init}");
}
