//! 検査の核。ファイルの一覧と中身を受け取り、報告を返す（ファイルシステムに触れない）。
//!
//! 状態を持たず、`std::fs`・`std::time`・乱数源を使わない。同じ入力には同じ報告を返す。
//! パスはすべて ghost/master からの相対パスで、区切りは `/`。

mod config;
mod discovery;
mod input;
mod module_name;
mod references;
mod report;
mod scenes;
mod syntax;
mod transpile_rules;

use std::collections::BTreeMap;

pub use input::{FileText, GhostInput, TomlSource};
pub use report::{Finding, FindingKind, Report, Severity, Stats, msg};

/// 辞書ファイルとして読むべきパス（.pasta のパターンに一致したもの）を、決まった順で返す。
pub fn select_dictionary_files(input: &GhostInput) -> Vec<String> {
    let (config, _) = config::read_config(&input.pasta_toml);
    discovery::discover(input, &config).0
}

/// ゴースト全体を検査する。`files` には select_dictionary_files が返した全パスの中身を入れる。
///
/// 順番: 設定 → 集め方 → モジュール名・構文 → トランスパイル・シーン → 飛び先。
/// 構文エラーが 1 つでもあれば飛び先の検査を丸ごと省く（6.8）。
pub fn validate_ghost(input: &GhostInput, files: &BTreeMap<String, FileText>) -> Report {
    let (config, mut findings) = config::read_config(&input.pasta_toml);

    let (dictionary_files, found) = discovery::discover(input, &config);
    findings.extend(found);
    findings.extend(module_name::check(&dictionary_files));

    let texts: Vec<(String, FileText)> = dictionary_files
        .iter()
        .map(|path| (path.clone(), file_text(files, path)))
        .collect();
    let (parsed, found) = syntax::check(&texts);
    findings.extend(found);

    findings.extend(transpile_rules::check(&parsed));
    let (table, found) = scenes::check(&parsed, &config.aliases);
    findings.extend(found);

    let references_skipped = findings.iter().any(|f| f.code == FindingKind::Syntax);
    let mut unchecked_dynamic_calls = 0;
    if !references_skipped {
        let (found, dynamic) = references::check(&table, &parsed);
        findings.extend(found);
        unchecked_dynamic_calls = dynamic;
    }

    Report::new(
        findings,
        Stats {
            dictionary_files: dictionary_files.len(),
            unchecked_dynamic_calls,
            references_skipped,
            ..Stats::default()
        },
    )
}

/// 殻が渡し忘れたファイルは黙って通さず、読めないファイルとして扱う
fn file_text(files: &BTreeMap<String, FileText>, path: &str) -> FileText {
    files.get(path).cloned().unwrap_or(FileText::Unreadable {
        reason: "読み込まれていません".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_not_passed_is_unreadable_not_loaded() {
        let mut files = BTreeMap::new();
        files.insert("dic/a.pasta".to_string(), FileText::NotUtf8);
        assert!(matches!(
            file_text(&files, "dic/a.pasta"),
            FileText::NotUtf8
        ));
        assert!(matches!(
            file_text(&files, "dic/b.pasta"),
            FileText::Unreadable { reason } if reason == "読み込まれていません"
        ));
    }
}
