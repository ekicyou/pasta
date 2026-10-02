//! 動的単語参照（＠＄変数名）・動的関数呼び出し（＠＄変数名（…））の生成コードテスト
//!
//! Requirements: 1.5, 3.2, 3.3, 3.9, 4.4, 7.2, 7.5
//!
//! 生成コードは値を文字列化せず、参照変数の値（`var.x`・`save.x`・`args[n]`）と
//! その変数パスの文字列リテラルをランタイムへ渡す。アクション行はアクター経由
//! （`act.アクター:`）、代入の右辺・式の中は act 経由（`act:`）で呼ぶ。

use crate::common;

use common::e2e_helpers::transpile;
use insta::assert_snapshot;
use pasta_dsl::parser::{Action, Args, Span, VarScope};
use pasta_lua::LineEnding;
use pasta_lua::code_gen::LuaCodeGenerator;
use pasta_lua::code_gen::source_map::SourceMapSink;

/// シーン本体の 1 行から生成された Lua コードを返す
fn transpile_body(body: &str) -> String {
    transpile(&format!("＊メイン\n　{body}\n"))
}

/// 生成コードに `line` と完全一致する行があることを確かめる
fn assert_has_line(body: &str, line: &str) {
    let lua = transpile_body(body);
    assert!(
        lua.lines().any(|l| l.trim() == line),
        "{body:?} should generate {line:?}. Code:\n{lua}"
    );
}

// ========================================================================
// アクション行: アクター経由（3.2, 7.5）
// ========================================================================

#[test]
fn test_action_dynamic_word_ref() {
    assert_has_line(
        "さくら：＠＄x　です",
        r#"act.さくら:talk(act.さくら:word(var.x, "var.x"))"#,
    );
    assert_has_line(
        "さくら：＠＄＊k",
        r#"act.さくら:talk(act.さくら:word(save.k, "save.k"))"#,
    );
    // シーン引数は既存の変数展開と同じパス
    assert_has_line(
        "さくら：＠＄０",
        r#"act.さくら:talk(act.さくら:word(args[1], "args[1]"))"#,
    );
}

#[test]
fn test_action_dynamic_fn_call() {
    assert_has_line(
        "さくら：＠＄０（１）",
        r#"act.さくら:talk((act.さくら:expr_fn_var(args[1], "args[1]", 1)))"#,
    );
    assert_has_line(
        "さくら：＠＄＊f（）",
        r#"act.さくら:talk((act.さくら:expr_fn_var(save.f, "save.f")))"#,
    );
}

// ========================================================================
// 代入の右辺: act 経由（1.5, 3.3, 4.4）
// ========================================================================

#[test]
fn test_var_set_dynamic_word_ref() {
    assert_has_line("＄y＝＠＄＊k", r#"var.y = act:word(save.k, "save.k")"#);
    assert_has_line("＄＊y＝＠＄x", r#"save.y = act:word(var.x, "var.x")"#);
}

#[test]
fn test_expression_statement_dynamic_word_ref() {
    assert_has_line("＄＝＠＄x", r#"act:word(var.x, "var.x")"#);
}

#[test]
fn test_property_set_dynamic_word_ref() {
    assert_has_line(
        "＄％prop＝＠＄x",
        r#"act:set_property("prop", act:word(var.x, "var.x"))"#,
    );
}

// ========================================================================
// 式の中の動的関数呼び出し: act 経由（7.2）
// ========================================================================

#[test]
fn test_expr_dynamic_fn_call() {
    assert_has_line(
        "＄y＝＠＄f（１）",
        r#"var.y = act:expr_fn_var(var.f, "var.f", 1)"#,
    );
    assert_has_line(
        "＄y＝＠＄f（）＋１",
        r#"var.y = act:expr_fn_var(var.f, "var.f") + 1"#,
    );
    // 静的関数呼び出しの引数の中
    assert_has_line(
        "さくら：＠g（＠＄f（））",
        r#"act.さくら:talk((act.さくら:expr_fn("g", act:expr_fn_var(var.f, "var.f"))))"#,
    );
}

// ========================================================================
// 生成コードは値を文字列化しない（3.9）
// ========================================================================

#[test]
fn test_no_tostring_on_dynamic_value() {
    let lua = transpile_body("さくら：＠＄x＠＄f（）");
    assert!(!lua.contains("tostring(var."), "Code:\n{lua}");
}

// ========================================================================
// スナップショット
// ========================================================================

#[test]
fn test_dynamic_word_ref_snapshot() {
    let source = r#"
＊メイン
　＄x＝「挨拶」
　さくら：＠＄x　です
　うにゅう：＠＄＊k＠＄０（１、＄a）
　＄y＝＠＄＊k
　＄＊y＝＠＄x
　＄＝＠＄x
　＄％prop＝＠＄x
　＄z＝＠＄f（）＋１
"#;
    assert_snapshot!("dynamic_word_ref", transpile(source));
}

// ========================================================================
// ソースマップ: 2 変種の位置が記録される
// ========================================================================

#[derive(Default)]
struct LinePairSink {
    records: Vec<(u32, u32)>,
}

impl SourceMapSink for LinePairSink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((lua_line, pasta_line));
    }
}

#[test]
fn test_dynamic_actions_record_source_map() {
    let span_at = |line: usize| Span::new(line, 1, line, 10, line * 10, line * 10 + 9);
    let actions = [
        Action::DynamicWordRef {
            var_name: "x".to_string(),
            var_scope: VarScope::Local,
            span: span_at(5),
        },
        Action::DynamicFnCall {
            var_name: "f".to_string(),
            var_scope: VarScope::Global,
            args: Args {
                items: vec![],
                span: span_at(9),
            },
            span: span_at(9),
        },
    ];

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);
        for action in &actions {
            codegen.generate_action(action, "さくら").unwrap();
        }
    }

    assert_eq!(
        String::from_utf8(output).unwrap(),
        "act.さくら:talk(act.さくら:word(var.x, \"var.x\"))\n\
         act.さくら:talk((act.さくら:expr_fn_var(save.f, \"save.f\")))\n"
    );
    assert_eq!(sink.records, vec![(1, 5), (2, 9)]);
}
