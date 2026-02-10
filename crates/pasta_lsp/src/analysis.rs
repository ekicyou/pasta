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
    SemanticTokenType::new("scene"),        // 2: ローカルシーンマーカー (-/・)
    SemanticTokenType::DECORATOR,           // 3: 属性マーカー (&/＆)
    SemanticTokenType::new("word"),         // 4: 単語マーカー (@/＠)
    SemanticTokenType::VARIABLE,            // 5: 変数マーカー ($/＄)
    SemanticTokenType::new("call"),         // 6: Callマーカー (>/＞)
    SemanticTokenType::new("actor"),        // 7: アクター辞書マーカー (%/％)
    SemanticTokenType::new("actorName"),    // 8: アクター名 (：の前)
    SemanticTokenType::new("codeBlock"),    // 9: Luaコードブロック
    SemanticTokenType::new("talk"),         // 10: 文字列リテラル（Talk）
    SemanticTokenType::new("sakuraScript"), // 11: さくらスクリプト
    SemanticTokenType::new("escape"),       // 12: エスケープシーケンス
    SemanticTokenType::OPERATOR,            // 13: コロン区切り
    SemanticTokenType::NUMBER,              // 14: 数値リテラル
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
    pub const TALK: u32 = 10;
    pub const SAKURA_SCRIPT: u32 = 11;
    pub const ESCAPE: u32 = 12;
    pub const OPERATOR: u32 = 13;
    pub const NUMBER: u32 = 14;
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
        // Phase 0: Scan source lines for comment lines (AST does not preserve comments)
        let mut raw_tokens = Self::scan_comment_lines(source);

        // Phase 1: Full parse
        match pasta_dsl::parse_str(source, "<lsp>") {
            Ok(pasta_file) => {
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
    // Comment Line Scanner
    // ========================================================================

    /// Scan source text line-by-line to detect comment lines.
    /// Comments start with `#` or `＃` (optionally preceded by whitespace).
    /// The parser grammar treats these as silent `blank_line` rules, so they
    /// don't appear in the AST.  We detect them here and emit COMMENT tokens.
    fn scan_comment_lines(source: &str) -> Vec<RawToken> {
        let mut tokens = Vec::new();
        for (line_idx, line_text) in source.lines().enumerate() {
            let trimmed = line_text.trim_start();
            if trimmed.starts_with('＃') || trimmed.starts_with('#') {
                // Emit a COMMENT token for the entire line
                let line_len = utf8_len_to_utf16(line_text);
                if line_len > 0 {
                    tokens.push(RawToken {
                        line: line_idx as u32,
                        start_char: 0,
                        length: line_len,
                        token_type: token_type::COMMENT,
                        modifiers: 0,
                    });
                }
            }
        }
        tokens
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

        // 2) Variable name
        let name = &vs.name;
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
/// Returns the line content without trailing \r or \n.
fn get_line_text(source: &str, line: usize) -> &str {
    let (text, _) = get_line_text_and_offset(source, line);
    text
}

/// Get the byte offset where a line starts (1-based line number).
fn line_byte_offset(source: &str, line: usize) -> usize {
    let (_, offset) = get_line_text_and_offset(source, line);
    offset
}

/// Internal helper: split source into lines manually, handling both \n and \r\n.
/// Returns (line_text_without_eol, line_start_byte_offset) for the given 1-based line.
fn get_line_text_and_offset(source: &str, line: usize) -> (&str, usize) {
    let bytes = source.as_bytes();
    let mut current_line = 1usize;

    if line == 1 {
        // Fast path for first line
        let end = memchr_newline(bytes, 0);
        let text = strip_cr(&source[0..end]);
        return (text, 0);
    }

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            current_line += 1;
            i += 1;
            if current_line == line {
                let end = memchr_newline(bytes, i);
                let text = strip_cr(&source[i..end]);
                return (text, i);
            }
        } else {
            i += 1;
        }
    }

    // Requested line not found — return empty
    ("", source.len())
}

/// Find the next \n (or end of slice) starting from `start`.
#[inline]
fn memchr_newline(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Strip a trailing \r if present (for CRLF handling).
#[inline]
fn strip_cr(s: &str) -> &str {
    s.strip_suffix('\r').unwrap_or(s)
}

// ============================================================================
// Expression Text Scanning Helpers
// ============================================================================

/// Find a number literal (full-width or half-width digits, with optional decimal point)
/// at the start of the text (after skipping whitespace).
/// Returns (start_byte, end_byte) within text.
fn find_number_literal(text: &str) -> Option<(usize, usize)> {
    let trimmed_start = text.len() - text.trim_start().len();
    let mut chars = text[trimmed_start..].char_indices().peekable();
    let mut started = false;
    let mut end = trimmed_start;

    while let Some(&(i, c)) = chars.peek() {
        if is_digit_char(c) || (started && (c == '.' || c == '．')) {
            started = true;
            end = trimmed_start + i + c.len_utf8();
            chars.next();
        } else if !started {
            break;
        } else {
            break;
        }
    }

    if started {
        Some((trimmed_start, end))
    } else {
        None
    }
}

/// Check if a character is a digit (half-width or full-width).
#[inline]
fn is_digit_char(c: char) -> bool {
    c.is_ascii_digit() || ('０'..='９').contains(&c)
}

/// Find an opening parenthesis (half-width or full-width) in text.
/// Returns byte position.
fn find_open_paren(text: &str) -> Option<usize> {
    for (i, c) in text.char_indices() {
        if c == '(' || c == '（' {
            return Some(i);
        }
    }
    None
}

/// Find matching closing parenthesis, respecting nesting.
/// `start` is the byte offset after the opening paren.
fn find_close_paren(text: &str, start: usize) -> Option<usize> {
    let mut depth = 1i32;
    for (i, c) in text[start..].char_indices() {
        match c {
            '(' | '（' => depth += 1,
            ')' | '）' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Get the byte length of the char at the given byte position.
#[inline]
fn char_len_at(text: &str, byte_pos: usize) -> usize {
    text[byte_pos..].chars().next().map_or(1, |c| c.len_utf8())
}

/// Find the position and byte-length of a binary operator in the text.
/// Skips operators inside parentheses.
fn find_binary_op(text: &str, op_chars: &[&str]) -> Option<(usize, usize)> {
    let mut depth = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '（' => depth += 1,
            ')' | '）' => depth -= 1,
            _ if depth == 0 => {
                for &op in op_chars {
                    if text[i..].starts_with(op) {
                        return Some((i, op.len()));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Find the end of an argument in a comma/、-separated list.
/// Returns byte offset within the text where the argument ends.
fn find_arg_end(text: &str) -> usize {
    let mut depth = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '（' => depth += 1,
            ')' | '）' => depth -= 1,
            '、' | ',' if depth == 0 => return i,
            _ => {}
        }
    }
    text.len()
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
            RawToken {
                line: 0,
                start_char: 0,
                length: 3,
                token_type: 0,
                modifiers: 0,
            },
            RawToken {
                line: 0,
                start_char: 5,
                length: 2,
                token_type: 1,
                modifiers: 0,
            },
            RawToken {
                line: 2,
                start_char: 1,
                length: 4,
                token_type: 2,
                modifiers: 0,
            },
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

    #[test]
    fn test_analyze_crlf_does_not_panic() {
        // CRLF line endings must not cause panics (e.g., byte offset miscalculation)
        let source = "＃コメント\r\n＊挨拶\r\n  Alice：こんにちは\r\n";
        let result = AnalysisEngine::analyze(source);
        // Should produce tokens without panicking
        assert!(
            !result.tokens.is_empty(),
            "CRLF input should produce tokens"
        );
    }

    #[test]
    fn test_analyze_crlf_actor_scope() {
        let source = "＃アクター辞書\r\n％さくら\r\n　＠通常：\\s[0]\r\n";
        let result = AnalysisEngine::analyze(source);
        // Must not panic; may produce tokens or diagnostics
        let _total = result.tokens.len() + result.diagnostics.len();
    }

    #[test]
    fn test_line_byte_offset_lf() {
        let source = "abc\ndef\nghi";
        assert_eq!(line_byte_offset(source, 1), 0);
        assert_eq!(line_byte_offset(source, 2), 4);
        assert_eq!(line_byte_offset(source, 3), 8);
    }

    #[test]
    fn test_line_byte_offset_crlf() {
        let source = "abc\r\ndef\r\nghi";
        assert_eq!(line_byte_offset(source, 1), 0);
        assert_eq!(line_byte_offset(source, 2), 5); // "abc\r\n" = 5 bytes
        assert_eq!(line_byte_offset(source, 3), 10); // "abc\r\ndef\r\n" = 10 bytes
    }

    #[test]
    fn test_get_line_text_strips_cr() {
        let source = "hello\r\nworld\r\n";
        assert_eq!(get_line_text(source, 1), "hello");
        assert_eq!(get_line_text(source, 2), "world");
    }
}
