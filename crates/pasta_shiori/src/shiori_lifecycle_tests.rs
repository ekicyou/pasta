use super::*;
use std::path::PathBuf;
use tempfile::TempDir;
use super::shiori_request_tests::copy_fixture_to_temp;

// ========================================================================
// Task 11.3: 複数回load()テスト
// ========================================================================

#[test]
fn test_multiple_load_releases_previous_runtime() {
    let temp1 = copy_fixture_to_temp("minimal");
    let temp2 = copy_fixture_to_temp("with_config");

    let mut shiori = PastaShiori::default();

    // First load
    let result1 = shiori.load(0, temp1.path().as_os_str());
    assert!(result1.is_ok());
    assert!(result1.unwrap());
    assert!(shiori.runtime.is_some());

    let first_load_dir = shiori.load_dir.clone();
    assert!(first_load_dir.is_some());

    // Second load - should release previous runtime
    let result2 = shiori.load(0, temp2.path().as_os_str());
    assert!(result2.is_ok());
    assert!(result2.unwrap());
    assert!(shiori.runtime.is_some());

    // load_dir should be updated to new path
    let second_load_dir = shiori.load_dir.clone();
    assert!(second_load_dir.is_some());
    assert_ne!(first_load_dir.unwrap(), second_load_dir.unwrap());
}

#[test]
fn test_reload_same_directory() {
    let temp = copy_fixture_to_temp("minimal");

    let mut shiori = PastaShiori::default();

    // First load
    let result1 = shiori.load(0, temp.path().as_os_str());
    assert!(result1.unwrap());

    // Reload same directory - should work without error
    let result2 = shiori.load(0, temp.path().as_os_str());
    assert!(result2.unwrap());
    assert!(shiori.runtime.is_some());
}

// ========================================================================
// Task 12: E2Eテスト - SHIORI load → request → unload サイクル
// ========================================================================

#[test]
fn test_full_shiori_lifecycle() {
    let temp = copy_fixture_to_temp("minimal");

    let mut shiori = PastaShiori::default();

    // Phase 1: load()
    let load_result = shiori.load(42, temp.path().as_os_str());
    assert!(load_result.is_ok());
    assert!(load_result.unwrap());
    assert!(shiori.runtime.is_some());
    assert_eq!(shiori.hinst, 42);

    // Phase 2: request() - entry.lua should provide SHIORI.request
    // Use valid SHIORI request format (GET method required for parsing)
    let valid_request = "GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n";
    let request_result = shiori.request(valid_request);
    assert!(request_result.is_ok());
    let response = request_result.unwrap();
    // Should return 204 No Content from entry.lua (minimal fixture has no SHIORI.request)
    assert!(response.contains("SHIORI/3.0 204 No Content"));
    assert!(response.contains("Charset: UTF-8"));
    assert!(response.contains("Sender: Pasta"));

    // Phase 3: unload via drop
    drop(shiori);
    // If we get here without panic, cleanup was successful
}

#[test]
fn test_request_before_load_returns_error() {
    let mut shiori = PastaShiori::default();

    // Request before load should fail
    let result = shiori.request("test");
    assert!(result.is_err());
    match result.unwrap_err() {
        MyError::NotInitialized => {}
        _ => panic!("Expected NotInitialized error"),
    }
}

#[test]
fn test_load_nonexistent_directory_returns_false() {
    let mut shiori = PastaShiori::default();
    let nonexistent = PathBuf::from("/definitely/nonexistent/path");

    let result = shiori.load(0, nonexistent.as_os_str());
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false, not error
    assert!(shiori.runtime.is_none());
}

#[test]
fn test_request_after_load_failure_returns_load_error() {
    let temp = TempDir::new().unwrap();
    // Create an empty directory (no pasta.toml) → PastaLoader::load will fail
    let mut shiori = PastaShiori::default();

    let loaded = shiori.load(0, temp.path().as_os_str()).unwrap();
    assert!(!loaded);

    // Request should return Load error with the failure message
    let err = shiori.request("test").unwrap_err();
    match err {
        MyError::Load(msg) => {
            assert!(!msg.is_empty(), "Load error message should not be empty");
        }
        other => panic!("Expected MyError::Load, got {:?}", other),
    }
}

// ========================================================================
// Task 13: 複数インスタンス同時ロードテスト
// ========================================================================

