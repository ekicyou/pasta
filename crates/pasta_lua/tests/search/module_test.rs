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
fn test_search_word_deterministic_with_mock_selector() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // MockRandomSelector disables shuffling, so "場所" words come back in
    // registration order without repetition: 東京 → 大阪.
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(0)
        local first = SEARCH:search_word("場所", nil)
        local second = SEARCH:search_word("場所", nil)
        return first == "東京" and second == "大阪"
    "#,
    );

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let value = result.unwrap();
    assert!(
        value.as_boolean().unwrap_or(false),
        "Expected deterministic word order 東京 → 大阪"
    );
}

#[test]
fn test_search_word_global_not_visible_with_parent_scope() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // "場所" is registered globally only. With a parent scope the search is
    // local-only (no fallback), so it must return nil.
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("場所", "メイン_1")
        return word == nil
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(
        value.as_boolean().unwrap_or(false),
        "Global-only word should not be found with a parent scope"
    );
}

#[test]
fn test_search_scene_empty_name_raises_error() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // An empty search key is an internal error (InvalidScene), surfaced to
    // Lua as a runtime error, which pcall must catch.
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local ok = pcall(function()
            SEARCH:search_scene("", nil)
        end)
        return ok == false
    "#,
    );

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(
        value.as_boolean().unwrap_or(false),
        "Empty scene name should raise a Lua error"
    );
}

#[test]
fn test_search_scene_local_returns_transpiler_format() {
    let ctx = create_test_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();

    // Test local scene search returns names matching transpiler output format
    // Local scene: global_name = "メイン_1", local_name = "選択肢_1"
    let result = runtime.exec(
        r#"
        local SEARCH = require "@pasta_search"
        local global_name, local_name = SEARCH:search_scene("選択肢", "メイン_1")
        -- Verify format: global_name should be "メイン_1", local_name should be "選択肢_1"
        return global_name == "メイン_1" and local_name == "選択肢_1"
    "#,
    );

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let value = result.unwrap();
    assert!(
        value.as_boolean().unwrap_or(false),
        "Expected global_name='メイン_1' and local_name='選択肢_1'"
    );
}

// ---------------------------------------------------------------------------
// Selector sequence: positions into the candidate order decide a round's order.
// 挨拶 words in candidate order: こんにちは, おはよう, こんばんは.
// 場所 words in candidate order: 東京, 大阪.
// ---------------------------------------------------------------------------

/// Runs a Lua chunk on a fresh runtime and returns its string result.
fn exec_string(ctx: TranspileContext, code: &str) -> String {
    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let value = runtime.exec(code).unwrap();
    value.as_string().unwrap().to_string_lossy()
}

/// Test context plus three same-name scenes 雑談 (雑談_1..雑談_3, fewer than 10).
fn create_context_with_three_scenes() -> TranspileContext {
    let mut ctx = create_test_context();
    for _ in 0..3 {
        ctx.scene_registry.register_global("雑談", HashMap::new());
    }
    ctx
}

