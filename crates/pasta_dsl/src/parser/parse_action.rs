//! Action/expression parsing functions.

use super::*;

/// Parse call_scene.
pub(crate) fn parse_call_scene(pair: Pair<Rule>, filename: &str) -> Result<CallScene, ParseError> {
    let span = Span::from_pair(&pair);
    let mut target = None;
    let mut args = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                target = Some(CallTarget::Static(inner.as_str().to_string()));
            }
            Rule::call_target_expr => {
                if let Some(expr) = parse_expr_from_parts(inner, filename)? {
                    target = Some(CallTarget::Dynamic(expr));
                }
            }
            Rule::args => {
                args = Some(parse_args(inner, filename)?);
            }
            _ => {}
        }
    }

    Ok(CallScene {
        target: target.unwrap_or(CallTarget::Static(String::new())),
        args,
        span,
    })
}

/// Parse action_line.
pub(crate) fn parse_action_line(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<ActionLine, ParseError> {
    let span = Span::from_pair(&pair);
    let mut actor = String::new();
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                actor = inner.as_str().to_string();
            }
            Rule::actions => {
                actions = parse_actions(inner, filename)?;
            }
            _ => {}
        }
    }

    Ok(ActionLine {
        actor,
        actions,
        span,
    })
}

/// Parse continue_action_line.
pub(crate) fn parse_continue_action_line(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<ContinueAction, ParseError> {
    let span = Span::from_pair(&pair);
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::actions {
            actions = parse_actions(inner, filename)?;
        }
    }

    Ok(ContinueAction { actions, span })
}

/// Parse actions.
pub(crate) fn parse_actions(pair: Pair<Rule>, filename: &str) -> Result<Vec<Action>, ParseError> {
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        let action_span = Span::from_pair(&inner);
        match inner.as_rule() {
            Rule::talk => {
                actions.push(Action::Talk {
                    text: inner.as_str().to_string(),
                    span: action_span,
                });
            }
            Rule::word_ref => {
                for id_inner in inner.into_inner() {
                    if id_inner.as_rule() == Rule::id {
                        actions.push(Action::WordRef {
                            name: id_inner.as_str().to_string(),
                            span: action_span,
                        });
                    }
                }
            }
            Rule::var_ref_local => {
                if let Some((name, scope)) = parse_var_ref_local_inner(inner, filename)? {
                    actions.push(Action::VarRef {
                        name,
                        scope,
                        span: action_span,
                    });
                }
            }
            Rule::var_ref_global => {
                for id_inner in inner.into_inner() {
                    if id_inner.as_rule() == Rule::id {
                        actions.push(Action::VarRef {
                            name: id_inner.as_str().to_string(),
                            scope: VarScope::Global,
                            span: action_span,
                        });
                    }
                }
            }
            Rule::var_ref_property => {
                for id_inner in inner.into_inner() {
                    if id_inner.as_rule() == Rule::property_id {
                        actions.push(Action::VarRef {
                            name: id_inner.as_str().to_string(),
                            scope: VarScope::Property,
                            span: action_span,
                        });
                    }
                }
            }
            Rule::fn_call_local => {
                let (name, args) = parse_fn_call_inner(inner, filename)?;
                actions.push(Action::FnCall {
                    name,
                    args,
                    scope: FnScope::Local,
                    span: action_span,
                });
            }
            Rule::fn_call_global => {
                let (name, args) = parse_fn_call_inner(inner, filename)?;
                actions.push(Action::FnCall {
                    name,
                    args,
                    scope: FnScope::Global,
                    span: action_span,
                });
            }
            Rule::word_ref_dynamic => {
                if let Some((var_name, var_scope)) = parse_dyn_name(inner, filename)? {
                    actions.push(Action::DynamicWordRef {
                        var_name,
                        var_scope,
                        span: action_span,
                    });
                }
            }
            Rule::fn_call_dynamic => {
                let (var_name, var_scope, args) = parse_dyn_fn_call_inner(inner, filename)?;
                actions.push(Action::DynamicFnCall {
                    var_name,
                    var_scope,
                    args,
                    span: action_span,
                });
            }
            Rule::sakura_script => {
                actions.push(Action::SakuraScript {
                    script: inner.as_str().to_string(),
                    span: action_span,
                });
            }
            Rule::at_escape | Rule::dollar_escape | Rule::sakura_escape => {
                actions.push(Action::Escape {
                    sequence: inner.as_str().to_string(),
                    span: action_span,
                });
            }
            _ => {}
        }
    }

    Ok(actions)
}

/// Parse function call inner parts.
pub(crate) fn parse_fn_call_inner(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<(String, Args), ParseError> {
    let mut name = String::new();
    let mut args = Args::empty();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                name = inner.as_str().to_string();
            }
            Rule::args => {
                args = parse_args(inner, filename)?;
            }
            _ => {}
        }
    }

    Ok((name, args))
}

