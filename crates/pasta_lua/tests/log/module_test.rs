//! Unit tests for the @pasta_log module.
//!
//! Tests cover:
//! - All 5 log level functions (trace/debug/info/warn/error)
//! - Value conversion (string, number, boolean, table, nil)
//! - Caller information capture (source, line, function name)
//! - Table JSON conversion with size/depth limits
//! - Edge cases (empty args, circular refs, large tables)

use mlua::{Lua, StdLib, Table};
use tracing_test::traced_test;

/// Create a minimal Lua VM for testing with the @pasta_log module registered.
fn create_test_lua_with_log() -> Lua {
    let lua =
        unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Register the log module
    let log_module = pasta_lua::runtime::log::register(&lua).expect("Failed to register log module");

    // Put it in package.loaded
    let package: Table = lua.globals().get("package").unwrap();
    let loaded: Table = package.get("loaded").unwrap();
    loaded.set("@pasta_log", log_module).unwrap();

    lua
}

// ============================================================================
// Requirement 1.1: 5-level log functions
// ============================================================================

#[traced_test]
#[test]
fn test_log_trace_outputs_message() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.trace("trace message test")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("trace message test"));
}

#[traced_test]
#[test]
fn test_log_debug_outputs_message() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.debug("debug message test")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("debug message test"));
}

#[traced_test]
#[test]
fn test_log_info_outputs_message() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info("info message test")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("info message test"));
}

#[traced_test]
#[test]
fn test_log_warn_outputs_message() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.warn("warn message test")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("warn message test"));
}

#[traced_test]
#[test]
fn test_log_error_outputs_message() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.error("error message test")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("error message test"));
}

// ============================================================================
// Requirement 1.3: String argument
// ============================================================================

#[traced_test]
#[test]
fn test_log_string_value() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info("hello world")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("hello world"));
}

// ============================================================================
// Requirement 1.4: Non-string value conversion
// ============================================================================

#[traced_test]
#[test]
fn test_log_integer_value() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info(123)
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("123"));
}

#[traced_test]
#[test]
fn test_log_number_value() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info(3.14)
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("3.14"));
}

#[traced_test]
#[test]
fn test_log_boolean_true() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info(true)
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("true"));
}

#[traced_test]
#[test]
fn test_log_boolean_false() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info(false)
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("false"));
}

#[traced_test]
#[test]
fn test_log_table_as_json() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info({key = "value"})
    "#)
    .exec()
    .unwrap();

    // Table should be converted to JSON
    assert!(logs_contain("key"));
    assert!(logs_contain("value"));
}

#[traced_test]
#[test]
fn test_log_array_table_as_json() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info({1, 2, 3})
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("[1,2,3]"));
}

// ============================================================================
// Requirement 1.5: nil / no argument
// ============================================================================

#[traced_test]
#[test]
fn test_log_nil_value() {
    let lua = create_test_lua_with_log();
    // Should not error
    lua.load(r#"
        local log = require "@pasta_log"
        log.info(nil)
    "#)
    .exec()
    .unwrap();
}

#[traced_test]
#[test]
fn test_log_no_argument() {
    let lua = create_test_lua_with_log();
    // Calling with no argument should not error
    lua.load(r#"
        local log = require "@pasta_log"
        log.info()
    "#)
    .exec()
    .unwrap();
}

// ============================================================================
// Requirement 1.4: Table size limit (>1000 elements)
// ============================================================================

#[traced_test]
#[test]
fn test_log_large_table_abbreviated() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        local t = {}
        for i = 1, 1001 do
            t[i] = i
        end
        log.info(t)
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("<table:"));
    assert!(logs_contain("elements>"));
}

// ============================================================================
// Requirement 2.1-2.2: Caller information
// ============================================================================

#[traced_test]
#[test]
fn test_log_contains_caller_source() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info("caller test")
    "#)
    .set_name("test_source.lua")
    .exec()
    .unwrap();

    assert!(logs_contain("lua_source="));
}

#[traced_test]
#[test]
fn test_log_contains_caller_line() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info("line test")
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("lua_line="));
}

#[traced_test]
#[test]
fn test_log_contains_caller_fn() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        local function test_func()
            log.info("from function")
        end
        test_func()
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("lua_fn="));
}

// ============================================================================
// Requirement 3.3: Module metadata
// ============================================================================

#[test]
fn test_module_has_version() {
    let lua = create_test_lua_with_log();
    let result: String = lua
        .load(r#"
            local log = require "@pasta_log"
            return log._VERSION
        "#)
        .eval()
        .unwrap();

    assert_eq!(result, "0.1.0");
}

#[test]
fn test_module_has_description() {
    let lua = create_test_lua_with_log();
    let result: String = lua
        .load(r#"
            local log = require "@pasta_log"
            return log._DESCRIPTION
        "#)
        .eval()
        .unwrap();

    assert!(!result.is_empty());
}

// ============================================================================
// Requirement 3.1: Module accessible via require
// ============================================================================

#[test]
fn test_module_accessible_via_require() {
    let lua = create_test_lua_with_log();
    let result: bool = lua
        .load(r#"
            local log = require "@pasta_log"
            return log ~= nil
        "#)
        .eval()
        .unwrap();

    assert!(result);
}

#[test]
fn test_module_has_all_functions() {
    let lua = create_test_lua_with_log();
    let result: bool = lua
        .load(r#"
            local log = require "@pasta_log"
            return type(log.trace) == "function"
                and type(log.debug) == "function"
                and type(log.info) == "function"
                and type(log.warn) == "function"
                and type(log.error) == "function"
        "#)
        .eval()
        .unwrap();

    assert!(result);
}

// ============================================================================
// Edge cases
// ============================================================================

#[traced_test]
#[test]
fn test_log_nested_table_json() {
    let lua = create_test_lua_with_log();
    lua.load(r#"
        local log = require "@pasta_log"
        log.info({nested = {a = 1}})
    "#)
    .exec()
    .unwrap();

    assert!(logs_contain("nested"));
}

#[traced_test]
#[test]
fn test_log_circular_reference_no_error() {
    let lua = create_test_lua_with_log();
    // Circular reference should fall back to tostring(), not error
    lua.load(r#"
        local log = require "@pasta_log"
        local t = {}
        t.self = t
        log.info(t)
    "#)
    .exec()
    .unwrap();

    // Should not crash — tostring fallback
}
