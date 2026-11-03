//! [`resolve_position`] の単体テスト（scene-play-event task 2.1・debug-mcp-runtime 4.2）。
//!
//! 位置はパスだけを受ける。相対はゴーストのフォルダーに結び、絶対（UNC を含む）は
//! そのまま使う。`file:///…` や `%41` は URI として復号せず、パスの文字のまま扱う。

use super::*;
use crate::debug::source_map::{SceneIdentityIndex, SourceMap};

/// テスト用 `SourceMap`（索引充填済み）を構築する。
///
/// `file` の global 会話_1（行 10..=40・parent None）と、その内側 local 挨拶_1
/// （行 20..=30・parent Some(会話_1)）を投入する。
fn map_with_index(file: &str) -> SourceMap {
    let mut b = SceneIdentityIndex::builder();
    b.add_scene(file, "会話_1", None, 10, 40, 0);
    b.add_scene(file, "挨拶_1", Some("会話_1"), 20, 30, 1);
    let index = b.finish();

    let map = SourceMap::new();
    map.set_scene_index(index)
        .expect("write-once index set must succeed");
    map
}

// ===========================================================================
// scene-play-event task 2.1: `resolve_position`（requirements 2.2-2.7/5.3）。
// ===========================================================================

/// `base`（ghost/master 相当の絶対パス）配下の `dic/boot.pasta` に索引を張る。
fn boot_fixture() -> (std::path::PathBuf, SourceMap) {
    let base = std::path::absolute("ghost_fixture/ghost/master").unwrap();
    let file = base.join("dic/boot.pasta");
    (base, map_with_index(&file.to_string_lossy()))
}

#[test]
fn resolve_position_joins_relative_path_to_base_dir() {
    let (base, map) = boot_fixture();
    let want = Some(("会話_1".to_string(), "__start__".to_string()));
    assert_eq!(resolve_position(&map, &base, "dic/boot.pasta", 15), want);
    assert_eq!(resolve_position(&map, &base, r"dic\boot.pasta", 15), want);
}

#[test]
fn resolve_position_uses_absolute_path_as_is() {
    let (base, map) = boot_fixture();
    let abs = base.join("dic/boot.pasta").to_string_lossy().into_owned();
    let other_base = std::path::absolute("elsewhere").unwrap();
    assert_eq!(
        resolve_position(&map, &other_base, &abs, 25),
        Some(("会話_1".to_string(), "挨拶_1".to_string()))
    );
}

#[test]
fn resolve_position_returns_none_when_not_found() {
    let (base, map) = boot_fixture();
    // 索引に無いファイル・シーンの無い行
    assert_eq!(resolve_position(&map, &base, "dic/other.pasta", 15), None);
    assert_eq!(resolve_position(&map, &base, "dic/boot.pasta", 100), None);
    // 索引の無いマップ
    assert_eq!(
        resolve_position(&SourceMap::new(), &base, "dic/boot.pasta", 15),
        None
    );
}

fn local() -> Option<(String, String)> {
    Some(("会話_1".to_string(), "挨拶_1".to_string()))
}

#[cfg(windows)]
#[test]
fn resolve_position_uses_unc_path_as_is() {
    let unc = r"\\server\share\dic\a.pasta";
    let map = map_with_index(unc);
    let base = std::path::absolute("elsewhere").unwrap();
    assert_eq!(resolve_position(&map, &base, unc, 25), local());
}

/// `file:///…` は URI として剥がさない（パスの文字のまま。索引の絶対パスには当たらない）。
#[test]
fn resolve_position_does_not_strip_file_uri() {
    let (base, map) = boot_fixture();
    let abs = base
        .join("dic/boot.pasta")
        .to_string_lossy()
        .replace('\\', "/");
    let uri = format!("file:///{}", abs.trim_start_matches('/'));
    assert_eq!(resolve_position(&map, &base, &uri, 25), None);
}

/// `%41` は復号しない（`dic/%41.pasta` は `dic/%41.pasta` のまま引ける）。
#[test]
fn resolve_position_keeps_percent_literal() {
    let base = std::path::absolute("ghost_fixture/ghost/master").unwrap();
    let map = map_with_index(&base.join("dic/%41.pasta").to_string_lossy());
    assert_eq!(resolve_position(&map, &base, "dic/%41.pasta", 25), local());
    assert_eq!(resolve_position(&map, &base, "dic/A.pasta", 25), None);
}

/// 実在しないパス（8.3 短縮名 `RUNNER~1` を含む）でも字句的に引ける（FS に依存しない）。
#[test]
fn resolve_position_resolves_nonexistent_path_fs_independent() {
    let base = std::path::absolute("definitely-nonexistent-9f3a/RUNNER~1/Temp").unwrap();
    let map = map_with_index(&base.join("talk.pasta").to_string_lossy());
    assert_eq!(resolve_position(&map, &base, "talk.pasta", 25), local());
}

/// 実在するパスでも `fs::canonicalize` を通さない（`\\?\` 前置のキーにならず引ける）。
#[cfg(windows)]
#[test]
fn resolve_position_does_not_canonicalize_existing_path() {
    let dir =
        std::path::absolute(std::env::temp_dir().join("pasta_resolve_position_canary")).unwrap();
    std::fs::create_dir_all(&dir).expect("temp dir 作成");
    let file = dir.join("talk.pasta");
    std::fs::write(&file, "# canary").expect("temp .pasta 作成");
    let map = map_with_index(&file.to_string_lossy());
    assert_eq!(resolve_position(&map, &dir, "talk.pasta", 25), local());
    let _ = std::fs::remove_file(&file);
}
