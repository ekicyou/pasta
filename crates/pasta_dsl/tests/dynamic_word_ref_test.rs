//! 動的単語参照（＠＄変数名）・動的関数呼び出し（＠＄変数名（…））のパーサーテスト
//!
//! Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.8,
//! 7.1, 7.2, 7.3

use pasta_dsl::ParseError;
use pasta_dsl::parser::{
    Action, Arg, BinOp, CallTarget, Expr, FileItem, FnScope, LocalSceneItem, PastaFile, SetValue,
    VarScope, VarSet, parse_str,
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

/// 台詞（Talk）だけを取り出す
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

// ========================================================================
// R1.1–1.3, 1.6, 1.7: アクション行の動的単語参照（スコープ・名前・位置）
// ========================================================================

#[test]
fn test_dynamic_word_ref_ast_has_scope_name_and_span() {
    for (body, var_name, var_scope, text) in [
        ("＠＄x　です", "x", VarScope::Local, "＠＄x　"),
        ("＠＄＊x　です", "x", VarScope::Global, "＠＄＊x　"),
        ("＠＄０　です", "０", VarScope::Args(0), "＠＄０　"),
        ("@$x です", "x", VarScope::Local, "@$x "),
        ("＠$＊x　です", "x", VarScope::Global, "＠$＊x　"),
    ] {
        let source = source_of(body);
        let file = parse_str(&source, "test.pasta").unwrap();
        let actions = first_action_line(&file);
        assert_eq!(actions.len(), 2, "body {body:?}: {actions:?}");
        match &actions[0] {
            Action::DynamicWordRef {
                var_name: n,
                var_scope: sc,
                span,
            } => {
                assert_eq!(n, var_name, "body {body:?}");
                assert_eq!(*sc, var_scope, "body {body:?}");
                assert_eq!((span.start_line, span.start_col), (2, BODY_COLUMN));
                assert_eq!(span.extract_source(&source).unwrap(), text);
            }
            other => panic!("expected DynamicWordRef for {body:?}, got {other:?}"),
        }
        // 末尾の空白は区切りとして消費され、台詞に出ない（R1.7）
        assert_eq!(shape(&actions[1]), "Talk(です)", "body {body:?}");
    }
}

#[test]
fn test_consecutive_dynamic_word_refs_emit_no_whitespace() {
    assert_eq!(
        shapes_of("＠＄x　＠＄＊y "),
        vec!["DynamicWordRef(x,Local)", "DynamicWordRef(y,Global)"]
    );
}

// ========================================================================
// R1.4, 1.5: 代入の右辺の動的単語参照（変数参照に化けない）
// ========================================================================

fn var_set_of(line: &str) -> VarSet {
    let file = parse_str(&format!("＊テスト\n　{line}\n"), "test.pasta")
        .unwrap_or_else(|e| panic!("parse failed for {line:?}: {e}"));
    file.items
        .into_iter()
        .filter_map(|item| match item {
            FileItem::GlobalSceneScope(gs) => Some(gs),
            _ => None,
        })
        .flat_map(|gs| gs.local_scenes)
        .flat_map(|ls| ls.items)
        .find_map(|item| match item {
            LocalSceneItem::VarSet(vs) => Some(vs),
            _ => None,
        })
        .expect("var set not found")
}

#[test]
fn test_var_set_rhs_dynamic_word_ref() {
    for (line, name, scope, var_name, var_scope) in [
        (
            "＄y＝＠＄x",
            Some("y"),
            VarScope::Local,
            "x",
            VarScope::Local,
        ),
        (
            "＄＊y＝＠＄＊x",
            Some("y"),
            VarScope::Global,
            "x",
            VarScope::Global,
        ),
        ("＄＝＠＄x", None, VarScope::Local, "x", VarScope::Local),
        (
            "＄％p＝＠＄x",
            Some("p"),
            VarScope::Property,
            "x",
            VarScope::Local,
        ),
        ("$y=@$0", Some("y"), VarScope::Local, "0", VarScope::Args(0)),
    ] {
        let vs = var_set_of(line);
        assert_eq!(vs.name.as_deref(), name, "line {line:?}");
        assert_eq!(vs.scope, scope, "line {line:?}");
        assert_eq!(
            vs.value,
            SetValue::DynamicWordRef {
                var_name: var_name.to_string(),
                var_scope,
            },
            "line {line:?}"
        );
    }
}

// ========================================================================
// R2.4, 7.1–7.3: 動的関数呼び出し（アクション行・式の中・Call ターゲット）
// ========================================================================

#[test]
fn test_dynamic_fn_call_in_action_line() {
    let source = source_of("＠＄f（１、＄a）です");
    let file = parse_str(&source, "test.pasta").unwrap();
    let actions = first_action_line(&file);
    match &actions[0] {
        Action::DynamicFnCall {
            var_name,
            var_scope,
            args,
            span,
        } => {
            assert_eq!(var_name, "f");
            assert_eq!(*var_scope, VarScope::Local);
            assert_eq!(
                args.items,
                vec![
                    Arg::Positional(Expr::Integer(1)),
                    Arg::Positional(Expr::VarRef {
                        name: "a".to_string(),
                        scope: VarScope::Local,
                    }),
                ]
            );
            assert_eq!((span.start_line, span.start_col), (2, BODY_COLUMN));
            assert_eq!(span.extract_source(&source).unwrap(), "＠＄f（１、＄a）");
            assert_eq!(args.span.extract_source(&source).unwrap(), "（１、＄a）");
        }
        other => panic!("expected DynamicFnCall, got {other:?}"),
    }
    assert_eq!(shape(&actions[1]), "Talk(です)");
}

#[test]
fn test_dynamic_fn_call_scopes_in_action_line() {
    assert_eq!(
        shapes_of("＠＄＊f（１）＠＄０（）"),
        vec!["DynamicFnCall(f,Global,1)", "DynamicFnCall(０,Args(0),0)"]
    );
}

/// ローカル変数 `var_name` を参照変数とし、引数が `items` の動的関数呼び出しか
fn is_dyn_call(expr: &Expr, var_name: &str, items: &[Arg]) -> bool {
    matches!(
        expr,
        Expr::DynamicFnCall { var_name: n, var_scope: VarScope::Local, args }
            if n == var_name && args.items == items
    )
}

#[test]
fn test_var_set_rhs_dynamic_fn_call_keeps_call() {
    // 既定アームが子を再帰すると `＄y＝１` に化ける
    match &var_set_of("＄y＝＠＄f（１）").value {
        SetValue::Expr(expr) => assert!(
            is_dyn_call(expr, "f", &[Arg::Positional(Expr::Integer(1))]),
            "{expr:?}"
        ),
        other => panic!("expected Expr, got {other:?}"),
    }
}

#[test]
fn test_var_set_rhs_dynamic_fn_call_in_arithmetic() {
    match &var_set_of("＄y＝＠＄f（）＋１").value {
        SetValue::Expr(Expr::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
        }) => {
            assert!(is_dyn_call(lhs, "f", &[]), "{lhs:?}");
            assert_eq!(**rhs, Expr::Integer(1));
        }
        other => panic!("expected Binary Add, got {other:?}"),
    }
}

