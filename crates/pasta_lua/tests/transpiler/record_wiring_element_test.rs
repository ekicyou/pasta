//! Task 2.4 — 要素生成の全 record 配線（RecordWiring / element_gen）。
//!
//! 仕様参照（`.kiro/specs/pasta-source-map/`）:
//! - requirements.md **1.1**: ソースマップ生成が有効である時、生成 `.lua` 行に由来
//!   `.pasta` 位置を記録する。
//! - requirements.md **1.3**: 単一の `.pasta` 要素が複数の `.lua` 行を生成する時、その
//!   全行を `.pasta` 位置へ対応づける。
//! - requirements.md **1.4**: 主要構文種別（アクション・スコープ・分岐・単語定義など）を
//!   網羅して記録する。本タスクは変数代入・シーン呼び出し・単語定義（グローバル/アクター/
//!   ローカル）・コードブロックを担当する（アクションは既配線）。
//!
//! design.md 参照:
//! - "RecordWiring"（315-358 行）: 各 `generate_*` で `out_line` デルタ検出パターンを踏襲し、
//!   出力行群へ AST ノードの `span` を対応づける。コードブロックは各出力行を
//!   `record_block_line(block.span.start_line + 行オフセット)` で **個別** 記録する。
//! - "Service Interface"（348-349 行）: `record_span(span)` / `record_block_line(pasta_line)`。
//!
//! # 検証戦略
//!
//! - 単体テスト: 構築した AST ノードに明示 `.pasta` span を与え、直接 `generate_*` を呼び、
//!   捕捉 sink が「記録された `.lua` 行 → 期待 `.pasta` 行」を一致させることを表明する。
//!   各 `generate_*` の record 配線を点で証明する。
//! - 統合テスト: 実 `.pasta` フィクスチャを `transpile_with_sink` に通し、パーサ由来の実 span に
//!   対してコードブロックの **行ごと** 写像（フェンス行を跨いだ `+1` オフセット）が正しいことを
//!   実証する。

use pasta_dsl::parser::parse_str;
use pasta_dsl::parser::{CodeBlock, Expr, KeyWords, SetValue, Span, VarScope, VarSet};
use pasta_lua::LineEnding;
use pasta_lua::LuaTranspiler;
use pasta_lua::code_gen::LuaCodeGenerator;
use pasta_lua::code_gen::source_map::SourceMapSink;

/// 捕捉 sink: 渡された全 `(lua_line, pasta_line)` を順序どおり貯める。
///
/// span 経由（`record`）でも line 経由（`record_line`）でも、最終的に格納するのは
/// `(出力 .lua 行, 由来 .pasta 行)` のペア。これにより構文種別を跨いで「出力行 →
/// 期待 .pasta 行」を一様に表明できる。
#[derive(Default)]
struct LinePairSink {
    records: Vec<(u32, u32)>,
}

impl SourceMapSink for LinePairSink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((lua_line, pasta_line));
    }
}

/// 既知 `.pasta` 行を持つ有効 span（end_byte > 0）を作る小ヘルパ。
fn span_at(line: usize) -> Span {
    // start_line/col, end_line/col, start_byte, end_byte（end_byte>0 で is_valid）
    Span::new(line, 1, line, 10, line * 10, line * 10 + 9)
}

// ============================================================================
// 1.1 — 変数代入（var_set）の record 配線
// ============================================================================

/// ローカル変数代入（式値）: 1 行出力 → その行が由来 `.pasta` 行へ対応づく。
#[test]
fn var_set_local_expr_records_its_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        let var_set = VarSet {
            name: Some("カウンタ".to_string()),
            scope: VarScope::Local,
            value: SetValue::Expr(Expr::Integer(10)),
            span: span_at(7),
        };
        codegen.generate_var_set(&var_set).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua, "var.カウンタ = 10\n");
    // 出力 .lua 行 1 が .pasta 行 7（var_set.span.start_line）へ対応づく。
    assert_eq!(
        sink.records,
        vec![(1, 7)],
        "1.1: var_set (local expr) must map its output line to its .pasta line"
    );
}

/// グローバル変数代入（単語参照）も同様に 1 行 → 由来行へ対応づく。
#[test]
fn var_set_global_wordref_records_its_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        let var_set = VarSet {
            name: Some("グローバル".to_string()),
            scope: VarScope::Global,
            value: SetValue::WordRef {
                name: "単語".to_string(),
            },
            span: span_at(12),
        };
        codegen.generate_var_set(&var_set).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua, "save.グローバル = act:word(\"単語\")\n");
    assert_eq!(
        sink.records,
        vec![(1, 12)],
        "1.1: var_set (global wordref) must map its output line to its .pasta line"
    );
}

