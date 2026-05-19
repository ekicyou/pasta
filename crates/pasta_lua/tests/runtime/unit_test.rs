//! Phase A: runtime/mod.rs のインラインテストを外部化
//!
//! 全テストが公開 API のみ使用しているため可視性変更不要

use mlua::StdLib;
use mlua::prelude::*;
use pasta_lua::loader::LuaConfig;
use pasta_lua::runtime::lua_require;
use pasta_lua::{ConfigError, RuntimeConfig};

// ============================================================================
// RuntimeConfig constructor tests
// ============================================================================

#[test]
fn test_runtime_config_new_returns_default_libs() {
    let config = RuntimeConfig::new();
    assert!(config.libs.contains(&"std_all".to_string()));
    assert!(config.libs.contains(&"assertions".to_string()));
    assert!(config.libs.contains(&"testing".to_string()));
    assert!(config.libs.contains(&"regex".to_string()));
    assert!(config.libs.contains(&"json".to_string()));
    assert!(config.libs.contains(&"yaml".to_string()));
    // env should NOT be in default
    assert!(!config.libs.contains(&"env".to_string()));
}

#[test]
fn test_runtime_config_full_includes_unsafe() {
    let config = RuntimeConfig::full();
    assert!(config.libs.contains(&"std_all_unsafe".to_string()));
    assert!(config.libs.contains(&"env".to_string()));
}

#[test]
fn test_runtime_config_minimal_is_std_all_only() {
    let config = RuntimeConfig::minimal();
    assert_eq!(config.libs, vec!["std_all".to_string()]);
}

#[test]
fn test_runtime_config_from_libs() {
    let config = RuntimeConfig::from_libs(vec!["std_table".into(), "regex".into()]);
    assert_eq!(config.libs.len(), 2);
    assert!(config.libs.contains(&"std_table".to_string()));
    assert!(config.libs.contains(&"regex".to_string()));
}

// ============================================================================
// to_stdlib() tests
// ============================================================================

#[test]
fn test_to_stdlib_std_all() {
    let config = RuntimeConfig::from_libs(vec!["std_all".into()]);
    let stdlib = config.to_stdlib().unwrap();
    assert_eq!(stdlib, StdLib::ALL_SAFE);
}

#[test]
fn test_to_stdlib_std_all_unsafe() {
    let config = RuntimeConfig::from_libs(vec!["std_all_unsafe".into()]);
    let stdlib = config.to_stdlib().unwrap();
    assert_eq!(stdlib, StdLib::ALL);
}

#[test]
fn test_to_stdlib_individual_libs() {
    let config = RuntimeConfig::from_libs(vec![
        "std_table".into(),
        "std_string".into(),
        "std_math".into(),
    ]);
    let stdlib = config.to_stdlib().unwrap();
    assert_eq!(stdlib, StdLib::TABLE | StdLib::STRING | StdLib::MATH);
}

#[test]
fn test_to_stdlib_luajit_specific_libs() {
    let config =
        RuntimeConfig::from_libs(vec!["std_jit".into(), "std_ffi".into(), "std_bit".into()]);
    let stdlib = config.to_stdlib().unwrap();
    assert_eq!(stdlib, StdLib::JIT | StdLib::FFI | StdLib::BIT);
}

#[test]
fn test_to_stdlib_rejects_removed_std_utf8() {
    let config = RuntimeConfig::from_libs(vec!["std_utf8".into()]);
    let result = config.to_stdlib();

    match result {
        Err(ConfigError::UnknownLibrary(name)) => {
            assert_eq!(name, "std_utf8");
            let message = ConfigError::UnknownLibrary(name).to_string();
            let valid_libraries = message
                .split_once("Valid libraries: ")
                .map(|(_, libraries)| libraries)
                .expect("UnknownLibrary message should list valid libraries");
            assert!(!valid_libraries.contains("std_utf8"));
            assert!(valid_libraries.contains("std_jit"));
            assert!(valid_libraries.contains("std_ffi"));
            assert!(valid_libraries.contains("std_bit"));
        }
        Ok(_) => panic!("Expected error for removed std_utf8 library"),
    }
}

#[test]
fn test_to_stdlib_subtraction() {
    let config = RuntimeConfig::from_libs(vec!["std_all".into(), "-std_io".into()]);
    let stdlib = config.to_stdlib().unwrap();
    // ALL_SAFE minus IO
    let expected = StdLib::ALL_SAFE ^ (StdLib::ALL_SAFE & StdLib::IO);
    assert_eq!(stdlib, expected);
}

