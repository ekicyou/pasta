//! Integration tests: Choice line transpilation to Lua.
//!
//! Verifies that `＠？target` and `＠？target「label」` choice lines
//! are correctly transpiled to `act:choice("target", "display")` Lua calls.

use pasta_dsl::parse_str;
use pasta_lua::LuaTranspiler;

/// Shorthand form: `＠？target` → `act:choice("target", "target")`
#[test]
fn test_choice_shorthand_generates_lua_call() {
    let source = "＊選択シーン\n    ＠？挨拶\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains(r#"act:choice("挨拶", "挨拶")"#),
        "Shorthand choice should use target as display.\nGenerated Lua:\n{}",
        lua_code
    );
}

/// Label form: `＠？target「表示テキスト」` → `act:choice("target", "表示テキスト")`
#[test]
fn test_choice_with_label_generates_lua_call() {
    let source = "＊選択シーン\n    ＠？挨拶「こんにちはを選ぶ」\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains(r#"act:choice("挨拶", "こんにちはを選ぶ")"#),
        "Label form should use label as display.\nGenerated Lua:\n{}",
        lua_code
    );
}

/// Multiple choice lines each generate individual `act:choice(...)` calls.
#[test]
fn test_multiple_choices_generate_individual_calls() {
    let source = "＊選択シーン\n    ＠？挨拶\n    ＠？別れ「さようなら」\n    ＠？雑談\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains(r#"act:choice("挨拶", "挨拶")"#),
        "First choice should be present.\nGenerated Lua:\n{}",
        lua_code
    );
    assert!(
        lua_code.contains(r#"act:choice("別れ", "さようなら")"#),
        "Second choice (with label) should be present.\nGenerated Lua:\n{}",
        lua_code
    );
    assert!(
        lua_code.contains(r#"act:choice("雑談", "雑談")"#),
        "Third choice should be present.\nGenerated Lua:\n{}",
        lua_code
    );
}

/// String escaping: display text containing `\` or `"` is safely escaped.
#[test]
fn test_choice_label_string_escaping() {
    // Label contains a backslash which requires long string format
    let source = "＊選択シーン\n    ＠？挨拶「パス\\付き」\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // The target "挨拶" has no special chars → double-quoted
    // The label "パス\付き" contains backslash → long string format
    assert!(
        lua_code.contains("act:choice("),
        "Choice call should be present.\nGenerated Lua:\n{}",
        lua_code
    );
    // Should use long string for the label (contains backslash)
    assert!(
        lua_code.contains(r"[[パス\付き]]"),
        "Label with backslash should use long string format.\nGenerated Lua:\n{}",
        lua_code
    );
}
