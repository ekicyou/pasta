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

// ---------------------------------------------------------------------------
// 範囲の式（1.7・1.8・1.9）
// ---------------------------------------------------------------------------

/// `crates/pasta_lua/tests/fixtures/scene_identity_index.pasta`（`scene_identity_index_test.rs`
/// の `FIXTURE`）を写したもの。元のファイルと同じく CRLF で、改行で終わる。
const INDEX_FIXTURE: &str = "\
＃ task 2.2 統合フィクスチャ（pasta-scene-kick-from-cursor）。
＃ finalize join（runtime 実 identity × 蓄積記録）で確定する
＃ SceneIdentityIndex を観測するための最小辞書。
＃ 構成:
＃  - global「会話」#1（行 5）: 本体トーク（__start__）＋ 名前付き local「挨拶」（行 8）
＃  - global「会話」#2（行 11）: 同 base 二度目（runtime 会話2）
＊会話
　さくら：「おはよう」
　＞挨拶

　・挨拶
　　さくら：「やあ」

＊会話
　さくら：「また会話だよ」
";

/// `crates/pasta_lua/tests/fixtures/scene_alias_mixed.pasta`（`scene_identity_index_test.rs`
/// の `ALIAS_MIXED_FIXTURE`）を写したもの。CRLF で、改行で終わる。
const ALIAS_MIXED_FIXTURE: &str = "\
＃ scene-name-alias task 6.1: 別名（会話）と本名（OnTalk）が混在するファイル。
＊会話
　さくら：「会話その1」
　＞挨拶

　・挨拶
　　さくら：「会話1の挨拶」

＊OnTalk
　さくら：「本名のOnTalk」

＊会話
　さくら：「会話その3」
";

/// `crates/pasta_lua/tests/scene_identity_index_test.rs` の `TWO_FILE_A` を写したもの（LF）。
const TWO_FILE_A: &str =
    "＊会話\n　さくら：「Aの会話」\n　＞挨拶\n\n　・挨拶\n　　さくら：「Aの挨拶」\n";

fn crlf(source: &str) -> String {
    source.replace('\n', "\r\n")
}

/// 木を (名前, 見出しの行, 範囲の終わりの行) の列にする（親・子の順。行は 1 始まり）
///
/// 1 始まりにするのは、pasta.dll の表（`scene_join.rs` の `end_line`）と並べて読むため。
fn line_table(nodes: &[SceneNode]) -> Vec<(String, u32, u32)> {
    let mut out = vec![];
    for n in nodes {
        out.push((
            n.name.clone(),
            n.heading.start.line + 1,
            n.range.end.line + 1,
        ));
        out.extend(line_table(&n.children));
    }
    out
}

fn row(name: &str, heading: u32, end: u32) -> (String, u32, u32) {
    (name.into(), heading, end)
}

/// pasta.dll の表と同じ終わりの行になる（1.7・1.8）
///
/// pasta.dll は「次の同じ階層か上の階層の宣言の行 − 1、無ければファイルの終わり」。
/// ファイルの終わりは、改行で終わる文書では最後の改行の後ろの空の行（行数 + 1）。
#[test]
fn end_lines_match_pasta_dll_table() {
    // 会話#1 は 会話#2 の前の行まで、最後のローカル 挨拶 も同じ行まで、会話#2 は終わりまで
    assert_eq!(
        line_table(&scenes(&crlf(INDEX_FIXTURE))),
        vec![row("会話", 7, 13), row("挨拶", 11, 13), row("会話", 14, 16)]
    );
    assert_eq!(
        line_table(&scenes(&crlf(ALIAS_MIXED_FIXTURE))),
        vec![
            row("会話", 2, 8),
            row("挨拶", 6, 8),
            row("OnTalk", 9, 11),
            row("会話", 12, 14),
        ]
    );
    assert_eq!(
        line_table(&scenes(TWO_FILE_A)),
        vec![row("会話", 1, 7), row("挨拶", 5, 7)]
    );
}

/// ローカルは次のローカルの前の行まで（1.7）
#[test]
fn local_ends_before_next_local() {
    let source = "＊会話\n　・朝\n　　さくら：おはよう\n\n　・夜\n　　さくら：おやすみ\n＊別\n　さくら：うん\n";
    assert_eq!(
        line_table(&scenes(source)),
        vec![
            row("会話", 1, 6),
            row("朝", 2, 4),
            row("夜", 5, 6),
            row("別", 7, 9),
        ]
    );
}

/// シーンの後ろのアクター辞書・単語定義の塊は、そのシーンの範囲に入る（1.7）
#[test]
fn actor_block_after_scene_belongs_to_its_range() {
    let source = "＊挨拶\n  さくら：やあ\n  ・朝\n    さくら：おはよう\n％さくら\n　＠通常：\\s[0]\n＠花：ばら\n＊別れ\n  さくら：またね\n";
    assert_eq!(
        line_table(&scenes(source)),
        vec![row("挨拶", 1, 7), row("朝", 3, 7), row("別れ", 8, 10)]
    );
}

