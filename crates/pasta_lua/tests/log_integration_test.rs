//! Integration tests for @pasta_log module registration.
//!
//! These tests verify that @pasta_log is properly integrated into
//! PastaLuaRuntime initialization sequences.

use pasta_lua::context::TranspileContext;
use pasta_lua::runtime::{PastaLuaRuntime, RuntimeConfig};
use tracing_test::traced_test;

/// Verify that @pasta_log module is accessible via require when using
/// PastaLuaRuntime::with_config().
#[test]
fn test_pasta_log_registered_via_with_config() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::new())
        .expect("Runtime creation should succeed");

    let result: bool = runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            return log ~= nil
        "#,
        )
        .eval()
        .unwrap();

    assert!(result, "@pasta_log should be accessible via require");
}

/// Verify that @pasta_log has _VERSION and _DESCRIPTION metadata.
#[test]
fn test_pasta_log_metadata_via_runtime() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::new())
        .expect("Runtime creation should succeed");

    let version: String = runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            return log._VERSION
        "#,
        )
        .eval()
        .unwrap();

    let description: String = runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            return log._DESCRIPTION
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(version, "0.1.0");
    assert!(!description.is_empty());
}

/// Verify that all 5 log level functions exist.
#[test]
fn test_pasta_log_all_functions_via_runtime() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::new())
        .expect("Runtime creation should succeed");

    let result: bool = runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            return type(log.trace) == "function"
                and type(log.debug) == "function"
                and type(log.info) == "function"
                and type(log.warn) == "function"
                and type(log.error) == "function"
        "#,
        )
        .eval()
        .unwrap();

    assert!(result, "All 5 log functions should exist");
}

/// Verify that @pasta_log coexists with other @pasta_* modules.
#[test]
fn test_pasta_log_coexists_with_other_modules() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::new())
        .expect("Runtime creation should succeed");

    // @pasta_search is always registered
    let result: bool = runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            local search = require "@pasta_search"
            return log ~= nil and search ~= nil
        "#,
        )
        .eval()
        .unwrap();

    assert!(result, "@pasta_log and @pasta_search should coexist");
}

/// Verify that @pasta_log works with minimal config (independent of RuntimeConfig.libs).
#[test]
fn test_pasta_log_works_with_minimal_config() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::minimal())
        .expect("Runtime creation should succeed");

    let result: bool = runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            return log ~= nil and type(log.info) == "function"
        "#,
        )
        .eval()
        .unwrap();

    assert!(
        result,
        "@pasta_log should be available even with minimal config"
    );
}

/// Verify that log output works through PastaLuaRuntime.
#[traced_test]
#[test]
fn test_pasta_log_output_via_runtime() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::new())
        .expect("Runtime creation should succeed");

    runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            log.info("integration test message")
        "#,
        )
        .exec()
        .unwrap();

    assert!(logs_contain("integration test message"));
}

/// Verify that PastaLogger not set still works (tracing only).
#[traced_test]
#[test]
fn test_pasta_log_without_pasta_logger() {
    let ctx = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(ctx, RuntimeConfig::new())
        .expect("Runtime creation should succeed");

    // No PastaLogger set, should still work without error
    assert!(runtime.logger().is_none());

    runtime
        .lua()
        .load(
            r#"
            local log = require "@pasta_log"
            log.info("no logger test")
            log.warn("warning without logger")
            log.error("error without logger")
        "#,
        )
        .exec()
        .unwrap();

    assert!(logs_contain("no logger test"));
    assert!(logs_contain("warning without logger"));
    assert!(logs_contain("error without logger"));
}
