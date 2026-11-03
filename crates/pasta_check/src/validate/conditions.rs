//! 条件の保存変数の未代入と、優先で塞がれた場面を注意する段（8.1〜8.5）。
//!
//! 構文木を読むだけで、Lua の中の代入は見ない（だから注意にとどめる）。

use std::collections::{BTreeMap, BTreeSet};

use pasta_core::SceneRegistry;
use pasta_core::registry::{SceneAttr, has_conditions, scene_priority};
use pasta_dsl::PastaFile;
use pasta_dsl::parser::{Attr, AttrBody, FileItem, LocalSceneItem, Span, VarScope, merge_attrs};

use super::scenes::{SceneModel, scene_attrs_of};
use super::{Finding, FindingKind, msg};

pub(crate) fn check(model: &SceneModel, parsed: &[(String, PastaFile)]) -> Vec<Finding> {
    let mut findings = unassigned(parsed);
    findings.extend(shadowed(model, parsed));
    findings
}

/// 条件・フラグの `＄＊名前` のうち、辞書のどこでも `＄＊名前＝…` していないもの（8.1・8.3）
fn unassigned(parsed: &[(String, PastaFile)]) -> Vec<Finding> {
    let mut assigned = BTreeSet::new();
    // 属性は書いた所ごとに 1 回（ファイル属性は受け継いだシーンの数でなく、その行に 1 回）
    let mut written: Vec<(&str, &Attr)> = Vec::new();
    for (path, file) in parsed {
        for item in &file.items {
            match item {
                FileItem::FileAttr(attr) => written.push((path, attr)),
                FileItem::GlobalSceneScope(scene) => {
                    written.extend(scene.attrs.iter().map(|a| (path.as_str(), a)));
                    for local in &scene.local_scenes {
                        written.extend(local.attrs.iter().map(|a| (path.as_str(), a)));
                        for item in &local.items {
                            if let LocalSceneItem::VarSet(set) = item {
                                assigned.extend(global_set(set));
                            }
                        }
                    }
                }
                FileItem::ActorScope(actor) => {
                    assigned.extend(actor.var_sets.iter().filter_map(global_set));
                }
                FileItem::GlobalWord(_) => {}
            }
        }
    }
    written
        .into_iter()
        .filter(|(_, a)| !matches!(a.body, AttrBody::Decl(_)))
        .filter_map(|(path, a)| Some((path, a, a.key.strip_prefix("＄＊")?)))
        .filter(|(_, _, name)| !assigned.contains(name))
        .map(|(path, a, name)| {
            finding(
                FindingKind::CondVarUnassigned,
                path,
                a.span,
                msg::cond_var_unassigned(name),
            )
        })
        .collect()
}

fn global_set(set: &pasta_dsl::parser::VarSet) -> Option<&str> {
    match set.scope {
        VarScope::Global => set.name.as_deref(),
        _ => None,
    }
}

struct Candidate<'a> {
    name: &'a str,
    path: &'a str,
    span: Span,
    attrs: Vec<SceneAttr>,
}

/// 同じ照合用の名前の候補に、条件もフラグも無い優先 P の場面があれば、P より低い場面を注意する（8.5）。
/// グループはグローバルどうし（別名を置き換えた後）と、同じ親の中のローカルどうし
fn shadowed(model: &SceneModel, parsed: &[(String, PastaFile)]) -> Vec<Finding> {
    // キーは（親の登録名。グローバルは ""、照合用の名前）
    let mut groups: BTreeMap<(&str, String), Vec<Candidate>> = BTreeMap::new();
    // model.globals は構文木の順に積んだもの（単独の `＊` も 1 つ）なので、同じ順で並ぶ
    let scenes = parsed.iter().flat_map(|(path, file)| {
        file.items
            .iter()
            .filter_map(|item| match item {
                FileItem::GlobalSceneScope(scene) => Some(scene),
                _ => None,
            })
            .zip(file.global_scene_attrs())
            .map(move |(scene, attrs)| (path.as_str(), scene, attrs))
    });
    for ((path, scene, attrs), global) in scenes.zip(&model.globals) {
        groups
            .entry(("", SceneRegistry::sanitize_name(&global.declared)))
            .or_default()
            .push(Candidate {
                name: &scene.name,
                path,
                span: scene.span,
                attrs: scene_attrs_of(&attrs),
            });
        for local in &scene.local_scenes {
            if let Some(name) = &local.name {
                groups
                    .entry((&global.registered, SceneRegistry::sanitize_name(name)))
                    .or_default()
                    .push(Candidate {
                        name,
                        path,
                        span: local.span,
                        // ローカルは受け継がない（scenes.rs の登録と同じ）
                        attrs: scene_attrs_of(&merge_attrs(&[], &local.attrs)),
                    });
            }
        }
    }

    let mut findings = Vec::new();
    for group in groups.values() {
        // 最も高い優先の、条件の無い場面（同じ優先なら先に書いたもの）
        let Some((blocker, top)) = group
            .iter()
            .filter(|c| !has_conditions(&c.attrs))
            .map(|c| (c, scene_priority(&c.attrs)))
            .reduce(|best, c| if c.1 > best.1 { c } else { best })
        else {
            continue;
        };
        for c in group.iter().filter(|c| scene_priority(&c.attrs) < top) {
            findings.push(finding(
                FindingKind::SceneShadowedByPriority,
                c.path,
                c.span,
                msg::scene_shadowed_by_priority(
                    c.name,
                    top,
                    blocker.path,
                    blocker.span.start_line as u32,
                ),
            ));
        }
    }
    findings
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
