//! 数の字句の値の規則（ponytail-dsl-parser 要件 2.1〜2.6・4.3）
//!
//! 小数点が無く i64 に収まれば整数、収まらなければいちばん近い小数、
//! 小数点があれば小数、有限の小数に収まらなければ字句の位置の構文の誤り。
//! 式・位置の引数・名前つきの引数・Call の動的ターゲット・括弧の中・属性値・
//! キューの引数のどこでも同じ規則であることを確かめる。

use pasta_dsl::ParseError;
use pasta_dsl::parser::{
    Arg, AttrValue, BinOp, CallScene, CallTarget, CueArgToken, Expr, FileItem, LocalSceneItem,
    PastaFile, SetValue, VarScope, parse_str,
};

const BIG: &str = "10000000000000000000"; // i64 の上限を超える 20 桁
const TOO_BIG_MSG: &str = "数が大きすぎます（小数で表せるのは、およそ 1.8×10^308 まで）";

fn parse(src: &str) -> PastaFile {
    parse_str(src, "num.pasta").expect("parse should succeed")
}

fn local_items(file: &PastaFile) -> Vec<LocalSceneItem> {
    file.items
        .iter()
        .filter_map(|i| match i {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .flat_map(|s| s.local_scenes.iter().flat_map(|l| l.items.clone()))
        .collect()
}

/// `＊s\n　＄x＝{rhs}\n` の右辺の式
fn var_expr(rhs: &str) -> Expr {
    let file = parse(&format!("＊s\n　＄x＝{rhs}\n"));
    match &local_items(&file)[0] {
        LocalSceneItem::VarSet(vs) => match &vs.value {
            SetValue::Expr(e) => e.clone(),
            other => panic!("expected Expr, got {other:?}"),
        },
        other => panic!("expected VarSet, got {other:?}"),
    }
}

fn call_scene(line: &str) -> CallScene {
    let file = parse(&format!("＊s\n　{line}\n"));
    match &local_items(&file)[0] {
        LocalSceneItem::CallScene(c) => c.clone(),
        other => panic!("expected CallScene, got {other:?}"),
    }
}

fn cue_args(line: &str) -> Vec<CueArgToken> {
    let file = parse(&format!("＊s\n　{line}\n"));
    match &local_items(&file)[0] {
        LocalSceneItem::CueCommand(c) => c.args.clone(),
        other => panic!("expected CueCommand, got {other:?}"),
    }
}

fn attr_value(lit: &str) -> AttrValue {
    let file = parse(&format!("＆n：{lit}\n"));
    match &file.items[0] {
        FileItem::FileAttr(a) => a.value().unwrap().clone(),
        other => panic!("expected FileAttr, got {other:?}"),
    }
}

/// 誤りの (line, column, message)
fn syntax_err(src: &str) -> (usize, usize, String) {
    match parse_str(src, "num.pasta") {
        Err(ParseError::SyntaxError {
            file,
            line,
            column,
            message,
        }) => {
            assert_eq!(file, "num.pasta");
            (line, column, message)
        }
        other => panic!("expected SyntaxError, got {other:?}"),
    }
}

fn too_big_at(line: usize, column: usize) -> (usize, usize, String) {
    (
        line,
        column,
        format!("Parse error in num.pasta at {line}:{column}: {TOO_BIG_MSG}"),
    )
}

// ============================================================================
// 値の規則（2.1・2.2・2.5・2.6）
// ============================================================================

#[test]
fn i64_bounds_stay_integer() {
    assert_eq!(var_expr("9223372036854775807"), Expr::Integer(i64::MAX));
    assert_eq!(var_expr("-9223372036854775808"), Expr::Integer(i64::MIN));
    assert_eq!(var_expr("０"), Expr::Integer(0));
}

#[test]
fn integer_beyond_i64_becomes_nearest_float() {
    assert_eq!(
        var_expr("9223372036854775808"),
        Expr::Float(9223372036854775808.0)
    );
    assert_eq!(
        var_expr("-9223372036854775809"),
        Expr::Float(-9223372036854775809.0)
    );
    assert_eq!(var_expr(BIG), Expr::Float(1e19));
    assert_eq!(var_expr("99999999999999999999"), Expr::Float(1e20));
    // 全角でも同じ
    assert_eq!(
        var_expr(&format!("１{}", "０".repeat(19))),
        Expr::Float(1e19)
    );
    // 300 桁は有限の小数に収まる
    assert_eq!(
        var_expr(&format!("1{}", "0".repeat(299))),
        Expr::Float(1e299)
    );
}

#[test]
fn decimal_point_stays_float() {
    assert_eq!(var_expr("3.5"), Expr::Float(3.5));
    assert_eq!(var_expr("３．５"), Expr::Float(3.5));
    assert_eq!(var_expr("1.0"), Expr::Float(1.0));
}

#[test]
fn integer_of_400_digits_is_syntax_error() {
    let src = format!("＊s\n　＄x＝1{}\n", "0".repeat(399));
    assert_eq!(syntax_err(&src), too_big_at(2, 5));
}

#[test]
fn decimal_of_400_digits_is_syntax_error() {
    let src = format!("＊s\n　＄x＝1{}.5\n", "0".repeat(399));
    assert_eq!(syntax_err(&src), too_big_at(2, 5));
}

#[test]
fn error_column_counts_fullwidth_chars_and_points_at_sign() {
    // 「　＄ｘ＝」の 4 文字の後、符号の位置が桁になる（バイトの数ではない）
    let src = format!("＊s\n　＄ｘ＝－１{}\n", "０".repeat(399));
    assert_eq!(syntax_err(&src), too_big_at(2, 5));
}

// ============================================================================
// 要件 2.4 の 4 つの例
// ============================================================================

#[test]
fn big_number_keeps_operator_and_term() {
    assert_eq!(
        var_expr(&format!("1＋{BIG}")),
        Expr::Binary {
            op: BinOp::Add,
            lhs: Box::new(Expr::Integer(1)),
            rhs: Box::new(Expr::Float(1e19)),
        }
    );
}

#[test]
fn big_number_keeps_call_args_and_positions() {
    let call = call_scene(&format!("＞t（{BIG}、2）"));
    let items = call.args.expect("args").items;
    assert_eq!(
        items,
        vec![
            Arg::Positional(Expr::Float(1e19)),
            Arg::Positional(Expr::Integer(2)),
        ]
    );
}

#[test]
fn big_number_in_cue_arg_is_float() {
    assert_eq!(
        cue_args("!wait(99999999999999999999)"),
        vec![CueArgToken::Float(1e20)]
    );
}

#[test]
fn big_number_in_attr_is_float() {
    assert_eq!(attr_value("99999999999999999999"), AttrValue::Float(1e20));
}

// ============================================================================
// どこでも同じ値（2.3）
// ============================================================================

#[test]
fn same_value_everywhere() {
    let v = Expr::Float(1e19);

    // 式（代入の右辺）・括弧の中
    assert_eq!(var_expr(BIG), v);
    assert_eq!(
        var_expr(&format!("（{BIG}）")),
        Expr::Paren(Box::new(v.clone()))
    );

    // 位置の引数・名前つきの引数
    let call = call_scene(&format!("＞t（{BIG}、k：{BIG}）"));
    assert_eq!(
        call.args.expect("args").items,
        vec![
            Arg::Positional(v.clone()),
            Arg::Keyword {
                key: "k".to_string(),
                value: v.clone(),
            },
        ]
    );

    // Call の動的ターゲット
    let call = call_scene(&format!("＞（{BIG}）"));
    assert_eq!(
        call.target,
        CallTarget::Dynamic(Expr::Paren(Box::new(v.clone())))
    );

    // 属性値・キューの引数
    assert_eq!(attr_value(BIG), AttrValue::Float(1e19));
    assert_eq!(
        cue_args(&format!("!wait({BIG})")),
        vec![CueArgToken::Float(1e19)]
    );

    // i64 に収まる数も、どこでも整数
    assert_eq!(
        attr_value("9223372036854775807"),
        AttrValue::Integer(i64::MAX)
    );
    assert_eq!(
        cue_args("!wait(9223372036854775807)"),
        vec![CueArgToken::Integer(i64::MAX)]
    );
    assert_eq!(attr_value("1.5"), AttrValue::Float(1.5));
    assert_eq!(cue_args("!wait(1.5)"), vec![CueArgToken::Float(1.5)]);
}

#[test]
fn too_big_is_error_everywhere() {
    let huge = format!("1{}", "0".repeat(399));
    // 位置の引数（＞t（ の後、5 桁目）
    assert_eq!(
        syntax_err(&format!("＊s\n　＞t（{huge}）\n")),
        too_big_at(2, 5)
    );
    // 名前つきの引数
    assert_eq!(
        syntax_err(&format!("＊s\n　＞t（k：{huge}）\n")),
        too_big_at(2, 7)
    );
    // アクション行の関数呼び出しの引数
    assert_eq!(
        syntax_err(&format!("＊s\n　さくら：＠f（{huge}）\n")),
        too_big_at(2, 9)
    );
    // 属性値
    assert_eq!(syntax_err(&format!("＆n：{huge}\n")), too_big_at(1, 4));
    // キューの引数
    assert_eq!(
        syntax_err(&format!("＊s\n　!wait({huge})\n")),
        too_big_at(2, 8)
    );
}

// ============================================================================
// アクターの番号の範囲（2.7）
// ============================================================================

fn actors(src: &str) -> Vec<(String, u32)> {
    match &parse(src).items[0] {
        FileItem::GlobalSceneScope(s) => s
            .actors
            .iter()
            .map(|a| (a.name.clone(), a.number))
            .collect(),
        other => panic!("expected GlobalSceneScope, got {other:?}"),
    }
}

#[test]
fn actor_number_beyond_u32_is_syntax_error() {
    // 2 行目の `4294967296` の始まり（全角の字も 1 桁と数える）
    assert_eq!(
        syntax_err("＊s\n　％a、b＝4294967296\n"),
        (
            2,
            7,
            "Parse error in num.pasta at 2:7: アクターの番号 4294967296 は 0〜4294967295 の範囲を超えています"
                .to_string()
        )
    );
    // 書いたとおりの数字（全角）を文に出す
    assert_eq!(
        syntax_err("＊s\n　％a＝４２９４９６７２９６\n").2,
        "Parse error in num.pasta at 2:5: アクターの番号 ４２９４９６７２９６ は 0〜4294967295 の範囲を超えています"
    );
}

#[test]
fn actor_number_at_u32_max_and_next_omitted_stays() {
    assert_eq!(
        actors("＊s\n　％a＝4294967295、b\n"),
        vec![("a".to_string(), u32::MAX), ("b".to_string(), u32::MAX)]
    );
    assert_eq!(
        actors("＊s\n　％a、b＝3、c\n"),
        vec![
            ("a".to_string(), 0),
            ("b".to_string(), 3),
            ("c".to_string(), 4)
        ]
    );
}

// ============================================================================
// シーン引数の番号の範囲（2.8）
// ============================================================================

fn arg_err(line: usize, col: usize, digits: &str) -> (usize, usize, String) {
    (
        line,
        col,
        format!(
            "Parse error in num.pasta at {line}:{col}: シーン引数の番号 {digits} は 0〜255 の範囲を超えています"
        ),
    )
}

#[test]
fn scene_arg_255_is_args() {
    assert_eq!(
        var_expr("＄２５５"),
        Expr::VarRef {
            name: "２５５".to_string(),
            scope: VarScope::Args(255)
        }
    );
    assert!(parse_str("＊s\n　さくら：＄２５５＠＄２５５\n", "num.pasta").is_ok());
}

#[test]
fn scene_arg_beyond_255_is_syntax_error() {
    // アクション行：`＄` の次の `２５６` の位置
    assert_eq!(
        syntax_err("＊s\n　さくら：＄２５６\n"),
        arg_err(2, 7, "２５６")
    );
    // 式の項
    assert_eq!(syntax_err("＊s\n　＄x＝＄256\n"), arg_err(2, 6, "256"));
    // 動的な単語参照
    assert_eq!(
        syntax_err("＊s\n　さくら：＠＄２５６\n"),
        arg_err(2, 8, "２５６")
    );
}
