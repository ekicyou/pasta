//! Integration tests for pasta.shiori.event module – core dispatching.
//!
//! Tests verify REG/EVENT module loading, event dispatching, error handling,
//! default boot handlers, and integration with the RES module.

use crate::common;

use common::{assert_shiori_response, create_runtime_with_pasta_path, value_as_str};

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
        REG.OnBoot = function(act) return "test" end
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
fn test_event_no_entry_returns_nil_when_scene_not_found() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local SHIORI_ACT = require "pasta.shiori.act"
        local act = SHIORI_ACT.new({}, { id = "UnknownEvent", method = "get", version = 30 })
        local response = EVENT.no_entry(act)
        -- EVENT.no_entry now returns thread|nil, not SHIORI response
        -- When no scene is found, it returns nil
        return response == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "EVENT.no_entry should return nil when scene not found: {:?}",
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
        
        REG.OnTest = function(act)
            return RES.ok("test response")
        end
        
        local req = { id = "OnTest", method = "get", version = 30 }
        return EVENT.fire(req)
    "#,
    );

    let response = result.expect("EVENT.fire should dispatch to registered handler");
    assert_shiori_response(
        &value_as_str(&response).unwrap(),
        "200 OK",
        Some("test response"),
    );
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
        -- When req.id is nil, REG[nil] is nil, so no_entry is called
        -- no_entry expects act.req.id but will get nil from act.req
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
        local SHIORI = require "pasta.shiori.entry"
        
        REG.OnError = function(act)
            error("Test error message")
        end
        
        local req = { id = "OnError", method = "get", version = 30 }
        local response = SHIORI.request(req)
        
        return response:find("500 Internal Server Error") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "SHIORI.request should return 500 on handler error: {:?}",
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
        local SHIORI = require "pasta.shiori.entry"
        
        REG.OnMultilineError = function(act)
            error("First line\nSecond line\nThird line")
        end
        
        local req = { id = "OnMultilineError", method = "get", version = 30 }
        local response = SHIORI.request(req)
        
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
        local SHIORI = require "pasta.shiori.entry"
        
        REG.OnEmptyError = function(act)
            error("")
        end
        
        local req = { id = "OnEmptyError", method = "get", version = 30 }
        local response = SHIORI.request(req)
        
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
        local SHIORI = require "pasta.shiori.entry"
        local RES = require "pasta.shiori.res"
        
        -- Test 1: RES.ok integration via handler
        REG.TestOk = function(act)
            return RES.ok("Hello World")
        end
        local res1 = EVENT.fire({ id = "TestOk", method = "get", version = 30 })
        
        -- Test 2: RES.no_content integration via EVENT.no_entry
        local res2 = EVENT.fire({ id = "Unregistered", method = "get", version = 30 })
        assert(res2:find("204 No Content") ~= nil, res2)
        
        -- Test 3: RES.err integration via error handling (use SHIORI.request for xpcall)
        REG.TestErr = function(act)
            error("Intentional error")
        end
        local res3 = SHIORI.request({ id = "TestErr", method = "get", version = 30 })
        assert(res3:find("500 Internal Server Error") ~= nil and res3:find("X%-Error%-Reason:") ~= nil, res3)
        
        return res1
    "#,
    );

    let res1 = result.expect("EVENT module should integrate correctly with RES module");
    assert_shiori_response(&value_as_str(&res1).unwrap(), "200 OK", Some("Hello World"));
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
        REG.OnBoot = function(act)
            return RES.ok("Booting up!")
        end
        
        REG.OnClose = function(act)
            return RES.ok("Shutting down!")
        end
        
        REG.OnGhostChanged = function(act)
            return RES.ok("Ghost changed!")
        end
        
        -- Dispatch to each handler and verify correct handler is called
        local boot_res = EVENT.fire({ id = "OnBoot", method = "get", version = 30 })
        local close_res = EVENT.fire({ id = "OnClose", method = "get", version = 30 })
        local ghost_res = EVENT.fire({ id = "OnGhostChanged", method = "get", version = 30 })
        local unknown_res = EVENT.fire({ id = "OnUnknown", method = "get", version = 30 })
        
        return { boot_res, close_res, ghost_res, unknown_res }
    "#,
    );

    let responses = result.expect("Multiple handlers should be dispatched correctly");
    let responses = responses.as_table().unwrap();
    let get = |i: usize| responses.get::<String>(i).unwrap();
    assert_shiori_response(&get(1), "200 OK", Some("Booting up!"));
    assert_shiori_response(&get(2), "200 OK", Some("Shutting down!"));
    assert_shiori_response(&get(3), "200 OK", Some("Ghost changed!"));
    assert_shiori_response(&get(4), "204 No Content", None);
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
        REG.OnBoot = function(act)
            return RES.ok("Custom Boot!")
        end
        
        local req = { id = "OnBoot", method = "get", version = 30 }
        return EVENT.fire(req)
    "#,
    );

    let response = result.expect("Custom OnBoot should override default");
    assert_shiori_response(
        &value_as_str(&response).unwrap(),
        "200 OK",
        Some("Custom Boot!"),
    );
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
        REG.OnTestNil = function(act)
            ref5 = act.req.reference[5]
            ref7 = act.req.reference[7]
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

// ============================================================================
// Default Event Handlers: choice_select.lua (Task 4.3)
// ============================================================================

/// Tests that default OnChoiceSelectEx handler is registered via choice_select.lua
#[test]
fn test_default_onchoiceselectex_handler_registered() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        local REG = require "pasta.shiori.event.register"
        
        -- OnChoiceSelectEx should be registered after loading EVENT module
        return type(REG.OnChoiceSelectEx) == "function"
    "#,
    );

    assert!(
        result.is_ok(),
        "Default OnChoiceSelectEx handler should be registered: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests that default OnChoiceSelectEx returns 204 when no matching scene exists
#[test]
fn test_default_onchoiceselectex_returns_204_no_match() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        
        local req = {
            id = "OnChoiceSelectEx",
            method = "get",
            version = 30,
            reference = { [0] = "label", [1] = "nonexistent_choice_id" }
        }
        local response = EVENT.fire(req)
        
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Default OnChoiceSelectEx should return 204 when no scene matches: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

/// Tests that OnChoiceSelectEx returns 204 when reference is missing
#[test]
fn test_default_onchoiceselectex_returns_204_no_reference() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local EVENT = require "pasta.shiori.event"
        
        local req = {
            id = "OnChoiceSelectEx",
            method = "get",
            version = 30,
        }
        local response = EVENT.fire(req)
        
        return response:find("204 No Content") ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Default OnChoiceSelectEx should return 204 when no reference: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}