#[test]
fn test_dynamic_fn_call_as_static_fn_call_arg() {
    match &actions_of("＠g（＠＄f（））")[0] {
        Action::FnCall { name, args, .. } => {
            assert_eq!(name, "g");
            match &args.items[..] {
                [Arg::Positional(expr)] => assert!(is_dyn_call(expr, "f", &[]), "{expr:?}"),
                other => panic!("expected one positional arg, got {other:?}"),
            }
        }
        other => panic!("expected FnCall, got {other:?}"),
    }
}

#[test]
fn test_dynamic_fn_call_as_call_target() {
    let file = parse_str("＊テスト\n　＞＠＄f（）\n", "test.pasta").unwrap();
    let call = file
        .items
        .into_iter()
        .filter_map(|item| match item {
            FileItem::GlobalSceneScope(gs) => Some(gs),
            _ => None,
        })
        .flat_map(|gs| gs.local_scenes)
        .flat_map(|ls| ls.items)
        .find_map(|item| match item {
            LocalSceneItem::CallScene(cs) => Some(cs),
            _ => None,
        })
        .expect("call scene not found");
    match &call.target {
        CallTarget::Dynamic(expr) => assert!(is_dyn_call(expr, "f", &[]), "{expr:?}"),
        other => panic!("expected dynamic target, got {other:?}"),
    }
}

#[test]
fn test_dynamic_fn_call_with_invalid_args_splits_into_word_ref_and_talk() {
    // 静的の `＠f（時間：朝）` と同じく、単語参照＋台詞に分かれる（R7.1 ただし書き）
    assert_eq!(
        shapes_of("＠f（時間：朝）"),
        vec!["WordRef(f)", "Talk(（時間：朝）)"]
    );
    assert_eq!(
        shapes_of("＠＄f（時間：朝）"),
        vec!["DynamicWordRef(f,Local)", "Talk(（時間：朝）)"]
    );
}
