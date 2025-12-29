//! Integration tests for pasta_lua transpiler.
//!
//! Tests that verify the transpiler generates Lua code matching the reference implementation.

use pasta_core::parse_str;
use pasta_lua::{LuaTranspiler, TranspilerConfig};

/// Sample Pasta source for testing
/// アクター定義では ＠ (word_marker) を使用して表情を定義
const SAMPLE_PASTA: &str = r#"
％さくら
　＠通常：\s[0]
　＠照れ：\s[1]

％うにゅう
　＠通常：\s[10]

＊メイン
　　さくら：こんにちは。
　　うにゅう：やふぅ。

　・自己紹介
　　　さくら：私はさくらです。
　　　うにゅう：ワイはうにゅうや。
"#;

#[test]
fn test_transpile_sample_pasta_header() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    // Parse the sample
    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors = file.actor_scopes();
    let scenes = file.global_scene_scopes();

    // Convert references to owned values for transpile
    let actors: Vec<_> = actors.into_iter().cloned().collect();
    let scenes: Vec<_> = scenes.into_iter().cloned().collect();

    let result = transpiler.transpile(&actors, &scenes, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();

    // Verify header
    assert!(
        lua_code.contains("local PASTA = require \"pasta.runtime\""),
        "Missing PASTA require statement"
    );
}

#[test]
fn test_transpile_sample_pasta_actors() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify actor definitions (Requirement 3a)
    assert!(
        lua_code.contains("PASTA:create_actor(\"さくら\")"),
        "Missing actor さくら"
    );
    assert!(
        lua_code.contains("PASTA:create_actor(\"うにゅう\")"),
        "Missing actor うにゅう"
    );

    // Verify actor attributes with StringLiteralizer (Requirement 2)
    assert!(
        lua_code.contains("ACTOR.通常 = [=[\\s[0]]=]"),
        "Missing さくら.通常 attribute with long string format"
    );
    assert!(
        lua_code.contains("ACTOR.照れ = [=[\\s[1]]=]"),
        "Missing さくら.照れ attribute"
    );
    assert!(
        lua_code.contains("ACTOR.通常 = [=[\\s[10]]=]"),
        "Missing うにゅう.通常 attribute"
    );
}

#[test]
fn test_transpile_sample_pasta_scenes() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify scene definition (Requirement 3b)
    assert!(
        lua_code.contains("PASTA:create_scene(\"メイン1\")"),
        "Missing scene メイン1"
    );

    // Verify entry point function (Requirement 3c)
    assert!(
        lua_code.contains("function SCENE.__start__(ctx, ...)"),
        "Missing __start__ function"
    );

    // Verify session initialization
    assert!(
        lua_code.contains("local args = { ... }"),
        "Missing args initialization"
    );
    assert!(
        lua_code.contains("local act, save, var = PASTA:create_session(SCENE, ctx)"),
        "Missing session initialization"
    );

    // Verify local scene function (Requirement 3c)
    assert!(
        lua_code.contains("function SCENE.__自己紹介_"),
        "Missing 自己紹介 local scene function"
    );
}

#[test]
fn test_transpile_sample_pasta_actions() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify talk actions (Requirement 3d)
    assert!(
        lua_code.contains("act.さくら:talk(\"こんにちは。\")"),
        "Missing さくら talk action"
    );
    assert!(
        lua_code.contains("act.うにゅう:talk(\"やふぅ。\")"),
        "Missing うにゅう talk action"
    );
}

#[test]
fn test_transpile_do_end_scope_separation() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify do...end scope separation (Requirement 1)
    // Count do/end pairs
    let do_count = lua_code.matches("\ndo\n").count() + lua_code.matches("do\n").count();
    let end_count = lua_code.matches("\nend\n").count() + lua_code.matches("end\n").count();

    // Should have at least 3 do...end blocks (2 actors + 1 scene)
    assert!(
        do_count >= 3,
        "Expected at least 3 do blocks, found {}",
        do_count
    );
    assert!(
        end_count >= do_count,
        "Unbalanced do/end blocks: {} do, {} end",
        do_count,
        end_count
    );
}

#[test]
fn test_string_literalizer_in_transpile() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    // Test with SakuraScript that needs long string format
    let pasta = r#"
％さくら
　＠通常：\s[0]
　＠照れ：\s[1]

＊メイン
　　さくら：テスト
"#;

    let file = parse_str(pasta, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify long string format for SakuraScript with brackets
    assert!(
        lua_code.contains("[=[\\s[0]]=]"),
        "SakuraScript should use [=[...]=] format due to brackets"
    );
}

#[test]
fn test_transpile_config_no_comments() {
    let config = TranspilerConfig::without_comments();
    let transpiler = LuaTranspiler::new(config);
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Config without comments should still produce valid code
    assert!(lua_code.contains("PASTA:create_actor"));
    assert!(lua_code.contains("PASTA:create_scene"));
}

/// Test case for the reference implementation sample.pasta
#[test]
fn test_transpile_reference_sample_structure() {
    let sample_pasta = include_str!("../../../.kiro/specs/pasta-lua-specification/sample.pasta");

    let file = parse_str(sample_pasta, "sample.pasta").unwrap();
    let actors = file.actor_scopes();
    let scenes = file.global_scene_scopes();

    // Verify the sample parses correctly
    assert_eq!(actors.len(), 2, "Expected 2 actors");
    assert_eq!(scenes.len(), 2, "Expected 2 global scenes");

    // Transpile
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let actors: Vec<_> = actors.into_iter().cloned().collect();
    let scenes: Vec<_> = scenes.into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify key structural elements
    assert!(
        lua_code.contains("PASTA:create_actor(\"さくら\")"),
        "Missing actor さくら"
    );
    assert!(
        lua_code.contains("PASTA:create_actor(\"うにゅう\")"),
        "Missing actor うにゅう"
    );
    assert!(
        lua_code.contains("PASTA:create_scene(\"メイン1\")"),
        "Missing scene メイン1"
    );
    assert!(
        lua_code.contains("PASTA:create_scene(\"会話分岐2\")"),
        "Missing scene 会話分岐2"
    );
}
