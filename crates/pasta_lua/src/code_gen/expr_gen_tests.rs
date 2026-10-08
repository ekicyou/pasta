use super::*;
use crate::config::LineEnding;
use pasta_dsl::parser::{Arg, FnScope, SetValue, Span, VarScope, VarSet};

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
// Expression generation (expr_text: expr_to_string; statements: VarSet name=None)
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
/// the exact Lua spellings (` + `, ` - `, ` * `, ` / `, ` % `), each
/// operation wrapped in its own parentheses.
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
        "do local _ = ((((((1 - 2)) * 3) / 4) % 5) + PASTA.num(\"+\", var.x, \"var.x\")) end\n"
    );

    let float_text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(Expr::Float(1.5))));
    assert_eq!(float_text, "do local _ = 1.5 end\n");

    let blank_text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(Expr::BlankString)));
    assert_eq!(blank_text, "do local _ = \"\" end\n");
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
    assert_eq!(text, "do local _ = args[3] end\n");
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
// Arithmetic: precedence regrouping into nested `(L op R)` (Req 1.2, 1.4, 4.5)
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

/// Render the expression itself (not an expression statement).
fn expr_text(expr: Expr) -> String {
    let mut output = Vec::new();
    let cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
    cg.expr_to_string(&expr).unwrap()
}

