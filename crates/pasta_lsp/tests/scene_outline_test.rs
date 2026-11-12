//! シーンの並び（`AnalysisResult.scenes`）の取り出しの試験
//!
//! 並びを作る関数は crate の外から呼べないので、公開の `analyze` の結果を通して確かめる。

use pasta_lsp::analysis::{AnalysisEngine, SceneKind, SceneNode};
use serde_json::json;

fn scenes(source: &str) -> Vec<SceneNode> {
    let result = AnalysisEngine::analyze(source);
    assert!(
        result.diagnostics.is_empty(),
        "試験の文書は誤りなく解ける: {:?}",
        result.diagnostics
    );
    result.scenes
}

/// 名前・種類・子だけの木（位置を比べない）
fn names(nodes: &[SceneNode]) -> Vec<(String, SceneKind, Vec<String>)> {
    nodes
        .iter()
        .map(|n| {
            (
                n.name.clone(),
                n.kind,
                n.children.iter().map(|c| c.name.clone()).collect(),
            )
        })
        .collect()
}

/// 要件の例の文書が、設計の API Contract の JSON の例と同じ木になる（1.1・1.2・1.4・1.9）
#[test]
fn requirement_example_matches_design_json() {
    let source = "＊挨拶\n  さくら：やあ\n  ・朝\n    さくら：おはよう\n＊\n  さくら：またね\n";
    let pos = |l: u32, c: u32| json!({ "line": l, "character": c });
    let rng = |sl, sc, el, ec| json!({ "start": pos(sl, sc), "end": pos(el, ec) });
    assert_eq!(
        serde_json::to_value(scenes(source)).unwrap(),
        json!([
            {
                "name": "挨拶",
                "kind": "global",
                "heading": rng(0, 0, 0, 3),
                "range": rng(0, 0, 3, 12),
                "children": [
                    {
                        "name": "朝",
                        "kind": "local",
                        "heading": rng(2, 2, 2, 4),
                        "range": rng(2, 0, 3, 12),
                        "children": []
                    }
                ]
            },
            {
                "name": "挨拶",
                "kind": "global",
                "heading": rng(4, 0, 4, 1),
                "range": rng(4, 0, 6, 0),
                "children": []
            }
        ])
    );
}

/// 全角と半角の印が同じ木になる（1.3）
#[test]
fn fullwidth_and_halfwidth_markers_give_same_tree() {
    let full = "＊挨拶\n  さくら：やあ\n  ・朝\n    さくら：おはよう\n";
    let half = "*挨拶\n  さくら：やあ\n  -朝\n    さくら：おはよう\n";
    let full_tree = scenes(full);
    assert_eq!(
        names(&full_tree),
        vec![("挨拶".into(), SceneKind::Global, vec!["朝".into()])]
    );
    assert_eq!(full_tree, scenes(half));
}

/// 名前の無い最初の部分は子にならない（1.5・3.2）
#[test]
fn unnamed_start_part_is_not_an_item() {
    let tree = scenes("＊挨拶\n  さくら：やあ\n  えっと：うん\n");
    assert_eq!(
        names(&tree),
        vec![("挨拶".into(), SceneKind::Global, vec![])]
    );
}

/// コードブロックの中の `＊`・`・` の行とコメントの行は見出しにならない（1.6）
#[test]
fn code_block_and_comment_lines_are_not_headings() {
    let source = "\
＃＊コメントのシーン
＊挨拶
  さくら：やあ
  ・朝
    さくら：おはよう
```lua
  ・偽のローカル
  ＊偽のグローバル
```
＃  ・コメントのローカル
";
    assert_eq!(
        names(&scenes(source)),
        vec![("挨拶".into(), SceneKind::Global, vec!["朝".into()])]
    );
}

/// 空の文書と、アクター辞書・単語定義だけの文書は空の並び（1.10・2.8）
#[test]
fn documents_without_scenes_give_empty_list() {
    assert!(scenes("").is_empty());
    assert!(scenes("％さくら\n　＠通常：\\s[0]\n").is_empty());
    assert!(scenes("＠挨拶：こんにちは、やあ\n").is_empty());
}

/// アクター辞書・単語定義はシーンの間にあっても項目にならない（2.8）
#[test]
fn actor_and_word_blocks_between_scenes_are_not_items() {
    let source = "＊挨拶\n  さくら：やあ\n％さくら\n　＠通常：\\s[0]\n＠花：ばら\n＊別れ\n  さくら：またね\n";
    assert_eq!(
        names(&scenes(source)),
        vec![
            ("挨拶".into(), SceneKind::Global, vec![]),
            ("別れ".into(), SceneKind::Global, vec![]),
        ]
    );
}

/// サロゲートペアと全角の空白を含む見出しの列が UTF-16 で合う（1.9）
#[test]
fn heading_columns_are_utf16() {
    let source = "＊𠮷野\n　さくら：やあ\n　　・𠮷の朝\n　　さくら：おはよう\n";
    let tree = scenes(source);
    // `＊𠮷野` = 1 + 2 + 1
    assert_eq!(tree[0].heading.start.character, 0);
    assert_eq!(tree[0].heading.end.character, 4);
    let local = &tree[0].children[0];
    assert_eq!(local.name, "𠮷の朝");
    assert_eq!(local.heading.start.line, 2);
    // 全角の空白 2 つの後ろから、`・𠮷の朝` = 1 + 2 + 1 + 1
    assert_eq!(local.heading.start.character, 2);
    assert_eq!(local.heading.end.character, 7);
    // `\r` は数えない
    let crlf = scenes("＊𠮷野\r\n　さくら：やあ\r\n");
    assert_eq!(crlf[0].heading.end.character, 4);
}
