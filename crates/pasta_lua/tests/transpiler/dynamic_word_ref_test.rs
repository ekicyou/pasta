//! 動的単語参照（＠＄変数名）・動的関数呼び出し（＠＄変数名（…））の生成コードテスト
//!
//! Requirements: 1.5, 3.2, 3.3, 3.9, 4.4, 5.5, 7.2, 7.5
//!
//! 生成コードは値を文字列化せず、参照変数の値（`var.x`・`save.x`・`args[n]`）と
//! その変数パスの文字列リテラルをランタイムへ渡す。アクション行はアクター経由
//! （`act:actor_proxy("アクター"):`）、代入の右辺・式の中は act 経由（`act:`）で呼ぶ。

use crate::common;

use common::e2e_helpers::transpile;
use insta::assert_snapshot;
use pasta_dsl::parser::{Action, Args, Span, VarScope, parse_str};
use pasta_lua::LineEnding;
use pasta_lua::LuaTranspiler;
use pasta_lua::code_gen::LuaCodeGenerator;
use pasta_lua::code_gen::source_map::SourceMapSink;

/// シーン本体の 1 行から生成された Lua コードを返す
fn transpile_body(body: &str) -> String {
    transpile(&format!("＊メイン\n　{body}\n"))
}

/// 生成コードに `line` と完全一致する行があることを確かめる
fn assert_has_line(body: &str, line: &str) {
    let lua = transpile_body(body);
    assert!(
        lua.lines().any(|l| l.trim() == line),
        "{body:?} should generate {line:?}. Code:\n{lua}"
    );
}

// ========================================================================
// アクション行: アクター経由（3.2, 7.5）
// ========================================================================

#[test]
fn test_action_dynamic_word_ref() {
    assert_has_line(
        "さくら：＠＄x　です",
        r#"act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word(var.x, "var.x"))"#,
    );
    assert_has_line(
        "さくら：＠＄＊k",
        r#"act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word(save.k, "save.k"))"#,
    );
    // シーン引数は既存の変数展開と同じパス
    assert_has_line(
        "さくら：＠＄０",
        r#"act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word(args[1], "args[1]"))"#,
    );
}

#[test]
fn test_action_dynamic_fn_call() {
    assert_has_line(
        "さくら：＠＄０（１）",
        r#"act:actor_proxy("さくら"):talk((act:actor_proxy("さくら"):expr_fn_var(args[1], "args[1]", 1)))"#,
    );
    assert_has_line(
        "さくら：＠＄＊f（）",
        r#"act:actor_proxy("さくら"):talk((act:actor_proxy("さくら"):expr_fn_var(save.f, "save.f")))"#,
    );
}

// ========================================================================
// 代入の右辺: act 経由（1.5, 3.3, 4.4）
// ========================================================================

#[test]
fn test_var_set_dynamic_word_ref() {
    assert_has_line("＄y＝＠＄＊k", r#"var.y = act:word(save.k, "save.k")"#);
    assert_has_line("＄＊y＝＠＄x", r#"save.y = act:word(var.x, "var.x")"#);
}

#[test]
fn test_expression_statement_dynamic_word_ref() {
    assert_has_line("＄＝＠＄x", r#"act:word(var.x, "var.x")"#);
}

#[test]
fn test_property_set_dynamic_word_ref() {
    assert_has_line(
        "＄％prop＝＠＄x",
        r#"act:set_property("prop", act:word(var.x, "var.x"))"#,
    );
}

// ========================================================================
// 式の中の動的関数呼び出し: act 経由（7.2）
// ========================================================================

#[test]
fn test_expr_dynamic_fn_call() {
    assert_has_line(
        "＄y＝＠＄f（１）",
        r#"var.y = act:expr_fn_var(var.f, "var.f", 1)"#,
    );
    assert_has_line(
        "＄y＝＠＄f（）＋１",
        r#"var.y = act:expr_fn_var(var.f, "var.f") + 1"#,
    );
    // 静的関数呼び出しの引数の中
    assert_has_line(
        "さくら：＠g（＠＄f（））",
        r#"act:actor_proxy("さくら"):talk((act:actor_proxy("さくら"):expr_fn("g", act:expr_fn_var(var.f, "var.f"))))"#,
    );
}

// ========================================================================
// 生成コードは値を文字列化しない（3.9）
// ========================================================================

#[test]
fn test_no_tostring_on_dynamic_value() {
    let lua = transpile_body("さくら：＠＄x＠＄f（）");
    assert!(!lua.contains("tostring(var."), "Code:\n{lua}");
}

// ========================================================================
// スナップショット
// ========================================================================

#[test]
fn test_dynamic_word_ref_snapshot() {
    let source = r#"
＊メイン
　＄x＝「挨拶」
　さくら：＠＄x　です
　うにゅう：＠＄＊k＠＄０（１、＄a）
　＄y＝＠＄＊k
　＄＊y＝＠＄x
　＄＝＠＄x
　＄％prop＝＠＄x
　＄z＝＠＄f（）＋１
"#;
    assert_snapshot!("dynamic_word_ref", transpile(source));
}

// ========================================================================
// ソースマップ: 2 変種の位置が記録される
// ========================================================================

#[derive(Default)]
struct LinePairSink {
    records: Vec<(u32, u32)>,
}

impl SourceMapSink for LinePairSink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((lua_line, pasta_line));
    }
}

#[test]
fn test_dynamic_actions_record_source_map() {
    let span_at = |line: usize| Span::new(line, 1, line, 10, line * 10, line * 10 + 9);
    let actions = [
        Action::DynamicWordRef {
            var_name: "x".to_string(),
            var_scope: VarScope::Local,
            span: span_at(5),
        },
        Action::DynamicFnCall {
            var_name: "f".to_string(),
            var_scope: VarScope::Global,
            args: Args {
                items: vec![],
                span: span_at(9),
            },
            span: span_at(9),
        },
    ];

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        codegen.set_source_map(&mut sink);
        for action in &actions {
            codegen.generate_action(action, "さくら").unwrap();
        }
    }

    assert_eq!(
        String::from_utf8(output).unwrap(),
        "act:actor_proxy(\"さくら\"):talk(act:actor_proxy(\"さくら\"):word(var.x, \"var.x\"))\n\
         act:actor_proxy(\"さくら\"):talk((act:actor_proxy(\"さくら\"):expr_fn_var(save.f, \"save.f\")))\n"
    );
    assert_eq!(sink.records, vec![(1, 5), (2, 9)]);
}

