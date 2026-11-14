//! 行事の表（calendar.csv）の検査のテスト（Req 6.1〜6.5）

use std::fs;

use pasta_check::ghost_files::collect;
use pasta_check::validate::{FindingKind, Severity, validate_ghost};

/// 殻（collect）が表を読み、表の誤りがあっても飛び先と条件の検査は省かない（Req 6.1・6.2・6.5）
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
