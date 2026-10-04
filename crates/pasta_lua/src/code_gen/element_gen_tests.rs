use super::*;
use crate::code_gen::source_map::SourceMapSink;
use crate::config::LineEnding;
use pasta_dsl::parser::{Arg, CallTarget, FnScope};

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

/// Static call without args forwards only `table.unpack(args)`.
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
        "act:call(SCENE.__global_name__, \"次シーン\", {}, table.unpack(args))\n"
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
        "act:call(SCENE.__global_name__, \"次\", {}, table.unpack(args))\n"
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
        "act:call(SCENE.__global_name__, \"次\", {}, 1, \"さくら\", table.unpack(args))\n"
    );
}

/// Dynamic target evaluates the expression and wraps it in `tostring(...)`.
#[test]
fn call_scene_dynamic_target_wraps_expr_in_tostring() {
    let text = gen_to_string(|cg| {
        cg.generate_call_scene(
            &call_scene(
                CallTarget::Dynamic(Expr::VarRef {
                    name: "行き先".to_string(),
                    scope: VarScope::Local,
                }),
                None,
            ),
            false,
        )
    });
    assert_eq!(
        text,
        "act:call(SCENE.__global_name__, tostring(var.行き先), {}, table.unpack(args))\n"
    );
}

/// Tail call prepends `return ` (Lua TCO).
#[test]
fn call_scene_tail_call_prepends_return() {
    let text = gen_to_string(|cg| {
        cg.generate_call_scene(
            &call_scene(CallTarget::Static("次".to_string()), None),
            true,
        )
    });
    assert!(
        text.starts_with("return act:call("),
        "tail call must start with 'return ', got: {}",
        text
    );
}

// ------------------------------------------------------------------
// Expression generation (via expression statements: VarSet name=None)
// ------------------------------------------------------------------

fn expr_stmt(expr: Expr) -> VarSet {
    VarSet {
        name: None,
        scope: VarScope::Local,
        value: SetValue::Expr(expr),
        span: Span::default(),
    }
}

/// Float, blank-string, paren, and all five binary operators render with
/// the exact Lua spellings (` + `, ` - `, ` * `, ` / `, ` % `).
#[test]
fn expr_renders_float_blank_string_paren_and_all_binary_ops() {
    use pasta_dsl::parser::BinOp;
    // ((1 - 2)) * 3 / 4 % 5 + var.x  (left-nested to exercise every op)
    let expr = Expr::Binary {
        op: BinOp::Add,
        lhs: Box::new(Expr::Binary {
            op: BinOp::Mod,
            lhs: Box::new(Expr::Binary {
                op: BinOp::Div,
                lhs: Box::new(Expr::Binary {
                    op: BinOp::Mul,
                    lhs: Box::new(Expr::Paren(Box::new(Expr::Binary {
                        op: BinOp::Sub,
                        lhs: Box::new(Expr::Integer(1)),
                        rhs: Box::new(Expr::Integer(2)),
                    }))),
                    rhs: Box::new(Expr::Integer(3)),
                }),
                rhs: Box::new(Expr::Integer(4)),
            }),
            rhs: Box::new(Expr::Integer(5)),
        }),
        rhs: Box::new(Expr::VarRef {
            name: "x".to_string(),
            scope: VarScope::Local,
        }),
    };
    let text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(expr)));
    assert_eq!(
        text,
        "act:arith(\"+\", act:arith(\"%\", act:arith(\"/\", act:arith(\"*\", \
         (act:arith(\"-\", 1, 2)), 3), 4), 5), var.x, nil, \"var.x\")\n"
    );

    let float_text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(Expr::Float(1.5))));
    assert_eq!(float_text, "1.5\n");

    let blank_text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(Expr::BlankString)));
    assert_eq!(blank_text, "\"\"\n");
}

/// Args-scope variable references convert 0-based AST index to 1-based
/// Lua index in expression position (`Args(2)` -> `args[3]`).
#[test]
fn expr_args_var_ref_converts_to_one_based_lua_index() {
    let text = gen_to_string(|cg| {
        cg.generate_var_set(&expr_stmt(Expr::VarRef {
            name: "2".to_string(),
            scope: VarScope::Args(2),
        }))
    });
    assert_eq!(text, "args[3]\n");
}

