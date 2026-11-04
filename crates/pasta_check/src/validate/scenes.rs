//! シーンを pasta_lua のトランスパイラと同じ手順で登録し、名前の重なりを見つける段（2.4・5.1〜5.4）。

use std::collections::{BTreeMap, HashMap};

use pasta_core::{
    CmpOp, MockRandomSelector, SceneAliasTable, SceneAttr, SceneAttrValue, SceneRegistry,
    SceneTable,
};
use pasta_dsl::PastaFile;
use pasta_dsl::parser::{Attr, AttrBody, AttrValue, FileItem, Span, merge_attrs};

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
    // SceneId の順（登録の順）の (パス, 書いた名前, 位置)。番号の重なりの報告に使う
    let mut positions: Vec<(&str, &str, Span)> = Vec::new();
    for (path, file) in parsed {
        let scenes = file.items.iter().filter_map(|item| match item {
            FileItem::GlobalSceneScope(scene) => Some(scene),
            _ => None,
        });
        // n 番目のグローバルシーンの属性（ファイル属性の受け継ぎと畳みの後）
        for (scene, attrs) in scenes.zip(file.global_scene_attrs()) {
            let declared = aliases.resolve(&scene.name).unwrap_or(&scene.name);
            let (_, counter) = registry.register_global(declared, scene_attrs_of(&attrs));
            positions.push((path, &scene.name, scene.span));
            for (local, local_counter) in scene.local_scenes.iter().zip(local_counters(scene)) {
                if let Some(name) = &local.name {
                    positions.push((path, name, local.span));
                    registry.register_local(
                        name,
                        declared,
                        counter,
                        local_counter,
                        // ローカルはファイル属性も親の属性も受け継がない（宣言行の中だけ畳む）
                        scene_attrs_of(&merge_attrs(&[], &local.attrs)),
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
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![])));
    // 実行時（pasta_lua の check_scene_orders）と同じ関数で判定する（6.1・3.3）
    for dup in table.duplicate_orders() {
        let (first_path, _, first_span) = positions[dup.scenes[0].0];
        for id in &dup.scenes[1..] {
            let (path, name, span) = positions[id.0];
            findings.push(finding(
                FindingKind::SceneOrderDuplicate,
                path,
                span,
                msg::scene_order_duplicate(
                    name,
                    dup.order,
                    first_path,
                    first_span.start_line as u32,
                ),
            ));
        }
    }
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

/// 構文木の属性 → 倉の項目（pasta_lua の context.rs に同じ写しがある）
pub(crate) fn scene_attrs_of(attrs: &[Attr]) -> Vec<SceneAttr> {
    attrs
        .iter()
        .map(|a| {
            let key = a.key.clone();
            match &a.body {
                AttrBody::Decl(v) => SceneAttr::Decl {
                    key,
                    value: scene_attr_value(v),
                },
                AttrBody::Cond { op, value } => SceneAttr::Cond {
                    key,
                    // 正規形の文字列 6 つが pasta_dsl と pasta_core の CmpOp の約束
                    op: CmpOp::from_normalized(op.as_str())
                        .expect("pasta_dsl と pasta_core の CmpOp の正規形は一致する"),
                    value: scene_attr_value(value),
                },
                AttrBody::Flag => SceneAttr::Flag { key },
            }
        })
        .collect()
}

/// 値の型の規則（要件 4.5）: 小数は整数の値なら整数、引用の有無は区別しない
pub(crate) fn scene_attr_value(v: &AttrValue) -> SceneAttrValue {
    match v {
        AttrValue::Integer(i) => SceneAttrValue::Integer(*i),
        AttrValue::Float(f) => SceneAttrValue::from_f64(*f),
        AttrValue::String(s) | AttrValue::AttrString(s) => SceneAttrValue::String(s.clone()),
    }
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