#[test]
fn test_multiple_instances_independent() {
    let temp1 = copy_fixture_to_temp("minimal");
    let temp2 = copy_fixture_to_temp("with_config");

    let mut shiori1 = PastaShiori::default();
    let mut shiori2 = PastaShiori::default();

    // Load both instances
    assert!(shiori1.load(1, temp1.path().as_os_str()).unwrap());
    assert!(shiori2.load(2, temp2.path().as_os_str()).unwrap());

    // Both should have independent runtimes
    assert!(shiori1.runtime.is_some());
    assert!(shiori2.runtime.is_some());

    // Both should respond to requests with 204 No Content
    // Use valid SHIORI request format for parsing
    let valid_request = "GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n";
    let response1 = shiori1.request(valid_request).unwrap();
    let response2 = shiori2.request(valid_request).unwrap();

    assert!(response1.contains("SHIORI/3.0 204 No Content"));
    assert!(response2.contains("SHIORI/3.0 204 No Content"));

    // Different hinst values
    assert_eq!(shiori1.hinst, 1);
    assert_eq!(shiori2.hinst, 2);

    // Different load_dirs
    assert_ne!(shiori1.load_dir, shiori2.load_dir);
}

#[test]
fn test_multiple_instances_share_global_registry() {
    let temp1 = copy_fixture_to_temp("minimal");
    let temp2 = copy_fixture_to_temp("with_config");

    let mut shiori1 = PastaShiori::default();
    let mut shiori2 = PastaShiori::default();

    // Load both instances
    shiori1.load(1, temp1.path().as_os_str()).unwrap();
    shiori2.load(2, temp2.path().as_os_str()).unwrap();

    // GlobalLoggerRegistry should have both loggers registered
    // (we can't directly test this without exposing internals,
    // but the fact that both load successfully indicates it works)

    // Cleanup - drop should unregister from GlobalLoggerRegistry
    drop(shiori1);
    drop(shiori2);
}

// ========================================================================
// Task 7.1: PastaShiori::load テスト - SHIORI 関数フラグ検証
// ========================================================================

#[test]
fn test_load_sets_shiori_flags_when_entry_lua_exists() {
    let temp = copy_fixture_to_temp("minimal");

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // entry.lua is present in pasta_scripts/, so cached functions should be Some
    assert!(shiori.load_fn.is_some(), "load_fn should be cached");
    assert!(shiori.request_fn.is_some(), "request_fn should be cached");
}

#[test]
fn test_load_flags_false_without_entry_lua() {
    let temp = copy_fixture_to_temp("minimal");

    // The framework entry.lua is always provided by self-deploy. To simulate
    // missing SHIORI functions, write a user-override entry.lua (in scripts/,
    // which has higher search priority than the self-deployed pasta_scripts/)
    // that defines NO SHIORI functions, suppressing the framework defaults.
    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path,
        "-- intentionally defines no SHIORI functions\n",
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // override defines no SHIORI functions, so cached functions should be None
    assert!(
        shiori.load_fn.is_none(),
        "load_fn should be None without entry.lua"
    );
    assert!(
        shiori.request_fn.is_none(),
        "request_fn should be None without entry.lua"
    );
}

// ========================================================================
// Task 7.2: PastaShiori::request テスト
// ========================================================================

#[test]
fn test_request_returns_204_from_lua() {
    let temp = copy_fixture_to_temp("minimal");

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // Use valid SHIORI request format for parsing
    let valid_request = "GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n";
    let response = shiori.request(valid_request).unwrap();

    // Verify SHIORI/3.0 response format
    assert!(response.starts_with("SHIORI/3.0 204 No Content\r\n"));
    assert!(response.contains("Charset: UTF-8\r\n"));
    assert!(response.contains("Sender: Pasta\r\n"));
    assert!(response.ends_with("\r\n\r\n"));
}

#[test]
fn test_request_returns_default_204_without_main_lua() {
    let temp = copy_fixture_to_temp("minimal");

    // Suppress the framework SHIORI functions via a no-function user override
    // (scripts/ has higher search priority than the self-deployed pasta_scripts/).
    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path,
        "-- intentionally defines no SHIORI functions\n",
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // Should still get 204 response (default fallback)
    let response = shiori.request("test").unwrap();
    assert!(response.contains("SHIORI/3.0 204 No Content"));
    assert!(response.contains("Charset: UTF-8"));
    assert!(response.contains("Sender: Pasta"));
}

#[test]
fn test_request_not_initialized_error() {
    let mut shiori = PastaShiori::default();

    // Request before load should return NotInitialized error
    let result = shiori.request("test");
    assert!(result.is_err());
    match result.unwrap_err() {
        MyError::NotInitialized => {}
        e => panic!("Expected NotInitialized, got {:?}", e),
    }
}

// ========================================================================
// Task 7.1: unload 呼び出しの検証テスト
// ========================================================================

