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
        // 宣言行の属性も構文木の位置で着色する（visit_decl_line は属性を出さない）
        for attr in &scene.attrs {
            Self::visit_attr(attr, source, tokens);
        }
        for word in &scene.words {
            Self::visit_keywords(word, source, tokens);
        }
        for item in &scene.actors {
            Self::visit_scene_actor_item(item, source, tokens);
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
            // 文法の attr は後ろの空白まで含むので、色は空白の手前まで
            let end_byte = source
                .get(attr.span.start_byte..attr.span.end_byte)
                .map_or(attr.span.end_byte, |t| {
                    attr.span.start_byte + t.trim_end().len()
                });
            let span = Span {
                end_byte,
                ..attr.span
            };
            Self::add_token_from_span(&span, source, token_type::DECORATOR, 0, tokens);
            // 次の属性が続く間は `＆` が来るので、出るのは行の最後の属性の後ろだけ
            push_trailing_comment(source, attr.span.start_line, attr.span.end_byte, tokens);
        }
    }

    /// アクター指定の行（`　％さくら、うにゅう`）は項目の色を出さず、行末のコメントだけ
    fn visit_scene_actor_item(item: &SceneActorItem, source: &str, tokens: &mut Vec<RawToken>) {
        if !item.span.is_valid() {
            return;
        }
        // 項目の後ろの `、` は span に入らない（最後の項目の後ろの `、` も文法が許す）
        let rest = source.get(item.span.end_byte..).unwrap_or("");
        let after = rest.trim_start();
        let after = after.strip_prefix(['、', '，', ',']).unwrap_or(after);
        let from = source.len() - after.len();
        push_trailing_comment(source, item.span.start_line, from, tokens);
    }

    fn visit_keywords(word: &KeyWords, source: &str, tokens: &mut Vec<RawToken>) {
        if word.span.is_valid() {
            // 重みの `＠数` を NUMBER、その間を WORD で出す。値の途中の `＠` は weight_spans に入らないので WORD のまま（要件 7.1・7.2）
            let (mut line, mut byte) = (word.span.start_line, word.span.start_byte);
            for w in &word.weight_spans {
                // add_token_from_span は行とバイトだけを読む
                let gap = Span {
                    start_line: line,
                    start_byte: byte,
                    end_line: w.start_line,
                    end_byte: w.start_byte,
                    ..*w
                };
                Self::add_token_from_span(&gap, source, token_type::WORD, 0, tokens);
                Self::add_token_from_span(w, source, token_type::NUMBER, 0, tokens);
                (line, byte) = (w.end_line, w.end_byte);
            }
            let rest = Span {
                start_line: line,
                start_byte: byte,
                ..word.span
            };
            Self::add_token_from_span(&rest, source, token_type::WORD, 0, tokens);
            // 囲まない値は `＃` 以降も span に入るので何も出ない（要件 4.4）
            push_trailing_comment(source, word.span.start_line, word.span.end_byte, tokens);
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
            Self::visit_close_fence(open_fence_line, close_fence_line, source, tokens);
        }
    }

    /// 閉じのフェンスの行: 開きと同じ数の `` ` `` だけを CODE_BLOCK にし、その後ろの
    /// 行末のコメントを COMMENT にする（本文は開きと同じ数の `` ` `` を含まない）
    fn visit_close_fence(
        open_line: usize,
        close_line: usize,
        source: &str,
        tokens: &mut Vec<RawToken>,
    ) {
        let fence_len = get_line_text(source, open_line)
            .trim_start()
            .chars()
            .take_while(|&c| c == '`')
            .count();
        let line_text = get_line_text(source, close_line);
        let found = (fence_len > 0).then(|| line_text.find(&"`".repeat(fence_len)));
        let Some(start) = found.flatten() else {
            Self::add_full_line_token(close_line, source, token_type::CODE_BLOCK, 0, tokens);
            return;
        };
        let end = start + fence_len;
        tokens.push(RawToken {
            line: (close_line - 1) as u32,
            start_char: utf8_offset_to_utf16(line_text, start),
            length: (end - start) as u32,
            token_type: token_type::CODE_BLOCK,
            modifiers: 0,
        });
        let line_start = line_byte_offset(source, close_line);
        push_trailing_comment(source, close_line, line_start + end, tokens);
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

    /// 宣言の行（`＊名前`・`＊`・`　・名前`・`％名前`）の、印と名前と、名前の直後の行末の
    /// コメントのトークンを出す。印は span の始まりから字下げを飛ばした所。属性は出さない
    /// （`visit_attr` が構文木の位置で出し、最後の属性の後ろのコメントもそちらが出す）。
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
        let name_end = decl_name_end(line_text, marker_start);
        let start_char = utf8_offset_to_utf16(line_text, marker_start);
        tokens.push(RawToken {
            line: (line - 1) as u32,
            start_char,
            length: utf8_offset_to_utf16(line_text, name_end) - start_char,
            token_type: marker_type,
            modifiers: 0,
        });
        // 属性が続くなら後ろは `＆` なので、ここでは何も出ない
        push_trailing_comment(source, line, line_start + name_end, tokens);
    }
}

/// 宣言の行の印と名前の終わり（行の中のバイト）。文法の `印 ~ id?` どおり、名前は
/// 印の直後から空白・`＆`・`＃` の手前まで。
fn decl_name_end(line_text: &str, marker_start: usize) -> usize {
    let after_marker = marker_start + char_len_at(line_text, marker_start);
    line_text[after_marker..]
        .find(|c: char| c.is_whitespace() || matches!(c, '＆' | '&' | '＃' | '#'))
        .map_or(line_text.len(), |p| after_marker + p)
}
