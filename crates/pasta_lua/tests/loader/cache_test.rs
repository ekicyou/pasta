//! Phase A: loader/cache.rs のインラインテストを外部化
//!
//! CURRENT_VERSION: pub に昇格

use pasta_lua::loader::{CURRENT_VERSION, CacheManager};
use std::fs;
use tempfile::TempDir;

fn create_test_cache_manager() -> (TempDir, CacheManager) {
    let temp = TempDir::new().unwrap();
    let manager = CacheManager::new(temp.path().to_path_buf(), "profile/pasta/cache/lua");
    (temp, manager)
}

// ========================================================================
// Version Management Tests (Task 1.2)
// ========================================================================

#[test]
fn test_prepare_cache_dir_creates_version_file() {
    let (temp, manager) = create_test_cache_manager();

    // First call should create version file
    manager.prepare_cache_dir().unwrap();

    let version_file = temp.path().join("profile/pasta/cache/lua/.cache_version");
    assert!(version_file.exists());
    assert_eq!(fs::read_to_string(&version_file).unwrap(), CURRENT_VERSION);
}

#[test]
fn test_prepare_cache_dir_preserves_cache_on_version_match() {
    let (temp, manager) = create_test_cache_manager();

    // Create initial cache
    manager.prepare_cache_dir().unwrap();

    // Create a test file in cache
    let test_file = temp
        .path()
        .join("profile/pasta/cache/lua/pasta/scene/test.lua");
    fs::write(&test_file, "-- test content").unwrap();

    // Call again - should preserve
    manager.prepare_cache_dir().unwrap();

    // Test file should still exist
    assert!(test_file.exists());
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "-- test content");
}

#[test]
fn test_prepare_cache_dir_clears_on_version_mismatch() {
    let (temp, manager) = create_test_cache_manager();

    // Create initial cache with old version
    let cache_dir = temp.path().join("profile/pasta/cache/lua");
    fs::create_dir_all(cache_dir.join("pasta/scene")).unwrap();
    fs::write(cache_dir.join(".cache_version"), "0.0.0-old").unwrap();

    // Create a test file
    let test_file = cache_dir.join("pasta/scene/test.lua");
    fs::write(&test_file, "-- old content").unwrap();

    // Call prepare - should clear
    manager.prepare_cache_dir().unwrap();

    // Test file should be deleted
    assert!(!test_file.exists());

    // Version file should be updated
    let version_file = cache_dir.join(".cache_version");
    assert_eq!(fs::read_to_string(&version_file).unwrap(), CURRENT_VERSION);
}

// ========================================================================
// Change Detection Tests (Task 2.2)
// ========================================================================

#[test]
fn test_needs_transpile_no_cache() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    // Create source file
    let dic_dir = temp.path().join("dic");
    fs::create_dir_all(&dic_dir).unwrap();
    let source = dic_dir.join("test.pasta");
    fs::write(&source, "# test").unwrap();

    // No cache exists
    assert!(manager.needs_transpile(&source).unwrap());
}

#[test]
fn test_needs_transpile_cache_older() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    // Create source file
    let dic_dir = temp.path().join("dic");
    fs::create_dir_all(&dic_dir).unwrap();
    let source = dic_dir.join("test.pasta");

    // Create cache first (older)
    let cache_path = manager.source_to_cache_path(&source);
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    fs::write(&cache_path, "-- cached").unwrap();

    // Wait a bit and create source (newer)
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(&source, "# updated").unwrap();

    // Source is newer
    assert!(manager.needs_transpile(&source).unwrap());
}

#[test]
fn test_needs_transpile_cache_newer() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    // Create source file first
    let dic_dir = temp.path().join("dic");
    fs::create_dir_all(&dic_dir).unwrap();
    let source = dic_dir.join("test.pasta");
    fs::write(&source, "# test").unwrap();

    // Wait a bit and create cache (newer)
    std::thread::sleep(std::time::Duration::from_millis(50));
    let cache_path = manager.source_to_cache_path(&source);
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    fs::write(&cache_path, "-- cached").unwrap();

    // Cache is newer
    assert!(!manager.needs_transpile(&source).unwrap());
}

// ========================================================================
// Path Conversion Tests (Task 2.3)
// ========================================================================

#[test]
fn test_source_to_module_name_simple() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/system.pasta");
    assert_eq!(manager.source_to_module_name(&source), "pasta.scene.system");
}

