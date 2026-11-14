//! `calendar.csv`（行事の表）の書式の検査と表への読み取り。

#![allow(dead_code)] // 公開 API は task 2.3 で出る

use super::civil::days_in_month;
use crate::error::{CalendarError, CalendarProblem};

/// 日付の欄。`md` は `月*100+日`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DateSpec {
    MonthDay(u32),
    Range { start: u32, end: u32 },
    Date { year: i32, md: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CalendarEntry {
    pub(crate) date: DateSpec,
    pub(crate) name: String,
}

/// 行事の表（書いた順）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CalendarTable {
    entries: Vec<CalendarEntry>,
}

impl CalendarTable {
    /// 本文（BOM 付きでもよい）を検査する。誤りがあれば全部の行の誤りを行の順に返す。
    pub fn parse(text: &str) -> Result<CalendarTable, Vec<CalendarError>> {
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line = trim(raw);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match parse_line(line) {
                Ok(entry) => entries.push(entry),
                Err(problem) => errors.push(CalendarError {
                    line: i + 1,
                    problem,
                }),
            }
        }
        if errors.is_empty() {
            Ok(CalendarTable { entries })
        } else {
            Err(errors)
        }
    }
}

/// 半角の空白・タブ・全角の空白を前後から外す。
fn trim(s: &str) -> &str {
    s.trim_matches([' ', '\t', '\u{3000}'])
}

/// 検査の順: 引用符 → 欄の数 → 日付の形 → 暦 → 空の名前。
fn parse_line(line: &str) -> Result<CalendarEntry, CalendarProblem> {
    if line.contains('"') {
        return Err(CalendarProblem::Quote);
    }
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 2 {
        return Err(CalendarProblem::FieldCount {
            count: fields.len(),
        });
    }
    let (written, name) = (trim(fields[0]), trim(fields[1]));
    let date = parse_date(written)?;
    if name.is_empty() {
        return Err(CalendarProblem::EmptyName);
    }
    Ok(CalendarEntry {
        date,
        name: name.to_string(),
    })
}

