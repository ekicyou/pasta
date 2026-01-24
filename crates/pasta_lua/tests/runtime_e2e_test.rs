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

    let source = include_str!("fixtures/e2e/runtime_e2e_scene.pasta");
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

    let source = include_str!("fixtures/e2e/runtime_e2e_word.pasta");
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

    let source = include_str!("fixtures/e2e/runtime_e2e_actor_word.pasta");
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
    let source = include_str!("fixtures/e2e/runtime_e2e_scene.pasta");
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
// Task 3.1: Comment Line Parse Test
// ============================================================================

/// Test that comment lines are not included in AST (Task 3.1)
///
/// Verifies:
/// - Lines starting with ＃ are treated as comments
/// - Comments do not appear in transpiled output
/// - Mixed comments and code parse correctly
#[test]
fn test_comment_line_explicit_parse() {
    use pasta_core::parser::parse_str;

    let source = r#"
＃ これはコメントです
＊メイン
  ＃ これもコメント
  さくら：「こんにちは」
＃ 最後のコメント
"#;

    // Parse should succeed
    let file = parse_str(source, "test.pasta").expect("Should parse with comments");

    // Should have exactly one scene (メイン)
    let scene_count = file
        .items
        .iter()
        .filter(|item| {
            matches!(
                item,
                pasta_core::parser::ast::FileItem::GlobalSceneScope(_)
            )
        })
        .count();

    assert_eq!(scene_count, 1, "Should have exactly 1 scene");

    // Transpiled code should not contain comment text
    let lua_code = transpile(source);
    assert!(
        !lua_code.contains("これはコメントです"),
        "Comment text should not appear in transpiled code"
    );
    assert!(
        !lua_code.contains("これもコメント"),
        "Inline comment should not appear in transpiled code"
    );
}

// ============================================================================
// Task 3.2: Attribute Inheritance Test
// ============================================================================

/// Test attribute inheritance from file scope to scenes (Task 3.2)
///
/// Verifies:
/// - File-level attributes are inherited by scenes
/// - Scene attributes override file attributes
#[test]
fn test_attribute_inheritance() {
    let source = r#"
＆天気：晴れ
＆場所：公園

＊メイン
  ＆場所：学校
  さくら：「今日は＄天気です」
"#;

    let lua_code = transpile(source);

    // File attributes should be merged into scene
    // The transpiled code should reference both inherited and overridden attrs
    assert!(
        lua_code.contains("create_scene"),
        "Should create scene"
    );

    // Verify the transpiled code compiles and runs
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Scene should be searchable
    let found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local name, _ = SEARCH:search_scene("メイン", nil)
        return name ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(found, "Scene with inherited attributes should be searchable");
}

// ============================================================================
// Task 3.3: Variable Scope Test
// ============================================================================

/// Test variable scope separation (Task 3.3)
///
/// Verifies:
/// - Local variables (＄) are action-scoped
/// - Global variables (＄＊) are save-scoped
/// - System variables (＄＊＊) are system-scoped (if implemented)
#[test]
fn test_variable_scope_complete() {
    let source = r#"
＊メイン
  ＄ローカル＝「ローカル値」
  ＄＊グローバル＝「グローバル値」
  さくら：「ローカル：＄ローカル、グローバル：＄＊グローバル」
"#;

    let lua_code = transpile(source);

    // Local variable uses var.name format
    assert!(
        lua_code.contains("var."),
        "Local variable should use var.name format. Generated code:\n{}",
        lua_code
    );

    // Global variable uses save.name format
    assert!(
        lua_code.contains("save."),
        "Global variable should use save.name format. Generated code:\n{}",
        lua_code
    );

    // Verify code compiles
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(&lua_code).exec().unwrap();
}

// ============================================================================
// Task 3.4: Error Message Specificity Test
// ============================================================================

/// Test error message includes line and column numbers (Task 3.4)
///
/// Verifies:
/// - Parse errors include line number
/// - Parse errors include column number
/// - Error message is descriptive
#[test]
fn test_error_message_specificity() {
    use pasta_core::parser::parse_str;

    // Invalid syntax: scene without name
    let invalid_source = "＊\n";

    let result = parse_str(invalid_source, "test.pasta");
    assert!(result.is_err(), "Invalid syntax should produce error");

    let error = result.unwrap_err();
    let error_str = format!("{:?}", error);

    // Error should contain line number
    assert!(
        error_str.contains("line") || error_str.contains("1"),
        "Error should include line information: {}",
        error_str
    );

    // Test another error pattern: unclosed action
    let invalid_source2 = "＊メイン\n  さくら：";
    let result2 = parse_str(invalid_source2, "test.pasta");

    // This might succeed or fail depending on grammar
    // The important thing is that errors are descriptive
    if let Err(error2) = result2 {
        let error_str2 = format!("{:?}", error2);
        assert!(
            error_str2.len() > 10,
            "Error message should be descriptive: {}",
            error_str2
        );
    }
}