/// Local fn call in expression position uses `act:expr_fn("name", ...)`;
/// global fn call uses `act:global_fn("name", ...)`.
#[test]
fn expr_fn_call_local_and_global_spellings() {
    let local_text = gen_to_string(|cg| {
        cg.generate_var_set(&expr_stmt(Expr::FnCall {
            name: "時刻".to_string(),
            args: Args::empty(),
            scope: FnScope::Local,
        }))
    });
    assert_eq!(local_text, "act:expr_fn(\"時刻\")\n");

    let global_text = gen_to_string(|cg| {
        cg.generate_var_set(&expr_stmt(Expr::FnCall {
            name: "rand".to_string(),
            args: Args {
                items: vec![Arg::Positional(Expr::Integer(6))],
                span: Span::default(),
            },
            scope: FnScope::Global,
        }))
    });
    assert_eq!(global_text, "act:global_fn(\"rand\", 6)\n");
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

/// `＠＊名前（…）` in expression position calls `act:global_fn("名前", …)`
/// (first arg stays act, implicitly via the method call) (Req 1.2).
#[test]
fn expr_global_fn_call_uses_act_global_fn() {
    let no_args = gen_to_string(|cg| {
        cg.generate_var_set(&expr_stmt(Expr::FnCall {
            name: "end".to_string(),
            args: Args::empty(),
            scope: FnScope::Global,
        }))
    });
    assert_eq!(no_args, "act:global_fn(\"end\")\n");
}

// ------------------------------------------------------------------
// Arithmetic: precedence regrouping into nested act:arith (Req 3.1, 3.2, 3.6)
// ------------------------------------------------------------------

/// Build a left-associative chain exactly like the parser's
/// `build_left_assoc_expr` (no precedence): `t0 op0 t1 op1 t2 ...`.
fn left_assoc(first: Expr, rest: Vec<(pasta_dsl::parser::BinOp, Expr)>) -> Expr {
    rest.into_iter().fold(first, |lhs, (op, rhs)| Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn int(n: i64) -> Expr {
    Expr::Integer(n)
}

fn var(name: &str, scope: VarScope) -> Expr {
    Expr::VarRef {
        name: name.to_string(),
        scope,
    }
}

fn paren(e: Expr) -> Expr {
    Expr::Paren(Box::new(e))
}

fn expr_text(expr: Expr) -> String {
    gen_to_string(|cg| cg.generate_var_set(&expr_stmt(expr)))
        .trim_end()
        .to_string()
}

/// `＊／％` fold before `＋－`, each left to right; parens are one term.
#[test]
fn arith_regroups_left_assoc_tree_by_lua_precedence() {
    use pasta_dsl::parser::BinOp::{Add, Div, Mod, Mul, Sub};
    let cases: Vec<(Expr, &str)> = vec![
        // 1＋2＊3
        (
            left_assoc(int(1), vec![(Add, int(2)), (Mul, int(3))]),
            r#"act:arith("+", 1, act:arith("*", 2, 3))"#,
        ),
        // 1－2－3
        (
            left_assoc(int(1), vec![(Sub, int(2)), (Sub, int(3))]),
            r#"act:arith("-", act:arith("-", 1, 2), 3)"#,
        ),
        // （1＋2）＊3
        (
            left_assoc(
                paren(left_assoc(int(1), vec![(Add, int(2))])),
                vec![(Mul, int(3))],
            ),
            r#"act:arith("*", (act:arith("+", 1, 2)), 3)"#,
        ),
        // 1－2＊3－4
        (
            left_assoc(int(1), vec![(Sub, int(2)), (Mul, int(3)), (Sub, int(4))]),
            r#"act:arith("-", act:arith("-", 1, act:arith("*", 2, 3)), 4)"#,
        ),
        // 8／2／2
        (
            left_assoc(int(8), vec![(Div, int(2)), (Div, int(2))]),
            r#"act:arith("/", act:arith("/", 8, 2), 2)"#,
        ),
        // 1＋2＊3＋4＊5
        (
            left_assoc(
                int(1),
                vec![(Add, int(2)), (Mul, int(3)), (Add, int(4)), (Mul, int(5))],
            ),
            r#"act:arith("+", act:arith("+", 1, act:arith("*", 2, 3)), act:arith("*", 4, 5))"#,
        ),
        // 2＊（3＋4）％5
        (
            left_assoc(
                int(2),
                vec![
                    (Mul, paren(left_assoc(int(3), vec![(Add, int(4))]))),
                    (Mod, int(5)),
                ],
            ),
            r#"act:arith("%", act:arith("*", 2, (act:arith("+", 3, 4))), 5)"#,
        ),
        // 10－－3 (negative literal stays a plain term)
        (
            left_assoc(int(10), vec![(Sub, int(-3))]),
            r#"act:arith("-", 10, -3)"#,
        ),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr_text(expr), expected);
    }
}

/// Operand descriptions: emitted as string literals, omitted when both are
/// absent, and `nil` stands in for a missing left description.
#[test]
fn arith_operand_descriptions() {
    use pasta_dsl::parser::BinOp::{Add, Mul};
    let local_x = || var("x", VarScope::Local);
    let f_call = Expr::FnCall {
        name: "f".to_string(),
        args: Args::empty(),
        scope: FnScope::Local,
    };
    let cases: Vec<(Expr, &str)> = vec![
        // ＄x＋＠f（）＊2
        (
            left_assoc(local_x(), vec![(Add, f_call.clone()), (Mul, int(2))]),
            r#"act:arith("+", var.x, act:arith("*", act:expr_fn("f"), 2, "@f()"), "var.x")"#,
        ),
        // ＄x＋1
        (
            left_assoc(local_x(), vec![(Add, int(1))]),
            r#"act:arith("+", var.x, 1, "var.x")"#,
        ),
        // 1＋2＊＄y
        (
            left_assoc(
                int(1),
                vec![(Add, int(2)), (Mul, var("y", VarScope::Local))],
            ),
            r#"act:arith("+", 1, act:arith("*", 2, var.y, nil, "var.y"))"#,
        ),
        // （＄x＋1）＊2
        (
            left_assoc(
                paren(left_assoc(local_x(), vec![(Add, int(1))])),
                vec![(Mul, int(2))],
            ),
            r#"act:arith("*", (act:arith("+", var.x, 1, "var.x")), 2)"#,
        ),
        // ＄＊s＋＄1 (global / scene argument)
        (
            left_assoc(
                var("s", VarScope::Global),
                vec![(Add, var("1", VarScope::Args(1)))],
            ),
            r#"act:arith("+", save.s, args[2], "save.s", "args[2]")"#,
        ),
        // ＠＊g（1）＊＠＄h（）
        (
            left_assoc(
                Expr::FnCall {
                    name: "g".to_string(),
                    args: Args {
                        items: vec![Arg::Positional(int(1))],
                        span: Span::default(),
                    },
                    scope: FnScope::Global,
                },
                vec![(
                    Mul,
                    Expr::DynamicFnCall {
                        var_name: "h".to_string(),
                        var_scope: VarScope::Local,
                        args: Args::empty(),
                    },
                )],
            ),
            r#"act:arith("*", act:global_fn("g", 1), act:expr_fn_var(var.h, "var.h"), "@*g()", "@$var.h()")"#,
        ),
        // （＄x）＋「a」: paren passes the inner description through; literal has none
        (
            left_assoc(paren(local_x()), vec![(Add, Expr::String("a".to_string()))]),
            r#"act:arith("+", (var.x), "a", "var.x")"#,
        ),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr_text(expr), expected);
    }
}

/// Arithmetic inside a function argument goes through the same generator
/// (every expression position shares `generate_expr_to_buffer`, Req 3.6).
#[test]
fn arith_in_function_argument_uses_same_generator() {
    use pasta_dsl::parser::BinOp::{Add, Mul};
    let expr = Expr::FnCall {
        name: "f".to_string(),
        args: Args {
            items: vec![Arg::Positional(left_assoc(
                int(1),
                vec![(Add, int(2)), (Mul, int(3))],
            ))],
            span: Span::default(),
        },
        scope: FnScope::Local,
    };
    assert_eq!(
        expr_text(expr),
        r#"act:expr_fn("f", act:arith("+", 1, act:arith("*", 2, 3)))"#
    );
}

/// A right-hand Binary (never produced by the parser) is one term.
#[test]
fn arith_right_hand_binary_is_one_term() {
    use pasta_dsl::parser::BinOp::{Add, Mul};
    let expr = Expr::Binary {
        op: Mul,
        lhs: Box::new(int(2)),
        rhs: Box::new(left_assoc(int(3), vec![(Add, int(4))])),
    };
    assert_eq!(
        expr_text(expr),
        r#"act:arith("*", 2, act:arith("+", 3, 4))"#
    );
}
