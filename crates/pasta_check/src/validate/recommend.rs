//! ＞おすすめ項目 の引数と、メニューのシーンの本体の台詞を検査する段（10.3〜10.6）。

use pasta_core::{SceneAliasTable, SceneRegistry};
use pasta_dsl::PastaFile;
use pasta_dsl::parser::{Arg, CallScene, CallTarget, Expr, FileItem, LocalSceneItem, Span};

use super::scenes::SceneModel;
use super::{FileText, Finding, FindingKind, col_after_indent, msg};

/// 項目と台詞の注意を返す。`texts` は字下げから始まる行の列を求めるのに使う
pub(crate) fn check(
    model: &SceneModel,
    aliases: &SceneAliasTable,
    parsed: &[(String, PastaFile)],
    texts: &[(String, FileText)],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let declared: Vec<String> = model
        .globals
        .iter()
        .map(|g| SceneRegistry::sanitize_name(&g.declared))
        .collect();
    // model.globals は Scenes が構文木の順に積んだものなので、同じ順で並ぶ（references.rs と同じ）
    let scenes = parsed.iter().flat_map(|(path, file)| {
        file.items.iter().filter_map(move |item| match item {
            FileItem::GlobalSceneScope(scene) => Some((path, scene)),
            _ => None,
        })
    });
    for ((path, scene), global) in scenes.zip(&model.globals) {
        for item in scene.local_scenes.iter().flat_map(|l| &l.items) {
            if let LocalSceneItem::CallScene(call) = item {
                check_item(&mut findings, path, call, aliases, &declared);
            }
        }

        if !is_menu_scene(&global.declared) {
            continue;
        }
        // 名前の無い最初の範囲だけ。名前つきのローカルシーン・呼んだ先は見ない
        let Some(body) = scene.local_scenes.first().filter(|l| l.name.is_none()) else {
            continue;
        };
        for item in &body.items {
            let mut span = match item {
                LocalSceneItem::ActionLine(line) => line.span,
                LocalSceneItem::ContinueAction(line) => line.span,
                _ => continue,
            };
            // アクション行・継続行の span は字下げから始まるので、選択肢と同じく中身の列にする
            span.start_col = col_after_indent(texts, path, span);
            findings.push(finding(
                FindingKind::RecommendMenuTalk,
                path,
                span,
                msg::recommend_menu_talk(&global.written),
            ));
        }
    }
    findings
}

fn check_item(
    findings: &mut Vec<Finding>,
    path: &str,
    call: &CallScene,
    aliases: &SceneAliasTable,
    declared: &[String],
) {
    if !matches!(&call.target, CallTarget::Static(key) if key == "おすすめ項目" || key == "recommend_item")
    {
        return;
    }
    let args = call.args.as_ref().map_or(&[][..], |a| &a.items[..]);
    if !(2..=4).contains(&args.len()) {
        findings.push(finding(
            FindingKind::RecommendItemInvalid,
            path,
            call.span,
            msg::recommend_item_arg_count(args.len()),
        ));
    }
    // 文字列だけの式でなければ実行時まで分からないので見ない（10.5）。
    // 名前つきの引数もキーを捨てて位置で渡る（expr_gen の generate_args_string）
    let (Some(Arg::Positional(expr)) | Some(Arg::Keyword { value: expr, .. })) = args.get(2) else {
        return;
    };
    let Some(name) = string_literal(expr).filter(|n| !n.is_empty()) else {
        return;
    };
    // recommend.lua は別名を置き換えて完全一致で引く（前方一致にしない）
    let key = SceneRegistry::sanitize_name(aliases.resolve(name).unwrap_or(name));
    if !declared.contains(&key) {
        findings.push(finding(
            FindingKind::RecommendItemInvalid,
            path,
            call.span,
            msg::recommend_item_scene_missing(name),
        ));
    }
}

/// メニューのリソースの ID（`sakura.recommendsites`・`kero.recommendsites`・`sakura.portalsites`・`char<数字>.recommendsites`）
fn is_menu_scene(declared: &str) -> bool {
    match declared {
        "sakura.recommendsites" | "kero.recommendsites" | "sakura.portalsites" => true,
        _ => declared
            .strip_prefix("char")
            .and_then(|s| s.strip_suffix(".recommendsites"))
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())),
    }
}

/// 文字列リテラルだけの式（括弧で囲んだものも）
fn string_literal(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::String(s) => Some(s),
        Expr::Paren(inner) => string_literal(inner),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_scene_ids() {
        assert!(is_menu_scene("sakura.recommendsites"));
        assert!(is_menu_scene("kero.recommendsites"));
        assert!(is_menu_scene("sakura.portalsites"));
        assert!(is_menu_scene("char2.recommendsites"));
        assert!(is_menu_scene("char10.recommendsites"));
        assert!(!is_menu_scene("char.recommendsites"));
        assert!(!is_menu_scene("char2x.recommendsites"));
        assert!(!is_menu_scene("kero.portalsites"));
        assert!(!is_menu_scene("おすすめメニュー"));
    }
}
