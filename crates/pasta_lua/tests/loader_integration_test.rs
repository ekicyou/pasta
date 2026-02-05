//! Integration tests for PastaLoader startup sequence.

use pasta_lua::loader::{LoaderError, PastaConfig, PastaLoader};
use std::path::PathBuf;
use tempfile::TempDir;

fn fixtures_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/loader")
        .join(name)
}

/// Copy fixture to a temporary directory for testing.
/// This avoids permission issues with profile directories in fixtures.
fn copy_fixture_to_temp(name: &str) -> TempDir {
    let src = fixtures_path(name);
    let temp = TempDir::new().unwrap();
    copy_dir_recursive(&src, temp.path()).unwrap();

    // Also copy scripts directory from crate root for pasta runtime modules
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_src = crate_root.join("scripts");
    let scripts_dst = temp.path().join("scripts");
    if scripts_src.exists() {
        std::fs::create_dir_all(&scripts_dst).unwrap();
        copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
    }

    // Also copy scriptlibs directory
    let scriptlibs_src = crate_root.join("scriptlibs");
    let scriptlibs_dst = temp.path().join("scriptlibs");
    if scriptlibs_src.exists() {
        std::fs::create_dir_all(&scriptlibs_dst).unwrap();
        copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
    }

    temp
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            // Skip profile directories
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Helper to extract string from Lua value
fn value_as_str(value: &mlua::Value) -> Option<String> {
    value
        .as_string()
        .and_then(|s| s.to_str().ok())
        .map(|s| s.to_string())
}

// ============================================================================
// Startup Sequence Tests
// ============================================================================

#[test]
fn test_load_minimal() {
    let temp = copy_fixture_to_temp("minimal");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify runtime is usable
    let result = runtime.exec("return 1 + 1").unwrap();
    assert_eq!(result.as_i64(), Some(2));
}

#[test]
fn test_load_with_config() {
    let temp = copy_fixture_to_temp("with_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify runtime is usable
    let result = runtime.exec("return 'hello'").unwrap();
    assert_eq!(value_as_str(&result).as_deref(), Some("hello"));
}

#[test]
fn test_load_with_custom_config() {
    let temp = copy_fixture_to_temp("with_custom_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify @pasta_config module is accessible
    let result = runtime
        .exec(
            r#"
        local config = require("@pasta_config")
        return config.ghost_name
    "#,
        )
        .unwrap();
    assert_eq!(value_as_str(&result).as_deref(), Some("TestGhost"));
}

#[test]
fn test_pasta_config_nested_table() {
    let temp = copy_fixture_to_temp("with_custom_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify nested table access
    let result = runtime
        .exec(
            r#"
        local config = require("@pasta_config")
        return config.user_data.key2
    "#,
        )
        .unwrap();
    assert_eq!(result.as_i64(), Some(42));
}

#[test]
fn test_pasta_config_deeply_nested() {
    let temp = copy_fixture_to_temp("with_custom_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify deeply nested table access
    let result = runtime
        .exec(
            r#"
        local config = require("@pasta_config")
        return config.user_data.nested.inner
    "#,
        )
        .unwrap();
    assert_eq!(value_as_str(&result).as_deref(), Some("data"));
}

#[test]
fn test_pasta_config_excludes_loader() {
    let temp = copy_fixture_to_temp("with_custom_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify [loader] section is NOT in @pasta_config
    let result = runtime
        .exec(
            r#"
        local config = require("@pasta_config")
        return config.loader
    "#,
        )
        .unwrap();
    assert!(result.is_nil());
}

#[test]
fn test_pasta_config_ghost_section() {
    let temp = copy_fixture_to_temp("with_ghost_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify [ghost] section is accessible via pasta.config
    let result = runtime
        .exec(
            r#"
        local config = require("pasta.config")
        return config.get("ghost", "spot_newlines", 1.5)
    "#,
        )
        .unwrap();
    // with_ghost_config/pasta.toml has spot_newlines = 2.0
    assert_eq!(result.as_f64(), Some(2.0));
}

#[test]
fn test_pasta_config_returns_default_for_missing_section() {
    let temp = copy_fixture_to_temp("minimal");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // [ghost] section doesn't exist in minimal fixture
    let result = runtime
        .exec(
            r#"
        local config = require("pasta.config")
        return config.get("ghost", "spot_newlines", 1.5)
    "#,
        )
        .unwrap();
    // Should return default value 1.5
    assert_eq!(result.as_f64(), Some(1.5));
}

#[test]
fn test_shiori_act_uses_config_spot_newlines() {
    let temp = copy_fixture_to_temp("with_ghost_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify SHIORI_ACT uses spot_newlines from config
    let result = runtime
        .exec(
            r#"
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = {
            sakura = { name = "さくら" },
            kero = { name = "うにゅう" },
        }
        local act = SHIORI_ACT.new(actors)
        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)
        act:talk(actors.sakura, "Hello")
        act:talk(actors.kero, "Hi")
        return act:build()
    "#,
        )
        .unwrap();

    let script = value_as_str(&result).unwrap();
    // spot_newlines = 2.0 → \n[200]
    assert!(
        script.contains("\\n[200]"),
        "Expected \\n[200] but got: {}",
        script
    );
}

// ============================================================================
// Package Path Tests
// ============================================================================

#[test]
fn test_package_path_set() {
    let temp = copy_fixture_to_temp("minimal");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // Verify package.path is set
    let result = runtime.exec("return package.path").unwrap();
    let path = value_as_str(&result).unwrap();

    // Should contain all search paths
    assert!(path.contains("profile/pasta/save/lua") || path.contains("profile\\pasta\\save\\lua"));
    assert!(path.contains("scripts"));
    assert!(
        path.contains("profile/pasta/cache/lua") || path.contains("profile\\pasta\\cache\\lua")
    );
    assert!(path.contains("scriptlibs"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_load_nonexistent_directory() {
    // Use a path that definitely doesn't exist on any OS
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("definitely_nonexistent_subdir");

    let result = PastaLoader::load(&nonexistent);

    assert!(result.is_err());
    match result {
        Err(LoaderError::DirectoryNotFound(path)) => {
            assert!(path.to_string_lossy().contains("definitely_nonexistent"));
        }
        _ => panic!("Expected DirectoryNotFound error"),
    }
}

#[test]
fn test_load_empty_dic() {
    // Create a temporary directory with no .pasta files
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path();

    std::fs::create_dir_all(base_dir.join("dic/empty")).unwrap();

    // Create minimal pasta.toml
    std::fs::write(base_dir.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    // Copy scripts directory for pasta module
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_src = crate_root.join("scripts");
    let scripts_dst = base_dir.join("scripts");
    if scripts_src.exists() {
        std::fs::create_dir_all(&scripts_dst).unwrap();
        copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
    }

    // Copy scriptlibs directory
    let scriptlibs_src = crate_root.join("scriptlibs");
    let scriptlibs_dst = base_dir.join("scriptlibs");
    if scriptlibs_src.exists() {
        std::fs::create_dir_all(&scriptlibs_dst).unwrap();
        copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
    }

    // Should succeed but with warning (no files found)
    let runtime = PastaLoader::load(base_dir).unwrap();

    // Runtime should still be usable
    let result = runtime.exec("return 42").unwrap();
    assert_eq!(result.as_i64(), Some(42));
}

// ============================================================================
// Config Loading Tests
// ============================================================================

#[test]
fn test_config_load_not_found() {
    let temp = TempDir::new().unwrap();
    let result = PastaConfig::load(temp.path());

    assert!(result.is_err());
    match result.unwrap_err() {
        LoaderError::ConfigNotFound(path) => {
            assert_eq!(path, temp.path().join("pasta.toml"));
        }
        _ => panic!("Expected ConfigNotFound error"),
    }
}

#[test]
fn test_config_load_with_file() {
    let base_dir = fixtures_path("with_custom_config");
    let config = PastaConfig::load(&base_dir).unwrap();

    assert!(config.loader.debug_mode);
    assert_eq!(
        config.custom_fields.get("ghost_name"),
        Some(&toml::Value::String("TestGhost".to_string()))
    );
}

// ============================================================================
// Directory Preparation Tests
// ============================================================================

/// Create a temporary directory with scripts copied and minimal pasta content.
fn create_temp_with_pasta(pasta_content: &str) -> TempDir {
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path();

    // Create minimal dic structure
    std::fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    std::fs::write(base_dir.join("dic/test/hello.pasta"), pasta_content).unwrap();

    // Create minimal pasta.toml
    std::fs::write(base_dir.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    // Copy scripts directory from crate root for pasta runtime modules
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_src = crate_root.join("scripts");
    let scripts_dst = base_dir.join("scripts");
    if scripts_src.exists() {
        std::fs::create_dir_all(&scripts_dst).unwrap();
        copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
    }

    // Also copy scriptlibs directory
    let scriptlibs_src = crate_root.join("scriptlibs");
    let scriptlibs_dst = base_dir.join("scriptlibs");
    if scriptlibs_src.exists() {
        std::fs::create_dir_all(&scriptlibs_dst).unwrap();
        copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
    }

    temp
}

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
