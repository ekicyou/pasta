//! lua_request module integration tests
//!
//! Tests for SHIORI request to Lua table conversion functionality.

use pasta_lua::mlua::Lua;

// Re-export lua_request module for testing (need to expose it)
mod lua_request_bridge {
    // We'll test through the public API
}

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
            year >= 2020 && year <= 2100,
            "year should be reasonable: {}",
            year
        );
        assert!(month >= 1 && month <= 12, "month should be 1-12: {}", month);
        assert!(day >= 1 && day <= 31, "day should be 1-31: {}", day);
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
        assert!(yday >= 1 && yday <= 366, "yday should be 1-366: {}", yday);
        assert!(wday <= 6, "wday should be 0-6: {}", wday);
    }
}

// ============================================================================
// Task 7.3: parse_request function tests - Basic functionality
// ============================================================================

mod parse_request_basic_tests {
    use super::*;
    use pasta::lua_request::parse_request;

    const SHIORI3_GET_REQUEST: &str = "GET SHIORI/3.0\r\n\
        Charset: UTF-8\r\n\
        Sender: SSP\r\n\
        SecurityLevel: local\r\n\
        ID: OnBoot\r\n\
        BaseID: OnBoot\r\n\
        Status: starting\r\n\
        Reference0: shell\r\n\
        Reference1: first\r\n\
        Reference2: second\r\n\
        \r\n";

    #[test]
    fn test_parse_request_extracts_method_and_version() {
        let lua = create_test_lua();
        let table = parse_request(&lua, SHIORI3_GET_REQUEST).expect("parse_request should succeed");

        let method: String = table.get("method").expect("method should exist");
        let version: i32 = table.get("version").expect("version should exist");

        assert_eq!(method, "get", "method should be 'get'");
        assert_eq!(version, 30, "version should be 30 for SHIORI/3.0");
    }

    #[test]
    fn test_parse_request_extracts_basic_fields() {
        let lua = create_test_lua();
        let table = parse_request(&lua, SHIORI3_GET_REQUEST).expect("parse_request should succeed");

        let charset: String = table.get("charset").expect("charset should exist");
        let id: String = table.get("id").expect("id should exist");
        let sender: String = table.get("sender").expect("sender should exist");
        let security_level: String = table
            .get("security_level")
            .expect("security_level should exist");
        let status: String = table.get("status").expect("status should exist");
        let base_id: String = table.get("base_id").expect("base_id should exist");

        assert_eq!(charset, "UTF-8");
        assert_eq!(id, "OnBoot");
        assert_eq!(sender, "SSP");
        assert_eq!(security_level, "local");
        assert_eq!(status, "starting");
        assert_eq!(base_id, "OnBoot");
    }

    #[test]
    fn test_parse_request_extracts_reference_array() {
        let lua = create_test_lua();
        let table = parse_request(&lua, SHIORI3_GET_REQUEST).expect("parse_request should succeed");

        let reference: pasta_lua::mlua::Table = table
            .get("reference")
            .expect("reference table should exist");

        let ref0: String = reference.get(0).expect("reference[0] should exist");
        let ref1: String = reference.get(1).expect("reference[1] should exist");
        let ref2: String = reference.get(2).expect("reference[2] should exist");

        assert_eq!(ref0, "shell");
        assert_eq!(ref1, "first");
        assert_eq!(ref2, "second");
    }

    #[test]
    fn test_parse_request_extracts_dic_subtable() {
        let lua = create_test_lua();
        let table = parse_request(&lua, SHIORI3_GET_REQUEST).expect("parse_request should succeed");

        let dic: pasta_lua::mlua::Table = table.get("dic").expect("dic table should exist");

        // All key-value pairs should be in dic
        let charset: String = dic.get("Charset").expect("dic['Charset'] should exist");
        let sender: String = dic.get("Sender").expect("dic['Sender'] should exist");
        let id: String = dic.get("ID").expect("dic['ID'] should exist");
        let ref0: String = dic
            .get("Reference0")
            .expect("dic['Reference0'] should exist");

        assert_eq!(charset, "UTF-8");
        assert_eq!(sender, "SSP");
        assert_eq!(id, "OnBoot");
        assert_eq!(ref0, "shell");
    }

    #[test]
    fn test_parse_notify_request() {
        let lua = create_test_lua();
        let request = "NOTIFY SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            Sender: SSP\r\n\
            ID: OnSecondChange\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");

