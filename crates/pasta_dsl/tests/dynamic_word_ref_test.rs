//! 動的単語参照（＠＄変数名）・動的関数呼び出し（＠＄変数名（…））のパーサーテスト
//!
//! Requirements: 1.6, 2.1, 2.2, 2.3, 2.5, 2.6, 2.8

use pasta_dsl::ParseError;
use pasta_dsl::parser::{
    Action, FileItem, FnScope, LocalSceneItem, PastaFile, VarScope, parse_str,
};

/// アクション行 `　さくら：{body}` を 1 行だけ持つシーンのソース（本文は 2 行目）
fn source_of(body: &str) -> String {
    format!("＊テスト\n　さくら：{}\n", body)
}

/// 本文の `＠` が置かれる 2 行目の列（`　さくら：` の直後）
const BODY_COLUMN: usize = 6;

fn first_action_line(file: &PastaFile) -> Vec<Action> {
    file.items
        .iter()
        .filter_map(|item| match item {
            FileItem::GlobalSceneScope(gs) => Some(gs),
            _ => None,
        })
        .flat_map(|gs| &gs.local_scenes)
        .flat_map(|ls| &ls.items)
        .find_map(|item| match item {
            LocalSceneItem::ActionLine(al) => Some(al.actions.clone()),
            _ => None,
        })
        .expect("action line not found")
}

fn actions_of(body: &str) -> Vec<Action> {
    let file = parse_str(&source_of(body), "test.pasta")
        .unwrap_or_else(|e| panic!("parse failed for {body:?}: {e}"));
    first_action_line(&file)
}

/// 位置情報を除いた AST の形（従来の AST との比較用）
fn shape(action: &Action) -> String {
    match action {
        Action::Talk { text, .. } => format!("Talk({text})"),
        Action::WordRef { name, .. } => format!("WordRef({name})"),
        Action::VarRef { name, scope, .. } => format!("VarRef({name},{scope:?})"),
        Action::FnCall {
            name, args, scope, ..
        } => format!("FnCall({name},{scope:?},{})", args.items.len()),
        Action::SakuraScript { script, .. } => format!("SakuraScript({script})"),
        Action::Escape { sequence, .. } => format!("Escape({sequence})"),
        Action::DynamicWordRef {
            var_name,
            var_scope,
            ..
        } => format!("DynamicWordRef({var_name},{var_scope:?})"),
        Action::DynamicFnCall {
            var_name,
            var_scope,
            args,
            ..
        } => format!(
            "DynamicFnCall({var_name},{var_scope:?},{})",
            args.items.len()
        ),
    }
}

fn shapes_of(body: &str) -> Vec<String> {
    actions_of(body).iter().map(shape).collect()
}

/// 台詞（Talk）だけを取り出す。動的参照そのものの AST はタスク 1.3 の範囲
fn talks_of(body: &str) -> Vec<String> {
    actions_of(body)
        .into_iter()
        .filter_map(|a| match a {
            Action::Talk { text, .. } => Some(text),
            _ => None,
        })
        .collect()
}

fn assert_parse_error_on_body_line(body: &str) {
    match parse_str(&source_of(body), "test.pasta") {
        Err(ParseError::SyntaxError { line, column, .. }) => {
            assert_eq!(line, 2, "line for {body:?}");
            assert!(
                column >= BODY_COLUMN,
                "column {column} for {body:?} should point at or after the marker"
            );
        }
        other => panic!("expected SyntaxError for {body:?}, got {other:?}"),
    }
}

// ========================================================================
// R1.6: 全角半角のマーカーを同一に扱う／動的単語参照だけが末尾の空白を消費する
// ========================================================================

#[test]
fn test_dynamic_word_ref_accepts_fullwidth_and_halfwidth_markers() {
    for body in [
        "＠＄x　です",
        "@$x　です",
        "＠$x　です",
        "@＄x　です",
        "＠＄＊x　です",
        "@$*x　です",
        "＠$＊x　です",
        "@＄*x　です",
        "＠＄０　です",
        "@$0　です",
    ] {
        assert_eq!(talks_of(body), vec!["です"], "body {body:?}");
    }
}

