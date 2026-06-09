//! Task 3.1 — 多対1・ループ再訪を含む `.pasta` E2E フィクスチャの整備と検証。
//!
//! spec: `pasta-debug-break-coalesce`
//! - requirements.md **6.2**: Requirement 1/2 を実 DAP-over-TCP E2E で検証可能とし、
//!   その E2E は最低限 (a) 1つの `.pasta` 行が複数の `.lua` 行へ展開される構成、および
//!   (b) 同一 `.pasta` 行をループで再訪する構成を網羅する。
//! - design.md "File Structure Plan"（fixture note 96）/ "Testing Strategy → Integration / E2E"。
//! - tasks.md **3.1**: 観測可能な「done」=「フィクスチャがトランスパイル・ロードでき、
//!   対象 `.pasta` 行がソースマップ上で2つ以上の `.lua` 行に対応することを試験で確認できる」。
//!
//! 本タスクの範囲は **フィクスチャ＋フィクスチャ検証テスト** のみであり、continue/loop の
//! 完全な DAP E2E フロー（Tasks 3.2/3.3）は含まない。ここでは下流の 3.2/3.3 が依存する
//! ソースマップ成果物が
//!   (a) 多対1（対象 `.pasta` 行 → ≥2 `.lua` 行）
//!   (b) ループ本体の `.pasta` 行が単一 `.lua` 座標を持ち、その `.lua` 行が生成コードの
//!       `for … do`/`end` 内側にある（＝実行時にループ反復ごとに同一 `.pasta` 行を再訪する）
//! という2性質を満たすことを、ローダのマップ構築経路（3.2/3.3 と同一）と生成 `.lua` の
//! 構造検証で実証する。実 DAP 停止カウント（N 訪問 → N 停止）は Task 3.3 が実ソケットで検証する。

use std::path::{Path, PathBuf};

use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::debug::source_map::canonicalize_chunk_name;
use pasta_lua::loader::{CacheManager, PastaLoader};

/// フィクスチャを本番ローダと同じく（sink 無し）トランスパイルし、生成 `.lua` テキストを
/// 返す。loader の `build_source_map`（sink 有り）の出力はこれとバイト一致するため
/// （`sink_attachment_is_byte_invariant`）、ソースマップの行番号と整合する。
fn transpile_fixture(file: &Path) -> String {
    let pasta = parse_str(FIXTURE, &file.to_string_lossy()).expect("fixture must parse");
    let transpiler = LuaTranspiler::default();
    let mut out = Vec::new();
    transpiler
        .transpile(&pasta, &mut out)
        .expect("fixture must transpile");
    String::from_utf8(out).expect("generated lua is valid utf-8")
}

/// フィクスチャ本体（多対1展開 + ループ再訪を 1 ファイルに含む最小辞書）。
/// Tasks 3.2/3.3 はこの同一フィクスチャをロードする。
const FIXTURE: &str = include_str!("../fixtures/debug_break_coalesce.pasta");

/// 多対1（a）の対象 `.pasta` トーク行のマーカー断片。`FIXTURE` 内で一意。
const MULTI_TO_ONE_MARKER: &str = "合計は＠加算ループ()";
/// ループ再訪（b）の対象 `.pasta` 行（ループ本体）のマーカー断片。`FIXTURE` 内で一意。
const LOOP_BODY_MARKER: &str = "total = total + i";

/// `FIXTURE` 内で `needle` を含む唯一の **非コメント** 行の 1-origin 行番号を返す
/// （テスト前提の固定）。コメント行（`＃`/`#`）はマーカー文字列を解説で含み得るため除外する。
fn unique_line_of(needle: &str) -> u32 {
    let hits: Vec<u32> = FIXTURE
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            !t.starts_with('＃') && !t.starts_with('#') && l.contains(needle)
        })
        .map(|(i, _)| i as u32 + 1)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "fixture invariant: marker {needle:?} must appear on exactly one line, got {hits:?}"
    );
    hits[0]
}

/// テスト用 base_dir 配下にフィクスチャ `.pasta` を書き出し、絶対パスを返す。
fn write_fixture(base_dir: &Path) -> PathBuf {
    let path = base_dir.join("dic/test/debug_break_coalesce.pasta");
    std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir dic");
    std::fs::write(&path, FIXTURE).expect("write fixture .pasta");
    path
}

