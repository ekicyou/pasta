//! 行事の表（calendar.csv）の検査のテスト

use std::fs;

use pasta_check::ghost_files::collect;
use pasta_check::validate::{FindingKind, Severity, validate_ghost};

/// 殻（collect）が表を読み、表の誤りがあっても飛び先と条件の検査は省かない
#[test]
fn bad_table_is_an_error_and_dictionary_checks_still_run() {
    let dir = tempfile::tempdir().unwrap();
    let master = dir.path();
    fs::write(
        master.join("pasta.toml"),
        "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n",
    )
    .unwrap();
    fs::create_dir(master.join("dic")).unwrap();
    fs::write(
        master.join("dic/a.pasta"),
        "＊会話\n　Alice：やあ\n　＞無いシーン\n＊条件＆＄＊初回\n　Alice：はじめまして\n",
    )
    .unwrap();
    fs::write(master.join("Calendar.CSV"), "0101,元日\n13xx,だめ\n").unwrap();

    let (input, files) = collect(master).unwrap();
    let report = validate_ghost(&input, &files);

    assert!(report.has_errors(), "{:?}", report.findings);
    assert_eq!(report.stats.dictionary_files, 1);
    assert!(report.stats.skip_reasons.is_empty(), "{:?}", report.stats);
    let codes = |kind| {
        report
            .findings
            .iter()
            .filter(|f| f.code == kind)
            .collect::<Vec<_>>()
    };
    let table = codes(FindingKind::CalendarInvalid);
    assert_eq!(table.len(), 1, "{:?}", report.findings);
    assert_eq!(table[0].path, "Calendar.CSV");
    assert_eq!(table[0].line, Some(2));
    assert_eq!(table[0].severity, Severity::Error);
    assert!(!codes(FindingKind::CallTargetMissing).is_empty());
    assert!(!codes(FindingKind::CondVarUnassigned).is_empty());
}

/// ghost/master に calendar 系のファイル（名前, 中身）と辞書 1 つを置いて、殻から検査する
fn run(files: &[(&str, &[u8])]) -> pasta_check::validate::Report {
    let dir = tempfile::tempdir().unwrap();
    let master = dir.path();
    fs::write(
        master.join("pasta.toml"),
        "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n",
    )
    .unwrap();
    fs::create_dir(master.join("dic")).unwrap();
    fs::write(master.join("dic/a.pasta"), "＊会話\n　Alice：やあ\n").unwrap();
    for (name, body) in files {
        fs::write(master.join(name), body).unwrap();
    }
    let (input, collected) = collect(master).unwrap();
    validate_ghost(&input, &collected)
}

fn table_findings(report: &pasta_check::validate::Report) -> Vec<(&str, Option<u32>, &str)> {
    report
        .findings
        .iter()
        .filter(|f| f.code == FindingKind::CalendarInvalid)
        .map(|f| (f.path.as_str(), f.line, f.message.as_str()))
        .collect()
}

/// 行の誤り 5 種は、種類ごとの文と行番号で出る
#[test]
fn each_line_error_has_its_message_and_line() {
    let cases = [
        ("0101,\"a\"", "引用符"),
        ("0101", "2 つの欄"),
        ("13xx,a", "MMDD"),
        ("0230,a", "暦にありません"),
        ("0101,", "名前が空"),
    ];
    for (line, expect) in cases {
        let report = run(&[("calendar.csv", format!("# c\n{line}\n").as_bytes())]);
        let found = table_findings(&report);
        assert_eq!(found.len(), 1, "{line}: {:?}", report.findings);
        assert_eq!(found[0].0, "calendar.csv", "{line}");
        assert_eq!(found[0].1, Some(2), "{line}");
        assert!(found[0].2.contains(expect), "{line}: {}", found[0].2);
        assert!(report.has_errors(), "{line}");
    }
}

#[test]
fn not_utf8_is_one_error_without_line() {
    let report = run(&[("calendar.csv", b"0101,\x8c\xb3\x93\xfa\n")]);
    let found = table_findings(&report);
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].1, None);
    assert!(found[0].2.contains("UTF-8"), "{}", found[0].2);
}

#[test]
fn folder_named_calendar_csv_is_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("calendar.csv")).unwrap();
    let (input, files) = collect(dir.path()).unwrap();
    let report = validate_ghost(&input, &files);
    let found = table_findings(&report);
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].1, None);
    assert!(found[0].2.contains("フォルダ"), "{}", found[0].2);
}

/// 大文字・小文字だけ違う 2 つ。区別するファイルシステム（unix）でだけ作れる
#[cfg(unix)]
#[test]
fn names_differing_only_in_case_are_both_reported() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("calendar.csv"), "0101,a\n").unwrap();
    fs::write(dir.path().join("Calendar.CSV"), "0101,b\n").unwrap();
    if fs::read_dir(dir.path()).unwrap().count() < 2 {
        return; // 大文字・小文字を畳むファイルシステム（macOS）
    }
    let (input, files) = collect(dir.path()).unwrap();
    let report = validate_ghost(&input, &files);
    let found = table_findings(&report);
    let paths: Vec<_> = found.iter().map(|f| f.0).collect();
    assert_eq!(paths, ["Calendar.CSV", "calendar.csv"], "{found:?}");
    assert!(
        found
            .iter()
            .all(|f| f.1.is_none() && f.2.contains("重なって"))
    );
}

#[test]
fn no_table_gives_no_calendar_finding() {
    let report = run(&[]);
    assert!(table_findings(&report).is_empty(), "{:?}", report.findings);
    assert!(!report.has_errors(), "{:?}", report.findings);
}

#[test]
fn upper_case_file_name_is_found() {
    let report = run(&[("Calendar.CSV", "0101,元日\n".as_bytes())]);
    assert!(table_findings(&report).is_empty(), "{:?}", report.findings);
    let report = run(&[("Calendar.CSV", b"0101\n")]);
    assert_eq!(table_findings(&report)[0].0, "Calendar.CSV");
}
