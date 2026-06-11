//! Action/expression parsing functions.

use super::*;

/// Parse call_scene.
pub(crate) fn parse_call_scene(pair: Pair<Rule>) -> Result<CallScene, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut target = None;
    let mut args = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                target = Some(CallTarget::Static(inner.as_str().to_string()));
            }
            Rule::call_target_expr => {
                if let Some(expr) = parse_expr_from_parts(inner) {
                    target = Some(CallTarget::Dynamic(expr));
                }
            }
            Rule::args => {
                args = Some(parse_args(inner)?);
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
pub(crate) fn parse_action_line(pair: Pair<Rule>) -> Result<ActionLine, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut actor = String::new();
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                actor = inner.as_str().to_string();
            }
            Rule::actions => {
                actions = parse_actions(inner)?;
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
pub(crate) fn parse_continue_action_line(pair: Pair<Rule>) -> Result<ContinueAction, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::actions {
            actions = parse_actions(inner)?;
        }
    }

    Ok(ContinueAction { actions, span })
}

/// Parse actions.
pub(crate) fn parse_actions(pair: Pair<Rule>) -> Result<Vec<Action>, ParseError> {
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        let action_span = Span::from(&inner.as_span());
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
                if let Some((name, scope)) = parse_var_ref_local_inner(inner) {
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
                let (name, args) = parse_fn_call_inner(inner)?;
                actions.push(Action::FnCall {
                    name,
                    args,
                    scope: FnScope::Local,
                    span: action_span,
                });
            }
            Rule::fn_call_global => {
                let (name, args) = parse_fn_call_inner(inner)?;
                actions.push(Action::FnCall {
                    name,
                    args,
                    scope: FnScope::Global,
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
pub(crate) fn parse_fn_call_inner(pair: Pair<Rule>) -> Result<(String, Args), ParseError> {
    let mut name = String::new();
    let mut args = Args::empty();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                name = inner.as_str().to_string();
            }
            Rule::args => {
                args = parse_args(inner)?;
            }
            _ => {}
        }
    }

    Ok((name, args))
}

/// Parse args.
pub(crate) fn parse_args(pair: Pair<Rule>) -> Result<Args, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut items = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::positional_arg => {
                // Collect all terms and operators for this positional argument
                // Grammar: expr = term ~ bin*, where bin = bin_op ~ term
                // Since expr and bin are silent rules, we get term, op, term, op, term...
                if let Some(expr) = parse_expr_from_parts(inner) {
                    items.push(Arg::Positional(expr));
                }
            }
            Rule::key_arg => {
                let (key, value) = parse_key_arg(inner)?;
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
        _ => None,
    }
}

/// Parse an expression from parts (terms and binary operators).
/// Handles the case where expr = term ~ (bin_op ~ term)* expands into multiple pairs.
pub(crate) fn parse_expr_from_parts(pair: Pair<Rule>) -> Option<Expr> {
    let mut terms: Vec<Expr> = Vec::new();
    let mut operators: Vec<BinOp> = Vec::new();

    for expr_inner in pair.into_inner() {
        if let Some(op) = bin_op_from_rule(expr_inner.as_rule()) {
            operators.push(op);
        } else if let Some(term) = try_parse_expr(expr_inner) {
            terms.push(term);
        }
    }

    if terms.is_empty() {
        return None;
    }

    Some(build_left_assoc_expr(terms, operators))
}

/// Parse key_arg.
pub(crate) fn parse_key_arg(pair: Pair<Rule>) -> Result<(String, Expr), ParseError> {
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
                        } else if let Some(expr) = try_parse_expr(kv_inner) {
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
pub(crate) fn try_parse_expr(pair: Pair<Rule>) -> Option<Expr> {
    match pair.as_rule() {
        Rule::number_literal => {
            let normalized = normalize_number_str(pair.as_str());
            if normalized.contains('.') {
                Some(Expr::Float(normalized.parse().ok()?))
            } else {
                Some(Expr::Integer(normalized.parse().ok()?))
            }
        }
        Rule::string_contents => Some(Expr::String(pair.as_str().to_string())),
        Rule::string_blank => Some(Expr::BlankString),
        Rule::var_ref_local => {
            parse_var_ref_local_inner(pair).map(|(name, scope)| Expr::VarRef { name, scope })
        }
        Rule::var_ref_global => {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::id {
                    return Some(Expr::VarRef {
                        name: inner.as_str().to_string(),
                        scope: VarScope::Global,
                    });
                }
            }
            None
        }
        Rule::var_ref_property => {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::property_id {
                    return Some(Expr::VarRef {
                        name: inner.as_str().to_string(),
                        scope: VarScope::Property,
                    });
                }
            }
            None
        }
        Rule::fn_call_local => {
            let (name, args) = parse_fn_call_inner(pair).ok()?;
            Some(Expr::FnCall {
                name,
                args,
                scope: FnScope::Local,
            })
        }
        Rule::fn_call_global => {
            let (name, args) = parse_fn_call_inner(pair).ok()?;
            Some(Expr::FnCall {
                name,
                args,
                scope: FnScope::Global,
            })
        }
        Rule::paren_expr => {
            for inner in pair.into_inner() {
                if let Some(expr) = try_parse_expr(inner) {
                    return Some(Expr::Paren(Box::new(expr)));
                }
            }
            None
        }
        Rule::add_op | Rule::sub_op | Rule::mul_op | Rule::div_op | Rule::modulo_op => {
            // Binary operators are handled at a higher level
            None
        }
        _ => {
            // Try recursively for nested expressions
            for inner in pair.into_inner() {
                if let Some(expr) = try_parse_expr(inner) {
                    return Some(expr);
                }
            }
            None
        }
    }
}

/// Extract variable name and scope from a `var_ref_local` pair.
///
/// Handles both named variables (`$var` → Local) and positional args (`$0` → Args(0)).
fn parse_var_ref_local_inner(pair: Pair<Rule>) -> Option<(String, VarScope)> {
    for var_id_pair in pair.into_inner() {
        if var_id_pair.as_rule() == Rule::var_id {
            for id_inner in var_id_pair.into_inner() {
                match id_inner.as_rule() {
                    Rule::id => {
                        return Some((id_inner.as_str().to_string(), VarScope::Local));
                    }
                    Rule::digit_id => {
                        let index = normalize_number_str(id_inner.as_str())
                            .parse::<u8>()
                            .unwrap_or(0);
                        return Some((id_inner.as_str().to_string(), VarScope::Args(index)));
                    }
                    _ => {}
                }
            }
        }
    }
    None
}