/// Extract `(var_name, var_scope)` from a pair containing `dyn_name_local` / `dyn_name_global`.
///
/// `dyn_name_local` splits into Local / Args(n) by the same rule as `var_ref_local`.
pub(crate) fn parse_dyn_name(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<Option<(String, VarScope)>, ParseError> {
    for inner in pair.into_inner() {
        let found = match inner.as_rule() {
            Rule::dyn_name_local => parse_var_ref_local_inner(inner, filename)?,
            Rule::dyn_name_global => inner
                .into_inner()
                .find(|id| id.as_rule() == Rule::id)
                .map(|id| (id.as_str().to_string(), VarScope::Global)),
            _ => None,
        };
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

/// Parse `fn_call_dynamic` into `(var_name, var_scope, args)`.
pub(crate) fn parse_dyn_fn_call_inner(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<(String, VarScope, Args), ParseError> {
    // The grammar requires a dyn_name; the fallback mirrors parse_fn_call_inner's empty name.
    let (var_name, var_scope) =
        parse_dyn_name(pair.clone(), filename)?.unwrap_or((String::new(), VarScope::Local));
    let args = match pair.into_inner().find(|p| p.as_rule() == Rule::args) {
        Some(args) => parse_args(args, filename)?,
        None => Args::empty(),
    };
    Ok((var_name, var_scope, args))
}

/// Parse args.
pub(crate) fn parse_args(pair: Pair<Rule>, filename: &str) -> Result<Args, ParseError> {
    let span = Span::from_pair(&pair);
    let mut items = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::positional_arg => {
                // Collect all terms and operators for this positional argument
                // Grammar: expr = term ~ bin*, where bin = bin_op ~ term
                // Since expr and bin are silent rules, we get term, op, term, op, term...
                if let Some(expr) = parse_expr_from_parts(inner, filename)? {
                    items.push(Arg::Positional(expr));
                }
            }
            Rule::key_arg => {
                let (key, value) = parse_key_arg(inner, filename)?;
                items.push(Arg::Keyword { key, value });
            }
            _ => {}
        }
    }

    Ok(Args { items, span })
}

/// Map a binary operator rule to its `BinOp`, or `None` for non-operator rules.
pub(crate) fn bin_op_from_rule(rule: Rule) -> Option<BinOp> {
    match rule {
        Rule::add_op => Some(BinOp::Add),
        Rule::sub_op => Some(BinOp::Sub),
        Rule::mul_op => Some(BinOp::Mul),
        Rule::div_op => Some(BinOp::Div),
        Rule::modulo_op => Some(BinOp::Mod),
        Rule::concat_op => Some(BinOp::Concat),
        _ => None,
    }
}

/// `pair` の子孫の二項演算子が MAX_EXPR_OPS を超えたら、
/// 超えた演算子の位置の SyntaxError を返す。
///
/// 式の鎖を組む前に呼び、深い木を作らせない。括弧と関数呼び出しの引数の中も同じ式として数える。
pub(crate) fn check_op_count(pair: &Pair<Rule>, filename: &str) -> Result<(), ParseError> {
    let mut ops = pair
        .clone()
        .into_inner()
        .flatten()
        .filter(|p| bin_op_from_rule(p.as_rule()).is_some());
    match ops.nth(MAX_EXPR_OPS) {
        Some(op) => Err(syntax_error(
            filename,
            op.line_col(),
            &format!("1 つの式の演算子が {MAX_EXPR_OPS} 個を超えています"),
        )),
        None => Ok(()),
    }
}

/// Parse an expression from parts (terms and binary operators).
/// Handles the case where expr = term ~ (bin_op ~ term)* expands into multiple pairs.
pub(crate) fn parse_expr_from_parts(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<Option<Expr>, ParseError> {
    check_op_count(&pair, filename)?;
    let mut terms: Vec<Expr> = Vec::new();
    let mut operators: Vec<BinOp> = Vec::new();

    for expr_inner in pair.into_inner() {
        if let Some(op) = bin_op_from_rule(expr_inner.as_rule()) {
            operators.push(op);
        } else if let Some(term) = try_parse_expr(expr_inner, filename)? {
            terms.push(term);
        }
    }

    if terms.is_empty() {
        return Ok(None);
    }

    Ok(Some(build_left_assoc_expr(terms, operators)))
}

/// Parse key_arg.
pub(crate) fn parse_key_arg(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<(String, Expr), ParseError> {
    check_op_count(&pair, filename)?;
    let mut key = String::new();
    let mut terms: Vec<Expr> = Vec::new();
    let mut operators: Vec<BinOp> = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::key_expr {
            for kv_inner in inner.into_inner() {
                match kv_inner.as_rule() {
                    Rule::id => {
                        // The first id is the key name
                        if key.is_empty() {
                            key = kv_inner.as_str().to_string();
                        }
                    }
                    rule => {
                        if let Some(op) = bin_op_from_rule(rule) {
                            operators.push(op);
                        } else if let Some(expr) = try_parse_expr(kv_inner, filename)? {
                            terms.push(expr);
                        }
                    }
                }
            }
        }
    }

    let value = if terms.is_empty() {
        Expr::BlankString
    } else {
        build_left_assoc_expr(terms, operators)
    };

    Ok((key, value))
}

/// Build a left-associative binary expression from terms and operators.
///
/// Callers guarantee `terms` is non-empty. Surplus operators or terms
/// (beyond the pairwise zip) are ignored, matching the grammar's
/// `term ~ (bin_op ~ term)*` shape.
pub(crate) fn build_left_assoc_expr(terms: Vec<Expr>, operators: Vec<BinOp>) -> Expr {
    let mut terms = terms.into_iter();
    let mut result = terms
        .next()
        .expect("build_left_assoc_expr requires at least one term");
    for (op, rhs) in operators.into_iter().zip(terms) {
        result = Expr::Binary {
            op,
            lhs: Box::new(result),
            rhs: Box::new(rhs),
        };
    }
    result
}

/// Try to parse an expression from a pair.
///
/// `Ok(None)` は「項にならない」。中の誤りは飲まずに `Err` で返す。
pub(crate) fn try_parse_expr(pair: Pair<Rule>, filename: &str) -> Result<Option<Expr>, ParseError> {
    let expr = match pair.as_rule() {
        Rule::number_literal => Some(match number_value(&pair, filename)? {
            Number::Integer(i) => Expr::Integer(i),
            Number::Float(f) => Expr::Float(f),
        }),
        Rule::string_contents => Some(Expr::String(pair.as_str().to_string())),
        Rule::string_blank => Some(Expr::BlankString),
        Rule::var_ref_local => parse_var_ref_local_inner(pair, filename)?
            .map(|(name, scope)| Expr::VarRef { name, scope }),
        Rule::var_ref_global => pair
            .into_inner()
            .find(|inner| inner.as_rule() == Rule::id)
            .map(|inner| Expr::VarRef {
                name: inner.as_str().to_string(),
                scope: VarScope::Global,
            }),
        Rule::var_ref_property => pair
            .into_inner()
            .find(|inner| inner.as_rule() == Rule::property_id)
            .map(|inner| Expr::VarRef {
                name: inner.as_str().to_string(),
                scope: VarScope::Property,
            }),
        Rule::fn_call_local => {
            let (name, args) = parse_fn_call_inner(pair, filename)?;
            Some(Expr::FnCall {
                name,
                args,
                scope: FnScope::Local,
            })
        }
        Rule::fn_call_global => {
            let (name, args) = parse_fn_call_inner(pair, filename)?;
            Some(Expr::FnCall {
                name,
                args,
                scope: FnScope::Global,
            })
        }
        Rule::fn_call_dynamic => {
            // Explicit arm: the default arm would recurse into args and return the first argument
            let (var_name, var_scope, args) = parse_dyn_fn_call_inner(pair, filename)?;
            Some(Expr::DynamicFnCall {
                var_name,
                var_scope,
                args,
            })
        }
        Rule::paren_expr => {
            parse_expr_from_parts(pair, filename)?.map(|e| Expr::Paren(Box::new(e)))
        }
        // 演算子は呼び出し元が bin_op_from_rule で先に分ける。項の位置に来る規則はすべて上で扱う
        rule => {
            return Err(syntax_error(
                filename,
                pair.line_col(),
                &format!("内部の誤り：式の項として扱えない規則 {rule:?}"),
            ));
        }
    };
    Ok(expr)
}

/// Extract variable name and scope from a `var_ref_local` pair.
///
/// Handles both named variables (`$var` → Local) and positional args (`$0` → Args(0)).
/// 0〜255 に収まらない番号は、番号の位置の構文の誤り。
fn parse_var_ref_local_inner(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<Option<(String, VarScope)>, ParseError> {
    for var_id_pair in pair.into_inner() {
        if var_id_pair.as_rule() == Rule::var_id {
            for id_inner in var_id_pair.into_inner() {
                match id_inner.as_rule() {
                    Rule::id => {
                        return Ok(Some((id_inner.as_str().to_string(), VarScope::Local)));
                    }
                    Rule::digit_id => {
                        // 文法が数字だけを通すので、失敗は範囲の外だけ
                        let index = normalize_number_str(id_inner.as_str())
                            .parse::<u8>()
                            .map_err(|_| {
                                syntax_error(
                                    filename,
                                    id_inner.line_col(),
                                    &format!(
                                        "シーン引数の番号 {} は 0〜255 の範囲を超えています",
                                        id_inner.as_str()
                                    ),
                                )
                            })?;
                        return Ok(Some((id_inner.as_str().to_string(), VarScope::Args(index))));
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(None)
}