/// (a) 多対1: 対象 `.pasta` トーク行が **2つ以上** の `.lua` 行へ展開されることを
/// ローダのマップ構築経路（3.2/3.3 と同一）で確認する。これが Continue が「同じ
/// `.pasta` 行へマップする複数の `.lua` 行」を消化して当該行を抜ける挙動（R1.1）の
/// 前提＝多対1 構成の実在証拠である。
#[test]
fn fixture_target_pasta_line_maps_to_multiple_lua_lines() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();
    let file = write_fixture(&base_dir);

    // フィクスチャがトランスパイル・マップ構築できること（＝「ロードできる」観測可能 done）。
    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
    let source_map = PastaLoader::build_source_map(std::slice::from_ref(&file), &cache_manager, false);

    let chunk = cache_manager
        .source_to_cache_path(&file)
        .to_string_lossy()
        .to_string();
    let file_key = file.to_string_lossy().to_string();

    // 対象 `.pasta` 行（多対1トーク行）の逆引きが ≥2 の `.lua` 行を返す。
    let target_line = unique_line_of(MULTI_TO_ONE_MARKER);
    let lua_coords = source_map.resolve_pasta_to_lua(&file_key, target_line);
    assert!(
        lua_coords.len() >= 2,
        "6.2(a): 対象 `.pasta` 行 {target_line} は ≥2 の `.lua` 行へ展開されること \
         (multi-to-one), got {lua_coords:?}"
    );

    // 逆引きで得た全 `.lua` 行が、当該チャンクで同一 `.pasta` 行へ前方解決し戻る
    // （アンカー抑制 `anchor == cur` の前提＝同一 `.pasta` 行に等価解決する不変条件）。
    for (mapped_chunk, lua_line) in &lua_coords {
        assert_eq!(
            canonicalize_chunk_name(mapped_chunk),
            canonicalize_chunk_name(&chunk),
            "逆引き結果は当該チャンクを指す"
        );
        let back = source_map
            .resolve_lua_to_pasta(&chunk, *lua_line)
            .expect("前方解決できること");
        assert_eq!(
            back.line, target_line,
            "6.2(a): `.lua` 行 {lua_line} は対象 `.pasta` 行 {target_line} へ等価解決する \
             （多対1の各 `.lua` 行が同一 `.pasta` 行を指す）"
        );
    }
}

/// (b) ループ再訪: ループ本体の `.pasta` 行が **単一の `.lua` 実行座標** を持ち（BP を
/// 張れば 1 座標）、その `.lua` 行が生成コードの `for ... do` … `end` の **内側** にある
/// ことを確認する。すなわち実行時にこの 1 本の `.lua` 行（= 同一 `.pasta` 行）がループ
/// 反復ごとに **複数回** 通過される。これが「同一 `.pasta` 行をループで再訪する構成」
/// （R2.2 / 6.2(b)）の実在証拠である（実 DAP 停止カウントは Task 3.3 が実ソケットで検証）。
#[test]
fn fixture_loop_revisits_the_same_pasta_line() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();
    let file = write_fixture(&base_dir);

    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
    let source_map = PastaLoader::build_source_map(std::slice::from_ref(&file), &cache_manager, false);
    let chunk = cache_manager
        .source_to_cache_path(&file)
        .to_string_lossy()
        .to_string();
    let file_key = file.to_string_lossy().to_string();

    // ループ本体の `.pasta` 行は 1 つの `.lua` 実行座標へ対応する（ループ反復は同一行を再訪）。
    let loop_line = unique_line_of(LOOP_BODY_MARKER);
    let loop_coords = source_map.resolve_pasta_to_lua(&file_key, loop_line);
    assert_eq!(
        loop_coords.len(),
        1,
        "6.2(b): ループ本体 `.pasta` 行 {loop_line} は単一の `.lua` 座標を持つ \
         （ループ反復で同一行を再訪する），got {loop_coords:?}"
    );
    let (loop_chunk, loop_lua_line) = &loop_coords[0];
    assert_eq!(
        canonicalize_chunk_name(loop_chunk),
        canonicalize_chunk_name(&chunk),
        "ループ本体の逆引きは当該チャンクを指す"
    );

    // 生成 `.lua` を取得し、ループ本体の `.lua` 行が `for ... do`/`end` の内側にある
    // ことを構造的に確認する（＝この 1 本の `.lua` 行が実行時にループ反復ごとに再訪される）。
    // 生成テキストは loader のマップ構築（sink 有り）とバイト一致するため行番号が整合する。
    let generated_lua = transpile_fixture(&file);
    let lines: Vec<&str> = generated_lua.lines().collect();
    let body_idx = (*loop_lua_line as usize)
        .checked_sub(1)
        .expect("1-origin lua line");
    assert!(
        body_idx < lines.len(),
        "ループ本体 `.lua` 行 {loop_lua_line} は生成出力の範囲内"
    );

    // 本体行の手前に `for ... do`、後ろに対応する `end` がある（同一ブロック内の再訪）。
    let for_before = lines[..body_idx]
        .iter()
        .rposition(|l| l.contains("for ") && l.contains(" do"));
    assert!(
        for_before.is_some(),
        "6.2(b): ループ本体 `.lua` 行 {loop_lua_line} の手前に `for ... do` が存在する \
         （生成コード: {lines:#?}）"
    );
    let end_after = lines[body_idx + 1..]
        .iter()
        .any(|l| l.trim() == "end");
    assert!(
        end_after,
        "6.2(b): ループ本体 `.lua` 行 {loop_lua_line} の後ろにループ終端 `end` が存在する"
    );

    // 念のため、本体 `.lua` 行自体がループの累積式（再訪で繰り返し実行される副作用）であること。
    assert!(
        lines[body_idx].contains("total = total + i"),
        "6.2(b): ループ本体 `.lua` 行 {loop_lua_line} は累積式（毎反復実行）である: {:?}",
        lines[body_idx]
    );
}
