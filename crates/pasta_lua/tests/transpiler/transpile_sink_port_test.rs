//! トランスパイル API へのソースマップ受け渡し口（TranspileSinkPort）。
//!
//! 本番ローダが装着できる sink 受け渡し口（`transpile_with_source_map`）を確立し、in-test の
//! 捕捉 sink を通して `.pasta`（トーク行）をトランスパイルすると、sink が
//! `(lua_line, .pasta span)` を少なくとも 1 件捕捉することを表明する。sink はメモリ内構造へ記録する
//! `&mut dyn SourceMapSink` で、ディスク中間ファイルを必須としない。薄いラッパ `transpile` は従来どおり sink なし。

use pasta_dsl::parser::Span;
use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::code_gen::source_map::SourceMapSink;

/// in-test の捕捉 sink: 渡された `(lua_line, span)` をすべて Vec に貯める。
#[derive(Default)]
struct CapturingSink {
    records: Vec<(u32, Span)>,
}

impl SourceMapSink for CapturingSink {
    // Core (required) method: capture the direct line mapping. This test exercises
    // the span-based `record` path (which carries the full span, asserted on via
    // `span.is_valid()`), so `record_line` synthesizes a line-only span to satisfy
    // the trait contract.
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((
            lua_line,
            Span::new(pasta_line as usize, 0, pasta_line as usize, 0, 0, 0),
        ));
    }

    // Override the default sugar to capture the FULL originating span (the test
    // asserts `span.is_valid()`, i.e. end_byte > 0).
    fn record(&mut self, lua_line: u32, span: Span) {
        self.records.push((lua_line, span));
    }
}

/// トーク（アクション）を含む代表 `.pasta`。`generate_action` の record_span_since 配線を踏む。
const TALK_FIXTURE: &str = "\
＊メイン
  さくら：「こんにちは」
";

/// **sink あり transpile で記録が収集される**。
///
/// 新設の受け渡し口（`transpile_with_source_map`）に in-test 捕捉 sink を渡してトークを含む
/// `.pasta` をトランスパイルすると、sink は少なくとも 1 件の `(lua_line, .pasta span)` を
/// 捕捉する。lua_line は 1 以上（実出力行）、span は有効（end_byte > 0）であること。
#[test]
fn test_transpile_with_source_map_collects_records() {
    let file = parse_str(TALK_FIXTURE, "test.pasta").expect("parse ok");
    let transpiler = LuaTranspiler::default();

    let mut sink = CapturingSink::default();
    let mut output = Vec::new();
    transpiler
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    // 出力は実際に Lua を生成している（vacuous でない）。
    let lua = String::from_utf8(output).expect("utf-8");
    assert!(
        lua.contains("act:actor_proxy(\"さくら\"):talk("),
        "talk action emitted: {lua}"
    );

    // sink は少なくとも 1 件の (lua_line, span) を捕捉する。
    assert!(
        !sink.records.is_empty(),
        "1.1/3.1: sink must capture at least one (lua_line -> pasta span) record"
    );
    let (lua_line, span) = sink.records[0];
    assert!(
        lua_line >= 1,
        "recorded lua_line must be a real output line"
    );
    assert!(
        span.is_valid(),
        "recorded span must be valid (end_byte > 0): {span:?}"
    );
}

/// **sink=None の `transpile_with_source_map` は従来 `transpile` とバイト一致**。
///
/// 受け渡し口に `None` を渡した場合と、薄いラッパ `transpile` を呼んだ場合とで、生成
/// バイトが完全一致する（受け渡し口の追加が OFF 経路の出力を一切変えない）。
#[test]
fn test_transpile_with_none_sink_byte_matches_wrapper() {
    let file = parse_str(TALK_FIXTURE, "test.pasta").expect("parse ok");
    let transpiler = LuaTranspiler::default();

    let mut out_none = Vec::new();
    transpiler
        .transpile_with_source_map(&file, &mut out_none, None)
        .expect("transpile ok");

    let mut out_wrapper = Vec::new();
    transpiler
        .transpile(&file, &mut out_wrapper)
        .expect("transpile ok");

    assert_eq!(
        out_none, out_wrapper,
        "7.1: transpile_with_source_map(None) must be byte-identical to the thin transpile wrapper"
    );
    assert!(
        !out_wrapper.is_empty(),
        "fixture must produce real output bytes"
    );
}
