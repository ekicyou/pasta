//! Element parsing functions: attributes, keywords, code blocks, variables.

use super::*;

/// Parse attribute.
pub(crate) fn parse_attr(pair: Pair<Rule>, filename: &str) -> Result<Attr, ParseError> {
    let span = Span::from_pair(&pair);
    let mut key = String::new();
    let mut body = AttrBody::Flag;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::key_attr => {
                let mut value = AttrValue::AttrString(String::new());
                for kv_inner in inner.into_inner() {
                    if kv_inner.as_rule() == Rule::id {
                        key = kv_inner.as_str().to_string();
                    } else if let Some(v) = parse_attr_value(&kv_inner, filename)? {
                        value = v;
                    }
                }
                body = AttrBody::Decl(value);
            }
            Rule::attr_cond => {
                let mut op = CmpOp::Eq;
                let mut value = AttrValue::AttrString(String::new());
                for c in inner.into_inner() {
                    match c.as_rule() {
                        Rule::attr_var_global | Rule::attr_var_local => key = attr_var_key(&c),
                        Rule::cmp_op => op = cmp_op(c.as_str()),
                        _ => {
                            if let Some(v) = parse_attr_value(&c, filename)? {
                                value = v;
                            }
                        }
                    }
                }
                body = AttrBody::Cond { op, value };
            }
            Rule::attr_flag => {
                if let Some(var) = inner.into_inner().next() {
                    key = attr_var_key(&var);
                }
            }
            _ => {}
        }
    }

    Ok(Attr { key, body, span })
}

/// 属性の値の字句。値でない子は None。
fn parse_attr_value(pair: &Pair<Rule>, filename: &str) -> Result<Option<AttrValue>, ParseError> {
    Ok(Some(match pair.as_rule() {
        Rule::number_literal => match number_value(pair, filename)? {
            Number::Integer(i) => AttrValue::Integer(i),
            Number::Float(f) => AttrValue::Float(f),
        },
        Rule::string_contents => AttrValue::String(pair.as_str().to_string()),
        Rule::string_blank => AttrValue::String(String::new()),
        Rule::attr_string => AttrValue::AttrString(pair.as_str().to_string()),
        _ => return Ok(None),
    }))
}

/// 条件・フラグの変数の名前（印は全角に揃え、名前は書いたまま）。
fn attr_var_key(pair: &Pair<Rule>) -> String {
    let mark = if pair.as_rule() == Rule::attr_var_global {
        "＄＊"
    } else {
        "＄"
    };
    let name = pair
        .clone()
        .into_inner()
        .next()
        .map_or("", |id| id.as_str());
    format!("{mark}{name}")
}

/// 比較演算子の字句（`Rule::cmp_op`）を正規形の型へ。
fn cmp_op(s: &str) -> CmpOp {
    match s {
        "≧" | ">=" => CmpOp::Ge,
        "≦" | "<=" => CmpOp::Le,
        "≠" | "!=" => CmpOp::Ne,
        "＞" | ">" => CmpOp::Gt,
        "＜" | "<" => CmpOp::Lt,
        _ => CmpOp::Eq, // "＝" | "="（文法がこの 12 種しか通さない）
    }
}

/// 数の字句の値。
pub(crate) enum Number {
    Integer(i64),
    Float(f64), // 常に有限
}

/// 数の字句（`Rule::number_literal`）の値。
///
/// 小数点が無く i64 に収まれば整数、収まらなければいちばん近い小数（Lua と同じ）、
/// 小数点があれば小数。有限の小数に収まらなければ、字句の位置の構文の誤り。
pub(crate) fn number_value(pair: &Pair<Rule>, filename: &str) -> Result<Number, ParseError> {
    let normalized = normalize_number_str(pair.as_str());
    if !normalized.contains('.')
        && let Ok(i) = normalized.parse::<i64>()
    {
        return Ok(Number::Integer(i));
    }
    // 文法が数字だけを通すので parse は失敗しない。大きすぎると inf になる
    match normalized.parse::<f64>() {
        Ok(f) if f.is_finite() => Ok(Number::Float(f)),
        _ => Err(syntax_error(
            filename,
            pair.line_col(),
            "数が大きすぎます（小数で表せるのは、およそ 1.8×10^308 まで）",
        )),
    }
}

