//! Parser module for Pasta DSL.
//!
//! This module provides parsing functionality for the Pasta DSL using pest (PEG parser).
//! The parser converts DSL source code into an Abstract Syntax Tree (AST) representation.

mod ast;

pub use ast::*;

use crate::error::PastaError;
use pest::Parser as PestParser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::path::Path;

#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
pub struct PastaParser;

/// Parse a Pasta script file
pub fn parse_file(path: &Path) -> Result<PastaFile, PastaError> {
    let source = std::fs::read_to_string(path)?;
    parse_str(&source, path.to_string_lossy().as_ref())
}

/// Parse a Pasta script from a string
pub fn parse_str(source: &str, filename: &str) -> Result<PastaFile, PastaError> {
    let mut pairs = PastaParser::parse(Rule::file, source)
        .map_err(|e| PastaError::PestError(format!("Parse error in {}: {}", filename, e)))?;

    let file_pair = pairs.next().unwrap(); // file rule always produces one pair
    let mut global_words = Vec::new();
    let mut labels = Vec::new();

    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::top_level_line => {
                // top_level_lineの内部を処理
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::global_word_def => {
                            global_words.push(parse_word_def(inner_pair)?);
                        }
                        Rule::global_label => {
                            labels.push(parse_global_label(inner_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            Rule::global_word_def => {
                global_words.push(parse_word_def(pair)?);
            }
            Rule::global_label => {
                labels.push(parse_global_label(pair)?);
            }
            Rule::EOI => {} // End of input, ignore
            _ => {}
        }
    }

    let span = Span::new(1, 1, source.lines().count(), source.len());

    Ok(PastaFile {
        path: Path::new(filename).to_path_buf(),
        global_words,
        labels,
        span,
    })
}

fn parse_word_def(pair: Pair<Rule>) -> Result<WordDef, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut values = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::word_name => {
                name = inner_pair.as_str().to_string();
            }
            Rule::word_value_list => {
                // word_value_list is atomic, so we need to split manually
                let list_str = inner_pair.as_str();
                // Split by full-width space (　) or tab
                values = list_str
                    .split(|c| c == '　' || c == '\t')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => {}
        }
    }

    Ok(WordDef { name, values, span })
}

