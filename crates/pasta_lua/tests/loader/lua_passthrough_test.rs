//! Integration tests for Lua passthrough feature.
//!
//! Tests that .lua files placed in dictionary directories are detected,
//! copied to cache without transpilation, and included in scene_dic.lua.

use crate::common;

use common::copy_dir_recursive;
use pasta_lua::loader::{CacheManager, LoaderError, PastaLoader};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Copy scripts/scriptlibs from crate root to temp dir.
fn copy_runtime_deps(base_dir: &std::path::Path) {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    for dir_name in &["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(dir_name);
        let dst = base_dir.join(dir_name);
        if src.exists() {
            fs::create_dir_all(&dst).unwrap();
            copy_dir_recursive(&src, &dst).unwrap();
        }
    }
}

/// Helper: Create a temp dir with pasta.toml and runtime deps.
fn create_test_base() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path();

    fs::write(base_dir.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

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
    fs::write(base_dir.join("dic/test/valid.lua"), "return {}\n").unwrap();

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
fn test_hyphen_normalized_module_name_conflict() {
    // Module names normalize '-' to '_', so "my-mod.pasta" and "my_mod.lua"
    // collide on the same module key -> .pasta wins, .lua is ignored.
    let temp = create_test_base();
    let base_dir = temp.path();

    fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    fs::write(
        base_dir.join("dic/test/my-mod.pasta"),
        "＊ハイフン\n  ゴースト：「パスタ優先」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/test/my_mod.lua"),
        "-- lua side should be ignored\nreturn {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Normalized module appears exactly once in scene_dic.lua
    let scene_dic_path = base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = fs::read_to_string(&scene_dic_path).unwrap();
    let count = scene_dic.matches("pasta.scene.test.my_mod").count();
    assert_eq!(
        count, 1,
        "Normalized module should appear exactly once, found {} in:\n{}",
        count, scene_dic
    );

    // Cached module body comes from the transpiled .pasta, not the raw .lua
    let cache_path = base_dir.join("profile/pasta/cache/lua/pasta/scene/test/my_mod.lua");
    assert!(
        cache_path.exists(),
        "Cache file should exist at: {}",
        cache_path.display()
    );
    let cached = fs::read_to_string(&cache_path).unwrap();
    assert!(
        !cached.contains("-- lua side should be ignored"),
        "Cache must contain transpiled .pasta code, not the conflicting raw .lua"
    );
}

#[test]
fn test_conflict_uses_real_module_names_for_dic_like_dirs() {
    // The conflict check must strip `dic` only at a path component boundary,
    // exactly like the real module names do.
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::write(
        base_dir.join("pasta.toml"),
        "[loader]\ndebug_mode = true\npasta_patterns = [\"dic/**/*.pasta\", \"dicx/**/*.pasta\"]\n",
    )
    .unwrap();

    fs::create_dir_all(base_dir.join("dic/dicx")).unwrap();
    fs::create_dir_all(base_dir.join("dic/x")).unwrap();
    fs::create_dir_all(base_dir.join("dicx")).unwrap();
    // Same real module name `pasta.scene.dicx.a` -> .pasta wins.
    fs::write(
        base_dir.join("dicx/a.pasta"),
        "＊衝突\n  ゴースト：「パスタ優先」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/dicx/a.lua"),
        "-- lua side should be ignored\nreturn {}\n",
    )
    .unwrap();
    // Different real module names (`pasta.scene.x.b` / `pasta.scene.dicx.b`) -> both load.
    fs::write(
        base_dir.join("dic/x/b.pasta"),
        "＊別名\n  ゴースト：「B」\n",
    )
    .unwrap();
    fs::write(base_dir.join("dicx/b.lua"), "-- dicx b\nreturn {}\n").unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    let scene_dic =
        fs::read_to_string(base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua")).unwrap();
    assert_eq!(scene_dic.matches("\"pasta.scene.dicx.a\"").count(), 1);
    assert!(scene_dic.contains("\"pasta.scene.x.b\""));
    assert!(scene_dic.contains("\"pasta.scene.dicx.b\""));
    let cached =
        fs::read_to_string(base_dir.join("profile/pasta/cache/lua/pasta/scene/dicx/a.lua"))
            .unwrap();
    assert!(!cached.contains("-- lua side should be ignored"));
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

/// Load and expect `ModuleNameCollision`; returns the groups as (module, path) pairs.
fn load_expect_collision(base_dir: &std::path::Path) -> (String, Vec<Vec<(String, PathBuf)>>) {
    match PastaLoader::load(base_dir) {
        Err(err @ LoaderError::ModuleNameCollision { .. }) => {
            let msg = err.to_string();
            let LoaderError::ModuleNameCollision { groups } = err else {
                unreachable!()
            };
            let groups = groups
                .into_iter()
                .map(|g| g.into_iter().map(|f| (f.module_name, f.path)).collect())
                .collect();
            (msg, groups)
        }
        Err(other) => panic!("Expected ModuleNameCollision, got: {}", other),
        Ok(_) => panic!("Expected ModuleNameCollision, got Ok"),
    }
}

fn dic(name: &str) -> PathBuf {
    PathBuf::from("dic").join(name)
}

#[test]
fn test_pasta_hyphen_underscore_collision_rejected() {
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(
        base_dir.join("dic/会話-朝.pasta"),
        "＊朝１\n  ゴースト：「一」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/会話_朝.pasta"),
        "＊朝２\n  ゴースト：「二」\n",
    )
    .unwrap();

    let (msg, groups) = load_expect_collision(base_dir);
    let m = "pasta.scene.会話_朝".to_string();
    assert_eq!(
        groups,
        vec![vec![
            (m.clone(), dic("会話-朝.pasta")),
            (m, dic("会話_朝.pasta"))
        ]]
    );
    assert_eq!(
        msg,
        format!(
            "Module name collision: pasta.scene.会話_朝: {}, {}",
            dic("会話-朝.pasta").display(),
            dic("会話_朝.pasta").display()
        )
    );
}

#[test]
fn test_lua_hyphen_dot_collision_rejected() {
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(base_dir.join("dic/x-y.lua"), "return {}\n").unwrap();
    fs::write(base_dir.join("dic/x.y.lua"), "return {}\n").unwrap();

    let (msg, groups) = load_expect_collision(base_dir);
    let m = "pasta.scene.x_y".to_string();
    assert_eq!(
        groups,
        vec![vec![(m.clone(), dic("x-y.lua")), (m, dic("x.y.lua"))]]
    );
    assert!(msg.contains(&dic("x-y.lua").display().to_string()), "{msg}");
    assert!(msg.contains(&dic("x.y.lua").display().to_string()), "{msg}");
}

#[test]
fn test_pasta_case_only_collision_rejected() {
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(base_dir.join("dic/A-b.pasta"), "＊大\n  ゴースト：「A」\n").unwrap();
    fs::write(base_dir.join("dic/a_b.pasta"), "＊小\n  ゴースト：「a」\n").unwrap();

    let (msg, groups) = load_expect_collision(base_dir);
    assert_eq!(
        groups,
        vec![vec![
            ("pasta.scene.A_b".to_string(), dic("A-b.pasta")),
            ("pasta.scene.a_b".to_string(), dic("a_b.pasta")),
        ]]
    );
    assert_eq!(
        msg,
        format!(
            "Module name collision: {} (pasta.scene.A_b), {} (pasta.scene.a_b)",
            dic("A-b.pasta").display(),
            dic("a_b.pasta").display()
        )
    );
}

#[test]
fn test_shadowed_lua_files_collide_with_each_other() {
    // All three are pasta.scene.q_r; the two .lua collide before being shadowed.
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(
        base_dir.join("dic/q-r.pasta"),
        "＊キュー\n  ゴースト：「Q」\n",
    )
    .unwrap();
    fs::write(base_dir.join("dic/q-r.lua"), "return {}\n").unwrap();
    fs::write(base_dir.join("dic/q.r.lua"), "return {}\n").unwrap();

    let (msg, groups) = load_expect_collision(base_dir);
    let m = "pasta.scene.q_r".to_string();
    assert_eq!(
        groups,
        vec![vec![(m.clone(), dic("q-r.lua")), (m, dic("q.r.lua"))]]
    );
    assert!(msg.contains("pasta.scene.q_r"), "{msg}");
}

#[test]
fn test_all_collision_groups_in_one_error() {
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(
        base_dir.join("dic/z-z.pasta"),
        "＊ゼット\n  ゴースト：「Z」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/z_z.pasta"),
        "＊ゼット２\n  ゴースト：「Z」\n",
    )
    .unwrap();
    fs::write(base_dir.join("dic/m-m.lua"), "return {}\n").unwrap();
    fs::write(base_dir.join("dic/m.m.lua"), "return {}\n").unwrap();

    let (msg, groups) = load_expect_collision(base_dir);
    let names: Vec<&str> = groups.iter().map(|g| g[0].0.as_str()).collect();
    assert_eq!(names, vec!["pasta.scene.m_m", "pasta.scene.z_z"]);
    assert!(msg.contains(" / "), "{msg}");
}

