//! 祝日の規則を内閣府の CSV（今の法律の 2022 年以降）と全部の日で突き合わせる（Req 2.8, 8.1, 8.2, 8.8）。
//! データは出典付きでコミット済み。ネットワークは使わない。差し替え後も年の範囲は CSV から決める。

use pasta_core::holiday_name;
use std::collections::HashMap;

const CSV: &str = include_str!("fixtures/syukujitsu/syukujitsu.csv");
const FIRST_YEAR: i32 = 2022;

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[test]
fn holiday_rules_match_cao_csv_for_every_day() {
    // 見出しを飛ばし `Y/M/D,名前` を読む。lines() は LF・CRLF のどちらも切る。
    let mut expected: HashMap<(i32, u32, u32), &str> = HashMap::new();
    for line in CSV.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let (date, name) = line.split_once(',').expect("CSV 行は `日付,名前`");
        let mut p = date
            .split('/')
            .map(|s| s.trim().parse::<i64>().expect("日付は数"));
        let (y, m, d) = (p.next().unwrap(), p.next().unwrap(), p.next().unwrap());
        expected.insert((y as i32, m as u32, d as u32), name.trim());
    }
    let last_year = expected.keys().map(|k| k.0).max().expect("CSV が空");
    let rows = expected.keys().filter(|k| k.0 >= FIRST_YEAR).count();
    // 読み取りの不具合で何も比べずに通ることを防ぐ（毎年 16 日前後）。
    assert!(
        rows >= 50 && last_year >= FIRST_YEAR,
        "CSV の読み取りが不自然: {rows} 行, 最終年 {last_year}"
    );

    let mut mismatches = Vec::new();
    for y in FIRST_YEAR..=last_year {
        for m in 1..=12 {
            for d in 1..=days_in_month(y, m) {
                let actual = holiday_name(y, m, d).map(|n| match n {
                    "振替休日" | "国民の休日" => "休日",
                    n => n,
                });
                let want = expected.get(&(y, m, d)).copied();
                if actual != want {
                    mismatches.push(format!("{y}-{m:02}-{d:02}: rule={actual:?} csv={want:?}"));
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} 件の食い違い:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
