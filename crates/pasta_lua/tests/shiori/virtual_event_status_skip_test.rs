//! Integration tests for pasta.shiori.event.virtual_dispatcher status suppression.
//!
//! Tests for choosing/talking status skip behavior in dispatch().

use crate::common;

use common::create_runtime_with_pasta_path;

// ============================================================================
// choosing 状態での OnTalk/OnHour 抑制テスト
// Requirements: 4.1, 4.2, 4.3
// ============================================================================

#[test]
fn test_skip_when_choosing() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        dispatcher._set_scene_executor(function(event_name)
            return coroutine.create(function() return event_name end)
        end)

        -- Initialize
        local act1 = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act1)

        -- Call at next hour with "choosing" status - dispatch() should block
        local act2 = { req = {
            id = "OnSecondChange",
            status = "choosing",
            date = { unix = 1702652400, hour = 15 }
        } }
        local result = dispatcher.dispatch(act2)

        return result == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Should skip when status is 'choosing': {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_skip_when_csv_talking_choosing() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local dispatcher = require "pasta.shiori.event.virtual_dispatcher"
        dispatcher._reset()

        dispatcher._set_scene_executor(function(event_name)
            return coroutine.create(function() return event_name end)
        end)

        -- Initialize
        local act1 = { req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        } }
        dispatcher.dispatch(act1)

        -- Call with CSV status containing both talking and choosing - dispatch() should block
        local act2 = { req = {
            id = "OnSecondChange",
            status = "talking,choosing,balloon(0=2)",
            date = { unix = 1702652400, hour = 15 }
        } }
        local result = dispatcher.dispatch(act2)

        return result == nil
    "#,
    );

    assert!(
        result.is_ok(),
        "Should skip when status is CSV 'talking,choosing,balloon(0=2)': {:?}",
        result
    );
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}
