//! Semantic token analysis engine for the Pasta WASM analyzer (VSCode extension).
//!
//! Converts pasta_dsl AST nodes into LSP semantic tokens with
//! proper UTF-8 → UTF-16 position conversion.

mod scenes;
mod text_utils;
mod token_types;
mod visit_action;
mod visit_expr;
mod visit_scope;

// Re-export public API (preserve existing import paths)
pub use scenes::{SceneKind, SceneNode};
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
    /// シーンの並び（グローバルシーン ＞ ローカルシーン・文書の順）
    pub scenes: Vec<SceneNode>,
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
        let mut raw_tokens = Vec::new();
        // 受け取った文字のまま 1 回だけ解く（改行を補わない。保存時と同じ誤りを出す）
        let (diagnostics, failed_lines, scenes) = match pasta_dsl::parse_str(source, "<lsp>") {
            Ok(pasta_file) => {
                Self::visit_file_items(&pasta_file.items, source, &mut raw_tokens);
                let scenes = scenes::collect_scenes(&pasta_file.items, source);
                (vec![], vec![], scenes)
            }
            Err(e) => {
                // 色だけを部分解析の AST から作る（部分解析の誤りは読まない）
                let partial = pasta_dsl::parse_str_partial(source);
                Self::visit_file_items(&partial.items, source, &mut raw_tokens);
                let scenes = scenes::collect_scenes(&partial.items, source);
                let finding = pasta_check::validate::syntax_finding("<lsp>", e);
                (
                    vec![finding_to_diagnostic(&finding, source)],
                    partial.failed_lines,
                    scenes,
                )
            }
        };
        let comments = Self::scan_comment_lines(source, &failed_lines, &raw_tokens);
        raw_tokens.extend(comments);

        AnalysisResult {
            tokens: encode_tokens(&mut raw_tokens),
            diagnostics,
            scenes,
        }
    }

    // ========================================================================
    // Comment Line Scanner
    // ========================================================================

    /// 行全体のコメント（`#`・`＃` で始まる行）を COMMENT にする。コメントは AST に
    /// 残らないので文字から拾う。部分解析が解けなかった行（`failed_lines`・1 始まり・
    /// 半開区間）と、コードブロックの本文の行は除く。本文の行は、走査が出した
    /// CODE_BLOCK（フェンスの行に 1 つずつ）を行の順に 2 つずつ組にして決める。
    fn scan_comment_lines(
        source: &str,
        failed_lines: &[std::ops::Range<usize>],
        visited: &[RawToken],
    ) -> Vec<RawToken> {
        let mut fences: Vec<u32> = visited
            .iter()
            .filter(|t| t.token_type == token_type::CODE_BLOCK)
            .map(|t| t.line)
            .collect();
        fences.sort_unstable();
        let in_code_body = |line: u32| {
            fences
                .as_chunks::<2>()
                .0
                .iter()
                .any(|[open, close]| *open < line && line < *close)
        };
        let skipped = |line_idx: usize| {
            failed_lines.iter().any(|r| r.contains(&(line_idx + 1)))
                || in_code_body(line_idx as u32)
        };
        let mut tokens = Vec::new();
        for (line_idx, line_text) in source.lines().enumerate() {
            let trimmed = line_text.trim_start();
            if !skipped(line_idx) && (trimmed.starts_with('＃') || trimmed.starts_with('#')) {
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
