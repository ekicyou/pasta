//! ShioriTestEnv integration tests
//!
//! Tests for the SHIORI test environment wrapper:
//! lifecycle, multiple requests, Lua runtime access, and X-Pasta-Time injection.

mod common;

use common::test_env::ShioriTestEnv;

// ============================================================================
// Fixture load + request lifecycle
// ============================================================================

#[test]
fn test_env_load_and_request() {
    let mut env = ShioriTestEnv::new("shiori_lifecycle");

    let request = "GET SHIORI/3.0\r\n\
                   Charset: UTF-8\r\n\
                   ID: OnBoot\r\n\
                   \r\n";

    let resp = env.request(request).expect("request should succeed");
    assert!(resp.status_code == 200 || resp.status_code == 500,
        "Expected 200 or 500, got: {}", resp.status_code);
}

// ============================================================================
// Multiple requests maintain Lua state
// ============================================================================

#[test]
fn test_env_multiple_requests_maintain_state() {
    let mut env = ShioriTestEnv::new("shiori_lifecycle");

    // First request
    let req = "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnBoot\r\n\r\n";
    let _ = env.request(req).expect("first request should succeed");

    // Check request count via runtime
    let runtime = env.runtime().expect("runtime should exist");
    let lua = runtime.lua();
    let shiori_table: pasta_lua::mlua::Table = lua.globals().get("SHIORI").unwrap();
    let count1: i64 = shiori_table.get("request_count").unwrap();
    assert_eq!(count1, 1, "request_count should be 1 after first request");

    // Second request
    let _ = env.request(req).expect("second request should succeed");
    let count2: i64 = shiori_table.get("request_count").unwrap();
    assert_eq!(count2, 2, "request_count should be 2 after second request");
}

// ============================================================================
// Runtime access for Lua state inspection
// ============================================================================

#[test]
fn test_env_runtime_access() {
    let env = ShioriTestEnv::new("shiori_lifecycle");

    let runtime = env.runtime();
    assert!(runtime.is_some(), "runtime should be available after load");

    // Verify we can access Lua globals
    let lua = runtime.unwrap().lua();
    let globals = lua.globals();
    let shiori_table: pasta_lua::mlua::Table = globals.get("SHIORI").unwrap();
    let loaded_hinst: i64 = shiori_table.get("loaded_hinst").unwrap();
    assert_eq!(loaded_hinst, 0, "hinst should be 0 (default from ShioriTestEnv)");
}

// ============================================================================
// X-Pasta-Time header produces fixed time
// ============================================================================

#[test]
fn test_env_x_pasta_time_fixed_time() {
    let mut env = ShioriTestEnv::new("shiori_lifecycle");

    let request = "GET SHIORI/3.0\r\n\
                   Charset: UTF-8\r\n\
                   ID: OnBoot\r\n\
                   X-Pasta-Time: 2025-07-15T10:30:00Z\r\n\
                   \r\n";

    let resp = env.request(request).expect("request with X-Pasta-Time should succeed");
    // The request should process successfully (not 400)
    assert_ne!(resp.status_code, 400,
        "Valid X-Pasta-Time should not produce 400: {:?}", resp);
}

// ============================================================================
// Invalid X-Pasta-Time produces 400 with X-ERROR-REASON
// ============================================================================

#[test]
fn test_env_invalid_x_pasta_time_returns_400() {
    let mut env = ShioriTestEnv::new("shiori_lifecycle");

    let request = "GET SHIORI/3.0\r\n\
                   Charset: UTF-8\r\n\
                   ID: OnBoot\r\n\
                   X-Pasta-Time: not-a-valid-date\r\n\
                   \r\n";

    let resp = env.request(request).expect("request should return parsed response");
    assert_eq!(resp.status_code, 400, "Invalid X-Pasta-Time should produce 400");
    let reason = resp.header("X-ERROR-REASON");
    assert!(reason.is_some(), "400 response should have X-ERROR-REASON header");
    assert!(reason.unwrap().contains("X-Pasta-Time"),
        "X-ERROR-REASON should mention X-Pasta-Time: {}", reason.unwrap());
}
