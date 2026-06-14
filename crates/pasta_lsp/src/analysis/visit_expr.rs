//! AST visitor methods for semantic token generation: VarSet/expression tokenization.
//!
//! This file exceeds the 300-line guideline as documented in the design spec
//! (guideline exception: tightly-coupled visitor methods).

use pasta_dsl::parser::ast::*;

use super::text_utils::*;
use super::token_types::*;

// ============================================================================
// AST Visitors (split impl on AnalysisEngine): VarSet / expression tokenization
// ============================================================================

impl super::AnalysisEngine {
    pub(super) fn visit_var_set(vs: &VarSet, source: &str, tokens: &mut Vec<RawToken>) {
        if !vs.span.is_valid() {
            return;
        }
        let line = vs.span.start_line;
        let (line_text, span_text, span_start_in_line) = Self::span_line_window(&vs.span, source);

        // Parse sub-tokens from the span text using cursor-based scanning
        Self::tokenize_var_set_text(span_text, span_start_in_line, line, vs, tokens, line_text);
    }

    /// Tokenize the text content of a VarSet line into fine-grained semantic tokens.
    fn tokenize_var_set_text(
        span_text: &str,
        base_offset: usize,
        line: usize,
        vs: &VarSet,
        tokens: &mut Vec<RawToken>,
        line_text: &str,
    ) {
        let line0 = (line - 1) as u32;
        let mut cursor = 0usize; // byte offset within span_text

        // 1) Marker: ＄＊ or ＄ (variable marker with optional global prefix)
        //
        // 健全な var_set 行は必ずマーカーで始まる。span_text がどのマーカーでも
        // 始まらない場合、span がこの行に整合していない（部分パースのチャンク相対
        // span がフルソースの別行に当たった等）ため、安全に何も出さず打ち切る。
        // これを fallback で `$` 等にしてしまうと cursor が span_text 長を超え、
        // 後続の `span_text[cursor..]` がパニックする。
        let marker = match vs.scope {
            VarScope::Global => {
                // Try ＄＊ first, then $*
                if span_text.starts_with("＄＊") {
                    "＄＊"
                } else if span_text.starts_with("$*") {
                    "$*"
                } else if span_text.starts_with("＄") {
                    "＄"
                } else if span_text.starts_with("$") {
                    "$"
                } else {
                    return;
                }
            }
            _ => {
                if span_text.starts_with("＄") {
                    "＄"
                } else if span_text.starts_with("$") {
                    "$"
                } else {
                    return;
                }
            }
        };
        // Emit marker token
        let marker_offset = base_offset + cursor;
        tokens.push(RawToken {
            line: line0,
            start_char: utf8_offset_to_utf16(line_text, marker_offset),
            length: utf8_len_to_utf16(marker),
            token_type: token_type::VARIABLE,
            modifiers: 0,
        });
        cursor += marker.len();

        // 2) Variable name (skip for var_set_none where name is None)
        if let Some(name) = &vs.name
            && let Some(name_pos) = span_text[cursor..].find(name.as_str())
        {
            let name_start = cursor + name_pos;
            let name_offset = base_offset + name_start;
            let global_mod = if vs.scope == VarScope::Global {
                1 << 2
            } else {
                0
            };
            tokens.push(RawToken {
                line: line0,
                start_char: utf8_offset_to_utf16(line_text, name_offset),
                length: utf8_len_to_utf16(name),
                token_type: token_type::VARIABLE,
                modifiers: (1 << 1) | global_mod, // definition + optional global
            });
            cursor = name_start + name.len();
        }

        // 3) Assignment operator ＝ or =
        let remaining = &span_text[cursor..];
        let eq_char = if remaining.starts_with('＝') {
            Some(('＝', '＝'.len_utf8()))
        } else if remaining.starts_with('=') {
            Some(('=', 1))
        } else {
            None
        };
        if let Some((_, eq_len)) = eq_char {
            let eq_offset = base_offset + cursor;
            tokens.push(RawToken {
                line: line0,
                start_char: utf8_offset_to_utf16(line_text, eq_offset),
                length: utf8_len_to_utf16(&span_text[cursor..cursor + eq_len]),
                token_type: token_type::OPERATOR,
                modifiers: 0,
            });
            cursor += eq_len;
        }

        // 4) Right-hand side value
        let rhs_text = &span_text[cursor..];
        let rhs_base = base_offset + cursor;
        Self::tokenize_expr_text(rhs_text, rhs_base, line0, &vs.value, tokens, line_text);
    }