/// `＊／％` fold before `＋－`, each left to right; parens are one term.
#[test]
fn arith_regroups_left_assoc_tree_by_lua_precedence() {
    use pasta_dsl::parser::BinOp::{Add, Div, Mod, Mul, Sub};
    let cases: Vec<(Expr, &str)> = vec![
        // 1＋2＊3
        (
            left_assoc(int(1), vec![(Add, int(2)), (Mul, int(3))]),
            r#"(1 + (2 * 3))"#,
        ),
        // 1－2－3
        (
            left_assoc(int(1), vec![(Sub, int(2)), (Sub, int(3))]),
            r#"((1 - 2) - 3)"#,
        ),
        // （1＋2）＊3
        (
            left_assoc(
                paren(left_assoc(int(1), vec![(Add, int(2))])),
                vec![(Mul, int(3))],
            ),
            r#"(((1 + 2)) * 3)"#,
        ),
        // 1－2＊3－4
        (
            left_assoc(int(1), vec![(Sub, int(2)), (Mul, int(3)), (Sub, int(4))]),
            r#"((1 - (2 * 3)) - 4)"#,
        ),
        // 8／2／2
        (
            left_assoc(int(8), vec![(Div, int(2)), (Div, int(2))]),
            r#"((8 / 2) / 2)"#,
        ),
        // 1＋2＊3＋4＊5
        (
            left_assoc(
                int(1),
                vec![(Add, int(2)), (Mul, int(3)), (Add, int(4)), (Mul, int(5))],
            ),
            r#"((1 + (2 * 3)) + (4 * 5))"#,
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
            r#"((2 * ((3 + 4))) % 5)"#,
        ),
        // 10－－3 (negative literal stays a plain term; spaces keep `--` from becoming a comment)
        (left_assoc(int(10), vec![(Sub, int(-3))]), r#"(10 - -3)"#),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr_text(expr), expected);
    }
}

/// Operand descriptions: the third argument of `PASTA.num` as a string
/// literal, only when the operand has one (a paren passes its inner one through).
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
            r#"(PASTA.num("+", var.x, "var.x") + (PASTA.num("*", act:expr_fn("f"), "@f()") * 2))"#,
        ),
        // ＄x＋1
        (
            left_assoc(local_x(), vec![(Add, int(1))]),
            r#"(PASTA.num("+", var.x, "var.x") + 1)"#,
        ),
        // 1＋2＊＄y
        (
            left_assoc(
                int(1),
                vec![(Add, int(2)), (Mul, var("y", VarScope::Local))],
            ),
            r#"(1 + (2 * PASTA.num("*", var.y, "var.y")))"#,
        ),
        // （＄x＋1）＊2
        (
            left_assoc(
                paren(left_assoc(local_x(), vec![(Add, int(1))])),
                vec![(Mul, int(2))],
            ),
            r#"(((PASTA.num("+", var.x, "var.x") + 1)) * 2)"#,
        ),
        // ＄＊s＋＄1 (global / scene argument)
        (
            left_assoc(
                var("s", VarScope::Global),
                vec![(Add, var("1", VarScope::Args(1)))],
            ),
            r#"(PASTA.num("+", save.s, "save.s") + PASTA.num("+", args[2], "args[2]"))"#,
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
            r#"(PASTA.num("*", act:global_fn("g", 1), "@*g()") * PASTA.num("*", act:expr_fn_var(var.h, "var.h"), "@$var.h()"))"#,
        ),
        // （＄x）＋「a」: paren passes the inner description through; literal has none
        (
            left_assoc(paren(local_x()), vec![(Add, Expr::String("a".to_string()))]),
            r#"(PASTA.num("+", (var.x), "var.x") + PASTA.num("+", "a"))"#,
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
    assert_eq!(expr_text(expr), r#"act:expr_fn("f", (1 + (2 * 3)))"#);
}

/// A right-hand Binary (never produced by the parser) is one term; its kind is
/// the kind of its regrouped top operation.
#[test]
fn arith_right_hand_binary_is_one_term() {
    use pasta_dsl::parser::BinOp::{Add, Mul};
    let expr = Expr::Binary {
        op: Mul,
        lhs: Box::new(int(2)),
        rhs: Box::new(left_assoc(int(3), vec![(Add, int(4))])),
    };
    assert_eq!(expr_text(expr), r#"(2 * (3 + 4))"#);

    // 2＊（「a」＆1＋2）as a right-hand term: concat folds last, so it is a string
    let expr = Expr::Binary {
        op: Mul,
        lhs: Box::new(int(2)),
        rhs: Box::new(left_assoc(
            Expr::String("a".to_string()),
            vec![(pasta_dsl::parser::BinOp::Concat, int(1)), (Add, int(2))],
        )),
    };
    assert_eq!(
        expr_text(expr),
        r#"(2 * PASTA.num("*", ("a" .. PASTA.str((1 + 2)))))"#
    );
}

/// Arithmetic wraps every operand that is not statically a number with
/// `PASTA.num` (string literals, blank strings, concats, variables, calls);
/// number literals and arithmetic results stay bare (Req 1.2, 4.5).
#[test]
fn arith_wraps_operands_by_static_kind() {
    use pasta_dsl::parser::BinOp::{Add, Concat, Sub};
    let s = |t: &str| Expr::String(t.to_string());
    let cases: Vec<(Expr, &str)> = vec![
        // 1.5－2
        (
            left_assoc(Expr::Float(1.5), vec![(Sub, int(2))]),
            r#"(1.5 - 2)"#,
        ),
        // 「1」＋「」
        (
            left_assoc(s("1"), vec![(Add, Expr::BlankString)]),
            r#"(PASTA.num("+", "1") + PASTA.num("+", ""))"#,
        ),
        // （1＆2）－（3－4）: paren takes the kind of its content
        (
            left_assoc(
                paren(left_assoc(int(1), vec![(Concat, int(2))])),
                vec![(Sub, paren(left_assoc(int(3), vec![(Sub, int(4))])))],
            ),
            r#"(PASTA.num("-", ((PASTA.str(1) .. PASTA.str(2)))) - ((3 - 4)))"#,
        ),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr_text(expr), expected);
    }
}

// ------------------------------------------------------------------
// Concatenation: a third, lowest level below arithmetic (Req 1.3, 2.1–2.3, 3.3)
// ------------------------------------------------------------------

/// Transpile one line of a `メイン` scene and return its generated Lua line.
fn scene_line(line: &str) -> String {
    let source = format!("＊メイン\n　{}\n", line);
    let file = pasta_dsl::parser::parse_str(&source, "test.pasta").unwrap();
    let mut output = Vec::new();
    crate::LuaTranspiler::default()
        .transpile(&file, &mut output)
        .unwrap();
    let lua = String::from_utf8(output).unwrap();
    let mut lines = lua
        .lines()
        .map(str::trim)
        .skip_while(|l| !l.contains("act:init_scene"));
    lines
        .find(|l| !l.is_empty() && !l.contains("act:init_scene"))
        .unwrap()
        .to_string()
}

/// The design's generated forms, from DSL text through the parser
/// (assignment, Call target, and property-assignment right-hand sides).
#[test]
fn concat_generated_forms_from_dsl() {
    let cases = [
        (
            "＄表示＝「合計」＆＄n＆「個」",
            r#"var.表示 = (("合計" .. PASTA.str(var.n, "var.n")) .. "個")"#,
        ),
        (
            "＄s＝「合計」＆＄a＋＄b",
            r#"var.s = ("合計" .. PASTA.str((PASTA.num("+", var.a, "var.a") + PASTA.num("+", var.b, "var.b"))))"#,
        ),
        (
            "＄n＝（「1」＆「2」）＋1",
            r#"var.n = (PASTA.num("+", (("1" .. "2"))) + 1)"#,
        ),
        (
            "＞＄種類＆「_挨拶」",
            r#"return act:call(SCENE.__global_name__, act:call_key((PASTA.str(var.種類, "var.種類") .. "_挨拶")), {}, table.unpack(args))"#,
        ),
        (
            "＄＊回数＝＄＊回数＋1",
            r#"save.回数 = (PASTA.num("+", save.回数, "save.回数") + 1)"#,
        ),
        (
            "＄b＝1＋2＊＄y",
            r#"var.b = (1 + (2 * PASTA.num("*", var.y, "var.y")))"#,
        ),
        (
            "＄c＝（＄x＋1）＊2",
            r#"var.c = (((PASTA.num("+", var.x, "var.x") + 1)) * 2)"#,
        ),
        ("＄n＝「1」＋2", r#"var.n = (PASTA.num("+", "1") + 2)"#),
        (
            "＄z＝＠＄f（）＋1",
            r#"var.z = (PASTA.num("+", act:expr_fn_var(var.f, "var.f"), "@$var.f()") + 1)"#,
        ),
        (
            "＄＊g＝＠関数（2＋1）",
            r#"save.g = act:expr_fn("関数", (2 + 1))"#,
        ),
        (
            "＄％p＝＄x＋1",
            r#"act:set_property("p", (PASTA.num("+", var.x, "var.x") + 1))"#,
        ),
        (
            "＄％p＝「a」＆＄x",
            r#"act:set_property("p", ("a" .. PASTA.str(var.x, "var.x")))"#,
        ),
    ];
    for (dsl, expected) in cases {
        assert_eq!(scene_line(dsl), expected, "{dsl}");
    }
}

/// `＊／％`, then `＋－`, then `＆`, each left to right.
#[test]
fn concat_regroups_below_arithmetic() {
    use pasta_dsl::parser::BinOp::{Add, Concat, Mul};
    let s = |t: &str| Expr::String(t.to_string());
    let cases: Vec<(Expr, &str)> = vec![
        // 「x」＆1＋2＊3
        (
            left_assoc(s("x"), vec![(Concat, int(1)), (Add, int(2)), (Mul, int(3))]),
            r#"("x" .. PASTA.str((1 + (2 * 3))))"#,
        ),
        // 1＋2＆3＊4
        (
            left_assoc(int(1), vec![(Add, int(2)), (Concat, int(3)), (Mul, int(4))]),
            r#"(PASTA.str((1 + 2)) .. PASTA.str((3 * 4)))"#,
        ),
        // 「a」＆「b」＆「c」
        (
            left_assoc(s("a"), vec![(Concat, s("b")), (Concat, s("c"))]),
            r#"(("a" .. "b") .. "c")"#,
        ),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr_text(expr), expected);
    }
}

/// Concat leaves string literals, blank strings, and concat results bare and
/// wraps numbers, arithmetic results, variables, and calls with `PASTA.str`;
/// the description is the second argument, only when there is one
/// (a parenthesized variable keeps it, nested operations have none).
#[test]
fn concat_operand_descriptions() {
    use pasta_dsl::parser::BinOp::{Add, Concat};
    let s = |t: &str| Expr::String(t.to_string());
    let cases: Vec<(Expr, &str)> = vec![
        // 「a」＆＄x
        (
            left_assoc(s("a"), vec![(Concat, var("x", VarScope::Local))]),
            r#"("a" .. PASTA.str(var.x, "var.x"))"#,
        ),
        // （＄＊g）＆「a」
        (
            left_assoc(paren(var("g", VarScope::Global)), vec![(Concat, s("a"))]),
            r#"(PASTA.str((save.g), "save.g") .. "a")"#,
        ),
        // ＄x＋1＆（＄y＆「a」）
        (
            left_assoc(
                var("x", VarScope::Local),
                vec![
                    (Add, int(1)),
                    (
                        Concat,
                        paren(left_assoc(
                            var("y", VarScope::Local),
                            vec![(Concat, s("a"))],
                        )),
                    ),
                ],
            ),
            r#"(PASTA.str((PASTA.num("+", var.x, "var.x") + 1)) .. ((PASTA.str(var.y, "var.y") .. "a")))"#,
        ),
        // 「a」＆1＆「」
        (
            left_assoc(s("a"), vec![(Concat, int(1)), (Concat, Expr::BlankString)]),
            r#"(("a" .. PASTA.str(1)) .. "")"#,
        ),
        // 1.5＆＠f（）
        (
            left_assoc(
                Expr::Float(1.5),
                vec![(
                    Concat,
                    Expr::FnCall {
                        name: "f".to_string(),
                        args: Args::empty(),
                        scope: FnScope::Local,
                    },
                )],
            ),
            r#"(PASTA.str(1.5) .. PASTA.str(act:expr_fn("f"), "@f()"))"#,
        ),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr_text(expr), expected);
    }
}