fn parse_global_label(pair: Pair<Rule>) -> Result<LabelDef, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut attributes = Vec::new();
    let mut local_words = Vec::new();
    let mut local_labels = Vec::new();
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label_name => {
                name = inner_pair.as_str().to_string();
                // Validate reserved label pattern: __*__ is reserved for system use
                if name.starts_with("__") && name.ends_with("__") {
                    return Err(PastaError::ParseError {
                        file: "<input>".to_string(),
                        line: start.0,
                        column: start.1,
                        message: format!(
                            "Label name '{}' is reserved for system use. \
                            Label names starting and ending with '__' are not allowed. \
                            Consider using '{}' or '_{}_' instead.",
                            name,
                            name.trim_start_matches('_').trim_end_matches('_'),
                            name.trim_matches('_')
                        ),
                    });
                }
            }
            Rule::label_body_line => {
                // label_body_line contains indented content
                for content_pair in inner_pair.into_inner() {
                    match content_pair.as_rule() {
                        Rule::comment_marker | Rule::comment_content => {
                            // Skip comments
                        }
                        Rule::rune_start => {
                            // Skip, handled by rune_block_content
                        }
                        Rule::rune_block_content => {
                            statements.push(parse_rune_block_content(content_pair)?);
                        }
                        Rule::at_marker => {
                            // Skip marker
                        }
                        Rule::word_def_content => {
                            local_words.push(parse_word_def_from_parts(content_pair)?);
                        }
                        Rule::amp_marker => {
                            // Skip marker
                        }
                        Rule::attribute_content => {
                            attributes.push(parse_attribute_from_parts(content_pair)?);
                        }
                        Rule::dollar_marker => {
                            // Skip marker
                        }
                        Rule::var_assign_content => {
                            statements.push(parse_var_assign_content(content_pair)?);
                        }
                        Rule::local_label_marker => {
                            // Skip marker
                        }
                        Rule::local_label_content => {
                            local_labels.push(parse_local_label_content(content_pair)?);
                        }
                        Rule::call_marker => {
                            // Skip marker
                        }
                        Rule::call_content => {
                            statements.push(parse_call_content(content_pair)?);
                        }
                        // Phase 1 (REQ-BC-1): Jump removed - use Call instead
                        // Rule::jump_marker => {}
                        // Rule::jump_content => { statements.push(parse_jump_content(content_pair)?); }
                        Rule::speech_line_content => {
                            statements.push(parse_speech_line_content(content_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(LabelDef {
        name,
        scope: LabelScope::Global,
        params: Vec::new(), // Global labels don't have params
        attributes,
        local_words,
        local_labels,
        statements,
        span,
    })
}

fn parse_local_label_content(pair: Pair<Rule>) -> Result<LabelDef, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut params = Vec::new();
    let mut attributes = Vec::new();
    let mut local_words = Vec::new();
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label_name => {
                name = inner_pair.as_str().to_string();
                // Validate reserved label pattern: __*__ is reserved for system use
                if name.starts_with("__") && name.ends_with("__") {
                    return Err(PastaError::ParseError {
                        file: "<input>".to_string(),
                        line: start.0,
                        column: start.1,
                        message: format!(
                            "Label name '{}' is reserved for system use. \
                            Label names starting and ending with '__' are not allowed. \
                            Consider using '{}' or '_{}_' instead.",
                            name,
                            name.trim_start_matches('_').trim_end_matches('_'),
                            name.trim_matches('_')
                        ),
                    });
                }
            }
            Rule::label_params => {
                // Parse parameters: ＄値, ＄メッセージ, etc.
                for param_pair in inner_pair.into_inner() {
                    if param_pair.as_rule() == Rule::dollar_marker {
                        continue; // Skip the $ marker
                    } else if param_pair.as_rule() == Rule::var_name {
                        params.push(param_pair.as_str().to_string());
                    }
                }
            }
            Rule::local_label_body_line => {
                for content_pair in inner_pair.into_inner() {
                    match content_pair.as_rule() {
                        Rule::comment_marker | Rule::comment_content => {}
                        Rule::rune_start => {}
                        Rule::rune_block_content => {
                            statements.push(parse_rune_block_content(content_pair)?);
                        }
                        Rule::at_marker => {}
                        Rule::word_def_content => {
                            local_words.push(parse_word_def_from_parts(content_pair)?);
                        }
                        Rule::amp_marker => {}
                        Rule::attribute_content => {
                            attributes.push(parse_attribute_from_parts(content_pair)?);
                        }
                        Rule::dollar_marker => {}
                        Rule::var_assign_content => {
                            statements.push(parse_var_assign_content(content_pair)?);
                        }
                        Rule::call_marker => {}
                        Rule::call_content => {
                            statements.push(parse_call_content(content_pair)?);
                        }
                        // Phase 1 (REQ-BC-1): Jump removed - use Call instead
                        // Rule::jump_marker => {}
                        // Rule::jump_content => { statements.push(parse_jump_content(content_pair)?); }
                        Rule::speech_line_content => {
                            statements.push(parse_speech_line_content(content_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(LabelDef {
        name,
        scope: LabelScope::Local,
        params,
        attributes,
        local_words,
        local_labels: Vec::new(),
        statements,
        span,
    })
}

fn parse_var_assign_content(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut scope = VarScope::Local;
    let mut value = Expr::Literal(Literal::Number(0.0));

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::var_scope => {
                scope = VarScope::Global;
            }
            Rule::var_name => {
                name = inner_pair.as_str().to_string();
            }
            Rule::expr => {
                value = parse_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::VarAssign {
        name,
        scope,
        value,
        span,
    })
}

fn parse_word_def_from_parts(pair: Pair<Rule>) -> Result<WordDef, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut values = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::word_name => {
                name = inner_pair.as_str().to_string();
            }
            Rule::word_value_list => {
                let list_str = inner_pair.as_str();
                values = list_str
                    .split(|c| c == '　' || c == '\t')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => {}
        }
    }

    Ok(WordDef { name, values, span })
}

fn parse_attribute_from_parts(pair: Pair<Rule>) -> Result<Attribute, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut key = String::new();
    let mut value = AttributeValue::Literal(String::new());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::attribute_key => {
                key = inner_pair.as_str().to_string();
            }
            Rule::attribute_value => {
                value = parse_attribute_value(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Attribute { key, value, span })
}

fn parse_attribute_value(pair: Pair<Rule>) -> Result<AttributeValue, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::var_ref => Ok(AttributeValue::VarRef(
            inner_pair.into_inner().nth(1).unwrap().as_str().to_string(),
        )),
        Rule::literal_value => Ok(AttributeValue::Literal(inner_pair.as_str().to_string())),
        _ => Ok(AttributeValue::Literal(inner_pair.as_str().to_string())),
    }
}

fn parse_rune_block_content(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut content = String::new();
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::rune_content {
            content = inner_pair.as_str().to_string();
        }
    }

    Ok(Statement::RuneBlock { content, span })
}

fn parse_call_content(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut target = JumpTarget::Local(String::new());
    let mut filters = Vec::new();
    let mut args = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::jump_target => {
                target = parse_jump_target(inner_pair)?;
            }
            Rule::filter_list => {
                filters = parse_filter_list(inner_pair)?;
            }
            Rule::arg_list => {
                args = parse_arg_list_as_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::Call {
        target,
        filters,
        args,
        span,
    })
}

// NOTE: parse_jump_content removed in Phase 1 (REQ-BC-1: Jump statement removal)
// Jump statement (？) is no longer supported. Use Call (＞) instead.

fn parse_speech_line_content(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut speaker = String::new();
    let mut content = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::speaker => {
                speaker = inner_pair.as_str().trim().to_string();
            }
            Rule::speech_content => {
                content.extend(parse_speech_content(inner_pair)?);
            }
            Rule::continuation_line => {
                for cont_inner in inner_pair.into_inner() {
                    if cont_inner.as_rule() == Rule::speech_content {
                        content.extend(parse_speech_content(cont_inner)?);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Statement::Speech {
        speaker,
        content,
        span,
    })
}

#[allow(dead_code)]
fn parse_var_assign_from_parts(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut scope = VarScope::Local;
    let mut value = Expr::Literal(Literal::Number(0.0));

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::dollar_marker => {}
            Rule::var_scope => {
                scope = VarScope::Global;
            }
            Rule::var_name => {
                name = inner_pair.as_str().to_string();
            }
            Rule::expr => {
                value = parse_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::VarAssign {
        name,
        scope,
        value,
        span,
    })
}

// Old parse_statement - not used with new grammar structure
// fn parse_statement(pair: Pair<Rule>) -> Result<Option<Statement>, PastaError> {
//     ...
// }

fn parse_speech_content(pair: Pair<Rule>) -> Result<Vec<SpeechPart>, PastaError> {
    let mut parts = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::text_part => {
                parts.push(SpeechPart::Text(inner_pair.as_str().to_string()));
            }
            Rule::var_ref => {
                let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
                parts.push(SpeechPart::VarRef(var_name));
            }
            Rule::func_call => {
                let (name, args, scope) = parse_func_call(inner_pair)?;
                parts.push(SpeechPart::FuncCall { name, args, scope });
            }
            Rule::sakura_script => {
                // sakura_script = sakura_escape ~ sakura_command
                // We want the sakura_command part (second element)
                let cmd = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
                parts.push(SpeechPart::SakuraScript(cmd));
            }
            _ => {}
        }
    }

    Ok(parts)
}

#[allow(dead_code)]
fn parse_call_stmt(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut target = JumpTarget::Local(String::new());
    let mut filters = Vec::new();
    let mut args = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::jump_target => {
                target = parse_jump_target(inner_pair)?;
            }
            Rule::filter_list => {
                filters = parse_filter_list(inner_pair)?;
            }
            Rule::arg_list => {
                args = parse_arg_list_as_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::Call {
        target,
        filters,
        args,
        span,
    })
}

#[allow(dead_code)]
// parse_jump_stmt removed in Phase 1 (REQ-BC-1): Jump statement deprecated

fn parse_jump_target(pair: Pair<Rule>) -> Result<JumpTarget, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::dynamic_target => {
            let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
            Ok(JumpTarget::Dynamic(var_name))
        }
        Rule::long_jump => {
            let mut parts = inner_pair.into_inner();
            parts.next(); // Skip global marker
            let global = parts.next().unwrap().as_str().to_string();
            parts.next(); // Skip local marker
            let local = parts.next().unwrap().as_str().to_string();
            Ok(JumpTarget::LongJump { global, local })
        }
        Rule::global_target => {
            let mut parts = inner_pair.into_inner();
            parts.next(); // Skip marker
            let name = parts.next().unwrap().as_str().to_string();
            Ok(JumpTarget::Global(name))
        }
        Rule::local_target => Ok(JumpTarget::Local(inner_pair.as_str().to_string())),
        _ => Ok(JumpTarget::Local(inner_pair.as_str().to_string())),
    }
}

fn parse_filter_list(pair: Pair<Rule>) -> Result<Vec<Attribute>, PastaError> {
    let mut filters = Vec::new();

    for inner_pair in pair.into_inner() {
        // Each iteration should process: at_marker, attribute_key, colon, filter_value
        let mut key = String::new();
        let mut value = AttributeValue::Literal(String::new());
        let mut span_start = (1, 1);
        let mut span_end = (1, 1);

        for part in inner_pair.into_inner() {
            match part.as_rule() {
                Rule::attribute_key => {
                    key = part.as_str().to_string();
                    span_start = part.as_span().start_pos().line_col();
                }
                Rule::filter_value => {
                    value = parse_attribute_value(part.clone())?;
                    span_end = part.as_span().end_pos().line_col();
                }
                _ => {}
            }
        }

        if !key.is_empty() {
            filters.push(Attribute {
                key,
                value,
                span: Span::from_pest(span_start, span_end),
            });
        }
    }

    Ok(filters)
}

#[allow(dead_code)]
fn parse_var_assign(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut scope = VarScope::Local;
    let mut value = Expr::Literal(Literal::Number(0.0));

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::var_scope => {
                scope = VarScope::Global; // var_scope only appears for global vars
            }
            Rule::var_name => {
                name = inner_pair.as_str().to_string();
            }
            Rule::expr => {
                value = parse_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::VarAssign {
        name,
        scope,
        value,
        span,
    })
}

#[allow(dead_code)]
fn parse_rune_block(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut content = String::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::rune_content {
            content = inner_pair.as_str().to_string();
            break;
        }
    }

    Ok(Statement::RuneBlock { content, span })
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, PastaError> {
    let mut terms = Vec::new();
    let mut ops = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::term => {
                terms.push(parse_term(inner_pair)?);
            }
            Rule::bin_op => {
                ops.push(parse_bin_op(inner_pair)?);
            }
            _ => {}
        }
    }

    if terms.is_empty() {
        return Ok(Expr::Literal(Literal::Number(0.0)));
    }

    // Build left-associative binary operations
    let mut expr = terms.remove(0);
    for (op, rhs) in ops.into_iter().zip(terms.into_iter()) {
        expr = Expr::BinaryOp {
            op,
            lhs: Box::new(expr),
            rhs: Box::new(rhs),
        };
    }

    Ok(expr)
}

fn parse_term(pair: Pair<Rule>) -> Result<Expr, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::paren_expr => {
            let expr_pair = inner_pair.into_inner().nth(1).unwrap(); // Skip lparen, get expr
            Ok(Expr::Paren(Box::new(parse_expr(expr_pair)?)))
        }
        Rule::func_call => {
            let (name, args, scope) = parse_func_call(inner_pair)?;
            Ok(Expr::FuncCall { name, args, scope })
        }
        Rule::var_ref => {
            let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
            Ok(Expr::VarRef {
                name: var_name,
                scope: VarScope::Local, // Default to local, transpiler will resolve
            })
        }
        Rule::number_literal => {
            // Convert full-width digits to half-width before parsing
            let num_str = inner_pair.as_str();
            let normalized = num_str
                .chars()
                .map(|c| match c {
                    '０' => '0',
                    '１' => '1',
                    '２' => '2',
                    '３' => '3',
                    '４' => '4',
                    '５' => '5',
                    '６' => '6',
                    '７' => '7',
                    '８' => '8',
                    '９' => '9',
                    '－' => '-',
                    '．' => '.',
                    _ => c,
                })
                .collect::<String>();
            Ok(Expr::Literal(Literal::Number(normalized.parse().unwrap())))
        }
        Rule::string_literal => {
            let str_content = parse_string_literal(inner_pair)?;
            Ok(Expr::Literal(Literal::String(str_content)))
        }
        _ => Ok(Expr::Literal(Literal::Number(0.0))),
    }
}

