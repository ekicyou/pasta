//! Task 7.4 — 後方互換・ゼロコスト最終回帰ゲート（要件 7.1 / 7.2 / 7.3）。
//!
//! 仕様参照（`.kiro/specs/pasta-source-map/`）:
//! - requirements.md **7.1**:
//!   「While ソースマップ生成が無効である, the ソースマップ生成器 shall 生成 `.lua` の
//!   出力内容を従来と完全に一致させる（バイト不変）」
//! - requirements.md **7.2**:
//!   「The デバッグ基盤 shall 先行仕様で出荷済みの Lua レベルデバッグ（生成 `.lua` 上の
//!   BP・ステップ・変数 inspect・コルーチン inspect・VSCode attach）を引き続き動作させる」
//! - requirements.md **7.3**:
//!   「When 本機能を本番化する, the プロジェクト shall 実証用の使い捨て feature gate
//!   （`pasta-source-map-slice` 等）を本番経路へ統合または除去し、暫定ハーネスを残置しない」
//! - design.md "Performance / Backward Compatibility"（643-649 行）: 二相のゼロコスト。OFF 経路は
//!   sink 非装着で `record_*` が no-op ＝出力バイト不変・ラインフック未装着・`SourceMap` 非構築。
//! - design.md "Migration Strategy" → Rollback triggers（668 行）: ゼロコスト回帰失敗（7.1）／
//!   既存 Lua デバッグ回帰（7.2）／チャンク名突合不一致。
//!
//! # このモジュールの役割（最終 GO ゲート）
//!
//! 本仕様の **最後** のタスク。1.2 の単一フィクスチャより**広い代表入力集合**に対して、
//! デバッグ／ソースマップ無効（sink 非装着）での生成 `.lua` が **バイト不変** であることを
//! 最終確認する統合ゲートである。各入力について 2 方向で OFF 経路バイトを基準と突合する:
//!
//! 1. **sink 破棄バイト等価（7.1・観測専用性の teeth）**: 本番既定の薄いラッパ `transpile`
//!    （sink 非装着）の出力と、`transpile_with_sink(Some(&mut sink))`（sink を装着して記録を
//!    収集 → 破棄）の出力が **完全にバイト一致** することを表明する。これは「ソースマップ機構を
//!    装着しても生成バイトは 1 バイトも変わらない」＝ OFF 経路ゼロコストの本質を直接突く。配線が
//!    誤って出力を変えればここが落ちる（タウトロジーではない）。
//! 2. **コミット済みスナップショット基準（7.1・真の baseline）**: 各入力の OFF 経路出力を
//!    コミット済み `.snap` に固定する。既定 `transpile` の出力バイトが従来から 1 バイトでも
//!    ずれれば、このスナップショットが落ちる（受理済みベースラインとの突合）。
//!
//! 代表入力は task 1.2 の単一フィクスチャを超え、実運用フィクスチャ（`sample.pasta`・
//! `tail_call_optimization.pasta`・`zero_cost_all_syntax.pasta`）と主要構文種別ごとの最小入力
//! （単純トーク・単語参照・変数代入・シーン呼び出し・属性・複数シーン・アクター単語定義・
//! 動的呼び出し・分岐・コードブロック）を網羅する。
//!
//! 7.2（既存 Lua レベルデバッグ継続）と 7.3（スライス撤去）は、既存テストの継続パスと
//! コンパイル時表明で確認する（下記 `r7_2_*` / `r7_3_*` 参照）。

use crate::common;

use common::e2e_helpers::transpile;
use insta::assert_snapshot;
use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::code_gen::source_map::SourceMapSink;
use std::path::PathBuf;

/// 記録を貯めるだけの sink（装着 → 記録収集 → 破棄）。ON 経路の機構を実際に踏ませて、
/// それでも出力バイトが OFF 経路と一致することを示すために使う。
#[derive(Default)]
struct DiscardingSink {
    records: Vec<(u32, u32)>,
}

impl SourceMapSink for DiscardingSink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((lua_line, pasta_line));
    }
}

/// OFF 経路（sink 非装着 = 本番既定）の生成バイト。`e2e_helpers::transpile` と同一経路。
fn transpile_off_path(source: &str) -> Vec<u8> {
    let file = parse_str(source, "final_regression.pasta").expect("parse ok");
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler
        .transpile(&file, &mut output)
        .expect("transpile (off path) ok");
    output
}

