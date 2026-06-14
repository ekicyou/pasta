//! `source_map` インラインテスト分割（task 2.3・C1）で **クラスタ跨ぎ**に共有する
//! テストヘルパー。`source_map_resolve_tests.rs` / `source_map_builder_tests.rs` /
//! `source_map_sidecar_tests.rs` が `use super::source_map_test_support::*;` で参照する。
//!
//! 本番可視性は不変。ここに集約するのは test-only ヘルパーのみで `pub(super)` に留める。

use super::*;

/// テスト用の `.pasta` 位置を構築する小ヘルパ（`file` は固定）。
pub(super) fn pos(line: u32) -> PastaPos {
    PastaPos {
        file: "dict.pasta".to_string(),
        line,
    }
}

/// 既知 forward マップから `ChunkSourceMap` を構築する小ヘルパ。
///
/// マップ内容: 最終 `.lua` 行 10→`.pasta` 3, 12→`.pasta` 7, 13→`.pasta` 7,
/// 15→`.pasta` 7, 20→`.pasta` 9。`.lua` 11/14 など gap を残す。
pub(super) fn sample_map() -> ChunkSourceMap {
    let mut forward = BTreeMap::new();
    forward.insert(10u32, pos(3));
    forward.insert(12u32, pos(7));
    forward.insert(13u32, pos(7));
    forward.insert(15u32, pos(7));
    forward.insert(20u32, pos(9));
    ChunkSourceMap::from_forward(forward)
}
