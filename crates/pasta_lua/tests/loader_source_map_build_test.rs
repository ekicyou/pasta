//! Task 4.3 — ローダのマップ構築・集約（統合）の振る舞いテスト。
//!
//! 仕様参照:
//! - requirements.md **1.1**（トランスパイルでソースマップ生成が有効なとき、すべての
//!   構文要素から生成された `.lua` 行に由来 `.pasta` 位置を記録する）
//! - requirements.md **3.1**（有効時は構築したソースマップをメモリ内に保持し、
//!   ディスク中間ファイルを必須としない）
//! - requirements.md **7.1**（ソースマップ生成が無効なとき、生成 `.lua` の出力内容を
//!   従来と完全に一致させる＝バイト不変）
//! - design.md "Flow 1"（per-`.pasta` に `MapBuilderSink` 装着 → `transpile` →
//!   `normalize_output_with_shift` の `LineShift` で `finish` → チャンク名で集約）
//! - design.md File Structure Plan `loader/mod.rs` 166 / `loader/cache.rs` 167
//!
//! # このテストが表明すること
//!
//! 1. **マルチチャンク集約（1.1, 3.1）**: 複数 `.pasta` をローダのマップ構築経路
//!    （[`PastaLoader::build_source_map`]）へ通すと、複数チャンクのマップが構築・集約
//!    され、各チャンクで `.lua`→`.pasta` 前方解決・`.pasta`→`.lua` 逆引き解決が
//!    正しく行える。チャンク名キーは [`CacheManager::source_to_cache_path`] 由来で、
//!    正規化突合に乗る（task 1.1 の確定戦略）。
//! 2. **デバッグ無効＝バイト不変（7.1）**: 記録シンクを装着した構築経路の生成 `.lua`
//!    バイトは、シンク無し（本番ローダの `transpile`）の生成 `.lua` バイトと完全一致
//!    する（シンク装着が出力を変えない＝無効時バイト不変の根拠）。
//! 3. **無効時はマップ非構築**: ローダはデバッグ無効時にこの構築経路を呼ばないため、
//!    `SourceMap` の割当そのものが起きない（本テストは「呼ばなければ何も構築されない」
//!    ことを、構築経路が opt-in な独立関数であることで担保する）。

use std::path::PathBuf;

use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::debug::source_map::canonicalize_chunk_name;
use pasta_lua::loader::{CacheManager, PastaLoader};

/// 代表 `.pasta` A（シーン 1 本・トーク複数行）。複数の異なる `.pasta` 行が生成
/// `.lua` 行へ写像されることを確認するための最小入力。
const PASTA_A: &str = "\
＊あいさつ
  さくら：「おはよう！」
  さくら：「げんき？」
";

/// 代表 `.pasta` B（別ファイル・別シーン）。マルチチャンク集約を確認するための 2 本目。
const PASTA_B: &str = "\
＊おやすみ
  さくら：「おやすみなさい。」
";

/// テスト用 base_dir 配下に `.pasta` を書き出し、絶対パスを返す。
fn write_pasta(base_dir: &std::path::Path, rel: &str, src: &str) -> PathBuf {
    let path = base_dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir dic");
    std::fs::write(&path, src).expect("write .pasta");
    path
}