fn parse_bin_op(pair: Pair<Rule>) -> Result<BinOp, PastaError> {
    let op_pair = pair.into_inner().next().unwrap();
    match op_pair.as_rule() {
        Rule::add => Ok(BinOp::Add),
        Rule::sub => Ok(BinOp::Sub),
        Rule::mul => Ok(BinOp::Mul),
        Rule::div => Ok(BinOp::Div),
        Rule::modulo => Ok(BinOp::Mod),
        _ => Ok(BinOp::Add),
    }
}

fn parse_func_call(pair: Pair<Rule>) -> Result<(String, Vec<Argument>, FunctionScope), PastaError> {
    let mut name = String::new();
    let mut args = Vec::new();
    let mut scope = FunctionScope::Auto; // Default to auto-resolution

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::func_name => {
                let func_name_str = inner_pair.as_str();
                // Check if function name starts with * for global-only scope
                if func_name_str.starts_with('*') || func_name_str.starts_with('＊') {
                    scope = FunctionScope::GlobalOnly;
                    name = func_name_str[1..].trim_start().to_string(); // Remove * prefix
                } else {
                    name = func_name_str.to_string();
                }
            }
            Rule::arg_list => {
                args = parse_arg_list(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok((name, args, scope))
}

fn parse_arg_list(pair: Pair<Rule>) -> Result<Vec<Argument>, PastaError> {
    let mut args = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::argument => {
                args.push(parse_argument(inner_pair)?);
            }
            _ => {} // Skip lparen, rparen
        }
    }

    Ok(args)
}

