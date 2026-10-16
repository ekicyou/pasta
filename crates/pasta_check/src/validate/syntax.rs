//! 各辞書ファイルを pasta.dll と同じ関数で読み、最初の構文エラーと読めないファイルを報告する段（3.1〜3.4）。

use pasta_dsl::{ParseError, PastaFile};

use super::{FileText, Finding, FindingKind, msg};

/// 読めたファイルの構文木を、パスと組にして次の段に渡す。どのファイルで失敗しても残りを続ける（3.1）
pub(crate) fn check(files: &[(String, FileText)]) -> (Vec<(String, PastaFile)>, Vec<Finding>) {
    let mut parsed = Vec::new();
    let mut findings = Vec::new();
    for (path, text) in files {
        match text {
            FileText::Text { text } => match pasta_dsl::parse_str(text, path) {
                Ok(file) => parsed.push((path.clone(), file)),
                Err(e) => findings.push(syntax_finding(path, e)),
            },
            FileText::NotUtf8 => findings.push(Finding::new(
                FindingKind::FileNotUtf8,
                path,
                None,
                None,
                msg::file_not_utf8(),
            )),
            FileText::Unreadable { reason } => findings.push(Finding::new(
                FindingKind::FileUnreadable,
                path,
                None,
                None,
                msg::file_unreadable(reason),
            )),
        }
    }
    (parsed, findings)
}

/// 位置は行の頭に出すので、parse_str の文言から pest の短い説明（末尾の `  = expected id` の行）だけを取る（7.1 の 1 行）
fn syntax_finding(path: &str, e: ParseError) -> Finding {
    match e {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
        } => {
            let prefix = format!("Parse error in {file} at {line}:{column}: ");
            let desc = message.strip_prefix(&prefix).unwrap_or(&message);
            // pest の文言は ` --> 行:列`・該当行・`^---` の枠の後ろに `  = 説明` が来る
            let desc = desc.rsplit_once("\n  = ").map_or(desc, |(_, short)| short);
            Finding::new(
                FindingKind::Syntax,
                path,
                Some(line as u32),
                Some(column as u32),
                msg::syntax(desc),
            )
        }
        // AST の組み立ての失敗などは位置を持たない
        other => Finding::new(
            FindingKind::Syntax,
            path,
            None,
            None,
            msg::syntax(&other.to_string()),
        ),
    }
}
