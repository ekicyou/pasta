//! Integration tests for actor-word-dictionary feature.
//!
//! Tests 6-level fallback word search: Actor → Scene → Global
//! Each level: exact match → prefix match

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
// 6-level fallback search runtime tests (T1-T15)
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

/// T2: アクター辞書前方一致（L2）
/// さくら.word("表情") → "\s[0]" or "\s[1]"
/// NOTE: Requires finalize_scene() to setup SEARCH API - see new integration tests
#[test]
#[ignore = "Requires finalize_scene() - replaced by new act-word-global-dict-search tests"]
fn test_t2_actor_dict_prefix_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register actor words
        WORD.create_actor("さくら", "表情"):entry("\\s[0]"):entry("\\s[1]")
        
        -- Create actor and proxy
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("表情")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result == "\\s[0]" || result == "\\s[1]",
        "Expected \\s[0] or \\s[1], got: {}",
        result
    );
}

/// T3: シーン完全一致（L3）- 関数
/// さくら.word("日付") → "1月1日"（シーン関数）
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t3_scene_function_exact_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Create actor
        local sakura = ACTOR.get_or_create("さくら")
        
        -- Create scene with function
        local scene = { name = "テストシーン" }
        scene.日付 = function(act) return "1月1日" end
        
        -- Create act with scene
        local act = { current_scene = scene }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("日付")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert_eq!(result, "1月1日");
}

/// T4: シーン辞書前方一致（L4）
/// さくら.word("季節") → "春" or "夏"
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t4_scene_dict_prefix_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register scene words using __global_name__
        WORD.create_local("テストシーン", "季節"):entry("春"):entry("夏")
        
        -- Create actor
        local sakura = ACTOR.get_or_create("さくら")
        
        -- Create scene with __global_name__
        local scene = { name = "テストシーン", __global_name__ = "テストシーン" }
        
        -- Create act with scene
        local act = { current_scene = scene }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("季節")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result == "春" || result == "夏",
        "Expected 春 or 夏, got: {}",
        result
    );
}

/// T5: グローバル完全一致（L5）- 関数
/// さくら.word("時報") → "正午です"
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t5_global_function_exact_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        local GLOBAL = require("pasta.global")
        
        -- Add global function
        GLOBAL.時報 = function(act) return "正午です" end
        
        -- Create actor
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("時報")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert_eq!(result, "正午です");
}

/// T6: グローバル辞書前方一致（L6）
/// さくら.word("挨拶") → "こんにちは" or "おはよう"
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t6_global_dict_prefix_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register global words
        WORD.create_global("挨拶"):entry("こんにちは"):entry("おはよう")
        
        -- Create actor
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("挨拶")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result == "こんにちは" || result == "おはよう",
        "Expected こんにちは or おはよう, got: {}",
        result
    );
}

/// T7: フォールスルー（アクター→グローバル）
/// うにゅう.word("天気") → アクター辞書なし→グローバルへ
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t7_fallthrough_actor_to_global() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register only global words (no actor words for うにゅう)
        WORD.create_global("天気"):entry("雨"):entry("雪"):entry("台風")
        
        -- Create actor without any words
        local unyuu = ACTOR.get_or_create("うにゅう")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(unyuu, act)
        
        return proxy:word("天気")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result == "雨" || result == "雪" || result == "台風",
        "Expected 雨, 雪, or 台風, got: {}",
        result
    );
}

/// T8: 別アクターの辞書を参照しない
/// うにゅう.word("表情") → うにゅう自身の辞書から
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t8_actor_isolation() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register different words for different actors
        WORD.create_actor("さくら", "表情"):entry("\\s[0]"):entry("\\s[1]")
        WORD.create_actor("うにゅう", "表情"):entry("\\s[10]"):entry("\\s[11]")
        
        -- Create うにゅう actor
        local unyuu = ACTOR.get_or_create("うにゅう")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(unyuu, act)
        
        return proxy:word("表情")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result == "\\s[10]" || result == "\\s[11]",
        "Expected \\s[10] or \\s[11], got: {}",
        result
    );
}

/// T9: L2前方一致（プレフィックス検索）
/// さくら.word("表") → "\s[0]" or "\s[1]"
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t9_actor_prefix_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register actor words with key "表情"
        WORD.create_actor("さくら", "表情"):entry("\\s[0]"):entry("\\s[1]")
        
        -- Create actor and proxy
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        -- Search with prefix "表"
        return proxy:word("表")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(
        result == "\\s[0]" || result == "\\s[1]",
        "Expected \\s[0] or \\s[1], got: {}",
        result
    );
}

/// T10: L4前方一致（複数キーマッチ）
/// さくら.word("季") → "春", "夏", "暖かい", "涼しい"のいずれか
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t10_scene_prefix_multiple_keys() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register multiple scene words with prefix "季"
        WORD.create_local("テストシーン", "季節"):entry("春"):entry("夏")
        WORD.create_local("テストシーン", "季節感"):entry("暖かい"):entry("涼しい")
        
        -- Create actor with scene
        local sakura = ACTOR.get_or_create("さくら")
        local scene = { name = "テストシーン", __global_name__ = "テストシーン" }
        local act = { current_scene = scene }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        -- Search with prefix "季"
        return proxy:word("季")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    let valid_values = ["春", "夏", "暖かい", "涼しい"];
    assert!(
        valid_values.contains(&result.as_str()),
        "Expected one of {:?}, got: {}",
        valid_values,
        result
    );
}

/// T11: L6前方一致
/// うにゅう.word("天") → グローバル辞書から前方一致
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t11_global_prefix_match() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register global words with prefix "天"
        WORD.create_global("天気"):entry("雨"):entry("雪"):entry("台風")
        WORD.create_global("天気予報"):entry("晴れのち曇り")
        
        -- Create actor without any words
        local unyuu = ACTOR.get_or_create("うにゅう")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(unyuu, act)
        
        -- Search with prefix "天"
        return proxy:word("天")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    let valid_values = ["雨", "雪", "台風", "晴れのち曇り"];
    assert!(
        valid_values.contains(&result.as_str()),
        "Expected one of {:?}, got: {}",
        valid_values,
        result
    );
}

/// T12: 関数優先テスト（オーバーライド）
/// さくら.word("天気") → アクター関数が辞書より優先
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

/// T13: 全レベル検索失敗 → nil
/// さくら.word("存在しない") → nil
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t13_not_found_returns_nil() {
    let lua = setup_runtime();

    let result: mlua::Value = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Create actor without any words
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("存在しない")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert!(result.is_nil(), "Expected nil, got: {:?}", result);
}

/// T14: 単一値配列（後方互換）
/// さくら.word("単一") → "固定値"
#[test]
#[ignore = "Requires finalize_scene() - mock act lacks word method"]
fn test_t14_single_value_array() {
    let lua = setup_runtime();

    let result: String = lua
        .load(
            r#"
        local WORD = require("pasta.word")
        local ACTOR = require("pasta.actor")
        
        -- Register single value
        WORD.create_actor("さくら", "単一"):entry("固定値")
        
        -- Create actor
        local sakura = ACTOR.get_or_create("さくら")
        local act = { current_scene = nil }
        local proxy = ACTOR.create_proxy(sakura, act)
        
        return proxy:word("単一")
    "#,
        )
        .eval()
        .expect("Lua eval failed");

    assert_eq!(result, "固定値");
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