#[test]
fn test_pasta_shadows_case_only_different_lua() {
    // .pasta and .lua differing only in case are the same module: .pasta wins.
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(base_dir.join("dic/A-b.pasta"), "＊大\n  ゴースト：「A」\n").unwrap();
    fs::write(
        base_dir.join("dic/a_b.lua"),
        "-- lua side should be ignored\nreturn {}\n",
    )
    .unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    let scene_dic =
        fs::read_to_string(base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua")).unwrap();
    assert!(scene_dic.contains("\"pasta.scene.A_b\""), "{scene_dic}");
    assert!(!scene_dic.contains("\"pasta.scene.a_b\""), "{scene_dic}");
}

#[test]
fn test_non_colliding_japanese_and_subfolders_load() {
    let temp = create_test_base();
    let base_dir = temp.path();
    fs::create_dir_all(base_dir.join("dic/朝")).unwrap();
    fs::create_dir_all(base_dir.join("dic/夜")).unwrap();
    fs::write(
        base_dir.join("dic/朝/会話.pasta"),
        "＊朝\n  ゴースト：「朝」\n",
    )
    .unwrap();
    fs::write(
        base_dir.join("dic/夜/会話.pasta"),
        "＊夜\n  ゴースト：「夜」\n",
    )
    .unwrap();
    fs::write(base_dir.join("dic/朝/道具.lua"), "return {}\n").unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    let scene_dic =
        fs::read_to_string(base_dir.join("profile/pasta/cache/lua/pasta/scene_dic.lua")).unwrap();
    for m in [
        "pasta.scene.朝.会話",
        "pasta.scene.夜.会話",
        "pasta.scene.朝.道具",
    ] {
        assert!(scene_dic.contains(m), "{m} missing in:\n{scene_dic}");
    }
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

/// `dic/` の `.lua` の写しは元とバイトで同じ（複数の行・CRLF・LF の混在を含む）。
/// 止まるエラーの行が元の `.lua` の行になる前提（6.3）。
#[test]
fn test_lua_copy_is_byte_identical_to_source() {
    let temp = create_test_base();
    let base_dir = temp.path();

    let source = b"-- \xe9\x81\x93\xe5\x85\xb7\r\nlocal M = {}\r\n\r\nfunction M.f()\n  return 1\r\nend\n\nreturn M\r\n";
    fs::create_dir_all(base_dir.join("dic")).unwrap();
    fs::write(base_dir.join("dic/道具.lua"), source).unwrap();

    let _runtime = PastaLoader::load(base_dir).unwrap();

    let cached = fs::read(base_dir.join("profile/pasta/cache/lua/pasta/scene/道具.lua")).unwrap();
    assert_eq!(cached, source.to_vec());
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
    fs::write(base_dir.join("dic/orphan/target.lua"), "return {}\n").unwrap();

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
    manager.prepare_cache_dir("").unwrap();

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

// ============================================================================
// load-error-logging: Task 6.2 - process_incremental error propagation tests
// ============================================================================

/// Verify that a broken .pasta file causes PastaLoader::load to return Err.
#[test]
fn test_partial_transpile_error_on_broken_pasta() {
    let temp = create_test_base();
    let base_dir = temp.path();

    // Create a valid .pasta file
    fs::create_dir_all(base_dir.join("dic/ok")).unwrap();
    fs::write(
        base_dir.join("dic/ok/good.pasta"),
        "＊テスト\n  ゴースト：「こんにちは」\n",
    )
    .unwrap();

    // Create a broken .pasta file (invalid syntax)
    fs::create_dir_all(base_dir.join("dic/bad")).unwrap();
    fs::write(
        base_dir.join("dic/bad/broken.pasta"),
        "＊壊れた\n  {{{{invalid syntax}}}}\n",
    )
    .unwrap();

    let result = PastaLoader::load(base_dir);
    assert!(result.is_err(), "Should fail with partial transpile error");
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("Expected error but got Ok"),
    };
    let msg = format!("{}", err);
    assert!(
        msg.contains("Partial transpilation failure"),
        "Error should mention partial transpilation failure: {}",
        msg
    );
    assert!(
        msg.contains("broken.pasta"),
        "Error should contain failed filename: {}",
        msg
    );
}

/// Verify that PartialTranspileError Display includes file paths.
#[test]
fn test_partial_transpile_error_display_includes_paths() {
    use pasta_lua::loader::TranspileFailure;
    use std::path::PathBuf;

    let err = LoaderError::partial_transpile(
        3,
        2,
        vec![
            TranspileFailure {
                source_path: PathBuf::from("dic/talk.pasta"),
                error: "Parse error".to_string(),
            },
            TranspileFailure {
                source_path: PathBuf::from("dic/click.pasta"),
                error: "Transpile error".to_string(),
            },
        ],
    );

    let msg = format!("{}", err);
    assert!(
        msg.contains("3 succeeded"),
        "Should show success count: {}",
        msg
    );
    assert!(
        msg.contains("2 failed"),
        "Should show failure count: {}",
        msg
    );
    assert!(
        msg.contains("dic/talk.pasta"),
        "Should contain first failure path: {}",
        msg
    );
    assert!(
        msg.contains("dic/click.pasta"),
        "Should contain second failure path: {}",
        msg
    );
}
