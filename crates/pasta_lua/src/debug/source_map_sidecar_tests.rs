//! `source_map` の **サイドカー系**インラインテスト外出し（task 2.3・C1）。
//! `SidecarWriter`（任意ディスクサイドカー出力・往復同一性・非致命）の単体仕様を
//! 集約する。
//!
//! 移動のみ（振る舞い不変）。クラスタ跨ぎ共有ヘルパー（`pos`/`sample_map`）は
//! `source_map_test_support.rs` を `use` する。

use super::source_map_test_support::*;
use super::*;

// =======================================================================
// SidecarWriter（task 6.1・任意ディスクサイドカー出力・3.2）
// =======================================================================

use tempfile::TempDir;

/// サイドカーパスは生成 `.lua` の真隣（`<lua_path>.map`・design 483）。
#[test]
fn sidecar_path_appends_map_next_to_lua() {
    let lua = Path::new("/cache/pasta/scene/sys.lua");
    assert_eq!(
        sidecar_path_for_lua(lua),
        PathBuf::from("/cache/pasta/scene/sys.lua.map"),
        "サイドカーは生成 .lua の隣に <lua>.map として並ぶ（3.2）"
    );
}

/// 3.2 完了条件（往復同一性）: `write_sidecar` で出力 → `read_sidecar` で再読込し、
/// 再読込した行ペア写像がメモリ内 [`ChunkSourceMap`] と一致する。さらに JSON が
/// `version`＋`pasta_file`＋行ペア列を含むことを表明する（design 601-602）。
#[test]
fn write_then_read_sidecar_round_trips_to_memory_map() {
    let dir = TempDir::new().unwrap();
    let lua_path = dir.path().join("sys.lua");
    let map = sample_map(); // 10→3, 12→7, 13→7, 15→7, 20→9
    let pasta_file = "dict.pasta";

    write_sidecar(&lua_path, pasta_file, &map).expect("write_sidecar must succeed");

    // サイドカーファイルが生成 .lua の隣に存在する。
    let sidecar = sidecar_path_for_lua(&lua_path);
    assert!(sidecar.exists(), "サイドカー <lua>.map が生成されること（3.2）");

    // JSON が version + pasta_file + ペア列を含む（自己記述スキーマ・602）。
    let raw = std::fs::read_to_string(&sidecar).unwrap();
    let parsed: SidecarFile = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed.version, SIDECAR_VERSION, "version フィールドを持つ（602）");
    assert_eq!(parsed.pasta_file, pasta_file, "pasta_file フィールドを持つ（602）");
    // 行ペアは .lua 行昇順かつ決定的（8.3）。
    assert_eq!(
        parsed.pairs,
        vec![[10, 3], [12, 7], [13, 7], [15, 7], [20, 9]],
        "行ペア列は .lua 行昇順で [lua_line, pasta_line]（602/8.3）"
    );

    // 再読込した写像がメモリ内写像と一致する（往復同一性・3.2 完了条件）。
    let reread = read_sidecar(&lua_path).expect("read_sidecar must succeed");
    for lua_line in [10u32, 12, 13, 15, 20] {
        assert_eq!(
            reread.pasta_for_lua(lua_line),
            map.pasta_for_lua(lua_line),
            "再読込写像はメモリ写像と同一の lua_line→pasta_line ペアを持つ（3.2）"
        );
    }
    // gap も一致（対応なしは双方 None）。
    assert_eq!(reread.pasta_for_lua(11), None);
    assert_eq!(reread.len(), map.len());
    // 逆引きも一致（同一 .pasta 行 7 へ 12/13/15 が昇順で復元）。
    assert_eq!(reread.lua_lines_for_pasta(7), vec![12, 13, 15]);
}