/// sink 装着経路（記録を収集 → 破棄）の生成バイト。ソースマップ機構を実際に踏む。
/// 返り値は (出力バイト, 収集された記録件数)。
fn transpile_with_discarding_sink(source: &str) -> (Vec<u8>, usize) {
    let file = parse_str(source, "final_regression.pasta").expect("parse ok");
    let transpiler = LuaTranspiler::default();
    let mut sink = DiscardingSink::default();
    let mut output = Vec::new();
    transpiler
        .transpile_with_sink(&file, &mut output, Some(&mut sink))
        .expect("transpile (sink path) ok");
    (output, sink.records.len())
}

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixtures_path().join(name))
        .unwrap_or_else(|e| panic!("Failed to read fixture {name}: {e}"))
}

/// 代表入力集合（id, ソース）。task 1.2 の単一フィクスチャを超える広い網羅。
///
/// 主要構文種別を最小入力で網羅 ＋ 実運用フィクスチャ（密な複合入力）を含める。各 id は
/// スナップショット名にも使うので安定・一意であること。
fn broad_fixtures() -> Vec<(&'static str, String)> {
    vec![
        // --- 実運用フィクスチャ（密な複合入力） ---
        ("fixture_sample", read_fixture("sample.pasta")),
        (
            "fixture_tail_call_optimization",
            read_fixture("tail_call_optimization.pasta"),
        ),
        (
            "fixture_zero_cost_all_syntax",
            read_fixture("zero_cost_all_syntax.pasta"),
        ),
        // --- 主要構文種別ごとの最小入力 ---
        (
            "kind_simple_talk",
            "＊メイン\n  さくら：「こんにちは」\n".to_string(),
        ),
        (
            "kind_word_reference",
            "＠挨拶：おはよう、こんにちは、こんばんは\n\n＊メイン\n  さくら：「＠挨拶！」\n".to_string(),
        ),
        (
            "kind_variable_assignment",
            "＊メイン\n  ＄カウンタ＝「10」\n  ＄＊グローバル＝「永続値」\n  さくら：「カウンタは＄カウンタ、グローバルは＄＊グローバル」\n".to_string(),
        ),
        (
            "kind_scene_call",
            "＊メイン\n  さくら：「サブルーチンを呼びます」\n  ＞サブ\n\n  ・サブ\n    うにゅう：「サブルーチンです」\n".to_string(),
        ),
        (
            "kind_scene_attributes",
            "＆天気：晴れ\n＆場所：東京\n\n＊メイン\n  ＆時間帯：朝\n  さくら：「今日の天気は晴れです」\n".to_string(),
        ),
        (
            "kind_multiple_scenes",
            "＊挨拶\n  さくら：「おはようございます」\n\n＊挨拶\n  さくら：「こんにちは」\n\n＊メイン\n  ＞挨拶\n".to_string(),
        ),
        (
            "kind_actor_word_definition",
            "％さくら\n  ＠一人称：私、わたし、あたし\n\n％うにゅう\n  ＠一人称：僕、ぼく\n\n＊メイン\n  さくら：「＠一人称は元気です」\n".to_string(),
        ),
        (
            "kind_dynamic_call",
            "＊メイン\n  ＄target＝「挨拶」\n  ＞＄target\n".to_string(),
        ),
        (
            "kind_choice_branch",
            "＊選択シーン\n    ＠？挨拶「あいさつする」\n".to_string(),
        ),
        (
            "kind_code_block",
            "％さくら\n　＠一人称：私\n```lua\nfunction ACTOR.時刻(act)\n    return \"朝\"\nend\n```\n\n＊メイン\n  さくら：「テスト」\n".to_string(),
        ),
    ]
}

// ============================================================================
// 7.1 — 広い代表入力でのゼロコスト・バイト不変（最終 GO ゲート）
// ============================================================================

