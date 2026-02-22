//! Integration tests for the pasta_search module.
//!
//! These tests verify that the @pasta_search Lua module works correctly
//! with PastaLuaRuntime and SearchContext.

use pasta_lua::{PastaLuaRuntime, TranspileContext};
use std::collections::HashMap;

/// Helper to create a TranspileContext with test data.
fn create_test_context() -> TranspileContext {
    let mut ctx = TranspileContext::new();

    // Register global scenes
    ctx.scene_registry.register_global("挨拶", HashMap::new());
    ctx.scene_registry.register_global("会話", HashMap::new());

    // Register a local scene under 会話
    let (_, counter) = ctx.scene_registry.register_global("メイン", HashMap::new());
    ctx.scene_registry
        .register_local("選択肢", "メイン", counter, 1, HashMap::new());

    // Register global words
    ctx.word_registry.register_global(
        "挨拶",
        vec![
            "こんにちは".to_string(),
            "おはよう".to_string(),
            "こんばんは".to_string(),
        ],
    );
    ctx.word_registry
        .register_global("場所", vec!["東京".to_string(), "大阪".to_string()]);

    // Register local words
    ctx.word_registry
        .register_local("メイン_1", "挨拶", vec!["やあ".to_string()]);

    ctx
}

#[test]
fn test_runtime_creation() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx);
    assert!(runtime.is_ok());
}

#[test]
fn test_require_pasta_search() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test that @pasta_search can be required
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        return SEARCH ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_search_scene_global() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test global scene search
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local global_name, local_name = SEARCH:search_scene("挨拶", nil)
        return global_name
    "#,
    );

    if let Err(ref e) = result {
        eprintln!("Error: {:?}", e);
    }
    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let value = result.unwrap();
    assert!(value.is_string());
}

#[test]
fn test_search_scene_not_found() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test scene not found returns nil
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local global_name, local_name = SEARCH:search_scene("存在しないシーン", nil)
        return global_name == nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_search_word_global() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test global word search
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("挨拶", nil)
        return word ~= nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_search_word_local_fallback() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test local word search (fallback strategy: local found → return local only)
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("挨拶", "メイン_1")
        return word == "やあ"
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_search_word_not_found() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test word not found returns nil
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("存在しない単語", nil)
        return word == nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_set_scene_selector() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test set_scene_selector doesn't error
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_scene_selector(0, 1, 2)
        return true
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_set_word_selector() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test set_word_selector doesn't error
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(0, 1, 2)
        return true
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_set_selector_reset() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test reset selector (no args)
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_scene_selector()
        SEARCH:set_word_selector()
        return true
    "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_multiple_runtime_instances() {
    // Create two independent contexts
    let ctx1 = create_test_context();
    let ctx2 = create_test_context();

    let runtime1 = PastaLuaRuntime::new(ctx1).unwrap();
    let runtime2 = PastaLuaRuntime::new(ctx2).unwrap();

    // Both should work independently
    let result1 = runtime1.exec(
        r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_word("挨拶", nil)
    "#,
    );

    let result2 = runtime2.exec(
        r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_word("場所", nil)
    "#,
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_require_returns_same_instance() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Multiple requires should return the same instance
    let result = runtime.exec(
        r#"
        local SEARCH1 = require "@pasta_search"
        local SEARCH2 = require "@pasta_search"
        return SEARCH1 == SEARCH2
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_boolean().unwrap_or(false));
}

#[test]
fn test_set_selector_invalid_argument() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test that non-integer argument causes error
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_scene_selector("not an integer")
        return true
    "#,
    );

    assert!(result.is_err());
}

#[test]
fn test_search_scene_returns_transpiler_format() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test that search_scene returns names matching transpiler output format
    // Global scene: global_name = "挨拶_1", local_name = "__start__"
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local global_name, local_name = SEARCH:search_scene("挨拶", nil)
        -- Verify format: global_name should be "挨拶_1", local_name should be "__start__"
        return global_name == "挨拶_1" and local_name == "__start__"
    "#,
    );

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let value = result.unwrap();
    assert!(
        value.as_boolean().unwrap_or(false),
        "Expected global_name='挨拶_1' and local_name='__start__'"
    );
}

#[test]
fn test_search_scene_local_returns_transpiler_format() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test local scene search returns names matching transpiler output format
    // Local scene: global_name = "メイン_1", local_name = "__選択肢_1__"
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local global_name, local_name = SEARCH:search_scene("選択肢", "メイン_1")
        -- Verify format: global_name should be "メイン_1", local_name should be "__選択肢_1__"
        return global_name == "メイン_1" and local_name == "__選択肢_1__"
    "#,
    );

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let value = result.unwrap();
    assert!(
        value.as_boolean().unwrap_or(false),
        "Expected global_name='メイン_1' and local_name='__選択肢_1__'"
    );
}
