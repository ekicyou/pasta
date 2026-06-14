//! `source_map` の **解決系**インラインテスト外出し（task 2.3・C1）。
//! `canonicalize_chunk_name`（正規化規則）・`ChunkSourceMap`（1 チャンク双方向写像）・
//! `SourceMap`（マルチチャンク集約と双方向解決）の単体仕様を集約する。
//!
//! 移動のみ（振る舞い不変）。クラスタ跨ぎ共有ヘルパー（`pos`/`sample_map`）は
//! `source_map_test_support.rs` を `use` する。`pos_in`/`sample_source_map` は
//! 本クラスタ専用のため局所に残す。

use super::source_map_test_support::*;
use super::*;

// =======================================================================
// canonicalize_chunk_name（task 1.1 で確定した正規化規則の単体仕様）
// =======================================================================

/// `@` 接頭辞・パス区切り混在・大小文字の正規化を、実機（task 1.1）で実測した
/// 形に基づいて固定する。フック source（`@` 付き・区切り混在）とローダ由来キー
/// （区切り混在）が **同一正規形** へ落ちることを表明する（requirements 4.2/5.1・
/// design "Source Identity" 437-440）。
#[test]
fn canonicalize_chunk_name_unifies_at_separators_and_case() {
    // 実機実測のフック source 形（`@` 付き・前置部 `/`・モジュール展開部 `\`）。
    let hook = r"@C:/base/profile/pasta/cache/lua/pasta\scene\sys.lua";
    // 実機実測のローダ由来キー形（区切り混在の絶対パス）。
    let loader = r"C:\base\profile/pasta/cache/lua\pasta/scene\sys.lua";

    let hook_canon = canonicalize_chunk_name(hook);
    let loader_canon = canonicalize_chunk_name(loader);

    // `@` は除去される。
    assert!(!hook_canon.starts_with('@'));
    // 区切りは `/` に統一される（`\` は残らない）。
    assert!(!hook_canon.contains('\\'));
    assert!(!loader_canon.contains('\\'));
    // フック source とローダ由来キーは同一正規形へ落ちる（突合可能）。
    assert_eq!(
        hook_canon, loader_canon,
        "正規化後はフック source とローダ由来キーが一致しなければならない"
    );
}

/// Windows は大小文字無視（ドライブレター/フォルダ名の大小差を吸収）。非 Windows
/// は大小を保持する。`#[cfg]` で実機規約に整合させる。
#[test]
fn canonicalize_chunk_name_case_policy_matches_platform() {
    let upper = canonicalize_chunk_name(r"@C:/Base/SYS.lua");
    let lower = canonicalize_chunk_name(r"@c:/base/sys.lua");
    #[cfg(windows)]
    assert_eq!(upper, lower, "Windows は大小文字無視");
    #[cfg(not(windows))]
    assert_ne!(upper, lower, "非 Windows は大小を区別する");
}

// =======================================================================
// ChunkSourceMap（task 3.2・1 チャンクの双方向行写像）
// =======================================================================

/// 2.2: 対応を持つ最終 `.lua` 行は、由来 `.pasta` 位置を一意に解決できる。
#[test]
fn chunk_source_map_resolves_mapped_lua_line() {
    let map = sample_map();
    assert_eq!(map.pasta_for_lua(10), Some(&pos(3)));
    assert_eq!(map.pasta_for_lua(20), Some(&pos(9)));
    // 同一 `.pasta` 行 7 へ集約される複数 `.lua` 行も各々正しく解決する。
    assert_eq!(map.pasta_for_lua(12), Some(&pos(7)));
    assert_eq!(map.pasta_for_lua(15), Some(&pos(7)));
}

/// 1.2 / 2.3: 対応を持たない（補助/挿入行＝gap）最終 `.lua` 行は「対応なし」
/// （`None`）を返し、誤った `.pasta` 位置へ対応づけない。
#[test]
fn chunk_source_map_unmapped_lua_line_returns_none() {
    let map = sample_map();
    // gap（記録されていない `.lua` 行）。
    assert_eq!(map.pasta_for_lua(11), None);
    assert_eq!(map.pasta_for_lua(14), None);
    // 範囲外の行も「対応なし」。
    assert_eq!(map.pasta_for_lua(1), None);
    assert_eq!(map.pasta_for_lua(999), None);
}

