//! Semantic token analysis engine for the Pasta Language Server.
//!
//! Converts pasta_dsl AST nodes into LSP semantic tokens with
//! proper UTF-8 → UTF-16 position conversion.

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Position, Range, SemanticToken, SemanticTokenModifier,
    SemanticTokenType, SemanticTokensLegend,
};

use pasta_dsl::parser::ast::*;

// ============================================================================
// Token Type Definitions
// ============================================================================

/// Pasta DSL固有のセマンティックトークンタイプ
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::COMMENT,             // 0: コメント (#/＃)
    SemanticTokenType::NAMESPACE,           // 1: グローバルシーンマーカー (*/＊)
    SemanticTokenType::new("scene"),         // 2: ローカルシーンマーカー (-/・)
    SemanticTokenType::DECORATOR,           // 3: 属性マーカー (&/＆)
    SemanticTokenType::new("word"),          // 4: 単語マーカー (@/＠)
    SemanticTokenType::VARIABLE,            // 5: 変数マーカー ($/＄)
    SemanticTokenType::new("call"),          // 6: Callマーカー (>/＞)
    SemanticTokenType::new("actor"),         // 7: アクター辞書マーカー (%/％)
    SemanticTokenType::new("actorName"),     // 8: アクター名 (：の前)
    SemanticTokenType::new("codeBlock"),     // 9: Luaコードブロック
    SemanticTokenType::STRING,              // 10: 文字列リテラル（Talk）
    SemanticTokenType::new("sakuraScript"), // 11: さくらスクリプト
    SemanticTokenType::new("escape"),       // 12: エスケープシーケンス
    SemanticTokenType::OPERATOR,            // 13: コロン区切り
];

/// トークンモディファイア
pub const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,   // 0: 宣言
    SemanticTokenModifier::DEFINITION,    // 1: 定義
    SemanticTokenModifier::new("global"), // 2: グローバルスコープ
];

/// Token type indices for convenience
pub mod token_type {
    pub const COMMENT: u32 = 0;
    pub const NAMESPACE: u32 = 1;
    pub const SCENE: u32 = 2;
    pub const DECORATOR: u32 = 3;
    pub const WORD: u32 = 4;
    pub const VARIABLE: u32 = 5;
    pub const CALL: u32 = 6;
    pub const ACTOR: u32 = 7;
    pub const ACTOR_NAME: u32 = 8;
    pub const CODE_BLOCK: u32 = 9;
    pub const STRING: u32 = 10;
    pub const SAKURA_SCRIPT: u32 = 11;
    pub const ESCAPE: u32 = 12;
    pub const OPERATOR: u32 = 13;
}

/// Generate the SemanticTokensLegend for registration with the LSP client.
pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: TOKEN_MODIFIERS.to_vec(),
    }
}

// ============================================================================
// Raw Token (intermediate representation)
// ============================================================================

/// 中間表現: 絶対位置トークン（AST走査で生成）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawToken {
    /// 0-based行番号
    pub line: u32,
    /// 0-based UTF-16列オフセット
    pub start_char: u32,
    /// UTF-16コードユニット数
    pub length: u32,
    /// TOKEN_TYPESのインデックス
    pub token_type: u32,
    /// TOKEN_MODIFIERSのビットマスク
    pub modifiers: u32,
}

// ============================================================================
// UTF-8 → UTF-16 Conversion
// ============================================================================

/// UTF-8テキストの行内バイトオフセットをUTF-16コードユニットオフセットに変換
pub fn utf8_offset_to_utf16(line_text: &str, byte_offset: usize) -> u32 {
    if byte_offset == 0 {
        return 0;
    }
    let clamped = byte_offset.min(line_text.len());
    line_text[..clamped].encode_utf16().count() as u32
}

/// UTF-8テキストのバイト長をUTF-16コードユニット数に変換
pub fn utf8_len_to_utf16(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

// ============================================================================
// Analysis Engine
// ============================================================================

/// 解析結果
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    /// エンコード済みセマンティックトークン（LSP deltaエンコーディング）
    pub tokens: Vec<SemanticToken>,
    /// パースエラーから生成されたDiagnostics
    pub diagnostics: Vec<Diagnostic>,
}

/// 解析エンジン
pub struct AnalysisEngine;