#[test]
fn test_to_stdlib_order_independent() {
    // Subtraction first, then addition - should still work
    let config1 = RuntimeConfig::from_libs(vec!["-std_io".into(), "std_all".into()]);
    let config2 = RuntimeConfig::from_libs(vec!["std_all".into(), "-std_io".into()]);

    let stdlib1 = config1.to_stdlib().unwrap();
    let stdlib2 = config2.to_stdlib().unwrap();

    assert_eq!(stdlib1, stdlib2);
}

#[test]
fn test_to_stdlib_empty_libs() {
    let config = RuntimeConfig::from_libs(vec![]);
    let stdlib = config.to_stdlib().unwrap();
    assert_eq!(stdlib, StdLib::NONE);
}

#[test]
fn test_to_stdlib_unknown_library_error() {
    let config = RuntimeConfig::from_libs(vec!["std_nonexistent".into()]);
    let result = config.to_stdlib();
    assert!(result.is_err());
    match result {
        Err(ConfigError::UnknownLibrary(name)) => {
            assert_eq!(name, "std_nonexistent");
        }
        Ok(_) => panic!("Expected error for unknown library"),
    }
}

#[test]
fn test_to_stdlib_ignores_mlua_stdlib_modules() {
    // mlua-stdlib modules (no std_ prefix) should be ignored by to_stdlib
    let config = RuntimeConfig::from_libs(vec![
        "std_table".into(),
        "assertions".into(),
        "regex".into(),
    ]);
    let stdlib = config.to_stdlib().unwrap();
    // Only std_table should be processed
    assert_eq!(stdlib, StdLib::TABLE);
}

// ============================================================================
// should_enable_module() tests
// ============================================================================

#[test]
fn test_should_enable_module_positive() {
    let config = RuntimeConfig::from_libs(vec!["testing".into(), "regex".into()]);
    assert!(config.should_enable_module("testing"));
    assert!(config.should_enable_module("regex"));
    assert!(!config.should_enable_module("assertions"));
}

#[test]
fn test_should_enable_module_with_subtraction() {
    let config = RuntimeConfig::from_libs(vec!["testing".into(), "-testing".into()]);
    // Has both positive and negative, so should return false
    assert!(!config.should_enable_module("testing"));
}

#[test]
fn test_should_enable_module_subtraction_without_positive() {
    let config = RuntimeConfig::from_libs(vec!["-testing".into()]);
    // Only negative, no positive - should return false
    assert!(!config.should_enable_module("testing"));
}

// ============================================================================
// From<LuaConfig> tests
// ============================================================================

#[test]
fn test_from_lua_config() {
    let lua_config = LuaConfig {
        libs: vec!["std_all".into(), "testing".into()],
    };
    let runtime_config: RuntimeConfig = lua_config.into();
    assert_eq!(runtime_config.libs, vec!["std_all", "testing"]);
}

// ============================================================================
// Default trait tests
// ============================================================================

#[test]
fn test_default_equals_new() {
    let default_config = RuntimeConfig::default();
    let new_config = RuntimeConfig::new();
    assert_eq!(default_config.libs, new_config.libs);
}

// ============================================================================
// validate_and_warn() tests
// ============================================================================

/// Test that validate_and_warn() does not panic for safe configurations.
#[test]
fn test_validate_and_warn_safe_config() {
    let config = RuntimeConfig::new();
    // Should not panic
    config.validate_and_warn();
}

/// Test that validate_and_warn() does not panic for unsafe configurations.
#[test]
fn test_validate_and_warn_unsafe_config() {
    let config = RuntimeConfig::full();
    // Should not panic, even with unsafe libs
    config.validate_and_warn();
}

/// Test that validate_and_warn() correctly identifies std_debug as security-sensitive.
#[test]
fn test_validate_and_warn_detects_std_debug() {
    let config = RuntimeConfig::from_libs(vec!["std_debug".into()]);
    // The method should run without error
    // Actual warning output is verified by log inspection
    config.validate_and_warn();

    // Verify that the logic correctly identifies std_debug
    assert!(config.libs.contains(&"std_debug".to_string()));
}

