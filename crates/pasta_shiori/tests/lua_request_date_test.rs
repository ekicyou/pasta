//! lua_request module integration tests — lua_date conversion cluster
//!
//! Tests for the `lua_date` / `lua_date_from` datetime-table conversion functions.

use pasta_lua::mlua::Lua;

/// Test helper: Create a Lua instance for testing
fn create_test_lua() -> Lua {
    Lua::new()
}

// ============================================================================
// Task 7.2: lua_date function tests
// ============================================================================

mod lua_date_tests {
    use super::*;
    use pasta::lua_request::{lua_date, lua_date_from};
    use time::{Date, Month, PrimitiveDateTime, Time, UtcOffset};

    #[test]
    fn test_lua_date_from_with_fixed_datetime() {
        let lua = create_test_lua();

        // 2025年1月2日 15:30:45.123456789 (UTC+9, 木曜日)
        let date = Date::from_calendar_date(2025, Month::January, 2).unwrap();
        let time = Time::from_hms_nano(15, 30, 45, 123_456_789).unwrap();
        let primitive = PrimitiveDateTime::new(date, time);
        let dt = primitive.assume_offset(UtcOffset::from_hms(9, 0, 0).unwrap());

        let table = lua_date_from(&lua, dt).expect("lua_date_from should succeed");

        // Verify exact values
        let year: i32 = table.get("year").unwrap();
        let month: u8 = table.get("month").unwrap();
        let day: u8 = table.get("day").unwrap();
        let hour: u8 = table.get("hour").unwrap();
        let min: u8 = table.get("min").unwrap();
        let sec: u8 = table.get("sec").unwrap();
        let ns: u32 = table.get("ns").unwrap();

        assert_eq!(year, 2025);
        assert_eq!(month, 1); // January
        assert_eq!(day, 2);
        assert_eq!(hour, 15);
        assert_eq!(min, 30);
        assert_eq!(sec, 45);
        assert_eq!(ns, 123_456_789);

        // Unix timestamp (2025-01-02T15:30:45 UTC+9 = 2025-01-02T06:30:45 UTC)
        let unix: i64 = table.get("unix").unwrap();
        assert_eq!(unix, dt.unix_timestamp());

        // 2025-01-02 is the 2nd day of the year
        let yday: u16 = table.get("yday").unwrap();
        let ordinal: u16 = table.get("ordinal").unwrap();
        assert_eq!(yday, 2);
        assert_eq!(ordinal, 2);

        // 2025-01-02 is Thursday (4 days from Sunday)
        let wday: u8 = table.get("wday").unwrap();
        let num_days_from_sunday: u8 = table.get("num_days_from_sunday").unwrap();
        assert_eq!(wday, 4); // Thursday
        assert_eq!(num_days_from_sunday, 4);
    }

    #[test]
    fn test_lua_date_from_sunday() {
        let lua = create_test_lua();

        // 2025年1月5日 (日曜日)
        let date = Date::from_calendar_date(2025, Month::January, 5).unwrap();
        let time = Time::from_hms(0, 0, 0).unwrap();
        let primitive = PrimitiveDateTime::new(date, time);
        let dt = primitive.assume_offset(UtcOffset::UTC);

        let table = lua_date_from(&lua, dt).expect("lua_date_from should succeed");

        let wday: u8 = table.get("wday").unwrap();
        assert_eq!(wday, 0, "Sunday should be 0");
    }

    #[test]
    fn test_lua_date_from_saturday() {
        let lua = create_test_lua();

        // 2025年1月4日 (土曜日)
        let date = Date::from_calendar_date(2025, Month::January, 4).unwrap();
        let time = Time::from_hms(23, 59, 59).unwrap();
        let primitive = PrimitiveDateTime::new(date, time);
        let dt = primitive.assume_offset(UtcOffset::UTC);

        let table = lua_date_from(&lua, dt).expect("lua_date_from should succeed");

        let wday: u8 = table.get("wday").unwrap();
        assert_eq!(wday, 6, "Saturday should be 6");
    }

    #[test]
    fn test_lua_date_from_leap_year() {
        let lua = create_test_lua();

        // 2024年12月31日 (うるう年の最終日 = 366日目)
        let date = Date::from_calendar_date(2024, Month::December, 31).unwrap();
        let time = Time::from_hms(12, 0, 0).unwrap();
        let primitive = PrimitiveDateTime::new(date, time);
        let dt = primitive.assume_offset(UtcOffset::UTC);

        let table = lua_date_from(&lua, dt).expect("lua_date_from should succeed");

        let yday: u16 = table.get("yday").unwrap();
        assert_eq!(yday, 366, "Leap year should have 366 days");
    }

    #[test]
    fn test_lua_date_returns_table_with_basic_datetime_fields() {
        let lua = create_test_lua();
        let result = lua_date(&lua);

        assert!(result.is_ok(), "lua_date should return Ok");
        let table = result.unwrap();

        // Verify basic datetime fields exist and are numeric
        let year: i32 = table.get("year").expect("year field should exist");
        let month: u8 = table.get("month").expect("month field should exist");
        let day: u8 = table.get("day").expect("day field should exist");
        let hour: u8 = table.get("hour").expect("hour field should exist");
        let min: u8 = table.get("min").expect("min field should exist");
        let sec: u8 = table.get("sec").expect("sec field should exist");
        let ns: u32 = table.get("ns").expect("ns field should exist");

        // Basic sanity checks
        assert!(
            (2020..=2100).contains(&year),
            "year should be reasonable: {}",
            year
        );
        assert!((1..=12).contains(&month), "month should be 1-12: {}", month);
        assert!((1..=31).contains(&day), "day should be 1-31: {}", day);
        assert!(hour <= 23, "hour should be 0-23: {}", hour);
        assert!(min <= 59, "min should be 0-59: {}", min);
        assert!(sec <= 59, "sec should be 0-59: {}", sec);
        assert!(ns <= 999_999_999, "ns should be 0-999999999: {}", ns);
    }

    #[test]
    fn test_lua_date_contains_yday_and_wday_fields() {
        let lua = create_test_lua();
        let table = lua_date(&lua).expect("lua_date should succeed");

        // yday (ordinal day of year)
        let yday: u16 = table.get("yday").expect("yday field should exist");
        let ordinal: u16 = table.get("ordinal").expect("ordinal field should exist");

        // wday (day of week, 0=Sunday)
        let wday: u8 = table.get("wday").expect("wday field should exist");
        let num_days_from_sunday: u8 = table
            .get("num_days_from_sunday")
            .expect("num_days_from_sunday field should exist");

        // Verify aliases are equal
        assert_eq!(yday, ordinal, "yday and ordinal should be the same");
        assert_eq!(
            wday, num_days_from_sunday,
            "wday and num_days_from_sunday should be the same"
        );

        // Sanity checks
        assert!((1..=366).contains(&yday), "yday should be 1-366: {}", yday);
        assert!(wday <= 6, "wday should be 0-6: {}", wday);
    }
}
