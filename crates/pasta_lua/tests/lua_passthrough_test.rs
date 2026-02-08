//! Integration tests for Lua passthrough feature.
//!
//! Tests that .lua files placed in dictionary directories are detected,
//! copied to cache without transpilation, and included in scene_dic.lua.

use pasta_lua::loader::{CacheManager, LoaderError, PastaLoader};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Copy scripts/scriptlibs from crate root to temp dir.
fn copy_runtime_deps(base_dir: &std::path::Path) {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    for dir_name in &["scripts", "scriptlibs"] {
        let src = crate_root.join(dir_name);
        let dst = base_dir.join(dir_name);
        if src.exists() {
            fs::create_dir_all(&dst).unwrap();
            copy_dir_recursive(&src, &dst).unwrap();
        }
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Helper: Create a temp dir with pasta.toml and runtime deps.
fn create_test_base() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path();

    fs::write(
        base_dir.join("pasta.toml"),
        "[loader]\ndebug_mode = true\n",
    )
    .unwrap();

    copy_runtime_deps(base_dir);
    temp
}

/// Helper to extract string from Lua value.
#[allow(dead_code)]
fn value_as_str(value: &mlua::Value) -> Option<String> {
    value
        .as_string()
        .and_then(|s| s.to_str().ok())
        .map(|s| s.to_string())
}

// ============================================================================
// Task 1: init.* file rejection
// ============================================================================

#[test]
fn test_init_lua_rejected() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create dic structure with init.lua
    fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    fs::write(base_dir.join("dic/test/init.lua"), "-- init lua").unwrap();

    let result = PastaLoader::load(base_dir);
    assert!(result.is_err());
    match &result {
        Err(LoaderError::InvalidFileName(path)) => {
            assert!(
                path.to_string_lossy().contains("init.lua"),
                "Expected path containing init.lua, got: {}",
                path.display()
            );
        }
        Err(other) => panic!("Expected InvalidFileName error, got: {}", other),
        Ok(_) => panic!("Expected InvalidFileName error, got Ok"),
    }
}

#[test]
fn test_init_pasta_rejected() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create dic structure with init.pasta
    fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    fs::write(base_dir.join("dic/test/init.pasta"), "# init pasta").unwrap();

    let result = PastaLoader::load(base_dir);
    assert!(result.is_err());
    match &result {
        Err(LoaderError::InvalidFileName(path)) => {
            assert!(
                path.to_string_lossy().contains("init.pasta"),
                "Expected path containing init.pasta, got: {}",
                path.display()
            );
        }
        Err(other) => panic!("Expected InvalidFileName error, got: {}", other),
        Ok(_) => panic!("Expected InvalidFileName error, got Ok"),
    }
}

// ============================================================================
// Task 2: .lua file detection
// ============================================================================

#[test]
fn test_lua_file_detected_and_cached() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create dic structure with a .lua file
    fs::create_dir_all(base_dir.join("dic/utils")).unwrap();
    fs::write(
        base_dir.join("dic/utils/helper.lua"),
        "-- helper lua\nreturn { helper = true }\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify cache file was created
    let cache_path = base_dir.join("profile/pasta/cache/lua/pasta/scene/utils/helper.lua");
    assert!(
        cache_path.exists(),
        "Cache file should exist at: {}",
        cache_path.display()
    );

    // Verify cache content is the original lua (not transpiled)
    let cached = fs::read_to_string(&cache_path).unwrap();
    assert!(cached.contains("-- helper lua"));
    assert!(cached.contains("return { helper = true }"));
}

#[test]
fn test_lua_file_in_scene_dic() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create dic structure with a .lua file
    fs::create_dir_all(base_dir.join("dic/utils")).unwrap();
    fs::write(
        base_dir.join("dic/utils/helper.lua"),
        "-- helper lua\nreturn {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify scene_dic.lua contains require for the .lua module
    let scene_dic_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = fs::read_to_string(&scene_dic_path).unwrap();
    assert!(
        scene_dic.contains("require(\"pasta.scene.utils.helper\")"),
        "scene_dic.lua should contain require for lua module, got:\n{}",
        scene_dic
    );
}

