//! E2E integration tests for local scene call resolution.
//!
//! These tests verify:
//! 1. Transpiled DSL → Lua execution → finalize_scene() → act:call
//!    resolves local scenes correctly through the finalize path (Path B).
//! 2. Duplicate local scenes (same name) are resolved via random selection.
//! 3. Prefix matching correctly resolves partial local scene names.
//!
//! # Requirements Coverage
//! - Req 4.1: E2E integration tests for local scene call

use crate::common;
use common::e2e_helpers::{create_runtime_with_finalize, transpile};

// ============================================================================
// Task 5.2: Simple local scene call test
// ============================================================================

/// Verify that a local scene defined in DSL can be resolved and executed
/// through the finalize path (transpile → execute → finalize → SCENE.search → call).
#[test]
fn test_local_scene_call_simple() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = "\
＊テスト
　　＞サブシーン
　・サブシーン
　　さくら：サブシーンだよ。
";
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify SCENE.search finds the local scene via finalize path
    let found: bool = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"
        local gn = SCENE.get_global_table("テスト1").__global_name__
        local result = SCENE.search("サブシーン", gn)
        return result ~= nil and type(result.func) == "function"
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        found,
        "Local scene 'サブシーン' should be found and callable via finalize path"
    );
}

// ============================================================================
// Task 5.3: Duplicate local scene random selection test
// ============================================================================

/// Verify that duplicate local scenes (same name) can both be resolved
/// through the finalize path without errors.
#[test]
fn test_local_scene_call_duplicate_random_selection() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = "\
＊テスト
　　＞Head0
　・Head0
　　さくら：バリエーション１。
　・Head0
　　さくら：バリエーション２。
";
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Call multiple times — both implementations should resolve without error
    let success: bool = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"
        local gn = SCENE.get_global_table("テスト1").__global_name__
        for i = 1, 10 do
            local result = SCENE.search("Head0", gn)
            if result == nil or type(result.func) ~= "function" then
                return false
            end
        end
        return true
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        success,
        "Duplicate local scenes 'Head0' should always resolve without error"
    );
}

// ============================================================================
// Task 5.4: Prefix match search test
// ============================================================================

/// Verify that prefix matching resolves local scenes correctly
/// (e.g., searching "Head" matches both "Head0_1" and "Head1_1").
#[test]
fn test_local_scene_call_prefix_match() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = "\
＊テスト
　　＞Head0
　・Head0
　　さくら：ゼロ。
　・Head1
　　さくら：ワン。
";
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Prefix search "Head" should match one of Head0_1 or Head1_1
    let success: bool = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"
        local gn = SCENE.get_global_table("テスト1").__global_name__
        local result = SCENE.search("Head", gn)
        if result == nil then
            return false
        end
        -- local_name should contain "Head"
        return result.local_name ~= nil and string.find(result.local_name, "Head") ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        success,
        "Prefix search 'Head' should find a local scene via finalize path"
    );
}
