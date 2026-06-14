//! Task 5.3 — `.pasta` ブレークポイント翻訳と最近接調整
//! ([`translate_pasta_breakpoints`]・design "BpTranslator" 511-528・Flow 2
//! 215-236・requirements 4.1 / 4.2 / 4.3 / 8.2).
//!
//! `.pasta` 行 BP を `resolve_pasta_to_lua` で `.lua` 実行座標群へ翻訳して登録
//! （4.1・複数 `.lua` 行は全登録 8.2）、対応なしは
//! `nearest_pasta_line_with_mapping` で後続最近接へ調整して `verified`＋調整後
//! 行を返す（4.3）。登録した `.pasta` BP が **生フック座標**で `should_pause`
//! を発火することも end-to-end で検証する（5.1 review のキー再整合・4.2）。

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::debug::{SharedSourceMode, SourceMode};
use crate::debug::breakpoints::BreakpointSet;
use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
use crate::debug::types::SourceRef;

use super::{SourceMapWiring, is_pasta_source, translate_pasta_breakpoints};

/// design "BpTranslator" 514: `.pasta` source は翻訳経路、`.lua` source は直接
/// 登録経路。拡張子で判別する（大小無視）。
#[test]
fn is_pasta_source_detects_pasta_extension() {
    assert!(is_pasta_source("C:/proj/scene.pasta"));
    assert!(is_pasta_source(r"C:\proj\scene.PASTA"), "拡張子は大小無視");
    assert!(is_pasta_source("scene.pasta"));
    // `.lua` source（フック源 `@` 付きを含む）は翻訳しない → 直接登録経路。
    assert!(!is_pasta_source("@e2e_scenario"));
    assert!(!is_pasta_source("C:/proj/cache/scene.lua"));
    assert!(!is_pasta_source(r"@C:\proj\cache\scene.lua"));
}

/// 集約 `SourceMap` を、`.pasta` ファイル → [(chunk, lua_line, pasta_line)] の
/// 列から構築する小ヘルパ。各 (chunk, lua_line) を `pasta_line` へ対応づける。
fn map_from(file: &str, entries: &[(&str, u32, u32)]) -> SourceMap {
    let mut per_chunk: BTreeMap<String, BTreeMap<u32, PastaPos>> = BTreeMap::new();
    for &(chunk, lua_line, pasta_line) in entries {
        per_chunk.entry(chunk.to_string()).or_default().insert(
            lua_line,
            PastaPos {
                file: file.to_string(),
                line: pasta_line,
            },
        );
    }
    let mut sm = SourceMap::new();
    for (chunk, forward) in per_chunk {
        sm.insert_chunk(chunk, file.to_string(), ChunkSourceMap::from_forward(forward));
    }
    sm
}

/// Pasta-active な wiring を構築する。
fn pasta_wiring(map: SourceMap) -> SourceMapWiring {
    SourceMapWiring {
        source_map: Some(Arc::new(map)),
        source_mode: SharedSourceMode::new(SourceMode::Pasta),
    }
}

/// 4.1 / 8.2 / 4.2: `.pasta` 行 → 対応 `.lua` 行群を全登録し、`verified`＋元行を
/// 返す。さらに登録された各 `.lua` 実行座標が **生フック source**（`@`・`\`）で
/// `should_pause` を発火する（canonicalize 再整合の end-to-end 証明・4.2）。
#[test]
fn pasta_line_registers_all_lua_lines_and_fires_should_pause() {
    // `.pasta` 行 7 → `.lua` 12, 13（同一 chunk・1→多展開・8.2）。
    let file = "C:/proj/scene.pasta";
    let map = map_from(
        file,
        &[
            ("C:/proj/cache/scene.lua", 12, 7),
            ("C:/proj/cache/scene.lua", 13, 7),
        ],
    );
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    let resolved =
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[7]);

    // 応答: 1 件・verified・元の `.pasta` 行 7（4.1）。
    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].verified, "対応ありは verified (4.1)");
    assert_eq!(resolved[0].line, 7, "verified 行は元の `.pasta` 行");
    assert_eq!(resolved[0].source, SourceRef::new(file));

    // 4.2 end-to-end: 生フック座標（`@`・`\`・大小違い）で両 `.lua` 行が発火する。
    // 登録 chunk は `resolve_pasta_to_lua` の正規化済み chunk だが、
    // `should_pause` が両側を canonicalize するため一致する。
    assert!(
        set.should_pause(r"@C:\proj\cache\scene.lua", 12),
        "`.pasta` 行 7 の `.lua` 行 12 が生フック座標で発火する (4.2/8.2)"
    );
    assert!(
        set.should_pause(r"@C:\proj\cache\scene.lua", 13),
        "`.pasta` 行 7 の `.lua` 行 13 が生フック座標で発火する (4.2/8.2)"
    );
    // 未登録 `.lua` 行は発火しない。
    assert!(!set.should_pause(r"@C:\proj\cache\scene.lua", 11));
}

