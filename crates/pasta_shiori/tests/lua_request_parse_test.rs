//! lua_request module integration tests — parse_request cluster
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
        assert!((2020..=2100).contains(&year), "year should be reasonable");
        assert!((1..=12).contains(&month), "month should be 1-12");
        assert!((1..=31).contains(&day), "day should be 1-31");
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
            (20..=29).contains(&version),
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

// ============================================================================
// G1 (3.35): parse_request edge cases — previously unreached branches
// ============================================================================

mod g1_edge_case_tests {
    use super::*;
    use pasta::error::MyError;
    use pasta::lua_request::parse_request;

    /// Reference index that overflows i32 hits the "Invalid reference number"
    /// Script error branch (previously unreached — grammar accepts any digit
    /// run, the i32 parse is the only guard).
    #[test]
    fn test_reference_number_overflow_returns_script_error() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            Reference99999999999: value\r\n\
            \r\n";

        let err = parse_request(&lua, request).expect_err("overflowing reference must error");
        match err {
            MyError::Script { ref message } => {
                assert!(
                    message.contains("Invalid reference number: '99999999999'"),
                    "Error should name the offending number: {message}"
                );
            }
            other => panic!("Expected MyError::Script, got {other:?}"),
        }
    }

    /// Duplicate header keys: both the top-level field and dic entry take the
    /// LAST occurrence (Table::set overwrites). Pins the production semantics
    /// (note: the test-only ShioriRequest helper in src uses first-wins for
    /// dic — production behavior is authoritative).
    #[test]
    fn test_duplicate_keys_last_value_wins() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: First\r\n\
            ID: Second\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");
        let id: String = table.get("id").unwrap();
        assert_eq!(id, "Second", "top-level id should be the last occurrence");

        let dic: pasta_lua::mlua::Table = table.get("dic").unwrap();
        let dic_id: String = dic.get("ID").unwrap();
        assert_eq!(dic_id, "Second", "dic ID should be the last occurrence");
    }

    /// The grammar accepts bare-LF line endings (_eol = "\r\n" | "\n" | "\r").
    /// All existing tests use CRLF; pin the LF tolerance.
    #[test]
    fn test_lf_only_line_endings_accepted() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\nCharset: UTF-8\nID: OnBoot\n\n";

        let table = parse_request(&lua, request).expect("LF-only request should parse");
        let method: String = table.get("method").unwrap();
        let id: String = table.get("id").unwrap();
        assert_eq!(method, "get");
        assert_eq!(id, "OnBoot");
    }

    /// Unknown (key_other) headers land in dic only and never pollute the
    /// fixed top-level fields.
    #[test]
    fn test_custom_header_stored_in_dic_only() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            X-Custom-Key: custom-value\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");

        let dic: pasta_lua::mlua::Table = table.get("dic").unwrap();
        let custom: String = dic.get("X-Custom-Key").unwrap();
        assert_eq!(custom, "custom-value");

        // No top-level entry is created for unknown keys.
        let top_level: Option<String> = table.get("X-Custom-Key").unwrap();
        assert!(top_level.is_none(), "unknown keys must stay inside dic");
        // Fixed fields not present in the request stay unset.
        let sender: Option<String> = table.get("sender").unwrap();
        assert!(sender.is_none(), "sender was not sent and must stay nil");
    }

    /// Empty header values are accepted (remain matches zero characters).
    #[test]
    fn test_empty_header_value_accepted() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            Status: \r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("empty value should parse");
        let status: String = table.get("status").unwrap();
        assert_eq!(status, "", "empty value should yield an empty string");

        let dic: pasta_lua::mlua::Table = table.get("dic").unwrap();
        let dic_status: String = dic.get("Status").unwrap();
        assert_eq!(dic_status, "");
    }
}

// ============================================================================
// Task 3.2: X-Pasta-Time header injection tests
// ============================================================================

mod x_pasta_time_tests {
    use super::*;
    use pasta::lua_request::parse_request;

    /// Valid UTC time: X-Pasta-Time ヘッダーで req.date が上書きされることを確認
    #[test]
    fn test_valid_utc_time_overrides_date() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            X-Pasta-Time: 2025-07-15T10:30:00Z\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");
        let date: pasta_lua::mlua::Table = table.get("date").expect("date table should exist");

        let year: i32 = date.get("year").unwrap();
        let month: u8 = date.get("month").unwrap();
        let day: u8 = date.get("day").unwrap();
        let hour: u8 = date.get("hour").unwrap();
        let min: u8 = date.get("min").unwrap();
        let sec: u8 = date.get("sec").unwrap();

