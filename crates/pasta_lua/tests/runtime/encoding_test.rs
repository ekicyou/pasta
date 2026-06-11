//! Encoding integration tests for pasta_lua.
//!
//! Tests the complete integration of encoding functionality including:
//! - package.path setting with ANSI encoding on Windows
//! - @enc module UTF-8/ANSI conversion
//! - Japanese path require support

use pasta_lua::context::TranspileContext;
use pasta_lua::loader::LoaderContext;
use pasta_lua::runtime::{PastaLuaRuntime, RuntimeConfig};

/// Create a test runtime with @enc module registered.
fn create_test_runtime() -> PastaLuaRuntime {
    let context = TranspileContext::new();
    let loader_context = LoaderContext::new(
        "/test/path",
        vec!["scripts".to_string()],
        toml::Table::new(),
    );
    PastaLuaRuntime::from_loader(context, loader_context, RuntimeConfig::new(), &[], None).unwrap()
}

// ============================================================================
// @enc Module E2E Tests
// ============================================================================

#[test]
fn test_enc_module_require() {
    let runtime = create_test_runtime();

    // Should be able to require @enc module
    let result: mlua::Table = runtime
        .exec(r#"return require "@enc""#)
        .unwrap()
        .as_table()
        .unwrap()
        .clone();

    // Check _VERSION exists
    let version: String = result.get("_VERSION").unwrap();
    assert_eq!(version, "0.1.0");

    // Check _DESCRIPTION exists
    let desc: String = result.get("_DESCRIPTION").unwrap();
    assert!(desc.contains("Encoding"));
}

#[test]
fn test_enc_to_ansi_ascii() {
    let runtime = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_ansi("hello world")
        if err then
            return nil, err
        end
        return result
    "#;

    let result: mlua::String = runtime.exec(script).unwrap().as_string().unwrap().clone();
    assert_eq!(result.as_bytes(), b"hello world");
}

#[test]
fn test_enc_to_utf8_ascii() {
    let runtime = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_utf8("hello world")
        if err then
            return nil, err
        end
        return result
    "#;

    let result: mlua::String = runtime.exec(script).unwrap().as_string().unwrap().clone();
    assert_eq!(result.as_bytes(), b"hello world");
}

#[test]
fn test_enc_type_error_handling() {
    let runtime = create_test_runtime();

    // Test with number instead of string
    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_ansi(12345)
        if err then
            return "error"
        end
        return "success"
    "#;

    let result = runtime.exec(script).unwrap();
    let status = result.as_string().unwrap().to_str().unwrap();
    assert_eq!(status, "error");
}

#[test]
fn test_enc_nil_handling() {
    let runtime = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_utf8(nil)
        if err then
            return "error"
        end
        return "success"
    "#;

    let result = runtime.exec(script).unwrap();
    let status = result.as_string().unwrap().to_str().unwrap();
    assert_eq!(status, "error");
}

/// to_ansi に不正な UTF-8 バイト列を渡すと (nil, "invalid UTF-8 ...") を返す
/// （review-improvement-loop セル 3.16 / G1 追加: 入力検証エラーパス）。
#[test]
fn test_enc_to_ansi_invalid_utf8_input_error() {
    let runtime = create_test_runtime();

    // "\255\254" は UTF-8 として不正なバイト列（Lua 文字列は任意バイトを保持できる）
    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_ansi("\255\254")
        if result ~= nil then
            return "unexpected_success"
        end
        return err
    "#;

    let result = runtime.exec(script).unwrap();
    let err = result.as_string().unwrap().to_str().unwrap().to_string();
    assert!(
        err.contains("invalid UTF-8"),
        "error should mention invalid UTF-8: {err}"
    );
}

/// ANSI コードページで表現できない文字（絵文字）は (nil, "ANSI conversion failed ...")
/// を返す（Windows のみ: Unix 実装は UTF-8 パススルー）。
#[cfg(windows)]
#[test]
fn test_enc_to_ansi_unmappable_char_error() {
    let runtime = create_test_runtime();

    // "\240\159\152\128" = U+1F600 (😀)。CP932/CP1252 等の ANSI には存在しない
    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_ansi("\240\159\152\128")
        if result ~= nil then
            return "unexpected_success"
        end
        return err
    "#;

    let result = runtime.exec(script).unwrap();
    let err = result.as_string().unwrap().to_str().unwrap().to_string();
    assert!(
        err.contains("ANSI conversion failed"),
        "error should mention ANSI conversion failure: {err}"
    );
}

/// ANSI として不正なバイト列を to_utf8 に渡すと (nil, "UTF-8 conversion failed ...")
/// を返す（Windows のみ: MB_ERR_INVALID_CHARS による検証）。
#[cfg(windows)]
#[test]
fn test_enc_to_utf8_invalid_ansi_bytes_error() {
    let runtime = create_test_runtime();

    // 0x81 単独は CP932 では後続バイト欠落、CP1252 では未定義 → どちらも不正
    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_utf8("\129")
        if result ~= nil then
            return "unexpected_success"
        end
        return err
    "#;

    let result = runtime.exec(script).unwrap();
    let err = result.as_string().unwrap().to_str().unwrap().to_string();
    assert!(
        err.contains("UTF-8 conversion failed"),
        "error should mention UTF-8 conversion failure: {err}"
    );
}

// ============================================================================
// Windows-specific tests (Japanese path support)
// ============================================================================

#[cfg(windows)]
#[test]
fn test_enc_roundtrip_japanese() {
    let runtime = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local original = "日本語パス"
        
        -- Convert to ANSI
        local ansi, err1 = enc.to_ansi(original)
        if not ansi then
            return "ansi_error"
        end
        
        -- Convert back to UTF-8
        local utf8, err2 = enc.to_utf8(ansi)
        if not utf8 then
            return "utf8_error"
        end
        
        -- Should match original
        if utf8 == original then
            return "success"
        else
            return "mismatch"
        end
    "#;

    let result = runtime.exec(script).unwrap();
    let status = result.as_string().unwrap().to_str().unwrap();
    assert_eq!(status, "success");
}

#[cfg(windows)]
#[test]
fn test_enc_ansi_bytes_differ_from_utf8() {
    let runtime = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local original = "日本語"
        local ansi, err = enc.to_ansi(original)
        if not ansi then
            return "error", 0, 0
        end
        
        -- ANSI encoding should be different from UTF-8
        if ansi ~= original then
            return "different"
        else
            return "same"
        end
    "#;

    let result = runtime.exec(script).unwrap();
    let status = result.as_string().unwrap().to_str().unwrap();
    assert_eq!(status, "different");
}

// ============================================================================
// Package path encoding tests
// ============================================================================

#[test]
fn test_package_path_set_correctly() {
    let context = TranspileContext::new();
    let loader_context = LoaderContext::new(
        "/test/base",
        vec!["scripts".to_string(), "lib".to_string()],
        toml::Table::new(),
    );

    let runtime =
        PastaLuaRuntime::from_loader(context, loader_context, RuntimeConfig::new(), &[], None)
            .unwrap();

    // Get package.path
    let script = r#"return package.path"#;
    let path: mlua::String = runtime.exec(script).unwrap().as_string().unwrap().clone();

    // Should contain expected patterns
    let path_str = path.to_str().unwrap();
    assert!(
        path_str.contains("scripts/?.lua"),
        "path should contain scripts/?.lua"
    );
    assert!(
        path_str.contains("lib/?.lua"),
        "path should contain lib/?.lua"
    );
    assert!(
        path_str.contains("/test/base"),
        "path should contain base dir"
    );
}

#[cfg(windows)]
#[test]
fn test_package_path_with_japanese_base() {
    let context = TranspileContext::new();
    let loader_context = LoaderContext::new(
        "C:\\ユーザー\\テスト",
        vec!["scripts".to_string()],
        toml::Table::new(),
    );

    let runtime =
        PastaLuaRuntime::from_loader(context, loader_context, RuntimeConfig::new(), &[], None)
            .unwrap();

    // Should not panic, and package.path should be set
    let script = r#"return package.path"#;
    let result = runtime.exec(script);
    assert!(result.is_ok(), "package.path should be set without error");

    let path: mlua::String = result.unwrap().as_string().unwrap().clone();
    // The path should not be empty
    assert!(!path.as_bytes().is_empty());
}

// ============================================================================
// Backward compatibility tests
// ============================================================================

#[test]
fn test_pasta_config_module_still_works() {
    let context = TranspileContext::new();

    let mut custom_fields = toml::Table::new();
    custom_fields.insert(
        "ghost_name".to_string(),
        toml::Value::String("TestGhost".to_string()),
    );

    let loader_context =
        LoaderContext::new("/test/path", vec!["scripts".to_string()], custom_fields);

    let runtime =
        PastaLuaRuntime::from_loader(context, loader_context, RuntimeConfig::new(), &[], None)
            .unwrap();

    // @pasta_config should still work
    let script = r#"
        local config = require "@pasta_config"
        return config.ghost_name
    "#;

    let result: String = runtime
        .exec(script)
        .unwrap()
        .as_string()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(result, "TestGhost");
}

#[test]
fn test_runtime_new_api_unchanged() {
    // PastaLuaRuntime::new should work with TranspileContext
    let context = TranspileContext::new();
    let runtime = PastaLuaRuntime::new(context);
    assert!(runtime.is_ok());
}

#[test]
fn test_runtime_with_config_api_unchanged() {
    // PastaLuaRuntime::with_config should work
    let context = TranspileContext::new();
    let runtime = PastaLuaRuntime::with_config(context, RuntimeConfig::minimal());
    assert!(runtime.is_ok());
}
