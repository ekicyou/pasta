//! [`resolve_and_kick`] / [`uri_to_pasta_path`] の単体テスト（task 3.1・
//! requirements 2.1-2.6/7.1/8.1/8.3）。
//!
//! 観測対象:
//! 1. 正規化済みパスの一致: 同一ファイルを指す別形式 uri（パーセントエンコード・
//!    区切り違い）が同一シーンへ解決する（`std::path::absolute` 正規化の固定）。
//! 2. global 確定 → 取次呼出（`scene == "会話1"`・parent None）。
//! 3. local 確定 → 取次呼出（`scene == ":会話1:挨拶_1"`・parent Some）。
//! 4. 未検出 → 取次しない・`ResolveOutcome::NotFound`。

use std::sync::{Arc, Mutex};

use super::*;
use crate::debug::kick::{KickRequest, KickSink};
use crate::debug::source_map::{SceneIdentityIndex, SourceMap};

/// 取次呼出を記録するモック [`KickSink`]。`(sink, captured)` を返す。
fn mock_sink() -> (KickSink, Arc<Mutex<Vec<String>>>) {
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let cap = captured.clone();
    let sink: KickSink = Arc::new(move |req: KickRequest| {
        cap.lock().unwrap().push(req.scene);
    });
    (sink, captured)
}

/// テスト用 `SourceMap`（索引充填済み）を構築する。
///
/// `file` の global 会話1（行 10..=40・parent None）と、その内側 local 挨拶_1
/// （行 20..=30・parent Some(会話1)）を投入する。
fn map_with_index(file: &str) -> SourceMap {
    let mut b = SceneIdentityIndex::builder();
    b.add_scene(file, "会話1", None, 10, 40, 0);
    b.add_scene(file, "挨拶_1", Some("会話1"), 20, 30, 1);
    let index = b.finish();

    let map = SourceMap::new();
    map.set_scene_index(index)
        .expect("write-once index set must succeed");
    map
}

#[test]
fn global_confirmed_kicks_with_plain_scene_id() {
    // global 本体行（local 範囲外）→ scene == "会話1"（parent None）。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);
    let (sink, captured) = mock_sink();

    let outcome = resolve_and_kick(&map, &sink, "file:///C:/work/dic/talk.pasta", 15);

    assert_eq!(outcome, ResolveOutcome::Resolved("会話1".to_string()));
    assert_eq!(captured.lock().unwrap().as_slice(), &["会話1".to_string()]);
}

#[test]
fn local_confirmed_kicks_with_composite_scene() {
    // local 範囲内（最内 local 優先）→ scene == ":会話1:挨拶_1"（parent Some）。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);
    let (sink, captured) = mock_sink();

    let outcome = resolve_and_kick(&map, &sink, "file:///C:/work/dic/talk.pasta", 25);

    assert_eq!(
        outcome,
        ResolveOutcome::Resolved(":会話1:挨拶_1".to_string())
    );
    assert_eq!(
        captured.lock().unwrap().as_slice(),
        &[":会話1:挨拶_1".to_string()]
    );
}

#[test]
fn not_found_does_not_kick() {
    // 最終シーン終端より後ろ（下方に有効シーンなし）→ 未検出・sink 不呼出。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);
    let (sink, captured) = mock_sink();

    let outcome = resolve_and_kick(&map, &sink, "file:///C:/work/dic/talk.pasta", 100);

    assert_eq!(outcome, ResolveOutcome::NotFound);
    assert!(
        captured.lock().unwrap().is_empty(),
        "未検出では取次してはならない"
    );
}

#[test]
fn differently_formatted_equivalent_uris_resolve_to_same_scene() {
    // 同一ファイルを指す別形式 uri（パーセントエンコード・区切り違い）が、
    // std::path::absolute 正規化を経て同一シーンへ解決する（正規化の固定）。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);

    // 形式 A: スラッシュ・素のドライブパス（uri スキームなし）。
    let (sink_a, cap_a) = mock_sink();
    let out_a = resolve_and_kick(&map, &sink_a, "C:/work/dic/talk.pasta", 25);

    // 形式 B: file:// + パーセントエンコード（`%20` ではなくここでは大小・スキーム差）。
    let (sink_b, cap_b) = mock_sink();
    let out_b = resolve_and_kick(&map, &sink_b, "file:///C:/work/dic/talk.pasta", 25);

    assert_eq!(out_a, out_b, "別形式 uri は同一シーンへ解決する");
    assert_eq!(
        out_a,
        ResolveOutcome::Resolved(":会話1:挨拶_1".to_string())
    );
    assert_eq!(*cap_a.lock().unwrap(), *cap_b.lock().unwrap());
}

#[test]
fn uri_to_pasta_path_strips_scheme_and_decodes() {
    // file:// スキーム除去・パーセントデコード・先頭ドライブ補正の固定。
    let p = uri_to_pasta_path("file:///C:/work/my%20dic/talk.pasta");
    // 空白が復元され、先頭の余分な `/` が落ちている（絶対パスとして c:/... 始まり）。
    assert!(
        p.contains("my dic"),
        "%20 が空白へデコードされる: {p}"
    );
    assert!(
        !p.starts_with("/C:") && !p.starts_with("/c:"),
        "Windows ドライブの先頭 `/` が補正される: {p}"
    );
}
