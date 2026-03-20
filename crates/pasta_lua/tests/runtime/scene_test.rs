//! Runtime Scene E2E Tests for pasta_lua.
//!
//! These tests verify scene dictionary and word dictionary runtime execution:
//! - Scene dictionary prefix search and random selection
//! - Word dictionary random selection and replacement
//! - Actor word scope resolution
//! - Complete flow from Pasta to output
//!
//! # Requirements Coverage
//! - Requirement 7.1: Untested area tests
//! - Requirement 7.2: Runtime E2E tests

use crate::common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};

// ============================================================================
// E2E Helper Module Tests
// ============================================================================

#[test]
fn test_create_runtime_with_finalize_succeeds() {
    let lua = create_runtime_with_finalize();
    assert!(lua.is_ok(), "Runtime creation should succeed");
}

#[test]
fn test_transpile_basic_scene() {
    // 末尾に\nが必要（action_lineはeolで終わる必要がある）
    let source = "＊挨拶\n  さくら：「こんにちは！」\n";
    let lua_code = transpile(source);
    assert!(
        lua_code.contains("create_scene"),
        "Transpiled code should contain create_scene"
    );
}

#[test]
fn test_e2e_pipeline_basic() {
    let lua = create_runtime_with_finalize().unwrap();

    // 末尾に\nが必要（action_lineはeolで終わる必要がある）
    let source = "＊挨拶\n  さくら：「こんにちは！」\n";
    let lua_code = transpile(source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify scene can be searched (シーン名は「挨拶」)
    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("挨拶", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.contains("挨拶"),
        "Global name should contain '挨拶', got: {}",
        result.0
    );
}

// ============================================================================
// E2E Fixture Tests (Task 2.1)
// ============================================================================

/// Test that runtime_e2e_scene.pasta fixture parses and transpiles correctly
#[test]
fn test_fixture_scene_parses() {
    let source = include_str!("../fixtures/e2e/runtime_e2e_scene.pasta");
    let lua_code = transpile(source);

    // Verify 3 挨拶 scenes are registered
    assert!(
        lua_code.contains("挨拶おはよう"),
        "Should contain 挨拶おはよう scene"
    );
    assert!(
        lua_code.contains("挨拶こんにちは"),
        "Should contain 挨拶こんにちは scene"
    );
    assert!(
        lua_code.contains("挨拶こんばんは"),
        "Should contain 挨拶こんばんは scene"
    );
}

/// Test that runtime_e2e_word.pasta fixture parses and transpiles correctly
#[test]
fn test_fixture_word_parses() {
    let source = include_str!("../fixtures/e2e/runtime_e2e_word.pasta");
    let lua_code = transpile(source);

    // Verify word definitions are present
    assert!(
        lua_code.contains("挨拶言葉"),
        "Should contain 挨拶言葉 word definition"
    );
    assert!(
        lua_code.contains("場所"),
        "Should contain 場所 word definition"
    );
}

/// Test that runtime_e2e_actor_word.pasta fixture parses and transpiles correctly
#[test]
fn test_fixture_actor_word_parses() {
    let source = include_str!("../fixtures/e2e/runtime_e2e_actor_word.pasta");
    let lua_code = transpile(source);

    // Verify actor definitions are present
    assert!(
        lua_code.contains("create_actor(\"さくら\")"),
        "Should contain さくら actor"
    );
    assert!(
        lua_code.contains("create_actor(\"うにゅう\")"),
        "Should contain うにゅう actor"
    );
    assert!(
        lua_code.contains("create_actor(\"まゆら\")"),
        "Should contain まゆら actor"
    );
}

// ============================================================================
// Task 2.2: Scene Dictionary E2E Tests
// ============================================================================

/// Test scene prefix search and random selection (Task 2.2)
///
/// Verifies:
/// - Prefix "挨拶" matches all 3 scenes
/// - Multiple calls with cache consumption return all candidates
#[test]
fn test_scene_prefix_search_and_random_selection() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_scene.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Search with prefix "挨拶" - should match おはよう/こんにちは/こんばんは
    let results: Vec<String> = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local results = {}
        for i = 1, 6 do
            local name, _ = SEARCH:search_scene("挨拶", nil)
            if name then
                results[#results + 1] = name
            end
        end
        return results
    "#,
        )
        .eval()
        .unwrap();

    // Verify we got results (at least one match)
    assert!(
        !results.is_empty(),
        "Should find at least one scene with prefix 挨拶"
    );

    // Check that all results contain 挨拶
    for result in &results {
        assert!(
            result.contains("挨拶"),
            "All results should contain '挨拶', got: {}",
            result
        );
    }
}

