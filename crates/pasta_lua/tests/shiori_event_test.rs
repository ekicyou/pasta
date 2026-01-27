//! Integration tests for pasta.shiori.event module (SHIORI event dispatching).
//!
//! Tests verify that the EVENT module correctly dispatches SHIORI events to registered handlers.

use pasta_lua::{PastaLuaRuntime, TranspileContext};
use std::path::PathBuf;

/// Helper to create an empty TranspileContext for testing.
fn create_empty_context() -> TranspileContext {
    TranspileContext::new()
}

/// Helper to get the scripts directory path as a Lua-compatible string.
fn get_scripts_dir() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .to_string_lossy()
        .replace('\\', "/")
}

/// Helper to create a runtime with package.path configured for pasta modules.
fn create_runtime_with_pasta_path() -> PastaLuaRuntime {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let scripts_dir = get_scripts_dir();
    runtime
        .exec(&format!(
            r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
        ))
        .expect("Failed to configure package.path");
    runtime
}

// ============================================================================
// Task 1.1, 3.1: REG Module Tests
// ============================================================================

#[test]
fn test_reg_module_loads() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        return REG ~= nil
    "#,
    );

    assert!(result.is_ok(), "REG module should load: {:?}", result);
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_reg_module_exports_empty_table() {
    let runtime = create_runtime_with_pasta_path();

    // Note: Before EVENT module is loaded, REG should be empty.
    // After EVENT module loads boot.lua, REG.OnBoot will be set.
    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        return type(REG) == "table" and next(REG) == nil
    "#,
    );

    assert!(result.is_ok(), "REG should be empty table: {:?}", result);
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_reg_allows_handler_registration() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        REG.OnBoot = function(req) return "test" end
        return type(REG.OnBoot) == "function"
    "#,
    );

    assert!(
        result.is_ok(),
        "REG should allow handler registration: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 2.1-2.7, 3.2-3.4: EVENT Module Tests
// ============================================================================

#[test]
fn test_event_module_loads() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        return EVENT ~= nil
    "#,
    );

    assert!(result.is_ok(), "EVENT module should load: {:?}", result);
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_event_no_entry_returns_204() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local req = { id = "UnknownEvent", method = "get", version = 30 }
        local response = EVENT.no_entry(req)
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT.no_entry should return 204: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_event_fire_dispatches_registered_handler() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        local RES = require "pasta.shiori.res"
        
        REG.OnTest = function(req)
            return RES.ok("test response")
        end
        
        local req = { id = "OnTest", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil and response:find("Value: test response") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT.fire should dispatch to registered handler: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_event_fire_returns_no_content_for_unregistered() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local req = { id = "UnregisteredEvent", method = "get", version = 30 }
        local response = EVENT.fire(req)
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT.fire should return 204 for unregistered event: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_event_fire_handles_nil_id() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local req = { method = "get", version = 30 }  -- no id field
        local response = EVENT.fire(req)
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT.fire should return 204 for nil id: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_event_fire_catches_handler_error() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        
        REG.OnError = function(req)
            error("Test error message")
        end
        
        local req = { id = "OnError", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        return response:find("500 Internal Server Error") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT.fire should return 500 on handler error: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_error_message_no_newline() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        
        REG.OnMultilineError = function(req)
            error("First line\nSecond line\nThird line")
        end
        
        local req = { id = "OnMultilineError", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        -- X-Error-Reason should contain only the first line
        local has_500 = response:find("500 Internal Server Error") ~= nil
        local has_first_line = response:find("X%-Error%-Reason:") ~= nil
        local no_newline_in_reason = response:match("X%-Error%-Reason:[^\r\n]+Second") == nil
        
        return has_500 and has_first_line and no_newline_in_reason
    "#,
    );

    assert!(
        result.is_ok(),
        "Error message should not contain newlines: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_event_fire_handles_empty_error_message() {
    let runtime = create_runtime_with_pasta_path();

    // Note: In mlua, error("") still includes file location information,
    // so it won't be truly empty. This test verifies that the first line
    // is extracted correctly even when the user provides an empty message.
    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        
        REG.OnEmptyError = function(req)
            error("")
        end
        
        local req = { id = "OnEmptyError", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        -- Should return 500 with file location info (mlua adds it automatically)
        local has_500 = response:find("500 Internal Server Error") ~= nil
        local has_error_reason = response:find("X%-Error%-Reason:") ~= nil
        
        return has_500 and has_error_reason
    "#,
    );

    assert!(
        result.is_ok(),
        "Should handle empty error message gracefully: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 4.1, 4.2: Integration Tests
// ============================================================================

/// Task 4.1: Tests integration with RES module (RES.ok, RES.no_content, RES.err)
#[test]
fn test_event_module_with_res_module() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        local RES = require "pasta.shiori.res"
        
        -- Test 1: RES.ok integration via handler
        REG.TestOk = function(req)
            return RES.ok("Hello World")
        end
        local res1 = EVENT.fire({ id = "TestOk", method = "get", version = 30 })
        local ok_works = res1:find("200 OK") ~= nil and res1:find("Value: Hello World") ~= nil
        
        -- Test 2: RES.no_content integration via EVENT.no_entry
        local res2 = EVENT.fire({ id = "Unregistered", method = "get", version = 30 })
        local no_content_works = res2:find("204 No Content") ~= nil
        
        -- Test 3: RES.err integration via error handling
        REG.TestErr = function(req)
            error("Intentional error")
        end
        local res3 = EVENT.fire({ id = "TestErr", method = "get", version = 30 })
        local err_works = res3:find("500 Internal Server Error") ~= nil and res3:find("X%-Error%-Reason:") ~= nil
        
        return ok_works and no_content_works and err_works
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT module should integrate correctly with RES module: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Task 4.2: Tests complete handler registration and dispatch flow
#[test]
fn test_handler_registration_and_dispatch() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        local EVENT = require "pasta.shiori.event"
        local RES = require "pasta.shiori.res"
        
        -- Register multiple handlers
        REG.OnBoot = function(req)
            return RES.ok("Booting up!")
        end
        
        REG.OnClose = function(req)
            return RES.ok("Shutting down!")
        end
        
        REG.OnGhostChanged = function(req)
            return RES.ok("Ghost changed!")
        end
        
        -- Dispatch to each handler and verify correct handler is called
        local boot_res = EVENT.fire({ id = "OnBoot", method = "get", version = 30 })
        local close_res = EVENT.fire({ id = "OnClose", method = "get", version = 30 })
        local ghost_res = EVENT.fire({ id = "OnGhostChanged", method = "get", version = 30 })
        local unknown_res = EVENT.fire({ id = "OnUnknown", method = "get", version = 30 })
        
        local boot_correct = boot_res:find("Booting up!") ~= nil
        local close_correct = close_res:find("Shutting down!") ~= nil
        local ghost_correct = ghost_res:find("Ghost changed!") ~= nil
        local unknown_correct = unknown_res:find("204 No Content") ~= nil
        
        return boot_correct and close_correct and ghost_correct and unknown_correct
    "#,
    );

    assert!(
        result.is_ok(),
        "Multiple handlers should be dispatched correctly: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Default Event Handlers: boot.lua
// ============================================================================

/// Tests that default OnBoot handler is registered via boot.lua
#[test]
fn test_default_onboot_handler_registered() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        
        -- OnBoot should be registered after loading EVENT module
        return type(REG.OnBoot) == "function"
    "#,
    );

    assert!(
        result.is_ok(),
        "Default OnBoot handler should be registered: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests that default OnBoot returns 204 No Content
#[test]
fn test_default_onboot_returns_204() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        
        local req = { id = "OnBoot", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Default OnBoot should return 204: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests that custom OnBoot overrides default
#[test]
fn test_custom_onboot_overrides_default() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        -- Override default OnBoot
        REG.OnBoot = function(req)
            return RES.ok("Custom Boot!")
        end
        
        local req = { id = "OnBoot", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil and response:find("Custom Boot!") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Custom OnBoot should override default: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}
