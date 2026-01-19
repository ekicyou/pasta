//! SHIORI Lifecycle Integration Tests - Verifying Lua Code Execution
//!
//! These tests verify that the SHIORI load/request/unload lifecycle
//! actually executes Lua code with observable side effects.
//!
//! Related specification: `.kiro/specs/shiori-lifecycle-lua-execution-test/`

mod common;

use common::copy_fixture_to_temp;
use pasta::{PastaShiori, Shiori};

// ========================================================================
// Task 3.1: SHIORI.load実行確認テスト
// ========================================================================

/// Test that SHIORI.load sets observable Lua global variables.
/// Verifies: loaded_hinst and load_dir are set in Lua state.
#[test]
fn test_shiori_load_sets_globals() {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let mut shiori = PastaShiori::default();

    // Execute SHIORI.load
    let load_result = shiori.load(42, temp.path().as_os_str());
    assert!(load_result.is_ok(), "load() should succeed");
    assert!(load_result.unwrap(), "load() should return true");

    // Access Lua globals to verify load was called
    let runtime = shiori.runtime().expect("runtime should exist after load");
    let lua = runtime.lua();
    let globals = lua.globals();

    // Verify SHIORI.loaded_hinst was set
    let shiori_table: pasta_lua::mlua::Table =
        globals.get("SHIORI").expect("SHIORI table should exist");
    let loaded_hinst: i64 = shiori_table
        .get("loaded_hinst")
        .expect("SHIORI.loaded_hinst should be set");
    assert_eq!(loaded_hinst, 42, "loaded_hinst should match passed value");

    // Verify SHIORI.load_dir was set
    let load_dir: String = shiori_table
        .get("load_dir")
        .expect("SHIORI.load_dir should be set");
    let expected_path = temp.path().to_string_lossy().to_string();
    assert_eq!(load_dir, expected_path, "load_dir should match temp path");
}

// ========================================================================
// Task 3.2: SHIORI.request呼び出しカウントテスト
// ========================================================================

/// Test that SHIORI.request increments the request counter.
/// Verifies: request_count is incremented on each call.
#[test]
fn test_shiori_request_increments_counter() {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let mut shiori = PastaShiori::default();

    // Load
    assert!(shiori.load(1, temp.path().as_os_str()).unwrap());

    // First request
    let _ = shiori
        .request("GET SHIORI/3.0\r\nID: OnBoot\r\n\r\n")
        .unwrap();

    // Verify request_count is 1
    let runtime = shiori.runtime().expect("runtime should exist");
    let lua = runtime.lua();
    let shiori_table: pasta_lua::mlua::Table = lua
        .globals()
        .get("SHIORI")
        .expect("SHIORI table should exist");
    let count1: i64 = shiori_table
        .get("request_count")
        .expect("SHIORI.request_count should exist");
    assert_eq!(count1, 1, "request_count should be 1 after first request");

    // Second request
    let _ = shiori
        .request("GET SHIORI/3.0\r\nID: OnSecondChange\r\n\r\n")
        .unwrap();

    // Verify request_count is 2
    let count2: i64 = shiori_table
        .get("request_count")
        .expect("SHIORI.request_count should exist");
    assert_eq!(count2, 2, "request_count should be 2 after second request");
}

// ========================================================================
// Task 3.3: Pasta DSLシーン呼び出しテスト
// ========================================================================

/// Test that SHIORI.request calls Pasta DSL scene via @pasta_search.
/// Verifies: Response contains scene output "ライフサイクルテスト成功！"
#[test]
fn test_shiori_request_calls_pasta_scene() {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let mut shiori = PastaShiori::default();

    // Load
    assert!(shiori.load(1, temp.path().as_os_str()).unwrap());

    // Request - should trigger scene call
    let response = shiori
        .request("GET SHIORI/3.0\r\nID: OnBoot\r\n\r\n")
        .unwrap();

    // Verify response contains scene output
    assert!(
        response.contains("ライフサイクルテスト成功"),
        "Response should contain scene output 'ライフサイクルテスト成功', got: {}",
        response
    );

    // Verify response is valid SHIORI format
    assert!(
        response.contains("SHIORI/3.0 200 OK"),
        "Response should be 200 OK, got: {}",
        response
    );
}