// ========================================================================
// マニュアルのコード例が読み込めること（5.5）
// ========================================================================

/// 動的参照を記載した文法の章（book/src/grammar/ 配下）
const MANUAL_CHAPTERS: [&str; 5] = [
    "words.md",
    "markers.md",
    "action-line.md",
    "variables.md",
    "actor-dictionary.md",
];

/// Markdown から言語注記が `pasta` のコードブロックを（開きフェンスの行番号, 本文）で返す。
/// 閉じフェンスは開きと同じ長さ以上のバッククォートだけの行（CommonMark）。
/// そのため 4 連フェンスの中の ```lua ブロックも本文に含まれる。
fn pasta_code_blocks(markdown: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    // (開きの行番号, フェンス長, pasta か, 本文)
    let mut open: Option<(usize, usize, bool, String)> = None;
    for (i, line) in markdown.lines().enumerate() {
        let trimmed = line.trim_start();
        let ticks = trimmed.chars().take_while(|&c| c == '`').count();
        let info = trimmed[ticks..].trim();
        match open.as_mut() {
            None if ticks >= 3 => open = Some((i + 1, ticks, info == "pasta", String::new())),
            None => {}
            Some((_, len, _, _)) if ticks >= *len && info.is_empty() => {
                let (start, _, is_pasta, body) = open.take().unwrap();
                if is_pasta {
                    blocks.push((start, body));
                }
            }
            Some((_, _, _, body)) => {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    blocks
}

/// 全角・半角いずれの組み合わせの `＠＄` を含むか
fn has_dynamic_ref(code: &str) -> bool {
    ["＠＄", "@$", "＠$", "@＄"]
        .iter()
        .any(|m| code.contains(m))
}

#[test]
fn test_manual_dynamic_ref_examples_transpile() {
    let grammar_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../book/src/grammar");
    let mut total = 0;
    let mut failures = Vec::new();
    for chapter in MANUAL_CHAPTERS {
        let path = grammar_dir.join(chapter);
        let markdown = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let blocks: Vec<_> = pasta_code_blocks(&markdown)
            .into_iter()
            .filter(|(_, code)| has_dynamic_ref(code))
            .collect();
        println!("{chapter}: {} block(s)", blocks.len());
        total += blocks.len();
        for (line, code) in blocks {
            let name = format!("{chapter}:{line}");
            let result = parse_str(&code, &name)
                .map_err(|e| format!("parse: {e}"))
                .and_then(|file| {
                    LuaTranspiler::default()
                        .transpile(&file, &mut Vec::new())
                        .map(|_| ())
                        .map_err(|e| format!("transpile: {e}"))
                });
            if let Err(e) = result {
                failures.push(format!("{name}: {e}\n{code}"));
            }
        }
    }
    assert!(
        total > 0,
        "no pasta code block with ＠＄ found in the manual"
    );
    assert!(
        failures.is_empty(),
        "manual examples failed to load:\n{}",
        failures.join("\n")
    );
}
