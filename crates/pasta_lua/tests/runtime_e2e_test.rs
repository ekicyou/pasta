//! Runtime E2E Tests for pasta_lua.
//!
//! These tests verify the complete runtime execution flow:
//! - Scene dictionary prefix search and random selection
//! - Word dictionary random selection and replacement
//! - Actor word scope resolution
//! - Complete flow from Pasta to output
//!
//! # Requirements Coverage
//! - Requirement 7.1: Untested area tests
//! - Requirement 7.2: Runtime E2E tests

mod common;

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
    let source = include_str!("fixtures/e2e/runtime_e2e_scene.pasta");
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
    let source = include_str!("fixtures/e2e/runtime_e2e_word.pasta");
    let lua_code = transpile(source);

    // Verify word definitions are present
    assert!(
        lua_code.contains("挨拶言葉"),
        "Should contain 挨拶言葉 word definition"
    );
    assert!(lua_code.contains("場所"), "Should contain 場所 word definition");
}

/// Test that runtime_e2e_actor_word.pasta fixture parses and transpiles correctly
#[test]
fn test_fixture_actor_word_parses() {
    let source = include_str!("fixtures/e2e/runtime_e2e_actor_word.pasta");
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