// ========================================================================
// Task 3.4: SHIORI.unloadファイルマーカーテスト
// ========================================================================

/// Test that SHIORI.unload creates a marker file on drop.
/// Verifies: unload_called.marker file exists after PastaShiori is dropped.
#[test]
fn test_shiori_unload_creates_marker() {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let marker_path = temp.path().join("unload_called.marker");

    // Ensure marker doesn't exist before test
    if marker_path.exists() {
        std::fs::remove_file(&marker_path).unwrap();
    }

    {
        let mut shiori = PastaShiori::default();
        assert!(shiori.load(1, temp.path().as_os_str()).unwrap());

        // Verify marker doesn't exist yet
        assert!(
            !marker_path.exists(),
            "Marker should not exist before unload"
        );

        // shiori will be dropped here
    }

    // After drop, the marker file should exist
    assert!(
        marker_path.exists(),
        "SHIORI.unload should have created the marker file on drop"
    );
}

// ========================================================================
// Task 3.5: 統合E2Eライフサイクルテスト
// ========================================================================

/// E2E test verifying entire SHIORI lifecycle with Lua execution.
/// Combines all previous tests into a single comprehensive verification.
#[test]
fn test_shiori_lifecycle_lua_execution_verified() {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let marker_path = temp.path().join("unload_called.marker");

    // Ensure marker doesn't exist before test
    if marker_path.exists() {
        std::fs::remove_file(&marker_path).unwrap();
    }

    {
        let mut shiori = PastaShiori::default();

        // ================================================================
        // Phase 1: SHIORI.load - verify Lua globals are set
        // ================================================================
        let load_result = shiori.load(999, temp.path().as_os_str());
        assert!(load_result.is_ok(), "load() should succeed");
        assert!(load_result.unwrap(), "load() should return true");

        // Verify loaded_hinst and load_dir via Lua globals
        let runtime = shiori.runtime().expect("runtime should exist after load");
        let lua = runtime.lua();
        let globals = lua.globals();

        let shiori_table: pasta_lua::mlua::Table =
            globals.get("SHIORI").expect("SHIORI table should exist");
        let loaded_hinst: i64 = shiori_table
            .get("loaded_hinst")
            .expect("SHIORI.loaded_hinst should be set");
        assert_eq!(loaded_hinst, 999, "loaded_hinst should be 999");

        // ================================================================
        // Phase 2: SHIORI.request - verify scene execution and counter
        // ================================================================
        let response = shiori
            .request("GET SHIORI/3.0\r\nID: OnBoot\r\n\r\n")
            .unwrap();

        // Verify scene output is in response
        assert!(
            response.contains("ライフサイクルテスト成功"),
            "Response should contain Pasta scene output"
        );

        // Verify request_count was incremented
        let request_count: i64 = shiori_table
            .get("request_count")
            .expect("SHIORI.request_count should exist");
        assert_eq!(request_count, 1, "request_count should be 1");

        // Make another request to verify counter increments
        let _ = shiori
            .request("GET SHIORI/3.0\r\nID: OnSecondChange\r\n\r\n")
            .unwrap();
        let request_count_2: i64 = shiori_table
            .get("request_count")
            .expect("SHIORI.request_count should exist");
        assert_eq!(
            request_count_2, 2,
            "request_count should be 2 after second request"
        );

        // ================================================================
        // Phase 3: SHIORI.unload - marker file will be created on drop
        // ================================================================
        // Verify marker doesn't exist yet
        assert!(
            !marker_path.exists(),
            "Marker should not exist before unload"
        );

        // shiori dropped here
    }

    // Verify SHIORI.unload created the marker file
    assert!(
        marker_path.exists(),
        "SHIORI.unload should have created unload_called.marker"
    );
}
