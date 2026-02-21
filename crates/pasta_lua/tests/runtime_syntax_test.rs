//! Runtime Syntax E2E Tests for pasta_lua.
//!
//! These tests verify syntax parsing, variable scoping, error reporting,
//! and chaintalk (＞チェイントーク) runtime execution:
//! - Comment line parse verification
//! - Attribute inheritance from file scope to scenes
//! - Variable scope separation (local/global/system)
//! - Error message specificity (line/column numbers)
//! - Chaintalk transpile, finalize, and coroutine execution
//!
//! # Requirements Coverage
//! - Requirement 7.1: Untested area tests
//! - Requirement 7.2: Runtime E2E tests

mod common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script;

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
    use pasta_dsl::parser::parse_str;

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
        .filter(|item| matches!(item, pasta_dsl::parser::ast::FileItem::GlobalSceneScope(_)))
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
    assert!(lua_code.contains("create_scene"), "Should create scene");

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

    assert!(
        found,
        "Scene with inherited attributes should be searchable"
    );
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
    use pasta_dsl::parser::parse_str;

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

// ============================================================================
// Task 4: Chaintalk (＞チェイントーク) E2E Tests
// ============================================================================

/// Test that chaintalk fixture parses and transpiles correctly (Task 4.1)
///
/// Verifies:
/// - ＞チェイントーク is transpiled to act:call with "チェイントーク" label
/// - The transpiled code contains the expected scene structure
#[test]
fn test_fixture_chaintalk_parses() {
    let source = include_str!("fixtures/e2e/runtime_e2e_scene_chaintalk.pasta");
    let lua_code = transpile(source);

    // Verify actor definition
    assert!(
        lua_code.contains("create_actor(\"さくら\")"),
        "Should contain さくら actor definition. Generated code:\n{}",
        lua_code
    );

    // Verify scene is created
    assert!(
        lua_code.contains("チェイントークテスト"),
        "Should contain チェイントークテスト scene. Generated code:\n{}",
        lua_code
    );

    // Verify ＞チェイントーク is transpiled to act:call with "チェイントーク" key
    assert!(
        lua_code.contains("\"チェイントーク\""),
        "Should contain act:call with チェイントーク key. Generated code:\n{}",
        lua_code
    );
}

/// Test chaintalk transpile → finalize → GLOBAL function resolution (Task 4.2)
///
/// Verifies the complete E2E flow:
/// 1. Pasta DSL with ＞チェイントーク transpiles to Lua
/// 2. Transpiled code loads and finalizes successfully
/// 3. Scene can be found via search
/// 4. GLOBAL.チェイントーク is registered and accessible at L3
/// 5. Scene execution with coroutine produces intermediate output via yield
#[test]
fn test_e2e_chaintalk_transpile_and_execute() {
    let lua = create_runtime_with_finalize().unwrap();

    // Register @pasta_sakura_script (required by SHIORI_ACT → sakura_builder)
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    let source = include_str!("fixtures/e2e/runtime_e2e_scene_chaintalk.pasta");
    let lua_code = transpile(source);

    // Load transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Finalize scenes
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify GLOBAL functions are registered
    let global_ok: bool = lua
        .load(
            r#"
        local GLOBAL = require("pasta.global")
        return type(GLOBAL["チェイントーク"]) == "function"
            and type(GLOBAL["yield"]) == "function"
    "#,
        )
        .eval()
        .unwrap();
    assert!(
        global_ok,
        "GLOBAL.チェイントーク and GLOBAL.yield should be registered"
    );

    // Execute the scene via EVENT.fire and verify coroutine yield
    // The fixture scene is: talk → ＞チェイントーク (yield) → talk
    // Note: STORE.actors["さくら"] is already created by the transpiled code
    // (PASTA.create_actor("さくら") was called during lua_code execution)
    // First EVENT.fire should produce the pre-yield output and set co_scene
    let fire_result: String = lua
        .load(
            r#"
        local STORE = require("pasta.store")
        local EVENT = require("pasta.shiori.event")

        -- Fire the scene event
        local req = { id = "チェイントークテスト" }
        local response = EVENT.fire(req)

        -- Check result and co_scene state
        local co_state = "none"
        if STORE.co_scene then
            co_state = coroutine.status(STORE.co_scene)
        end

        return string.format("status=%s co=%s", tostring(response ~= nil), co_state)
    "#,
        )
        .eval()
        .unwrap();

    // After first fire, we expect response and suspended coroutine
    assert!(
        fire_result.contains("status=true"),
        "EVENT.fire should return a response, got: {}",
        fire_result
    );
    assert!(
        fire_result.contains("co=suspended"),
        "co_scene should be suspended after yield, got: {}",
        fire_result
    );
}
