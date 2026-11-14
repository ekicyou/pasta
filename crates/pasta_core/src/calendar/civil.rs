//! 日付の算術（Howard Hinnant の civil 算術）。`os.time` は使わない。通日は 1970-01-01 を 0 とする。

pub(crate) fn is_leap(year: i32) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

/// 1〜12 以外の月は 0。
pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

pub(crate) fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = i64::from((m + 9) % 12); // 3月=0 … 2月=11
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub(crate) fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((yoe + era * 400 + i64::from(m <= 2)) as i32, m, d)
}

/// 0＝日曜。
pub(crate) fn weekday(z: i64) -> u32 {
    (z + 4).rem_euclid(7) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_and_month_days() {
        assert!(is_leap(2000) && is_leap(2024) && !is_leap(1900) && !is_leap(2023));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 4), 30);
        assert_eq!(days_in_month(2023, 12), 31);
        assert_eq!(days_in_month(2023, 0), 0);
        assert_eq!(days_in_month(2023, 13), 0);
    }

    #[test]
    fn epoch_and_known_weekdays() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(weekday(0), 4); // 木曜
        let w = |y, m, d| weekday(days_from_civil(y, m, d));
        assert_eq!(w(2000, 1, 1), 6); // 土
        assert_eq!(w(2024, 2, 29), 4); // 木
        assert_eq!(w(2026, 11, 15), 0); // 日
        assert_eq!(w(1900, 1, 1), 1); // 月
        assert_eq!(w(2100, 12, 31), 5); // 金
    }

    #[test]
    fn every_day_1900_2100_round_trips() {
        let start = days_from_civil(1900, 1, 1);
        let end = days_from_civil(2100, 12, 31);
        let mut prev = start - 1;
        for z in start..=end {
            assert_eq!(z, prev + 1);
            prev = z;
            let (y, m, d) = civil_from_days(z);
            assert!(d >= 1 && d <= days_in_month(y, m), "{y}-{m}-{d}");
            assert_eq!(days_from_civil(y, m, d), z);
            assert_eq!(weekday(z), (weekday(z - 1) + 1) % 7);
        }
    }

    #[test]
    fn year_one_and_negative_years() {
        assert_eq!(civil_from_days(days_from_civil(1, 1, 1)), (1, 1, 1));
        assert_eq!(weekday(days_from_civil(1, 1, 1)), 1); // 先発グレゴリオ暦で月曜
        for y in [0, -1, -4, -100, -400, -1000] {
            let z = days_from_civil(y, 3, 1);
            assert_eq!(civil_from_days(z), (y, 3, 1));
            assert_eq!(
                days_from_civil(y, 3, 1) - days_from_civil(y, 2, 1),
                days_in_month(y, 2) as i64
            );
        }
        assert!(is_leap(0) && is_leap(-4) && !is_leap(-100) && is_leap(-400));
    }
}
