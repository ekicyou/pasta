//! Integration tests: CueCommand passthrough in Lua transpilation.
//!
//! Verifies that scenes containing CueCommand items are transpiled
//! to Lua without errors, and that CueCommand lines are silently
//! skipped (not emitted as Lua code).

use pasta_dsl::parse_str;
use pasta_lua::LuaTranspiler;

/// Pasta source containing cue commands mixed with regular items.
const PASTA_WITH_CUE: &str = r#"
＊起動挨拶
    さくら：こんにちは！
    !wait@さくら(500)
    うにゅう：やあ！
    !bgm(morning.ogg)
    ＄score＝100
"#;

/// Verify that transpilation succeeds for scenes with CueCommand items.
#[test]
fn test_transpile_scene_with_cue_commands_succeeds() {
    let file = parse_str(PASTA_WITH_CUE, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let result = transpiler.transpile(&file, &mut output);
    assert!(
        result.is_ok(),
        "Transpilation should succeed for scenes with cue commands: {:?}",
        result.err()
    );
}

/// Verify that CueCommand lines are NOT emitted as Lua code.
#[test]
fn test_cue_commands_not_emitted_in_lua_output() {
    let file = parse_str(PASTA_WITH_CUE, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // CueCommand content should NOT appear in Lua output
    assert!(
        !lua_code.contains("!wait"),
        "CueCommand '!wait' should not appear in Lua output"
    );
    assert!(
        !lua_code.contains("!bgm"),
        "CueCommand '!bgm' should not appear in Lua output"
    );
}

/// Verify that non-cue items in the same scene are still emitted correctly.
#[test]
fn test_non_cue_items_emitted_correctly_alongside_cue() {
    let file = parse_str(PASTA_WITH_CUE, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Action lines should still be emitted
    assert!(
        lua_code.contains("act.さくら:talk"),
        "Action line for さくら should be emitted"
    );
    assert!(
        lua_code.contains("act.うにゅう:talk"),
        "Action line for うにゅう should be emitted"
    );
    // Variable set should still be emitted
    assert!(
        lua_code.contains("var.score"),
        "Variable set should be emitted"
    );
}

/// Verify transpilation with fullwidth cue marker.
#[test]
fn test_transpile_fullwidth_cue_marker_passthrough() {
    let pasta = "＊テスト\n    さくら：はい\n    ！effect(flash)\n    さくら：終了\n";
    let file = parse_str(pasta, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let result = transpiler.transpile(&file, &mut output);
    assert!(
        result.is_ok(),
        "Transpilation should succeed with fullwidth cue marker"
    );

    let lua_code = String::from_utf8(output).unwrap();
    assert!(
        !lua_code.contains("!effect"),
        "Fullwidth cue command should not appear in Lua output"
    );
    assert!(
        lua_code.contains("act.さくら:talk"),
        "Action lines should still be emitted"
    );
}

/// Requirements.md sample ("起動挨拶") should transpile without error.
#[test]
fn test_transpile_requirements_sample_scene() {
    let pasta = r#"
＊起動挨拶
    さくら：こんにちは！
    !wait@さくら(500)
    ！bgm(morning.ogg)
    うにゅう：やあ！
"#;

    let file = parse_str(pasta, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let result = transpiler.transpile(&file, &mut output);
    assert!(
        result.is_ok(),
        "Requirements sample should transpile: {:?}",
        result.err()
    );

    let lua_code = String::from_utf8(output).unwrap();
    assert!(
        lua_code.contains("PASTA.create_scene"),
        "Scene should be created"
    );
    assert!(
        lua_code.contains("act.さくら:talk"),
        "Action lines should be present"
    );
    assert!(
        lua_code.contains("act.うにゅう:talk"),
        "All action lines should be present"
    );
}
