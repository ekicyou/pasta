use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Neutralize ambient DAP debug env vars before any test thread starts.
///
/// The developer session may export `PASTA_DEBUG=1` / `PASTA_DEBUG_PORT=9276`.
/// Without this guard, every `PastaLoader::load` driven through SHIORI here would
/// enable the DAP listener and bind that single fixed port; the multiple loads in
/// this test binary then collide with `AddrInUse` and the tests fail. Tests must
/// behave identically regardless of the session environment, so we clear these.
///
/// Running inside a `#[ctor]` (executed before `main`, while the process is still
/// single-threaded) makes the `remove_var` calls race-free under the Rust 2024
/// edition where `std::env::remove_var` is `unsafe`.
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// Copy fixture to a temporary directory for testing.
pub(super) fn copy_fixture_to_temp(fixture_name: &str) -> TempDir {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("pasta_lua/tests/fixtures/loader")
        .join(fixture_name);
    let temp = TempDir::new().unwrap();
    copy_dir_recursive(&src, temp.path()).unwrap();

    // Copy pasta_scripts directory (standard runtime)
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("pasta_lua");
    let scripts_src = crate_root.join("pasta_scripts");
    let scripts_dst = temp.path().join("pasta_scripts");
    if scripts_src.exists() {
        std::fs::create_dir_all(&scripts_dst).unwrap();
        copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
    }

    // Copy scriptlibs directory
    let scriptlibs_src = crate_root.join("scriptlibs");
    let scriptlibs_dst = temp.path().join("scriptlibs");
    if scriptlibs_src.exists() {
        std::fs::create_dir_all(&scriptlibs_dst).unwrap();
        copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
    }

    temp
}

pub(super) fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            // Skip profile directories
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

// ========================================================================
// Task 1.1: 400 Bad Requestレスポンス生成機能テスト
// ========================================================================

#[test]
fn test_400_response_via_my_error() {
    let err = MyError::InvalidPastaTime {
        value: "bad-value".to_string(),
        reason: "parse failed".to_string(),
    };
    let response = err.to_shiori_400_response();

    // SHIORI/3.0プロトコル準拠を検証
    assert!(response.starts_with("SHIORI/3.0 400 Bad Request\r\n"));
    assert!(response.contains("Charset: UTF-8\r\n"));
    assert!(response.contains("X-ERROR-REASON:"));
    assert!(!response.contains("Sender:"));
    assert!(response.ends_with("\r\n\r\n"));
}

// ========================================================================
// Task 4.1: パース成功・失敗パステスト
// ========================================================================

#[test]
fn test_request_with_valid_shiori_request() {
    // 有効なSHIORIリクエスト形式でのテスト
    let temp = copy_shiori_lifecycle_fixture();
    let mut shiori = PastaShiori::default();

    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // 有効なSHIORI/3.0リクエスト
    let valid_request = "GET SHIORI/3.0\r\n\
        Charset: UTF-8\r\n\
        ID: OnBoot\r\n\
        Reference0: first\r\n\
        \r\n";

    let result = shiori.request(valid_request);
    assert!(result.is_ok());
    let response = result.unwrap();
    // パースが成功してLuaが呼ばれた証拠として、400 Bad Request以外を期待
    // (シーンが見つからない場合は500、見つかった場合は200が返る)
    assert!(
        !response.contains("SHIORI/3.0 400 Bad Request"),
        "Parse should have succeeded, but got 400 Bad Request: {}",
        response
    );
    // 有効なSHIORIレスポンス形式であることを確認
    assert!(
        response.starts_with("SHIORI/3.0"),
        "Expected valid SHIORI response format, got: {}",
        response
    );
}

#[test]
fn test_request_with_invalid_shiori_request_returns_400() {
    // 無効なSHIORIリクエスト形式でのテスト
    let temp = copy_shiori_lifecycle_fixture();
    let mut shiori = PastaShiori::default();

    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // 完全に無効なリクエスト（パース失敗を引き起こす）
    let invalid_request = "THIS IS NOT A VALID SHIORI REQUEST";

    let result = shiori.request(invalid_request);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(
        response.contains("SHIORI/3.0 400 Bad Request"),
        "Expected 400 Bad Request, got: {}",
        response
    );
}