impl AnalysisEngine {
    /// ドキュメント全体を解析し、トークンとDiagnosticsを生成する
    pub fn analyze(source: &str) -> AnalysisResult {
        // Phase 1: Full parse
        match pasta_dsl::parse_str(source, "<lsp>") {
            Ok(pasta_file) => {
                let mut raw_tokens = Vec::new();
                Self::visit_file_items(&pasta_file.items, source, &mut raw_tokens);
                let tokens = encode_tokens(&mut raw_tokens);
                AnalysisResult {
                    tokens,
                    diagnostics: vec![],
                }
            }
            Err(_) => {
                // Phase 2/3: Partial parse
                let partial = pasta_dsl::parse_str_partial(source);
                let mut raw_tokens = Vec::new();
                Self::visit_file_items(&partial.items, source, &mut raw_tokens);
                let tokens = encode_tokens(&mut raw_tokens);

                let diagnostics = partial
                    .errors
                    .iter()
                    .map(|e| {
                        let line = if e.line > 0 { e.line - 1 } else { 0 };
                        Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: line as u32,
                                    character: 0,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: e.message.clone(),
                            ..Default::default()
                        }
                    })
                    .collect();

                AnalysisResult {
                    tokens,
                    diagnostics,
                }
            }
        }
    }

    // ========================================================================
    // AST Visitors
    // ========================================================================

    fn visit_file_items(items: &[FileItem], source: &str, tokens: &mut Vec<RawToken>) {
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
            Self::add_token_from_span(&scene.span, source, token_type::NAMESPACE, 0, tokens);
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
        if scene.span.is_valid() {
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
            Self::add_token_from_span(&actor.span, source, token_type::ACTOR, 0, tokens);
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
        if vs.span.is_valid() {
            Self::add_token_from_span(&vs.span, source, token_type::VARIABLE, 0, tokens);
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
                    Self::add_token_from_span(span, source, token_type::STRING, 0, tokens);
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
            let end_in_line = span.end_byte.saturating_sub(line_start).min(line_text.len());
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
                        span.end_byte.saturating_sub(line_start).min(line_text.len()),
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

// ============================================================================
// Delta Encoding
// ============================================================================

/// RawToken列をLSP deltaエンコーディングに変換
pub fn encode_tokens(raw: &mut [RawToken]) -> Vec<SemanticToken> {
    if raw.is_empty() {
        return vec![];
    }

    raw.sort_by(|a, b| a.line.cmp(&b.line).then(a.start_char.cmp(&b.start_char)));

    let mut result = Vec::with_capacity(raw.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for token in raw.iter() {
        let delta_line = token.line - prev_line;
        let delta_start = if delta_line == 0 {
            token.start_char - prev_start
        } else {
            token.start_char
        };

        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: token.modifiers,
        });

        prev_line = token.line;
        prev_start = token.start_char;
    }

    result
}

// ============================================================================
// Source Text Utilities
// ============================================================================

/// Get the text of a specific line (1-based line number).
fn get_line_text(source: &str, line: usize) -> &str {
    source.lines().nth(line - 1).unwrap_or("")
}

/// Get the byte offset where a line starts (1-based line number).
fn line_byte_offset(source: &str, line: usize) -> usize {
    let mut offset = 0;
    for (i, l) in source.lines().enumerate() {
        if i + 1 == line {
            return offset;
        }
        offset += l.len() + 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_offset_to_utf16_ascii() {
        let text = "hello";
        assert_eq!(utf8_offset_to_utf16(text, 0), 0);
        assert_eq!(utf8_offset_to_utf16(text, 3), 3);
        assert_eq!(utf8_offset_to_utf16(text, 5), 5);
    }

    #[test]
    fn test_utf8_offset_to_utf16_japanese() {
        let text = "＊挨拶";
        assert_eq!(utf8_offset_to_utf16(text, 0), 0);
        assert_eq!(utf8_offset_to_utf16(text, 3), 1);
        assert_eq!(utf8_offset_to_utf16(text, 6), 2);
        assert_eq!(utf8_offset_to_utf16(text, 9), 3);
    }

    #[test]
    fn test_utf8_offset_to_utf16_emoji() {
        let text = "a😀b";
        assert_eq!(utf8_offset_to_utf16(text, 0), 0);
        assert_eq!(utf8_offset_to_utf16(text, 1), 1);
        assert_eq!(utf8_offset_to_utf16(text, 5), 3);
        assert_eq!(utf8_offset_to_utf16(text, 6), 4);
    }

    #[test]
    fn test_encode_tokens_empty() {
        let mut raw: Vec<RawToken> = vec![];
        assert!(encode_tokens(&mut raw).is_empty());
    }

    #[test]
    fn test_encode_tokens_single() {
        let mut raw = vec![RawToken {
            line: 0,
            start_char: 0,
            length: 3,
            token_type: token_type::COMMENT,
            modifiers: 0,
        }];
        let encoded = encode_tokens(&mut raw);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0].delta_line, 0);
        assert_eq!(encoded[0].delta_start, 0);
        assert_eq!(encoded[0].length, 3);
    }

    #[test]
    fn test_encode_tokens_delta() {
        let mut raw = vec![
            RawToken { line: 0, start_char: 0, length: 3, token_type: 0, modifiers: 0 },
            RawToken { line: 0, start_char: 5, length: 2, token_type: 1, modifiers: 0 },
            RawToken { line: 2, start_char: 1, length: 4, token_type: 2, modifiers: 0 },
        ];
        let encoded = encode_tokens(&mut raw);
        assert_eq!(encoded.len(), 3);
        assert_eq!(encoded[0].delta_line, 0);
        assert_eq!(encoded[0].delta_start, 0);
        assert_eq!(encoded[1].delta_line, 0);
        assert_eq!(encoded[1].delta_start, 5);
        assert_eq!(encoded[2].delta_line, 2);
        assert_eq!(encoded[2].delta_start, 1);
    }

    #[test]
    fn test_analyze_simple_scene() {
        let source = "＊挨拶\n  Alice：こんにちは\n";
        let result = AnalysisEngine::analyze(source);
        assert!(!result.tokens.is_empty());
        assert!(result.diagnostics.is_empty());
    }
}
