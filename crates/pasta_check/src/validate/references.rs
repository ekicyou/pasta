//! 静的な飛び先の Call・選択肢を辞書のシーンと組み込みの名前で探し、動的な Call を数える段（6.1〜6.9）。
//!
//! 候補は辞書のシーンと組み込みの名前だけ。Lua の中身は読まない（6.9）。

use pasta_core::{SceneAliasTable, SceneRegistry, SceneTable};
use pasta_dsl::PastaFile;
use pasta_dsl::parser::{CallTarget, Expr, FileItem, LocalSceneItem, Span};

use super::scenes::SceneModel;
use super::{FileText, Finding, FindingKind, builtins, col_after_indent, msg};

/// 飛び先の注意と、飛び先を検査しなかった動的な Call の数を返す。`texts` は選択肢の列を求めるのに使う
pub(crate) fn check(
    model: &SceneModel,
    aliases: &SceneAliasTable,
    parsed: &[(String, PastaFile)],
    texts: &[(String, FileText)],
) -> (Vec<Finding>, usize) {
    let mut findings = Vec::new();
    let mut dynamic = 0;
    // model.globals は Scenes が構文木の順に積んだもの（単独の `＊` も 1 つ）なので、同じ順で並ぶ
    let scenes = parsed.iter().flat_map(|(path, file)| {
        file.items.iter().filter_map(move |item| match item {
            FileItem::GlobalSceneScope(scene) => Some((path, scene)),
            _ => None,
        })
    });
    for ((path, scene), global) in scenes.zip(&model.globals) {
        let items = scene.local_scenes.iter().flat_map(|l| &l.items);
        for item in items {
            match item {
                LocalSceneItem::CallScene(call) => {
                    let key = match &call.target {
                        CallTarget::Static(key) => key.as_str(),
                        CallTarget::Dynamic(expr) => match string_literal(expr) {
                            Some(key) => key,
                            None => {
                                dynamic += 1;
                                continue;
                            }
                        },
                    };
                    // act.lua find_act_handler の L2（ローカル）・L3/L4（組み込み）・L5（グローバル）。
                    // L1（current_scene の完全一致）は Lua の表なので見ない
                    if !(found_local(&model.table, &global.registered, key)
                        || builtins::is_builtin(key)
                        || found_global(&model.table, aliases, key))
                    {
                        findings.push(finding(
                            FindingKind::CallTargetMissing,
                            path,
                            call.span,
                            msg::call_target_missing(key),
                        ));
                    }
                }
                LocalSceneItem::Choice(choice) => {
                    // choice_select.lua: ローカル → グローバル
                    let target = choice.target.as_str();
                    if not_scene_choice(target) {
                        continue;
                    }
                    if !(found_local(&model.table, &global.registered, target)
                        || found_global(&model.table, aliases, target))
                    {
                        // 選択肢の行の span は字下げから始まるので、Call と同じく `＠？` の列にする
                        let mut span = choice.span;
                        span.start_col = col_after_indent(texts, path, span);
                        findings.push(finding(
                            FindingKind::ChoiceTargetMissing,
                            path,
                            span,
                            msg::choice_target_missing(target),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    (findings, dynamic)
}

/// pasta_lua の search_scene（ローカル）と同じ: 照合用の名前でそのグローバルシーンの配下を前方一致
fn found_local(table: &SceneTable, registered: &str, key: &str) -> bool {
    table
        .collect_scene_candidates(registered, &SceneRegistry::sanitize_name(key))
        .is_ok()
}

/// pasta_lua の search_scene（グローバル）と同じ: 別名を置き換えてから照合用の名前で前方一致
fn found_global(table: &SceneTable, aliases: &SceneAliasTable, key: &str) -> bool {
    let key = aliases.resolve(key).unwrap_or(key);
    table
        .collect_scene_candidates("", &SceneRegistry::sanitize_name(key))
        .is_ok()
}

/// 文字列リテラルだけの式（括弧で囲んだものも）。実行時は書いた名前で `act:call_key` になる（6.2）
fn string_literal(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::String(key) => Some(key),
        Expr::Paren(inner) => string_literal(inner),
        _ => None,
    }
}

/// `On`・`script:` で始まるジャンプ先は SSP が別の意味で使うので見ない（6.5。sakura_builder.lua と同じ判定）
fn not_scene_choice(target: &str) -> bool {
    target.starts_with("On") || target.starts_with("script:")
}

fn finding(code: FindingKind, path: &str, span: Span, message: String) -> Finding {
    Finding::new(
        code,
        path,
        Some(span.start_line as u32),
        Some(span.start_col as u32),
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `＠？script:…` は DSL の id に `:` が書けないので、判定だけをここで確かめる
    #[test]
    fn on_and_script_choices_are_not_checked() {
        assert!(not_scene_choice("OnBoot"));
        assert!(not_scene_choice("script:\\e"));
        assert!(!not_scene_choice("選択"));
        assert!(!not_scene_choice("scripted"));
    }
}