#[test]
fn test_source_to_module_name_nested() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/baseware/greet.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.baseware.greet"
    );
}

#[test]
fn test_source_to_module_name_deep_nested() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/dialog/npc/shopkeeper.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.dialog.npc.shopkeeper"
    );
}

#[test]
fn test_source_to_module_name_with_hyphen() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/my-scene.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.my_scene"
    );
}

#[test]
fn test_source_to_module_name_japanese() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/挨拶/朝.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.挨拶.朝"
    );
}

#[test]
fn test_source_to_cache_path_simple() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/system.pasta");
    let expected = temp
        .path()
        .join("profile/pasta/cache/lua/pasta/scene/system.lua");
    assert_eq!(manager.source_to_cache_path(&source), expected);
}

#[test]
fn test_source_to_cache_path_nested() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/baseware/greet.pasta");
    let expected = temp
        .path()
        .join("profile/pasta/cache/lua/pasta/scene/baseware/greet.lua");
    assert_eq!(manager.source_to_cache_path(&source), expected);
}

// ========================================================================
// Cache Save Tests (Task 2.4)
// ========================================================================

#[test]
fn test_save_cache_creates_file() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let source = temp.path().join("dic/test.pasta");
    let lua_code = "-- generated lua code";

    let module_name = manager.save_cache(&source, lua_code).unwrap();

    assert_eq!(module_name, "pasta.scene.test");

    let cache_path = manager.source_to_cache_path(&source);
    assert!(cache_path.exists());
    assert_eq!(fs::read_to_string(&cache_path).unwrap(), lua_code);
}

#[test]
fn test_save_cache_creates_nested_dirs() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let source = temp.path().join("dic/deep/nested/path/scene.pasta");
    let lua_code = "-- nested lua";

    manager.save_cache(&source, lua_code).unwrap();

    let cache_path = manager.source_to_cache_path(&source);
    assert!(cache_path.exists());
}

// ========================================================================
// scene_dic.lua Generation Tests (Task 2.5)
// ========================================================================

#[test]
fn test_generate_scene_dic() {
    let (_temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let modules = vec![
        "pasta.scene.system".to_string(),
        "pasta.scene.baseware.greet".to_string(),
    ];

    let path = manager.generate_scene_dic(&modules).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();

    // Check header
    assert!(content.contains("Auto-generated by pasta_lua CacheManager"));

    // Check sorted requires
    assert!(content.contains("require(\"pasta.scene.baseware.greet\")"));
    assert!(content.contains("require(\"pasta.scene.system\")"));

    // Check finalize_scene call
    assert!(content.contains("require(\"pasta\").finalize_scene()"));

    // Check sorting (baseware should come before system)
    let pos_baseware = content.find("pasta.scene.baseware.greet").unwrap();
    let pos_system = content.find("pasta.scene.system").unwrap();
    assert!(pos_baseware < pos_system);
}

#[test]
fn test_generate_scene_dic_empty() {
    let (_temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let modules: Vec<String> = vec![];
    let path = manager.generate_scene_dic(&modules).unwrap();

    let content = fs::read_to_string(&path).unwrap();

    // Should still have header and finalize_scene
    assert!(content.contains("Auto-generated"));
    assert!(content.contains("require(\"pasta\").finalize_scene()"));
}

// ========================================================================
// Orphan Detection Tests
// ========================================================================

// ========================================================================
// G1 (review-improvement-loop cell 3.14): untested public behavior
// ========================================================================

/// バージョンファイルに前後空白・改行が付いていても trim 比較で一致し、
/// キャッシュが温存されること（prepare_cache_dir の trim 挙動）。
#[test]
fn test_prepare_cache_dir_version_match_with_whitespace() {
    let (temp, manager) = create_test_cache_manager();

    let cache_dir = temp.path().join("profile/pasta/cache/lua");
    fs::create_dir_all(cache_dir.join("pasta/scene")).unwrap();
    // 改行・空白付きで書かれたバージョンファイル（手編集や別ツール由来を想定）
    fs::write(
        cache_dir.join(".cache_version"),
        format!("  {}\n", CURRENT_VERSION),
    )
    .unwrap();

    let test_file = cache_dir.join("pasta/scene/keep.lua");
    fs::write(&test_file, "-- keep me").unwrap();

    manager.prepare_cache_dir().unwrap();

    // trim 一致なのでキャッシュクリアは発生しない
    assert!(
        test_file.exists(),
        "cache must be preserved on trimmed match"
    );
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "-- keep me");
}

/// キャッシュは存在するがソースが消えている場合、needs_transpile は
/// MetadataError を返すこと（メタデータ取得エラーパス）。
#[test]
fn test_needs_transpile_missing_source_errors() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    // ソースは作らず、対応するキャッシュだけ作る
    let source = temp.path().join("dic/ghost.pasta");
    let cache_path = manager.source_to_cache_path(&source);
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    fs::write(&cache_path, "-- cached").unwrap();

    let err = manager.needs_transpile(&source).unwrap_err();
    assert!(
        matches!(err, pasta_lua::loader::LoaderError::MetadataError { .. }),
        "missing source with existing cache must yield MetadataError, got: {err:?}"
    );
}

