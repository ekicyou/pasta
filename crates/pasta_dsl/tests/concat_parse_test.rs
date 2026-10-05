//! 連結演算子 ＆／&（string-concat-operator）のパーステスト
//!
//! 受理（全角・半角・空白・位置ごと）、既存構文の不変、パースエラーの切り分けを固定する。
//! 文法仕様: `bin_op` に `concat_op = @{ amp }` を加えただけで、木は優先順位なしの左結合。

use pasta_dsl::ParseError;
use pasta_dsl::parser::{
    Action, Arg, Args, AttrValue, BinOp, CallScene, CallTarget, CueArgToken, Expr, FileItem,
    FnScope, GlobalSceneScope, LocalSceneItem, PastaFile, PastaParser2, Rule, SetValue, VarScope,
    VarSet, parse_str,
};
use pest::Parser as _;

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

fn string(s: &str) -> Expr {
    Expr::String(s.to_string())
}

fn local_var(name: &str) -> Expr {
    Expr::VarRef {
        name: name.to_string(),
        scope: VarScope::Local,
    }
}

fn binary(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn concat(lhs: Expr, rhs: Expr) -> Expr {
    binary(BinOp::Concat, lhs, rhs)
}

fn parse_ok(src: &str) -> PastaFile {
    parse_str(src, "concat_test.pasta")
        .unwrap_or_else(|e| panic!("parse should succeed for {:?}: {}", src, e))
}

fn first_global_scene(file: &PastaFile) -> &GlobalSceneScope {
    file.items
        .iter()
        .find_map(|i| match i {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .expect("global scene expected")
}

/// `＊scene\n　{line}\n` をパースし、開始ローカルシーンの最初の項目を取り出す
fn parse_scene_item(line: &str) -> LocalSceneItem {
    let file = parse_ok(&format!("＊scene\n　{}\n", line));
    first_global_scene(&file).local_scenes[0]
        .items
        .first()
        .cloned()
        .expect("local scene item expected")
}

fn parse_var_set(line: &str) -> VarSet {
    match parse_scene_item(line) {
        LocalSceneItem::VarSet(vs) => vs,
        other => panic!("Expected VarSet, got {:?}", other),
    }
}

fn parse_set_expr(line: &str) -> Expr {
    match parse_var_set(line).value {
        SetValue::Expr(e) => e,
        other => panic!("Expected SetValue::Expr, got {:?}", other),
    }
}

fn parse_call(line: &str) -> CallScene {
    match parse_scene_item(line) {
        LocalSceneItem::CallScene(c) => c,
        other => panic!("Expected CallScene, got {:?}", other),
    }
}

fn parse_actions(line: &str) -> Vec<Action> {
    match parse_scene_item(line) {
        LocalSceneItem::ActionLine(a) => a.actions,
        LocalSceneItem::ContinueAction(a) => a.actions,
        other => panic!("Expected action line, got {:?}", other),
    }
}

fn arg_items(args: &Args) -> Vec<Arg> {
    args.items.clone()
}

/// `＊scene\n　{line}\n` がパースエラーになり、その位置が 2 行目の
/// 最初の `＆`／`&` 以降を指すことを確かめる（`＆` より前で失敗していない）。
fn assert_parse_error_at_amp(line: &str) {
    let src = format!("＊scene\n　{}\n", line);
    let err = parse_str(&src, "concat_test.pasta")
        .err()
        .unwrap_or_else(|| panic!("Expected parse error for {:?}", line));
    match err {
        ParseError::SyntaxError {
            line: l, column, ..
        } => {
            assert_eq!(l, 2, "error must be on the written line: {:?}", line);
            // 列は 1 始まりの文字単位。行頭の全角空白 1 文字ぶんを足す
            let amp_col = line
                .chars()
                .position(|c| c == '＆' || c == '&')
                .expect("test line must contain ＆")
                + 2;
            assert!(
                column >= amp_col,
                "error column {} must be at or after the ＆ (col {}) for {:?}",
                column,
                amp_col,
                line
            );
        }
        other => panic!("Expected SyntaxError for {:?}, got {:?}", line, other),
    }
}

// ---------------------------------------------------------------------------
// 1.1・1.2・1.4: 全角・半角・空白の有無で同じ木
// ---------------------------------------------------------------------------

#[test]
fn test_concat_fullwidth_halfwidth_and_spaces_same_tree() {
    let expected = concat(string("a"), Expr::Integer(1));
    for rhs in [
        "「a」＆1",
        "「a」&1",
        "「a」　＆　1",
        "「a」 & 1",
        "「a」\t&\t1",
    ] {
        assert_eq!(parse_var_set_expr(rhs), expected, "rhs = {:?}", rhs);
    }
}

#[test]
fn test_concat_chain_is_left_associative() {
    // 「合計」＆＄n＆「個」 → (「合計」＆＄n)＆「個」
    assert_eq!(
        parse_var_set_expr("「合計」＆＄n＆「個」"),
        concat(concat(string("合計"), local_var("n")), string("個"))
    );
}

#[test]
fn test_concat_mixed_with_arith_keeps_left_assoc_tree() {
    // パーサは優先順位を持たない: 「a」＆1＋2 → (「a」＆1)＋2
    assert_eq!(
        parse_var_set_expr("「a」＆1＋2"),
        binary(
            BinOp::Add,
            concat(string("a"), Expr::Integer(1)),
            Expr::Integer(2)
        )
    );
    // 括弧は Paren として残る
    assert_eq!(
        parse_var_set_expr("（「1」＆「2」）＋1"),
        binary(
            BinOp::Add,
            Expr::Paren(Box::new(concat(string("1"), string("2")))),
            Expr::Integer(1)
        )
    );
}

// ---------------------------------------------------------------------------
// 1.5: 被演算子は算術と同じ（変数・関数呼び出し・数値・負数・文字列・括弧）
// ---------------------------------------------------------------------------

#[test]
fn test_concat_accepts_all_arith_operand_kinds() {
    let expr =
        parse_var_set_expr("＄＊g＆＄％p＆＠f（）＆＠＊h（）＆＠＄d（）＆－1＆2.5＆「」＆（＄a）");
    // 左結合の連鎖を平らにして被演算子の種類を確かめる
    let mut operands = Vec::new();
    let mut cur = expr;
    while let Expr::Binary { op, lhs, rhs } = cur {
        assert_eq!(op, BinOp::Concat);
        operands.push(*rhs);
        cur = *lhs;
    }
    operands.push(cur);
    operands.reverse();
    assert_eq!(operands.len(), 9, "operands = {:?}", operands);
    assert_eq!(
        operands[0],
        Expr::VarRef {
            name: "g".into(),
            scope: VarScope::Global
        }
    );
    assert_eq!(
        operands[1],
        Expr::VarRef {
            name: "p".into(),
            scope: VarScope::Property
        }
    );
    assert!(
        matches!(&operands[2], Expr::FnCall { name, scope: FnScope::Local, .. } if name == "f")
    );
    assert!(
        matches!(&operands[3], Expr::FnCall { name, scope: FnScope::Global, .. } if name == "h")
    );
    assert!(matches!(&operands[4], Expr::DynamicFnCall { var_name, .. } if var_name == "d"));
    assert_eq!(operands[5], Expr::Integer(-1));
    assert_eq!(operands[6], Expr::Float(2.5));
    assert_eq!(operands[7], Expr::BlankString);
    assert_eq!(operands[8], Expr::Paren(Box::new(local_var("a"))));
}

// ---------------------------------------------------------------------------
// 1.6・4.4・6.2: 式の位置ごとの受理
// ---------------------------------------------------------------------------

#[test]
fn test_concat_in_assignments_local_global_property() {
    let expected = SetValue::Expr(concat(string("a"), local_var("b")));
    for (line, name, scope) in [
        ("＄x＝「a」＆＄b", "x", VarScope::Local),
        ("＄＊x＝「a」＆＄b", "x", VarScope::Global),
        ("＄％foo＝「a」＆＄b", "foo", VarScope::Property),
    ] {
        let vs = parse_var_set(line);
        assert_eq!(vs.name.as_deref(), Some(name), "line = {:?}", line);
        assert_eq!(vs.scope, scope, "line = {:?}", line);
        assert_eq!(vs.value, expected, "line = {:?}", line);
    }
}

#[test]
fn test_concat_in_expression_statement() {
    let vs = parse_var_set("＄＝＠f（「a」＆＄b）");
    assert_eq!(vs.name, None);
    match vs.value {
        SetValue::Expr(Expr::FnCall { name, args, .. }) => {
            assert_eq!(name, "f");
            assert_eq!(
                arg_items(&args),
                vec![Arg::Positional(concat(string("a"), local_var("b")))]
            );
        }
        other => panic!("Expected FnCall, got {:?}", other),
    }
}

#[test]
fn test_concat_in_positional_and_keyword_args() {
    match parse_set_expr("＄x＝＠f（「a」＆1、k：＄b＆「c」）") {
        Expr::FnCall { args, .. } => assert_eq!(
            arg_items(&args),
            vec![
                Arg::Positional(concat(string("a"), Expr::Integer(1))),
                Arg::Keyword {
                    key: "k".into(),
                    value: concat(local_var("b"), string("c")),
                },
            ]
        ),
        other => panic!("Expected FnCall, got {:?}", other),
    }
}

#[test]
fn test_concat_in_call_args() {
    let call = parse_call("＞シーン（「a」＆＄b、k：1＆「c」）");
    assert!(matches!(&call.target, CallTarget::Static(n) if n == "シーン"));
    assert_eq!(
        arg_items(call.args.as_ref().expect("args expected")),
        vec![
            Arg::Positional(concat(string("a"), local_var("b"))),
            Arg::Keyword {
                key: "k".into(),
                value: concat(Expr::Integer(1), string("c")),
            },
        ]
    );
}

#[test]
fn test_concat_in_dynamic_call_target() {
    for (line, expected) in [
        (
            "＞＄種類＆「_挨拶」",
            concat(local_var("種類"), string("_挨拶")),
        ),
        (
            "＞＄種類&「_挨拶」",
            concat(local_var("種類"), string("_挨拶")),
        ),
        (
            "＞「挨拶」＆＄種類",
            concat(string("挨拶"), local_var("種類")),
        ),
    ] {
        let call = parse_call(line);
        match call.target {
            CallTarget::Dynamic(e) => assert_eq!(e, expected, "line = {:?}", line),
            other => panic!("Expected Dynamic target for {:?}, got {:?}", line, other),
        }
        assert!(call.args.is_none());
    }
}

#[test]
fn test_concat_in_talk_line_fn_call_args() {
    let expected = vec![Arg::Positional(concat(string("a"), local_var("b")))];

    for (line, scope) in [
        ("さくら：＠f（「a」＆＄b）", FnScope::Local),
        ("さくら：＠＊f（「a」＆＄b）", FnScope::Global),
    ] {
        let actions = parse_actions(line);
        assert_eq!(actions.len(), 1, "actions = {:?}", actions);
        match &actions[0] {
            Action::FnCall {
                name,
                args,
                scope: s,
                ..
            } => {
                assert_eq!((name.as_str(), *s), ("f", scope));
                assert_eq!(arg_items(args), expected);
            }
            other => panic!("Expected FnCall for {:?}, got {:?}", line, other),
        }
    }

    let actions = parse_actions("：＠＄f（「a」＆＄b）");
    assert_eq!(actions.len(), 1, "actions = {:?}", actions);
    match &actions[0] {
        Action::DynamicFnCall { var_name, args, .. } => {
            assert_eq!(var_name, "f");
            assert_eq!(arg_items(args), expected);
        }
        other => panic!("Expected DynamicFnCall, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 4.1–4.3: 式以外の位置の ＆ は不変
// ---------------------------------------------------------------------------

#[test]
fn test_concat_leaves_line_head_attributes_unchanged() {
    let file = parse_ok("＆作者：Alice\n&版:2\n＊会話\n　＆季節：春\n　さくら：やあ\n");
    let file_attrs: Vec<_> = file
        .items
        .iter()
        .filter_map(|i| match i {
            FileItem::FileAttr(a) => Some((a.key.clone(), a.value.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        file_attrs,
        vec![
            ("作者".to_string(), AttrValue::AttrString("Alice".into())),
            ("版".to_string(), AttrValue::Integer(2)),
        ]
    );
    let scene = first_global_scene(&file);
    assert_eq!(scene.attrs.len(), 1);
    assert_eq!(
        (scene.attrs[0].key.as_str(), &scene.attrs[0].value),
        ("季節", &AttrValue::AttrString("春".into()))
    );
}

/// 1 行を `rule` でパースし、`attr` ペアの原文を並べる
fn attr_texts(rule: Rule, line: &str) -> Vec<String> {
    PastaParser2::parse(rule, line)
        .unwrap_or_else(|e| panic!("{:?} should parse {:?}: {}", rule, line, e))
        .flatten()
        .filter(|p| p.as_rule() == Rule::attr)
        .map(|p| p.as_str().to_string())
        .collect()
}

#[test]
fn test_concat_leaves_scene_declaration_attributes_unchanged() {
    // シーン宣言行の ＆ は従来どおり属性マーカーとして切れる（AST は宣言行の属性を
    // 保持しないため、文法の結果で確かめる）
    assert_eq!(
        attr_texts(Rule::global_scene_line, "＊会話＆作者：Alice&版:2\n"),
        vec!["＆作者：Alice", "&版:2"]
    );
    assert_eq!(
        attr_texts(Rule::local_scene_line, "　・子＆k：v\n"),
        vec!["＆k：v"]
    );
    // 宣言行の名前は属性に食われない
    let file = parse_ok("＊会話＆作者：Alice&版:2\n　さくら：やあ\n　・子＆k：v\n　さくら：ね\n");
    let scene = first_global_scene(&file);
    assert_eq!(scene.name, "会話");
    assert_eq!(scene.local_scenes[1].name.as_deref(), Some("子"));
}

#[test]
fn test_concat_leaves_talk_text_ampersand_as_text() {
    for (line, amp) in [("さくら：＄a＆＄b", "＆"), ("：＄a&＄b", "&")] {
        let actions = parse_actions(line);
        assert_eq!(actions.len(), 3, "actions = {:?}", actions);
        assert!(matches!(&actions[0], Action::VarRef { name, .. } if name == "a"));
        assert!(
            matches!(&actions[1], Action::Talk { text, .. } if text == amp),
            "actions = {:?}",
            actions
        );
        assert!(matches!(&actions[2], Action::VarRef { name, .. } if name == "b"));
    }
}

#[test]
fn test_concat_leaves_ampersand_in_string_literal_unchanged() {
    assert_eq!(parse_var_set_expr("「a＆b」"), string("a＆b"));
    assert_eq!(parse_var_set_expr("\"a&b\""), string("a&b"));
}

#[test]
fn test_concat_leaves_word_definitions_unchanged() {
    let file = parse_ok("＠単語：A＆B、C&D\n＊scene\n　＠語：E＆F\n　さくら：やあ\n");
    let global_words: Vec<_> = file
        .items
        .iter()
        .filter_map(|i| match i {
            FileItem::GlobalWord(w) => Some(w.words.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        global_words,
        vec![vec!["A＆B".to_string(), "C&D".to_string()]]
    );
    let scene = first_global_scene(&file);
    assert_eq!(scene.words[0].words, vec!["E＆F".to_string()]);
}

#[test]
fn test_concat_leaves_choice_line_unchanged() {
    match parse_scene_item("＠？挨拶「A＆B」") {
        LocalSceneItem::Choice(c) => {
            assert_eq!(c.target, "挨拶");
            assert_eq!(c.label.as_deref(), Some("A＆B"));
        }
        other => panic!("Expected Choice, got {:?}", other),
    }
}

#[test]
fn test_concat_leaves_cue_command_args_unchanged() {
    match parse_scene_item("！cmd（a＆b、「x＆y」）") {
        LocalSceneItem::CueCommand(c) => {
            assert_eq!(c.command, "cmd");
            assert_eq!(
                c.args,
                vec![
                    CueArgToken::Ident("a＆b".into()),
                    CueArgToken::StringLiteral("x＆y".into()),
                ]
            );
        }
        other => panic!("Expected CueCommand, got {:?}", other),
    }
}

#[test]
fn test_concat_leaves_comments_unchanged() {
    let file = parse_ok("＃＆作者：x\n＊scene\n　＄x＝1　＃＆「a」\n");
    assert!(
        !file
            .items
            .iter()
            .any(|i| matches!(i, FileItem::FileAttr(_))),
        "comment must not become an attribute"
    );
    match &first_global_scene(&file).local_scenes[0].items[0] {
        LocalSceneItem::VarSet(vs) => assert_eq!(vs.value, SetValue::Expr(Expr::Integer(1))),
        other => panic!("Expected VarSet, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 4.5–4.8・5.1・5.2: パースエラーのまま残る形
// ---------------------------------------------------------------------------

#[test]
fn test_concat_call_target_followed_by_name_is_error() {
    // 4.5: ＆ の直後が名前（属性フィルター構文のために残す）
    assert_parse_error_at_amp("＞＄名前＆k＝v");
    assert_parse_error_at_amp("＞シーン名＆k＝v");
    // 4.6: 静的ターゲット（名前）の直後の ＆
    assert_parse_error_at_amp("＞挨拶＆＄種類");
    // 対照: 被演算子の開始文字が続けば受理される
    assert!(matches!(
        parse_call("＞＄名前＆「k」").target,
        CallTarget::Dynamic(_)
    ));
}

#[test]
fn test_concat_word_ref_followed_by_amp_is_error() {
    // 4.7: 単語参照は項でない
    assert_parse_error_at_amp("＄x＝＠単語名＆category＝food");
    assert_parse_error_at_amp("＄x＝＠単語名＆「a」");
    // 対照: 単語参照だけの右辺は従来どおり受理
    assert_eq!(
        parse_var_set("＄x＝＠単語名").value,
        SetValue::WordRef {
            name: "単語名".into()
        }
    );
}

#[test]
fn test_concat_rhs_must_start_with_operand_char() {
    // 4.8: ＆ の直後に名前（識別子）は置けない
    assert_parse_error_at_amp("＄x＝「a」＆name");
    assert_parse_error_at_amp("＄x＝＠f（「a」＆name）");
}

#[test]
fn test_concat_missing_operand_is_error() {
    // 5.1
    assert_parse_error_at_amp("＄x＝「a」＆");
    assert_parse_error_at_amp("＄x＝＆「a」");
    assert_parse_error_at_amp("＄x＝「a」＆＆「b」");
    assert_parse_error_at_amp("＄x＝「a」&");
}

#[test]
fn test_concat_unbalanced_paren_is_error() {
    // 5.2
    assert_parse_error_at_amp("＄x＝（「a」＆＄b");
    assert_parse_error_at_amp("＄x＝「a」＆＄b）");
    assert_parse_error_at_amp("＄x＝（「a」＆（＄b）");
    // 対照: 対応が取れていれば受理
    assert_eq!(
        parse_set_expr("＄x＝（「a」＆（＄b））"),
        Expr::Paren(Box::new(concat(
            string("a"),
            Expr::Paren(Box::new(local_var("b")))
        )))
    );
}