/// 8.2: 単一 `.pasta` 行が複数 `.lua` 行へ展開される場合、逆引きはそれら全ての
/// `.lua` 行を返す。さらに 8.3: 返り値は `.lua` 行の昇順かつ決定的順序。
#[test]
fn chunk_source_map_reverse_returns_all_lua_lines_ascending() {
    let map = sample_map();
    // `.pasta` 行 7 へは `.lua` 12/13/15 が展開される（昇順）。
    assert_eq!(map.lua_lines_for_pasta(7), vec![12, 13, 15]);
    // 単一対応の `.pasta` 行も Vec で返る。
    assert_eq!(map.lua_lines_for_pasta(3), vec![10]);
    assert_eq!(map.lua_lines_for_pasta(9), vec![20]);
    // 対応を持たない `.pasta` 行は空 Vec。
    assert_eq!(map.lua_lines_for_pasta(100), Vec::<u32>::new());
}

/// 8.1: forward は最終 `.lua` 行をキーとし、1 `.lua` 行 → 高々 1 `.pasta` 位置。
/// 同一キーへの後勝ち（last-write-wins）を `from_forward` 経由で確認する
/// （BTreeMap キー一意性が不変条件を担保）。
#[test]
fn chunk_source_map_one_lua_line_maps_to_at_most_one_pasta() {
    let mut forward = BTreeMap::new();
    forward.insert(10u32, pos(3));
    // 同一 `.lua` 行への再挿入は上書き（last-write-wins）。
    forward.insert(10u32, pos(8));
    let map = ChunkSourceMap::from_forward(forward);
    assert_eq!(map.pasta_for_lua(10), Some(&pos(8)));
    // 逆引きでも後勝ちのみが見える（行 3 へは対応なし）。
    assert_eq!(map.lua_lines_for_pasta(3), Vec::<u32>::new());
    assert_eq!(map.lua_lines_for_pasta(8), vec![10]);
}

/// 8.3: 逆引きの提示順序は決定的（同一入力で繰り返し呼んでも同一 Vec）。
#[test]
fn chunk_source_map_reverse_is_deterministic_across_calls() {
    let map = sample_map();
    let first = map.lua_lines_for_pasta(7);
    let second = map.lua_lines_for_pasta(7);
    let third = map.lua_lines_for_pasta(7);
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!(first, vec![12, 13, 15]);
}

// =======================================================================
// SourceMap（task 3.4・マルチチャンク集約と双方向解決（正規化キー））
// =======================================================================

/// 指定 `.pasta` ファイルの位置を作る小ヘルパ。
fn pos_in(file: &str, line: u32) -> PastaPos {
    PastaPos {
        file: file.to_string(),
        line,
    }
}

/// 2 チャンク（異なるチャンク名・異なる `.pasta` ファイル）を持つ集約 `SourceMap`
/// を構築する。
///
/// - chunk "a"（`.pasta` ファイル `C:/proj/scene/a.pasta`）:
///   最終 `.lua` 10→`.pasta` 3, 12→`.pasta` 7, 13→`.pasta` 7, 20→`.pasta` 9
/// - chunk "b"（`.pasta` ファイル `C:/proj/scene/b.pasta`）:
///   最終 `.lua` 5→`.pasta` 7（chunk a と同じ `.pasta` 行番号だが別ファイル）
///
/// さらに、**同一 `.pasta` ファイルが複数チャンクへ写像される**ケース（4.1 クロス
/// チャンク）を表現するため chunk "c" も同じ `C:/proj/scene/a.pasta` を使い、
/// 最終 `.lua` 4→`.pasta` 7 を持たせる。
fn sample_source_map() -> SourceMap {
    let file_a = "C:/proj/scene/a.pasta";
    let file_b = "C:/proj/scene/b.pasta";

    let mut fwd_a = BTreeMap::new();
    fwd_a.insert(10u32, pos_in(file_a, 3));
    fwd_a.insert(12u32, pos_in(file_a, 7));
    fwd_a.insert(13u32, pos_in(file_a, 7));
    fwd_a.insert(20u32, pos_in(file_a, 9));

    let mut fwd_b = BTreeMap::new();
    fwd_b.insert(5u32, pos_in(file_b, 7));

    let mut fwd_c = BTreeMap::new();
    fwd_c.insert(4u32, pos_in(file_a, 7));

    let mut sm = SourceMap::new();
    // チャンク名は呼び出し側があえて非正規形（`@`・`\`・大小混在）で渡す。
    sm.insert_chunk(
        r"@C:\proj\cache\A.lua".to_string(),
        file_a.to_string(),
        ChunkSourceMap::from_forward(fwd_a),
    );
    sm.insert_chunk(
        r"@C:\proj\cache\b.lua".to_string(),
        file_b.to_string(),
        ChunkSourceMap::from_forward(fwd_b),
    );
    sm.insert_chunk(
        r"@C:\proj\cache\c.lua".to_string(),
        file_a.to_string(),
        ChunkSourceMap::from_forward(fwd_c),
    );
    sm
}