/// 1.1 / 3.1: 複数 `.pasta` から複数チャンクのマップが構築・集約され、各チャンクで
/// 双方向解決ができる。
#[test]
fn loader_builds_and_aggregates_multi_chunk_source_map() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();

    let file_a = write_pasta(&base_dir, "dic/baseware/a.pasta", PASTA_A);
    let file_b = write_pasta(&base_dir, "dic/baseware/b.pasta", PASTA_B);

    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");

    // --- ローダのマップ構築経路（デバッグ有効時のみローダが呼ぶ） ---
    // sidecar=false: メモリ既定経路のみ（このテストはサイドカーを検証しない）。
    let source_map =
        PastaLoader::build_source_map(&[file_a.clone(), file_b.clone()], &cache_manager, false);

    // チャンク名キー = source_to_cache_path 由来（task 1.1 の確定戦略）。
    let chunk_a = cache_manager
        .source_to_cache_path(&file_a)
        .to_string_lossy()
        .to_string();
    let chunk_b = cache_manager
        .source_to_cache_path(&file_b)
        .to_string_lossy()
        .to_string();
    // チャンク名は正規化後に区別される（2 つの別チャンク）。
    assert_ne!(
        canonicalize_chunk_name(&chunk_a),
        canonicalize_chunk_name(&chunk_b),
        "2 つの `.pasta` は別チャンクへ写像される"
    );

    let file_a_key = file_a.to_string_lossy().to_string();
    let file_b_key = file_b.to_string_lossy().to_string();

    // --- (a) 複数チャンクが集約されている: 各チャンクから `.pasta` へ前方解決できる ---
    // チャンク A: 少なくとも 1 つの `.lua` 行が `.pasta` 位置へ解決でき、その位置の
    // ファイルは file_a を指す。
    let resolved_a: Vec<_> = (1u32..=200)
        .filter_map(|lua_line| {
            source_map
                .resolve_lua_to_pasta(&chunk_a, lua_line)
                .map(|pos| (lua_line, pos.clone()))
        })
        .collect();
    assert!(
        !resolved_a.is_empty(),
        "チャンク A は少なくとも 1 つの `.lua`→`.pasta` 対応を持つ（1.1）"
    );
    for (_lua_line, pos) in &resolved_a {
        assert_eq!(
            canonicalize_chunk_name(&pos.file),
            canonicalize_chunk_name(&file_a_key),
            "チャンク A の解決先 `.pasta` ファイルは file_a"
        );
    }

    // チャンク B も同様（別ファイルを指す）。
    let resolved_b: Vec<_> = (1u32..=200)
        .filter_map(|lua_line| {
            source_map
                .resolve_lua_to_pasta(&chunk_b, lua_line)
                .map(|pos| (lua_line, pos.clone()))
        })
        .collect();
    assert!(
        !resolved_b.is_empty(),
        "チャンク B は少なくとも 1 つの `.lua`→`.pasta` 対応を持つ（1.1）"
    );
    for (_lua_line, pos) in &resolved_b {
        assert_eq!(
            canonicalize_chunk_name(&pos.file),
            canonicalize_chunk_name(&file_b_key),
            "チャンク B の解決先 `.pasta` ファイルは file_b"
        );
    }

    // --- (b) 逆引き解決: `.pasta` 行 → 正しいチャンクの `.lua` 行群 ---
    // チャンク A のある解決済み `.pasta` 行で逆引きし、その `.lua` 行群が前方解決と
    // 整合する（同一チャンクへ戻る）。
    let (lua_line_a, pos_a) = &resolved_a[0];
    let back_a = source_map.resolve_pasta_to_lua(&pos_a.file, pos_a.line);
    assert!(
        back_a
            .iter()
            .any(|(chunk, lua)| canonicalize_chunk_name(chunk)
                == canonicalize_chunk_name(&chunk_a)
                && lua == lua_line_a),
        "`.pasta` 行 {} の逆引きはチャンク A の `.lua` 行 {} を含む（3.1 双方向）: {:?}",
        pos_a.line,
        lua_line_a,
        back_a
    );

    // B 側の `.pasta` 行を A のファイルキーで逆引きしても出てこない（ファイル分離）。
    let (lua_line_b, pos_b) = &resolved_b[0];
    let back_b = source_map.resolve_pasta_to_lua(&pos_b.file, pos_b.line);
    assert!(
        back_b
            .iter()
            .any(|(chunk, lua)| canonicalize_chunk_name(chunk)
                == canonicalize_chunk_name(&chunk_b)
                && lua == lua_line_b),
        "`.pasta` 行 {} の逆引きはチャンク B の `.lua` 行 {} を含む: {:?}",
        pos_b.line,
        lua_line_b,
        back_b
    );
    // クロスチャンク混線がない: B の `.pasta` ファイル行の逆引きに A のチャンクは出ない。
    assert!(
        !back_b
            .iter()
            .any(|(chunk, _)| canonicalize_chunk_name(chunk)
                == canonicalize_chunk_name(&chunk_a)),
        "B のファイル行の逆引きに A のチャンクが混入してはならない: {:?}",
        back_b
    );
}

/// 7.1: デバッグ無効＝バイト不変。記録シンクを装着した構築経路の生成 `.lua` バイトは、
/// シンク無し（本番ローダの `transpile`）の生成 `.lua` バイトと完全一致する。
///
/// これは「シンク装着が出力を 1 バイトも変えない」ことの直接検証であり、無効時に
/// ローダがシンクを装着しなければ生成 `.lua` は従来と完全一致する（7.1）ことの根拠。
#[test]
fn sink_attachment_is_byte_invariant_against_sinkless_transpile() {
    for src in [PASTA_A, PASTA_B] {
        let pasta_file = parse_str(src, "dic/x.pasta").expect("parse");
        let transpiler = LuaTranspiler::default();

        // シンク無し（本番ローダの transpile＝debug 無効経路）。
        let mut out_sinkless = Vec::new();
        transpiler
            .transpile(&pasta_file, &mut out_sinkless)
            .expect("transpile sinkless");

        // シンク有り（debug 有効の構築経路）。
        let mut sink = pasta_lua::debug::source_map::MapBuilderSink::new(
            "dic/x.pasta".to_string(),
            "chunk-x".to_string(),
        );
        let mut out_with_sink = Vec::new();
        transpiler
            .transpile_with_source_map(&pasta_file, &mut out_with_sink, Some(&mut sink))
            .expect("transpile with sink");

        assert_eq!(
            out_sinkless, out_with_sink,
            "7.1: シンク装着は生成 `.lua` を 1 バイトも変えない（無効時バイト不変の根拠）"
        );
    }
}