#[test]
fn test_word_selector_picks_second_candidate_then_rest_then_new_round() {
    let result = exec_string(
        create_test_context(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(1)
        local r = {}
        for i = 1, 4 do r[i] = SEARCH:search_word("挨拶", nil) end
        return table.concat(r, ",")
    "#,
    );
    assert_eq!(result, "おはよう,こんにちは,こんばんは,おはよう");
}

#[test]
fn test_scene_selector_picks_second_candidate_then_rest_then_new_round() {
    let result = exec_string(
        create_context_with_three_scenes(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_scene_selector(1)
        local r = {}
        for i = 1, 4 do r[i] = SEARCH:search_scene("雑談", nil) end
        return table.concat(r, ",")
    "#,
    );
    assert_eq!(result, "雑談_2,雑談_1,雑談_3,雑談_2");
}

#[test]
fn test_word_selector_ignores_out_of_range_and_duplicate_positions() {
    let result = exec_string(
        create_test_context(),
        r#"
        local SEARCH = require "@pasta_search"
        local function round()
            local r = {}
            for i = 1, 3 do r[i] = SEARCH:search_word("挨拶", nil) end
            return table.concat(r, ",")
        end
        SEARCH:set_word_selector(7, 1)
        local out_of_range = round()
        SEARCH:set_word_selector(1, 1, 0)
        local duplicate = round()
        return out_of_range .. "|" .. duplicate
    "#,
    );
    assert_eq!(
        result,
        "おはよう,こんにちは,こんばんは|おはよう,こんにちは,こんばんは"
    );
}

#[test]
fn test_word_selector_negative_integer_errors_and_keeps_round() {
    let result = exec_string(
        create_test_context(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(1)
        local first = SEARCH:search_word("挨拶", nil)
        local ok, err = pcall(function() SEARCH:set_word_selector(-1) end)
        local second = SEARCH:search_word("挨拶", nil)
        return table.concat({first, tostring(ok), tostring(err), second}, "|")
    "#,
    );
    let parts: Vec<&str> = result.split('|').collect();
    assert_eq!(parts[0], "おはよう");
    assert_eq!(parts[1], "false");
    assert!(
        parts[2].contains("expected non-negative integer argument"),
        "unexpected error message: {}",
        parts[2]
    );
    assert_eq!(parts[3], "こんにちは");
}

#[test]
fn test_word_selector_recall_mid_round_clears_round() {
    let result = exec_string(
        create_test_context(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(1)
        local r = {}
        r[1] = SEARCH:search_word("挨拶", nil)
        r[2] = SEARCH:search_word("挨拶", nil)
        SEARCH:set_word_selector(1)
        r[3] = SEARCH:search_word("挨拶", nil)
        return table.concat(r, ",")
    "#,
    );
    assert_eq!(result, "おはよう,こんにちは,おはよう");
}

#[test]
fn test_word_selector_round_independent_of_other_searches() {
    let result = exec_string(
        create_test_context(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(1)
        local r = {}
        r[1] = SEARCH:search_word("挨拶", nil)
        r[2] = SEARCH:search_word("場所", nil)
        r[3] = SEARCH:search_word("挨拶", nil)
        r[4] = SEARCH:search_word("場所", nil)
        r[5] = SEARCH:search_word("挨拶", nil)
        return table.concat(r, ",")
    "#,
    );
    assert_eq!(result, "おはよう,大阪,こんにちは,東京,こんばんは");
}

#[test]
fn test_scene_and_word_selectors_follow_own_sequences() {
    let result = exec_string(
        create_context_with_three_scenes(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_scene_selector(2)
        SEARCH:set_word_selector(1)
        local r = {}
        r[1] = SEARCH:search_scene("雑談", nil)
        r[2] = SEARCH:search_word("挨拶", nil)
        r[3] = SEARCH:search_scene("雑談", nil)
        r[4] = SEARCH:search_word("挨拶", nil)
        return table.concat(r, ",")
    "#,
    );
    assert_eq!(result, "雑談_3,おはよう,雑談_1,こんにちは");
}

#[test]
fn test_word_selector_reset_restores_default_selection() {
    // 3 candidates, 40 trials: false failure probability is about 3^-39.
    let result = exec_string(
        create_test_context(),
        r#"
        local SEARCH = require "@pasta_search"
        SEARCH:set_word_selector(1)
        local seen, kinds = {}, 0
        for _ = 1, 40 do
            SEARCH:set_word_selector()
            local w = SEARCH:search_word("挨拶", nil)
            if not seen[w] then seen[w] = true; kinds = kinds + 1 end
        end
        return tostring(kinds)
    "#,
    );
    let kinds: usize = result.parse().unwrap();
    assert!(
        kinds >= 2,
        "expected 2+ distinct first results, got {kinds}"
    );
}
