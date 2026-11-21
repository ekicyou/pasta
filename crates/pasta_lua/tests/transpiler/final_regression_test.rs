//! 後方互換・ゼロコスト最終回帰ゲート。
//!
//! デバッグ／ソースマップ無効（sink 非装着）での生成 `.lua` が従来と **バイト不変** であることを、
//! 広い代表入力集合で最終確認する。OFF 経路は sink 非装着で `record_*` が no-op ＝ 出力バイト不変・
//! ラインフック未装着・`SourceMap` 非構築。各入力について 2 方向で基準と突合する:
//!
//! 1. **sink 破棄バイト等価**: 本番既定の薄いラッパ `transpile`（sink 非装着）の出力と、
//!    `transpile_with_source_map(Some(&mut sink))`（sink を装着して記録を収集 → 破棄）の出力が
//!    完全にバイト一致する。配線が誤って出力を変えればここが落ちる（タウトロジーではない）。
//! 2. **コミット済みスナップショット基準**: 各入力の OFF 経路出力をコミット済み `.snap` に固定する。
//!    既定 `transpile` の出力が従来から 1 バイトでもずれればスナップショットが落ちる。
//!
//! 代表入力は実運用フィクスチャ（`sample.pasta`・`tail_call_optimization.pasta`・
//! `zero_cost_all_syntax.pasta`）と主要構文種別ごとの最小入力（単純トーク・単語参照・変数代入・
//! シーン呼び出し・属性・複数シーン・アクター単語定義・動的呼び出し・分岐・コードブロック）を網羅する。

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
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
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

/// 代表入力集合（id, ソース）。
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
            "＆天気：晴れ＆場所：東京\n\n\n＊メイン\n  ＆時間帯：朝\n  さくら：「今日の天気は晴れです」\n".to_string(),
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
// 広い代表入力でのゼロコスト・バイト不変（最終 GO ゲート）
// ============================================================================

/// **sink 装着でも OFF 経路とバイト一致（広い網羅・観測専用性の teeth）**。
///
/// 代表入力集合の各入力について、`transpile`（sink 非装着・本番既定）の出力と
/// `transpile_with_source_map(Some(&mut sink))`（ソースマップ機構を踏んで記録を収集 → 破棄）の
/// 出力が **完全にバイト一致** することを表明する。ソースマップ配線（producer の `record_*`）が
/// 誤って 1 バイトでも生成出力を変えればここが落ちる（OFF 経路ゼロコスト = 観測専用の本質）。
///
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

/// **OFF 経路の決定性（広い網羅）**。
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

/// **コミット済みスナップショット基準（真の baseline 突合）**。
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
        // コミット済み .snap が OFF 経路の受理済みベースライン。
        assert_snapshot!(format!("r7_1_off_path__{id}"), lua);
    }
}

// ============================================================================
// Lua ブロックを含まない辞書の特性化
// ============================================================================

/// **Lua ブロックを含まない辞書の変換結果の特性化**。
///
/// 直す前のコードの出力をそのまま期待値として固める。項目の無いローカルシーン・入れ子の
/// 閉じが続く所・アクター辞書だけの区切り・ファイルの最後のシーン・シーン単語だけのシーン・
/// グローバル単語を 1 本の辞書に含める。直した後もこのスナップショットが変わらないこと。
#[test]
fn ponytail_7_5_no_lua_block_characterization() {
    let source = "\
＠挨拶：おはよう、こんにちは

＊メイン
  さくら：「＠挨拶」
  ＞サブ

  ・空

  ・サブ
    うにゅう：「サブです」

％さくら
  ＠一人称：私、わたし

＊単語だけ
  ＠天気：晴れ、雨

＊最後
  さくら：「最後です」

  ・末尾
    うにゅう：「おしまい」
";
    let lua = transpile(source);
    assert!(!lua.is_empty(), "must transpile to real Lua output");
    assert_snapshot!("ponytail_7_5_no_lua_block_characterization", lua);
}
