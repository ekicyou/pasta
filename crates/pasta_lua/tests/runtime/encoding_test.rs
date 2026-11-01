//! Encoding integration tests for pasta_lua.
//!
//! Tests the complete integration of encoding functionality including:
//! - package.path setting (UTF-8 無変換・要件 2.1)
//! - @enc module UTF-8/ANSI conversion
//! - Japanese path require support
//!
//! どのテストも一時ゴーストを本番の読み込み（`PastaLoader::load`）に通した VM で確かめる。

use crate::common::{copy_fixture_into, copy_fixture_to_temp};
use pasta_lua::context::TranspileContext;
use pasta_lua::loader::{PastaLoader, default_lua_search_paths};
use pasta_lua::runtime::{PastaLuaRuntime, RuntimeConfig};
use std::path::Path;
use tempfile::TempDir;

/// 最小のゴーストを読み込んだ VM を作る（`TempDir` はテストの間持っておく）。
fn create_test_runtime() -> (TempDir, PastaLuaRuntime) {
    let temp = copy_fixture_to_temp("minimal");
    let runtime = PastaLoader::load(temp.path()).unwrap();
    (temp, runtime)
}

// ============================================================================
// @enc Module E2E Tests
// ============================================================================

#[test]
fn test_enc_module_require() {
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_ansi("hello world")
        if err then
            return nil, err
        end
        return result
    "#;

    let result: mlua::LuaString = runtime.exec(script).unwrap().as_string().unwrap().clone();
    assert_eq!(result.as_bytes(), b"hello world");
}

#[test]
fn test_enc_to_utf8_ascii() {
    let (_temp, runtime) = create_test_runtime();

    let script = r#"
        local enc = require "@enc"
        local result, err = enc.to_utf8("hello world")
        if err then
            return nil, err
        end
        return result
    "#;

    let result: mlua::LuaString = runtime.exec(script).unwrap().as_string().unwrap().clone();
    assert_eq!(result.as_bytes(), b"hello world");
}

#[test]
fn test_enc_type_error_handling() {
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

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
    let (_temp, runtime) = create_test_runtime();

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

/// 既定の `lua_search_paths` の各項目を、ゴーストのフォルダの絶対パスで `/?.lua`・`/?/init.lua` に並べた値。
fn expected_package_path(base: &Path) -> String {
    let base = std::path::absolute(base).unwrap();
    default_lua_search_paths()
        .iter()
        .flat_map(|p| {
            let dir = base.join(p).to_string_lossy().replace('\\', "/");
            [format!("{dir}/?.lua"), format!("{dir}/?/init.lua")]
        })
        .collect::<Vec<_>>()
        .join(";")
}

#[test]
fn test_package_path_set_correctly() {
    let (temp, runtime) = create_test_runtime();

    let path: mlua::LuaString = runtime
        .exec("return package.path")
        .unwrap()
        .as_string()
        .unwrap()
        .clone();

    // 検索パス設定は検索パスの各項目から組んだ文字列と**バイト同一**である（要件 2.1 / 3.1 / 3.2）。
    assert_eq!(
        path.as_bytes().as_ref(),
        expected_package_path(temp.path()).as_bytes()
    );
    assert!(
        path.to_str().unwrap().contains("/scripts/?.lua"),
        "path should contain scripts/?.lua"
    );
}

#[test]
fn test_package_path_with_japanese_base() {
    let temp = TempDir::new().unwrap();
    let base = temp.path().join("ユーザー").join("テスト");
    copy_fixture_into("minimal", &base);
    let runtime = PastaLoader::load(&base).unwrap();

    let path: mlua::LuaString = runtime
        .exec("return package.path")
        .unwrap()
        .as_string()
        .unwrap()
        .clone();
    // 非 ANSI パスでも ANSI へ変換せず、UTF-8 のまま設定される（要件 2.1 / 3.8）。
    let expected = expected_package_path(&base);
    assert_eq!(path.as_bytes().as_ref(), expected.as_bytes());
    assert_eq!(
        path.to_str().unwrap(),
        expected,
        "package.path は UTF-8 として解釈できなければならない"
    );
}

// ============================================================================
// Backward compatibility tests
// ============================================================================

#[test]
fn test_pasta_config_module_still_works() {
    // pasta.toml に `ghost_name = "TestGhost"` を書いたゴースト
    let temp = copy_fixture_to_temp("with_custom_config");
    let runtime = PastaLoader::load(temp.path()).unwrap();

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
