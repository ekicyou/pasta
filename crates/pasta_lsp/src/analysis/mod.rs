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

/// 報告 1 件を、幅の無い点の Error の診断にする（行・桁が無ければ 1 行目の頭。要件 11.2）
fn finding_to_diagnostic(finding: &pasta_check::validate::Finding, source: &str) -> Diagnostic {
    let line = finding.line.map_or(0, |l| l.saturating_sub(1));
    let character = finding.column.map_or(0, |c| {
        let line_text = get_line_text(source, line as usize + 1);
        text_utils::char_column_to_utf16(line_text, c.saturating_sub(1) as usize)
    });
    let point = Position { line, character };
    Diagnostic {
        range: Range {
            start: point,
            end: point,
        },
        message: finding.message.clone(),
        severity: 1,
    }
}

/// 解析エンジン
pub struct AnalysisEngine;

impl AnalysisEngine {
    /// ドキュメント全体を解析し、トークンとDiagnosticsを生成する。
    ///
    /// 共有する状態を持たない純粋な関数。panic は包まない（wasm32 では
    /// trap になり、拡張の try/catch が受ける）。
    pub fn analyze(source: &str) -> AnalysisResult {
        // Phase 0: Scan source lines for comment lines (AST does not preserve comments)
        let mut raw_tokens = Self::scan_comment_lines(source);

        // 受け取った文字のまま 1 回だけ解く（改行を補わない。保存時と同じ誤りを出す）
        let diagnostics = match pasta_dsl::parse_str(source, "<lsp>") {
            Ok(pasta_file) => {
                Self::visit_file_items(&pasta_file.items, source, &mut raw_tokens);
                vec![]
            }
            Err(e) => {
                // 色だけを部分解析の AST から作る（部分解析の誤りは読まない）
                let partial = pasta_dsl::parse_str_partial(source);
                Self::visit_file_items(&partial.items, source, &mut raw_tokens);
                let finding = pasta_check::validate::syntax_finding("<lsp>", e);
                vec![finding_to_diagnostic(&finding, source)]
            }
        };

        AnalysisResult {
            tokens: encode_tokens(&mut raw_tokens),
            diagnostics,
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

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_check::validate::{Finding, FindingKind};

    /// 位置の無い報告は 1 行目の頭の点にする（要件 11.2）
    #[test]
    fn finding_without_position_goes_to_file_head() {
        let finding = Finding::new(FindingKind::Syntax, "<lsp>", None, None, "構文エラー: x");
        let d = finding_to_diagnostic(&finding, "＊挨拶\n  さくら：やあ\n");
        let head = Position {
            line: 0,
            character: 0,
        };
        assert_eq!(
            d,
            Diagnostic {
                range: Range {
                    start: head,
                    end: head
                },
                message: "構文エラー: x".into(),
                severity: 1,
            }
        );
    }

    /// 行より長い桁は行の長さで止め、CRLF の `\r` は数えない
    #[test]
    fn column_past_line_end_is_clamped() {
        let finding = Finding::new(FindingKind::Syntax, "<lsp>", Some(1), Some(99), "x");
        let d = finding_to_diagnostic(&finding, "a𠮷\r\nb\r\n");
        assert_eq!(d.range.start.character, 3);
        assert_eq!(text_utils::char_column_to_utf16("a𠮷", 2), 3);
    }
}
