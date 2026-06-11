//! 式パース（二項演算・括弧・キーワード引数）の網羅テスト
//!
//! parse_action.rs の `parse_expr_from_parts` / `build_left_assoc_expr` /
//! `try_parse_expr` / `parse_key_arg` が構築する AST 形状を検証する。
//! 文法仕様: `expr = term ~ bin*` — 演算子優先順位なしの左結合。

use pasta_dsl::parser::{
    parse_str, Arg, BinOp, Expr, FileItem, LocalSceneItem, SetValue, VarScope,
};

/// `＊scene\n　＄x＝{rhs}\n` をパースし、最初の VarSet の値 Expr を取り出す
fn parse_var_set_expr(rhs: &str) -> Expr {
    let src = format!("＊scene\n　＄x＝{}\n", rhs);
    let file = parse_str(&src, "expr_test.pasta").expect("parse should succeed");
    let scene = file
        .items
        .iter()
        .find_map(|i| match i {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .expect("global scene expected");
    let item = scene.local_scenes[0]
        .items
        .first()
        .expect("local scene item expected");
    match item {
        LocalSceneItem::VarSet(vs) => match &vs.value {
            SetValue::Expr(e) => e.clone(),
            other => panic!("Expected SetValue::Expr, got {:?}", other),
        },
        other => panic!("Expected VarSet, got {:?}", other),
    }
}

// ============================================================================
// 二項演算子（半角）
// ============================================================================

#[test]
fn test_binary_add_halfwidth() {
    let expr = parse_var_set_expr("1+2");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Add);
            assert_eq!(*lhs, Expr::Integer(1));
            assert_eq!(*rhs, Expr::Integer(2));
        }
        other => panic!("Expected Binary, got {:?}", other),
    }
}

#[test]
fn test_binary_sub() {
    let expr = parse_var_set_expr("10-3");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Sub);
            assert_eq!(*lhs, Expr::Integer(10));
            assert_eq!(*rhs, Expr::Integer(3));
        }
        other => panic!("Expected Binary Sub, got {:?}", other),
    }
}

#[test]
fn test_binary_div() {
    let expr = parse_var_set_expr("10/2");
    match expr {
        Expr::Binary { op, .. } => assert_eq!(op, BinOp::Div),
        other => panic!("Expected Binary Div, got {:?}", other),
    }
}

#[test]
fn test_binary_mod() {
    let expr = parse_var_set_expr("10%3");
    match expr {
        Expr::Binary { op, .. } => assert_eq!(op, BinOp::Mod),
        other => panic!("Expected Binary Mod, got {:?}", other),
    }
}

// ============================================================================
// 二項演算子（全角）
// ============================================================================

#[test]
fn test_binary_add_fullwidth_digits_and_operator() {
    // 全角数字＋全角演算子も半角と同じ AST になる
    let expr = parse_var_set_expr("１＋２");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Add);
            assert_eq!(*lhs, Expr::Integer(1));
            assert_eq!(*rhs, Expr::Integer(2));
        }
        other => panic!("Expected Binary, got {:?}", other),
    }
}

// ============================================================================
// 結合規則: 左結合・優先順位なし
// ============================================================================

#[test]
fn test_binary_left_associative_chain() {
    // 1+2+3 → (1+2)+3
    let expr = parse_var_set_expr("1+2+3");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Add);
            assert_eq!(*rhs, Expr::Integer(3));
            match *lhs {
                Expr::Binary { op, ref lhs, ref rhs } => {
                    assert_eq!(op, BinOp::Add);
                    assert_eq!(**lhs, Expr::Integer(1));
                    assert_eq!(**rhs, Expr::Integer(2));
                }
                ref other => panic!("Expected nested Binary lhs, got {:?}", other),
            }
        }
        other => panic!("Expected Binary, got {:?}", other),
    }
}

#[test]
fn test_binary_no_operator_precedence() {
    // 文法仕様 (expr = term ~ bin*) により優先順位はなく左結合:
    // 1+2*3 → (1+2)*3 となる（数学的優先順位 1+(2*3) ではない）
    let expr = parse_var_set_expr("1+2*3");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Mul, "outermost op must be Mul (left-assoc)");
            assert_eq!(*rhs, Expr::Integer(3));
            match *lhs {
                Expr::Binary { op, .. } => assert_eq!(op, BinOp::Add),
                ref other => panic!("Expected Binary Add lhs, got {:?}", other),
            }
        }
        other => panic!("Expected Binary, got {:?}", other),
    }
}