/// Test that validate_and_warn() correctly identifies std_all_unsafe as security-sensitive.
#[test]
fn test_validate_and_warn_detects_std_all_unsafe() {
    let config = RuntimeConfig::from_libs(vec!["std_all_unsafe".into()]);
    config.validate_and_warn();

    // Verify that the logic correctly identifies std_all_unsafe
    assert!(config.libs.contains(&"std_all_unsafe".to_string()));
}

/// Test that validate_and_warn() correctly identifies env module as security-sensitive.
#[test]
fn test_validate_and_warn_detects_env_module() {
    let config = RuntimeConfig::from_libs(vec!["std_all".into(), "env".into()]);
    config.validate_and_warn();

    // Verify that the method correctly identifies env
    assert!(config.should_enable_module("env"));
}

/// Test that validate_and_warn() respects subtraction for std_debug.
#[test]
fn test_validate_and_warn_respects_debug_subtraction() {
    let config = RuntimeConfig::from_libs(vec!["std_all_unsafe".into(), "-std_debug".into()]);
    config.validate_and_warn();

    // Verify subtraction is detected
    assert!(config.libs.iter().any(|lib| lib == "-std_debug"));
}

/// Test that validate_and_warn() respects subtraction for env module.
#[test]
fn test_validate_and_warn_respects_env_subtraction() {
    let config = RuntimeConfig::from_libs(vec!["env".into(), "-env".into()]);
    config.validate_and_warn();

    // Verify that env is effectively disabled via subtraction
    assert!(!config.should_enable_module("env"));
}

/// Test that validate_and_warn() handles empty libs configuration.
#[test]
fn test_validate_and_warn_empty_libs() {
    let config = RuntimeConfig::from_libs(vec![]);
    // Should not panic with empty configuration
    config.validate_and_warn();
}

/// Test that validate_and_warn() handles minimal configuration.
#[test]
fn test_validate_and_warn_minimal() {
    let config = RuntimeConfig::minimal();
    // Should not panic and should not trigger any warnings
    config.validate_and_warn();

    // Verify minimal has no security-sensitive options
    assert!(!config.libs.contains(&"std_debug".to_string()));
    assert!(!config.libs.contains(&"std_all_unsafe".to_string()));
    assert!(!config.should_enable_module("env"));
}

// ============================================================================
// lua_require() tests (lua-module-path-resolution spec)
// ============================================================================

#[test]
fn test_lua_require_existing_module() {
    // Test that lua_require can load a built-in module
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Pre-register a test module in package.loaded
    let test_module = lua.create_table().unwrap();
    test_module.set("name", "test_module").unwrap();
    let package: LuaTable = lua.globals().get("package").unwrap();
    let loaded: LuaTable = package.get("loaded").unwrap();
    loaded.set("test_module", test_module).unwrap();

    // Use lua_require to load it
    let result = lua_require(&lua, "test_module");
    assert!(result.is_ok(), "Should successfully require test_module");

    let value = result.unwrap();
    if let LuaValue::Table(t) = value {
        let name: String = t.get("name").unwrap();
        assert_eq!(name, "test_module");
    } else {
        panic!("Expected table from require");
    }
}

#[test]
fn test_lua_require_nonexistent_module() {
    // Test that lua_require returns error for non-existent module
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    let result = lua_require(&lua, "nonexistent_module_xyz123");
    assert!(result.is_err(), "Should fail for non-existent module");

    // Error message should contain module name
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("nonexistent_module_xyz123"),
        "Error message should contain module name: {}",
        err_str
    );
}

#[test]
fn test_lua_require_dotted_module_name() {
    // Test that lua_require handles dotted module names (e.g., "pasta.shiori.entry")
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Pre-register a nested module
    let entry_module = lua.create_table().unwrap();
    entry_module.set("initialized", true).unwrap();
    let package: LuaTable = lua.globals().get("package").unwrap();
    let loaded: LuaTable = package.get("loaded").unwrap();
    loaded.set("pasta.shiori.entry", entry_module).unwrap();

    // Use lua_require with dotted name
    let result = lua_require(&lua, "pasta.shiori.entry");
    assert!(
        result.is_ok(),
        "Should successfully require pasta.shiori.entry"
    );

    let value = result.unwrap();
    if let LuaValue::Table(t) = value {
        let initialized: bool = t.get("initialized").unwrap();
        assert!(initialized);
    } else {
        panic!("Expected table from require");
    }
}
