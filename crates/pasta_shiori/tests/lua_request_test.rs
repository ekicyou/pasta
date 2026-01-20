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
    use pasta::lua_request::lua_date;

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
