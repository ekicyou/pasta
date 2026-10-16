//! 構文としては読めるが pasta_lua のトランスパイルが失敗する 3 つの書き方を見つける段（3.5）。
//!
//! 条件は pasta_lua の `code_gen`（`element_gen.rs`・`expr_gen.rs`・`scope_gen.rs`）と同じ。
//! pasta_lua は最初の 1 つで止まるが、ここでは見つけたものを全部報告する（失敗するかどうかは同じ）。

use pasta_dsl::PastaFile;
use pasta_dsl::parser::{
    Action, Args, CallTarget, Expr, FileItem, LocalSceneItem, SetValue, Span, VarScope, VarSet,
};

use super::{FileText, Finding, FindingKind, col_after_indent, msg};

/// `texts` は継続行の列を求めるのに使う
pub(crate) fn check(parsed: &[(String, PastaFile)], texts: &[(String, FileText)]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (path, file) in parsed {
        let mut found = |kind: FindingKind, span: Span| {
            let message = match kind {
                FindingKind::TranspileArgAssign => msg::transpile_arg_assign(),
                FindingKind::TranspilePropertyInExpr => msg::transpile_property_in_expr(),
                _ => msg::transpile_continuation_without_actor(),
            };
            findings.push(Finding::new(
                kind,
                path,
                Some(span.start_line as u32),
                Some(span.start_col as u32),
                message,
            ));
        };
        let scenes = file.items.iter().filter_map(|item| match item {
            FileItem::GlobalSceneScope(scene) => Some(scene),
            _ => None,
        });
        for local in scenes.flat_map(|scene| &scene.local_scenes) {
            // アクター行はローカルシーンごとに取り直す（scope_gen.rs の last_actor）
            let mut has_actor = false;
            for item in &local.items {
                match item {
                    LocalSceneItem::VarSet(var_set) => {
                        if let Some(kind) = var_set_error(var_set) {
                            found(kind, var_set.span);
                        }
                    }
                    LocalSceneItem::CallScene(call) => {
                        let key_bad = match &call.target {
                            CallTarget::Static(_) => false,
                            CallTarget::Dynamic(expr) => has_property(expr),
                        };
                        if key_bad || call.args.as_ref().is_some_and(args_have_property) {
                            found(FindingKind::TranspilePropertyInExpr, call.span);
                        }
                    }
                    LocalSceneItem::ActionLine(line) => {
                        has_actor = true;
                        line.actions
                            .iter()
                            .filter_map(action_error_span)
                            .for_each(|s| found(FindingKind::TranspilePropertyInExpr, s));
                    }
                    LocalSceneItem::ContinueAction(cont) => {
                        if !has_actor {
                            // 継続行の span は字下げから始まるので、`：` の列にする
                            let mut span = cont.span;
                            span.start_col = col_after_indent(texts, path, span);
                            found(FindingKind::TranspileContinuationWithoutActor, span);
                        }
                        cont.actions
                            .iter()
                            .filter_map(action_error_span)
                            .for_each(|s| found(FindingKind::TranspilePropertyInExpr, s));
                    }
                    LocalSceneItem::CueCommand(_) | LocalSceneItem::Choice(_) => {}
                }
            }
        }
    }
    findings
}

/// `generate_var_set`: 引数への代入は誤り。プロパティの参照は、ローカル・グローバル変数への
/// 代入の右辺がそのままのときだけ許す。プロパティへの代入は左辺だけが許され、右辺は式として読む
fn var_set_error(var_set: &VarSet) -> Option<FindingKind> {
    if var_set.name.is_some() {
        match var_set.scope {
            VarScope::Args(_) => return Some(FindingKind::TranspileArgAssign),
            VarScope::Local | VarScope::Global
                if matches!(
                    var_set.value,
                    SetValue::Expr(Expr::VarRef {
                        scope: VarScope::Property,
                        ..
                    })
                ) =>
            {
                return None;
            }
            _ => {}
        }
    }
    let bad = match &var_set.value {
        SetValue::Expr(expr) => has_property(expr),
        SetValue::WordRef { .. } => false,
        SetValue::DynamicWordRef { var_scope, .. } => *var_scope == VarScope::Property,
    };
    bad.then_some(FindingKind::TranspilePropertyInExpr)
}

/// `generate_action`: 台詞の中の `＄％名前` は許す。動的な参照の名前と関数の引数は式として読む
fn action_error_span(action: &Action) -> Option<Span> {
    let bad = match action {
        Action::FnCall { args, .. } => args_have_property(args),
        Action::DynamicWordRef { var_scope, .. } => *var_scope == VarScope::Property,
        Action::DynamicFnCall {
            var_scope, args, ..
        } => *var_scope == VarScope::Property || args_have_property(args),
        Action::Talk { .. }
        | Action::WordRef { .. }
        | Action::VarRef { .. }
        | Action::SakuraScript { .. }
        | Action::Escape { .. } => false,
    };
    let span = match action {
        Action::Talk { span, .. }
        | Action::WordRef { span, .. }
        | Action::VarRef { span, .. }
        | Action::FnCall { span, .. }
        | Action::SakuraScript { span, .. }
        | Action::Escape { span, .. }
        | Action::DynamicWordRef { span, .. }
        | Action::DynamicFnCall { span, .. } => *span,
    };
    bad.then_some(span)
}

fn args_have_property(args: &Args) -> bool {
    args.items.iter().any(|arg| match arg {
        pasta_dsl::parser::Arg::Positional(expr)
        | pasta_dsl::parser::Arg::Keyword { value: expr, .. } => has_property(expr),
    })
}

/// `generate_expr_to_buffer`: 式のどこかにプロパティの参照があれば `resolve_var_path` が失敗する
fn has_property(expr: &Expr) -> bool {
    match expr {
        Expr::VarRef { scope, .. } => *scope == VarScope::Property,
        Expr::FnCall { args, .. } => args_have_property(args),
        Expr::DynamicFnCall {
            var_scope, args, ..
        } => *var_scope == VarScope::Property || args_have_property(args),
        Expr::Paren(inner) => has_property(inner),
        Expr::Binary { lhs, rhs, .. } => has_property(lhs) || has_property(rhs),
        Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::BlankString => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 引数への代入は今の文法では書けない（`＄０＝…` は構文エラー）が、AST に来れば
    /// pasta_lua と同じく誤りにする
    #[test]
    fn assignment_to_scene_argument_is_error() {
        let span = Span::new(4, 3, 4, 8, 0, 0);
        let var_set = VarSet {
            name: Some("0".into()),
            scope: VarScope::Args(0),
            value: SetValue::Expr(Expr::Integer(1)),
            span,
        };
        assert_eq!(
            var_set_error(&var_set),
            Some(FindingKind::TranspileArgAssign)
        );

        let mut file = pasta_dsl::parse_str("＊場面\n  ＄x＝1\n", "dic/a.pasta").unwrap();
        let FileItem::GlobalSceneScope(scene) = &mut file.items[0] else {
            panic!("グローバルシーン");
        };
        scene.local_scenes[0].items = vec![LocalSceneItem::VarSet(var_set)];
        let found = check(&[("dic/a.pasta".into(), file)], &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code, FindingKind::TranspileArgAssign);
        assert_eq!((found[0].line, found[0].column), (Some(4), Some(3)));
        assert_eq!(found[0].message, msg::transpile_arg_assign());
    }
}
