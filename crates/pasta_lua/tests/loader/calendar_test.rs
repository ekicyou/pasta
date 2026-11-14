//! date-var-calendar task 3.4: 読み込みの段 1.6（行事の表 `calendar.csv`）の結合テスト。

use crate::common;

use common::create_temp_with_pasta;
use pasta_lua::loader::{LoaderError, PastaLoader};
use std::path::Path;

const DIC: &str = "＊会話\n　さくら：「こんにちは」\n";

fn load_err(dir: &Path) -> LoaderError {
    match PastaLoader::load(dir) {
        Ok(_) => panic!("load must fail"),
        Err(e) => e,
    }
}

/// `@pasta_calendar.event(y, m, d)` の戻り値（名前か None）。
fn event(dir: &Path, y: i32, m: i32, d: i32) -> Option<String> {
    let runtime = PastaLoader::load(dir).expect("load must succeed");
    runtime
        .lua()
        .load(r#"return require("@pasta_calendar").event(...)"#)
        .call::<Option<String>>((y, m, d))
        .expect("event must not error")
}

#[test]
fn no_table_loads_with_empty_event_table() {
    let temp = create_temp_with_pasta(DIC);
    assert_eq!(event(temp.path(), 2027, 1, 1), None);
}

#[test]
fn upper_case_file_name_is_read() {
    let temp = create_temp_with_pasta(DIC);
    std::fs::write(temp.path().join("Calendar.CSV"), "0101,元日の行事\n").unwrap();
    assert_eq!(
        event(temp.path(), 2027, 1, 1).as_deref(),
        Some("元日の行事")
    );
}

#[test]
fn line_errors_name_file_and_line() {
    // Req 5.1〜5.5: 引用符・欄の数・日付の形・存在しない日・空の名前
    let cases = [
        ("0101,\"a\"", "quote"),
        ("0101", "2 fields"),
        ("13xx,a", "MMDD"),
        ("0230,a", "does not exist"),
        ("0101,", "empty"),
    ];
    for (line, expect) in cases {
        let temp = create_temp_with_pasta(DIC);
        let path = temp.path().join("calendar.csv");
        std::fs::write(&path, format!("# c\n{line}\n")).unwrap();
        let err = load_err(temp.path());
        let msg = err.to_string();
        assert!(matches!(err, LoaderError::Calendar { .. }), "{line}: {msg}");
        assert!(msg.contains("calendar.csv"), "{line}: {msg}");
        assert!(msg.contains("line 2:"), "{line}: {msg}");
        assert!(msg.contains(expect), "{line}: {msg}");
    }
}

#[test]
fn shift_jis_bytes_are_encoding_error() {
    let temp = create_temp_with_pasta(DIC);
    // "0101,元日" in Shift_JIS
    std::fs::write(temp.path().join("calendar.csv"), b"0101,\x8c\xb3\x93\xfa\n").unwrap();
    let err = load_err(temp.path());
    assert!(matches!(err, LoaderError::CalendarNotUtf8(_)), "{err}");
    assert!(err.to_string().contains("calendar.csv"));
}

#[test]
fn folder_named_calendar_csv_is_io_error() {
    let temp = create_temp_with_pasta(DIC);
    std::fs::create_dir(temp.path().join("calendar.csv")).unwrap();
    let err = load_err(temp.path());
    assert!(matches!(err, LoaderError::Io(..)), "{err}");
    assert!(err.to_string().contains("calendar.csv"));
}

#[test]
fn rewriting_table_and_reloading_changes_event() {
    let temp = create_temp_with_pasta(DIC);
    let path = temp.path().join("calendar.csv");
    std::fs::write(&path, "0101,甲\n").unwrap();
    assert_eq!(event(temp.path(), 2027, 1, 1).as_deref(), Some("甲"));
    std::fs::write(&path, "0101,乙\n").unwrap();
    assert_eq!(event(temp.path(), 2027, 1, 1).as_deref(), Some("乙"));
}

#[cfg(unix)]
#[test]
fn symlink_calendar_csv_is_not_counted() {
    let temp = create_temp_with_pasta(DIC);
    let real = temp.path().join("real.csv");
    std::fs::write(&real, "0101,x\n").unwrap();
    std::os::unix::fs::symlink(&real, temp.path().join("calendar.csv")).unwrap();
    assert_eq!(event(temp.path(), 2027, 1, 1), None);
}

#[cfg(unix)]
#[test]
fn names_differing_only_in_case_are_duplicate() {
    let temp = create_temp_with_pasta(DIC);
    std::fs::write(temp.path().join("calendar.csv"), "0101,a\n").unwrap();
    std::fs::write(temp.path().join("Calendar.CSV"), "0101,b\n").unwrap();
    let both = ["calendar.csv", "Calendar.CSV"].iter().all(|n| {
        std::fs::read_dir(temp.path())
            .unwrap()
            .any(|e| e.unwrap().file_name() == *n)
    });
    if !both {
        return; // case-insensitive file system (macOS): the second write hit the first file
    }
    let err = load_err(temp.path());
    assert!(
        matches!(err, LoaderError::CalendarDuplicate { .. }),
        "{err}"
    );
    let msg = err.to_string();
    assert!(
        msg.contains("calendar.csv") && msg.contains("Calendar.CSV"),
        "{msg}"
    );
}