        let method: String = table.get("method").expect("method should exist");
        assert_eq!(method, "notify", "method should be 'notify'");
    }

    #[test]
    fn test_parse_request_contains_date_subtable() {
        let lua = create_test_lua();
        let table = parse_request(&lua, SHIORI3_GET_REQUEST).expect("parse_request should succeed");

        // Verify date subtable exists and contains expected fields
        let date: pasta_lua::mlua::Table = table.get("date").expect("date table should exist");

        // Verify all expected fields exist
        let year: i32 = date.get("year").expect("date.year should exist");
        let month: u8 = date.get("month").expect("date.month should exist");
        let day: u8 = date.get("day").expect("date.day should exist");
        let hour: u8 = date.get("hour").expect("date.hour should exist");
        let min: u8 = date.get("min").expect("date.min should exist");
        let sec: u8 = date.get("sec").expect("date.sec should exist");
        let _ns: u32 = date.get("ns").expect("date.ns should exist");
        let _yday: u16 = date.get("yday").expect("date.yday should exist");
        let _wday: u8 = date.get("wday").expect("date.wday should exist");

        // Sanity checks
        assert!(year >= 2020 && year <= 2100, "year should be reasonable");
        assert!(month >= 1 && month <= 12, "month should be 1-12");
        assert!(day >= 1 && day <= 31, "day should be 1-31");
        assert!(hour <= 23, "hour should be 0-23");
        assert!(min <= 59, "min should be 0-59");
        assert!(sec <= 59, "sec should be 0-59");
    }
}

// ============================================================================
// Task 7.4: parse_request function tests - Error handling
// ============================================================================

mod parse_request_error_tests {
    use super::*;
    use pasta::lua_request::parse_request;

    #[test]
    fn test_parse_request_returns_error_for_invalid_request() {
        let lua = create_test_lua();
        let invalid_request = "INVALID REQUEST FORMAT";

        let result = parse_request(&lua, invalid_request);

        assert!(result.is_err(), "Invalid request should return error");
    }

    #[test]
    fn test_parse_request_returns_error_for_empty_string() {
        let lua = create_test_lua();

        let result = parse_request(&lua, "");

        assert!(result.is_err(), "Empty string should return error");
    }

    #[test]
    fn test_parse_request_returns_error_for_partial_request() {
        let lua = create_test_lua();
        let partial_request = "GET SHIORI/3.0\r\n"; // Missing final CRLF

        let result = parse_request(&lua, partial_request);

        assert!(result.is_err(), "Partial request should return error");
    }
}

// ============================================================================
// Task 7.5: SHIORI 2.x format tests
// ============================================================================

mod shiori2_tests {
    use super::*;
    use pasta::lua_request::parse_request;

    #[test]
    fn test_parse_shiori2_get_request() {
        let lua = create_test_lua();
        // SHIORI/2.x format
        let request = "GET Version SHIORI/2.6\r\n\
            Charset: UTF-8\r\n\
            Sender: SSP\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");

        let method: String = table.get("method").expect("method should exist");
        let version: i32 = table.get("version").expect("version should exist");

        assert_eq!(method, "get");
        assert!(
            version >= 20 && version <= 29,
            "SHIORI/2.x version should be 20-29: {}",
            version
        );
    }
}

// ============================================================================
// Task 7.6*: Edge case tests (optional)
// ============================================================================

mod edge_case_tests {
    use super::*;
    use pasta::lua_request::parse_request;

    #[test]
    fn test_parse_request_with_many_references() {
        let lua = create_test_lua();
        let mut request = String::from(
            "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: TestEvent\r\n",
        );

        // Add 15 references
        for i in 0..15 {
            request.push_str(&format!("Reference{}: value{}\r\n", i, i));
        }
        request.push_str("\r\n");

        let table = parse_request(&lua, &request).expect("parse_request should succeed");

        let reference: pasta_lua::mlua::Table = table
            .get("reference")
            .expect("reference table should exist");

        // Check first and last references
        let ref0: String = reference.get(0).expect("reference[0] should exist");
        let ref14: String = reference.get(14).expect("reference[14] should exist");

        assert_eq!(ref0, "value0");
        assert_eq!(ref14, "value14");
    }

    #[test]
    fn test_parse_request_with_japanese_values() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnTalk\r\n\
            Reference0: こんにちは\r\n\
            Reference1: さようなら\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");

        let reference: pasta_lua::mlua::Table = table
            .get("reference")
            .expect("reference table should exist");

        let ref0: String = reference.get(0).expect("reference[0] should exist");
        let ref1: String = reference.get(1).expect("reference[1] should exist");

        assert_eq!(ref0, "こんにちは");
        assert_eq!(ref1, "さようなら");
    }
}
