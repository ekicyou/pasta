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
fn test_cache_cleared_on_load() {
    // Use helper that copies scripts
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();

    // Create cache directory with old file
    std::fs::create_dir_all(base_dir.join("profile/pasta/cache/lua")).unwrap();
    std::fs::write(
        base_dir.join("profile/pasta/cache/lua/old_cache.lua"),
        "-- old cache",
    )
    .unwrap();

    // Load should clear cache
    let _runtime = PastaLoader::load(base_dir).unwrap();

    // Old cache file should be gone
    assert!(
        !base_dir
            .join("profile/pasta/cache/lua/old_cache.lua")
            .exists()
    );

    // New cache file should exist
    assert!(
        base_dir
            .join("profile/pasta/cache/lua/dic_test_hello.lua")
            .exists()
    );
}
