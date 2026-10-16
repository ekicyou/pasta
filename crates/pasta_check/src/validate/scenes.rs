//! シーンを pasta_lua のトランスパイラと同じ手順で登録し、名前の重なりを見つける段（2.4・5.1〜5.4）。

use std::collections::{BTreeMap, HashMap};

use pasta_core::{MockRandomSelector, SceneAliasTable, SceneRegistry, SceneTable};
use pasta_dsl::PastaFile;
use pasta_dsl::parser::{Attr, FileItem, Span};

use super::{Finding, FindingKind, builtins, msg};

/// 登録したシーン。References が飛び先を探すのに読む
pub(crate) struct SceneModel {
    pub table: SceneTable,
    /// グローバルシーン（構文木の順。単独の `＊` も 1 つ）
    pub globals: Vec<GlobalScene>,
}

pub(crate) struct GlobalScene {
    pub path: String,
    /// 書いた名前（単独の `＊` は parser が入れた前の名前）
    pub written: String,
    /// 別名を置き換えた後の名前
    pub declared: String,
    /// 登録名（`registered_name(declared, 通し番号)`）。ローカルの検索の範囲になる
    pub registered: String,
    pub span: Span,
}

/// 登録したシーンの表と、名前の重なりの注意を返す。表には乱数源を使わない空のモック選択子を渡す
pub(crate) fn check(
    parsed: &[(String, PastaFile)],
    aliases: &SceneAliasTable,
) -> (SceneModel, Vec<Finding>) {
    // pasta_lua の transpiler.rs process_global_scene と同じ手順。
    // 実行時の通し番号（scene.lua の create_scene）は全ファイルで 1 つなので、登録表も 1 つ
    let mut registry = SceneRegistry::new();
    let mut globals = Vec::new();
    let mut findings = Vec::new();
    // 名前を書いたグローバルシーン（単独の `＊` でないもの）の globals での位置
    let mut named = Vec::new();
    for (path, file) in parsed {
        let scenes = file.items.iter().filter_map(|item| match item {
            FileItem::GlobalSceneScope(scene) => Some(scene),
            _ => None,
        });
        for scene in scenes {
            let declared = aliases.resolve(&scene.name).unwrap_or(&scene.name);
            let (_, counter) = registry.register_global(declared, attr_map(&scene.attrs));
            for (local, local_counter) in scene.local_scenes.iter().zip(local_counters(scene)) {
                if let Some(name) = &local.name {
                    registry.register_local(
                        name,
                        declared,
                        counter,
                        local_counter,
                        attr_map(&local.attrs),
                    );
                }
            }

            // ローカルの重なりは単独の `＊` の中も 1 つのグローバルシーンとして見る
            let locals: Vec<Decl> = scene
                .local_scenes
                .iter()
                .filter_map(|l| l.name.as_deref().map(|n| (n, n, path.as_str(), l.span)))
                .collect();
            collisions(&mut findings, FindingKind::LocalSceneCollision, &locals);
            // 単独の `＊` は名前を書いた宣言ではないので、グローバルの名前としては報告しない
            if !scene.is_continuation {
                if builtins::is_builtin(declared) {
                    findings.push(finding(
                        FindingKind::SceneShadowedByBuiltin,
                        path,
                        scene.span,
                        msg::scene_shadowed_by_builtin(&scene.name),
                    ));
                }
                named.push(globals.len());
            }
            globals.push(GlobalScene {
                path: path.clone(),
                written: scene.name.clone(),
                declared: declared.to_string(),
                registered: SceneRegistry::registered_name(declared, counter),
                span: scene.span,
            });
        }
    }

    let decls: Vec<Decl> = named
        .iter()
        .map(|&i| {
            let g = &globals[i];
            (
                g.written.as_str(),
                g.declared.as_str(),
                g.path.as_str(),
                g.span,
            )
        })
        .collect();
    collisions(&mut findings, FindingKind::GlobalSceneCollision, &decls);

    let table =
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![])))
            .expect("from_scene_registry は失敗を返さない");
    (SceneModel { table, globals }, findings)
}

/// (書いた名前, 比べる名前, パス, 位置)
type Decl<'a> = (&'a str, &'a str, &'a str, Span);

/// 照合用の名前が同じで比べる名前が違う宣言に、1 つずつ注意を付ける（5.1・5.2）。
/// 比べる名前が同じもの（同名・別名を置き換えて同じ）は報告しない（5.3）。
/// 示すほかの宣言は、比べる名前が違うもののうち最初の 1 つ
fn collisions(findings: &mut Vec<Finding>, code: FindingKind, decls: &[Decl]) {
    let mut groups: BTreeMap<String, Vec<&Decl>> = BTreeMap::new();
    for d in decls {
        groups
            .entry(SceneRegistry::sanitize_name(d.1))
            .or_default()
            .push(d);
    }
    for (match_name, group) in &groups {
        for &&(written, compared, path, span) in group {
            if let Some(&&(other, _, other_path, other_span)) =
                group.iter().find(|o| o.1 != compared)
            {
                findings.push(finding(
                    code,
                    path,
                    span,
                    msg::scene_collision(
                        written,
                        match_name,
                        other,
                        other_path,
                        other_span.start_line as u32,
                    ),
                ));
            }
        }
    }
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

/// pasta_lua の context.rs register_global_scene_named・register_local_scene と同じ属性の形
fn attr_map(attrs: &[Attr]) -> HashMap<String, String> {
    attrs
        .iter()
        .map(|a| (a.key.clone(), a.value.to_string()))
        .collect()
}

/// pasta_lua の context.rs local_scene_counters と同じ: 照合用の名前ごとの通し番号（名前の無い開始のブロックは 0）
fn local_counters(scene: &pasta_dsl::parser::GlobalSceneScope) -> Vec<usize> {
    let mut counters: HashMap<String, usize> = HashMap::new();
    scene
        .local_scenes
        .iter()
        .map(|local| match &local.name {
            Some(name) => {
                let count = counters
                    .entry(SceneRegistry::sanitize_name(name))
                    .or_insert(0);
                *count += 1;
                *count
            }
            None => 0,
        })
        .collect()
}