/// 正規化キーで chunk を引いて期待される解決後 `.pasta` を返す共通アサート。
///
/// 3.3 / Source Identity: `resolve_lua_to_pasta` の `chunk` 引数を `@` 付き・区切り
/// 混在・大小違いで渡しても、格納時の正規形と一致して解決できることを表明する。
#[test]
fn source_map_resolve_lua_to_pasta_matches_normalized_chunk_key() {
    let sm = sample_source_map();
    let file_a = "C:/proj/scene/a.pasta";

    // 格納時とは異なる等価形（前方スラッシュ・大小違い・`@` 無し）でも一致する。
    assert_eq!(
        sm.resolve_lua_to_pasta("C:/proj/cache/a.lua", 10),
        Some(&pos_in(file_a, 3))
    );
    // 別の等価形（`@` 付き・別大小）でも一致。
    #[cfg(windows)]
    assert_eq!(
        sm.resolve_lua_to_pasta(r"@C:\PROJ\CACHE\A.LUA", 20),
        Some(&pos_in(file_a, 9))
    );
    // 1 `.pasta` 行 7 へ集約される複数 `.lua` 行も各々解決できる。
    assert_eq!(
        sm.resolve_lua_to_pasta("c:/proj/cache/a.lua", 12),
        Some(&pos_in(file_a, 7))
    );
}

/// 整合性エラー（design 610/617）: フック source が `SourceMap` キーに無い場合は
/// `None`（誤った `.pasta` 対応づけを禁止・2.3）。対応の無い `.lua` 行も `None`。
#[test]
fn source_map_resolve_lua_to_pasta_chunk_or_line_miss_returns_none() {
    let sm = sample_source_map();
    // チャンク名不一致 -> None（.lua フォールバック）。
    assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/unknown.lua", 10), None);
    // 既知チャンクだが対応の無い `.lua` 行 -> None。
    assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/a.lua", 11), None);
    assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/a.lua", 999), None);
}

/// 4.1 / 3.3: `.pasta` 行 → 全 `(chunk, lua_line)` を昇順・決定的に返す。
/// 1 `.pasta` 行が複数 `.lua` 行・複数チャンクへ展開されるケースを含む。
/// 照合は正規化された `.pasta` ファイルキーで行う（区切り・大小違いでも一致）。
#[test]
fn source_map_resolve_pasta_to_lua_returns_all_ascending() {
    let sm = sample_source_map();

    // `C:/proj/scene/a.pasta` の `.pasta` 行 7:
    //   - chunk a の `.lua` 12, 13
    //   - chunk c の `.lua` 4
    // 期待: `(chunk, lua_line)` を **チャンク名昇順 → `.lua` 行昇順**で決定的に
    // （design 435・8.3）。正規化キーで集約されるため chunk キーは格納時の正規形
    // （Windows は小文字・`/`・`@` 無し）になる。チャンク名 "a.lua" < "c.lua"。
    let resolved = sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 7);
    #[cfg(windows)]
    let expected = vec![
        ("c:/proj/cache/a.lua".to_string(), 12u32),
        ("c:/proj/cache/a.lua".to_string(), 13u32),
        ("c:/proj/cache/c.lua".to_string(), 4u32),
    ];
    // 非 Windows では正規形は元の大小（小文字化しない）。"a.lua" < "c.lua"。
    #[cfg(not(windows))]
    let expected = vec![
        ("C:/proj/cache/a.lua".to_string(), 12u32),
        ("C:/proj/cache/a.lua".to_string(), 13u32),
        ("C:/proj/cache/c.lua".to_string(), 4u32),
    ];
    assert_eq!(resolved, expected);

    // 別ファイル `b.pasta` の同じ `.pasta` 行 7 は chunk b の `.lua` 5 のみ
    // （ファイルキーで分離されている）。区切り違いの query でも一致。
    let resolved_b = sm.resolve_pasta_to_lua(r"C:\proj\scene\b.pasta", 7);
    #[cfg(windows)]
    assert_eq!(resolved_b, vec![("c:/proj/cache/b.lua".to_string(), 5u32)]);
    #[cfg(not(windows))]
    assert_eq!(resolved_b, vec![("C:/proj/cache/b.lua".to_string(), 5u32)]);

    // 対応の無い `.pasta` 行は空 Vec。
    assert!(sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 100).is_empty());
    // 未知ファイルも空 Vec。
    assert!(sm.resolve_pasta_to_lua("C:/proj/scene/zzz.pasta", 7).is_empty());
}

