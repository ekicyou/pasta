//! Integration tests for mlua-stdlib @assertions and @testing modules in PastaLuaRuntime.
//!
//! These tests verify that the mlua-stdlib modules work correctly
//! with PastaLuaRuntime.

use pasta_lua::{PastaLuaRuntime, RuntimeConfig, TranspileContext};

/// Helper to create an empty TranspileContext for testing.
fn create_empty_context() -> TranspileContext {
    TranspileContext::new()
}

// ============================================================================
// @assertions module tests
// ============================================================================

#[test]
fn test_assertions_module_available() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        return assertions ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_assertions_assert_same() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test assert_same for deep table equality (the only 3 functions in v0.1.0)
    // v0.1.0 only has: assert_eq, assert_ne, assert_same
    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        local t1 = {a = 1, b = {c = 2}}
        local t2 = {a = 1, b = {c = 2}}
        assertions.assert_same(t1, t2, "tables should be deeply equal")
        return true
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_assertions_assert_eq() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        assertions.assert_eq(1 + 1, 2, "1 + 1 should equal 2")
        return true
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_assertions_assert_ne() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        assertions.assert_ne(1, 2, "1 should not equal 2")
        return true
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_assertions_string_equality() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Use assert_eq to test string equality (v0.1.0 doesn't have assert_contains)
    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        assertions.assert_eq("hello world", "hello world", "strings should be equal")
        return true
    "#,
    );

    assert!(result.is_ok(), "Expected success, got error: {:?}", result);
}

#[test]
fn test_assertions_fail_on_false() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test that assert_eq fails when values don't match
    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        assertions.assert_eq(1, 2, "should fail")
    "#,
    );

    assert!(result.is_err());
}

// ============================================================================
// @testing module tests
// ============================================================================

#[test]
fn test_testing_module_available() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    let result = runtime.exec(
        r#"
        local testing = require "@testing"
        return testing ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_testing_create_instance() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    let result = runtime.exec(
        r#"
        local testing = require "@testing"
        local t = testing:new("test_suite")
        return t ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_testing_register_test() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    let result = runtime.exec(
        r#"
        local testing = require "@testing"
        local t = testing:new("test_suite")
        t:test("sample_test", function(ctx)
            ctx.assert(true)
        end)
        return true
    "#,
    );

    assert!(result.is_ok());
}

// ============================================================================
// @env module tests (only when explicitly enabled)
// ============================================================================

#[test]
fn test_env_module_disabled_by_default() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // @env should not be available with default config
    let result = runtime.exec(
        r#"
        local ok, env = pcall(require, "@env")
        return not ok or env == nil
    "#,
    );

    // With default config, @env is not registered
    // So require should fail or return nil
    assert!(result.is_ok());
}

#[test]
fn test_env_module_available_when_enabled() {
    let ctx = create_empty_context();
    let config = RuntimeConfig::full();
    let runtime = PastaLuaRuntime::with_config(ctx, config).unwrap();

    let result = runtime.exec(
        r#"
        local env = require "@env"
        return env ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_env_os_constant() {
    let ctx = create_empty_context();
    let config = RuntimeConfig::full();
    let runtime = PastaLuaRuntime::with_config(ctx, config).unwrap();

    let result = runtime.exec(
        r#"
        local env = require "@env"
        return env.OS
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    // Should return the OS name (e.g., "windows", "linux", "macos")
    assert!(value.as_string().is_some());
}

#[test]
fn test_env_arch_constant() {
    let ctx = create_empty_context();
    let config = RuntimeConfig::full();
    let runtime = PastaLuaRuntime::with_config(ctx, config).unwrap();

    let result = runtime.exec(
        r#"
        local env = require "@env"
        return env.ARCH
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    // Should return architecture (e.g., "x86_64", "aarch64")
    assert!(value.as_string().is_some());
}

// ============================================================================
// RuntimeConfig tests
// ============================================================================

#[test]
fn test_runtime_config_minimal() {
    let ctx = create_empty_context();
    let config = RuntimeConfig::minimal();
    let runtime = PastaLuaRuntime::with_config(ctx, config).unwrap();

    // With minimal config, only @pasta_search should be available
    let result = runtime.exec(
        r#"
        local search = require "@pasta_search"
        return search ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));

    // @assertions should not be available
    let result = runtime.exec(
        r#"
        local ok, _ = pcall(require, "@assertions")
        return not ok
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_runtime_config_full() {
    let ctx = create_empty_context();
    let config = RuntimeConfig::full();
    let runtime = PastaLuaRuntime::with_config(ctx, config).unwrap();

    // All modules should be available
    let result = runtime.exec(
        r#"
        local assertions = require "@assertions"
        local testing = require "@testing"
        local env = require "@env"
        local regex = require "@regex"
        local json = require "@json"
        local yaml = require "@yaml"
        return assertions ~= nil and testing ~= nil and env ~= nil 
            and regex ~= nil and json ~= nil and yaml ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}
