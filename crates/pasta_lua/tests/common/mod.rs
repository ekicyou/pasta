//! Common test utilities for pasta_lua integration tests.
//!
//! This module provides shared helper functions for E2E testing of the
//! Pasta DSL to Lua transpilation and runtime execution pipeline.

#![allow(dead_code)]

pub mod e2e_helpers;

use pasta_lua::{PastaLuaRuntime, TranspileContext};
use std::path::PathBuf;

/// Normalize line endings to LF (\n) for cross-platform comparison.
/// This handles the case where Git's autocrlf setting converts LF to CRLF on Windows.
pub fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace("\r", "\n")
}

/// Helper to create an empty TranspileContext for testing.
pub fn create_empty_context() -> TranspileContext {
    TranspileContext::new()
}

/// Helper to get the scripts directory path as a Lua-compatible string.
pub fn get_scripts_dir() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/")
}

/// Helper to create a runtime with package.path configured for pasta modules.
pub fn create_runtime_with_pasta_path() -> PastaLuaRuntime {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let scripts_dir = get_scripts_dir();
    runtime
        .exec(&format!(
            r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
        ))
        .expect("Failed to configure package.path");

    // Mock @pasta_persistence module (required by pasta.save which is required by act)
    runtime
        .exec(
            r#"
            package.loaded["@pasta_persistence"] = {
                load = function() return {} end,
                save = function(data) return true end
            }
            "#,
        )
        .expect("Failed to mock @pasta_persistence");

    // Mock @pasta_search module (required by pasta.scene)
    runtime
        .exec(
            r#"
            package.loaded["@pasta_search"] = setmetatable({}, {
                __index = function() return function() return nil end end
            })
            "#,
        )
        .expect("Failed to mock @pasta_search");

    // Mock @pasta_sakura_script module (required by pasta.shiori.sakura_builder)
    runtime
        .exec(
            r#"
            package.loaded["@pasta_sakura_script"] = {
                talk_to_script = function(actor, text) return text or "" end
            }
            "#,
        )
        .expect("Failed to mock @pasta_sakura_script");

    runtime
}

// ============================================================================
// Loader Test Helpers
// ============================================================================

/// Path to loader test fixtures.
pub fn loader_fixtures_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/loader")
        .join(name)
}

/// Recursively copy a directory, skipping profile directories.
pub fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
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

/// Copy fixture to a temporary directory for testing.
/// This avoids permission issues with profile directories in fixtures.
pub fn copy_fixture_to_temp(name: &str) -> tempfile::TempDir {
    let src = loader_fixtures_path(name);
    let temp = tempfile::TempDir::new().unwrap();
    copy_dir_recursive(&src, temp.path()).unwrap();

    // Also copy scripts directory from crate root for pasta runtime modules
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_src = crate_root.join("pasta_scripts");
    let scripts_dst = temp.path().join("pasta_scripts");
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

/// Create a temporary directory with scripts copied and minimal pasta content.
pub fn create_temp_with_pasta(pasta_content: &str) -> tempfile::TempDir {
    let temp = tempfile::TempDir::new().unwrap();
    let base_dir = temp.path();

    // Create minimal dic structure
    std::fs::create_dir_all(base_dir.join("dic/test")).unwrap();
    std::fs::write(base_dir.join("dic/test/hello.pasta"), pasta_content).unwrap();

    // Create minimal pasta.toml
    std::fs::write(base_dir.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    // Copy pasta_scripts directory from crate root for pasta runtime modules
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_src = crate_root.join("pasta_scripts");
    let scripts_dst = base_dir.join("pasta_scripts");
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

/// Helper to extract string from Lua value.
pub fn value_as_str(value: &mlua::Value) -> Option<String> {
    value
        .as_string()
        .and_then(|s| s.to_str().ok())
        .map(|s| s.to_string())
}

// ============================================================================
// Sakura Script Test Helpers
// ============================================================================

use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script;

/// Helper to create a Lua runtime with sakura_script module registered.
pub fn create_sakura_test_runtime() -> mlua::Lua {
    let lua = mlua::Lua::new();
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();

    // Register as @pasta_sakura_script
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    lua
}

/// Helper to create runtime with custom TalkConfig.
pub fn create_sakura_test_runtime_with_config(config: &TalkConfig) -> mlua::Lua {
    let lua = mlua::Lua::new();
    let module = sakura_script::register(&lua, Some(config)).unwrap();

    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    lua
}