/// 4.3 最近接調整: 対応の無い `.pasta` 行は後続最近接の対応行へ調整され、その
/// `.lua` 座標で登録、応答は `verified`＋**調整後**行。
#[test]
fn unmapped_pasta_line_adjusts_to_nearest_subsequent() {
    // 対応 `.pasta` 行 = {3, 7}。`.pasta` 行 4（対応なし）→ 最近接 7 へ調整。
    let file = "C:/proj/scene.pasta";
    let map = map_from(
        file,
        &[
            ("C:/proj/cache/scene.lua", 10, 3),
            ("C:/proj/cache/scene.lua", 20, 7),
        ],
    );
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    let resolved =
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[4]);

    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].verified, "調整後は verified (4.3)");
    assert_eq!(
        resolved[0].line, 7,
        "対応なし行 4 は後続最近接の対応行 7 へ調整される (4.3)"
    );
    // 調整先 `.pasta` 行 7 の `.lua` 行 20 が登録・発火する。
    assert!(
        set.should_pause(r"@C:\proj\cache\scene.lua", 20),
        "調整後の `.pasta` 行 7 の `.lua` 座標で停止する (4.3)"
    );
    // 元の対応行 3 の `.lua` 10 は（行 4 のリクエストでは）登録されない。
    assert!(!set.should_pause(r"@C:\proj\cache\scene.lua", 10));
}

/// 4.3: 後続に対応行が一切無い `.pasta` 行は **誤マッピングせず** unverified を
/// 返す（最近接が存在しない場合のみ）。
#[test]
fn no_subsequent_mapping_returns_unverified() {
    let file = "C:/proj/scene.pasta";
    // 対応 `.pasta` 行 = {3}。行 5 以降に対応なし。
    let map = map_from(file, &[("C:/proj/cache/scene.lua", 10, 3)]);
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    let resolved =
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[5]);

    assert_eq!(resolved.len(), 1);
    assert!(
        !resolved[0].verified,
        "後続最近接が無い場合は unverified（誤マッピング禁止・4.3）"
    );
    assert_eq!(resolved[0].line, 5, "unverified は元の行を保持");
    // 何も登録されない。
    assert!(!set.should_pause(r"@C:\proj\cache\scene.lua", 10));
}

/// 4.1 / 8.2: 複数 `.pasta` 行を 1 回の setBreakpoints で要求した場合、全行分の
/// `.lua` 実行座標が **同一 present source** で蓄積登録され、行ごとに後勝ち
/// eviction しない（1 register 呼び出しに集約）。
#[test]
fn multiple_pasta_lines_all_register_without_mutual_eviction() {
    let file = "C:/proj/scene.pasta";
    // `.pasta` 3 → `.lua` 10、`.pasta` 7 → `.lua` 20, 21。
    let map = map_from(
        file,
        &[
            ("C:/proj/cache/scene.lua", 10, 3),
            ("C:/proj/cache/scene.lua", 20, 7),
            ("C:/proj/cache/scene.lua", 21, 7),
        ],
    );
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    let resolved =
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[3, 7]);

    assert_eq!(resolved.len(), 2);
    assert!(resolved.iter().all(|r| r.verified));
    assert_eq!(resolved[0].line, 3);
    assert_eq!(resolved[1].line, 7);

    // 全 `.lua` 座標が同時に登録されている（行ごとの register で互いを評価
    // 退避していない）。
    assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 10), "行 3 の座標");
    assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 20), "行 7 の座標 1");
    assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 21), "行 7 の座標 2");
}