/// 冪等性（design 484）: 同一マップを 2 回書くと **同一バイト列**になる
/// （決定論的出力・再トランスパイルで同一内容）。
#[test]
fn write_sidecar_is_byte_deterministic() {
    let dir = TempDir::new().unwrap();
    let lua_a = dir.path().join("a.lua");
    let lua_b = dir.path().join("b.lua");
    let map = sample_map();

    write_sidecar(&lua_a, "dict.pasta", &map).unwrap();
    write_sidecar(&lua_b, "dict.pasta", &map).unwrap();

    let bytes_a = std::fs::read(sidecar_path_for_lua(&lua_a)).unwrap();
    let bytes_b = std::fs::read(sidecar_path_for_lua(&lua_b)).unwrap();
    assert_eq!(bytes_a, bytes_b, "同一マップ → 同一バイト列（冪等・484）");

    // 同一パスへの再書き込みも同一バイト列。
    write_sidecar(&lua_a, "dict.pasta", &map).unwrap();
    let bytes_a2 = std::fs::read(sidecar_path_for_lua(&lua_a)).unwrap();
    assert_eq!(bytes_a, bytes_a2, "再書き込みも同一バイト列（冪等・484）");
}

/// 非致命（3.1 / design Error 611, 616）: 書き込み不能パスへの `write_sidecar` は
/// **panic せず** `Err` を返し、メモリ内写像は一切変更されない。呼び出し側はこの
/// `Err` を warn でログして握り潰し、メモリ既定経路を継続する想定。
#[test]
fn write_sidecar_failure_is_non_fatal_and_leaves_memory_map_intact() {
    let dir = TempDir::new().unwrap();
    // 親パスを **ファイル**として先に作る。`write_sidecar` は親を
    // `create_dir_all` で作ろうとするが、同名ファイルが既にあるため失敗する
    // （`create_dir_all` がディレクトリを作れない・決定論的に I/O エラー・OS 非依存）。
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"i am a file, not a dir").unwrap();
    // サイドカーの親（= blocker/sub）がファイル配下になり作成不能。
    let bad_lua = blocker.join("sub").join("x.lua");
    assert!(blocker.is_file(), "前提: 親パスはファイル（ディレクトリ作成不能）");

    let map = sample_map();
    let before = map.len();

    // panic せず Err を返す（catch_unwind は不要だが、Err 経路であることを表明）。
    let result = write_sidecar(&bad_lua, "dict.pasta", &map);
    assert!(
        result.is_err(),
        "書き込み不能パスは Err を返す（非致命・611）。panic/abort しない"
    );

    // メモリ写像は一切変更されていない（読み取り専用・3.1）。
    assert_eq!(map.len(), before, "メモリ写像は不変（3.1）");
    assert_eq!(map.pasta_for_lua(10), Some(&pos(3)));
    assert_eq!(map.pasta_for_lua(20), Some(&pos(9)));
    // サイドカーファイルは生成されていない。
    assert!(!sidecar_path_for_lua(&bad_lua).exists());
}

/// `read_sidecar` のエラー経路: サイドカーが存在しない → `NotFound` の `Err`、
/// JSON 不正 → `InvalidData` の `Err`。いずれも panic せず非致命（3.1/611 の
/// 「warn して継続」を呼び出し側が選べる単一の `Result` 経路）。
#[test]
fn read_sidecar_missing_or_corrupt_returns_err_without_panic() {
    let dir = TempDir::new().unwrap();
    let lua_path = dir.path().join("sys.lua");

    // (1) サイドカー不存在 → NotFound。
    let missing = read_sidecar(&lua_path);
    assert_eq!(
        missing.expect_err("不存在は Err").kind(),
        std::io::ErrorKind::NotFound
    );

    // (2) JSON 不正（壊れたサイドカー）→ InvalidData。
    std::fs::write(sidecar_path_for_lua(&lua_path), b"{ not valid json !!").unwrap();
    let corrupt = read_sidecar(&lua_path);
    assert_eq!(
        corrupt.expect_err("JSON 不正は Err").kind(),
        std::io::ErrorKind::InvalidData
    );
}

