//! AST visitor methods for semantic token generation.
//!
//! This file exceeds the 300-line guideline as documented in the design spec
//! (guideline exception: tightly-coupled visitor methods).

use pasta_dsl::parser::ast::*;

use super::text_utils::*;
use super::token_types::*;

// ============================================================================
// AST Visitors (split impl on AnalysisEngine)
// ============================================================================

impl super::AnalysisEngine {
    pub(super) fn visit_file_items(items: &[FileItem], source: &str, tokens: &mut Vec<RawToken>) {
        for item in items {
            match item {
                FileItem::FileAttr(attr) => Self::visit_attr(attr, source, tokens),
                FileItem::GlobalWord(word) => Self::visit_keywords(word, source, tokens),
                FileItem::GlobalSceneScope(scene) => {
                    Self::visit_global_scene(scene, source, tokens);
                }
                FileItem::ActorScope(actor) => Self::visit_actor_scope(actor, source, tokens),
            }
        }
    }

    fn visit_global_scene(scene: &GlobalSceneScope, source: &str, tokens: &mut Vec<RawToken>) {
        if scene.span.is_valid() {
            // Only emit NAMESPACE token for the marker line (e.g., "＊メイン"),
            // NOT the entire scope. Otherwise it overshadows child tokens.
            let line = scene.span.start_line;
            let line_text = get_line_text(source, line);
            let line_start = line_byte_offset(source, line);
            let start_in_line = scene.span.start_byte.saturating_sub(line_start);
            let start_char = utf8_offset_to_utf16(line_text, start_in_line);
            let line_len = utf8_len_to_utf16(line_text);
            if line_len > start_char {
                tokens.push(RawToken {
                    line: (line - 1) as u32,
                    start_char,
                    length: line_len.saturating_sub(start_char),
                    token_type: token_type::NAMESPACE,
                    modifiers: 0,
                });
            }
        }
        for attr in &scene.attrs {
            Self::visit_attr(attr, source, tokens);
        }
        for word in &scene.words {
            Self::visit_keywords(word, source, tokens);
        }
        for cb in &scene.code_blocks {
            Self::visit_code_block(cb, source, tokens);
        }
        for local in &scene.local_scenes {
            Self::visit_local_scene(local, source, tokens);
        }
    }

    fn visit_local_scene(scene: &LocalSceneScope, source: &str, tokens: &mut Vec<RawToken>) {
        // Only emit SCENE token for named local scenes (with explicit marker like ・ランダム).
        // Anonymous local_start_scene_scope (name=None) has no marker line, and emitting
        // a SCENE token would overlap with the action line tokens within the scope.
        if scene.name.is_some() && scene.span.is_valid() {
            let line = scene.span.start_line;
            let line_text = get_line_text(source, line);
            let line_start = line_byte_offset(source, line);
            let start_in_line = scene.span.start_byte.saturating_sub(line_start);
            let start_char = utf8_offset_to_utf16(line_text, start_in_line);
            let line_len = utf8_len_to_utf16(line_text);
            tokens.push(RawToken {
                line: (line - 1) as u32,
                start_char,
                length: line_len.saturating_sub(start_char),
                token_type: token_type::SCENE,
                modifiers: 0,
            });
        }
        for attr in &scene.attrs {
            Self::visit_attr(attr, source, tokens);
        }
        for cb in &scene.code_blocks {
            Self::visit_code_block(cb, source, tokens);
        }
        for item in &scene.items {
            Self::visit_local_scene_item(item, source, tokens);
        }
    }

    fn visit_actor_scope(actor: &ActorScope, source: &str, tokens: &mut Vec<RawToken>) {
        if actor.span.is_valid() {
            // Only emit ACTOR token for the marker line (e.g., "％さくら"),
            // NOT the entire scope. Otherwise it overshadows child tokens.
            let line = actor.span.start_line;
            let line_text = get_line_text(source, line);
            let line_start = line_byte_offset(source, line);
            let start_in_line = actor.span.start_byte.saturating_sub(line_start);
            let start_char = utf8_offset_to_utf16(line_text, start_in_line);
            let line_len = utf8_len_to_utf16(line_text);
            if line_len > start_char {
                tokens.push(RawToken {
                    line: (line - 1) as u32,
                    start_char,
                    length: line_len.saturating_sub(start_char),
                    token_type: token_type::ACTOR,
                    modifiers: 0,
                });
            }
        }
        for attr in &actor.attrs {
            Self::visit_attr(attr, source, tokens);
        }
        for word in &actor.words {
            Self::visit_keywords(word, source, tokens);
        }
        for vs in &actor.var_sets {
            Self::visit_var_set(vs, source, tokens);
        }
        for cb in &actor.code_blocks {
            Self::visit_code_block(cb, source, tokens);
        }
    }

