//! Runtime Scene E2E Tests for pasta_lua.

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

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

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
// E2E Fixture Tests
// ============================================================================

/// Test that runtime_e2e_scene.pasta fixture parses and transpiles correctly
#[test]
fn test_fixture_scene_parses() {
    let source = include_str!("../fixtures/e2e/runtime_e2e_scene.pasta");
    let lua_code = transpile(source);

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
// Scene Dictionary E2E Tests
// ============================================================================

/// Test scene prefix search and random selection
#[test]
fn test_scene_prefix_search_and_random_selection() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_scene.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

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

    assert!(
        !results.is_empty(),
        "Should find at least one scene with prefix 挨拶"
    );

    for result in &results {
        assert!(
            result.contains("挨拶"),
            "All results should contain '挨拶', got: {}",
            result
        );
    }
}

// ============================================================================
// Word Dictionary E2E Tests
// ============================================================================

/// Test word random selection
#[test]
fn test_word_random_selection_and_replacement() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

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

    assert!(
        !results.is_empty(),
        "Should find at least one word for 挨拶言葉"
    );

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
// Actor Word Scope E2E Tests
// ============================================================================

/// Test actor word scope resolution
///
/// NOTE: Current search_word API does not support actor parameter.
/// This test verifies the current behavior (global-only search).
/// Actor-scoped word search is tracked as a future enhancement.
#[test]
fn test_actor_word_scope_resolution() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_actor_word.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

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
// Complete Flow E2E Test
// ============================================================================

/// Test complete flow from Pasta to output
#[test]
fn test_complete_flow_pasta_to_output() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_scene.pasta");
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

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
// Actor Word Shuffle Tests
// ============================================================================

/// アクタースコープ単語のシャッフル＆順次消費をプロキシ経由で検証
///
/// PROXY_IMPL.word() → Level 2 (SEARCH:search_word) → WordTable のシャッフルパスが
/// 正しく動作し、3値すべてが順次消費されることを確認する。
/// set_word_selector() でモックセレクタを注入して決定論的に検証する。
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

    assert_eq!(results.len(), 3, "Should get 3 results");
    assert_eq!(results[0], "\\s[0]", "First should be \\s[0]");
    assert_eq!(results[1], "\\s[100]", "Second should be \\s[100]");
    assert_eq!(results[2], "\\s[200]", "Third should be \\s[200]");
}

/// アクタースコープからグローバルへのフォールバック検証
///
/// アクター辞書に存在しない単語がグローバル単語にフォールバックすることを確認する。
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

/// 単一値アクター単語の後方互換性
///
/// アクター辞書に単一値定義された単語が常に同じ値を返すことを確認する。
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

    assert_eq!(results.len(), 3, "Should get 3 results");
    for (i, val) in results.iter().enumerate() {
        assert_eq!(
            val,
            "\\s[1]",
            "Call {} should return \\s[1], got {}",
            i + 1,
            val
        );
    }
}

// ============================================================================
// 空ローカルシーン E2E Tests
// ============================================================================

/// 名前付き空ローカルシーンのE2Eテスト
///
/// `・分岐会話無し` の直後に `・分岐会話アリ` が続く構文をパース→トランスパイル→実行する。
/// アクション行なし分岐と通常分岐の混在を検証する。
#[test]
fn test_e2e_empty_local_scene() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = "＊サンプル\n\u{3000}ぱすた：分岐するよ！\n\u{3000}\u{3000}＞分岐\n\n\u{3000}\u{3000}・分岐会話無し\n\u{3000}\u{3000}・分岐会話アリ\n\u{3000}ぱすた：分岐したよ！\n";
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("サンプル", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.contains("サンプル"),
        "Global name should contain 'サンプル', got: {}",
        result.0
    );
}

/// 空スタートスコープのE2Eテスト
///
/// `local_start_scene_scope` がアクション行0個で直接ローカルシーンに分かれるケースを検証する。
#[test]
fn test_e2e_empty_start_scope_local_scene() {
    let lua = create_runtime_with_finalize().unwrap();

    let source = "＊空スタート\n\u{3000}\u{3000}・分岐A\n\u{3000}ぱすた：Aルート！\n";
    let lua_code = transpile(source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("空スタート", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.contains("空スタート"),
        "Global name should contain '空スタート', got: {}",
        result.0
    );
}