/// プロパティ代入（generate_property_set へ委譲する分岐）も 1 行 → 由来行へ対応づく。
#[test]
fn var_set_property_records_its_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        let var_set = VarSet {
            name: Some("mood".to_string()),
            scope: VarScope::Property,
            value: SetValue::Expr(Expr::String("happy".to_string())),
            span: span_at(20),
        };
        codegen.generate_var_set(&var_set).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua, "act:set_property(\"mood\", \"happy\")\n");
    assert_eq!(
        sink.records,
        vec![(1, 20)],
        "1.1: var_set (property) must map its delegated output line to its .pasta line"
    );
}

// ============================================================================
// 1.1 — シーン呼び出し（call_scene）の record 配線
//
// `generate_call_scene` は pub(super)（直接呼べない）ため、実 `.pasta` を
// transpile_with_sink に通し、`act:call(...)` 出力行が呼び出し `.pasta` 行へ対応づくことを
// 表明する。呼び出し行の `.pasta` 行番号は固定フィクスチャで既知。
// ============================================================================

/// `＞` 呼び出しを含む `.pasta` を実トランスパイルし、`act:call(...)` 出力行が呼び出し元
/// `.pasta` 行（このフィクスチャでは行 4）へ対応づくことを表明する。
#[test]
fn call_scene_records_call_pasta_line_via_pipeline() {
    // 行1: 空, 行2: ＊メイン, 行3: トーク, 行4: ＞サブ会話, ...
    let source = "\n＊メイン\n  さくら：「よびます」\n  ＞サブ会話\n\n  ・サブ会話\n    うにゅう：「サブです」\n";
    let file = parse_str(source, "call.pasta").expect("parse ok");

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    LuaTranspiler::default()
        .transpile_with_sink(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    let lua = String::from_utf8(output).unwrap();
    // 出力中の `act:call(` 行（0/1 始まり差を吸収するため 1 始まりへ補正）を特定。
    let call_lua_line = lua
        .lines()
        .position(|l| l.contains("act:call("))
        .map(|i| i as u32 + 1)
        .expect("transpiled output must contain an act:call(...) line");

    // その出力 .lua 行が呼び出し元 .pasta 行（4）へ対応づいて記録されていること。
    assert!(
        sink.records.contains(&(call_lua_line, 4)),
        "1.1: call_scene output line {call_lua_line} must map to .pasta call line 4; records = {:?}",
        sink.records
    );
}

// ============================================================================
// 1.1 / 1.3 — 単語定義（global / local）の record 配線
// ============================================================================

/// グローバル単語定義（単一キー）: 1 行 → 由来行へ対応づく。
#[test]
fn global_word_single_key_records_its_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        let word = KeyWords {
            names: vec!["挨拶".to_string()],
            words: vec!["おはよう".to_string(), "こんにちは".to_string()],
            span: span_at(3),
        };
        codegen.generate_global_word(&word).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(
        lua,
        "PASTA.create_word(\"挨拶\"):entry(\"おはよう\", \"こんにちは\")\n"
    );
    assert_eq!(
        sink.records,
        vec![(1, 3)],
        "1.1: global word definition must map its output line to its .pasta line"
    );
}

/// グローバル単語定義（複数キー）: 1 要素 → 複数 `.lua` 行。**全行**が同一 `.pasta` 行へ対応づく（1.3）。
#[test]
fn global_word_multi_key_maps_all_lines_to_same_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        // 3 キー → 3 出力行。すべて span_at(5).start_line == 5 へ写像されること。
        let word = KeyWords {
            names: vec!["朝".to_string(), "昼".to_string(), "夜".to_string()],
            words: vec!["はい".to_string()],
            span: span_at(5),
        };
        codegen.generate_global_word(&word).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua.matches('\n').count(), 3, "3 keys must emit 3 lines: {lua}");
    // 1.3: 3 出力行すべてが同一 .pasta 行(5)へ対応づく。
    assert_eq!(
        sink.records,
        vec![(1, 5), (2, 5), (3, 5)],
        "1.3: every .lua line of a multi-key word definition maps to the same .pasta line"
    );
}

/// シーンローカル単語定義も同じ写像（colon 構文 SCENE:create_word）。
#[test]
fn local_word_records_its_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        let word = KeyWords {
            names: vec!["場所".to_string()],
            words: vec!["東京".to_string()],
            span: span_at(8),
        };
        codegen.generate_local_word(&word).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua, "SCENE:create_word(\"場所\"):entry(\"東京\")\n");
    assert_eq!(
        sink.records,
        vec![(1, 8)],
        "1.1: local word definition must map its output line to its .pasta line"
    );
}

// ============================================================================
// 1.1 / 1.3 — コードブロック（code_block）の行ごと record 配線
// ============================================================================