#[test]
fn test_dynamic_fn_call_accepts_fullwidth_and_halfwidth_markers() {
    for body in [
        "＠＄f（）です",
        "@$f()です",
        "＠$＊f（１）です",
        "＠＄０（）です",
    ] {
        assert_eq!(talks_of(body), vec!["です"], "body {body:?}");
    }
}

#[test]
fn test_dynamic_fn_call_requires_paren_right_after_name() {
    // `＠＄f （）` は静的の `＠f （）` と同じく単語参照＋台詞「（）」
    assert_eq!(talks_of("＠＄f （）"), vec!["（）"]);
    assert_eq!(shapes_of("＠f （）"), vec!["WordRef(f)", "Talk(（）)"]);
}

// ========================================================================
// R2.1: ＠＠ はエスケープのまま（＠＠＄x は「＠」＋変数展開）
// ========================================================================

#[test]
fn test_at_escape_followed_by_var_ref() {
    for (body, escape) in [("＠＠＄x", "＠＠"), ("＠@＄x", "＠@"), ("@@$x", "@@")] {
        assert_eq!(
            shapes_of(body),
            vec![format!("Escape({escape})"), "VarRef(x,Local)".to_string()],
            "body {body:?}"
        );
    }
}

// ========================================================================
// R2.2: ＄＄ はエスケープのまま
// ========================================================================

#[test]
fn test_dollar_escape_unchanged() {
    assert_eq!(shapes_of("＄＄です"), vec!["Escape(＄＄)", "Talk(です)"]);
    assert_eq!(shapes_of("$$x"), vec!["Escape($$)", "Talk(x)"]);
}

// ========================================================================
// R2.3: 静的の ＠名前・＠名前（）・＠＊名前（） は従来どおり
// ========================================================================

#[test]
fn test_static_word_ref_unchanged() {
    assert_eq!(
        shapes_of("＠名前　です"),
        vec!["WordRef(名前)", "Talk(です)"]
    );
}

#[test]
fn test_static_fn_call_unchanged() {
    let actions = actions_of("＠名前（）です");
    assert_eq!(
        actions.iter().map(shape).collect::<Vec<_>>(),
        vec!["FnCall(名前,Local,0)", "Talk(です)"]
    );
    assert!(matches!(
        &actions[0],
        Action::FnCall {
            scope: FnScope::Local,
            ..
        }
    ));
}

#[test]
fn test_static_global_fn_call_unchanged() {
    assert_eq!(
        shapes_of("＠＊名前（）です"),
        vec!["FnCall(名前,Global,0)", "Talk(です)"]
    );
    assert_eq!(
        shapes_of("＠＊名前（１、２）"),
        vec!["FnCall(名前,Global,2)"]
    );
}

#[test]
fn test_var_ref_unchanged() {
    assert_eq!(
        shapes_of("＄x＄＊y"),
        vec!["VarRef(x,Local)", "VarRef(y,Global)"]
    );
    let actions = actions_of("＄０");
    assert!(matches!(
        &actions[0],
        Action::VarRef {
            scope: VarScope::Args(0),
            ..
        }
    ));
}

// ========================================================================
// R2.5: ＠＄％ はパースエラー
// ========================================================================

#[test]
fn test_property_dynamic_word_ref_is_parse_error() {
    assert_parse_error_on_body_line("＠＄％p");
    assert_parse_error_on_body_line("@$%p");
}

#[test]
fn test_property_dynamic_fn_call_is_parse_error() {
    assert_parse_error_on_body_line("＠＄％p（）");
}

// ========================================================================
// R2.6: ＠＄ の後に変数名が無いとパースエラー
// ========================================================================

#[test]
fn test_dynamic_word_ref_without_name_is_parse_error() {
    for body in ["＠＄", "＠＄＄", "＠＄　x", "@$ x", "＠＄＊", "＠＄＊０"] {
        assert_parse_error_on_body_line(body);
    }
}

// ========================================================================
// R2.8: ＠＊＄ はパースエラー
// ========================================================================

#[test]
fn test_global_dynamic_fn_call_is_parse_error() {
    assert_parse_error_on_body_line("＠＊＄x（）");
    assert_parse_error_on_body_line("@*$x()");
}
