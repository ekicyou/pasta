//! Source-map seam tests for the Lua code generator (Task 5.1).
//!
//! These tests prove the R4 feasibility seam:
//! - With NO sink attached, generated Lua is byte-identical to before (covered by
//!   the existing insta snapshots; here we assert the inert default explicitly).
//! - With a CAPTURING sink attached, the generator records `(out_line -> .pasta span)`
//!   pairs for the representative action-emitting path, proving `.pasta`<->`.lua`
//!   line correspondence is RESOLVABLE (Requirement 4.1) via a real recording seam
//!   (Requirement 4.2).
//! - The `out_line` counter accurately tracks the number of generated output lines.

use pasta_dsl::parser::{Action, Span};
use pasta_lua::LineEnding;
use pasta_lua::code_gen::LuaCodeGenerator;
use pasta_lua::code_gen::source_map::SourceMapSink;

/// A capturing sink that records every `(lua_line, span)` pair it is handed.
#[derive(Default)]
struct CapturingSink {
    records: Vec<(u32, Span)>,
}

impl SourceMapSink for CapturingSink {
    fn record(&mut self, lua_line: u32, span: Span) {
        self.records.push((lua_line, span));
    }
}

/// A representative `.pasta` span (distinct, valid: end_byte > 0).
fn sample_span() -> Span {
    // start_line/col, end_line/col, start_byte, end_byte
    Span::new(7, 3, 7, 20, 42, 60)
}

#[test]
fn test_no_sink_is_byte_identical_and_records_nothing() {
    // No sink: the seam is inert. Output must match the plain path exactly.
    // Force LF so the byte assertion is platform-independent (Windows default is CRLF).
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);

    let action = Action::Talk {
        text: "こんにちは".to_string(),
        span: sample_span(),
    };
    codegen.generate_action(&action, "さくら").unwrap();

    let with_seam = String::from_utf8(output).unwrap();

    // Baseline: the exact bytes the generator has always produced for a talk action.
    assert_eq!(with_seam, "act.さくら:talk(\"こんにちは\")\n");
}

#[test]
fn test_capturing_sink_records_line_to_span_for_action_path() {
    let mut sink = CapturingSink::default();
    let span = sample_span();

    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);

        let action = Action::Talk {
            text: "こんにちは".to_string(),
            span,
        };
        codegen.generate_action(&action, "さくら").unwrap();
    }

    // Output bytes are unchanged by the presence of the sink.
    let lua = String::from_utf8(output).unwrap();
    assert_eq!(lua, "act.さくら:talk(\"こんにちは\")\n");

    // The seam recorded exactly the line that was emitted for this action,
    // mapped back to the originating `.pasta` span.
    assert_eq!(
        sink.records,
        vec![(1, span)],
        "expected a single (lua_line=1 -> span) record for the talk action"
    );
}

#[test]
fn test_capturing_sink_records_multiple_actions_with_increasing_lines() {
    let mut sink = CapturingSink::default();
    let span_a = Span::new(1, 1, 1, 10, 0, 9);
    let span_b = Span::new(2, 1, 2, 10, 10, 19);

    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::new(&mut output);
        codegen.set_source_map(&mut sink);

        codegen
            .generate_action(
                &Action::Talk {
                    text: "A".to_string(),
                    span: span_a,
                },
                "さくら",
            )
            .unwrap();
        codegen
            .generate_action(
                &Action::Talk {
                    text: "B".to_string(),
                    span: span_b,
                },
                "さくら",
            )
            .unwrap();
    }

    // Two emitted lines -> two records on consecutive output lines.
    assert_eq!(sink.records, vec![(1, span_a), (2, span_b)]);
}

#[test]
fn test_out_line_counter_matches_emitted_line_count() {
    let mut output = Vec::new();
    let mut codegen = LuaCodeGenerator::new(&mut output);

    // Header emits 3 lines: two requires + one blank line.
    codegen.write_header().unwrap();
    assert_eq!(codegen.out_line(), 3, "header should emit 3 output lines");

    // Each talk action emits exactly one line.
    codegen
        .generate_action(
            &Action::Talk {
                text: "x".to_string(),
                span: sample_span(),
            },
            "さくら",
        )
        .unwrap();
    assert_eq!(codegen.out_line(), 4);

    drop(codegen);

    // Cross-check: counter equals the actual number of LF-terminated lines.
    let s = String::from_utf8(output).unwrap();
    let emitted = s.matches('\n').count() as u32;
    assert_eq!(emitted, 4, "actual emitted lines should match out_line");
}