/// Parse key_words.
pub(crate) fn parse_key_words(pair: Pair<Rule>, filename: &str) -> Result<KeyWords, ParseError> {
    let span = Span::from_pair(&pair);
    let mut names = Vec::new();
    let mut words = Vec::new();
    let mut weight_spans = Vec::new();

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
                for item in inner.into_inner() {
                    // word_item = { word ~ word_weight? | word_weight }
                    let item_text = item.as_str();
                    let item_pos = item.as_span().start_pos().line_col();
                    let mut value = None;
                    let mut weight = None;
                    for word_inner in item.into_inner() {
                        match word_inner.as_rule() {
                            Rule::string_blank => value = Some(String::new()),
                            Rule::string_contents | Rule::word_nofenced | Rule::sakura_script => {
                                value = Some(word_inner.as_str().to_string())
                            }
                            Rule::word_weight => weight = Some(word_inner),
                            _ => {}
                        }
                    }
                    let value = value.unwrap_or_default();
                    let Some(weight) = weight else {
                        words.push(value);
                        continue;
                    };
                    let weight_text = weight.as_str();
                    let err = |what: String| {
                        syntax_error(
                            filename,
                            item_pos,
                            &format!("単語 `{}` の値 `{item_text}` {what}", names.join("、")),
                        )
                    };
                    if value.is_empty() {
                        return Err(err(format!(
                            "は重みの `＠数` を除くと空になります（`{weight_text}` を文字として出すときは「」で囲みます）"
                        )));
                    }
                    // `＠` は 1 文字なので、その後ろが数字
                    let digits: String = weight_text.chars().skip(1).collect();
                    let n: usize = match normalize_number_str(&digits).parse() {
                        Ok(0) => {
                            return Err(err(format!(
                                "の重みは 1 以上の整数で書きます（`{weight_text}` を文字として出すときは「」で囲みます）"
                            )));
                        }
                        Ok(n) => n,
                        Err(_) => return Err(err("の重みが大きすぎます".to_string())),
                    };
                    // 並びが持てる要素の数の上限（ターゲットで決まり、空きメモリに依らない）。それより下のメモリ不足は設計の Risks どおり受け入れる
                    let max_len = isize::MAX as usize / std::mem::size_of::<String>();
                    if n > max_len - words.len() {
                        return Err(err("の重みが大きすぎます".to_string()));
                    }
                    words.extend(std::iter::repeat_n(value, n));
                    weight_spans.push(Span::from_pair(&weight));
                }
            }
            _ => {}
        }
    }

    Ok(KeyWords {
        names,
        words,
        weight_spans,
        span,
    })
}

/// Parse code block.
pub(crate) fn parse_code_block(pair: Pair<Rule>) -> Result<CodeBlock, ParseError> {
    let span = Span::from_pair(&pair);
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
pub(crate) fn parse_var_set(pair: Pair<Rule>, filename: &str) -> Result<VarSet, ParseError> {
    check_op_count(&pair, filename)?;
    let span = Span::from_pair(&pair);
    let scope = match pair.as_rule() {
        Rule::var_set_global => VarScope::Global,
        Rule::var_set_property => VarScope::Property,
        _ => VarScope::Local, // var_set_local, var_set_none
    };

    let mut name: Option<String> = None;
    let mut terms: Vec<Expr> = Vec::new();
    let mut operators: Vec<BinOp> = Vec::new();
    let mut word_ref_value: Option<SetValue> = None;

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
                        word_ref_value = Some(SetValue::WordRef {
                            name: word_inner.as_str().to_string(),
                        });
                        break;
                    }
                }
            }
            Rule::word_ref_dynamic => {
                // Explicit arm: the default arm would not yield the dynamic reference
                word_ref_value = parse_dyn_name(inner, filename)?.map(|(var_name, var_scope)| {
                    SetValue::DynamicWordRef {
                        var_name,
                        var_scope,
                    }
                });
            }
            rule => {
                if let Some(op) = bin_op_from_rule(rule) {
                    operators.push(op);
                } else if let Some(expr) = try_parse_expr(inner, filename)? {
                    // Parse as expression (term)
                    terms.push(expr);
                }
            }
        }
    }

    // Build value based on whether we have a (static or dynamic) word_ref or expr
    let value = word_ref_value.unwrap_or_else(|| {
        SetValue::Expr(if terms.is_empty() {
            Expr::BlankString
        } else {
            build_left_assoc_expr(terms, operators)
        })
    });

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