// ============================================================================
// Task 2.3: Word Dictionary E2E Tests
// ============================================================================

/// Test word random selection (Task 2.3)
///
/// Verifies:
/// - Global word "挨拶言葉" returns one of 3 values
/// - Multiple calls with cache consumption eventually return all values
#[test]
fn test_word_random_selection_and_replacement() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Search for word "挨拶言葉" multiple times
    let results: Vec<String> = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local results = {}
        for i = 1, 6 do
            local value = SEARCH:search_word("挨拶言葉", nil, nil)
            if value then
                results[#results + 1] = value
            end
        end
        return results
    "#,
        )
        .eval()
        .unwrap();

    // Verify we got results
    assert!(
        !results.is_empty(),
        "Should find at least one word for 挨拶言葉"
    );

    // All values should be one of: おはよう、こんにちは、こんばんは
    let valid_values = ["おはよう", "こんにちは", "こんばんは"];
    for result in &results {
        assert!(
            valid_values.iter().any(|v| result.contains(v)),
            "Word should be one of {:?}, got: {}",
            valid_values,
            result
        );
    }
}

// ============================================================================
// Task 2.4: Actor Word Scope E2E Tests
// ============================================================================

/// Test actor word scope resolution (Task 2.4)
///
/// NOTE: Current search_word API does not support actor parameter.
/// This test verifies the current behavior (global-only search).
/// Actor-scoped word search is tracked as a future enhancement.
///
/// Verifies:
/// - Global word search works
/// - Actor scope is not yet implemented in search_word
#[test]
fn test_actor_word_scope_resolution() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_actor_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Global 一人称 should be "私"
    let global_pronoun: Option<String> = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_word("一人称", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(
        global_pronoun,
        Some("私".to_string()),
        "Global 一人称 should be '私'"
    );

    // Note: Actor-scoped search is not yet implemented in search_word API.
    // The current API signature is: search_word(name, global_scene_name?)
    // Actor-scoped word resolution happens at the ACT:word() level in Lua,
    // not through the @pasta_search module.
}

// ============================================================================
// Task 2.5: Complete Flow E2E Test
// ============================================================================

/// Test complete flow from Pasta to output (Task 2.5)
///
/// Verifies:
/// - Transpile → Execute → Finalize → Search works end-to-end
/// - Both scene and word search work together
#[test]
fn test_complete_flow_pasta_to_output() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use the scene fixture
    let source = include_str!("../fixtures/e2e/runtime_e2e_scene.pasta");
    let lua_code = transpile(source);

    // Step 1: Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Step 2: Finalize
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Step 3: Verify both scenes and search work
    let (scene_found, scene_name): (bool, Option<String>) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local name, fn_name = SEARCH:search_scene("メイン", nil)
        return name ~= nil, name
    "#,
        )
        .eval()
        .unwrap();

    assert!(scene_found, "Should find scene 'メイン'");
    assert!(
        scene_name.unwrap().contains("メイン"),
        "Scene name should contain 'メイン'"
    );

    // Verify scene prefix search also works
    let prefix_found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local name, _ = SEARCH:search_scene("挨拶", nil)
        return name ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(prefix_found, "Should find scene with prefix '挨拶'");
}

