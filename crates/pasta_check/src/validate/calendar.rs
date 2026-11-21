//! 行事の表（ghost/master 直下の calendar.csv）の選択と検査。pasta.dll の段 1.6 と同じ規則。

use std::collections::BTreeMap;

use pasta_core::{CalendarProblem, CalendarTable, is_calendar_file_name};

use super::{FileText, Finding, FindingKind, GhostInput, file_text, msg};

/// 殻に読ませる表のパス（直下のファイルで名前が当たるもの・昇順）
pub(super) fn paths(input: &GhostInput) -> Vec<String> {
    let mut found: Vec<String> = input
        .entries
        .iter()
        .filter(|p| !p.contains('/') && is_calendar_file_name(p))
        .cloned()
        .collect();
    found.sort();
    found
}

/// ローダの read_dir と同じく、直下のファイルとフォルダを区別せずに数える
pub(super) fn check(input: &GhostInput, files: &BTreeMap<String, FileText>) -> Vec<Finding> {
    let file_paths = paths(input);
    let mut all: Vec<&String> = file_paths
        .iter()
        .chain(
            input
                .dirs
                .iter()
                .filter(|d| !d.contains('/') && is_calendar_file_name(d)),
        )
        .collect();
    all.sort();
    let error = |path: &str, line: Option<u32>, message: String| {
        Finding::new(FindingKind::CalendarInvalid, path, line, None, message)
    };
    match all.as_slice() {
        [] => vec![],
        [path] if !file_paths.contains(path) => {
            vec![error(path, None, msg::calendar_unreadable("フォルダです"))]
        }
        [path] => match file_text(files, path) {
            FileText::NotUtf8 => vec![error(path, None, msg::calendar_not_utf8())],
            FileText::Unreadable { reason } => {
                vec![error(path, None, msg::calendar_unreadable(&reason))]
            }
            FileText::Text { text } => CalendarTable::parse(&text)
                .err()
                .unwrap_or_default()
                .into_iter()
                .map(|e| error(path, Some(e.line as u32), problem_message(&e.problem)))
                .collect(),
        },
        _ => all
            .iter()
            .map(|path| {
                let others: Vec<&str> = all
                    .iter()
                    .filter(|o| o != &path)
                    .map(|o| o.as_str())
                    .collect();
                error(path, None, msg::calendar_duplicate(&others))
            })
            .collect(),
    }
}

fn problem_message(problem: &CalendarProblem) -> String {
    match problem {
        CalendarProblem::Quote => msg::calendar_quote(),
        CalendarProblem::FieldCount { count } => msg::calendar_field_count(*count),
        CalendarProblem::BadDateForm { written } => msg::calendar_bad_date_form(written),
        CalendarProblem::NoSuchDate { written } => msg::calendar_no_such_date(written),
        CalendarProblem::EmptyName => msg::calendar_empty_name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::TomlSource;

    fn input(entries: &[&str], dirs: &[&str]) -> GhostInput {
        GhostInput {
            pasta_toml: TomlSource::Missing,
            entries: entries.iter().map(|s| s.to_string()).collect(),
            dirs: dirs.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn lines(input: &GhostInput, files: &BTreeMap<String, FileText>) -> Vec<(String, Option<u32>)> {
        check(input, files)
            .into_iter()
            .map(|f| (f.path, f.line))
            .collect()
    }

    #[test]
    fn counts_files_and_folders_directly_under_master() {
        let none = BTreeMap::new();
        assert!(
            lines(
                &input(&["dic/calendar.csv", "calendar.csv.bak"], &[]),
                &none
            )
            .is_empty()
        );
        assert_eq!(
            lines(&input(&[], &["calendar.csv"]), &none),
            [("calendar.csv".into(), None)]
        );
        assert_eq!(
            lines(&input(&["Calendar.CSV"], &["calendar.csv"]), &none),
            [("Calendar.CSV".into(), None), ("calendar.csv".into(), None)]
        );
        let files = BTreeMap::from([(
            "calendar.csv".to_string(),
            FileText::Text {
                text: "0101,a\n\"x\",b\n0230,c\n".into(),
            },
        )]);
        assert_eq!(
            lines(&input(&["calendar.csv"], &[]), &files),
            [
                ("calendar.csv".into(), Some(2)),
                ("calendar.csv".into(), Some(3))
            ]
        );
    }
}
