//! SHIORI Integration Tests with hello-pasta Sample Ghost
//!
//! These tests verify the complete SHIORI load/request pipeline
//! using the actual hello-pasta ghost from pasta_sample_ghost.

mod common;

use common::copy_sample_ghost_to_temp;
use pasta::PastaShiori;

// ========================================================================
// PastaShiori::load のテスト
// ========================================================================

/// Test that PastaShiori::load successfully initializes with hello-pasta ghost.
#[test]
fn test_load_hello_pasta() {
    let temp = copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();

    let result = shiori.load(temp.path().as_os_str());
    assert!(result.is_ok(), "load() should not return error");
    assert!(result.unwrap(), "load() should return true on success");
}

// ========================================================================
// OnBoot リクエストのテスト
// ========================================================================

/// Test that OnBoot request returns expected SakuraScript response.
#[test]
fn test_onboot_response() {
    let temp = copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();

    // Load ghost
    let load_result = shiori.load(temp.path().as_os_str());
    assert!(load_result.is_ok(), "load() should succeed");
    assert!(load_result.unwrap(), "load() should return true");

    // Build complete SHIORI/3.0 OnBoot request (CRLF line endings)
    let request = "GET SHIORI/3.0\r\n\
        Charset: UTF-8\r\n\
        Sender: SSP\r\n\
        SecurityLevel: local\r\n\
        ID: OnBoot\r\n\
        Reference0: マスターシェル\r\n\
        \r\n";

    let response = shiori.request(request);
    assert!(response.is_ok(), "request() should not return error");
    let response = response.unwrap();

    println!("=== Response ===");
    println!("{}", response);

    // Verify 200 OK status
    assert!(
        response.contains("SHIORI/3.0 200 OK"),
        "Response should contain '200 OK', got: {}",
        response
    );

    // Verify Value header exists
    assert!(
        response.contains("Value:"),
        "Response should contain 'Value:' header, got: {}",
        response
    );

    // Verify spot switching tag \p[0] (女の子 = spot 0)
    // Note: pasta uses \p[n] format instead of \0/\1.
    // OnBoot (dic/01-boot.pasta) has no expression, so \s[n] is not emitted.
    assert!(
        response.contains("\\p[0]"),
        "Response should contain spot switching tag '\\p[0]', got: {}",
        response
    );

    // Verify wait tag (\_w[) from pasta.toml [talk] section
    assert!(
        response.contains("\\_w["),
        "Response should contain wait tag '\\_w[', got: {}",
        response
    );

    // Verify text content from OnBoot scene (dic/01-boot.pasta)
    assert!(
        response.contains("本日のおすすめは"),
        "Response should contain OnBoot greeting text, got: {}",
        response
    );
}