/// 4.3 最近接調整: `from_line` に対応が無ければ、それ以上で対応を持つ最初の
/// `.pasta` 行を返す。`from_line` 自身が対応を持てば `from_line`。対応が
/// `from_line` 以降に無ければ `None`。照合は正規化ファイルキー。
#[test]
fn source_map_nearest_pasta_line_with_mapping() {
    let sm = sample_source_map();
    // a.pasta の対応 `.pasta` 行は {3, 7, 9}。
    // from_line=3（自身が対応を持つ）-> 3。
    assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 3), Some(3));
    // from_line=4（対応なし）-> 次の対応行 7。
    assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 4), Some(7));
    // from_line=8（対応なし）-> 次の対応行 9。
    assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 8), Some(9));
    // from_line=1（最小対応行 3 より前）-> 3。
    assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 1), Some(3));
    // from_line=10（最大対応行 9 より後）-> None。
    assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 10), None);
    // 正規化キー（区切り・大小違い）でも一致。
    #[cfg(windows)]
    assert_eq!(sm.nearest_pasta_line_with_mapping(r"C:\PROJ\Scene\A.pasta", 4), Some(7));
    // 未知ファイル -> None。
    assert_eq!(sm.nearest_pasta_line_with_mapping("C:/proj/scene/zzz.pasta", 1), None);
}

/// 8.3: 双方向解決の提示順序は決定的（同一入力で繰り返し呼んでも同一）。
#[test]
fn source_map_resolution_is_deterministic_across_calls() {
    let sm = sample_source_map();
    let first = sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 7);
    let second = sm.resolve_pasta_to_lua("C:/proj/scene/a.pasta", 7);
    assert_eq!(first, second);
    assert_eq!(
        sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 4),
        sm.nearest_pasta_line_with_mapping("C:/proj/scene/a.pasta", 4)
    );
}

// =======================================================================
// 追補テスト（review-improvement-loop cell 3.30・G1 テスト網羅）
// =======================================================================

/// 空の `ChunkSourceMap`（`new`/`Default`）は件数 0・空判定 true で、前方/逆引き
/// とも「対応なし」を返す（2.3: 誤対応づけをしない最小ケース）。
#[test]
fn chunk_source_map_empty_has_no_mappings() {
    let empty = ChunkSourceMap::new();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(empty.pasta_for_lua(1), None);
    assert_eq!(empty.lua_lines_for_pasta(1), Vec::<u32>::new());

    // `Default` 構築も `new` と同一挙動。
    let defaulted = ChunkSourceMap::default();
    assert!(defaulted.is_empty());
    assert_eq!(defaulted.pasta_for_lua(1), None);

    // 非空マップでは len/is_empty が件数を反映する（対になる正例）。
    let map = sample_map();
    assert_eq!(map.len(), 5);
    assert!(!map.is_empty());
}

/// `canonicalize_chunk_name` は (1) `@` の無い入力では strip を素通りし、
/// (2) 正規形入力に対して **冪等**（再適用しても不変）であり、(3) `@` は
/// **先頭 1 個のみ**除去する（`strip_prefix`・途中の `@` は保持）。
/// STORE と QUERY が同一 canonicalizer を多段に通っても安全であることの根拠。
#[test]
fn canonicalize_chunk_name_is_idempotent_and_strips_only_leading_at() {
    // `@` 無し入力: 区切り・大小規則のみが効く。
    let plain = canonicalize_chunk_name(r"C:\proj\cache\sys.lua");
    assert!(!plain.contains('\\'));
    #[cfg(windows)]
    assert_eq!(plain, "c:/proj/cache/sys.lua");
    #[cfg(not(windows))]
    assert_eq!(plain, "C:/proj/cache/sys.lua");

    // 冪等性: 正規形を再正規化しても不変。
    assert_eq!(canonicalize_chunk_name(&plain), plain);
    let hook = canonicalize_chunk_name(r"@C:\proj\cache\sys.lua");
    assert_eq!(canonicalize_chunk_name(&hook), hook);

    // 先頭 `@` のみ除去（`@@x` → `@x`・途中の `@` は保持）。
    assert_eq!(canonicalize_chunk_name("@@x.lua"), "@x.lua");
    assert_eq!(canonicalize_chunk_name("a@b.lua"), "a@b.lua");

    // 空文字列・`@` 単独も panic せず決定的に処理する。
    assert_eq!(canonicalize_chunk_name(""), "");
    assert_eq!(canonicalize_chunk_name("@"), "");
}