/// 無効時はマップ非構築（割当ゼロ）: ローダはデバッグ無効時に
/// [`PastaLoader::build_source_map`] を呼ばないため `SourceMap` は構築されない。
///
/// 本テストは「空の入力で構築経路を呼んでもチャンクが 0 で、解決は常に空/None になる」
/// ことを以て、構築が完全に opt-in（呼ばなければ何も起きない）であることを表明する。
#[test]
fn no_map_built_when_no_files_passed() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();
    let cache_manager = CacheManager::new(base_dir, "profile/pasta/cache/lua");

    // デバッグ無効経路の模擬: 構築経路へ何も渡さない（ローダが呼ばない状況の極限）。
    let source_map = PastaLoader::build_source_map(&[], &cache_manager, false);

    // チャンクが無いので前方解決は常に None、逆引きは常に空。
    assert!(source_map.resolve_lua_to_pasta("any-chunk", 1).is_none());
    assert!(
        source_map
            .resolve_pasta_to_lua("any.pasta", 1)
            .is_empty()
    );
}

/// 3.2（task 6.1）: サイドカー **有効時**、ローダのマップ構築経路は各生成 `.lua` の隣に
/// `<lua>.map` を出力する。出力されたサイドカーを再読込すると、メモリ内集約 `SourceMap`
/// の前方写像と一致する（往復同一性・3.2 完了条件）。
#[test]
fn loader_writes_sidecar_when_enabled_and_round_trips() {
    use pasta_lua::debug::source_map::{read_sidecar, sidecar_path_for_lua};

    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();

    let file_a = write_pasta(&base_dir, "dic/baseware/a.pasta", PASTA_A);
    let file_b = write_pasta(&base_dir, "dic/baseware/b.pasta", PASTA_B);
    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
    // サイドカーは生成 `.lua` の隣（cache 配下 pasta/scene/...）へ書くので親が要る。
    cache_manager
        .prepare_cache_dir()
        .expect("prepare cache dir");

    // sidecar=true でマップ構築（ローダのデバッグ有効＋サイドカー有効経路）。
    let source_map =
        PastaLoader::build_source_map(&[file_a.clone(), file_b.clone()], &cache_manager, true);

    for file in [&file_a, &file_b] {
        let lua_path = cache_manager.source_to_cache_path(file);
        let chunk = lua_path.to_string_lossy().to_string();

        // (1) サイドカーが生成 `.lua` の隣に存在する。
        let sidecar = sidecar_path_for_lua(&lua_path);
        assert!(
            sidecar.exists(),
            "サイドカー有効時、{} が生成されること（3.2）",
            sidecar.display()
        );

        // (2) 再読込した写像がメモリ内集約マップの当該チャンクと一致する（往復同一性）。
        let reread = read_sidecar(&lua_path).expect("read_sidecar");
        // メモリ側で当該チャンクが持つ全 `.lua`→`.pasta` 対応を列挙し、サイドカー側と
        // 同一であることを表明する（昇順走査で網羅）。
        let mut compared = 0usize;
        for lua_line in 1u32..=500 {
            let mem = source_map.resolve_lua_to_pasta(&chunk, lua_line).cloned();
            let disk = reread.pasta_for_lua(lua_line).cloned();
            assert_eq!(
                mem.as_ref().map(|p| p.line),
                disk.as_ref().map(|p| p.line),
                "再読込サイドカーの `.lua`{} → `.pasta` 行はメモリ写像と一致（3.2 往復）",
                lua_line
            );
            if mem.is_some() {
                compared += 1;
            }
        }
        assert!(
            compared > 0,
            "当該チャンクは少なくとも 1 対応を持つ（サイドカーが空でない）"
        );
    }
}

/// 3.2 ゲーティング: サイドカー **無効時**（既定）、ローダのマップ構築経路は `.lua.map`
/// を一切出力しない（メモリ既定経路のみ・3.1）。
#[test]
fn loader_writes_no_sidecar_when_disabled() {
    use pasta_lua::debug::source_map::sidecar_path_for_lua;

    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();

    let file_a = write_pasta(&base_dir, "dic/baseware/a.pasta", PASTA_A);
    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
    cache_manager
        .prepare_cache_dir()
        .expect("prepare cache dir");

    // sidecar=false（既定）: メモリだけ。`.lua.map` は書かれない。
    let _source_map =
        PastaLoader::build_source_map(std::slice::from_ref(&file_a), &cache_manager, false);

    let lua_path = cache_manager.source_to_cache_path(&file_a);
    let sidecar = sidecar_path_for_lua(&lua_path);
    assert!(
        !sidecar.exists(),
        "サイドカー無効時は `.lua.map` を出力しない（3.1・既定経路不変）: {}",
        sidecar.display()
    );
}