/// `SidecarFile::to_chunk` は同一 `lua_line` の重複ペアを **last-write-wins** で
/// 復元する（`forward` の `BTreeMap` キー一意性 = 8.1 の不変条件が読込側でも保たれ、
/// 手書き・破損気味のサイドカーでも 1 `.lua` 行 → 高々 1 `.pasta` 位置に確定する）。
#[test]
fn sidecar_to_chunk_duplicate_lua_line_is_last_write_wins() {
    let sidecar = SidecarFile {
        version: SIDECAR_VERSION,
        pasta_file: "dict.pasta".to_string(),
        pairs: vec![[5, 1], [5, 9], [7, 2]],
    };
    let map = sidecar.to_chunk();
    // 重複キー 5 は後勝ち（.pasta 9）で一意に確定。
    assert_eq!(map.pasta_for_lua(5), Some(&pos(9)));
    assert_eq!(map.lua_lines_for_pasta(1), Vec::<u32>::new());
    assert_eq!(map.lua_lines_for_pasta(9), vec![5]);
    assert_eq!(map.len(), 2);
}

/// 文書化テスト（現行挙動の固定）: `read_sidecar` は `version` フィールドを
/// **検証しない**（未知バージョンでも行ペアを復元する・前方互換読み）。スキーマ
/// 変更時に読み手が判別できるよう `version` を **持つ**（602）が、現行 reader は
/// 値を選別しない。将来 version ゲートを導入する場合は本テストを更新すること。
#[test]
fn read_sidecar_currently_accepts_unknown_version() {
    let dir = TempDir::new().unwrap();
    let lua_path = dir.path().join("sys.lua");
    let future = SidecarFile {
        version: SIDECAR_VERSION + 999,
        pasta_file: "dict.pasta".to_string(),
        pairs: vec![[10, 3]],
    };
    std::fs::write(
        sidecar_path_for_lua(&lua_path),
        serde_json::to_vec(&future).unwrap(),
    )
    .unwrap();

    let reread = read_sidecar(&lua_path).expect("未知 version でも現行は読める");
    assert_eq!(reread.pasta_for_lua(10), Some(&pos(3)));
}

/// `write_sidecar` は親ディレクトリ不在でも idempotent に作成して書き込む
/// （呼び出し順に依存しない堅牢性・`save_cache` 前でも成功する）。あわせて
/// 写像ゼロ件のサイドカーも往復同一（空 `pairs` → 空マップ復元）。
#[test]
fn write_sidecar_creates_parents_and_round_trips_empty_map() {
    let dir = TempDir::new().unwrap();
    // 親 `nested/deep` は未作成。
    let lua_path = dir.path().join("nested").join("deep").join("x.lua");
    let empty = ChunkSourceMap::new();

    write_sidecar(&lua_path, "dict.pasta", &empty)
        .expect("親ディレクトリを作成して書き込めること");
    assert!(sidecar_path_for_lua(&lua_path).exists());

    // 空マップの往復同一性。
    let reread = read_sidecar(&lua_path).expect("read_sidecar");
    assert!(reread.is_empty());
    let parsed: SidecarFile =
        serde_json::from_slice(&std::fs::read(sidecar_path_for_lua(&lua_path)).unwrap())
            .unwrap();
    assert_eq!(parsed.pairs, Vec::<[u32; 2]>::new());
    assert_eq!(parsed.version, SIDECAR_VERSION);
}

/// `sidecar_path_for_lua` は拡張子を **付加**する（置換しない）: `.lua` 以外・
/// 拡張子なしのパスでも一様に `<path>.map` を導出する（決定的なファイル名規則）。
#[test]
fn sidecar_path_appends_for_any_extension() {
    assert_eq!(
        sidecar_path_for_lua(Path::new("/cache/noext")),
        PathBuf::from("/cache/noext.map")
    );
    assert_eq!(
        sidecar_path_for_lua(Path::new("/cache/a.tar.lua")),
        PathBuf::from("/cache/a.tar.lua.map")
    );
}