/// Task 8.2 — ゼロコスト/サンドボックス **集約回帰ゲート（バイト一致サブ）**。
///
/// このモジュールは「本番 transpile（sink 非装着）出力のバイト不変」を **一箇所に集約** し、
/// 対応する要件へ明示マッピングする durable な回帰ゲートである。R5.2/5.3/5.5 のランタイム不変
/// 条件は `tests/runtime/debug_integration_test.rs` の同名モジュールが受け持つ（ランタイム
/// アクセスが要るため分割。本モジュールは code_gen 本番 API に届く transpiler ターゲット側）。
///
/// 要件マッピング（`.kiro/specs/pasta-vscode-lua-debug/requirements.md`）:
/// - **R4.6**: スライス無効時、実証スライスのコード経路を本番動作へ露出せず追加コストを与えない。
///   → 本番 transpile（sink=None）は出力にゼロバイトも追加しない。
/// - **R5.2**: 無効時、本番実行の挙動に追加コストを与えない（出力バイト一致＝回帰なし）。
///
/// design.md 参照: "Performance/Regression"（無効ビルドでのトランスパイル出力バイト一致・
/// `SourceMapSink=None`・回帰なし）、"DebugConfig & Gate"（4.6 = None sink / feature gate）、
/// および本ファイル冒頭が依拠する insta スナップショット群（5.1）が standing gate。
///
/// 既存の `test_no_sink_is_byte_identical_and_records_nothing`（action 単位）と insta
/// スナップショット（5.1）に加え、ここでは **本番 transpile API 全体**（`LuaTranspiler`）を
/// 代表 `.pasta` 入力に通し、(a) 既知 golden とのバイト完全一致、(b) 複数回実行の決定性、
/// (c) `None` sink がゼロバイトも足さないこと、を直接表明する。
mod zero_cost_sandbox_regression {
    use pasta_dsl::parser::parse_str;
    use pasta_lua::LuaTranspiler;

    /// 代表 `.pasta` 入力（複数行・複数アクションを含み、出力行数が安定して再現可能）。
    /// 末尾改行はパーサ要件（action 行は eol 終端）。
    const FIXTURE: &str = "\
＊メイン
  さくら：「こんにちは」
  うにゅう：「やあ」
";

    /// 本番 transpile（`LuaTranspiler::default()` = sink 非装着）を実行し、生成 `.lua` バイトを返す。
    /// これは production の唯一の transpile 経路であり、source-map sink は一切装着しない。
    fn transpile_production(source: &str) -> Vec<u8> {
        let file = parse_str(source, "test.pasta").expect("parse ok");
        let transpiler = LuaTranspiler::default();
        let mut output = Vec::new();
        transpiler
            .transpile(&file, &mut output)
            .expect("transpile ok");
        output
    }

    /// **R5.2 — 決定性**: 本番 transpile を 2 回独立に走らせると、生成バイトは完全一致する。
    /// sink 非装着パスが非決定的な副作用を持たないこと（= ゼロコスト・回帰なし）の直接証明。
    #[test]
    fn r5_2_production_transpile_is_deterministic_byte_for_byte() {
        let run1 = transpile_production(FIXTURE);
        let run2 = transpile_production(FIXTURE);
        assert_eq!(
            run1, run2,
            "R5.2: production transpile (no sink) must be byte-for-byte deterministic"
        );
        // 空でない実出力に対する決定性であることを保証（vacuous でない）。
        assert!(!run1.is_empty(), "fixture must produce real output bytes");
    }

    /// **R4.6 — None sink はゼロバイトも追加しない**（バイト golden）。
    ///
    /// `source_map: Option<&mut dyn SourceMapSink>` フィールドは常に存在し、本番では既定
    /// `None`。その存在が出力へ一切のバイトを足さないことを、既知 golden との完全一致で表明。
    /// （行末は環境差を避けるため LF 正規化して比較する。）
    #[test]
    fn r4_6_none_sink_emits_zero_extra_bytes_golden() {
        let produced = transpile_production(FIXTURE);
        let produced = String::from_utf8(produced).expect("utf-8");
        // Windows の autocrlf 差を吸収（バイト一致の本質は「シームが余分なバイトを足さない」）。
        let produced_lf = produced.replace("\r\n", "\n");

        // 既知 golden: 本番 transpile（scene scaffold + 2 talk アクション）の完全出力。
        // None sink でも余分な行・余分なバイトは一切付かない（シームはゼロバイト）。
        let golden = "\
local PASTA = require \"pasta\"
local GLOBAL = require \"pasta.global\"

do
    local SCENE = PASTA.create_scene(\"メイン\")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk(\"「こんにちは」\")
        act.うにゅう:talk(\"「やあ」\")
    end
end
";
        assert_eq!(
            produced_lf, golden,
            "R4.6: no-sink production transpile must byte-match the golden (seam adds zero bytes)"
        );
    }
}
