//! Element parsing functions: attributes, keywords, code blocks, variables.

use super::*;

/// Parse attribute.
pub(crate) fn parse_attr(pair: Pair<Rule>) -> Result<Attr, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut key = String::new();
    let mut value = AttrValue::AttrString(String::new());

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::key_attr {
            for kv_inner in inner.into_inner() {
                match kv_inner.as_rule() {
                    Rule::id => {
                        key = kv_inner.as_str().to_string();
                    }
                    Rule::number_literal => {
                        value = parse_attr_number(kv_inner.as_str());
                    }
                    Rule::string_contents | Rule::string_blank => {
                        value = AttrValue::String(kv_inner.as_str().to_string());
                    }
                    Rule::attr_string => {
                        value = AttrValue::AttrString(kv_inner.as_str().to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(Attr { key, value, span })
}

/// Parse attribute number value.
pub(crate) fn parse_attr_number(s: &str) -> AttrValue {
    let normalized = normalize_number_str(s);
    if normalized.contains('.') {
        AttrValue::Float(normalized.parse().unwrap_or(0.0))
    } else {
        AttrValue::Integer(normalized.parse().unwrap_or(0))
    }
}

/// Parse key_words.
pub(crate) fn parse_key_words(pair: Pair<Rule>) -> Result<KeyWords, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut names = Vec::new();
    let mut words = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::key_list => {
                for key_inner in inner.into_inner() {
                    if key_inner.as_rule() == Rule::id {
                        names.push(key_inner.as_str().to_string());
                    }
                }
            }
            Rule::words => {
                for word_inner in inner.into_inner() {
                    match word_inner.as_rule() {
                        Rule::string_contents | Rule::string_blank => {
                            words.push(word_inner.as_str().to_string());
                        }
                        Rule::word_nofenced => {
                            words.push(word_inner.as_str().to_string());
                        }
                        Rule::sakura_script => {
                            words.push(word_inner.as_str().to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(KeyWords { names, words, span })
}

/// Parse code block.
pub(crate) fn parse_code_block(pair: Pair<Rule>) -> Result<CodeBlock, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut language = None;
    let mut content = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                language = Some(inner.as_str().to_string());
            }
            Rule::code_contents => {
                content = inner.as_str().to_string();
            }
            _ => {}
        }
    }

    Ok(CodeBlock {
        language,
        content,
        span,
    })
}

/// Parse var_set.
pub(crate) fn parse_var_set(pair: Pair<Rule>) -> Result<VarSet, ParseError> {
    let span = Span::from(&pair.as_span());
    let scope = match pair.as_rule() {
        Rule::var_set_global => VarScope::Global,
        Rule::var_set_property => VarScope::Property,
        _ => VarScope::Local, // var_set_local, var_set_none
    };

    let mut name: Option<String> = None;
    let mut terms: Vec<Expr> = Vec::new();
    let mut operators: Vec<BinOp> = Vec::new();
    let mut word_ref_name: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id | Rule::property_id => {
                // The first id is the variable name
                if name.is_none() {
                    name = Some(inner.as_str().to_string());
                }
            }
            Rule::word_ref => {
                // word_ref = { word_marker ~ id ~ s }
                // word_marker is a hidden rule, so only id is in inner pairs
                for word_inner in inner.into_inner() {
                    if word_inner.as_rule() == Rule::id {
                        word_ref_name = Some(word_inner.as_str().to_string());
                        break;
                    }
                }
            }
            Rule::add_op => operators.push(BinOp::Add),
            Rule::sub_op => operators.push(BinOp::Sub),
            Rule::mul_op => operators.push(BinOp::Mul),
            Rule::div_op => operators.push(BinOp::Div),
            Rule::modulo_op => operators.push(BinOp::Mod),
            _ => {
                // Try to parse as expression (term)
                if let Some(expr) = try_parse_expr(inner) {
                    terms.push(expr);
                }
            }
        }
    }

    // Build value based on whether we have word_ref or expr
    let value = if let Some(ref_name) = word_ref_name {
        // word_ref was detected
        SetValue::WordRef { name: ref_name }
    } else {
        // Build left-associative binary expression
        let expr = if terms.is_empty() {
            Expr::BlankString
        } else {
            let mut result = terms.remove(0);
            for (i, op) in operators.into_iter().enumerate() {
                if i < terms.len() {
                    result = Expr::Binary {
                        op,
                        lhs: Box::new(result),
                        rhs: Box::new(terms[i].clone()),
                    };
                }
            }
            result
        };
        SetValue::Expr(expr)
    };

    Ok(VarSet {
        name,
        scope,
        value,
        span,
    })
}

/// Normalize number string by converting full-width characters to half-width.
///
/// Converts:
/// - Full-width digits ('０'..'９') to half-width ('0'..'9')
/// - Full-width minus ('－') to half-width ('-')
/// - Full-width decimal point ('．') to half-width ('.')
pub(crate) fn normalize_number_str(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '０'..='９' => ((c as u32 - '０' as u32) + '0' as u32) as u8 as char,
            '－' => '-',
            '．' => '.',
            _ => c,
        })
        .collect()
}