/// コードブロック: 各出力 `.lua` 行を **個別** に由来 `.pasta` 行へ対応づける（議題3）。
///
/// `block.span.start_line` は開きフェンス行。content はその次行から始まるので、
/// content 行オフセット `i` の由来 `.pasta` 行は `start_line + 1 + i`（フェンス行ぶんの
/// 定数 `+1` 補正）。3 行の content に対し 3 件の行ごと写像が記録されること。
#[test]
fn code_block_maps_each_line_individually_with_fence_offset() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        // 開きフェンスが .pasta 行 12 にあると仮定（span.start_line = 12）。
        // content は 3 行: 由来 .pasta 行は 13, 14, 15。
        let block = CodeBlock {
            language: Some("lua".to_string()),
            content: "function ACTOR.時刻(act)\n    return \"朝\"\nend".to_string(),
            span: span_at(12),
        };
        codegen.generate_code_block(&block).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua.matches('\n').count(), 3, "3 content lines emit 3 lua lines: {lua}");
    // 各出力行が個別の .pasta 行へ写像される（フェンス行 12 を跨いだ +1 補正）。
    assert_eq!(
        sink.records,
        vec![(1, 13), (2, 14), (3, 15)],
        "1.3: each code-block .lua line maps individually to start_line + 1 + offset"
    );
}

// ============================================================================
// 統合: 実 `.pasta` を transpile_with_sink に通し、実 span でコードブロック行写像を検証
// ============================================================================

/// 実フィクスチャ（`zero_cost_all_syntax.pasta`）を実トランスパイルし、パーサ由来の実 span に
/// 対してコードブロックの行ごと写像が正しいことを実証する（`+1` フェンスオフセットの実地検証）。
///
/// このフィクスチャは 2 つの `lua` コードブロックを含む:
/// - アクター `さくら` 内（開きフェンス行 12、content: 13/14/15 行）
/// - グローバルシーン末尾（開きフェンス行 32、content: 33/34 行）
///
/// それぞれの content 各行 `L` について、`record_line(lua_line, L)` が記録されていること
/// （出力 .lua 行は scene scaffold により後方にずれるが、由来 .pasta 行は content 実行と一致）。
#[test]
fn integration_code_block_lines_map_to_real_pasta_lines() {
    let source = include_str!("../fixtures/zero_cost_all_syntax.pasta");
    let file = parse_str(source, "zero_cost_all_syntax.pasta").expect("parse ok");

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    let transpiler = LuaTranspiler::default();
    transpiler
        .transpile_with_sink(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    // 記録された由来 .pasta 行の集合を抽出。
    let pasta_lines: std::collections::BTreeSet<u32> =
        sink.records.iter().map(|&(_, p)| p).collect();

    // コードブロック content 行（13,14,15 と 33,34）がすべて記録由来 .pasta 行に含まれる。
    // これが含まれることで、フェンス行を跨いだ `+1` オフセットが実 span で正しいと実証される。
    for expected in [13u32, 14, 15, 33, 34] {
        assert!(
            pasta_lines.contains(&expected),
            "code-block content .pasta line {expected} must be recorded; recorded set = {pasta_lines:?}"
        );
    }

    // フェンス行（12, 32）そのものは content ではないので、コードブロック由来としては
    // 記録されない（content は次行から始まる = +1 補正の本質）。なお 12 行目はアクター
    // ヘッダ等とは無関係であり、本フィクスチャでコードブロック以外がその行を記録しないため
    // 「フェンス行が記録されない」ことを直接表明できる。
    assert!(
        !pasta_lines.contains(&12),
        "opening fence line 12 must NOT be recorded as a code-block content line (set = {pasta_lines:?})"
    );

    // コードブロックの各 .lua 出力行は 1:1 で content 行に対応する。記録に重複 lua_line が
    // ないこと（行ごと個別写像）も確認しておく。
    let mut seen = std::collections::HashSet::new();
    for &(lua_line, _) in &sink.records {
        assert!(
            seen.insert(lua_line),
            "each output .lua line should be recorded at most once (dup lua_line {lua_line})"
        );
    }
}

/// 統合: sink=None（本番既定）でフィクスチャをトランスパイルしても、sink=Some とバイト一致。
/// 配線追加が OFF 経路の出力を一切変えないこと（要件 7.1 ゼロコスト）の実地確認。
#[test]
fn integration_sink_none_is_byte_identical_to_sink_some() {
    let source = include_str!("../fixtures/zero_cost_all_syntax.pasta");
    let file = parse_str(source, "zero_cost_all_syntax.pasta").expect("parse ok");
    let transpiler = LuaTranspiler::default();

    let mut out_none = Vec::new();
    transpiler
        .transpile_with_sink(&file, &mut out_none, None)
        .expect("transpile ok");

    let mut sink = LinePairSink::default();
    let mut out_some = Vec::new();
    transpiler
        .transpile_with_sink(&file, &mut out_some, Some(&mut sink))
        .expect("transpile ok");

    assert_eq!(
        out_none, out_some,
        "7.1: attaching a sink must not change generated bytes (record wiring is observe-only)"
    );
    assert!(!out_none.is_empty(), "fixture must produce real output bytes");
    // sink=Some 側は実際に記録を収集している（vacuous でない）。
    assert!(
        !sink.records.is_empty(),
        "sink=Some must collect records for the all-syntax fixture"
    );
}
