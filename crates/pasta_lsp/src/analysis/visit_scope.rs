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
            let token = Self::marker_line_token(&scene.span, source, token_type::NAMESPACE);
            if token.length > 0 {
                tokens.push(token);
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
            tokens.push(Self::marker_line_token(
                &scene.span,
                source,
                token_type::SCENE,
            ));
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
            let token = Self::marker_line_token(&actor.span, source, token_type::ACTOR);
            if token.length > 0 {
                tokens.push(token);
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
        let close_fence_line = Self::last_line_in_byte_range(source, cb.span.end_byte, open_fence_line);

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

    /// スコープマーカー行（＊/・/％）のトークンを構築する: span 開始位置から行末まで。
    /// `length` は 0 になりうる（span 開始が行末以降）— 扱いは呼び出し側が決める。
    fn marker_line_token(span: &Span, source: &str, token_type: u32) -> RawToken {
        let line = span.start_line;
        let line_text = get_line_text(source, line);
        let line_start = line_byte_offset(source, line);
        let start_in_line = span.start_byte.saturating_sub(line_start);
        let start_char = utf8_offset_to_utf16(line_text, start_in_line);
        let line_len = utf8_len_to_utf16(line_text);
        RawToken {
            line: (line - 1) as u32,
            start_char,
            length: line_len.saturating_sub(start_char),
            token_type,
            modifiers: 0,
        }
    }
}
