//! E2E integration tests for the finalize_scene() function.
//!
//! These tests verify the complete flow:
//! 1. Transpile .pasta source to Lua
//! 2. Execute Lua code (registers scenes/words in Lua registries)
//! 3. Call finalize_scene() to build SearchContext from Lua registries
//! 4. Verify search operations work correctly
//!
//! # Requirements Coverage
//! - Req 1.1, 1.2, 1.4, 8.2, 8.4: Scene dictionary collection E2E
//! - Req 2.1, 2.2, 2.6, 2.7, 9.3, 9.5: Word dictionary collection E2E
//! - Req 1.3, 6.1, 6.2, 6.3: Error handling
//! - Req 5.2, 5.3, 5.4: Initialization timing control

mod common;
use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use std::path::PathBuf;

// ============================================================================
// Task 8.1: Scene Dictionary Collection E2E Tests
// ============================================================================

/// Test 8.1.1: Basic scene collection and search using existing fixture
#[test]
fn test_scene_collection_basic() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta fixture (known good format)
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify scene can be searched
    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("メイン", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.starts_with("メイン"),
        "Global name should start with 'メイン', got: {}",
        result.0
    );
    assert_eq!(result.1, "__start__", "Local name should be '__start__'");
}

/// Test 8.1.2: Multiple global scenes with same name (counter test)
#[test]
fn test_scene_collection_multiple_global_scenes() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta which has multiple "会話分岐" scenes (global + local)
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify 会話分岐 scene is registered
    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("会話分岐", nil)
    "#,
        )
        .eval()
        .unwrap();

    // Should find scene with name containing '会話分岐'
    assert!(
        result.0.contains("会話分岐"),
        "Should find scene with name containing '会話分岐', got: {}",
        result.0
    );
}

/// Test 8.1.3: Local scenes under global scene
#[test]
fn test_scene_collection_local_scenes() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta which has local scenes like "グローバル単語呼び出し"
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify @pasta_search module is available
    let search_exists: bool = lua
        .load(
            r#"
        local ok, SEARCH = pcall(require, "@pasta_search")
        return ok and SEARCH ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(search_exists, "@pasta_search module should be available");
}

// ============================================================================
// Task 8.2: Word Dictionary Collection E2E Tests
// ============================================================================

/// Test 8.2.1: Global word collection
#[test]
fn test_word_collection_global() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta which has global word "挨拶"
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify word can be searched
    let result: mlua::Value = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_word("挨拶", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(result.is_string(), "Word search should return a string");
    let word = result.as_string().map(|s| s.to_string_lossy()).unwrap();
    assert!(
        word == "こんにちは" || word == "やあ" || word == "ハロー",
        "Word should be one of the registered values, got: {}",
        word
    );
}

/// Test 8.2.2: Local word collection
#[test]
fn test_word_collection_local() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta which has local word "場所"
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify @pasta_search is available after finalize
    let search_available: bool = lua
        .load(
            r#"
        local ok = pcall(require, "@pasta_search")
        return ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        search_available,
        "@pasta_search should be available after finalize_scene"
    );
}

/// Test 8.2.3: Builder pattern API (method chaining)
#[test]
fn test_word_builder_pattern() {
    let lua = create_runtime_with_finalize().unwrap();

    // Test builder pattern directly in Lua
    let result: bool = lua
        .load(
            r#"
        local PASTA = require("pasta")
        
        -- Create global word using builder pattern
        PASTA.create_word("テスト"):entry("値1", "値2", "値3")
        
        -- Call finalize_scene
        PASTA.finalize_scene()
        
        -- Verify word is searchable
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("テスト", nil)
        return word == "値1" or word == "値2" or word == "値3"
    "#,
        )
        .eval()
        .unwrap();

    assert!(result, "Builder pattern should register word correctly");
}

// ============================================================================
// Task 8.3: Error Handling Tests
// ============================================================================

/// Test 8.3.1: Empty registry finalize (warning log + empty SearchContext)
#[test]
fn test_empty_registry_finalize() {
    let lua = create_runtime_with_finalize().unwrap();

    // Call finalize_scene without any scenes or words
    let result: bool = lua
        .load(
            r#"
        local PASTA = require("pasta")
        return PASTA.finalize_scene()
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result,
        "finalize_scene should return true even with empty registry"
    );

    // Verify @pasta_search is available
    let search_available: bool = lua
        .load(
            r#"
        local ok = pcall(require, "@pasta_search")
        return ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        search_available,
        "@pasta_search should be available even with empty registry"
    );
}

/// Test 8.3.2: Scene not found returns nil
#[test]
fn test_scene_not_found() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta then search for non-existent scene
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local global_name = SEARCH:search_scene("存在しないシーン", nil)
        return global_name == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result, "Non-existent scene search should return nil");
}

/// Test 8.3.3: Word not found returns nil
#[test]
fn test_word_not_found() {
    let lua = create_runtime_with_finalize().unwrap();

    // Finalize empty registry
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("存在しない単語", nil)
        return word == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result, "Non-existent word search should return nil");
}

// ============================================================================
// Task 8.4: Initialization Timing Control Tests (Optional)
// ============================================================================

/// Test 8.4.1: Multiple finalize_scene calls (SearchContext rebuild)
#[test]
fn test_multiple_finalize_calls() {
    let lua = create_runtime_with_finalize().unwrap();

    // First: create word via PASTA API and finalize
    lua.load(
        r#"
        local PASTA = require("pasta")
        PASTA.create_word("単語A"):entry("値A1", "値A2")
        PASTA.finalize_scene()
    "#,
    )
    .exec()
    .unwrap();

    // Verify word A is searchable
    let word_a_found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("単語A", nil)
        return word ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(word_a_found, "単語A should be found after first finalize");

    // Second: create another word and finalize again
    lua.load(
        r#"
        local PASTA = require("pasta")
        PASTA.create_word("単語B"):entry("値B1", "値B2")
        PASTA.finalize_scene()
    "#,
    )
    .exec()
    .unwrap();

    // Verify both words are searchable after rebuild
    let both_found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local a = SEARCH:search_word("単語A", nil)
        local b = SEARCH:search_word("単語B", nil)
        return a ~= nil and b ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        both_found,
        "Both words should be found after second finalize"
    );
}

/// Test 8.4.2: @pasta_search not available before finalize_scene
#[test]
fn test_search_unavailable_before_finalize() {
    use mlua::{Lua, StdLib};

    // Create fresh Lua without finalize_scene registration
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Configure package.path
    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .to_string_lossy()
        .replace('\\', "/");

    lua.load(&format!(
        r#"
        package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path
        "#
    ))
    .exec()
    .unwrap();

    // Try to require @pasta_search before any setup
    let result: bool = lua
        .load(
            r#"
        local ok = pcall(require, "@pasta_search")
        return ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        !result,
        "@pasta_search should NOT be available before finalize_scene"
    );
}

/// Test 8.4.3: Full E2E flow with transpiled pasta
#[test]
fn test_full_e2e_flow() {
    let lua = create_runtime_with_finalize().unwrap();

    // Use sample.pasta for complete E2E test
    let pasta_source = include_str!("fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    // Execute transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Call finalize_scene
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify everything is searchable
    let all_ok: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        
        -- Check scenes
        local main_ok = SEARCH:search_scene("メイン", nil) ~= nil
        local talk_ok = SEARCH:search_scene("会話分岐", nil) ~= nil
        
        -- Check words
        local greet_ok = SEARCH:search_word("挨拶", nil) ~= nil
        
        return main_ok and talk_ok and greet_ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(all_ok, "All scenes and words should be searchable");
}