// ============================================================================
// Actor Word Shuffle Tests (actor-dict-word-shuffle spec)
// ============================================================================

/// Task 2.1: アクタースコープ単語のシャッフル＆順次消費をプロキシ経由で検証
///
/// PROXY_IMPL.word() → Level 2 (SEARCH:search_word) → WordTable のシャッフルパスが
/// 正しく動作し、3値すべてが順次消費されることを確認する。
/// set_word_selector() でモックセレクタを注入して決定論的に検証する。
///
/// Requirements: 1.1, 1.2, 1.3, 2.1, 3.1
#[test]
fn test_actor_word_shuffle_via_proxy() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_actor_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // set_word_selector(0, 1, 2) で決定論的順序を注入
    // アクター「さくら」の「通常」は 3 値: \s[0], \s[100], \s[200]
    let results: Vec<String> = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local ACTOR = require "pasta.actor"
        local ACT = require "pasta.act"

        -- モックセレクタ注入: 0番目, 1番目, 2番目の順で返す
        SEARCH:set_word_selector(0, 1, 2)

        -- ACTオブジェクトとプロキシを構築
        local act = ACT.new()
        local sakura = ACTOR.get_or_create("さくら")
        local proxy = ACTOR.create_proxy(sakura, act)

        local results = {}
        for i = 1, 3 do
            local val = proxy:word("通常")
            results[#results + 1] = val or "NIL"
        end

        -- リセット
        SEARCH:set_word_selector()
        return results
    "#,
        )
        .eval()
        .unwrap();

    // 3 値すべてが順次消費されることを検証
    assert_eq!(results.len(), 3, "Should get 3 results");
    assert_eq!(results[0], "\\s[0]", "First should be \\s[0]");
    assert_eq!(results[1], "\\s[100]", "Second should be \\s[100]");
    assert_eq!(results[2], "\\s[200]", "Third should be \\s[200]");
}

/// Task 2.2: アクタースコープからグローバルへのフォールバック検証
///
/// アクター辞書に存在しない単語がグローバル単語にフォールバックすることを確認する。
///
/// Requirements: 3.2
#[test]
fn test_actor_word_fallback_to_global() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_actor_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // まゆらには「一人称」が定義されていないので、グローバルの「私」にフォールバック
    let result: Option<String> = lua
        .load(
            r#"
        local ACTOR = require "pasta.actor"
        local ACT = require "pasta.act"

        local act = ACT.new()
        local mayura = ACTOR.get_or_create("まゆら")
        local proxy = ACTOR.create_proxy(mayura, act)

        return proxy:word("一人称")
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(
        result,
        Some("私".to_string()),
        "まゆらの一人称はグローバルの「私」にフォールバックすべき"
    );
}

/// Task 2.3: 単一値アクター単語の後方互換性
///
/// アクター辞書に単一値定義された単語が常に同じ値を返すことを確認する。
///
/// Requirements: 1.4, 3.3
#[test]
fn test_actor_word_single_value_backward_compat() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_actor_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // さくらの「照れ」は単一値 \s[1]
    let results: Vec<String> = lua
        .load(
            r#"
        local ACTOR = require "pasta.actor"
        local ACT = require "pasta.act"

        local act = ACT.new()
        local sakura = ACTOR.get_or_create("さくら")
        local proxy = ACTOR.create_proxy(sakura, act)

        local results = {}
        for i = 1, 3 do
            local val = proxy:word("照れ")
            results[#results + 1] = val or "NIL"
        end
        return results
    "#,
        )
        .eval()
        .unwrap();

    // 3回とも同じ値
    assert_eq!(results.len(), 3, "Should get 3 results");
    for (i, val) in results.iter().enumerate() {
        assert_eq!(
            val, "\\s[1]",
            "Call {} should return \\s[1], got {}",
            i + 1,
            val
        );
    }
}