/// 空の `SourceMap`（チャンク未登録）は全 `resolve_*` で「対応なし」を返す
/// （2.3: 登録前の query で誤対応づけ・panic をしない）。
#[test]
fn source_map_empty_resolves_to_nothing() {
    let sm = SourceMap::new();
    assert_eq!(sm.resolve_lua_to_pasta("any.lua", 1), None);
    assert!(sm.resolve_pasta_to_lua("any.pasta", 1).is_empty());
    assert_eq!(sm.nearest_pasta_line_with_mapping("any.pasta", 1), None);
}

/// 空の `ChunkSourceMap` を `insert_chunk` しても、解決は全て「対応なし」のまま
/// （ファイルキーだけが登録され、写像ゼロ件でも `range` 系が安全に動く）。
#[test]
fn source_map_insert_empty_chunk_yields_no_resolutions() {
    let mut sm = SourceMap::new();
    sm.insert_chunk(
        "@C:/proj/cache/empty.lua".to_string(),
        "C:/proj/scene/empty.pasta".to_string(),
        ChunkSourceMap::new(),
    );
    // 登録済みチャンクだが対応行ゼロ → None。
    assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/empty.lua", 1), None);
    // 逆引き・最近接も「対応なし」。
    assert!(sm.resolve_pasta_to_lua("C:/proj/scene/empty.pasta", 1).is_empty());
    assert_eq!(
        sm.nearest_pasta_line_with_mapping("C:/proj/scene/empty.pasta", 1),
        None
    );
}

/// STORE 側 `.pasta` ファイルキーの正規化: `insert_chunk` に **非正規形**（`\`
/// 区切り）の `pasta_file` を渡しても、正規形（`/` 区切り）の query で逆引き・
/// 最近接が一致する（design Validation Hook 475・STORE/QUERY 同一規則）。
/// 一方、前方解決が返す `PastaPos::file` は **格納時の生文字列**を保持する
/// （正規化はキー照合のみで、producer が記録したパス表現は不変）。
#[test]
fn source_map_canonicalizes_pasta_file_key_on_store_side() {
    let raw_file = r"C:\proj\scene\raw.pasta"; // 非正規形（backslash）
    let mut fwd = BTreeMap::new();
    fwd.insert(10u32, pos_in(raw_file, 3));

    let mut sm = SourceMap::new();
    sm.insert_chunk(
        "@C:/proj/cache/raw.lua".to_string(),
        raw_file.to_string(),
        ChunkSourceMap::from_forward(fwd),
    );

    // 正規形（`/` 区切り）query が STORE 時の非正規形キーへ一致する。
    // チャンク名の正規形は Windows では小文字・非 Windows は大小保持。
    #[cfg(windows)]
    let expected_chunk = "c:/proj/cache/raw.lua".to_string();
    #[cfg(not(windows))]
    let expected_chunk = "C:/proj/cache/raw.lua".to_string();
    assert_eq!(
        sm.resolve_pasta_to_lua("C:/proj/scene/raw.pasta", 3),
        vec![(expected_chunk, 10u32)]
    );
    assert_eq!(
        sm.nearest_pasta_line_with_mapping("C:/proj/scene/raw.pasta", 1),
        Some(3)
    );
    // 前方解決の返す PastaPos::file は格納時の生文字列（backslash のまま）。
    assert_eq!(
        sm.resolve_lua_to_pasta("C:/proj/cache/raw.lua", 10),
        Some(&pos_in(raw_file, 3))
    );
}

// =======================================================================
// 防御的ハードニング回帰（review-improvement-loop cell 3.31・G3）
// =======================================================================

