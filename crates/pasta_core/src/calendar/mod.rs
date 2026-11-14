//! 暦（日付の算術・祝日の規則・`calendar.csv` の表）。

mod civil;
mod holiday;
mod table;

pub use crate::error::{CalendarError, CalendarProblem};
pub use holiday::holiday_name;
pub use table::CalendarTable;

/// ghost/master の直下に置く行事の表のファイル名。
pub const CALENDAR_FILE_NAME: &str = "calendar.csv";

/// ghost/master の直下のファイル名が行事の表か（ASCII の大文字・小文字を区別しない。完全一致のみ）。
pub fn is_calendar_file_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(CALENDAR_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_predicate() {
        for ok in ["calendar.csv", "Calendar.CSV", "CALENDAR.csv"] {
            assert!(is_calendar_file_name(ok), "{ok}");
        }
        for ng in ["calendar.csv.bak", "dic/calendar.csv", "xcalendar.csv", ""] {
            assert!(!is_calendar_file_name(ng), "{ng}");
        }
    }
}
