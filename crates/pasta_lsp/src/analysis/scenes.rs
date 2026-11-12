//! 構文木からシーンの並び（グローバルシーン ＞ ローカルシーン の 2 階層）を作る。
//!
//! 位置は着色・診断と同じ 0 始まりの行・UTF-16 の列。

use pasta_dsl::parser::ast::FileItem;

use super::text_utils::get_line_text;
use super::token_types::{utf8_len_to_utf16, utf8_offset_to_utf16};
use super::{Position, Range};

/// シーンの並びの 1 項目（エディタの種類に寄らない素の木）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SceneNode {
    /// シーンの名前（続きのグローバルシーンは受け継いだ名前）
    pub name: String,
    pub kind: SceneKind,
    /// 見出しの行（印から行の終わりまで・1 行）
    pub heading: Range,
    /// シーン全体（見出しの行の頭から、範囲の最後の行の終わりまで）
    pub range: Range,
    /// ローカルシーン（グローバルシーンだけが持つ。ローカルシーンは常に空）
    pub children: Vec<SceneNode>,
}

/// シーンの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SceneKind {
    Global,
    Local,
}

/// 構文木の項目から、シーンの並びを文書の順に作る
///
/// 範囲は pasta.dll と同じく「次の同じ階層か上の階層の見出しの前の行まで、無ければ
/// 親（グローバルは文書）の最後の行まで」。「次の見出し」は並びに出た見出しだけで決める。
pub(super) fn collect_scenes(items: &[FileItem], source: &str) -> Vec<SceneNode> {
    // 0 始まりの最後の行。改行で終わる文書は、最後の改行の後ろの空の行
    let last_line = source.matches('\n').count() as u32;
    let globals: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            FileItem::GlobalSceneScope(scene) if scene.span.is_valid() => Some(scene),
            _ => None,
        })
        .collect();
    globals
        .iter()
        .enumerate()
        .map(|(i, scene)| {
            let line = heading_line(scene.span.start_line);
            let end = globals.get(i + 1).map_or(last_line, |next| {
                heading_line(next.span.start_line).saturating_sub(1)
            });
            let locals: Vec<_> = scene
                .local_scenes
                .iter()
                .filter(|local| local.span.is_valid())
                .filter_map(|local| {
                    Some((local.name.as_deref()?, heading_line(local.span.start_line)))
                })
                .collect();
            let children = locals
                .iter()
                .enumerate()
                .map(|(j, &(name, local_line))| {
                    let local_end = locals
                        .get(j + 1)
                        .map_or(end, |&(_, next)| next.saturating_sub(1));
                    node(
                        name,
                        SceneKind::Local,
                        local_line,
                        local_end,
                        source,
                        vec![],
                    )
                })
                .collect();
            node(&scene.name, SceneKind::Global, line, end, source, children)
        })
        .collect()
}

/// 1 始まりの span の行を 0 始まりにする
fn heading_line(start_line: usize) -> u32 {
    start_line.saturating_sub(1) as u32
}

fn node(
    name: &str,
    kind: SceneKind,
    line: u32,
    end_line: u32,
    source: &str,
    children: Vec<SceneNode>,
) -> SceneNode {
    // 壊れた span でも範囲が見出しより前で終わらない（panic もしない）
    let end_line = end_line.max(line);
    let text = get_line_text(source, line as usize + 1);
    let marker = text.len() - text.trim_start().len();
    let at = |line, character| Position { line, character };
    SceneNode {
        name: name.to_string(),
        kind,
        heading: Range {
            start: at(line, utf8_offset_to_utf16(text, marker)),
            end: at(line, utf8_len_to_utf16(text)),
        },
        range: Range {
            start: at(line, 0),
            end: at(
                end_line,
                utf8_len_to_utf16(get_line_text(source, end_line as usize + 1)),
            ),
        },
        children,
    }
}
