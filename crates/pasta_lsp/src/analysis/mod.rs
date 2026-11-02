//! Semantic token analysis engine for the Pasta Language Server.
//!
//! Converts pasta_dsl AST nodes into LSP semantic tokens with
//! proper UTF-8 → UTF-16 position conversion.

mod text_utils;
mod token_types;
mod visit_action;
mod visit_expr;
mod visit_scope;

// Re-export public API (preserve existing import paths)
pub use text_utils::{get_line_text, line_byte_offset};
pub use token_types::*;

// ============================================================================
// Analysis Engine
// ============================================================================

/// 解析結果。直列化した形がそのまま JS へ渡る（欄の名前・順は拡張の型と同じ）
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AnalysisResult {
    /// エンコード済みセマンティックトークン（LSP deltaエンコーディング）
    pub tokens: Vec<SemanticToken>,
    /// パースエラーから生成されたDiagnostics
    pub diagnostics: Vec<Diagnostic>,
}

/// 診断
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
    pub range: Range,
    pub message: String,
    /// 常に 1（Error）。JS の形を保つために欄を残す
    pub severity: u32,
}

/// 範囲
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// 位置（0 始まり・UTF-16）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// 解析エンジン
pub struct AnalysisEngine;

impl AnalysisEngine {
    /// ドキュメント全体を解析し、トークンとDiagnosticsを生成する。
    ///
    /// 共有する状態を持たない純粋な関数。panic は包まない（wasm32 では
    /// trap になり、拡張の try/catch が受ける）。
    pub fn analyze(source: &str) -> AnalysisResult {
        // Ensure source ends with newline — the grammar requires every line
        // to terminate with eol (NEWLINE). Editors like VSCode may omit it.
        let source = if source.ends_with('\n') {
            std::borrow::Cow::Borrowed(source)
        } else {
            std::borrow::Cow::Owned(format!("{}\n", source))
        };
        let source: &str = &source;

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
                            message: e.message.clone(),
                            severity: 1,
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
}