/// 改行で終わる・終わらない・CRLF で、終わりの位置が決まりどおり（1.7・1.9）
#[test]
fn end_position_by_line_ending() {
    let end = |source: &str| {
        let tree = scenes(source);
        let e = tree.last().unwrap().range.end;
        (e.line, e.character)
    };
    // 改行で終わる: 最後の改行の後ろの空の行の頭
    assert_eq!(end("＊挨拶\n  さくら：やあ\n"), (2, 0));
    assert_eq!(end("＊挨拶\r\n  さくら：やあ\r\n"), (2, 0));
    // 改行で終わらない: 最後の行の終わり（会話の行は改行が要るので、空白だけの行で終える）
    assert_eq!(end("＊挨拶\n  さくら：やあ\n  "), (2, 2));
    assert_eq!(end("＊挨拶\r\n  さくら：やあ\r\n  "), (2, 2));
    // 次の見出しの前の行で終わるとき、その行の `\r` は数えない
    let tree = scenes("＊挨拶\r\n  さくら：やあ\r\n＊別れ\r\n");
    assert_eq!(
        (tree[0].range.end.line, tree[0].range.end.character),
        (1, 8)
    );
    let local = &scenes("＊挨拶\r\n  ・朝\r\n    さくら：おはよう\r\n  ・夜\r\n")[0].children[0];
    // `    さくら：おはよう` = 4 + 3 + 1 + 4
    assert_eq!((local.range.end.line, local.range.end.character), (2, 12));
}

/// 属性は項目にならず、名前にも入らない
#[test]
fn attributes_are_not_items() {
    let tree = scenes("＆a：1\n＊会話＆b：2\n　＆c：3\n　さくら：やあ\n");
    assert_eq!(
        names(&tree),
        vec![("会話".into(), SceneKind::Global, vec![])]
    );
}

fn pos(p: pasta_lsp::analysis::Position) -> (u32, u32) {
    (p.line, p.character)
}

/// 見出しは範囲に含まれる・子の範囲は親の範囲に含まれる・兄弟は重ならない
fn assert_invariants(nodes: &[SceneNode], parent: Option<&SceneNode>) {
    for n in nodes {
        let (rs, re) = (pos(n.range.start), pos(n.range.end));
        assert!(
            rs <= pos(n.heading.start) && pos(n.heading.end) <= re,
            "見出しは範囲の中: {n:?}"
        );
        if let Some(p) = parent {
            assert!(
                pos(p.range.start) <= rs && re <= pos(p.range.end),
                "子は親の中: {n:?}"
            );
        }
        assert_invariants(&n.children, Some(n));
    }
    for w in nodes.windows(2) {
        assert!(
            pos(w[0].range.end) < pos(w[1].range.start),
            "兄弟は重ならない: {w:?}"
        );
    }
}

/// すべての試験の文書で、範囲の並びの決まりが成り立つ（1.7・1.8）
#[test]
fn range_invariants_hold_for_all_test_documents() {
    let docs = [
        "＊挨拶\n  さくら：やあ\n  ・朝\n    さくら：おはよう\n＊\n  さくら：またね\n".to_string(),
        "＊挨拶\n  さくら：やあ\n％さくら\n　＠通常：\\s[0]\n＠花：ばら\n＊別れ\n  さくら：またね\n"
            .to_string(),
        "＊𠮷野\n　さくら：やあ\n　　・𠮷の朝\n　　さくら：おはよう\n".to_string(),
        "＊会話\n　・朝\n　　さくら：おはよう\n\n　・夜\n　　さくら：おやすみ\n＊別\n　さくら：うん\n"
            .to_string(),
        "＊挨拶\n  さくら：やあ\n  ".to_string(),
        "＊挨拶\r\n  ・朝\r\n    さくら：おはよう\r\n  ・夜\r\n".to_string(),
        crlf(INDEX_FIXTURE),
        crlf(ALIAS_MIXED_FIXTURE),
        TWO_FILE_A.to_string(),
    ];
    for doc in &docs {
        assert_invariants(&scenes(doc), None);
    }
}

/// 真ん中のシーンに構文の誤りがある文書は、前後のシーンだけを出し、前のシーンの範囲を
/// 次に出たシーンの見出しの前の行で終える（1.11）
#[test]
fn syntax_error_scene_is_skipped_and_ranges_use_emitted_headings() {
    let source = "＊挨拶\n  さくら：やあ\n＊壊れ\n壊れた行\n＊別れ\n  さくら：またね\n";
    let result = AnalysisEngine::analyze(source);
    assert!(!result.diagnostics.is_empty(), "構文の誤りが診断に出る");
    assert_eq!(
        line_table(&result.scenes),
        vec![row("挨拶", 1, 4), row("別れ", 5, 7)]
    );
}
