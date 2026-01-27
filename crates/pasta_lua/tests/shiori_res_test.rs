//! Integration tests for pasta.shiori.res module (SHIORI/3.0 response builder).
//!
//! Tests verify that the RES module correctly generates SHIORI/3.0 responses.

use pasta_lua::{PastaLuaRuntime, TranspileContext};
use std::path::PathBuf;

/// Helper to create an empty TranspileContext for testing.
fn create_empty_context() -> TranspileContext {
    TranspileContext::new()
}

/// Helper to get the scripts directory path as a Lua-compatible string.
fn get_scripts_dir() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .to_string_lossy()
        .replace('\\', "/")
}

/// Helper to create a runtime with package.path configured for pasta modules.
fn create_runtime_with_pasta_path() -> PastaLuaRuntime {
    let ctx = create_empty_context();
    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let scripts_dir = get_scripts_dir();
    runtime
        .exec(&format!(
            r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
        ))
        .expect("Failed to configure package.path");
    runtime
}

// ============================================================================
// Unit Tests: RES module functions
// ============================================================================

#[test]
fn test_res_module_loads() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        return RES ~= nil
    "#,
    );

    assert!(result.is_ok(), "RES module should load: {:?}", result);
    assert!(result.unwrap().as_boolean().unwrap_or(false));
}

#[test]
fn test_res_env_defaults() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local assertions = require "@assertions"
        
        assertions.assert_eq(RES.env.charset, "UTF-8", "default charset")
        assertions.assert_eq(RES.env.sender, "Pasta", "default sender")
        assertions.assert_eq(RES.env.security_level, "local", "default security_level")
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.env defaults failed: {:?}", result);
}

#[test]
fn test_res_ok_generates_200_response() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.ok("test value")
        
        -- Check status line
        assert(response:find("SHIORI/3.0 200 OK"), "should contain 200 OK status")
        -- Check Value header
        assert(response:find("Value: test value"), "should contain Value header")
        -- Check standard headers
        assert(response:find("Charset: UTF%-8"), "should contain Charset header")
        assert(response:find("Sender: Pasta"), "should contain Sender header")
        assert(response:find("SecurityLevel: local"), "should contain SecurityLevel header")
        -- Check termination
        assert(response:sub(-4) == "\r\n\r\n", "should end with double CRLF")
        
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.ok test failed: {:?}", result);
}

#[test]
fn test_res_no_content_generates_204_response() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.no_content()
        
        -- Check status line
        assert(response:find("SHIORI/3.0 204 No Content"), "should contain 204 status")
        -- Check standard headers
        assert(response:find("Charset: UTF%-8"), "should contain Charset header")
        -- Check termination
        assert(response:sub(-4) == "\r\n\r\n", "should end with double CRLF")
        
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.no_content test failed: {:?}", result);
}

#[test]
fn test_res_no_content_with_custom_header() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.no_content({["X-Custom"] = "custom-value"})
        
        -- Check custom header
        assert(response:find("X%-Custom: custom%-value"), "should contain custom header")
        -- Still a 204 response
        assert(response:find("SHIORI/3.0 204 No Content"), "should be 204")
        
        return true
    "#,
    );

    assert!(
        result.is_ok(),
        "RES.no_content with custom header failed: {:?}",
        result
    );
}

#[test]
fn test_res_err_generates_500_response() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.err("test error reason")
        
        -- Check status line
        assert(response:find("SHIORI/3.0 500 Internal Server Error"), "should contain 500 status")
        -- Check X-Error-Reason header
        assert(response:find("X%-Error%-Reason: test error reason"), "should contain X-Error-Reason header")
        
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.err test failed: {:?}", result);
}

#[test]
fn test_res_warn_generates_204_with_warning() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.warn("test warning")
        
        -- Check status line (204, not 500)
        assert(response:find("SHIORI/3.0 204 No Content"), "should be 204")
        -- Check X-Warn-Reason header
        assert(response:find("X%-Warn%-Reason: test warning"), "should contain X-Warn-Reason header")
        
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.warn test failed: {:?}", result);
}

