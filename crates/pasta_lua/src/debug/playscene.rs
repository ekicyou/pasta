//! `OnPastaSceneAt` の位置の引き（[`resolve_position`]。`@pasta_debug.scene_at` が呼ぶ）。
//!
//! 位置はパスだけを受ける（URI は受けない。`file://` も `%xx` もパスの文字のまま）。
//! 相対パスはゴーストのフォルダーに結び、[`std::path::absolute`] で絶対にする。
//! `fs::canonicalize` は使わない（CI の 8.3 短縮名 `RUNNER~1` のパスの不具合を避ける）。
//! 区切りと Windows の大小は [`SourceMap::scene_at`] の中の正規化に任せる。

use super::source_map::SourceMap;

/// 位置 `(file, line)` を登録名の組 `(大域, ローカル)` に引く（純粋・ファイルは開かない）。
///
/// - `file` が相対パスなら `base_dir`（`ghost/master/`）に結び、絶対パス（UNC を含む）は
///   そのまま使う。結んだパスを `std::path::absolute` に通す（失敗なら結んだまま）。
/// - 大域なら `(大域, "__start__")`、ローカルなら `(親の大域, ローカル)`。
/// - 見つからなければ `None`。
pub fn resolve_position(
    map: &SourceMap,
    base_dir: &std::path::Path,
    file: &str,
    line: u32,
) -> Option<(String, String)> {
    let joined = base_dir.join(file); // 絶対パスなら join はそれをそのまま返す
    let path = std::path::absolute(&joined).unwrap_or(joined);
    let identity = map.scene_at(&path.to_string_lossy(), line)?;
    Some(match identity.parent {
        Some(global) => (global, identity.scene_id),
        None => (identity.scene_id, "__start__".to_string()),
    })
}

#[cfg(test)]
#[path = "playscene_tests.rs"]
mod tests;