#[test]
fn test_profile_lua_excluded() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create dic structure with a valid .lua file
    fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    fs::write(
        base_dir.join("dic/test/valid.lua"),
        "return {}\n",
    )
    .unwrap();

    // Create profile .lua file (should NOT be picked up)
    fs::create_dir_all(base_dir.join("profile/pasta/some")).unwrap();
    fs::write(
        base_dir.join("profile/pasta/some/internal.lua"),
        "return {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify only dic/test/valid.lua is in scene_dic
    let scene_dic_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = fs::read_to_string(&scene_dic_path).unwrap();
    assert!(scene_dic.contains("pasta.scene.test.valid"));
    assert!(!scene_dic.contains("internal"));
}

// ============================================================================
// Task 3: Module name conflict detection
// ============================================================================

#[test]
fn test_pasta_takes_priority_over_lua_on_conflict() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create dic structure with both .pasta and .lua having same module name
    fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    fs::write(
        base_dir.join("dic/test/conflict.pasta"),
        "＊コンフリクト\n  ゴースト：「パスタ優先」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/test/conflict.lua"),
        "-- this should be ignored\nreturn {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify scene_dic.lua contains the module only once
    let scene_dic_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = fs::read_to_string(&scene_dic_path).unwrap();
    let count = scene_dic.matches("pasta.scene.test.conflict").count();
    assert_eq!(
        count, 1,
        "Module should appear exactly once in scene_dic.lua, found: {}",
        count
    );

    // Verify cached file is from .pasta (transpiled), not raw .lua
    let cache_path = base_dir.join("profile/pasta/cache/lua/pasta/scene/test/conflict.lua");
    let cached = fs::read_to_string(&cache_path).unwrap();
    assert!(
        !cached.contains("-- this should be ignored"),
        "Cache should contain transpiled .pasta code, not raw .lua"
    );
}

#[test]
fn test_no_conflict_different_dirs() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create .pasta in one dir and .lua in another (no conflict)
    fs::create_dir_all(base_dir.join("dic/a")).unwrap();
    fs::create_dir_all(base_dir.join("dic/b")).unwrap();
    fs::write(
        base_dir.join("dic/a/helper.pasta"),
        "＊ヘルパーA\n  ゴースト：「A」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/b/helper.lua"),
        "-- helper B\nreturn {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Both should appear in scene_dic.lua
    let scene_dic_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = fs::read_to_string(&scene_dic_path).unwrap();
    assert!(
        scene_dic.contains("pasta.scene.a.helper"),
        "Should contain a.helper"
    );
    assert!(
        scene_dic.contains("pasta.scene.b.helper"),
        "Should contain b.helper"
    );
}

// ============================================================================
// Task 4: .lua passthrough processing
// ============================================================================

#[test]
fn test_lua_not_transpiled() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create a .lua file with content that would fail parsing as Pasta
    fs::create_dir_all(base_dir.join("dic/raw")).unwrap();
    fs::write(
        base_dir.join("dic/raw/custom.lua"),
        r#"
-- Pure Lua code that is NOT valid Pasta DSL
local M = {}

function M.greet(name)
    return "Hello, " .. name .. "!"
end

return M
"#,
    )
    .unwrap();

    // Should succeed (no parse/transpile attempt on .lua)
    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify cache content matches source exactly
    let cache_path = base_dir.join("profile/pasta/cache/lua/pasta/scene/raw/custom.lua");
    let cached = fs::read_to_string(&cache_path).unwrap();
    assert!(cached.contains("function M.greet(name)"));
    assert!(cached.contains("return M"));
}

