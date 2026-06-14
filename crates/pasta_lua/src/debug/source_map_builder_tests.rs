//! `source_map` の **構築系**インラインテスト外出し（task 2.3・C1）。
//! `MapBuilderSink`（記録 → `finish(shift)` 補正でチャンク写像を確定）の単体仕様を
//! 集約する。
//!
//! 移動のみ（振る舞い不変）。クラスタ跨ぎ共有ヘルパー（`pos`）は
//! `source_map_test_support.rs` を `use` する。

use super::source_map_test_support::*;
use super::*;

// =======================================================================
// MapBuilderSink（task 3.3・記録→補正でチャンク写像を確定）
// =======================================================================

use crate::normalize::normalize_output_with_shift;
use pasta_dsl::parser::Span;

/// 2.1: 記録（pre-normalize 行）→ `finish(shift)` 補正で、チャンク写像が最終
/// `.lua` 行に整合する。削除行（normalize が落とした空行）に紐づく記録は除外される。
///
/// 実 `normalize_output_with_shift` で本物の `LineShift` を得る（テスト用の
/// LineShift 直接構築は不可＝`deleted` 非公開のため、実 normalize 経路を使う）。
/// 入力 `"l1\nl2\nl3\n\nend\n"` の pre-normalize 行（1 始まり）:
///   1: "l1"  -> 最終 1
///   2: "l2"  -> 最終 2
///   3: "l3"  -> 最終 3
///   4: ""    -> 削除（`end` 直前の空行）-> None
///   5: "end" -> 最終 4
#[test]
fn map_builder_sink_finish_rebases_to_final_lua_lines() {
    let input = "l1\nl2\nl3\n\nend\n";
    let (out, shift) = normalize_output_with_shift(input);
    // 前提の確認: 空行(pre 4)が削除され "end" が最終 4 行へ繰り上がる。
    assert_eq!(out, "l1\nl2\nl3\nend\n");
    assert_eq!(shift.map(1), Some(1));
    assert_eq!(shift.map(4), None); // 削除行
    assert_eq!(shift.map(5), Some(4)); // "end" が繰り上がる

    let mut sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk-a".to_string());
    // producer は pre-normalize の out_line を記録する。
    sink.record_line(1, 10); // pre 1 -> .pasta 10
    sink.record_line(2, 11); // pre 2 -> .pasta 11
    sink.record_line(3, 12); // pre 3 -> .pasta 12
    sink.record_line(4, 99); // pre 4（削除行）-> finish で除外されるべき
    sink.record_line(5, 20); // pre 5 -> .pasta 20（"end" 行）

    let map = sink.finish(&shift);

    // pre-normalize 行が最終 `.lua` 行へ rebase された。
    assert_eq!(map.pasta_for_lua(1), Some(&pos(10)));
    assert_eq!(map.pasta_for_lua(2), Some(&pos(11)));
    assert_eq!(map.pasta_for_lua(3), Some(&pos(12)));
    // pre 5 は最終 4 へ繰り上がる。
    assert_eq!(map.pasta_for_lua(4), Some(&pos(20)));
    // 削除行（pre 4）の記録は除外され、最終 .lua には現れない。
    assert!(
        map.pasta_for_lua(5).is_none(),
        "削除行に紐づく記録は最終写像に残ってはならない（2.1）"
    );
    // 削除によって生じる旧位置（pre 5 のままの行 5）は存在しない。
    assert_eq!(map.len(), 4);
}

/// 8.1: 同一 pre-normalize 行を 2 回記録すると、後勝ち（last-write-wins）で
/// 単一の決定論的 `.pasta` 位置になる。BTreeMap キー一意性が担保。
#[test]
fn map_builder_sink_same_pre_line_is_last_write_wins() {
    // 削除なし（恒等）の本物 shift を使う。
    let (out, shift) = normalize_output_with_shift("a\nb\n");
    assert_eq!(out, "a\nb\n");
    assert_eq!(shift.map(1), Some(1));
    assert_eq!(shift.map(2), Some(2));

    let mut sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());
    sink.record_line(2, 5); // 先の記録
    sink.record_line(2, 8); // 同一 pre-line を再記録 -> 後勝ち

    let map = sink.finish(&shift);
    // 最終行 2 は後勝ちの `.pasta` 8 に確定（決定論的・単一位置）。
    assert_eq!(map.pasta_for_lua(2), Some(&pos(8)));
    // 旧位置（`.pasta` 5）は残らない。
    assert_eq!(map.lua_lines_for_pasta(5), Vec::<u32>::new());
    assert_eq!(map.lua_lines_for_pasta(8), vec![2]);
}

/// 8.1: 確定結果は決定論的（同一記録列・同一 shift で繰り返し finish しても同一）。
#[test]
fn map_builder_sink_finish_is_deterministic() {
    let (_out, shift) = normalize_output_with_shift("a\nb\nc\n");

    let build = || {
        let mut sink =
            MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());
        sink.record_line(3, 7);
        sink.record_line(1, 3);
        sink.record_line(2, 5);
        let map = sink.finish(&shift);
        (
            map.pasta_for_lua(1).cloned(),
            map.pasta_for_lua(2).cloned(),
            map.pasta_for_lua(3).cloned(),
        )
    };

    assert_eq!(build(), build());
    assert_eq!(
        build(),
        (Some(pos(3)), Some(pos(5)), Some(pos(7)))
    );
}

/// `.pasta` 行は `span.start_line` を **直接**採用する（research.md D-3）。trait 既定の
/// `record(lua_line, span)` が `record_line(lua_line, span.start_line)` へ委譲する
/// ことを、`finish` 後の最終写像で表明する（byte 走査廃止）。
#[test]
fn map_builder_sink_record_uses_span_start_line() {
    let (_out, shift) = normalize_output_with_shift("a\nb\n");
    let mut sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());

    // Span::new(start_line, start_col, end_line, end_col, start_byte, end_byte)
    // start_line=42 を採用すべき（start_byte=9999 のバイト走査は使わない）。
    let span = Span::new(42, 1, 42, 9, 9999, 10001);
    sink.record(1, span); // pre 1 -> .pasta start_line(42)

    let map = sink.finish(&shift);
    assert_eq!(
        map.pasta_for_lua(1),
        Some(&pos(42)),
        "record は span.start_line を直接 .pasta 行として採用する（D-3）"
    );
}

/// `chunk_name` アクセサがコンストラクタ引数を保持する（3.4 集約が参照する識別子）。
#[test]
fn map_builder_sink_retains_chunk_name() {
    let sink = MapBuilderSink::new("dict.pasta".to_string(), "my-chunk".to_string());
    assert_eq!(sink.chunk_name(), "my-chunk");
}

/// 記録ゼロ件の `MapBuilderSink::finish` は空の `ChunkSourceMap` を返す
/// （producer が一行も record しなかったチャンクでも安全に確定できる）。
#[test]
fn map_builder_sink_finish_with_no_records_yields_empty_map() {
    let (_out, shift) = normalize_output_with_shift("a\nb\n");
    let sink = MapBuilderSink::new("dict.pasta".to_string(), "chunk".to_string());
    let map = sink.finish(&shift);
    assert!(map.is_empty());
    assert_eq!(map.pasta_for_lua(1), None);
}
