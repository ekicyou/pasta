//! Property scope (VarScope::Property) code generation tests.
//!
//! Covers requirements:
//! - 2.1〜2.6: SET (各種値タイプ)
//! - 3.1〜3.4: GET代入
//! - 4.1, 4.5: インラインGET
//! - Expression中のProperty参照 → TranspileError::PropertyInExpression

use pasta_dsl::parser::{Action, BinOp, Expr, SetValue, Span, VarScope, VarSet};
use pasta_lua::code_gen::LuaCodeGenerator;
use pasta_lua::TranspileError;

// ============================================================================
// SET tests (generate_property_set via generate_var_set)
// ============================================================================

/// 2.1: ＄％prop＝123 → act:set_property("prop", 123)
#[test]
fn test_property_set_integer() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("prop".to_string()),
        scope: VarScope::Property,
        value: SetValue::Expr(Expr::Integer(123)),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"act:set_property("prop", 123)"#),
        "expected set_property with integer, got: {}",
        result
    );
}

/// 2.2: ＄％system.name＝「テスト」 → act:set_property("system.name", "テスト")
#[test]
fn test_property_set_string() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("system.name".to_string()),
        scope: VarScope::Property,
        value: SetValue::Expr(Expr::String("テスト".to_string())),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains("act:set_property(\"system.name\""),
        "expected set_property with dotted name, got: {}",
        result
    );
    assert!(
        result.contains("テスト"),
        "expected string value, got: {}",
        result
    );
}

/// 2.3: ＄％prop＝＄var → act:set_property("prop", var.var)
#[test]
fn test_property_set_local_var_ref() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("prop".to_string()),
        scope: VarScope::Property,
        value: SetValue::Expr(Expr::VarRef {
            name: "var".to_string(),
            scope: VarScope::Local,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"act:set_property("prop", var.var)"#),
        "expected set_property with local var ref, got: {}",
        result
    );
}

/// 2.4: ＄％prop＝＄＊gvar → act:set_property("prop", save.gvar)
#[test]
fn test_property_set_global_var_ref() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("prop".to_string()),
        scope: VarScope::Property,
        value: SetValue::Expr(Expr::VarRef {
            name: "gvar".to_string(),
            scope: VarScope::Global,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"act:set_property("prop", save.gvar)"#),
        "expected set_property with global var ref, got: {}",
        result
    );
}

/// 2.5: ＄％prop＝＠word → act:set_property("prop", act:word("word"))
#[test]
fn test_property_set_word_ref() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("prop".to_string()),
        scope: VarScope::Property,
        value: SetValue::WordRef {
            name: "word".to_string(),
        },
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"act:set_property("prop", act:word("word"))"#),
        "expected set_property with word ref, got: {}",
        result
    );
}

/// 2.6: ＄％prop＝1＋2 → act:set_property("prop", 1 + 2)
#[test]
fn test_property_set_binary_expr() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("prop".to_string()),
        scope: VarScope::Property,
        value: SetValue::Expr(Expr::Binary {
            op: BinOp::Add,
            lhs: Box::new(Expr::Integer(1)),
            rhs: Box::new(Expr::Integer(2)),
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"act:set_property("prop", 1 + 2)"#),
        "expected set_property with binary expr, got: {}",
        result
    );
}

// ============================================================================
// GET assignment tests (direct assignment in generate_var_set)
// ============================================================================

/// 3.1: ＄var＝＄％p → var.var = act:get_property("p")
#[test]
fn test_property_get_assign_local() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("var".to_string()),
        scope: VarScope::Local,
        value: SetValue::Expr(Expr::VarRef {
            name: "p".to_string(),
            scope: VarScope::Property,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"var.var = act:get_property("p")"#),
        "expected local get_property assignment, got: {}",
        result
    );
}

/// 3.2: ＄＊var＝＄％p → save.var = act:get_property("p")
#[test]
fn test_property_get_assign_global() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("var".to_string()),
        scope: VarScope::Global,
        value: SetValue::Expr(Expr::VarRef {
            name: "p".to_string(),
            scope: VarScope::Property,
        }),
        span: Span::default(),
    };
    codegen.generate_var_set(&var_set).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"save.var = act:get_property("p")"#),
        "expected global get_property assignment, got: {}",
        result
    );
}

// ============================================================================
// Inline GET tests (generate_action with VarRef + Property)
// ============================================================================

/// 4.1: さくら：＄％p → act.さくら:talk(tostring(act:get_property("p")))
#[test]
fn test_property_inline_get() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let action = Action::VarRef {
        name: "p".to_string(),
        scope: VarScope::Property,
        span: Span::default(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(
        result.contains(r#"act.さくら:talk(tostring(act:get_property("p")))"#),
        "expected inline get_property via talk, got: {}",
        result
    );
}

// ============================================================================
// Property in expression → error
// ============================================================================

/// Expression中のProperty参照はTranspileError::PropertyInExpressionを返す
#[test]
fn test_property_in_expression_returns_error() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    let var_set = VarSet {
        name: Some("x".to_string()),
        scope: VarScope::Local,
        value: SetValue::Expr(Expr::Binary {
            op: BinOp::Add,
            lhs: Box::new(Expr::VarRef {
                name: "p".to_string(),
                scope: VarScope::Property,
            }),
            rhs: Box::new(Expr::Integer(1)),
        }),
        span: Span::default(),
    };

    let err = codegen.generate_var_set(&var_set).unwrap_err();
    assert!(
        matches!(err, TranspileError::PropertyInExpression),
        "expected PropertyInExpression error, got: {:?}",
        err
    );
}