#[test]
fn test_pasta_and_lua_mixed() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create both .pasta and .lua (different names, no conflict)
    fs::create_dir_all(base_dir.join("dic/mix")).unwrap();
    fs::write(
        base_dir.join("dic/mix/scene.pasta"),
        "＊ミックス\n  ゴースト：「混在テスト」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/mix/util.lua"),
        "-- utility\nreturn { util = true }\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Both should be in scene_dic
    let scene_dic_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = fs::read_to_string(&scene_dic_path).unwrap();
    assert!(scene_dic.contains("pasta.scene.mix.scene"));
    assert!(scene_dic.contains("pasta.scene.mix.util"));
}

// ============================================================================
// Task 4.3: Incremental update
// ============================================================================

#[test]
fn test_lua_incremental_update() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create .lua file
    fs::create_dir_all(base_dir.join("dic/inc")).unwrap();
    fs::write(
        base_dir.join("dic/inc/module.lua"),
        "return { version = 1 }\n",
    )
    .unwrap();

    // First load
    let _runtime = PastaLoader::load(base_dir).unwrap();

    let cache_path = base_dir.join("profile/pasta/cache/lua/pasta/scene/inc/module.lua");
    let cached_v1 = fs::read_to_string(&cache_path).unwrap();
    assert!(cached_v1.contains("version = 1"));

    // Modify source
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(
        base_dir.join("dic/inc/module.lua"),
        "return { version = 2 }\n",
    )
    .unwrap();

    // Second load
    let _runtime2 = PastaLoader::load(base_dir).unwrap();

    let cached_v2 = fs::read_to_string(&cache_path).unwrap();
    assert!(
        cached_v2.contains("version = 2"),
        "Cache should be updated after source change"
    );
}

// ============================================================================
// Task 5: Orphan cache detection
// ============================================================================

#[test]
fn test_lua_orphan_cache_detected() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create .lua file and load
    fs::create_dir_all(base_dir.join("dic/orphan")).unwrap();
    fs::write(
        base_dir.join("dic/orphan/target.lua"),
        "return {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Verify cache exists
    let cache_path = base_dir.join("profile/pasta/cache/lua/pasta/scene/orphan/target.lua");
    assert!(cache_path.exists(), "Cache should exist after first load");

    // Delete source .lua
    fs::remove_file(base_dir.join("dic/orphan/target.lua")).unwrap();

    // Second load - orphan should be detected (but not deleted)
    let _runtime2 = PastaLoader::load(base_dir).unwrap();

    // Cache file should still exist (orphan detection only, no auto-delete)
    assert!(
        cache_path.exists(),
        "Orphan cache should still exist (not auto-deleted)"
    );
}

// ============================================================================
// Task 5: Orphan detection via CacheManager directly
// ============================================================================

#[test]
fn test_cache_manager_orphan_with_lua_source() {
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path();
    let manager = CacheManager::new(base_dir.to_path_buf(), "profile/pasta/cache/lua");
    manager.prepare_cache_dir().unwrap();

    // Create cache files
    let scene_dir = base_dir.join("profile/pasta/cache/lua/pasta/scene");
    fs::create_dir_all(scene_dir.join("sub")).unwrap();
    fs::write(scene_dir.join("sub/active_pasta.lua"), "-- from pasta").unwrap();
    fs::write(scene_dir.join("sub/active_lua.lua"), "-- from lua").unwrap();
    fs::write(scene_dir.join("sub/orphan.lua"), "-- orphan").unwrap();

    // Source paths include both .pasta and .lua
    let source_paths = vec![
        base_dir.join("dic/sub/active_pasta.pasta"),
        base_dir.join("dic/sub/active_lua.lua"),
    ];

    let orphans = manager.find_orphaned_caches(&source_paths);

    assert_eq!(orphans.len(), 1, "Should find exactly 1 orphan");
    assert!(
        orphans[0].to_string_lossy().contains("orphan"),
        "Orphan should be the unmatched cache file"
    );
}
