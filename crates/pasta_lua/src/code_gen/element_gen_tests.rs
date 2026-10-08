use super::*;
use crate::code_gen::source_map::SourceMapSink;
use crate::config::LineEnding;
use pasta_dsl::parser::{Arg, Args, BinOp, CallTarget, FnScope};

/// Test sink capturing each `(lua_line, pasta_line)` record.
#[derive(Default)]
struct CapturingSink {
    records: Vec<(u32, u32)>,
}

impl SourceMapSink for CapturingSink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((lua_line, pasta_line));
    }
}

fn gen_to_string<F>(f: F) -> String
where
    F: FnOnce(&mut LuaCodeGenerator<'_, Vec<u8>>) -> Result<(), TranspileError>,
{
    let mut output = Vec::new();
    {
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        f(&mut cg).unwrap();
    }
    String::from_utf8(output).unwrap()
}

// ------------------------------------------------------------------
// generate_action edge cases
// ------------------------------------------------------------------

/// A malformed escape sequence with no second character emits NOTHING:
/// no bytes, no out_line advance, and no source-map record even with a
/// sink attached (the documented empty-escape case).
#[test]
fn escape_with_single_char_sequence_emits_nothing() {
    let mut sink = CapturingSink::default();
    let mut output = Vec::new();
    let final_out_line;
    {
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        cg.set_source_map(&mut sink);
        let action = Action::Escape {
            sequence: "@".to_string(), // no char at index 1
            span: Span::new(1, 1, 1, 2, 0, 1),
        };
        cg.generate_action(&action, "さくら").unwrap();
        final_out_line = cg.out_line();
    }
    assert!(output.is_empty(), "empty escape must emit no bytes");
    assert_eq!(final_out_line, 0, "out_line must not advance");
    assert!(sink.records.is_empty(), "no record without an emitted line");
}

/// `\\` and `\%` escapes talk both characters (`[[\\]]`/`[[\%]]` evaluate to the 2-char
/// strings), while `＠＠`/`＄＄`/`@@`/`$$` keep talking only the second character
/// (Req 4.1/4.6/4.7; paragraph-break-tag-only-talk Req 8.4).
#[test]
fn escape_backslash_talks_two_chars_others_talk_one() {
    let cases = [
        ("\\\\", "act:actor_proxy(\"さくら\"):talk([[\\\\]])\n"),
        ("\\%", "act:actor_proxy(\"さくら\"):talk([[\\%]])\n"),
        ("＠＠", "act:actor_proxy(\"さくら\"):talk(\"＠\")\n"),
        ("＄＄", "act:actor_proxy(\"さくら\"):talk(\"＄\")\n"),
        ("@@", "act:actor_proxy(\"さくら\"):talk(\"@\")\n"),
        ("$$", "act:actor_proxy(\"さくら\"):talk(\"$\")\n"),
    ];
    for (sequence, expected) in cases {
        let text = gen_to_string(|cg| {
            cg.generate_action(
                &Action::Escape {
                    sequence: sequence.to_string(),
                    span: Span::default(),
                },
                "さくら",
            )
        });
        assert_eq!(text, expected, "escape {:?}", sequence);
    }
}

/// SakuraScript action emits `act:actor_proxy("{actor}"):sakura_script(<literal>)`.
#[test]
fn sakura_script_action_emits_sakura_script_call() {
    let text = gen_to_string(|cg| {
        cg.generate_action(
            &Action::SakuraScript {
                script: "\\s[0]".to_string(),
                span: Span::default(),
            },
            "さくら",
        )
    });
    assert!(
        text.contains("act:actor_proxy(\"さくら\"):sakura_script("),
        "must route through sakura_script, got: {}",
        text
    );
    assert!(
        text.contains("\\s[0]"),
        "script payload preserved: {}",
        text
    );
}

// ------------------------------------------------------------------
// Continuation lines
// ------------------------------------------------------------------

/// A continuation line before any speaker line is an InvalidContinuation
/// error (no actor to inherit).
#[test]
fn continue_action_without_prior_actor_is_invalid_continuation_error() {
    let mut output = Vec::new();
    let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
    let cont = ContinueAction {
        actions: vec![Action::Talk {
            text: "続き".to_string(),
            span: Span::default(),
        }],
        span: Span::new(2, 1, 2, 3, 5, 8),
    };
    let err = cg.generate_continue_action(&cont, &None).unwrap_err();
    assert!(
        matches!(err, TranspileError::InvalidContinuation { .. }),
        "expected InvalidContinuation, got: {:?}",
        err
    );
}

/// `generate_action_line` records its actor as `last_actor`, and a
/// following continuation line inherits that speaker.
#[test]
fn continue_action_inherits_actor_from_preceding_action_line() {
    let text = gen_to_string(|cg| {
        let mut last_actor: Option<String> = None;
        cg.generate_action_line(
            &ActionLine {
                actor: "うにゅう".to_string(),
                actions: vec![Action::Talk {
                    text: "やあ".to_string(),
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            &mut last_actor,
        )?;
        assert_eq!(last_actor.as_deref(), Some("うにゅう"));
        cg.generate_continue_action(
            &ContinueAction {
                actions: vec![Action::Talk {
                    text: "続き".to_string(),
                    span: Span::default(),
                }],
                span: Span::default(),
            },
            &last_actor,
        )
    });
    assert_eq!(
        text,
        "act:actor_proxy(\"うにゅう\"):talk(\"やあ\")\nact:actor_proxy(\"うにゅう\"):talk(\"続き\")\n",
        "continuation must reuse the inherited speaker"
    );
}

// ------------------------------------------------------------------
// generate_call_scene
// ------------------------------------------------------------------

fn call_scene(target: CallTarget, args: Option<Args>) -> CallScene {
    CallScene {
        target,
        args,
        span: Span::default(),
    }
}

/// Static mid call (not tail) goes through `act:call_restore` and forwards only
/// `table.unpack(args)`.
#[test]
fn call_scene_static_without_args_forwards_table_unpack_only() {
    let text = gen_to_string(|cg| {
        cg.generate_call_scene(
            &call_scene(CallTarget::Static("次シーン".to_string()), None),
            false,
        )
    });
    assert_eq!(
        text,
        "act:call_restore(SCENE.__global_name__, \"次シーン\", {}, table.unpack(args))\n"
    );
}

/// `Some(args)` with an EMPTY item list behaves like no args at all
/// (still only `table.unpack(args)` — no leading comma artifacts).
#[test]
fn call_scene_with_empty_args_list_matches_no_args_form() {
    let text = gen_to_string(|cg| {
        cg.generate_call_scene(
            &call_scene(CallTarget::Static("次".to_string()), Some(Args::empty())),
            false,
        )
    });
    assert_eq!(
        text,
        "act:call_restore(SCENE.__global_name__, \"次\", {}, table.unpack(args))\n"
    );
}

/// Positional and keyword args are emitted in order before
/// `table.unpack(args)`; keyword keys are dropped (value-only).
#[test]
fn call_scene_emits_positional_and_keyword_args_before_unpack() {
    let args = Args {
        items: vec![
            Arg::Positional(Expr::Integer(1)),
            Arg::Keyword {
                key: "名前".to_string(),
                value: Expr::String("さくら".to_string()),
            },
        ],
        span: Span::default(),
    };
    let text = gen_to_string(|cg| {
        cg.generate_call_scene(
            &call_scene(CallTarget::Static("次".to_string()), Some(args)),
            false,
        )
    });
    assert_eq!(
        text,
        "act:call_restore(SCENE.__global_name__, \"次\", {}, 1, \"さくら\", table.unpack(args))\n"
    );
}

fn dyn_call(expr: Expr, args: Option<Args>, is_tail_call: bool) -> String {
    gen_to_string(|cg| {
        cg.generate_call_scene(&call_scene(CallTarget::Dynamic(expr), args), is_tail_call)
    })
}

fn var_ref(name: &str, scope: VarScope) -> Expr {
    Expr::VarRef {
        name: name.to_string(),
        scope,
    }
}

/// Tail static call is unchanged: `return act:call(...)` (Lua TCO, Req 4.9).
#[test]
fn call_scene_tail_call_prepends_return() {
    let text = gen_to_string(|cg| {
        cg.generate_call_scene(
            &call_scene(CallTarget::Static("次".to_string()), None),
            true,
        )
    });
    assert_eq!(
        text,
        "return act:call(SCENE.__global_name__, \"次\", {}, table.unpack(args))\n"
    );
}

/// The 4 forms: tail/mid × static/dynamic (Req 1.8, 4.9). The key expression
/// sits left of the args (evaluation order unchanged, Req 5.9, 6.5) and no
/// `tostring(...)` is generated.
#[test]
fn call_scene_dynamic_tail_and_mid_forms() {
    let args = || {
        Some(Args {
            items: vec![Arg::Positional(Expr::Integer(1))],
            span: Span::default(),
        })
    };
    assert_eq!(
        dyn_call(var_ref("行き先", VarScope::Local), args(), true),
        "return act:call(SCENE.__global_name__, act:call_key(var.行き先, \"var.行き先\"), {}, 1, table.unpack(args))\n"
    );
    assert_eq!(
        dyn_call(var_ref("行き先", VarScope::Local), args(), false),
        "act:call_restore(SCENE.__global_name__, act:call_key(var.行き先, \"var.行き先\"), {}, 1, table.unpack(args))\n"
    );
}

/// Key form 1: a single variable reference passes the value and its path
/// (`＄x`・`＄＊x`・`＄０`).
#[test]
fn call_scene_dynamic_key_single_var_ref_passes_value_and_path() {
    let cases = [
        (
            var_ref("x", VarScope::Local),
            "act:call_key(var.x, \"var.x\")",
        ),
        (
            var_ref("x", VarScope::Global),
            "act:call_key(save.x, \"save.x\")",
        ),
        (
            var_ref("0", VarScope::Args(0)),
            "act:call_key(args[1], \"args[1]\")",
        ),
    ];
    for (expr, key) in cases {
        assert_eq!(
            dyn_call(expr, None, false),
            format!("act:call_restore(SCENE.__global_name__, {key}, {{}}, table.unpack(args))\n")
        );
    }
}

/// Key form 2: a single function call (incl. parenthesized) passes the value,
/// `nil` and the operand notation (`@f()`・`@*f()`・`@$var.v()`).
#[test]
fn call_scene_dynamic_key_single_fn_call_passes_value_and_desc() {
    let local_fn = || Expr::FnCall {
        name: "f".to_string(),
        args: Args::empty(),
        scope: FnScope::Local,
    };
    let cases = [
        (
            local_fn(),
            "act:call_key(act:expr_fn(\"f\"), nil, \"@f()\")",
        ),
        (
            Expr::FnCall {
                name: "g".to_string(),
                args: Args::empty(),
                scope: FnScope::Global,
            },
            "act:call_key(act:global_fn(\"g\"), nil, \"@*g()\")",
        ),
        (
            Expr::DynamicFnCall {
                var_name: "v".to_string(),
                var_scope: VarScope::Local,
                args: Args::empty(),
            },
            "act:call_key(act:expr_fn_var(var.v, \"var.v\"), nil, \"@$var.v()\")",
        ),
        (
            Expr::Paren(Box::new(local_fn())),
            "act:call_key((act:expr_fn(\"f\")), nil, \"@f()\")",
        ),
    ];
    for (expr, key) in cases {
        assert_eq!(
            dyn_call(expr, None, true),
            format!("return act:call(SCENE.__global_name__, {key}, {{}}, table.unpack(args))\n")
        );
    }
}

/// Key form 3: anything else (string, number, arithmetic, concat) passes the
/// value only.
#[test]
fn call_scene_dynamic_key_other_expr_passes_value_only() {
    let cases = [
        (Expr::String("名前".to_string()), "act:call_key(\"名前\")"),
        (Expr::Integer(3), "act:call_key(3)"),
        (
            Expr::Binary {
                op: BinOp::Concat,
                lhs: Box::new(Expr::String("a".to_string())),
                rhs: Box::new(var_ref("x", VarScope::Local)),
            },
            "act:call_key((\"a\" .. PASTA.str(var.x, \"var.x\")))",
        ),
    ];
    for (expr, key) in cases {
        let text = dyn_call(expr, None, false);
        assert_eq!(
            text,
            format!("act:call_restore(SCENE.__global_name__, {key}, {{}}, table.unpack(args))\n")
        );
        assert!(!text.contains("tostring("), "no tostring: {text}");
    }
}

// ------------------------------------------------------------------
// Word definitions
// ------------------------------------------------------------------

fn key_words(names: &[&str], words: &[&str]) -> KeyWords {
    KeyWords {
        names: names.iter().map(|s| s.to_string()).collect(),
        words: words.iter().map(|s| s.to_string()).collect(),
        span: Span::new(3, 1, 3, 10, 30, 50),
    }
}

/// A word definition with an empty value list emits nothing (early
/// return), for both global and local flavors.
#[test]
fn word_definition_with_no_words_emits_nothing() {
    let global = gen_to_string(|cg| cg.generate_global_word(&key_words(&["挨拶"], &[])));
    assert!(global.is_empty(), "global: got {:?}", global);
    let local = gen_to_string(|cg| cg.generate_local_word(&key_words(&["挨拶"], &[])));
    assert!(local.is_empty(), "local: got {:?}", local);
}

/// Global words use `PASTA.create_word` (dot), local words use
/// `SCENE:create_word` (colon); multiple key names emit one line each
/// with the SAME entry list.
#[test]
fn word_definition_prefixes_and_multi_name_lines() {
    let kw = key_words(&["挨拶", "あいさつ"], &["おはよう", "こんにちは"]);

    let global = gen_to_string(|cg| cg.generate_global_word(&kw));
    assert_eq!(
        global,
        "PASTA.create_word(\"挨拶\"):entry(\"おはよう\", \"こんにちは\")\n\
         PASTA.create_word(\"あいさつ\"):entry(\"おはよう\", \"こんにちは\")\n"
    );

    let local = gen_to_string(|cg| cg.generate_local_word(&kw));
    assert_eq!(
        local,
        "SCENE:create_word(\"挨拶\"):entry(\"おはよう\", \"こんにちは\")\n\
         SCENE:create_word(\"あいさつ\"):entry(\"おはよう\", \"こんにちは\")\n"
    );
}

// ------------------------------------------------------------------
// Code blocks
// ------------------------------------------------------------------

/// A code block with an INVALID span (synthetic/headerless) still emits
/// its content lines but records NO source-map entries (sentinel path).
#[test]
fn code_block_with_invalid_span_emits_lines_without_records() {
    let mut sink = CapturingSink::default();
    let mut output = Vec::new();
    {
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        cg.set_source_map(&mut sink);
        cg.generate_code_block(&CodeBlock {
            language: Some("lua".to_string()),
            content: "local a = 1\nlocal b = 2".to_string(),
            span: Span::default(), // invalid: end_byte == 0
        })
        .unwrap();
    }
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "local a = 1\nlocal b = 2\n",
        "content must still be emitted verbatim"
    );
    assert!(
        sink.records.is_empty(),
        "invalid span must not pollute the source map, got {:?}",
        sink.records
    );
}

// ------------------------------------------------------------------
// Actor reference / global function call forms (runtime safety)
// ------------------------------------------------------------------

/// Every action arm emits ONE line that obtains the actor through
/// `act:actor_proxy("名前")` (name passed as a string literal), keeping the
/// arguments unchanged. Actor names colliding with act members / Lua
/// keywords (`talk`, `var`, `end`) produce the same form (Req 2.1, 2.2, 5.3, 5.4).
#[test]
fn action_arms_use_actor_proxy_with_string_name_for_any_actor() {
    let one = || Args {
        items: vec![Arg::Positional(Expr::Integer(1))],
        span: Span::default(),
    };
    for actor in ["さくら", "talk", "var", "end"] {
        let actions = vec![
            Action::Talk {
                text: "x".to_string(),
                span: Span::default(),
            },
            Action::WordRef {
                name: "w".to_string(),
                span: Span::default(),
            },
            Action::VarRef {
                name: "x".to_string(),
                scope: VarScope::Local,
                span: Span::default(),
            },
            Action::VarRef {
                name: "p".to_string(),
                scope: VarScope::Property,
                span: Span::default(),
            },
            Action::FnCall {
                name: "f".to_string(),
                args: Args::empty(),
                scope: FnScope::Local,
                span: Span::default(),
            },
            Action::FnCall {
                name: "g".to_string(),
                args: one(),
                scope: FnScope::Global,
                span: Span::default(),
            },
            Action::FnCall {
                name: "end".to_string(),
                args: Args::empty(),
                scope: FnScope::Global,
                span: Span::default(),
            },
            Action::SakuraScript {
                script: "\\n".to_string(),
                span: Span::default(),
            },
            Action::Escape {
                sequence: "@@".to_string(),
                span: Span::default(),
            },
            Action::DynamicWordRef {
                var_name: "w".to_string(),
                var_scope: VarScope::Local,
                span: Span::default(),
            },
            Action::DynamicFnCall {
                var_name: "f".to_string(),
                var_scope: VarScope::Local,
                args: one(),
                span: Span::default(),
            },
        ];
        let text = gen_to_string(|cg| {
            for a in &actions {
                cg.generate_action(a, actor)?;
            }
            Ok(())
        });
        let p = format!("act:actor_proxy(\"{}\")", actor);
        let expected = [
            format!("{p}:talk(\"x\")"),
            format!("{p}:talk({p}:word(\"w\"))"),
            format!("{p}:talk(var.x, \"var.x\")"),
            format!("{p}:talk(tostring(act:get_property(\"p\")))"),
            format!("{p}:talk(({p}:expr_fn(\"f\")))"),
            format!("{p}:talk((act:global_fn(\"g\", 1)))"),
            format!("{p}:talk((act:global_fn(\"end\")))"),
            format!("{p}:sakura_script([[\\n]])"),
            format!("{p}:talk(\"@\")"),
            format!("{p}:talk({p}:word(var.w, \"var.w\"))"),
            format!("{p}:talk(({p}:expr_fn_var(var.f, \"var.f\", 1)))"),
        ]
        .map(|l| l + "\n")
        .concat();
        assert_eq!(text, expected, "actor = {}", actor);
    }
}

// ------------------------------------------------------------------
// 式文 `＄＝式`（VarSet name=None）の生成形（expr-nil-coercion 1.4・2.4）
// ------------------------------------------------------------------

/// 式が関数呼び出しそのもの（`FnCall`・`DynamicFnCall`）なら現行どおり 1 行に書き、
/// それ以外（二項演算・括弧・リテラル・変数参照）は `do local _ = 式 end` で値を捨てる。
/// 後者を素の式で書くと Lua の文にならない（読み込みエラー、または直前の行の続きの呼び出し）。
#[test]
fn expr_statement_wraps_non_call_expressions_in_discard_block() {
    let stmt = |expr: Expr| {
        gen_to_string(|cg| {
            cg.generate_var_set(&VarSet {
                name: None,
                scope: VarScope::Local,
                value: SetValue::Expr(expr),
                span: Span::default(),
            })
        })
    };
    let f_call = || Expr::FnCall {
        name: "f".to_string(),
        args: Args::empty(),
        scope: FnScope::Local,
    };
    let cases: Vec<(Expr, &str)> = vec![
        // 関数呼び出し・動的関数呼び出しは 1 行のまま
        (f_call(), "act:expr_fn(\"f\")"),
        (
            Expr::FnCall {
                name: "g".to_string(),
                args: Args::empty(),
                scope: FnScope::Global,
            },
            "act:global_fn(\"g\")",
        ),
        (
            Expr::DynamicFnCall {
                var_name: "h".to_string(),
                var_scope: VarScope::Local,
                args: Args::empty(),
            },
            "act:expr_fn_var(var.h, \"var.h\")",
        ),
        // 二項演算
        (
            Expr::Binary {
                op: BinOp::Concat,
                lhs: Box::new(var_ref("未代入", VarScope::Local)),
                rhs: Box::new(Expr::String("x".to_string())),
            },
            "do local _ = (PASTA.str(var.未代入, \"var.未代入\") .. \"x\") end",
        ),
        // 括弧（中身が関数呼び出しでも括弧は呼び出しそのものではない）
        (
            Expr::Paren(Box::new(f_call())),
            "do local _ = (act:expr_fn(\"f\")) end",
        ),
        // リテラル
        (Expr::Integer(1), "do local _ = 1 end"),
        (Expr::String("s".to_string()), "do local _ = \"s\" end"),
        // 変数参照
        (var_ref("x", VarScope::Local), "do local _ = var.x end"),
        (var_ref("g", VarScope::Global), "do local _ = save.g end"),
    ];
    for (expr, expected) in cases {
        let label = format!("{:?}", expr);
        assert_eq!(stmt(expr), format!("{expected}\n"), "{label}");
    }
}
