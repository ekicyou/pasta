//! AST visitor methods for semantic token generation: scope-level visitors.
//!
//! This file exceeds the 300-line guideline as documented in the design spec
//! (guideline exception: tightly-coupled visitor methods).

use pasta_dsl::parser::ast::*;

use super::text_utils::*;
use super::token_types::*;

// ============================================================================
// AST Visitors (split impl on AnalysisEngine): scope-level
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
            Self::visit_decl_line(&scene.span, source, token_type::NAMESPACE, tokens);
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
            Self::visit_decl_line(&scene.span, source, token_type::SCENE, tokens);
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
            Self::visit_decl_line(&actor.span, source, token_type::ACTOR, tokens);
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
        if !cb.span.is_valid() {
            return;
        }
        // Emit a `codeBlock` token ONLY on the opening fence line and the closing
        // fence line. Body lines (between the fences) get no token, so that an
        // editor-side injection grammar can colorize the body as Lua.
        //
        // Note: the parser's `cb.span` covers the whole block, but its `end_line`
        // lands on the position *after* the closing fence's trailing newline (i.e.
        // the start of the next line), not on the closing fence line itself.
        // Derive the actual closing fence line from the span's byte range instead.
        let open_fence_line = cb.span.start_line;
        let close_fence_line =
            Self::last_line_in_byte_range(source, cb.span.end_byte, open_fence_line);

        Self::add_full_line_token(open_fence_line, source, token_type::CODE_BLOCK, 0, tokens);
        // A real fenced block has distinct open/close fence lines. Guard against a
        // degenerate single-line span to avoid emitting a duplicate token.
        if close_fence_line != open_fence_line {
            Self::add_full_line_token(close_fence_line, source, token_type::CODE_BLOCK, 0, tokens);
        }
    }

    /// Find the closing fence line for a code block: the largest (1-based) line
    /// number whose line start byte is strictly less than `end_byte`, i.e. the
    /// last line that actually contains span content. Falls back to `min_line`
    /// when the range is degenerate.
    fn last_line_in_byte_range(source: &str, end_byte: usize, min_line: usize) -> usize {
        if end_byte == 0 {
            return min_line;
        }
        // The last content byte is end_byte - 1; the line containing it is the
        // closing fence line. Walk forward counting newlines up to that byte.
        let last_byte = end_byte - 1;
        let bytes = source.as_bytes();
        let limit = last_byte.min(bytes.len());
        let mut line = 1usize;
        for &b in &bytes[..limit] {
            if b == b'\n' {
                line += 1;
            }
        }
        line.max(min_line)
    }

    /// Emit a token covering the entire content of a single (1-based) line.
    /// Used for code-block fence lines, where the whole fence line is highlighted.
    fn add_full_line_token(
        line: usize,
        source: &str,
        token_type: u32,
        modifiers: u32,
        tokens: &mut Vec<RawToken>,
    ) {
        let line_text = get_line_text(source, line);
        if line_text.is_empty() {
            return;
        }
        tokens.push(RawToken {
            line: (line - 1) as u32,
            start_char: 0,
            length: utf8_len_to_utf16(line_text),
            token_type,
            modifiers,
        });
    }

    /// 宣言の行（`＊名前`・`＊`・`　・名前`・`％名前`）の、印と名前・属性ごと・行末の
    /// コメントのトークンを出す。印は span の始まりから字下げを飛ばした所。
    fn visit_decl_line(span: &Span, source: &str, marker_type: u32, tokens: &mut Vec<RawToken>) {
        let line = span.start_line;
        let line_text = get_line_text(source, line);
        let line_start = line_byte_offset(source, line);
        let from = span
            .start_byte
            .saturating_sub(line_start)
            .min(line_text.len());
        let marker_start = from + (line_text[from..].len() - line_text[from..].trim_start().len());
        if marker_start >= line_text.len() {
            return;
        }
        let parts = scan_decl_line(line_text, marker_start);
        let mut push = |range: std::ops::Range<usize>, token_type: u32| {
            let start_char = utf8_offset_to_utf16(line_text, range.start);
            tokens.push(RawToken {
                line: (line - 1) as u32,
                start_char,
                length: utf8_offset_to_utf16(line_text, range.end) - start_char,
                token_type,
                modifiers: 0,
            });
        };
        push(marker_start..parts.name_end, marker_type);
        for attr in &parts.attrs {
            push(attr.clone(), token_type::DECORATOR);
        }
        let body_end = parts.attrs.last().map_or(parts.name_end, |a| a.end);
        push_trailing_comment(source, line, line_start + body_end, tokens);
    }
}

/// 宣言の行の、印と名前の終わりと、属性ごとの範囲（どれも行の中のバイト）
struct DeclParts {
    name_end: usize,
    attrs: Vec<std::ops::Range<usize>>,
}

/// 解けた宣言の行を文法の `印 ~ id? ~ s ~ attrs? ~ or_comment_eol` どおりに読む。
/// 名前は印の直後から空白・`＆`・`＃` の手前まで。属性は `＆ id s ： s 値`（値が
/// `「`・`"` で始まるときだけ引用、ほかは `：`・`＆`・`＃`・空白の手前まで）。
fn scan_decl_line(line_text: &str, marker_start: usize) -> DeclParts {
    let is_attr = |c: char| matches!(c, '＆' | '&');
    let is_hash = |c: char| matches!(c, '＃' | '#');
    let is_kv = |c: char| matches!(c, '：' | ':');
    let after_marker = marker_start + char_len_at(line_text, marker_start);
    let name_end = line_text[after_marker..]
        .find(|c: char| c.is_whitespace() || is_attr(c) || is_hash(c))
        .map_or(line_text.len(), |p| after_marker + p);
    let skip_ws = |i: usize| line_text.len() - line_text[i..].trim_start().len();
    let mut attrs = Vec::new();
    let mut i = skip_ws(name_end);
    while line_text[i..].starts_with(is_attr) {
        let start = i;
        // id ~ s ~ kv_marker ~ s（id に `：` は入らない）
        let Some(p) = line_text[i..].find(is_kv) else {
            break;
        };
        i = skip_ws(i + p + char_len_at(line_text, i + p));
        i = if line_text[i..].starts_with(['「', '"']) {
            string_literal_end(line_text, i)
        } else {
            line_text[i..]
                .find(|c: char| c.is_whitespace() || is_kv(c) || is_attr(c) || is_hash(c))
                .map_or(line_text.len(), |p| i + p)
        };
        attrs.push(start..i);
        i = skip_ws(i);
    }
    DeclParts { name_end, attrs }
}