/// **7.1 — sink 装着でも OFF 経路とバイト一致（広い網羅・観測専用性の teeth）**。
///
/// 代表入力集合の各入力について、`transpile`（sink 非装着・本番既定）の出力と
/// `transpile_with_sink(Some(&mut sink))`（ソースマップ機構を踏んで記録を収集 → 破棄）の
/// 出力が **完全にバイト一致** することを表明する。ソースマップ配線（producer の `record_*`）が
/// 誤って 1 バイトでも生成出力を変えればここが落ちる（OFF 経路ゼロコスト = 観測専用の本質）。
///
/// task 1.2（単一フィクスチャ）／record_wiring（単一フィクスチャの sink=None vs Some）を超え、
/// 実運用フィクスチャ ＋ 主要構文種別を **広く** カバーする最終ゲート。
#[test]
fn r7_1_off_path_byte_invariant_across_broad_fixtures() {
    let mut total_records = 0usize;
    for (id, source) in broad_fixtures() {
        let off = transpile_off_path(&source);
        let (with_sink, records) = transpile_with_discarding_sink(&source);

        assert!(
            !off.is_empty(),
            "7.1: fixture '{id}' must transpile to real Lua output bytes (non-vacuous)"
        );
        assert_eq!(
            off, with_sink,
            "7.1: fixture '{id}' OFF-path bytes (no sink) must be byte-identical to the \
             sink-attached transpile (source-map wiring is observe-only / zero-cost)"
        );
        total_records += records;
    }

    // 機構が実際に踏まれている（sink=Some 側が記録を収集している = vacuous でない）こと。
    // 全フィクスチャ合計で 1 件以上の記録があることで、上のバイト等価が「sink が何も
    // しないから当然一致」というタウトロジーでないことを担保する。
    assert!(
        total_records > 0,
        "7.1: the source-map sink path must actually record mappings across the broad set \
         (otherwise byte-equality would be vacuous)"
    );
}

/// **7.1 — OFF 経路の決定性（広い網羅）**。
///
/// 各入力について OFF 経路 `transpile` を 2 回独立に実行し、生成バイトが完全一致すること
/// （OFF 経路に非決定的副作用が無い ＝ バイト不変が安定）を表明する。
#[test]
fn r7_1_off_path_is_deterministic_across_broad_fixtures() {
    for (id, source) in broad_fixtures() {
        let run1 = transpile_off_path(&source);
        let run2 = transpile_off_path(&source);
        assert_eq!(
            run1, run2,
            "7.1: fixture '{id}' OFF-path transpile must be byte-for-byte deterministic"
        );
        assert!(
            !run1.is_empty(),
            "fixture '{id}' must produce real output bytes"
        );
    }
}

/// **7.1 — コミット済みスナップショット基準（真の baseline 突合）**。
///
/// 各入力の OFF 経路出力をコミット済み `.snap` に固定する。既定 `transpile` の出力バイトが
/// 従来から 1 バイトでもずれれば、このスナップショットが落ちる（受理済みベースラインとの突合
/// ＝ 非タウトロジーな baseline 比較）。`assert_snapshot!` の名前は安定 id を用いる。
#[test]
fn r7_1_off_path_matches_committed_snapshots() {
    for (id, source) in broad_fixtures() {
        let lua = transpile(&source);
        assert!(
            !lua.is_empty(),
            "7.1: fixture '{id}' must transpile to real Lua output bytes"
        );
        // コミット済み .snap が OFF 経路の受理済みベースライン。後続変更が OFF 経路の
        // バイト列を変えるとここが落ちる（7.1 最終回帰の baseline 突合）。
        assert_snapshot!(format!("r7_1_off_path__{id}"), lua);
    }
}

// ============================================================================
// 7.2 — 既存 Lua レベルデバッグの継続（named 継続ゲート）
// ============================================================================

