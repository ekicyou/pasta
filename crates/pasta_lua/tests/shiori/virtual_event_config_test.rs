//! Integration tests for pasta.shiori.event.virtual_dispatcher module.
//!
//! Tests for configuration, state management, and second_change integration.

use crate::common;

use common::create_runtime_with_pasta_path;

// ============================================================================
// Task 4.5: Config and Status Tests
// ============================================================================

#[test]
fn test_config_default_values() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()
        
        -- Trigger config load by calling dispatch
        local act = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act)
        
        local state = dispatcher._get_internal_state()
        local cfg = state.cached_config
        
        -- Default values: min=180, max=300, margin=30
        return cfg ~= nil
           and cfg.talk_interval_min == 180
           and cfg.talk_interval_max == 300
           and cfg.hour_margin == 30
    "#,
    );

    assert!(
        result.is_ok(),
        "Config should have default values: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_skip_when_talking() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()
        
        -- Set up mock scene executor (should never be called when "talking")
        local scene_results = {
            OnHour = "hour",
            OnTalk = "talk"
        }
        dispatcher._set_scene_executor(function(event_name)
            return scene_results[event_name]
        end)
        
        -- Initialize
        local act1 = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act1)
        
        -- Call at next hour with "talking" status - should skip
        local act2 = { req = {
            id = "OnSecondChange",
            status = "talking",  -- Currently talking
            date = { unix = 1702652400 }  -- Next hour
        } }
        local hour_result = dispatcher.check_hour(act2)
        local talk_result = dispatcher.check_talk(act2)
        
        return hour_result == nil and talk_result == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Should skip when status is 'talking': {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// Task 4.6: State Management Tests
// ============================================================================

#[test]
fn test_module_state_reset() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        
        -- Set some state
        local act = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act)
        
        local state_before = dispatcher._get_internal_state()
        local had_state = state_before.next_hour_unix > 0
        
        -- Reset
        dispatcher._reset()
        
        local state_after = dispatcher._get_internal_state()
        local is_reset = state_after.next_hour_unix == 0
                     and state_after.next_talk_time == 0
                     and state_after.cached_config == nil
        
        return had_state and is_reset
    "#,
    );

    assert!(result.is_ok(), "State should reset properly: {:?}", result);
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_internal_state_getter() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()
        
        local state = dispatcher._get_internal_state()
        
        return type(state) == "table"
           and state.next_hour_unix == 0
           and state.next_talk_time == 0
           and state.cached_config == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Internal state getter should work: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

// ============================================================================
// second_change module integration
// ============================================================================

#[test]
fn test_second_change_module_loads() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local second_change = require "pasta.shiori.event.second_change"
        return second_change ~= nil
    "#,
    );

    assert!(
        result.is_ok(),
        "second_change module should load: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_onsecondchange_handler_registered() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local REG = require "pasta.shiori.event.register"
        require "pasta.shiori.event.second_change"
        
        return type(REG.OnSecondChange) == "function"
    "#,
    );

    assert!(
        result.is_ok(),
        "OnSecondChange handler should be registered: {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}
