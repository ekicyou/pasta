//! Phase A: loader/cache.rs のインラインテストを外部化
//!
//! CURRENT_VERSION: pub に昇格

use pasta_lua::loader::{CacheManager, CURRENT_VERSION};
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
