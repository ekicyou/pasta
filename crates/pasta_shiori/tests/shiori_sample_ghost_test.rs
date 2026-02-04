//! SHIORI Integration Tests with hello-pasta Sample Ghost
//!
//! These tests verify the complete SHIORI load/request pipeline
//! using the actual hello-pasta ghost from pasta_sample_ghost.
//!
//! Related specification: `.kiro/specs/shiori-integration-test/`

mod common;

use common::copy_sample_ghost_to_temp;
use pasta::{PastaShiori, Shiori};

// ========================================================================
// Task 3.2: PastaShiori::load のテスト
// ========================================================================

/// Test that PastaShiori::load successfully initializes with hello-pasta ghost.
/// Requirements: 4.1, 4.2, 4.3
#[test]
fn test_load_hello_pasta() {
    let temp = copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();

    let result = shiori.load(1, temp.path().as_os_str());
    assert!(result.is_ok(), "load() should not return error");
    assert!(result.unwrap(), "load() should return true on success");
}

// ========================================================================
// Task 3.3: OnBoot リクエストのテスト
// ========================================================================

/// Test that OnBoot request returns expected SakuraScript response.
/// Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6
#[test]
fn test_onboot_response() {
    let temp = copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();

    // Load ghost
    let load_result = shiori.load(1, temp.path().as_os_str());
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

    // 5.1: Verify 200 OK status
    assert!(
        response.contains("SHIORI/3.0 200 OK"),
        "Response should contain '200 OK', got: {}",
        response
    );

    // 5.2: Verify Value header exists
    assert!(
        response.contains("Value:"),
        "Response should contain 'Value:' header, got: {}",
        response
    );

    // 5.3: Verify spot switching tags (\p[0] or \p[1])
    // Note: pasta uses \p[n] format instead of \0/\1
    let has_spot_tag = response.contains("\\p[0]") || response.contains("\\p[1]");
    assert!(
        has_spot_tag,
        "Response should contain spot switching tags (\\p[0] or \\p[1]), got: {}",
        response
    );

    // 5.4: Verify expression tag (\s[n])
    // Note: pasta uses \s[n] format with surface ID numbers
    assert!(
        response.contains("\\s["),
        "Response should contain expression tag '\\s[', got: {}",
        response
    );

    // 5.5: Verify wait tag (\_w[) from pasta.toml [talk] section
    assert!(
        response.contains("\\_w["),
        "Response should contain wait tag '\\_w[', got: {}",
        response
    );

    // 5.6: Verify text content from OnBoot scene
    assert!(
        response.contains("起動したよ")
            || response.contains("起動挨拶")
            || response.contains("おはよう"),
        "Response should contain OnBoot greeting text, got: {}",
        response
    );
}