/// base_dir 外の絶対パスを渡すと、キャッシュパスがキャッシュディレクトリ外へ
/// 解決され、save_cache がディレクトリトラバーサルとして拒否すること。
#[test]
fn test_save_cache_rejects_source_outside_base_dir() {
    let (_temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    // base_dir と無関係な絶対パス（strip_prefix 失敗 → 絶対パス join → 境界外）
    let outside = TempDir::new().unwrap();
    let evil_source = outside.path().join("evil.pasta");

    let err = manager.save_cache(&evil_source, "-- evil").unwrap_err();
    assert!(
        matches!(err, pasta_lua::loader::LoaderError::CacheWriteError { .. }),
        "out-of-base absolute source must be rejected as CacheWriteError, got: {err:?}"
    );
    let msg = format!("{err}");
    assert!(
        msg.contains("Failed to write cache file"),
        "error display should describe cache write failure: {msg}"
    );
    // 境界外への書き込みが発生していないこと
    assert!(!outside.path().join("evil.lua").exists());
}

/// 同一ソースへの save_cache 再実行は既存キャッシュを上書きすること（増分更新）。
#[test]
fn test_save_cache_overwrites_existing() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let source = temp.path().join("dic/test.pasta");
    manager.save_cache(&source, "-- version 1").unwrap();
    let module_name = manager.save_cache(&source, "-- version 2").unwrap();

    assert_eq!(module_name, "pasta.scene.test");
    let cache_path = manager.source_to_cache_path(&source);
    assert_eq!(fs::read_to_string(&cache_path).unwrap(), "-- version 2");
}

/// dic/ プレフィックスを持たないソースでもモジュール名へ変換できること
/// （strip_prefix 不一致時のフォールバック挙動）。
#[test]
fn test_source_to_module_name_without_dic_prefix() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("scripts/helper.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.scripts.helper"
    );
}

/// ハイフン入りソースのキャッシュパスはアンダースコアへ正規化されること
/// （source_to_module_name 側のみ既存テストあり、パス側を補完）。
#[test]
fn test_source_to_cache_path_hyphen_normalized() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dic/my-scene.pasta");
    let expected = temp
        .path()
        .join("profile/pasta/cache/lua/pasta/scene/my_scene.lua");
    assert_eq!(manager.source_to_cache_path(&source), expected);
}

/// scene ディレクトリ未生成時、find_orphaned_caches は空を返すこと（早期 return）。
#[test]
fn test_find_orphaned_caches_empty_when_no_scene_dir() {
    let (temp, manager) = create_test_cache_manager();
    // prepare_cache_dir を呼ばない＝ scene ディレクトリなし

    let sources = vec![temp.path().join("dic/a.pasta")];
    assert!(manager.find_orphaned_caches(&sources).is_empty());
}

/// orphan 走査は scene_dic.lua と非 .lua ファイルをスキップし、
/// ネストした orphan は検出すること（unix 限定の内部テストの Windows 互換版）。
#[test]
fn test_find_orphaned_caches_skips_scene_dic_and_non_lua() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let scene_dir = temp.path().join("profile/pasta/cache/lua/pasta/scene");
    fs::write(scene_dir.join("scene_dic.lua"), "-- generated").unwrap();
    fs::write(scene_dir.join("readme.txt"), "not lua").unwrap();
    fs::create_dir_all(scene_dir.join("nested")).unwrap();
    fs::write(scene_dir.join("nested/orphan.lua"), "-- nested orphan").unwrap();

    // ソースは一切ない → .lua のうち scene_dic.lua 以外がすべて orphan
    let orphans = manager.find_orphaned_caches(&[]);

    assert_eq!(
        orphans.len(),
        1,
        "only nested/orphan.lua expected: {orphans:?}"
    );
    assert!(orphans[0].ends_with("orphan.lua"));
}

