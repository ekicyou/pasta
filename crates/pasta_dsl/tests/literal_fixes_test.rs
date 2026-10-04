//! dsl-literal-fixes: 引用文字列・空の引用・単語値の `#` のパース結果を固定する
//!
//! 「エラー行の契約」（design.md）の行番号をそのまま固定する。

use pasta_dsl::ParseError;
use pasta_dsl::parser::{
    Attr, AttrValue, CueArgToken, Expr, FileItem, GlobalSceneScope, KeyWords, LocalSceneItem,
    PastaFile, SetValue, parse_str,
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
        for kw in all_words(&file) {
            assert_eq!(kw.words, [""], "source: {src:?}");
        }
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

// ============================================================================
// 1.6・1.7・4.1〜4.3: 単語値・属性値の引用なしの値（グローバル・ローカル・アクター）
// ============================================================================

/// 単語定義・属性を書く文脈: (名前, 先頭の行, 字下げ, 定義を書いた最初の行の番号)
const DEF_CONTEXTS: [(&str, &str, &str, usize); 3] = [
    ("global", "", "", 1),
    ("local", "＊s", "　", 2),
    ("actor", "％さくら", "　", 2),
];

/// 文脈の中に `lines` を（字下げして）置いたソースを作る
fn in_context(ctx: (&str, &str, &str, usize), lines: &[&str], nl: &str) -> String {
    let (_, head, indent, _) = ctx;
    let mut src = String::new();
    if !head.is_empty() {
        src.push_str(head);
        src.push_str(nl);
    }
    for line in lines {
        src.push_str(indent);
        src.push_str(line);
        src.push_str(nl);
    }
    src
}

fn all_words(file: &PastaFile) -> Vec<&KeyWords> {
    file.items
        .iter()
        .flat_map(|i| match i {
            FileItem::GlobalWord(kw) => vec![kw],
            FileItem::GlobalSceneScope(s) => s.words.iter().collect(),
            FileItem::ActorScope(a) => a.words.iter().collect(),
            _ => vec![],
        })
        .collect()
}

fn all_attrs(file: &PastaFile) -> Vec<&Attr> {
    file.items
        .iter()
        .flat_map(|i| match i {
            FileItem::FileAttr(a) => vec![a],
            FileItem::GlobalSceneScope(s) => s.attrs.iter().collect(),
            FileItem::ActorScope(a) => a.attrs.iter().collect(),
            _ => vec![],
        })
        .collect()
}

/// 1 行の単語定義を各文脈に置き、候補を確かめる
fn assert_word_candidates(line: &str, expected: &[&str]) {
    for ctx in DEF_CONTEXTS {
        let src = in_context(ctx, &[line], "\n");
        let file = parse_ok(&src);
        match &all_words(&file)[..] {
            [kw] => assert_eq!(kw.words, expected, "context: {}, source: {src:?}", ctx.0),
            other => panic!("expected one word def, got {other:?}\nsource: {src:?}"),
        }
    }
}

/// (閉じ忘れた引用, 閉じた引用) の組。閉じた側はパースが通る
const UNCLOSED_QUOTES: [(&[&str], &[&str]); 8] = [
    (&["＠w：「a", "b」"], &["＠w：「ab」"]),
    (&["＠w：「a", "＠v：b」"], &["＠w：「a」", "＠v：b」"]),
    (&["＠w：「abc"], &["＠w：「abc」"]),
    (&["＠w：\"abc"], &["＠w：\"abc\""]),
    (&["＠w：あ、「い"], &["＠w：あ、「い」"]),
    (&["＆k：「a", "b」"], &["＆k：「ab」"]),
    (&["＆k：「abc"], &["＆k：「abc」"]),
    (&["＆k：\"abc"], &["＆k：\"abc\""]),
];

#[test]
fn unclosed_quote_in_word_or_attr_value_errors_at_start_line() {
    for ctx in DEF_CONTEXTS {
        for nl in NEWLINES {
            for (unclosed, closed) in UNCLOSED_QUOTES {
                parse_ok(&in_context(ctx, closed, nl));
                let src = in_context(ctx, unclosed, nl);
                assert_eq!(
                    error_line(&src),
                    ctx.3,
                    "context: {}, source: {src:?}",
                    ctx.0
                );
            }
        }
    }
}

#[test]
fn quote_char_after_first_char_is_part_of_word_value() {
    assert_word_candidates("＠w：あ「い、う\"え", &["あ「い", "う\"え"]);
}

#[test]
fn quote_char_after_first_char_is_part_of_attr_value() {
    for ctx in DEF_CONTEXTS {
        let src = in_context(ctx, &["＆k：a「b"], "\n");
        let file = parse_ok(&src);
        match &all_attrs(&file)[..] {
            [attr] => assert_eq!(
                attr.value,
                AttrValue::AttrString("a「b".to_string()),
                "context: {}, source: {src:?}",
                ctx.0
            ),
            other => panic!("expected one attr, got {other:?}\nsource: {src:?}"),
        }
    }
}

#[test]
fn hash_in_unquoted_word_value_is_part_of_value() {
    assert_word_candidates("＠w：あ、い ＃c", &["あ", "い ＃c"]);
    assert_word_candidates("＠話題：＃伺か", &["＃伺か"]);
}

// ============================================================================
// 3.1〜3.3: 空の引用 `「」`・`""` は空文字列
// ============================================================================

#[test]
fn blank_quote_in_word_value_is_empty_string() {
    assert_word_candidates("＠w：「」、\"\"", &["", ""]);
    assert_word_candidates("＠w：「」、あ", &["", "あ"]);
}

#[test]
fn blank_quote_in_attr_value_is_empty_string() {
    for ctx in DEF_CONTEXTS {
        for line in ["＆k：「」", "＆k：\"\""] {
            let src = in_context(ctx, &[line], "\n");
            let file = parse_ok(&src);
            match &all_attrs(&file)[..] {
                [attr] => assert_eq!(
                    attr.value,
                    AttrValue::String(String::new()),
                    "context: {}, source: {src:?}",
                    ctx.0
                ),
                other => panic!("expected one attr, got {other:?}\nsource: {src:?}"),
            }
        }
    }
}

#[test]
fn blank_quote_in_expression_is_empty_string() {
    for q in ["「」", "\"\""] {
        let src = format!("＊s\n　＄x＝{q}\n");
        let file = parse_ok(&src);
        match &first_scene(&file).local_scenes[0].items[0] {
            LocalSceneItem::VarSet(vs) => {
                assert_eq!(
                    vs.value,
                    SetValue::Expr(Expr::BlankString),
                    "source: {src:?}"
                )
            }
            other => panic!("expected VarSet, got {other:?}"),
        }
    }
}

#[test]
fn blank_quote_in_cue_command_arg_is_empty_string() {
    for q in ["「」", "\"\""] {
        let src = format!("＊s\n　！cmd（{q}）\n");
        let file = parse_ok(&src);
        match &first_scene(&file).local_scenes[0].items[0] {
            LocalSceneItem::CueCommand(node) => assert_eq!(
                node.args,
                [CueArgToken::StringLiteral(String::new())],
                "source: {src:?}"
            ),
            other => panic!("expected CueCommand, got {other:?}"),
        }
    }
}

#[test]
fn hash_after_quoted_word_value_is_comment() {
    assert_word_candidates("＠w：「あ」 ＃c", &["あ"]);
}
