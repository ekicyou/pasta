//! ShioriResponse parser tests
//!
//! Tests for SHIORI/3.0 response parsing into structured fields.

mod common;

use common::response::{ShioriResponse, ShioriResponseError};

// ============================================================================
// 200 OK with Value
// ============================================================================

#[test]
fn test_parse_200_ok_with_value() {
    let text = "SHIORI/3.0 200 OK\r\n\
                Charset: UTF-8\r\n\
                Sender: Pasta\r\n\
                Value: Hello World\r\n\
                \r\n";

    let resp = ShioriResponse::parse(text).unwrap();
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.status_text, "OK");
    assert_eq!(resp.value, Some("Hello World".to_string()));
    assert!(resp.is_success());
    assert_eq!(resp.header("Charset"), Some("UTF-8"));
    assert_eq!(resp.header("Sender"), Some("Pasta"));
}

// ============================================================================
// 204 No Content (no Value)
// ============================================================================

#[test]
fn test_parse_204_no_content() {
    let text = "SHIORI/3.0 204 No Content\r\n\
                Charset: UTF-8\r\n\
                Sender: Pasta\r\n\
                \r\n";

    let resp = ShioriResponse::parse(text).unwrap();
    assert_eq!(resp.status_code, 204);
    assert_eq!(resp.status_text, "No Content");
    assert_eq!(resp.value, None);
    assert!(resp.is_success());
}

// ============================================================================
// 400 / 500 status codes
// ============================================================================

#[test]
fn test_parse_400_bad_request() {
    let text = "SHIORI/3.0 400 Bad Request\r\n\
                Charset: UTF-8\r\n\
                X-ERROR-REASON: Invalid input\r\n\
                \r\n";

    let resp = ShioriResponse::parse(text).unwrap();
    assert_eq!(resp.status_code, 400);
    assert_eq!(resp.status_text, "Bad Request");
    assert!(!resp.is_success());
    assert_eq!(resp.header("X-ERROR-REASON"), Some("Invalid input"));
}

#[test]
fn test_parse_500_internal_server_error() {
    let text = "SHIORI/3.0 500 Internal Server Error\r\n\
                Charset: UTF-8\r\n\
                X-ERROR-REASON: something broke\r\n\
                \r\n";

    let resp = ShioriResponse::parse(text).unwrap();
    assert_eq!(resp.status_code, 500);
    assert_eq!(resp.status_text, "Internal Server Error");
    assert!(!resp.is_success());
}

// ============================================================================
// Multiple custom headers
// ============================================================================

#[test]
fn test_parse_multiple_custom_headers() {
    let text = "SHIORI/3.0 200 OK\r\n\
                Charset: UTF-8\r\n\
                Sender: Pasta\r\n\
                X-Custom-One: value1\r\n\
                X-Custom-Two: value2\r\n\
                Value: test\r\n\
                \r\n";

    let resp = ShioriResponse::parse(text).unwrap();
    assert_eq!(resp.header("X-Custom-One"), Some("value1"));
    assert_eq!(resp.header("X-Custom-Two"), Some("value2"));
    assert_eq!(resp.value, Some("test".to_string()));
}

// ============================================================================
// Case-insensitive header lookup
// ============================================================================

#[test]
fn test_header_case_insensitive() {
    let text = "SHIORI/3.0 200 OK\r\n\
                Charset: UTF-8\r\n\
                X-Error-Reason: test error\r\n\
                \r\n";

    let resp = ShioriResponse::parse(text).unwrap();
    assert_eq!(resp.header("x-error-reason"), Some("test error"));
    assert_eq!(resp.header("X-ERROR-REASON"), Some("test error"));
    assert_eq!(resp.header("X-Error-Reason"), Some("test error"));
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_parse_empty_string_returns_error() {
    let result = ShioriResponse::parse("");
    assert_eq!(result.unwrap_err(), ShioriResponseError::Empty);
}

#[test]
fn test_parse_missing_status_code_returns_error() {
    let text = "SHIORI/3.0\r\n\r\n";
    let result = ShioriResponse::parse(text);
    assert!(matches!(
        result.unwrap_err(),
        ShioriResponseError::InvalidStatusLine(_)
    ));
}

#[test]
fn test_parse_non_numeric_status_code() {
    let text = "SHIORI/3.0 abc OK\r\n\r\n";
    let result = ShioriResponse::parse(text);
    assert!(matches!(
        result.unwrap_err(),
        ShioriResponseError::InvalidStatusLine(_)
    ));
}