/// Test that parsed request table fields are correctly passed to Lua.
/// This verifies Lua can actually read method, version, id, reference, dic, etc.
#[test]
fn test_request_parsed_table_fields_accessible_in_lua() {
    let temp = copy_shiori_lifecycle_fixture();

    // Override entry.lua to echo back req table fields for verification
    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::write(
        &entry_lua_path,
        r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return true
end

--- Echo back req table fields for verification
function SHIORI.request(req)
    -- Verify req is a table
    if type(req) ~= "table" then
        return "SHIORI/3.0 500 Internal Server Error\r\nValue: req is not a table\r\n\r\n"
    end

    -- Extract all expected fields from req table
    local method = req.method or "NIL"
    local version = req.version or "NIL"
    local id = req.id or "NIL"
    local charset = req.charset or "NIL"
    local sender = req.sender or "NIL"

    -- Extract reference array
    local ref0 = "NIL"
    local ref1 = "NIL"
    if req.reference then
        ref0 = req.reference[0] or "NIL"
        ref1 = req.reference[1] or "NIL"
    end

    -- Extract dic table entry
    local dic_id = "NIL"
    if req.dic then
        dic_id = req.dic["ID"] or "NIL"
    end

    -- Return all fields in response Value header for verification
    local fields = string.format(
        "method=%s,version=%s,id=%s,charset=%s,sender=%s,ref0=%s,ref1=%s,dic_id=%s",
        tostring(method), tostring(version), tostring(id),
        tostring(charset), tostring(sender),
        tostring(ref0), tostring(ref1), tostring(dic_id)
    )

    return "SHIORI/3.0 200 OK\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Value: " .. fields .. "\r\n" ..
        "\r\n"
end

function SHIORI.unload()
end
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    // Send a request with all fields populated
    let request = "GET SHIORI/3.0\r\n\
        Charset: UTF-8\r\n\
        Sender: SSP\r\n\
        ID: OnBoot\r\n\
        Reference0: ref_value_0\r\n\
        Reference1: ref_value_1\r\n\
        \r\n";

    let response = shiori.request(request).unwrap();

    // Verify response is 200 OK (not 400 Bad Request)
    assert!(
        response.contains("SHIORI/3.0 200 OK"),
        "Expected 200 OK, got: {}",
        response
    );

    // Verify each field was correctly parsed and accessible in Lua
    assert!(
        response.contains("method=get"),
        "Expected method=get in response: {}",
        response
    );
    assert!(
        response.contains("version=30"),
        "Expected version=30 in response: {}",
        response
    );
    assert!(
        response.contains("id=OnBoot"),
        "Expected id=OnBoot in response: {}",
        response
    );
    assert!(
        response.contains("charset=UTF-8"),
        "Expected charset=UTF-8 in response: {}",
        response
    );
    assert!(
        response.contains("sender=SSP"),
        "Expected sender=SSP in response: {}",
        response
    );
    assert!(
        response.contains("ref0=ref_value_0"),
        "Expected ref0=ref_value_0 in response: {}",
        response
    );
    assert!(
        response.contains("ref1=ref_value_1"),
        "Expected ref1=ref_value_1 in response: {}",
        response
    );
    assert!(
        response.contains("dic_id=OnBoot"),
        "Expected dic_id=OnBoot in response: {}",
        response
    );
}

// ========================================================================
// G1 (3.35): call_lua_load failure branches (previously unreached)
// ========================================================================

/// SHIORI.load returning false must surface as load() == Ok(false).
/// Characterization: the runtime stays initialized (load() returns early
/// AFTER self.runtime was set), so subsequent requests still reach Lua.
#[test]
fn test_load_returns_false_when_lua_load_returns_false() {
    let temp = copy_fixture_to_temp("minimal");

    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path,
        r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    return false
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    let result = shiori.load(0, temp.path().as_os_str());
    assert!(result.is_ok(), "Lua-level load failure is not a Rust error");
    assert!(
        !result.unwrap(),
        "SHIORI.load false must propagate as false"
    );

    // Characterize current semantics: runtime kept despite reported failure.
    assert!(shiori.runtime.is_some());
    let response = shiori
        .request("GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n")
        .unwrap();
    assert!(response.contains("SHIORI/3.0 204 No Content"));
}

/// SHIORI.load raising a Lua error must also surface as Ok(false).
#[test]
fn test_load_returns_false_when_lua_load_errors() {
    let temp = copy_fixture_to_temp("minimal");

    let entry_lua_path = temp.path().join("scripts/pasta/shiori/entry.lua");
    std::fs::create_dir_all(entry_lua_path.parent().unwrap()).unwrap();
    std::fs::write(
        &entry_lua_path,
        r#"
SHIORI = {}

function SHIORI.load(hinst, load_dir)
    error("intentional load failure")
end

function SHIORI.request(request)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n"
end
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    let result = shiori.load(0, temp.path().as_os_str());
    assert!(result.is_ok());
    assert!(
        !result.unwrap(),
        "SHIORI.load error must propagate as false"
    );
}

// ========================================================================
// G1 (3.35): call_lua_request error branches (previously unreached)
// ========================================================================

/// SHIORI.request raising a Lua error must surface as Err(MyError::Script)
/// — this is the path windows.rs turns into a SHIORI 500 response.
#[test]
fn test_request_lua_error_returns_script_error() {
    let temp = copy_fixture_to_temp("minimal");

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
    error("intentional request failure")
end
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    let err = shiori
        .request("GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n")
        .unwrap_err();
    match err {
        MyError::Script { ref message } => {
            assert!(
                message.contains("intentional request failure"),
                "Script message should carry the Lua error: {message}"
            );
        }
        other => panic!("Expected MyError::Script, got {:?}", other),
    }
}

/// SHIORI.request returning a non-string (nil) fails the mlua String
/// conversion and must also surface as Err(MyError::Script).
#[test]
fn test_request_non_string_return_is_script_error() {
    let temp = copy_fixture_to_temp("minimal");

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
    return nil
end
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());

    let err = shiori
        .request("GET SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n")
        .unwrap_err();
    assert!(
        matches!(err, MyError::Script { .. }),
        "nil return must become a Script error, got {:?}",
        err
    );
}

// ========================================================================
// G1 (3.35): runtime() accessor None branch
// ========================================================================

#[test]
fn test_runtime_accessor_none_before_load() {
    let shiori = PastaShiori::default();
    assert!(shiori.runtime().is_none());
}

/// Copy shiori_lifecycle fixture to a temporary directory.
fn copy_shiori_lifecycle_fixture() -> TempDir {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shiori_lifecycle");
    let temp = TempDir::new().unwrap();
    copy_dir_recursive(&src, temp.path()).unwrap();

    // Copy pasta_scripts directory from pasta_lua (standard runtime)
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("pasta_lua");
    let scripts_src = crate_root.join("pasta_scripts");
    let scripts_dst = temp.path().join("pasta_scripts");
    if scripts_src.exists() {
        std::fs::create_dir_all(&scripts_dst).unwrap();
        copy_dir_recursive(&scripts_src, &scripts_dst).unwrap();
    }

    // Copy scriptlibs directory
    let scriptlibs_src = crate_root.join("scriptlibs");
    let scriptlibs_dst = temp.path().join("scriptlibs");
    if scriptlibs_src.exists() {
        std::fs::create_dir_all(&scriptlibs_dst).unwrap();
        copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).unwrap();
    }

    // Copy fixture's scripts/ as user override (searched before pasta_scripts/)
    let fixture_scripts = src.join("scripts");
    let user_scripts_dst = temp.path().join("scripts");
    if fixture_scripts.exists() {
        std::fs::create_dir_all(&user_scripts_dst).unwrap();
        copy_dir_recursive(&fixture_scripts, &user_scripts_dst).unwrap();
    }

    temp
}
