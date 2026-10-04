//! dsl-literal-fixes: 引用文字列・空の引用・単語値の `#` のパース結果を固定する
//!
//! 「エラー行の契約」（design.md）の行番号をそのまま固定する。

use pasta_dsl::ParseError;
use pasta_dsl::parser::{
    Expr, FileItem, GlobalSceneScope, LocalSceneItem, PastaFile, SetValue, parse_str,
};

const NEWLINES: [&str; 2] = ["\n", "\r\n"];

fn parse_ok(src: &str) -> PastaFile {
    parse_str(src, "literal_fixes_test.pasta")
        .unwrap_or_else(|e| panic!("parse should succeed: {e}\nsource: {src:?}"))
}

/// パースエラーになることを確かめ、報告された行番号を返す
fn error_line(src: &str) -> usize {
    match parse_str(src, "literal_fixes_test.pasta") {
        Err(ParseError::SyntaxError { line, .. }) => line,
        Err(other) => panic!("expected SyntaxError, got {other:?}\nsource: {src:?}"),
        Ok(file) => panic!(
            "expected parse error, got {:?}\nsource: {src:?}",
            file.items
        ),
    }
}

fn first_scene(file: &PastaFile) -> &GlobalSceneScope {
    file.items
        .iter()
        .find_map(|i| match i {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .expect("global scene expected")
}

// ============================================================================
// 1.1〜1.4: 改行をまたぐ引用はパースエラー（エラー行の契約）
// ============================================================================

const ALL_FENCES: [(&str, &str); 4] = [
    ("「", "」"),
    ("\"", "\""),
    ("「「", "」」"),
    ("\"\"", "\"\""),
];

/// 2 行目に `line` を置いたシーンを、`{q}` を改行（LF・CRLF）をまたぐ引用に置き換えて
/// パースし、すべて `expected` 行のパースエラーになることを確かめる
fn assert_cross_line_error(line: &str, fences: &[(&str, &str)], expected: usize) {
    for nl in NEWLINES {
        for (open, close) in fences {
            let body = line.replace("{q}", &format!("{open}a{nl}b{close}"));
            let src = format!("＊s{nl}{body}{nl}");
            assert_eq!(error_line(&src), expected, "source: {src:?}");
        }
    }
}

#[test]
fn cross_line_quote_in_expression_errors_at_start_line() {
    assert_cross_line_error("　＄x＝{q}", &ALL_FENCES, 2);
}

#[test]
fn cross_line_quote_in_call_arg_errors_at_start_line() {
    assert_cross_line_error("　＞t（{q}）", &ALL_FENCES, 2);
}

#[test]
fn cross_line_quote_in_call_dynamic_target_errors_at_start_line() {
    assert_cross_line_error("　＞{q}", &ALL_FENCES, 2);
}

#[test]
fn cross_line_quote_in_cue_command_arg_errors_at_start_line() {
    assert_cross_line_error("　！cmd（{q}）", &ALL_FENCES, 2);
}

#[test]
fn cross_line_quote_in_choice_label_errors_at_start_line() {
    // 選択肢の表示テキストは「」1 重だけを取る
    assert_cross_line_error("　＠？t{q}", &[("「", "」")], 2);
}

#[test]
fn cross_line_quote_in_action_line_fn_arg_errors_at_next_line() {
    // 開始した行は台詞として文法に合うため、エラーは次の行で出る
    assert_cross_line_error("　さくら：＠f（{q}）", &ALL_FENCES, 3);
}

// ============================================================================
// 1.5: 行内で閉じた引用は中身をそのまま値にする
// ============================================================================

const VERBATIM: &str = "a b#c＃d、e\\f";

#[test]
fn closed_quote_keeps_contents_verbatim_in_expression() {
    for (open, close) in ALL_FENCES {
        let src = format!("＊s\n　＄x＝{open}{VERBATIM}{close}\n");
        let file = parse_ok(&src);
        match &first_scene(&file).local_scenes[0].items[0] {
            LocalSceneItem::VarSet(vs) => {
                assert_eq!(vs.value, SetValue::Expr(Expr::String(VERBATIM.to_string())))
            }
            other => panic!("expected VarSet, got {other:?}"),
        }
    }
}

#[test]
fn closed_quote_keeps_contents_verbatim_in_word_value() {
    for (open, close) in ALL_FENCES {
        let src = format!("＠w：{open}{VERBATIM}{close}\n");
        let file = parse_ok(&src);
        match &file.items[..] {
            [FileItem::GlobalWord(kw)] => assert_eq!(kw.words, vec![VERBATIM.to_string()]),
            other => panic!("expected one GlobalWord, got {other:?}"),
        }
    }
}

// ============================================================================
// 2.2・2.3: 別々の行の `""` は行の構造を壊さない／同じ行の `""…""` は 2 重の囲み
// ============================================================================

#[test]
fn blank_quotes_on_separate_lines_make_two_words() {
    for nl in NEWLINES {
        let src = format!("＠a：\"\"{nl}＠b：\"\"{nl}");
        let file = parse_ok(&src);
        let names: Vec<&str> = file
            .items
            .iter()
            .map(|i| match i {
                FileItem::GlobalWord(kw) => kw.name(),
                other => panic!("expected GlobalWord, got {other:?}"),
            })
            .collect();
        assert_eq!(names, ["a", "b"], "source: {src:?}");
    }
}

#[test]
fn blank_quotes_on_separate_lines_make_two_assignments() {
    for nl in NEWLINES {
        let src = format!("＊s{nl}　＄x＝\"\"{nl}　＄y＝\"\"{nl}");
        let file = parse_ok(&src);
        let names: Vec<&str> = first_scene(&file).local_scenes[0]
            .items
            .iter()
            .map(|i| match i {
                LocalSceneItem::VarSet(vs) => vs.name.as_deref().expect("var name"),
                other => panic!("expected VarSet, got {other:?}"),
            })
            .collect();
        assert_eq!(names, ["x", "y"], "source: {src:?}");
    }
}

#[test]
fn blank_quotes_on_same_line_are_double_fence() {
    let file = parse_ok("＠w：\"\"、\"\"\n");
    match &file.items[..] {
        [FileItem::GlobalWord(kw)] => assert_eq!(kw.words, vec!["、".to_string()]),
        other => panic!("expected one GlobalWord, got {other:?}"),
    }
}