// ============================================================================
// 括弧式
// ============================================================================

#[test]
fn test_paren_single_term_in_binary() {
    // (5)*3 → Paren(5) * 3
    let expr = parse_var_set_expr("(5)*3");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Mul);
            assert_eq!(*rhs, Expr::Integer(3));
            match *lhs {
                Expr::Paren(ref inner) => assert_eq!(**inner, Expr::Integer(5)),
                ref other => panic!("Expected Paren lhs, got {:?}", other),
            }
        }
        other => panic!("Expected Binary, got {:?}", other),
    }
}

// ============================================================================
// 変数参照を含む式
// ============================================================================

#[test]
fn test_binary_with_local_var_ref_lhs() {
    // ＄y+1 → VarRef(y, Local) + 1
    let expr = parse_var_set_expr("＄y+1");
    match expr {
        Expr::Binary { op, lhs, rhs } => {
            assert_eq!(op, BinOp::Add);
            assert_eq!(*rhs, Expr::Integer(1));
            match *lhs {
                Expr::VarRef { ref name, ref scope } => {
                    assert_eq!(name, "y");
                    assert_eq!(*scope, VarScope::Local);
                }
                ref other => panic!("Expected VarRef lhs, got {:?}", other),
            }
        }
        other => panic!("Expected Binary, got {:?}", other),
    }
}

// ============================================================================
// 数値リテラル（float）
// ============================================================================

#[test]
fn test_float_literal_halfwidth() {
    let expr = parse_var_set_expr("2.5");
    match expr {
        Expr::Float(f) => assert!((f - 2.5).abs() < f64::EPSILON),
        other => panic!("Expected Float, got {:?}", other),
    }
}

#[test]
fn test_float_literal_fullwidth() {
    // ２．５ → 2.5（全角数字・全角小数点の正規化）
    let expr = parse_var_set_expr("２．５");
    match expr {
        Expr::Float(f) => assert!((f - 2.5).abs() < f64::EPSILON),
        other => panic!("Expected Float, got {:?}", other),
    }
}

// ============================================================================
// 関数呼び出し引数: キーワード引数・位置引数の混在
// ============================================================================

#[test]
fn test_fn_call_keyword_arg() {
    // ＠fn(key：1) → FnCall with Arg::Keyword
    let expr = parse_var_set_expr("＠fn(key：1)");
    match expr {
        Expr::FnCall { name, args, .. } => {
            assert_eq!(name, "fn");
            assert_eq!(args.items.len(), 1);
            match &args.items[0] {
                Arg::Keyword { key, value } => {
                    assert_eq!(key, "key");
                    assert_eq!(*value, Expr::Integer(1));
                }
                other => panic!("Expected Keyword arg, got {:?}", other),
            }
        }
        other => panic!("Expected FnCall, got {:?}", other),
    }
}

#[test]
fn test_fn_call_mixed_positional_and_keyword_args() {
    // ＠fn(1、key：2) → [Positional(1), Keyword(key, 2)]
    let expr = parse_var_set_expr("＠fn(1、key：2)");
    match expr {
        Expr::FnCall { name, args, .. } => {
            assert_eq!(name, "fn");
            assert_eq!(args.items.len(), 2);
            match &args.items[0] {
                Arg::Positional(e) => assert_eq!(*e, Expr::Integer(1)),
                other => panic!("Expected Positional first arg, got {:?}", other),
            }
            match &args.items[1] {
                Arg::Keyword { key, value } => {
                    assert_eq!(key, "key");
                    assert_eq!(*value, Expr::Integer(2));
                }
                other => panic!("Expected Keyword second arg, got {:?}", other),
            }
        }
        other => panic!("Expected FnCall, got {:?}", other),
    }
}

#[test]
fn test_fn_call_binary_expr_in_positional_arg() {
    // ＠fn(1+2) → Positional(Binary{Add, 1, 2})
    let expr = parse_var_set_expr("＠fn(1+2)");
    match expr {
        Expr::FnCall { args, .. } => {
            assert_eq!(args.items.len(), 1);
            match &args.items[0] {
                Arg::Positional(Expr::Binary { op, lhs, rhs }) => {
                    assert_eq!(*op, BinOp::Add);
                    assert_eq!(**lhs, Expr::Integer(1));
                    assert_eq!(**rhs, Expr::Integer(2));
                }
                other => panic!("Expected Positional Binary arg, got {:?}", other),
            }
        }
        other => panic!("Expected FnCall, got {:?}", other),
    }
}
