//! 静的な飛び先の Call・選択肢を辞書のシーンと組み込みの名前で探し、動的な Call を数える段（6.1〜6.9）。
//!
//! 候補は辞書のシーンと組み込みの名前だけ。Lua の中身は読まない（6.9）。

use std::collections::BTreeMap;

use pasta_core::registry::{scene_order, scene_priority};
use pasta_core::{SceneAliasTable, SceneId, SceneRegistry, SceneTable, SceneTableError};
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
                    match call_candidates(model, &global.registered, aliases, key) {
                        None => findings.push(finding(
                            FindingKind::CallTargetMissing,
                            path,
                            call.span,
                            msg::call_target_missing(key),
                        )),
                        Some(ids) => {
                            for unordered in order_mixed(&model.table, &ids) {
                                let places: Vec<(&str, u32)> = unordered
                                    .iter()
                                    .map(|id| {
                                        let (path, span) = &model.positions[id.0];
                                        (path.as_str(), span.start_line as u32)
                                    })
                                    .collect();
                                findings.push(finding(
                                    FindingKind::CallOrderMixed,
                                    path,
                                    call.span,
                                    msg::call_order_mixed(key, &places),
                                ));
                            }
                        }
                    }
                }
                LocalSceneItem::Choice(choice) => {
                    // choice_select.lua: ローカル → グローバル
                    let target = choice.target.as_str();
                    if not_scene_choice(target) {
                        continue;
                    }
                    if local_candidates(&model.table, &global.registered, target).is_err()
                        && global_candidates(&model.table, aliases, target).is_err()
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

/// Call の候補。act.lua find_act_handler の L2（ローカル）・L3/L4（組み込み）・L5（グローバル）の順で、
/// 最初に見つかった所の候補を返す。組み込みは空、どこにも無ければ None。
/// L1（current_scene の完全一致）は Lua の表なので見ない
fn call_candidates(
    model: &SceneModel,
    registered: &str,
    aliases: &SceneAliasTable,
    key: &str,
) -> Option<Vec<SceneId>> {
    if let Ok(ids) = local_candidates(&model.table, registered, key) {
        return Some(ids);
    }
    if builtins::is_builtin(key) {
        return Some(Vec::new());
    }
    global_candidates(&model.table, aliases, key).ok()
}

/// 候補を ＆優先 の値で組に分け、＆順番 の有るものと無いものが混ざる組の、順番の無い候補（SceneId の順）。
/// 条件は値が分からないので見ない（実行時の select_from_cache は条件を満たす最高の段で同じ判定をする）
fn order_mixed(table: &SceneTable, ids: &[SceneId]) -> Vec<Vec<SceneId>> {
    // 優先 → (順番の有る候補があるか, 順番の無い候補)
    let mut groups: BTreeMap<i64, (bool, Vec<SceneId>)> = BTreeMap::new();
    for &id in ids {
        let Some(info) = table.get_scene(id) else {
            continue;
        };
        let group = groups.entry(scene_priority(&info.attributes)).or_default();
        match scene_order(&info.attributes) {
            Some(_) => group.0 = true,
            None => group.1.push(id),
        }
    }
    groups
        .into_values()
        .filter(|(ordered, unordered)| *ordered && !unordered.is_empty())
        .map(|(_, mut unordered)| {
            unordered.sort_unstable_by_key(|id| id.0);
            unordered
        })
        .collect()
}

/// pasta_lua の search_scene（ローカル）と同じ: 照合用の名前でそのグローバルシーンの配下を前方一致
fn local_candidates(
    table: &SceneTable,
    registered: &str,
    key: &str,
) -> Result<Vec<SceneId>, SceneTableError> {
    table.collect_scene_candidates(registered, &SceneRegistry::sanitize_name(key))
}

/// pasta_lua の search_scene（グローバル）と同じ: 別名を置き換えてから照合用の名前で前方一致
fn global_candidates(
    table: &SceneTable,
    aliases: &SceneAliasTable,
    key: &str,
) -> Result<Vec<SceneId>, SceneTableError> {
    let key = aliases.resolve(key).unwrap_or(key);
    table.collect_scene_candidates("", &SceneRegistry::sanitize_name(key))
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
