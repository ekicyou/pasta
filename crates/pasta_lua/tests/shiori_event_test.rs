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

// ============================================================================
// Task 1.1, 2.5: Scene Function Fallback Tests
// ============================================================================

/// Tests that EVENT.no_entry attempts scene function fallback search
#[test]
fn test_no_entry_attempts_scene_fallback() {
    // Create context with a scene registered for event fallback
    let mut ctx = create_empty_context();
    // register_global returns (global_name, counter), e.g., ("OnTestEvent", 1)
    // The Lua side must use "OnTestEvent_1" as the global_name for SCENE.register
    let (_, counter) = ctx
        .scene_registry
        .register_global("OnTestEvent", std::collections::HashMap::new());

    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let scripts_dir = get_scripts_dir();
    runtime
        .exec(&format!(
            r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
        ))
        .expect("Failed to configure package.path");

    let result = runtime.exec(&format!(
        r#"
        local EVENT = require "pasta.shiori.event"
        local SCENE = require "pasta.scene"
        
        -- Register a scene function with event name pattern (Lua側)
        -- Rust側でregister_global("OnTestEvent")すると"OnTestEvent_1"として登録される
        local called = false
        SCENE.register("OnTestEvent_{counter}", "__start__", function(act, ...)
            called = true
        end)
        
        -- Fire unregistered event - should attempt scene fallback
        local req = {{ id = "OnTestEvent", method = "get", version = 30 }}
        local response = EVENT.fire(req)
        
        -- alpha01: scene function is called but returns 204 (no act output yet)
        return response:find("204 No Content") ~= nil and called == true
    "#,
        counter = counter
    ));

    assert!(
        result.is_ok(),
        "EVENT.no_entry should attempt scene fallback: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests that scene fallback returns 204 when scene not found
#[test]
fn test_scene_fallback_returns_204_when_not_found() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        
        -- Fire unregistered event with no matching scene
        local req = { id = "NonExistentEvent", method = "get", version = 30 }
        local response = EVENT.fire(req)
        
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Scene fallback should return 204 when not found: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests that scene function errors are caught and return 500
#[test]
fn test_scene_fallback_catches_errors() {
    // Create context with a scene registered for error testing
    let mut ctx = create_empty_context();
    // register_global returns (global_name, counter), e.g., ("OnErrorScene", 1)
    let (_, counter) = ctx
        .scene_registry
        .register_global("OnErrorScene", std::collections::HashMap::new());

    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let scripts_dir = get_scripts_dir();
    runtime
        .exec(&format!(
            r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
        ))
        .expect("Failed to configure package.path");

    let result = runtime.exec(&format!(
        r#"
        local EVENT = require "pasta.shiori.event"
        local SCENE = require "pasta.scene"
        
        -- Register a scene function that throws an error
        -- Rust側でregister_global("OnErrorScene")すると"OnErrorScene_1"として登録される
        SCENE.register("OnErrorScene_{counter}", "__start__", function(act, ...)
            error("Scene function error!")
        end)
        
        local req = {{ id = "OnErrorScene", method = "get", version = 30 }}
        local response = EVENT.fire(req)
        
        return response:find("500 Internal Server Error") ~= nil
    "#,
        counter = counter
    ));

    assert!(
        result.is_ok(),
        "Scene fallback should catch errors: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 2.1: OnFirstBoot / OnBoot / OnClose Tests
// ============================================================================

/// Tests OnFirstBoot handler registration and Reference0 access
#[test]
fn test_onfirstboot_handler_with_reference() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local vanish_flag = nil
        REG.OnFirstBoot = function(req)
            vanish_flag = req.reference[0]  -- バニッシュ復帰フラグ
            return RES.ok("First Boot!")
        end
        
        local req = {
            id = "OnFirstBoot",
            method = "get",
            version = 30,
            reference = { [0] = "1" }  -- バニッシュから復帰
        }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil and vanish_flag == "1"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnFirstBoot should access Reference0: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests OnBoot handler with multiple Reference fields
#[test]
fn test_onboot_handler_with_references() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local shell_name, shell_path, ghost_path = nil, nil, nil
        REG.OnBoot = function(req)
            shell_name = req.reference[0]
            shell_path = req.reference[6]
            ghost_path = req.reference[7]
            return RES.ok("Boot!")
        end
        
        local req = {
            id = "OnBoot",
            method = "get",
            version = 30,
            reference = {
                [0] = "master",
                [6] = "C:/ghost/shell/master",
                [7] = "C:/ghost"
            }
        }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil
            and shell_name == "master"
            and shell_path == "C:/ghost/shell/master"
            and ghost_path == "C:/ghost"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnBoot should access Reference0/6/7: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests OnClose handler with Reference0 (close reason)
#[test]
fn test_onclose_handler_with_reference() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local close_reason = nil
        REG.OnClose = function(req)
            close_reason = req.reference[0]
            return RES.ok("Goodbye!")
        end
        
        local req = {
            id = "OnClose",
            method = "get",
            version = 30,
            reference = { [0] = "user" }
        }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil and close_reason == "user"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnClose should access Reference0: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 2.2: OnGhostChanged / OnSecondChange / OnMinuteChange Tests
// ============================================================================

/// Tests OnGhostChanged handler with Reference0/1
#[test]
fn test_onghostchanged_handler_with_references() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local to_ghost, from_ghost = nil, nil
        REG.OnGhostChanged = function(req)
            to_ghost = req.reference[0]
            from_ghost = req.reference[1]
            return RES.ok("Changed!")
        end
        
        local req = {
            id = "OnGhostChanged",
            method = "get",
            version = 30,
            reference = {
                [0] = "NewGhost",
                [1] = "OldGhost"
            }
        }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil
            and to_ghost == "NewGhost"
            and from_ghost == "OldGhost"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnGhostChanged should access Reference0/1: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests OnSecondChange handler with Reference0/1
#[test]
fn test_onsecondchange_handler_with_references() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local current_sec, total_sec = nil, nil
        REG.OnSecondChange = function(req)
            current_sec = req.reference[0]
            total_sec = req.reference[1]
            return RES.no_content()  -- 通常は空応答
        end
        
        local req = {
            id = "OnSecondChange",
            method = "notify",
            version = 30,
            reference = {
                [0] = "30",
                [1] = "12345"
            }
        }
        local response = EVENT.fire(req)
        
        return response:find("204 No Content") ~= nil
            and current_sec == "30"
            and total_sec == "12345"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnSecondChange should access Reference0/1: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests OnMinuteChange handler with Reference0/1
#[test]
fn test_onminutechange_handler_with_references() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local current_min, current_hour = nil, nil
        REG.OnMinuteChange = function(req)
            current_min = req.reference[0]
            current_hour = req.reference[1]
            return RES.no_content()
        end
        
        local req = {
            id = "OnMinuteChange",
            method = "notify",
            version = 30,
            reference = {
                [0] = "45",
                [1] = "14"
            }
        }
        local response = EVENT.fire(req)
        
        return response:find("204 No Content") ~= nil
            and current_min == "45"
            and current_hour == "14"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnMinuteChange should access Reference0/1: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 2.3: OnMouseDoubleClick Tests
// ============================================================================

/// Tests OnMouseDoubleClick handler with Reference0/4
#[test]
fn test_onmousedoubleclick_handler_with_references() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local scope, hit_area = nil, nil
        REG.OnMouseDoubleClick = function(req)
            scope = req.reference[0]     -- 0: sakura, 1: kero
            hit_area = req.reference[4]  -- 当たり判定ID
            return RES.ok("Clicked!")
        end
        
        local req = {
            id = "OnMouseDoubleClick",
            method = "get",
            version = 30,
            reference = {
                [0] = "0",      -- sakura
                [4] = "Head"    -- 当たり判定ID
            }
        }
        local response = EVENT.fire(req)
        
        return response:find("200 OK") ~= nil
            and scope == "0"
            and hit_area == "Head"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnMouseDoubleClick should access Reference0/4: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 2.4: Unregistered Event Fallback Tests (Additional)
// ============================================================================

/// Tests that nil Reference access returns nil
#[test]
fn test_nil_reference_access() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        local RES = require "pasta.shiori.res"
        
        local ref5, ref7 = "unset", "unset"
        REG.OnTestNil = function(req)
            ref5 = req.reference[5]
            ref7 = req.reference[7]
            return RES.ok("OK")
        end
        
        local req = {
            id = "OnTestNil",
            method = "get",
            version = 30,
            reference = { [0] = "exists" }  -- Only ref0 exists
        }
        local response = EVENT.fire(req)
        
        return ref5 == nil and ref7 == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Nil Reference access should return nil: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}
