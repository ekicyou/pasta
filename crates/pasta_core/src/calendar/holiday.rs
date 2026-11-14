//! 祝日の規則（今の「国民の祝日に関する法律」だけ。2019〜2021 年の特例は持たない）。

use super::civil::{civil_from_days, days_from_civil, days_in_month, weekday};

/// 年月日（暦にある日）の祝日の名前。祝日でなければ None。
pub fn holiday_name(year: i32, month: u32, day: u32) -> Option<&'static str> {
    if day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let z = days_from_civil(year, month, day);
    if let Some(name) = base(z) {
        return Some(name);
    }
    // 振替休日: 前の日から切れ目なく続く基本の祝日の中に日曜日がある（法律の条の順で国民の休日より先）
    let mut p = z - 1;
    while base(p).is_some() {
        if weekday(p) == 0 {
            return Some("振替休日");
        }
        p -= 1;
    }
    if base(z - 1).is_some() && base(z + 1).is_some() {
        return Some("国民の休日");
    }
    None
}

/// 基本の祝日（固定の日・ハッピーマンデー・春分の日／秋分の日）。引数は通日。
fn base(z: i64) -> Option<&'static str> {
    let (y, m, d) = civil_from_days(z);
    let fixed = match (m, d) {
        (1, 1) => Some("元日"),
        (2, 11) => Some("建国記念の日"),
        (2, 23) => Some("天皇誕生日"),
        (4, 29) => Some("昭和の日"),
        (5, 3) => Some("憲法記念日"),
        (5, 4) => Some("みどりの日"),
        (5, 5) => Some("こどもの日"),
        (8, 11) => Some("山の日"),
        (11, 3) => Some("文化の日"),
        (11, 23) => Some("勤労感謝の日"),
        _ => None,
    };
    if fixed.is_some() {
        return fixed;
    }
    let monday =
        |n: u32, name| (weekday(z) == 1 && (7 * n - 6..=7 * n).contains(&d)).then_some(name);
    match m {
        1 => monday(2, "成人の日"),
        7 => monday(3, "海の日"),
        9 if d == equinox_day(y, 23.2488, 9) => Some("秋分の日"),
        9 => monday(3, "敬老の日"),
        10 => monday(2, "スポーツの日"),
        3 if d == equinox_day(y, 20.8431, 3) => Some("春分の日"),
        _ => None,
    }
}

/// 春分・秋分の式を年の範囲で区切らずに当てはめ、月の日の範囲に収める（どの年でも 1 日ずつ出す）。
fn equinox_day(y: i32, c: f64, month: u32) -> u32 {
    let t = i64::from(y) - 1980;
    let day = (c + 0.242194 * t as f64 - t.div_euclid(4) as f64).floor() as i64;
    day.clamp(1, i64::from(days_in_month(y, month))) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn year_holidays(y: i32) -> Vec<(u32, u32, &'static str)> {
        (1..=12)
            .flat_map(|m| (1..=days_in_month(y, m)).map(move |d| (m, d)))
            .filter_map(|(m, d)| holiday_name(y, m, d).map(|n| (m, d, n)))
            .collect()
    }

    #[test]
    fn all_holidays_of_2026() {
        // 内閣府の CSV の 2026 年（`休日` は振替休日・国民の休日に読み分け）
        let expected = vec![
            (1, 1, "元日"),
            (1, 12, "成人の日"),
            (2, 11, "建国記念の日"),
            (2, 23, "天皇誕生日"),
            (3, 20, "春分の日"),
            (4, 29, "昭和の日"),
            (5, 3, "憲法記念日"),
            (5, 4, "みどりの日"),
            (5, 5, "こどもの日"),
            (5, 6, "振替休日"),
            (7, 20, "海の日"),
            (8, 11, "山の日"),
            (9, 21, "敬老の日"),
            (9, 22, "国民の休日"),
            (9, 23, "秋分の日"),
            (10, 12, "スポーツの日"),
            (11, 3, "文化の日"),
            (11, 23, "勤労感謝の日"),
        ];
        assert_eq!(year_holidays(2026), expected);
    }

    #[test]
    fn named_cases() {
        assert_eq!(holiday_name(2026, 5, 6), Some("振替休日")); // 5/3 日曜 → 5/4・5/5 を越えて
        assert_eq!(holiday_name(2025, 2, 24), Some("振替休日")); // 2/23 日曜
        assert_eq!(holiday_name(2026, 9, 22), Some("国民の休日"));
        assert_eq!(holiday_name(2024, 1, 8), Some("成人の日")); // 第 2 月曜が 8 日
        assert_eq!(holiday_name(2024, 1, 15), None);
        assert_eq!(holiday_name(2030, 1, 14), Some("成人の日")); // 第 2 月曜が 14 日
        assert_eq!(holiday_name(2030, 1, 7), None);
    }

    #[test]
    fn equinoxes_2022_to_2028() {
        let spring = [21, 21, 20, 20, 20, 21, 20];
        let autumn = [23, 23, 22, 23, 23, 23, 22];
        for (i, y) in (2022..=2028).enumerate() {
            assert_eq!(holiday_name(y, 3, spring[i]), Some("春分の日"), "{y}");
            assert_eq!(holiday_name(y, 9, autumn[i]), Some("秋分の日"), "{y}");
        }
    }

    #[test]
    fn no_special_cases_before_2022() {
        assert_eq!(holiday_name(2019, 12, 23), None);
        assert_eq!(holiday_name(2019, 10, 14), Some("スポーツの日"));
        assert_eq!(holiday_name(2019, 10, 22), None); // 即位礼正殿の儀
        assert_eq!(holiday_name(2021, 7, 22), None); // 2021 年に動いた海の日
        assert_eq!(holiday_name(2021, 7, 19), Some("海の日"));
    }

    #[test]
    fn not_a_calendar_date() {
        assert_eq!(holiday_name(2026, 2, 30), None);
        assert_eq!(holiday_name(2026, 13, 1), None);
        assert_eq!(holiday_name(2026, 1, 0), None);
    }

    #[test]
    fn equinox_once_in_any_year() {
        for y in [1, 1969, 2100, 9999] {
            let h = year_holidays(y);
            let count = |m, name| h.iter().filter(|&&(hm, _, n)| hm == m && n == name).count();
            assert_eq!(count(3, "春分の日"), 1, "{y}");
            assert_eq!(count(9, "秋分の日"), 1, "{y}");
            assert_eq!(
                h.iter()
                    .filter(|&&(_, _, n)| n == "春分の日" || n == "秋分の日")
                    .count(),
                2,
                "{y}"
            );
        }
    }
}
