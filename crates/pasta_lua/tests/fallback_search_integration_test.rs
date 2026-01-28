//! Integration tests for actor-word-dictionary feature.
//!
//! Tests for parser/transpiler functionality and basic runtime behavior.
//! Complex fallback search tests are covered by finalize_scene_test.rs and runtime_e2e_test.rs.

use pasta_core::parse_str;
use pasta_lua::LuaTranspiler;

/// Test fixture for comprehensive fallback test
/// 構文: ％ = アクター定義, ＊ = シーン/トーク, ＠キー：値 = 単語定義
/// code_blockはインデントなしでアクター定義の後に配置
const COMPREHENSIVE_FALLBACK_PASTA: &str = r#"％さくら
　＠表情：\s[0]
　＠表情：\s[1]
```lua
function ACTOR.時刻(act)
    return "朝"
end

function ACTOR.天気(act)
    return "アクター関数の天気"
end
```

％うにゅう
　＠表情：\s[10]
　＠表情：\s[11]

＊メインシーン
　さくら：テスト
"#;

/// Test that comprehensive fallback fixture parses correctly
#[test]
fn test_comprehensive_fallback_parses() {
    let result = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let file = result.unwrap();

    // Verify structure
    let actors: Vec<_> = file
        .items
        .iter()
        .filter(|i| matches!(i, pasta_core::parser::FileItem::ActorScope(_)))
        .collect();
    assert_eq!(actors.len(), 2, "Expected 2 actors (さくら, うにゅう)");

    let scenes: Vec<_> = file
        .items
        .iter()
        .filter(|i| matches!(i, pasta_core::parser::FileItem::GlobalSceneScope(_)))
        .collect();
    assert_eq!(scenes.len(), 1, "Expected 1 scene (メインシーン)");
}

/// Test that actor code_blocks are parsed
#[test]
fn test_actor_code_blocks_parsed() {
    let file = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta").unwrap();

    for item in &file.items {
        if let pasta_core::parser::FileItem::ActorScope(actor) = item {
            if actor.name == "さくら" {
                assert!(
                    !actor.code_blocks.is_empty(),
                    "さくら should have code_blocks"
                );
                assert_eq!(actor.code_blocks[0].language.as_deref(), Some("lua"));
                assert!(actor.code_blocks[0].content.contains("function ACTOR.時刻"));
            }
        }
    }
}

/// Test that transpiler generates ACTOR:create_word API format for actor words
#[test]
fn test_transpiler_actor_array_format() {
    let file = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Check symmetric API format output
    assert!(
        lua_code.contains("ACTOR:create_word"),
        "Expected ACTOR:create_word API format"
    );
    assert!(lua_code.contains("[="), "Expected long string literals");
}

/// Test that transpiler expands actor code_blocks
#[test]
fn test_transpiler_expands_code_blocks() {
    let file = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Check code block expansion
    assert!(
        lua_code.contains("function ACTOR.時刻(act)"),
        "Expected ACTOR.時刻 function"
    );
    assert!(
        lua_code.contains("function ACTOR.天気(act)"),
        "Expected ACTOR.天気 function"
    );
}

/// Test that transpiler generates ACTOR:create_word format for actor words
#[test]
fn test_transpiler_actor_word_format() {
    let file = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Actor words use symmetric create_word API (new behavior)
    assert!(
        lua_code.contains(r#"ACTOR:create_word("#),
        "Expected ACTOR:create_word API format"
    );
}

/// Test basic structure of generated Lua for actors
#[test]
fn test_generated_lua_actor_structure() {
    let file = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Check actor creation
    assert!(
        lua_code.contains("PASTA.create_actor(\"さくら\")"),
        "Expected さくら actor creation"
    );
    assert!(
        lua_code.contains("PASTA.create_actor(\"うにゅう\")"),
        "Expected うにゅう actor creation"
    );

    // Check do-end blocks for actor scope (format may vary)
    assert!(
        lua_code.contains("do") && lua_code.contains("end"),
        "Expected do-end blocks"
    );
}

/// Test that actor code_blocks are expanded (current fixture only has actor code blocks)
#[test]
fn test_actor_code_blocks_are_expanded() {
    let file = parse_str(COMPREHENSIVE_FALLBACK_PASTA, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Actor code block should be expanded (current fixture has actor code blocks)
    assert!(
        lua_code.contains("function ACTOR.時刻(act)"),
        "Expected ACTOR.時刻 function from code block"
    );
    assert!(
        lua_code.contains("function ACTOR.天気(act)"),
        "Expected ACTOR.天気 function from code block"
    );
}

// ==============================================
// Runtime tests (only tests that don't require finalize_scene)
// Complex fallback tests are in finalize_scene_test.rs and runtime_e2e_test.rs
// ==============================================

use std::path::PathBuf;

/// Setup test runtime with pasta modules loaded
fn setup_runtime() -> mlua::Lua {
    use mlua::{Lua, StdLib};

    // Create Lua VM with safe standard libraries
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    // Configure package.path to include pasta scripts directory
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
    .expect("Failed to configure package.path");

    lua
}

/// T1: アクター完全一致（L1）- 関数
/// さくら.word("時刻") → "朝"（アクター関数）
/// Note: This test works because actor[name] exact match is Lua-only (no SEARCH API needed)
#[test]
fn test_t1_actor_function_exact_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Create actor and add function
        local sakura = ACTOR.get_or_create("さくら")
        sakura.時刻 = function(act) return "朝" end
        
        -- Create mock act
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("時刻")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert_eq!(result, "朝");
}

/// T12: 関数優先テスト（オーバーライド）
/// さくら.word("天気") → アクター関数が辞書より優先
/// Note: This test works because actor[name] exact match takes priority (no SEARCH API needed)
#[test]
fn test_t12_function_priority_over_dict() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register actor dict
        WORD.create_actor("さくら", "天気"):entry("辞書の天気")
        
        -- Create actor with function (should override dict)
        local sakura = ACTOR.get_or_create("さくら")
        sakura.天気 = function(act) return "アクター関数の天気" end
        
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("天気")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert_eq!(result, "アクター関数の天気");
}

/// T15: 空文字キー → nil
/// さくら.word("") → nil
#[test]
fn test_t15_empty_key_returns_nil() {
    let lua = setup_runtime();

    let result: mlua::Value = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Create actor with some words
        WORD.create_actor("さくら", "表情"):entry("\\s[0]")
        
        -- Create actor
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result.is_nil(),
        "Expected nil for empty key, got: {:?}",
        result
    );
}
