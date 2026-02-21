//! Integration tests for pasta.shiori.event module – scene fallback & specific handlers.
//!
//! Tests verify scene function fallback dispatching and specific SHIORI event
//! handlers (OnFirstBoot, OnBoot, OnClose, OnGhostChanged, OnSecondChange,
//! OnMinuteChange, OnMouseDoubleClick).

mod common;

use common::{create_empty_context, create_runtime_with_pasta_path, get_scripts_dir};
use pasta_lua::PastaLuaRuntime;

// ============================================================================
// Task 1.1, 2.5: Scene Function Fallback Tests
// ============================================================================

/// Tests that EVENT.no_entry attempts scene function fallback search
/// and EVENT.fire correctly resumes the returned coroutine
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

    // Mock @pasta_persistence module (required by pasta.save)
    // Note: Do NOT mock @pasta_search here - PastaLuaRuntime::new(ctx) already registered it
    //       with the scene_registry that has "OnTestEvent" registered
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

    let result = runtime.exec(&format!(
        r#"
        local EVENT = require "pasta.shiori.event"
        local SCENE = require "pasta.scene"
        local STORE = require "pasta.store"
        
        -- Setup actors for SHIORI_ACT
        STORE.actors = {{ sakura = {{ name = "さくら", spot = "sakura" }} }}
        
        -- Register a scene function with event name pattern (Lua側)
        -- Rust側でregister_global("OnTestEvent")すると"OnTestEvent_1"として登録される
        local called = false
        SCENE.register("OnTestEvent_{counter}", "__start__", function(act, ...)
            called = true
            -- Scene function must return a string for RES.ok() to generate 200 OK
            -- or nil for 204 No Content
            return nil
        end)
        
        -- Fire unregistered event - should attempt scene fallback via EVENT.no_entry
        -- EVENT.fire will resume the thread and generate appropriate response
        local req = {{ id = "OnTestEvent", method = "get", version = 30 }}
        local response = EVENT.fire(req)
        
        -- Scene function returns nil, so RES.ok(nil) -> RES.no_content() -> 204
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

    // Mock @pasta_persistence module (required by pasta.save)
    // Note: Do NOT mock @pasta_search here - PastaLuaRuntime::new(ctx) already registered it
    //       with the scene_registry that has "OnErrorScene" registered
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

    let result = runtime.exec(&format!(
        r#"
        local SHIORI = require "pasta.shiori.entry"
        local SCENE = require "pasta.scene"
        
        -- Register a scene function that throws an error
        -- Rust側でregister_global("OnErrorScene")すると"OnErrorScene_1"として登録される
        SCENE.register("OnErrorScene_{counter}", "__start__", function(act, ...)
            error("Scene function error!")
        end)
        
        local req = {{ id = "OnErrorScene", method = "get", version = 30 }}
        local response = SHIORI.request(req)
        
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
        REG.OnFirstBoot = function(act)
            vanish_flag = act.req.reference[0]  -- バニッシュ復帰フラグ
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
        REG.OnBoot = function(act)
            shell_name = act.req.reference[0]
            shell_path = act.req.reference[6]
            ghost_path = act.req.reference[7]
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
        REG.OnClose = function(act)
            close_reason = act.req.reference[0]
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
        REG.OnGhostChanged = function(act)
            to_ghost = act.req.reference[0]
            from_ghost = act.req.reference[1]
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
        REG.OnSecondChange = function(act)
            current_sec = act.req.reference[0]
            total_sec = act.req.reference[1]
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
        REG.OnMinuteChange = function(act)
            current_min = act.req.reference[0]
            current_hour = act.req.reference[1]
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
        REG.OnMouseDoubleClick = function(act)
            scope = act.req.reference[0]     -- 0: sakura, 1: kero
            hit_area = act.req.reference[4]  -- 当たり判定ID
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
