//! Partial parse support for Pasta DSL.
//!
//! Provides `parse_str_partial()` which attempts 3-phase fallback parsing:
//! - Phase 1: Full parse via `parse_str()`
//! - Phase 2: Scope boundary split and per-chunk parse
//! - Phase 3: Line-by-line fallback with rule inference

use crate::parser::ast::{FileItem, Span};
use crate::parser::{parse_str, Rule};

/// 部分パース結果
#[derive(Debug, Clone)]
pub struct PartialParseResult {
    /// パース成功した部分のASTアイテム
    pub items: Vec<FileItem>,
    /// 各行/スコープのパースエラー
    pub errors: Vec<PartialParseError>,
}

/// 部分パースエラー
#[derive(Debug, Clone)]
pub struct PartialParseError {
    /// エラーが発生した行番号（1-based）
    pub line: usize,
    /// エラーメッセージ
    pub message: String,
    /// エラー範囲のSpan（取得できた場合）
    pub span: Option<Span>,
}

/// 行頭パターンからpest Ruleを推論する
pub fn infer_rule_from_line(line: &str) -> Option<Rule> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let first_char = trimmed.chars().next()?;
    match first_char {
        '＊' | '*' => Some(Rule::global_scene_scope),
        '・' | '-' => Some(Rule::local_scene_line),
        '＆' | '&' => Some(Rule::file_attr_line),
        '＠' | '@' => Some(Rule::file_word_line),
        '％' | '%' => Some(Rule::actor_scope),
        '＄' | '$' => Some(Rule::var_set_line),
        '＞' | '>' => Some(Rule::call_scene_line),
        '!' | '！' => Some(Rule::cue_cmd_line),
        '＃' | '#' => Some(Rule::or_comment_eol),
        '`' if trimmed.starts_with("```") => Some(Rule::code_scope),
        _ => {
            // Check if it looks like an action line (identifier followed by colon)
            if trimmed.contains('：') || trimmed.contains(':') {
                Some(Rule::action_line)
            } else {
                None
            }
        }
    }
}

/// スコープ境界マーカーで分割するためのチャンク
#[derive(Debug)]
struct SourceChunk {
    text: String,
    start_line: usize, // 1-based
}

/// ソースをスコープ境界マーカーで分割する
fn split_by_scope_markers(source: &str) -> Vec<SourceChunk> {
    let mut chunks = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut current_start_line: usize = 1;
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1; // 1-based
        let trimmed = line.trim_start();
        let is_scope_boundary = if trimmed.is_empty() {
            false
        } else {
            let first_char = trimmed.chars().next().unwrap_or(' ');
            matches!(
                first_char,
                '＊' | '*' | '％' | '%' | '＆' | '&' | '＠' | '@'
            )
        };

        if is_scope_boundary && !current_lines.is_empty() {
            // Flush current chunk (append trailing \n so every line has eol)
            let mut text = current_lines.join("\n");
            text.push('\n');
            chunks.push(SourceChunk {
                text,
                start_line: current_start_line,
            });
            current_lines.clear();
            current_start_line = line_num;
        }

        if current_lines.is_empty() {
            current_start_line = line_num;
        }
        current_lines.push(line);
    }

    // Flush remaining (append trailing \n so every line has eol)
    if !current_lines.is_empty() {
        let mut text = current_lines.join("\n");
        text.push('\n');
        chunks.push(SourceChunk {
            text,
            start_line: current_start_line,
        });
    }

    chunks
}

/// 部分パースAPI — 3段階フォールバック戦略
///
/// Phase 1: `parse_str()`による全体パースを試行
/// Phase 2: スコープ境界分割 → 各チャンクを個別にパース
/// Phase 3: 行単位フォールバック → 各行を個別にパース
pub fn parse_str_partial(source: &str) -> PartialParseResult {
    // Phase 1: Full Parse
    if let Ok(pasta_file) = parse_str(source, "<partial>") {
        return PartialParseResult {
            items: pasta_file.items,
            errors: vec![],
        };
    }

    let mut partial_items = Vec::new();
    let mut partial_errors = Vec::new();

    // Phase 2: Scope Boundary Split
    let chunks = split_by_scope_markers(source);
    for chunk in chunks {
        // Try to infer rule from the first line of the chunk
        let first_line = chunk.text.lines().next().unwrap_or("");
        let rule = infer_rule_from_line(first_line);

        if let Some(rule) = rule {
            // Try parsing with a file-level wrapper for scope rules
            let parse_result = try_parse_chunk(&chunk.text, rule);
            match parse_result {
                Ok(items) => {
                    partial_items.extend(items);
                    continue;
                }
                Err(_) => {
                    // Phase 3: Line-by-Line Fallback for this chunk
                    for (line_idx, line) in chunk.text.lines().enumerate() {
                        let line_num = chunk.start_line + line_idx;
                        if line.trim().is_empty() {
                            continue;
                        }
                        let line_rule = infer_rule_from_line(line);
                        if let Some(lr) = line_rule {
                            match try_parse_chunk(line, lr) {
                                Ok(items) => partial_items.extend(items),
                                Err(e) => {
                                    partial_errors.push(PartialParseError {
                                        line: line_num,
                                        message: e.to_string(),
                                        span: None,
                                    });
                                }
                            }
                        } else {
                            partial_errors.push(PartialParseError {
                                line: line_num,
                                message: "Unable to infer parse rule for line".to_string(),
                                span: None,
                            });
                        }
                    }
                }
            }
        } else {
            // No rule inferred for this chunk, try line-by-line
            for (line_idx, line) in chunk.text.lines().enumerate() {
                let line_num = chunk.start_line + line_idx;
                if line.trim().is_empty() {
                    continue;
                }
                partial_errors.push(PartialParseError {
                    line: line_num,
                    message: "Unable to infer parse rule for line".to_string(),
                    span: None,
                });
            }
        }
    }

    PartialParseResult {
        items: partial_items,
        errors: partial_errors,
    }
}

/// チャンクをファイルとしてパースしてFileItemsを抽出
fn try_parse_chunk(text: &str, _rule: Rule) -> Result<Vec<FileItem>, String> {
    // Try wrapping in a full file parse, as individual rules may require
    // the file context (EOI etc.)
    match parse_str(text, "<partial>") {
        Ok(file) => Ok(file.items),
        Err(e) => Err(format!("{}", e)),
    }
}