#[test]
fn test_unload_called_on_drop() {
    let temp = copy_fixture_to_temp("minimal");

    // Create a user-override Lua script (in scripts/, higher search priority
    // than the self-deployed pasta_scripts/) that defines SHIORI.unload.
    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path,
        r#"
SHIORI = {}

-- Track if unload was called via a file marker
local unload_marker_path = nil

function SHIORI.load(hinst, load_dir)
    unload_marker_path = load_dir .. "/unload_called.marker"
    return true
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end

function SHIORI.unload()
    -- Write a marker file to indicate unload was called
    if unload_marker_path then
        local f = io.open(unload_marker_path, "w")
        if f then
            f:write("unloaded")
            f:close()
        end
    end
end
"#,
    )
    .unwrap();

    let marker_path = temp.path().join("unload_called.marker");

    // Ensure marker doesn't exist before test
    if marker_path.exists() {
        std::fs::remove_file(&marker_path).unwrap();
    }

    {
        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // Verify unload_fn is cached
        assert!(shiori.unload_fn.is_some(), "unload_fn should be cached");

        // shiori will be dropped here
    }

    // After drop, the marker file should exist
    assert!(
        marker_path.exists(),
        "SHIORI.unload should have created the marker file on drop"
    );
}

// ========================================================================
// Task 7.2: unload エラー耐性テスト
// ========================================================================

#[test]
fn test_unload_error_does_not_panic() {
    let temp = copy_fixture_to_temp("minimal");

    // Create a user-override Lua script (in scripts/, higher search priority
    // than the self-deployed pasta_scripts/) whose unload always errors.
    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path,
        r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return true
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end

function SHIORI.unload()
    error("Intentional unload error for testing")
end
"#,
    )
    .unwrap();

    {
        let mut shiori = PastaShiori::default();
        assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

        // Verify unload_fn is cached
        assert!(shiori.unload_fn.is_some(), "unload_fn should be cached");

        // shiori will be dropped here - should NOT panic even with error in unload
    }

    // If we reach here, the test passed (no panic occurred)
}

// ========================================================================
// Task 7.3: reload 時のキャッシュクリアテスト
// ========================================================================

#[test]
fn test_cached_functions_cleared_on_reload() {
    let temp1 = copy_fixture_to_temp("minimal");
    let temp2 = copy_fixture_to_temp("minimal");

    // Override temp2's entry.lua (in scripts/, higher search priority than the
    // self-deployed pasta_scripts/) to remove SHIORI.load but keep request.
    let entry_lua_path2 = temp2.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path2.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path2,
        r#"
SHIORI = {}

-- No SHIORI.load function defined

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();

    // First load - should have both load and request functions
    assert!(shiori.load(0, temp1.path().as_os_str()).unwrap());
    assert!(
        shiori.load_fn.is_some(),
        "First load: load_fn should be cached"
    );
    assert!(
        shiori.request_fn.is_some(),
        "First load: request_fn should be cached"
    );

    // Second load (reload) - should only have request function
    assert!(shiori.load(0, temp2.path().as_os_str()).unwrap());
    assert!(
        shiori.load_fn.is_none(),
        "After reload: load_fn should be None (not defined in temp2)"
    );
    assert!(
        shiori.request_fn.is_some(),
        "After reload: request_fn should still be cached"
    );
}

// ========================================================================
// Task 7.4: 複数インスタンス独立性テスト
// ========================================================================

#[test]
fn test_multiple_instances_independent_caches() {
    let temp1 = copy_fixture_to_temp("minimal");
    let temp2 = copy_fixture_to_temp("minimal");

    // Override temp1's entry.lua (in scripts/, higher search priority than the
    // self-deployed pasta_scripts/) to define all three SHIORI functions.
    let entry_lua_path1 = temp1.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path1.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path1,
        r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return true
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end

function SHIORI.unload()
    -- Instance 1 unload
end
"#,
    )
    .unwrap();

    // Override temp2's entry.lua (in scripts/, higher search priority than the
    // self-deployed pasta_scripts/) to only define request (no load/unload).
    let entry_lua_path2 = temp2.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path2.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path2,
        r#"
SHIORI = {}

-- No SHIORI.load or unload

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end
"#,
    )
    .unwrap();

    let mut shiori1 = PastaShiori::default();
    let mut shiori2 = PastaShiori::default();

    // Load both instances
    assert!(shiori1.load(1, temp1.path().as_os_str()).unwrap());
    assert!(shiori2.load(2, temp2.path().as_os_str()).unwrap());

    // Instance 1 should have all functions cached
    assert!(shiori1.load_fn.is_some(), "shiori1.load_fn should be Some");
    assert!(
        shiori1.request_fn.is_some(),
        "shiori1.request_fn should be Some"
    );
    assert!(
        shiori1.unload_fn.is_some(),
        "shiori1.unload_fn should be Some"
    );

    // Instance 2 should only have request_fn cached
    assert!(shiori2.load_fn.is_none(), "shiori2.load_fn should be None");
    assert!(
        shiori2.request_fn.is_some(),
        "shiori2.request_fn should be Some"
    );
    assert!(
        shiori2.unload_fn.is_none(),
        "shiori2.unload_fn should be None"
    );

    // Modifying one instance's cache should not affect the other
    // (This is implicitly verified by the above assertions)
}
