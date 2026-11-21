//! Integration tests for the `@pasta_calendar` module.

use crate::common::create_empty_context;

use pasta_core::CalendarTable;
use pasta_lua::{PastaLuaRuntime, RuntimeConfig};

fn eval_bool(runtime: &PastaLuaRuntime, code: &str) -> bool {
    runtime.exec(code).unwrap().as_boolean() == Some(true)
}

#[test]
fn test_calendar_holiday_and_empty_event() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    assert!(eval_bool(
        &runtime,
        r#"
        local cal = require("@pasta_calendar")
        return cal.holiday(2027, 1, 11) == "成人の日"
            and cal.holiday(2027, 1, 12) == nil
            and cal.event(2027, 1, 11) == nil
        "#,
    ));
}

#[test]
fn test_calendar_event_uses_configured_table() {
    let table = CalendarTable::parse("0101,お正月\n").unwrap();
    let config = RuntimeConfig::minimal().with_calendar(table);
    let runtime = PastaLuaRuntime::with_config(create_empty_context(), config).unwrap();

    assert!(eval_bool(
        &runtime,
        r#"
        local cal = require("@pasta_calendar")
        return cal.event(2027, 1, 1) == "お正月" and cal.event(2027, 1, 2) == nil
        "#,
    ));
}

#[test]
fn test_calendar_non_integer_args_error() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    assert!(eval_bool(
        &runtime,
        r#"
        local cal = require("@pasta_calendar")
        return not pcall(cal.holiday, 2027, 1.5, 11)
            and not pcall(cal.holiday, "2027", 1, 11)
            and not pcall(cal.event, 2027, 1, nil)
        "#,
    ));
}