        assert_eq!(year, 2025);
        assert_eq!(month, 7);
        assert_eq!(day, 15);
        assert_eq!(hour, 10);
        assert_eq!(min, 30);
        assert_eq!(sec, 0);
    }

    /// Valid offset time: タイムゾーンオフセット付き値のフィールドが正確なこと
    #[test]
    fn test_valid_offset_time_overrides_date() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            X-Pasta-Time: 2025-01-02T15:30:45+09:00\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");
        let date: pasta_lua::mlua::Table = table.get("date").expect("date table should exist");

        let year: i32 = date.get("year").unwrap();
        let month: u8 = date.get("month").unwrap();
        let day: u8 = date.get("day").unwrap();
        let hour: u8 = date.get("hour").unwrap();
        let min: u8 = date.get("min").unwrap();
        let sec: u8 = date.get("sec").unwrap();

        assert_eq!(year, 2025);
        assert_eq!(month, 1);
        assert_eq!(day, 2);
        assert_eq!(hour, 15);
        assert_eq!(min, 30);
        assert_eq!(sec, 45);
    }

    /// Timezone offset affects wday/yday: 2025-01-02 (木曜日, yday=2) を確認
    #[test]
    fn test_offset_time_wday_yday_correct() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            X-Pasta-Time: 2025-01-02T15:30:45+09:00\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");
        let date: pasta_lua::mlua::Table = table.get("date").expect("date table should exist");

        // 2025-01-02 is Thursday (wday=4) and the 2nd day of the year (yday=2)
        let wday: u8 = date.get("wday").unwrap();
        let yday: u16 = date.get("yday").unwrap();

        assert_eq!(wday, 4, "2025-01-02 should be Thursday (wday=4)");
        assert_eq!(yday, 2, "2025-01-02 should be yday=2");
    }

    /// No header: X-Pasta-Time 無しでも正常に date が取得できること
    #[test]
    fn test_no_header_uses_system_time() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            \r\n";

        let table = parse_request(&lua, request).expect("parse_request should succeed");
        let date: pasta_lua::mlua::Table = table.get("date").expect("date table should exist");

        // date fields exist and have sane values
        let year: i32 = date.get("year").unwrap();
        let month: u8 = date.get("month").unwrap();
        let day: u8 = date.get("day").unwrap();

        assert!(
            (2020..=2100).contains(&year),
            "year should be reasonable: {}",
            year
        );
        assert!((1..=12).contains(&month), "month should be 1-12: {}", month);
        assert!((1..=31).contains(&day), "day should be 1-31: {}", day);
    }

    /// Invalid value: 不正な X-Pasta-Time → エラーが返ること
    #[test]
    fn test_invalid_value_returns_error() {
        let lua = create_test_lua();
        let request = "GET SHIORI/3.0\r\n\
            Charset: UTF-8\r\n\
            ID: OnBoot\r\n\
            X-Pasta-Time: not-a-date\r\n\
            \r\n";

        let result = parse_request(&lua, request);
        assert!(result.is_err(), "Invalid X-Pasta-Time should return error");

        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("X-Pasta-Time") || err_msg.contains("not-a-date"),
            "Error message should mention the invalid value: {}",
            err_msg
        );
    }
}

// ============================================================================
// 3.37 (G3): FFI boundary stack-safety regression.
//
// parse_request is reached from the extern "C" `request` entry point with
// host-controlled text. Stack exhaustion inside that call graph aborts the
// whole host process (SSP) — it cannot be caught. The parser must therefore
// use constant stack space regardless of the number of headers.
// ============================================================================

mod stack_safety_tests {
    use super::*;
    use pasta::lua_request::parse_request;

    /// RED evidence (3.37): with the recursive parse1 (one stack frame per
    /// header pair), this test crashed the test process with
    /// "thread '...' has overflowed its stack" inside the 256 KiB worker
    /// thread. After the iterative rewrite it passes.
    #[test]
    fn test_parse_request_many_headers_constant_stack() {
        // 256 KiB: ample for the iterative parser + a Lua state, nowhere near
        // enough for one frame per header (50_000 frames).
        let worker = std::thread::Builder::new()
            .name("parse-small-stack".into())
            .stack_size(256 * 1024)
            .spawn(|| {
                let lua = create_test_lua();
                let mut request = String::from(
                    "GET SHIORI/3.0\r\n\
                     Charset: UTF-8\r\n\
                     Sender: SSP\r\n\
                     ID: OnTest\r\n",
                );
                for i in 0..50_000 {
                    request.push_str(&format!("Reference{i}: v{i}\r\n"));
                }
                request.push_str("\r\n");

                let table =
                    parse_request(&lua, &request).expect("hostile-size request should still parse");
                let reference: pasta_lua::mlua::Table = table.get("reference").unwrap();
                let last: String = reference.get(49_999).unwrap();
                assert_eq!(last, "v49999");
            })
            .expect("spawn worker");
        worker.join().expect("worker thread must not crash");
    }
}
