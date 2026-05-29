//! Integration tests: `!select` cue command transpilation to `act:choice_timeout(...)`.
//!
//! Verifies that `!select(秒数)` is transpiled to `act:choice_timeout(秒数)`,
//! `!select()` / `!select` is transpiled to `act:choice_timeout(nil)`,
//! and other cue commands remain silently skipped.

use pasta_dsl::parse_str;
use pasta_lua::LuaTranspiler;

/// `!select(5)` → `act:choice_timeout(5)`
#[test]
fn test_select_with_integer_seconds() {
    let source = "＊選択シーン\n    !select(5)\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains("act:choice_timeout(5)"),
        "!select(5) should generate act:choice_timeout(5).\nGenerated Lua:\n{}",
        lua_code
    );
}

/// `!select(10.0)` → `act:choice_timeout(10)`
#[test]
fn test_select_with_float_seconds() {
    let source = "＊選択シーン\n    !select(10.0)\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains("act:choice_timeout(10)"),
        "!select(10.0) should generate act:choice_timeout(10).\nGenerated Lua:\n{}",
        lua_code
    );
}

/// `!select()` (empty args) → `act:choice_timeout(nil)`
#[test]
fn test_select_empty_args() {
    let source = "＊選択シーン\n    !select()\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains("act:choice_timeout(nil)"),
        "!select() should generate act:choice_timeout(nil).\nGenerated Lua:\n{}",
        lua_code
    );
}

/// `!select` (no parens) → `act:choice_timeout(nil)`
#[test]
fn test_select_no_parens() {
    let source = "＊選択シーン\n    !select\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        lua_code.contains("act:choice_timeout(nil)"),
        "!select (no parens) should generate act:choice_timeout(nil).\nGenerated Lua:\n{}",
        lua_code
    );
}

/// Non-select cue commands (e.g. `!clear`) are still silently skipped (regression).
#[test]
fn test_non_select_cue_commands_still_skipped() {
    let source = "＊テスト\n    !clear\n    !bgm(morning.ogg)\n";
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    assert!(
        !lua_code.contains("!clear"),
        "Non-select cue commands should not appear in output.\nGenerated Lua:\n{}",
        lua_code
    );
    assert!(
        !lua_code.contains("!bgm"),
        "Non-select cue commands should not appear in output.\nGenerated Lua:\n{}",
        lua_code
    );
    assert!(
        !lua_code.contains("choice_timeout"),
        "Non-select cue commands should not generate choice_timeout.\nGenerated Lua:\n{}",
        lua_code
    );
}