fn parse_date(written: &str) -> Result<DateSpec, CalendarProblem> {
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    let no_such = || CalendarProblem::NoSuchDate {
        written: written.to_string(),
    };
    // MMDD は 2 月 29 日を受け付けるため、うるう年の月の日数で見る
    let month_day = |s: &str| {
        let md: u32 = s.parse().unwrap();
        let (m, d) = (md / 100, md % 100);
        (d >= 1 && d <= days_in_month(2000, m))
            .then_some(md)
            .ok_or_else(no_such)
    };
    if digits(written, 4) {
        return Ok(DateSpec::MonthDay(month_day(written)?));
    }
    if let Some((start, end)) = written.split_once('-')
        && digits(start, 4)
        && digits(end, 4)
    {
        return Ok(DateSpec::Range {
            start: month_day(start)?,
            end: month_day(end)?,
        });
    }
    if digits(written, 8) {
        let year: i32 = written[..4].parse().unwrap();
        let md: u32 = written[4..].parse().unwrap();
        let (m, d) = (md / 100, md % 100);
        if year < 1 || d < 1 || d > days_in_month(year, m) {
            return Err(no_such());
        }
        return Ok(DateSpec::Date { year, md });
    }
    Err(CalendarProblem::BadDateForm {
        written: written.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(text: &str) -> Vec<CalendarError> {
        CalendarTable::parse(text).expect_err("should fail")
    }

    fn one(text: &str) -> CalendarError {
        let e = err(text);
        assert_eq!(e.len(), 1, "{e:?}");
        e.into_iter().next().unwrap()
    }

    fn entry(date: DateSpec, name: &str) -> CalendarEntry {
        CalendarEntry {
            date,
            name: name.to_string(),
        }
    }

    #[test]
    fn accepts_three_forms_in_written_order() {
        let t = CalendarTable::parse("0315,a\n1231-0103,b\n20270111,c\n0229,d\n").unwrap();
        assert_eq!(
            t.entries,
            vec![
                entry(DateSpec::MonthDay(315), "a"),
                entry(
                    DateSpec::Range {
                        start: 1231,
                        end: 103
                    },
                    "b"
                ),
                entry(
                    DateSpec::Date {
                        year: 2027,
                        md: 111
                    },
                    "c"
                ),
                entry(DateSpec::MonthDay(229), "d"),
            ]
        );
    }

    #[test]
    fn accepts_bom_crlf_comments_blank_full_width_space_and_nashi() {
        let text = "\u{FEFF}# 注釈\r\n\r\n \t\u{3000}\r\n  # 字下げの注釈\r\n\u{3000}0101\u{3000},\t元日 \r\n0505,なし\r\n0505,なし\n";
        let t = CalendarTable::parse(text).unwrap();
        assert_eq!(
            t.entries,
            vec![
                entry(DateSpec::MonthDay(101), "元日"),
                entry(DateSpec::MonthDay(505), "なし"),
                entry(DateSpec::MonthDay(505), "なし"),
            ]
        );
    }

    #[test]
    fn empty_text_is_empty_table() {
        assert_eq!(CalendarTable::parse("").unwrap(), CalendarTable::default());
        assert_eq!(
            CalendarTable::parse("\u{FEFF}").unwrap(),
            CalendarTable::default()
        );
    }

    fn bad_form(w: &str) -> CalendarProblem {
        CalendarProblem::BadDateForm {
            written: w.to_string(),
        }
    }

    fn no_such(w: &str) -> CalendarProblem {
        CalendarProblem::NoSuchDate {
            written: w.to_string(),
        }
    }

    #[test]
    fn each_error_has_line_and_cause() {
        let cases: Vec<(&str, CalendarProblem)> = vec![
            ("日付,名前", bad_form("日付")),
            ("0101-,a", bad_form("0101-")),
            ("20270101-20270103,a", bad_form("20270101-20270103")),
            ("１２３４,a", bad_form("１２３４")),
            ("1/1,a", bad_form("1/1")),
            ("123,a", bad_form("123")),
            ("0230,a", no_such("0230")),
            ("1301,a", no_such("1301")),
            ("0100,a", no_such("0100")),
            ("0101-0230,a", no_such("0101-0230")),
            ("20270229,a", no_such("20270229")),
            ("00000101,a", no_such("00000101")),
            ("0101", CalendarProblem::FieldCount { count: 1 }),
            ("0101,a,b", CalendarProblem::FieldCount { count: 3 }),
            ("0101,\"a\"", CalendarProblem::Quote),
            ("0101,", CalendarProblem::EmptyName),
            ("0101,\u{3000} ", CalendarProblem::EmptyName),
        ];
        for (line, problem) in cases {
            // 2 行目に置き、行番号が物理の行（注釈を含めて 1 始まり）であることも見る
            let e = one(&format!("# c\r\n{line}\r\n0101,ok\r\n"));
            assert_eq!(e, CalendarError { line: 2, problem }, "input {line:?}");
        }
    }

    #[test]
    fn leap_day_forms_are_accepted() {
        assert!(CalendarTable::parse("0229,a\n0201-0229,b\n20280229,c\n").is_ok());
    }

    #[test]
    fn one_error_per_line_in_check_order() {
        // 引用符 → 欄の数 → 日付の形 → 暦 → 空の名前 の順で最初の 1 つ
        assert_eq!(one("x,\"y\",z").problem, CalendarProblem::Quote);
        assert_eq!(
            one("x,y,").problem,
            CalendarProblem::FieldCount { count: 3 }
        );
        assert_eq!(one("x,").problem, bad_form("x"));
        assert_eq!(one("0230,").problem, no_such("0230"));
    }

    #[test]
    fn all_errors_returned_in_line_order() {
        let e = err("0101,ok\n0230,a\n\n# c\n0101\n0101,\"q\"\n0505,ok\n0101,\n");
        assert_eq!(
            e,
            vec![
                CalendarError {
                    line: 2,
                    problem: no_such("0230")
                },
                CalendarError {
                    line: 5,
                    problem: CalendarProblem::FieldCount { count: 1 }
                },
                CalendarError {
                    line: 6,
                    problem: CalendarProblem::Quote
                },
                CalendarError {
                    line: 8,
                    problem: CalendarProblem::EmptyName
                },
            ]
        );
    }

    #[test]
    fn error_display_is_english() {
        assert_eq!(
            CalendarError {
                line: 3,
                problem: CalendarProblem::FieldCount { count: 1 }
            }
            .to_string(),
            "line 3: a line must have 2 fields \"date,name\" (got 1)"
        );
    }
}