/// **7.2 — 既存 Lua レベルデバッグ継続の named ゲート（ドキュメント＋存在表明）**。
///
/// 要件 7.2 が要求する「先行仕様で出荷済みの Lua レベルデバッグ（生成 `.lua` 上の BP・ステップ・
/// 変数 inspect・コルーチン inspect・VSCode attach）」は、本クレートの既存テストで実証済みであり、
/// 本タスクはそれらの **継続パス** を最終 GO ゲートとして確認する。具体的な実証テストは:
///
/// - `pasta_lua::debug::wiring::tests::full_dap_session_over_tcp_attach_bp_stack_vars_step_continue_terminated`
///   — TCP attach → `.lua` 行 BP → スタック → 変数 inspect → ステップ → continue → terminated の
///   E2E DAP セッション（VSCode attach・`.lua` BP・ステップ・変数 inspect を一括実証）。
/// - `pasta_lua::debug::wiring::tests::full_lua_debug_session_all_steps_all_var_types_coroutine_body`
///   — 全ステップ種別・全変数型・**コルーチン本体** inspect の E2E セッション（コルーチン inspect を実証）。
///
/// これらは `cargo test -p pasta_lua`（既定）で本ファイルと同時に走り、継続パスが最終ゲートで
/// 確認される。本テスト自体は「7.2 の責務が本クレートの常時コンパイル経路に存在し、撤去されて
/// いない」ことを表明する軽量ゲート（`enable`/`DebugConfig` がコンパイル可能＝デバッグ基盤が
/// 撤去されていない）。
#[test]
fn r7_2_existing_lua_level_debug_surface_is_present() {
    use pasta_lua::debug::{DebugConfig, enable};

    // デバッグ基盤の公開面が常時コンパイル経路に存在する（7.2 撤去されていない）。
    // OFF 既定（enabled=false）では enable は None を返す＝ゼロコスト（design 645）。
    let cfg = DebugConfig {
        enabled: false,
        ..Default::default()
    };
    let lua =
        unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()) };
    let handle = enable(&lua, &cfg, None, None).expect("enable must not error for disabled config");
    assert!(
        handle.is_none(),
        "7.2: with debug disabled (production default) enable() must return None \
         (no line hook, full JIT — zero-cost); the existing Lua-level debug machinery \
         is still present and exercised by the wiring.rs E2E tests"
    );
}

// ============================================================================
// 7.3 — 暫定スライスハーネスの撤去確認
// ============================================================================

/// **7.3 — 使い捨て feature gate `pasta-source-map-slice` の撤去確認（コンパイル時表明）**。
///
/// task 3.1 で `pasta-source-map-slice` feature / `source_map_slice` フィールド / スライスデモ
/// ハーネスは本番経路へ統合・撤去済み。本テストはその撤去を表明する:
///
/// - `pasta_lua::debug::source_map` の本番面（`SourceMap`/`ChunkSourceMap`・
///   `SourceMap::resolve_lua_to_pasta`）が **feature gate なし**で常時コンパイルされ参照可能で
///   あること（gate 撤去 = 常時コンパイル化の確認）。
/// - デッド予約 `DebugConfig.source_map_slice: bool` が存在せず `SourceMode` へ置換済みであること
///   （`source_mode` フィールドが参照可能・`source_map_slice` は不在 = コンパイルが通れば撤去確認）。
///
/// `cargo test`（既定 feature 集合）でビルド・パスすること自体が「スライス feature 無しで
/// ワークスペースが通る」ことの確認になる。
#[test]
fn r7_3_slice_harness_removed_production_paths_compile() {
    // 1) 本番ソースマップ面が feature gate 無しで常時コンパイルされている。
    //    本番集約 SourceMap を構築し、空マップで chunk-keyed resolve が None を返す
    //    （マルチチャンク本番化された面が gate 無しで生きている = スライス撤去後の常時化）。
    use pasta_lua::debug::source_map::SourceMap;
    let empty = SourceMap::default();
    assert!(
        empty.resolve_lua_to_pasta("any_chunk", 1).is_none(),
        "7.3: production source-map面 (SourceMap::resolve_lua_to_pasta) must be always-compiled \
         (no feature gate); empty aggregate map resolves to None"
    );

    // 2) デッド予約 source_map_slice:bool は SourceMode へ置換済み。source_mode が参照可能で
    //    あること（このコードがコンパイル＝置換済みの証跡; source_map_slice は不在）。
    use pasta_lua::debug::{DebugConfig, SourceMode};
    let cfg = DebugConfig::default();
    let _mode: SourceMode = cfg.source_mode;
    assert_eq!(
        cfg.source_mode,
        SourceMode::Pasta,
        "7.3: DebugConfig must expose `source_mode` (SourceMode) replacing the removed dead \
         `source_map_slice: bool`; default presentation mode is .pasta (6.1)"
    );
}