fn parse_arg_list_as_expr(pair: Pair<Rule>) -> Result<Vec<Expr>, PastaError> {
    let args = parse_arg_list(pair)?;
    Ok(args
        .into_iter()
        .map(|arg| match arg {
            Argument::Positional(expr) => expr,
            Argument::Named { value, .. } => value,
        })
        .collect())
}

fn parse_argument(pair: Pair<Rule>) -> Result<Argument, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::named_arg => {
            let mut parts = inner_pair.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            parts.next(); // Skip colon
            let value_pair = parts.next().unwrap();
            let value = parse_arg_value(value_pair)?;
            Ok(Argument::Named { name, value })
        }
        Rule::positional_arg => {
            let value_pair = inner_pair.into_inner().next().unwrap();
            let value = parse_arg_value(value_pair)?;
            Ok(Argument::Positional(value))
        }
        _ => Ok(Argument::Positional(Expr::Literal(Literal::Number(0.0)))),
    }
}

fn parse_arg_value(pair: Pair<Rule>) -> Result<Expr, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::string_literal => {
            let str_content = parse_string_literal(inner_pair)?;
            Ok(Expr::Literal(Literal::String(str_content)))
        }
        Rule::number_literal => Ok(Expr::Literal(Literal::Number(
            inner_pair.as_str().parse().unwrap(),
        ))),
        Rule::var_ref => {
            let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
            Ok(Expr::VarRef {
                name: var_name,
                scope: VarScope::Local,
            })
        }
        Rule::func_call => {
            let (name, args, scope) = parse_func_call(inner_pair)?;
            Ok(Expr::FuncCall { name, args, scope })
        }
        _ => Ok(Expr::Literal(Literal::Number(0.0))),
    }
}

fn parse_string_literal(pair: Pair<Rule>) -> Result<String, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::ja_string => {
            let content_pair = inner_pair.into_inner().next().unwrap();
            Ok(content_pair.as_str().to_string())
        }
        Rule::en_string => {
            let content_pair = inner_pair.into_inner().next().unwrap();
            // Handle escape sequences
            let content = content_pair.as_str();
            Ok(content
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"))
        }
        _ => Ok(String::new()),
    }
}