/// 同一チャンク名の **再投入は置換セマンティクス**: forward（`HashMap` 上書き）
/// だけでなく **逆引き索引からも旧チャンクのエントリが除去**され、stale な
/// `(chunk, lua_line)` が残留しない（3.30 申し送りの防御的ハードニング）。
/// 本番経路（loader）は 1 チャンク 1 回投入のため正常系の挙動は不変だが、
/// `insert_chunk` は公開 API であり外部呼び出しでの再投入に備える。
#[test]
fn source_map_reinsert_same_chunk_replaces_reverse_entries() {
    let file_a = "C:/proj/scene/a.pasta";
    let file_b = "C:/proj/scene/b.pasta";

    // 掃除の精密性確認用: 別チャンク "other" も file_a へ写像を持つ。
    let mut fwd_other = BTreeMap::new();
    fwd_other.insert(50u32, pos_in(file_a, 3));

    let mut fwd_v1 = BTreeMap::new();
    fwd_v1.insert(10u32, pos_in(file_a, 3));
    fwd_v1.insert(12u32, pos_in(file_a, 7));

    let mut sm = SourceMap::new();
    sm.insert_chunk(
        "@C:/proj/cache/other.lua".to_string(),
        file_a.to_string(),
        ChunkSourceMap::from_forward(fwd_other),
    );
    sm.insert_chunk(
        "@C:/proj/cache/x.lua".to_string(),
        file_a.to_string(),
        ChunkSourceMap::from_forward(fwd_v1),
    );

    // 再トランスパイル相当の v2: 同一チャンク名・同一 .pasta だが対応行が移動。
    let mut fwd_v2 = BTreeMap::new();
    fwd_v2.insert(20u32, pos_in(file_a, 5));
    sm.insert_chunk(
        "@C:/proj/cache/x.lua".to_string(),
        file_a.to_string(),
        ChunkSourceMap::from_forward(fwd_v2),
    );

    // 正規化キー（Windows は小文字・非 Windows は大小保持）。
    #[cfg(windows)]
    let (x_key, other_key) = (
        "c:/proj/cache/x.lua".to_string(),
        "c:/proj/cache/other.lua".to_string(),
    );
    #[cfg(not(windows))]
    let (x_key, other_key) = (
        "C:/proj/cache/x.lua".to_string(),
        "C:/proj/cache/other.lua".to_string(),
    );

    // forward は v2 のみ（HashMap insert 上書き — 従来から正しい）。
    assert_eq!(sm.resolve_lua_to_pasta("C:/proj/cache/x.lua", 10), None);
    assert_eq!(
        sm.resolve_lua_to_pasta("C:/proj/cache/x.lua", 20),
        Some(&pos_in(file_a, 5))
    );
    // 逆引きも v2 のみ: v1 の旧エントリ（行 3 の (x,10)・行 7 の (x,12)）が
    // 残留しない。別チャンク "other" の行 3 エントリは温存される（掃除の精密性）。
    assert_eq!(
        sm.resolve_pasta_to_lua(file_a, 3),
        vec![(other_key.clone(), 50u32)],
        "再投入チャンクの旧エントリのみ除去・他チャンクは温存"
    );
    assert!(
        sm.resolve_pasta_to_lua(file_a, 7).is_empty(),
        "旧 v1 だけが持っていた .pasta 行 7 は対応なしへ戻る"
    );
    assert_eq!(
        sm.resolve_pasta_to_lua(file_a, 5),
        vec![(x_key.clone(), 20u32)]
    );
    // 最近接調整も stale 行 7 を返さない（{3, 5} のみが対応行）。
    assert_eq!(sm.nearest_pasta_line_with_mapping(file_a, 4), Some(5));
    assert_eq!(sm.nearest_pasta_line_with_mapping(file_a, 6), None);

    // 再投入で .pasta ファイルが移った場合: 旧ファイルキー側の residue も掃除。
    let mut fwd_v3 = BTreeMap::new();
    fwd_v3.insert(30u32, pos_in(file_b, 9));
    sm.insert_chunk(
        "@C:/proj/cache/x.lua".to_string(),
        file_b.to_string(),
        ChunkSourceMap::from_forward(fwd_v3),
    );
    assert!(
        sm.resolve_pasta_to_lua(file_a, 5).is_empty(),
        "旧ファイル a.pasta 側の v2 エントリも除去される"
    );
    assert_eq!(sm.resolve_pasta_to_lua(file_b, 9), vec![(x_key, 30u32)]);
    // 他チャンクの a.pasta エントリは引き続き温存。
    assert_eq!(sm.resolve_pasta_to_lua(file_a, 3), vec![(other_key, 50u32)]);
}
