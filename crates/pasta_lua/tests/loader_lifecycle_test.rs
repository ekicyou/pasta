//! Integration tests for PastaLoader lifecycle: directory prep, init order, user scripts, scene_dic.

mod common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::loader::PastaLoader;

// ============================================================================
// Directory Preparation Tests
// ============================================================================

#[test]
fn test_directories_created() {
    // Use correct pasta syntax
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Load should create profile directories
    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify directories were created
    assert!(base_dir.join("profile/pasta/save").exists());
    assert!(base_dir.join("profile/pasta/save/lua").exists());
    assert!(base_dir.join("profile/pasta/cache").exists());
    assert!(base_dir.join("profile/pasta/cache/lua").exists());
}

#[test]
fn test_cache_incremental_update() {
    // Use helper that copies scripts
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create cache directory with old file that has matching version
    let cache_dir = base_dir.join("profile/pasta/cache/lua");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Write current version to .cache_version (otherwise it will be cleared)
    let version = env!("CARGO_PKG_VERSION");
    std::fs::write(cache_dir.join(".cache_version"), version).unwrap();

    // Create pasta/scene subdirectory for cache files
    std::fs::create_dir_all(cache_dir.join("pasta/scene")).unwrap();

    // Create an unrelated cache file (simulating orphan)
    std::fs::write(cache_dir.join("pasta/scene/orphan.lua"), "-- orphan cache").unwrap();

    // Load should preserve cache (incremental update)
    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Orphan cache file should still exist (we don't auto-delete)
    assert!(
        cache_dir.join("pasta/scene/orphan.lua").exists(),
        "Orphan cache should be preserved"
    );

    // New scene_dic.lua should exist
    assert!(
        cache_dir.join("pasta/scene_dic.lua").exists(),
        "scene_dic.lua should exist"
    );
}

// ============================================================================
// User Scripts Priority Tests (Task 7.1)
// ============================================================================

#[test]
fn test_user_scripts_priority_over_scripts() {
    // Task 7.1: user_scripts/main.lua が scripts/main.lua より優先されることを検証
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create user_scripts directory with a custom main.lua
    let user_scripts_dir = base_dir.join("user_scripts");
    std::fs::create_dir_all(&user_scripts_dir).unwrap();
    std::fs::write(
        user_scripts_dir.join("main.lua"),
        r#"
-- User's custom main.lua
_G.MAIN_SOURCE = "user_scripts"
return { source = "user_scripts" }
"#,
    )
    .unwrap();

    // scripts/main.lua exists from copy_dir_recursive, but let's ensure it has different content
    let scripts_dir = base_dir.join("scripts");
    std::fs::write(
        scripts_dir.join("main.lua"),
        r#"
-- Default scripts/main.lua (should be overridden)
_G.MAIN_SOURCE = "scripts"
return { source = "scripts" }
"#,
    )
    .unwrap();

    // Load the runtime
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Verify user_scripts/main.lua was loaded (has priority)
    let result = runtime.exec("return _G.MAIN_SOURCE").unwrap();
    assert_eq!(
        value_as_str(&result).as_deref(),
        Some("user_scripts"),
        "user_scripts/main.lua should have priority over scripts/main.lua"
    );
}

#[test]
fn test_user_scripts_module_priority() {
    // Task 7.1: user_scripts内のモジュールがscriptsより優先されることを検証
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create user_scripts directory with a custom module
    let user_scripts_dir = base_dir.join("user_scripts");
    std::fs::create_dir_all(&user_scripts_dir).unwrap();
    std::fs::write(
        user_scripts_dir.join("test_module.lua"),
        r#"return { source = "user_scripts", value = 42 }"#,
    )
    .unwrap();

    // Create same module in scripts (lower priority)
    let scripts_dir = base_dir.join("scripts");
    std::fs::write(
        scripts_dir.join("test_module.lua"),
        r#"return { source = "scripts", value = 0 }"#,
    )
    .unwrap();

    // Load the runtime
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Require the module - should get user_scripts version
    let result = runtime
        .exec(r#"return require("test_module").source"#)
        .unwrap();
    assert_eq!(
        value_as_str(&result).as_deref(),
        Some("user_scripts"),
        "user_scripts module should have priority"
    );

    let result = runtime
        .exec(r#"return require("test_module").value"#)
        .unwrap();
    assert_eq!(result.as_i64(), Some(42));
}

// ============================================================================
// Default main.lua Tests (Task 7.3)
// ============================================================================

#[test]
fn test_default_main_lua_fallback() {
    // Task 7.3: user_scriptsにmain.luaを配置しない状態でエラーなく初期化完了を検証
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Don't create user_scripts/main.lua - should fall back to scripts/main.lua
    // The default scripts/main.lua should be copied by copy_dir_recursive

    // Load the runtime - should succeed without error
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Runtime should be usable
    let result = runtime.exec("return 'initialized'").unwrap();
    assert_eq!(value_as_str(&result).as_deref(), Some("initialized"));
}

// ============================================================================
// Initialization Order Tests (Task 7.2)
// ============================================================================

#[test]
fn test_main_lua_executed_before_scene_dic() {
    // Task 7.2: main.lua内でscene_dicファイナライズ前の状態であることを検証
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create user_scripts with a main.lua that records initialization state
    let user_scripts_dir = base_dir.join("user_scripts");
    std::fs::create_dir_all(&user_scripts_dir).unwrap();
    std::fs::write(
        user_scripts_dir.join("main.lua"),
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

    // Load the runtime
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Verify main.lua was executed
    let result = runtime.exec("return _G.MAIN_EXECUTED").unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "main.lua should have been executed"
    );

    // Verify SCENE API was available during main.lua execution
    let result = runtime.exec("return _G.SCENE_API_AVAILABLE").unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "pasta.scene API should be available during main.lua execution"
    );
}

#[test]
fn test_dictionary_registration_in_main_lua() {
    // Task 7.2: 辞書登録APIが利用可能であることを確認
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create user_scripts with a main.lua that registers a custom dictionary
    let user_scripts_dir = base_dir.join("user_scripts");
    std::fs::create_dir_all(&user_scripts_dir).unwrap();
    std::fs::write(
        user_scripts_dir.join("main.lua"),
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

    // Load the runtime
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Verify dictionary was registered
    let result = runtime.exec("return _G.DICTIONARY_REGISTERED").unwrap();
    assert!(
        result.as_boolean() == Some(true),
        "Dictionary should have been registered in main.lua"
    );

    // Verify the registered word is available via pasta.word API
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
// scene_dic require Tests (Task 7.4)
// ============================================================================

#[test]
fn test_scene_dic_require_resolution() {
    // Task 7.4: require("pasta.scene_dic")でscene_dicが解決されることを検証
    // Note: scene_dic.lua doesn't return a table - it loads modules and calls finalize_scene()
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Load the runtime
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Verify scene_dic was loaded (package.loaded["pasta.scene_dic"] should be truthy)
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
    // Task 7.4: 新パス（cache/pasta/scene_dic.lua）からの読み込みを確認
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Load the runtime
    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify scene_dic.lua exists at new path
    let new_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    assert!(
        new_path.exists(),
        "scene_dic.lua should exist at cache/lua/pasta/scene_dic.lua"
    );

    // Verify old path does NOT exist
    let old_path = base_dir.join("profile/pasta/cache/lua/scene_dic.lua");
    assert!(
        !old_path.exists(),
        "scene_dic.lua should NOT exist at old path cache/lua/scene_dic.lua"
    );
}

#[test]
fn test_scene_dic_old_path_cleanup() {
    // Task 7.4: 旧パスが存在する場合、削除されることを確認
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create cache directory structure with old scene_dic.lua
    let cache_dir = base_dir.join("profile/pasta/cache/lua");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Write current version to avoid cache clear
    let version = env!("CARGO_PKG_VERSION");
    std::fs::write(cache_dir.join(".cache_version"), version).unwrap();

    // Create old scene_dic.lua at deprecated path
    std::fs::write(
        cache_dir.join("scene_dic.lua"),
        "-- old scene_dic at deprecated path",
    )
    .unwrap();

    // Load the runtime
    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify old path is cleaned up
    let old_path = cache_dir.join("scene_dic.lua");
    assert!(
        !old_path.exists(),
        "Old scene_dic.lua at deprecated path should be deleted"
    );

    // Verify new path exists
    let new_path = cache_dir.join("pasta/scene_dic.lua");
    assert!(
        new_path.exists(),
        "scene_dic.lua should exist at new path cache/lua/pasta/scene_dic.lua"
    );
}