#[test]
fn test_res_env_modification_reflected() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        
        -- Modify env
        RES.env.charset = "Shift_JIS"
        RES.env.sender = "TestGhost"
        
        local response = RES.no_content()
        
        -- Check modified headers
        assert(response:find("Charset: Shift_JIS"), "should reflect modified charset")
        assert(response:find("Sender: TestGhost"), "should reflect modified sender")
        
        return true
    "#,
    );

    assert!(
        result.is_ok(),
        "RES.env modification test failed: {:?}",
        result
    );
}

// ============================================================================
// Unit Tests: TEACH-related responses (311, 312)
// ============================================================================

#[test]
fn test_res_not_enough_generates_311_response() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.not_enough()
        
        assert(response:find("SHIORI/3.0 311 Not Enough"), "should contain 311 status")
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.not_enough test failed: {:?}", result);
}

#[test]
fn test_res_advice_generates_312_response() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.advice()
        
        assert(response:find("SHIORI/3.0 312 Advice"), "should contain 312 status")
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.advice test failed: {:?}", result);
}

// ============================================================================
// Unit Tests: Error responses (400)
// ============================================================================

#[test]
fn test_res_bad_request_generates_400_response() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.bad_request()
        
        assert(response:find("SHIORI/3.0 400 Bad Request"), "should contain 400 status")
        return true
    "#,
    );

    assert!(result.is_ok(), "RES.bad_request test failed: {:?}", result);
}

// ============================================================================
// Integration Tests: Header order and response format
// ============================================================================

#[test]
fn test_standard_headers_order() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        local response = RES.no_content()
        
        -- Find positions of standard headers
        local charset_pos = response:find("Charset:")
        local sender_pos = response:find("Sender:")
        local security_pos = response:find("SecurityLevel:")
        
        -- Verify order: Charset < Sender < SecurityLevel
        assert(charset_pos < sender_pos, "Charset should come before Sender")
        assert(sender_pos < security_pos, "Sender should come before SecurityLevel")
        
        return true
    "#,
    );

    assert!(result.is_ok(), "Header order test failed: {:?}", result);
}

#[test]
fn test_response_terminates_with_double_crlf() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        
        -- Test multiple response types
        local responses = {
            RES.ok("test"),
            RES.no_content(),
            RES.not_enough(),
            RES.advice(),
            RES.bad_request(),
            RES.err("error"),
            RES.warn("warning"),
        }
        
        for i, response in ipairs(responses) do
            assert(response:sub(-4) == "\r\n\r\n", 
                "Response " .. i .. " should end with \\r\\n\\r\\n")
        end
        
        return true
    "#,
    );

    assert!(result.is_ok(), "CRLF termination test failed: {:?}", result);
}

#[test]
fn test_defensive_nil_handling() {
    let runtime = create_runtime_with_pasta_path();

    let result = runtime.exec(
        r#"
        local RES = require "pasta.shiori.res"
        
        -- All these should work without error (nil dic)
        local r1 = RES.ok("value")        -- dic is nil
        local r2 = RES.no_content()       -- dic is nil
        local r3 = RES.not_enough()       -- dic is nil
        local r4 = RES.advice()           -- dic is nil
        local r5 = RES.bad_request()      -- dic is nil
        local r6 = RES.err("reason")      -- dic is nil
        local r7 = RES.warn("warning")    -- dic is nil
        
        -- Verify all responses are valid strings
        assert(type(r1) == "string", "RES.ok should return string")
        assert(type(r2) == "string", "RES.no_content should return string")
        assert(type(r3) == "string", "RES.not_enough should return string")
        assert(type(r4) == "string", "RES.advice should return string")
        assert(type(r5) == "string", "RES.bad_request should return string")
        assert(type(r6) == "string", "RES.err should return string")
        assert(type(r7) == "string", "RES.warn should return string")
        
        return true
    "#,
    );

    assert!(
        result.is_ok(),
        "Defensive nil handling test failed: {:?}",
        result
    );
}
