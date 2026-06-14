//! AST visitor methods for semantic token generation: action/cue visitors and
//! shared span→token helpers.
//!
//! This file exceeds the 300-line guideline as documented in the design spec
//! (guideline exception: tightly-coupled visitor methods).

use pasta_dsl::parser::ast::*;

use super::text_utils::*;
use super::token_types::*;

// ============================================================================
// AST Visitors (split impl on AnalysisEngine): action / cue visitors + helpers
// ============================================================================

impl super::AnalysisEngine {
    pub(super) fn visit_local_scene_item(item: &LocalSceneItem, source: &str, tokens: &mut Vec<RawToken>) {
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
        let (line_text, span_text, span_start_in_line) = Self::span_line_window(&cue.span, source);
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
        if let Some(ref scope) = cue.scope
            && scope.span.is_valid()
        {
            Self::add_token_from_span(&scope.span, source, token_type::WORD, 0, tokens);
            // カーソルを scope 後に進める
            let scope_end = scope
                .span
                .end_byte
                .saturating_sub(line_byte_offset(source, line));
            if scope_end > cursor + span_start_in_line {
                cursor = scope_end - span_start_in_line;
            }
        }

        // 4) 引数リスト
        if !cue.args.is_empty() {
            let remaining = &span_text[cursor..];
            // 開き括弧検出
            if let Some(paren_pos) = find_open_paren(remaining) {
                let abs_paren = cursor + paren_pos;
                let paren_len = Self::push_char_op_token(
                    span_text,
                    abs_paren,
                    base_offset,
                    line0,
                    tokens,
                    line_text,
                );

                let inner_start = abs_paren + paren_len;
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
                    Self::push_char_op_token(span_text, cp, base_offset, line0, tokens, line_text);
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
                if let Some(ch) = colon_char
                    && (ch == '：' || ch == ':')
                {
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

    /// `byte` を `[0, text.len()]` に収め、UTF-8 文字境界まで切り下げる。
    /// これにより `text[..byte]` / `text[byte..]` のバイトスライスが
    /// マルチバイト文字の途中で割れてパニックすることがなくなる。
    ///
    /// span のバイトオフセットは完全パース経路ではフルソース座標で行に整合
    /// するが、部分パース（`parse_str_partial`）のフォールバック経路では
    /// チャンク相対座標になり、対象行の外やマルチバイト文字の途中を
    /// 指しうる。本ヘルパーでどちらの経路でも安全に描画できるようにする。
    fn safe_boundary(text: &str, byte: usize) -> usize {
        let mut b = byte.min(text.len());
        while b > 0 && !text.is_char_boundary(b) {
            b -= 1;
        }
        b
    }

    /// span の開始行テキストと、行内に切り詰めた span スライスを取り出す。
    /// 戻り値: (line_text, span_text, span_start_in_line)
    ///
    /// span_start_in_line / span_end_in_line は char 境界へスナップし、
    /// かつ start <= end を保証する（不整合な span でもスライスが panic しない）。
    pub(super) fn span_line_window<'s>(span: &Span, source: &'s str) -> (&'s str, &'s str, usize) {
        let line = span.start_line;
        let line_text = get_line_text(source, line);
        let line_start = line_byte_offset(source, line);
        let span_end_in_line = Self::safe_boundary(line_text, span.end_byte.saturating_sub(line_start));
        let span_start_in_line =
            Self::safe_boundary(line_text, span.start_byte.saturating_sub(line_start)).min(span_end_in_line);
        (
            line_text,
            &line_text[span_start_in_line..span_end_in_line],
            span_start_in_line,
        )
    }

    /// `text[pos]` の 1 文字（括弧等）を OPERATOR トークンとして追加し、
    /// その文字のバイト長を返す。
    pub(super) fn push_char_op_token(
        text: &str,
        pos: usize,
        base_offset: usize,
        line: u32,
        tokens: &mut Vec<RawToken>,
        line_text: &str,
    ) -> usize {
        let ch = &text[pos..pos + char_len_at(text, pos)];
        tokens.push(RawToken {
            line,
            start_char: utf8_offset_to_utf16(line_text, base_offset + pos),
            length: utf8_len_to_utf16(ch),
            token_type: token_type::OPERATOR,
            modifiers: 0,
        });
        ch.len()
    }

    pub(super) fn add_token_from_span(
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
            let end_in_line =
                Self::safe_boundary(line_text, span.end_byte.saturating_sub(line_start));
            let start_in_line =
                Self::safe_boundary(line_text, span.start_byte.saturating_sub(line_start));
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
                    (
                        Self::safe_boundary(line_text, span.start_byte.saturating_sub(line_start)),
                        line_text.len(),
                    )
                } else if line_num == span.end_line {
                    (
                        0,
                        Self::safe_boundary(line_text, span.end_byte.saturating_sub(line_start)),
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