    /// Tokenize the right-hand side of a variable assignment.
    fn tokenize_expr_text(
        text: &str,
        base_offset: usize,
        line: u32,
        value: &SetValue,
        tokens: &mut Vec<RawToken>,
        line_text: &str,
    ) {
        match value {
            SetValue::WordRef { name } => {
                // Find @name or ＠name in text
                let marker_and_name = format!("＠{}", name);
                let alt_marker = format!("@{}", name);
                if let Some(pos) = text
                    .find(&marker_and_name)
                    .or_else(|| text.find(&alt_marker))
                {
                    let found = if text[pos..].starts_with('＠') {
                        &marker_and_name
                    } else {
                        &alt_marker
                    };
                    let offset = base_offset + pos;
                    tokens.push(RawToken {
                        line,
                        start_char: utf8_offset_to_utf16(line_text, offset),
                        length: utf8_len_to_utf16(found),
                        token_type: token_type::WORD,
                        modifiers: 0,
                    });
                }
            }
            SetValue::Expr(expr) => {
                Self::tokenize_expr_recursive(text, base_offset, line, expr, tokens, line_text);
            }
        }
    }

    /// Recursively tokenize an Expr by scanning the source text.
    /// Since Expr variants don't carry span info, we scan the text to find each sub-expression.
    fn tokenize_expr_recursive(
        text: &str,
        base_offset: usize,
        line: u32,
        expr: &Expr,
        tokens: &mut Vec<RawToken>,
        line_text: &str,
    ) {
        match expr {
            Expr::Integer(_) | Expr::Float(_) => {
                // Find the number literal in text
                if let Some((start, end)) = find_number_literal(text) {
                    let offset = base_offset + start;
                    let num_text = &text[start..end];
                    tokens.push(RawToken {
                        line,
                        start_char: utf8_offset_to_utf16(line_text, offset),
                        length: utf8_len_to_utf16(num_text),
                        token_type: token_type::NUMBER,
                        modifiers: 0,
                    });
                }
            }
            Expr::String(s) => {
                // String literal - find it in text
                if let Some(pos) = text.find(s.as_str()) {
                    let offset = base_offset + pos;
                    tokens.push(RawToken {
                        line,
                        start_char: utf8_offset_to_utf16(line_text, offset),
                        length: utf8_len_to_utf16(s),
                        token_type: token_type::TALK,
                        modifiers: 0,
                    });
                }
            }
            Expr::BlankString => {}
            Expr::VarRef { name, scope } => {
                // Find $name or ＄name (or $*name/＄＊name for global) in text
                let patterns = match scope {
                    VarScope::Global => vec![format!("＄＊{}", name), format!("$*{}", name)],
                    VarScope::Args(_) => vec![format!("＄{}", name), format!("${}", name)],
                    VarScope::Local => vec![format!("＄{}", name), format!("${}", name)],
                    VarScope::Property => vec![format!("＄％{}", name), format!("$%{}", name)],
                };
                for pat in &patterns {
                    if let Some(pos) = text.find(pat.as_str()) {
                        let offset = base_offset + pos;
                        tokens.push(RawToken {
                            line,
                            start_char: utf8_offset_to_utf16(line_text, offset),
                            length: utf8_len_to_utf16(pat),
                            token_type: token_type::VARIABLE,
                            modifiers: 0,
                        });
                        break;
                    }
                }
            }
            Expr::FnCall { name, scope, .. } => {
                // Find @name or ＠name (or @*name/＠＊name for global)
                let patterns = match scope {
                    FnScope::Global => vec![format!("＠＊{}", name), format!("@*{}", name)],
                    FnScope::Local => vec![format!("＠{}", name), format!("@{}", name)],
                };
                for pat in &patterns {
                    if let Some(pos) = text.find(pat.as_str()) {
                        let fn_offset = base_offset + pos;
                        // Emit function name token
                        tokens.push(RawToken {
                            line,
                            start_char: utf8_offset_to_utf16(line_text, fn_offset),
                            length: utf8_len_to_utf16(pat),
                            token_type: token_type::WORD,
                            modifiers: 0,
                        });
                        // Try to tokenize arguments inside parentheses
                        let after_name = pos + pat.len();
                        Self::tokenize_args_text(
                            &text[after_name..],
                            base_offset + after_name,
                            line,
                            expr,
                            tokens,
                            line_text,
                        );
                        break;
                    }
                }
            }
            Expr::Paren(inner) => {
                // Find opening paren
                if let Some(paren_pos) = find_open_paren(text) {
                    let paren_len = Self::push_char_op_token(
                        text,
                        paren_pos,
                        base_offset,
                        line,
                        tokens,
                        line_text,
                    );
                    // Inner expression
                    let inner_start = paren_pos + paren_len;
                    let close_pos = find_close_paren(text, inner_start);
                    let inner_text = if let Some(cp) = close_pos {
                        &text[inner_start..cp]
                    } else {
                        &text[inner_start..]
                    };
                    Self::tokenize_expr_recursive(
                        inner_text,
                        base_offset + inner_start,
                        line,
                        inner,
                        tokens,
                        line_text,
                    );
                    // Closing paren
                    if let Some(cp) = close_pos {
                        Self::push_char_op_token(text, cp, base_offset, line, tokens, line_text);
                    }
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                // Find the operator in the text, split into lhs and rhs
                // First tokenize lhs, then operator, then rhs
                // We need to find the binary op position - scan for it
                let op_chars: &[&str] = match op {
                    BinOp::Add => &["+", "＋"],
                    BinOp::Sub => &["-", "ー", "－"],
                    BinOp::Mul => &["*", "＊"],
                    BinOp::Div => &["/", "／"],
                    BinOp::Mod => &["%", "％"],
                };
                // Find a split point: we need to tokenize lhs first, then find op
                // Strategy: find the operator that splits the text
                if let Some((op_pos, op_len)) = find_binary_op(text, op_chars) {
                    let lhs_text = &text[..op_pos];
                    let rhs_text = &text[op_pos + op_len..];
                    // LHS
                    Self::tokenize_expr_recursive(
                        lhs_text,
                        base_offset,
                        line,
                        lhs,
                        tokens,
                        line_text,
                    );
                    // Operator
                    let op_str = &text[op_pos..op_pos + op_len];
                    tokens.push(RawToken {
                        line,
                        start_char: utf8_offset_to_utf16(line_text, base_offset + op_pos),
                        length: utf8_len_to_utf16(op_str),
                        token_type: token_type::OPERATOR,
                        modifiers: 0,
                    });
                    // RHS
                    Self::tokenize_expr_recursive(
                        rhs_text,
                        base_offset + op_pos + op_len,
                        line,
                        rhs,
                        tokens,
                        line_text,
                    );
                } else {
                    // Fallback: emit entire text as variable token
                    if !text.trim().is_empty() {
                        tokens.push(RawToken {
                            line,
                            start_char: utf8_offset_to_utf16(line_text, base_offset),
                            length: utf8_len_to_utf16(text),
                            token_type: token_type::VARIABLE,
                            modifiers: 0,
                        });
                    }
                }
            }
        }
    }

    /// Tokenize arguments inside parentheses of a function call.
    fn tokenize_args_text(
        text: &str,
        base_offset: usize,
        line: u32,
        fn_expr: &Expr,
        tokens: &mut Vec<RawToken>,
        line_text: &str,
    ) {
        if let Expr::FnCall { args, .. } = fn_expr {
            // Find opening paren
            if let Some(paren_pos) = find_open_paren(text) {
                let paren_len =
                    Self::push_char_op_token(text, paren_pos, base_offset, line, tokens, line_text);

                let inner_start = paren_pos + paren_len;
                let close_pos = find_close_paren(text, inner_start);
                let args_text = if let Some(cp) = close_pos {
                    &text[inner_start..cp]
                } else {
                    &text[inner_start..]
                };

                // Tokenize each argument by scanning the args text
                let mut arg_cursor = 0usize;
                for arg in &args.items {
                    let arg_expr = match arg {
                        Arg::Positional(e) => e,
                        Arg::Keyword { value, .. } => value,
                    };
                    let remaining = &args_text[arg_cursor..];
                    // Skip comma/separator
                    let skip = remaining
                        .find(|c: char| !c.is_whitespace() && c != '、' && c != ',')
                        .unwrap_or(0);
                    let arg_text_start = arg_cursor + skip;
                    // Estimate the end of this argument: find next comma or end
                    let arg_remaining = &args_text[arg_text_start..];
                    let arg_end = find_arg_end(arg_remaining);
                    let arg_slice = &args_text[arg_text_start..arg_text_start + arg_end];
                    Self::tokenize_expr_recursive(
                        arg_slice,
                        base_offset + inner_start + arg_text_start,
                        line,
                        arg_expr,
                        tokens,
                        line_text,
                    );
                    arg_cursor = arg_text_start + arg_end;
                }

                // Closing paren
                if let Some(cp) = close_pos {
                    Self::push_char_op_token(text, cp, base_offset, line, tokens, line_text);
                }
            }
        }
    }
}