/// 4.1 / 多チャンク: 同一 `.pasta` 行が複数チャンクへ展開される場合も全チャンクの
/// `.lua` 座標を登録する（design 215-236 のクロスチャンク・8.2）。
#[test]
fn pasta_line_spanning_multiple_chunks_registers_all() {
    let file = "C:/proj/scene.pasta";
    // `.pasta` 行 7 → chunk a の `.lua` 12, chunk b の `.lua` 5。
    let map = map_from(
        file,
        &[
            ("C:/proj/cache/a.lua", 12, 7),
            ("C:/proj/cache/b.lua", 5, 7),
        ],
    );
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    let resolved =
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[7]);

    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].verified);
    assert!(set.should_pause(r"@C:\proj\cache\a.lua", 12), "chunk a の座標");
    assert!(set.should_pause(r"@C:\proj\cache\b.lua", 5), "chunk b の座標");
}

/// `.pasta` present source は retain/置換キー: 同 `.pasta` の再 setBreakpoints は
/// 旧座標を置換しつつ、別 present source（別 `.pasta`/`.lua`）の BP を保持する。
/// requirements 4.4 / 8.2（present source 単位の権威的置換）。
#[test]
fn re_setting_pasta_source_replaces_only_its_own_coords() {
    let file = "C:/proj/scene.pasta";
    let map = map_from(
        file,
        &[
            ("C:/proj/cache/scene.lua", 10, 3),
            ("C:/proj/cache/scene.lua", 20, 7),
        ],
    );
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    // 行 3（→ `.lua` 10）を登録。
    translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[3]);
    assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 10));

    // 同 `.pasta` を行 7（→ `.lua` 20）で再設定。旧座標 10 は置換される。
    translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[7]);
    assert!(
        !set.should_pause(r"@C:\proj\cache\scene.lua", 10),
        "同一 present source の旧座標は権威的に置換される"
    );
    assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 20), "新座標が登録される");
}

/// 4.4 / 8.2: 空行リスト（DAP の「この source の BP を全消去」）での再設定は
/// 当該 present source の既存座標を権威的に除去し、空の resolved を返す。
/// [`BreakpointSet::register`] の置換セマンティクスが空 entries でも成立する
/// ことを翻訳経路越しに固定する。
#[test]
fn re_setting_pasta_source_with_empty_lines_clears_its_coords() {
    let file = "C:/proj/scene.pasta";
    let map = map_from(file, &[("C:/proj/cache/scene.lua", 10, 3)]);
    let wiring = pasta_wiring(map);
    let set = BreakpointSet::new();

    // 行 3（→ `.lua` 10）を登録して発火を確認。
    let resolved = translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[3]);
    assert_eq!(resolved.len(), 1);
    assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 10));

    // 同 `.pasta` を空行リストで再設定: resolved は空、旧座標は全除去。
    let cleared = translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[]);
    assert!(cleared.is_empty(), "空要求の resolved は空集合");
    assert!(
        !set.should_pause(r"@C:\proj\cache\scene.lua", 10),
        "空再設定は当該 present source の全座標を権威的に除去する"
    );
}

/// [`translate_pasta_breakpoints`] 冒頭の防御分岐: `pasta_active()` が保証する
/// はずの map が万一 `None` でも `.lua` 直接登録経路
/// （[`BreakpointSet::set_breakpoints`]）へ安全劣化し、ブリッジは決して
/// パニックしない（要求行どおり verified・present source 自身が実行座標）。
#[test]
fn translate_without_map_degrades_to_direct_lua_registration() {
    let wiring = SourceMapWiring {
        source_map: None,
        source_mode: SharedSourceMode::new(SourceMode::Pasta),
    };
    let set = BreakpointSet::new();
    let src = SourceRef::new("C:/proj/scene.pasta");

    let resolved = translate_pasta_breakpoints(&set, &wiring, &src, &[4, 9]);

    // `.lua` 直接経路の形: 要求順ミラー・全 verified・原行のまま。
    assert_eq!(resolved.len(), 2);
    assert!(resolved.iter().all(|bp| bp.verified), "直接経路は全行 verified");
    assert_eq!(resolved[0].line, 4);
    assert_eq!(resolved[1].line, 9);
    // present source == chunk として直接登録される（翻訳なし）。
    assert!(set.should_pause("C:/proj/scene.pasta", 4));
    assert!(set.should_pause("C:/proj/scene.pasta", 9));
}