    fn visit_attr(attr: &Attr, source: &str, tokens: &mut Vec<RawToken>) {
        if attr.span.is_valid() {
            Self::add_token_from_span(&attr.span, source, token_type::DECORATOR, 0, tokens);
        }
    }

    fn visit_keywords(word: &KeyWords, source: &str, tokens: &mut Vec<RawToken>) {
        if word.span.is_valid() {
            Self::add_token_from_span(&word.span, source, token_type::WORD, 0, tokens);
        }
    }

    fn visit_code_block(cb: &CodeBlock, source: &str, tokens: &mut Vec<RawToken>) {
        if cb.span.is_valid() {
            Self::add_token_from_span(&cb.span, source, token_type::CODE_BLOCK, 0, tokens);
        }
    }

    fn visit_var_set(vs: &VarSet, source: &str, tokens: &mut Vec<RawToken>) {
        if !vs.span.is_valid() {
            return;
        }
        let line = vs.span.start_line;
        let line_text = get_line_text(source, line);
        let line_start = line_byte_offset(source, line);
        let span_start_in_line = vs.span.start_byte.saturating_sub(line_start);
        let span_end_in_line = vs
            .span
            .end_byte
            .saturating_sub(line_start)
            .min(line_text.len());
        let span_text = &line_text[span_start_in_line..span_end_in_line];

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
        let marker = match vs.scope {
            VarScope::Global => {
                // Try ＄＊ first, then $*
                if span_text.starts_with("＄＊") {
                    "＄＊"
                } else if span_text.starts_with("$*") {
                    "$*"
                } else if span_text.starts_with("＄") {
                    "＄"
                } else {
                    "$"
                }
            }
            _ => {
                if span_text.starts_with("＄") {
                    "＄"
                } else {
                    "$"
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
        if let Some(name) = &vs.name {
            if let Some(name_pos) = span_text[cursor..].find(name.as_str()) {
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
                    let paren_char = &text[paren_pos..paren_pos + char_len_at(text, paren_pos)];
                    tokens.push(RawToken {
                        line,
                        start_char: utf8_offset_to_utf16(line_text, base_offset + paren_pos),
                        length: utf8_len_to_utf16(paren_char),
                        token_type: token_type::OPERATOR,
                        modifiers: 0,
                    });
                    // Inner expression
                    let inner_start = paren_pos + paren_char.len();
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
                        let close_char = &text[cp..cp + char_len_at(text, cp)];
                        tokens.push(RawToken {
                            line,
                            start_char: utf8_offset_to_utf16(line_text, base_offset + cp),
                            length: utf8_len_to_utf16(close_char),
                            token_type: token_type::OPERATOR,
                            modifiers: 0,
                        });
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
                let paren_char = &text[paren_pos..paren_pos + char_len_at(text, paren_pos)];
                tokens.push(RawToken {
                    line,
                    start_char: utf8_offset_to_utf16(line_text, base_offset + paren_pos),
                    length: utf8_len_to_utf16(paren_char),
                    token_type: token_type::OPERATOR,
                    modifiers: 0,
                });

                let inner_start = paren_pos + paren_char.len();
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
                    let close_char = &text[cp..cp + char_len_at(text, cp)];
                    tokens.push(RawToken {
                        line,
                        start_char: utf8_offset_to_utf16(line_text, base_offset + cp),
                        length: utf8_len_to_utf16(close_char),
                        token_type: token_type::OPERATOR,
                        modifiers: 0,
                    });
                }
            }
        }
    }

    fn visit_local_scene_item(item: &LocalSceneItem, source: &str, tokens: &mut Vec<RawToken>) {
        match item {
            LocalSceneItem::VarSet(vs) => Self::visit_var_set(vs, source, tokens),
            LocalSceneItem::CallScene(cs) => Self::visit_call_scene(cs, source, tokens),
            LocalSceneItem::ActionLine(al) => Self::visit_action_line(al, source, tokens),
            LocalSceneItem::ContinueAction(ca) => {
                Self::visit_continue_action(ca, source, tokens);
            }
            LocalSceneItem::CueCommand(cue) => {
                // キューコマンド行の細粒度セマンティックトークン生成
                if cue.span.is_valid() {
                    Self::visit_cue_command(cue, source, tokens);
                }
            }
            LocalSceneItem::Choice(_) => {
                // 選択肢行のセマンティックトークン生成（将来実装）
            }
        }
    }

    fn visit_cue_command(cue: &CueCommandNode, source: &str, tokens: &mut Vec<RawToken>) {
        let line = cue.span.start_line;
        let line_text = get_line_text(source, line);
        let line_start = line_byte_offset(source, line);
        let span_start_in_line = cue.span.start_byte.saturating_sub(line_start);
        let span_end_in_line = cue
            .span
            .end_byte
            .saturating_sub(line_start)
            .min(line_text.len());
        let span_text = &line_text[span_start_in_line..span_end_in_line];
        let base_offset = span_start_in_line;
        let line0 = (line - 1) as u32;

        // Skip leading whitespace (pad)
        let trimmed = span_text.trim_start();
        let mut cursor = span_text.len() - trimmed.len();

        // 1) マーカー: ！ (3 bytes) or ! (1 byte) — 全角優先
        let remaining = &span_text[cursor..];
        let marker = if remaining.starts_with('！') {
            "！"
        } else if remaining.starts_with('!') {
            "!"
        } else {
            return; // マーカー検出失敗 — サイレントフォールバック
        };
        tokens.push(RawToken {
            line: line0,
            start_char: utf8_offset_to_utf16(line_text, base_offset + cursor),
            length: utf8_len_to_utf16(marker),
            token_type: token_type::CUE_MARKER,
            modifiers: 0,
        });
        cursor += marker.len();

        // 2) コマンド名
        if let Some(name_pos) = span_text[cursor..].find(cue.command.as_str()) {
            let name_start = cursor + name_pos;
            tokens.push(RawToken {
                line: line0,
                start_char: utf8_offset_to_utf16(line_text, base_offset + name_start),
                length: utf8_len_to_utf16(&cue.command),
                token_type: token_type::CUE_COMMAND,
                modifiers: 0,
            });
            cursor = name_start + cue.command.len();
        }

        // 3) スコープ: ScopedName.span (@名前 全体を 1 WORD トークン)
        if let Some(ref scope) = cue.scope {
            if scope.span.is_valid() {
                Self::add_token_from_span(&scope.span, source, token_type::WORD, 0, tokens);
                // カーソルを scope 後に進める
                let scope_end = scope.span.end_byte.saturating_sub(line_start);
                if scope_end > cursor + span_start_in_line {
                    cursor = scope_end - span_start_in_line;
                }
            }
        }

        // 4) 引数リスト
        if !cue.args.is_empty() {
            let remaining = &span_text[cursor..];
            // 開き括弧検出
            if let Some(paren_pos) = find_open_paren(remaining) {
                let abs_paren = cursor + paren_pos;
                let paren_char =
                    &span_text[abs_paren..abs_paren + char_len_at(span_text, abs_paren)];
                tokens.push(RawToken {
                    line: line0,
                    start_char: utf8_offset_to_utf16(line_text, base_offset + abs_paren),
                    length: utf8_len_to_utf16(paren_char),
                    token_type: token_type::OPERATOR,
                    modifiers: 0,
                });

                let inner_start = abs_paren + paren_char.len();
                let close_pos = find_close_paren(span_text, inner_start);
                let args_text = if let Some(cp) = close_pos {
                    &span_text[inner_start..cp]
                } else {
                    &span_text[inner_start..]
                };

                // 各引数をスキャン
                let mut arg_cursor = 0usize;
                for arg in &cue.args {
                    let remaining_args = &args_text[arg_cursor..];
                    // カンマ・空白スキップ
                    let skip = remaining_args
                        .find(|c: char| !c.is_whitespace() && c != '、' && c != ',')
                        .unwrap_or(0);
                    let arg_text_start = arg_cursor + skip;
                    let arg_remaining = &args_text[arg_text_start..];
                    let arg_end = find_arg_end(arg_remaining);
                    let arg_slice = &args_text[arg_text_start..arg_text_start + arg_end];
                    let arg_base = base_offset + inner_start + arg_text_start;

                    match arg {
                        CueArgToken::Ident(s) => {
                            if let Some(pos) = arg_slice.find(s.as_str()) {
                                tokens.push(RawToken {
                                    line: line0,
                                    start_char: utf8_offset_to_utf16(line_text, arg_base + pos),
                                    length: utf8_len_to_utf16(s),
                                    token_type: token_type::CUE_COMMAND,
                                    modifiers: 0,
                                });
                            }
                        }
                        CueArgToken::StringLiteral(s) => {
                            // 文字列リテラルはソース上では括弧付き 「...」 or "..."
                            // arg_slice に括弧を含む全テキストがあるので、中身を検索
                            if let Some(pos) = arg_slice.find(s.as_str()) {
                                tokens.push(RawToken {
                                    line: line0,
                                    start_char: utf8_offset_to_utf16(line_text, arg_base + pos),
                                    length: utf8_len_to_utf16(s),
                                    token_type: token_type::TALK,
                                    modifiers: 0,
                                });
                            }
                        }
                        CueArgToken::Integer(_) | CueArgToken::Float(_) => {
                            if let Some((start, end)) = find_number_literal(arg_slice) {
                                let num_text = &arg_slice[start..end];
                                tokens.push(RawToken {
                                    line: line0,
                                    start_char: utf8_offset_to_utf16(line_text, arg_base + start),
                                    length: utf8_len_to_utf16(num_text),
                                    token_type: token_type::NUMBER,
                                    modifiers: 0,
                                });
                            }
                        }
                        CueArgToken::AtRef(name) => {
                            // @name — ＠ or @ + name
                            let patterns = [format!("＠{}", name), format!("@{}", name)];
                            for pat in &patterns {
                                if let Some(pos) = arg_slice.find(pat.as_str()) {
                                    tokens.push(RawToken {
                                        line: line0,
                                        start_char: utf8_offset_to_utf16(line_text, arg_base + pos),
                                        length: utf8_len_to_utf16(pat),
                                        token_type: token_type::WORD,
                                        modifiers: 0,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                    arg_cursor = arg_text_start + arg_end;
                }

                // 閉じ括弧
                if let Some(cp) = close_pos {
                    let close_char = &span_text[cp..cp + char_len_at(span_text, cp)];
                    tokens.push(RawToken {
                        line: line0,
                        start_char: utf8_offset_to_utf16(line_text, base_offset + cp),
                        length: utf8_len_to_utf16(close_char),
                        token_type: token_type::OPERATOR,
                        modifiers: 0,
                    });
                }
            }
        }
    }

    fn visit_call_scene(cs: &CallScene, source: &str, tokens: &mut Vec<RawToken>) {
        if cs.span.is_valid() {
            Self::add_token_from_span(&cs.span, source, token_type::CALL, 0, tokens);
        }
    }

    fn visit_action_line(al: &ActionLine, source: &str, tokens: &mut Vec<RawToken>) {
        if al.span.is_valid() {
            let line = al.span.start_line;
            let line_text = get_line_text(source, line);

            // Find actual actor name position within line_text.
            // The action_line span includes leading pad (whitespace), so we
            // cannot simply use span.start_byte to locate the actor name.
            // Instead, search for the actor name substring in the line text.
            let actor_byte_start = if let Some(pos) = line_text.find(&al.actor) {
                pos
            } else {
                // Fallback: skip leading whitespace
                line_text.len() - line_text.trim_start().len()
            };
            let actor_start_utf16 = utf8_offset_to_utf16(line_text, actor_byte_start);
            let actor_len_utf16 = utf8_len_to_utf16(&al.actor);

            // Actor name token
            tokens.push(RawToken {
                line: (line - 1) as u32,
                start_char: actor_start_utf16,
                length: actor_len_utf16,
                token_type: token_type::ACTOR_NAME,
                modifiers: 0,
            });

            // Colon separator: located immediately after the actor name
            let colon_byte_offset = actor_byte_start + al.actor.len();
            // Skip optional whitespace between actor name and colon
            let after_actor = &line_text[colon_byte_offset..];
            let trimmed = after_actor.trim_start();
            let ws_bytes = after_actor.len() - trimmed.len();
            let colon_actual_offset = colon_byte_offset + ws_bytes;
            if colon_actual_offset < line_text.len() {
                let colon_char = line_text[colon_actual_offset..].chars().next();
                if let Some(ch) = colon_char {
                    if ch == '：' || ch == ':' {
                        let colon_start = utf8_offset_to_utf16(line_text, colon_actual_offset);
                        let colon_len = ch.len_utf16() as u32;
                        tokens.push(RawToken {
                            line: (line - 1) as u32,
                            start_char: colon_start,
                            length: colon_len,
                            token_type: token_type::OPERATOR,
                            modifiers: 0,
                        });
                    }
                }
            }
        }

        for action in &al.actions {
            Self::visit_action(action, source, tokens);
        }
    }

    fn visit_continue_action(ca: &ContinueAction, source: &str, tokens: &mut Vec<RawToken>) {
        for action in &ca.actions {
            Self::visit_action(action, source, tokens);
        }
    }

    fn visit_action(action: &Action, source: &str, tokens: &mut Vec<RawToken>) {
        match action {
            Action::Talk { span, .. } => {
                if span.is_valid() {
                    Self::add_token_from_span(span, source, token_type::TALK, 0, tokens);
                }
            }
            Action::WordRef { span, .. } => {
                if span.is_valid() {
                    Self::add_token_from_span(span, source, token_type::WORD, 0, tokens);
                }
            }
            Action::VarRef { span, .. } => {
                if span.is_valid() {
                    Self::add_token_from_span(span, source, token_type::VARIABLE, 0, tokens);
                }
            }
            Action::FnCall { span, .. } => {
                if span.is_valid() {
                    Self::add_token_from_span(span, source, token_type::WORD, 0, tokens);
                }
            }
            Action::SakuraScript { span, .. } => {
                if span.is_valid() {
                    Self::add_token_from_span(span, source, token_type::SAKURA_SCRIPT, 0, tokens);
                }
            }
            Action::Escape { span, .. } => {
                if span.is_valid() {
                    Self::add_token_from_span(span, source, token_type::ESCAPE, 0, tokens);
                }
            }
        }
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn add_token_from_span(
        span: &Span,
        source: &str,
        token_type: u32,
        modifiers: u32,
        tokens: &mut Vec<RawToken>,
    ) {
        if !span.is_valid() {
            return;
        }

        if span.start_line == span.end_line {
            let line = span.start_line;
            let line_text = get_line_text(source, line);
            let line_start = line_byte_offset(source, line);
            let start_in_line = span.start_byte.saturating_sub(line_start);
            let end_in_line = span
                .end_byte
                .saturating_sub(line_start)
                .min(line_text.len());
            if start_in_line >= end_in_line {
                return;
            }
            let text_slice = &line_text[start_in_line..end_in_line];

            tokens.push(RawToken {
                line: (line - 1) as u32,
                start_char: utf8_offset_to_utf16(line_text, start_in_line),
                length: utf8_len_to_utf16(text_slice),
                token_type,
                modifiers,
            });
        } else {
            for line_num in span.start_line..=span.end_line {
                let line_text = get_line_text(source, line_num);
                let line_start = line_byte_offset(source, line_num);

                let (start_in_line, end_in_line) = if line_num == span.start_line {
                    (span.start_byte.saturating_sub(line_start), line_text.len())
                } else if line_num == span.end_line {
                    (
                        0,
                        span.end_byte
                            .saturating_sub(line_start)
                            .min(line_text.len()),
                    )
                } else {
                    (0, line_text.len())
                };

                if start_in_line >= end_in_line {
                    continue;
                }
                let text_slice = &line_text[start_in_line..end_in_line];
                tokens.push(RawToken {
                    line: (line_num - 1) as u32,
                    start_char: utf8_offset_to_utf16(line_text, start_in_line),
                    length: utf8_len_to_utf16(text_slice),
                    token_type,
                    modifiers,
                });
            }
        }
    }
}
