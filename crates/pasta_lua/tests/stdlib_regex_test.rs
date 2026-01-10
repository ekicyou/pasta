//! Integration tests for mlua-stdlib @regex module in PastaLuaRuntime.
//!
//! These tests verify that the @regex Lua module works correctly
//! with PastaLuaRuntime.

use mlua::Value;
use pasta_lua::{PastaLuaRuntime, TranspileContext};

/// Helper to create an empty TranspileContext for testing.
fn create_empty_context() -> TranspileContext {
    TranspileContext::new()
}

/// Helper to extract string from Lua Value.
fn value_to_string(value: &Value) -> String {
    value
        .as_string()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[test]
fn test_regex_module_available() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test that @regex can be required
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        return regex ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_regex_new() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test creating a regex object
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("\\d+")
        return re ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_regex_is_match() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test is_match function
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("hello")
        return re:is_match("hello world")
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_regex_is_match_false() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test is_match returning false
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("goodbye")
        return re:is_match("hello world")
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(!value.as_boolean().unwrap_or(true));
}

#[test]
fn test_regex_match_captures() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test match with capture groups
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("(\\d{4})-(\\d{2})-(\\d{2})")
        local matches = re:match("2024-01-15")
        return matches[1]
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value_to_string(&value), "2024");
}

#[test]
fn test_regex_named_captures() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test match with named capture groups
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})")
        local matches = re:match("2024-01-15")
        return matches["month"]
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value_to_string(&value), "01");
}

#[test]
fn test_regex_split() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test split function
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("[,.]")
        local parts = re:split("abc.def,ghi")
        return #parts
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_i64().unwrap_or(0), 3);
}

#[test]
fn test_regex_replace() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test replace function
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("\\s+")
        local result = re:replace("hello   world", " ")
        return result
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value_to_string(&value), "hello world");
}

#[test]
fn test_regex_shortcut_is_match() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test shortcut is_match function
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        return regex.is_match("\\d+", "abc123def")
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_regex_shortcut_match() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test shortcut match function
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local matches = regex.match("(\\w+)@(\\w+\\.\\w+)", "test@example.com")
        return matches[1] .. "@" .. matches[2]
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value_to_string(&value), "test@example.com");
}

#[test]
fn test_regex_escape() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test escape function
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local escaped = regex.escape("a*b+c?")
        return escaped
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value_to_string(&value), "a\\*b\\+c\\?");
}

#[test]
fn test_regex_invalid_pattern() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test invalid regex pattern handling
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re, err = regex.new("(")
        return re == nil and err ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_regex_japanese_text() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test regex with Japanese text (important for Pasta DSL)
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("こんにち[はわ]")
        return re:is_match("こんにちは世界")
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_regex_unicode_word_boundary() {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test Unicode patterns
    let result = runtime.exec(
        r#"
        local regex = require "@regex"
        local re = regex.new("[ぁ-んァ-ン一-龯]+")
        local matches = re:match("Hello世界です")
        return matches[0]
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value_to_string(&value), "世界です");
}