/// 旧配置（cache_dir 直下）の scene_dic.lua が generate_scene_dic で
/// 後方互換クリーンアップされ、新配置 pasta/ 配下に生成されること。
#[test]
fn test_generate_scene_dic_removes_legacy_location() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let cache_dir = temp.path().join("profile/pasta/cache/lua");
    let legacy = cache_dir.join("scene_dic.lua");
    fs::write(&legacy, "-- legacy location").unwrap();

    let path = manager
        .generate_scene_dic(&["pasta.scene.a".to_string()])
        .unwrap();

    assert!(!legacy.exists(), "legacy scene_dic.lua must be removed");
    assert_eq!(path, cache_dir.join("pasta/scene_dic.lua"));
    assert!(path.exists());
}

/// cache_dir() アクセサが base_dir + output_dir を指すこと。
#[test]
fn test_cache_dir_accessor() {
    let (temp, manager) = create_test_cache_manager();
    assert_eq!(
        manager.cache_dir(),
        temp.path().join("profile/pasta/cache/lua")
    );
}

// ========================================================================
// G3 (review-improvement-loop cell 3.15): hardening regression tests
// ========================================================================

/// `..` を含む相対ソースパスはトラバーサルとして拒否されること。
/// 旧実装のガード（`Path::starts_with` のみ）は字句比較のため
/// `dic/../../../evil.pasta` → `{cache_dir}/pasta/scene/../../../evil.lua` が
/// ガードを素通りし、OS 解決で cache_dir 外（cache/ 直下）へ書込まれていた。
#[test]
fn test_save_cache_rejects_parent_traversal_in_relative_source() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    let evil_source = std::path::Path::new("dic/../../../evil.pasta");
    let result = manager.save_cache(evil_source, "-- evil");

    assert!(
        matches!(
            result,
            Err(pasta_lua::loader::LoaderError::CacheWriteError { .. })
        ),
        "traversal source must be rejected as CacheWriteError, got: {result:?}"
    );
    // OS 解決後の脱出先（cache_dir 外 = cache/ 直下）にファイルが作られていないこと
    let escaped = temp.path().join("profile/pasta/cache/evil.lua");
    assert!(
        !escaped.exists(),
        "no file must be written outside cache_dir"
    );
}

/// dic 接頭辞の剥離はパスコンポーネント境界でのみ行うこと（モジュール名側）。
/// 旧実装は `str::strip_prefix("dic")` のため `dictionary.pasta` →
/// `pasta.scene.tionary` と誤剥離していた。
#[test]
fn test_source_to_module_name_dic_like_file_not_stripped() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dictionary.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.dictionary"
    );
}

/// dic 接頭辞の剥離はパスコンポーネント境界でのみ行うこと（ディレクトリ名類似）。
/// 旧実装は `dicx/foo.pasta` → `pasta.scene.x.foo` と誤剥離していた。
#[test]
fn test_source_to_module_name_dic_like_dir_not_stripped() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dicx/foo.pasta");
    assert_eq!(
        manager.source_to_module_name(&source),
        "pasta.scene.dicx.foo"
    );
}

/// dic 接頭辞の剥離はパスコンポーネント境界でのみ行うこと（キャッシュパス側）。
#[test]
fn test_source_to_cache_path_dic_like_file_not_stripped() {
    let (temp, manager) = create_test_cache_manager();
    let source = temp.path().join("dictionary.pasta");
    let expected = temp
        .path()
        .join("profile/pasta/cache/lua/pasta/scene/dictionary.lua");
    assert_eq!(manager.source_to_cache_path(&source), expected);
}

#[test]
fn test_find_orphaned_caches() {
    let (temp, manager) = create_test_cache_manager();
    manager.prepare_cache_dir().unwrap();

    // Create some cache files
    let scene_dir = temp.path().join("profile/pasta/cache/lua/pasta/scene");
    fs::write(scene_dir.join("active.lua"), "-- active").unwrap();
    fs::write(scene_dir.join("orphan.lua"), "-- orphan").unwrap();

    // Only active.pasta exists
    let source_paths = vec![temp.path().join("dic/active.pasta")];

    let orphans = manager.find_orphaned_caches(&source_paths);

    assert_eq!(orphans.len(), 1);
    assert!(orphans[0].ends_with("orphan.lua"));
}
