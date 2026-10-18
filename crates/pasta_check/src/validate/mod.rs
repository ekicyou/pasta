//! 検査の核。ファイルの一覧と中身を受け取り、報告を返す（ファイルシステムに触れない）。
//!
//! 状態を持たず、`std::fs`・`std::time`・乱数源を使わない。同じ入力には同じ報告を返す。
//! パスはすべて ghost/master からの相対パスで、区切りは `/`。

mod builtins;
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

use pasta_dsl::parser::Span;

pub use input::{FileText, GhostInput, TomlSource};
pub use report::{Finding, FindingKind, Report, Severity, SkipReason, Stats, msg};

/// 一致テスト用: pasta.dll が見るパスからモジュール名を作る（`CacheManager::source_to_module_name` と同じ規則）
#[doc(hidden)]
pub use module_name::module_name_of;

/// 一致テスト用: 組み込みの飛び先の名前（GLOBAL・act・SHIORI 用の act のメソッド）
#[doc(hidden)]
pub use builtins::{ACT_METHODS, GLOBAL_NAMES, SHIORI_ACT_METHODS};

/// 辞書ファイルとして読むべきパス（.pasta のパターンに一致したもの）を、決まった順で返す。
pub fn select_dictionary_files(input: &GhostInput) -> Vec<String> {
    let (found, _) = discover(input);
    found.pasta.into_iter().map(|f| f.path).collect()
}

/// 一致テスト用: pasta.dll が見るパスの（.pasta の一覧, .lua の一覧）。実物の相対パスの昇順
#[doc(hidden)]
pub fn pasta_dll_paths(input: &GhostInput) -> (Vec<String>, Vec<String>) {
    let (found, _) = discover(input);
    let dll = |files: Vec<discovery::DictFile>| files.into_iter().map(|f| f.dll_path).collect();
    (dll(found.pasta), dll(found.lua))
}

/// 一致テスト用: 辞書のパターンに一致したフォルダの、pasta.dll が見る綴りの相対パス（昇順・重複なし。
/// ghost/master そのものは ""）
#[doc(hidden)]
pub fn pasta_dll_folders(input: &GhostInput) -> Vec<String> {
    let (found, _) = discover(input);
    let mut folders: Vec<String> = found.folders.into_iter().map(|f| f.dll_path).collect();
    folders.sort();
    folders.dedup();
    folders
}

fn discover(input: &GhostInput) -> (discovery::Discovered, Vec<Finding>) {
    let (config, _) = config::read_config(&input.pasta_toml);
    discovery::discover(input, &config)
}

/// ゴースト全体を検査する。`files` には select_dictionary_files が返した全パスの中身を入れる。
///
/// 順番: 設定 → 集め方 → モジュール名・構文 → トランスパイル・シーン → 飛び先。
/// すべての段の報告がそろった後に省く理由を 1 回だけ決め、構文エラー・読めない辞書ファイル・
/// pasta.toml の誤りが 1 つでもあれば飛び先の検査を丸ごと省く（1.1〜1.3）。
pub fn validate_ghost(input: &GhostInput, files: &BTreeMap<String, FileText>) -> Report {
    let (config, mut findings) = config::read_config(&input.pasta_toml);

    let (discovered, found) = discovery::discover(input, &config);
    findings.extend(found);
    findings.extend(module_name::check(&discovered));
    let dictionary_files: Vec<String> = discovered.pasta.into_iter().map(|f| f.path).collect();

    let texts: Vec<(String, FileText)> = dictionary_files
        .iter()
        .map(|path| (path.clone(), file_text(files, path)))
        .collect();
    let (parsed, found) = syntax::check(&texts);
    findings.extend(found);

    findings.extend(transpile_rules::check(&parsed, &texts));
    let (scenes, found) = scenes::check(&parsed, &config.aliases);
    findings.extend(found);

    let skip_reasons = report::skip_reasons(&findings);
    let mut unchecked_dynamic_calls = 0;
    if skip_reasons.is_empty() {
        let (found, dynamic) = references::check(&scenes, &config.aliases, &parsed, &texts);
        findings.extend(found);
        unchecked_dynamic_calls = dynamic;
    }

    Report::new(
        findings,
        Stats {
            dictionary_files: dictionary_files.len(),
            unchecked_dynamic_calls,
            skip_reasons,
            ..Stats::default()
        },
    )
}

/// 字下げから始まる行の span（選択肢・継続行）の列を、字下げの後ろの中身の列にする（7.1）。
/// 字下げ（文法の pad）の文字はどれも Unicode の空白
fn col_after_indent(texts: &[(String, FileText)], path: &str, span: Span) -> usize {
    let line = texts.iter().find_map(|(p, t)| match t {
        FileText::Text { text } if p == path => text.lines().nth(span.start_line - 1),
        _ => None,
    });
    let pad = line.map_or(0, |l| l.chars().take_while(|c| c.is_whitespace()).count());
    span.start_col + pad
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
