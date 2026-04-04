//! Phase A: code_generator.rs のインラインテストを外部化
//!
//! generate_action, generate_var_set を pub に昇格済み

use pasta_dsl::parser::{Action, Args, Expr, FnScope, SetValue, Span, VarScope, VarSet};
use pasta_lua::TranspileError;
use pasta_lua::code_gen::LuaCodeGenerator;

#[test]
fn test_generate_talk_action() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::Talk {
        text: "こんにちは".to_string(),
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("act.さくら:talk(\"こんにちは\")"));
}

#[test]
fn test_generate_word_ref_action() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::WordRef {
        name: "挨拶".to_string(),
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("act.さくら:word(\"挨拶\")"));
}

#[test]
fn test_generate_var_ref_local() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::VarRef {
        name: "カウンタ".to_string(),
        scope: VarScope::Local,
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("act.さくら:talk(tostring(var.カウンタ))"));
}

#[test]
fn test_generate_var_ref_global() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::VarRef {
        name: "グローバル".to_string(),
        scope: VarScope::Global,
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("act.さくら:talk(tostring(save.グローバル))"));
}

#[test]
fn test_generate_var_ref_args() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    // $0 in Pasta -> args[1] in Lua (0-indexed to 1-indexed)
    let action = Action::VarRef {
        name: "0".to_string(),
        scope: VarScope::Args(0),
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("act.さくら:talk(tostring(args[1]))"));
}

#[test]
fn test_generate_escape_action() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::Escape {
        sequence: "@@".to_string(),
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("act.さくら:talk(\"@\")"));
}

#[test]
fn test_generate_header() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    codegen.write_header().unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("local PASTA = require \"pasta\""));
    assert!(result.contains("local GLOBAL = require \"pasta.global\""));
}

// ========================================================================
// VarSet with WordRef tests (Requirement 1.1, 1.2, 1.3, 4.1)
// ========================================================================

#[test]
fn test_generate_var_set_wordref_local() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("場所".to_string()),
        scope: VarScope::Local,
        value: SetValue::WordRef {
            name: "場所".to_string(),
        },
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("var.場所 = act:word(\"場所\")"),
        "Expected 'var.場所 = act:word(\"場所\")' but got: {}",
        result
    );
}

#[test]
fn test_generate_var_set_wordref_global() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("グローバル".to_string()),
        scope: VarScope::Global,
        value: SetValue::WordRef {
            name: "単語".to_string(),
        },
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("save.グローバル = act:word(\"単語\")"),
        "Expected 'save.グローバル = act:word(\"単語\")' but got: {}",
        result
    );
}

#[test]
fn test_generate_var_set_wordref_args_error() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("0".to_string()),
        scope: VarScope::Args(0),
        value: SetValue::WordRef {
            name: "単語".to_string(),
        },
        span: Span::default(),
    };
    let result = codegen.generate_var_set(&var_set);

    assert!(
        result.is_err(),
        "Expected error for Args scope WordRef assignment"
    );
    let err = result.unwrap_err();
    match err {
        TranspileError::InvalidAst { message, .. } => {
            assert!(
                message.contains("Cannot assign to scene argument"),
                "Expected 'Cannot assign to scene argument' error but got: {}",
                message
            );
        }
        _ => panic!("Expected InvalidAst error but got: {:?}", err),
    }
}

#[test]
fn test_generate_var_set_expr_still_works() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("カウンタ".to_string()),
        scope: VarScope::Local,
        value: SetValue::Expr(Expr::Integer(10)),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("var.カウンタ = 10"),
        "Expected 'var.カウンタ = 10' but got: {}",
        result
    );
}

// ========================================================================
// Requirement 1: ＠＊XX() → GLOBAL.XX 展開テスト
// ========================================================================

#[test]
fn test_generate_global_fn_call_action() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::FnCall {
        name: "func".to_string(),
        args: Args::empty(),
        scope: FnScope::Global,
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("GLOBAL.func(act)"),
        "Expected GLOBAL.func(act) in action output, got: {}",
        result
    );
}

#[test]
fn test_generate_local_fn_call_action_still_scene() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::FnCall {
        name: "func".to_string(),
        args: Args::empty(),
        scope: FnScope::Local,
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("act.さくら:expr_fn(\"func\")"),
        "Expected act.さくら:expr_fn(\"func\") for local scope, got: {}",
        result
    );
}

#[test]
fn test_generate_global_fn_call_in_var_set_rhs() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("結果".to_string()),
        scope: VarScope::Local,
        value: SetValue::Expr(Expr::FnCall {
            name: "func".to_string(),
            args: Args::empty(),
            scope: FnScope::Global,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("var.結果 = GLOBAL.func(act)"),
        "Expected 'var.結果 = GLOBAL.func(act)' but got: {}",
        result
    );
}

// ========================================================================
// Requirement 2.4, 2.5: ＄＝expr 式文コード生成テスト
// ========================================================================

#[test]
fn test_generate_var_set_none_expr_stmt() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: None,
        scope: VarScope::Local,
        value: SetValue::Expr(Expr::FnCall {
            name: "fn".to_string(),
            args: Args::empty(),
            scope: FnScope::Local,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("act:expr_fn(\"fn\")"),
        "Expected expression statement 'act:expr_fn(\"fn\")' but got: {}",
        result
    );
    assert!(
        !result.contains("="),
        "Expression statement should not contain assignment, got: {}",
        result
    );
}

#[test]
fn test_generate_var_set_none_global_expr_stmt() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: None,
        scope: VarScope::Local,
        value: SetValue::Expr(Expr::FnCall {
            name: "gfunc".to_string(),
            args: Args::empty(),
            scope: FnScope::Global,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("GLOBAL.gfunc(act)"),
        "Expected 'GLOBAL.gfunc(act)' but got: {}",
        result
    );
    assert!(
        !result.contains("="),
        "No assignment in expression statement, got: {}",
        result
    );
}
