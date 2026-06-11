//! Persistence Integration Tests
//!
//! Tests for the store-save-persistence feature:
//! - Runtime persistence load/save roundtrip
//! - Drop-time auto-save
//! - pasta.toml configuration integration

use pasta_lua::loader::PersistenceConfig;
use pasta_lua::runtime::persistence;
use tempfile::TempDir;

use crate::common;

/// Test JSON format load/save roundtrip via persistence module
#[test]
fn test_persistence_json_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.json");

    // Save data
    let data = serde_json::json!({
        "player_name": "Alice",
        "score": 1000,
        "flags": {
            "tutorial_complete": true
        }
    });
    persistence::save_to_file(&data, &file_path, false).unwrap();

    // Load and verify
    let loaded = persistence::load_from_file(&file_path).unwrap();
    assert_eq!(loaded["player_name"], "Alice");
    assert_eq!(loaded["score"], 1000);
    assert_eq!(loaded["flags"]["tutorial_complete"], true);
}

/// Test obfuscated (gzip) format load/save roundtrip
#[test]
fn test_persistence_obfuscated_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.dat");

    // Save with obfuscation
    let data = serde_json::json!({
        "secret": "hidden_value",
        "inventory": ["sword", "shield", "potion"]
    });
    persistence::save_to_file(&data, &file_path, true).unwrap();

    // Verify file is compressed (starts with gzip magic header)
    let raw = std::fs::read(&file_path).unwrap();
    assert!(raw.len() >= 2);
    assert_eq!(raw[0], 0x1f);
    assert_eq!(raw[1], 0x8b);

    // Load and verify
    let loaded = persistence::load_from_file(&file_path).unwrap();
    assert_eq!(loaded["secret"], "hidden_value");
    assert_eq!(loaded["inventory"][0], "sword");
}

/// Test PersistenceConfig effective_file_path conversion
#[test]
fn test_persistence_config_effective_path() {
    // Default (non-obfuscated)
    let config = PersistenceConfig::default();
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.json");

    // Obfuscated with .json extension -> .dat
    let config = PersistenceConfig {
        obfuscate: true,
        file_path: "profile/pasta/save/save.json".to_string(),
        debug_mode: false,
    };
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");

    // Obfuscated with .dat extension -> unchanged
    let config = PersistenceConfig {
        obfuscate: true,
        file_path: "profile/pasta/save/save.dat".to_string(),
        debug_mode: false,
    };
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");
}

/// Test directory auto-creation on save
#[test]
fn test_persistence_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir
        .path()
        .join("nested")
        .join("deep")
        .join("save.json");

    let data = serde_json::json!({"test": true});
    persistence::save_to_file(&data, &file_path, false).unwrap();

    assert!(file_path.exists());
    let loaded = persistence::load_from_file(&file_path).unwrap();
    assert_eq!(loaded["test"], true);
}

/// Test Lua module registration and basic operations
#[test]
fn test_lua_persistence_module() {
    let lua = common::e2e_helpers::create_runtime_with_finalize().unwrap();

    // Test load returns empty table when file doesn't exist
    let result: mlua::Table = lua
        .load(r#"return require("@pasta_persistence").load()"#)
        .eval()
        .unwrap();
    assert_eq!(result.len().unwrap(), 0);

    // Test save and load roundtrip
    lua.load(
        r#"
        local p = require("@pasta_persistence")
        local data = { name = "Test", count = 42 }
        local ok, err = p.save(data)
        assert(ok, err)
    "#,
    )
    .exec()
    .unwrap();

    let result: mlua::Table = lua
        .load(r#"return require("@pasta_persistence").load()"#)
        .eval()
        .unwrap();
    let name: String = result.get("name").unwrap();
    let count: i32 = result.get("count").unwrap();
    assert_eq!(name, "Test");
    assert_eq!(count, 42);
}

/// Test pasta.save module integration with ctx.save
#[test]
fn test_pasta_save_ctx_integration() {
    let lua = common::e2e_helpers::create_runtime_with_finalize().unwrap();

    // pasta.save should return a table
    let save_type: String = lua
        .load(r#"return type(require("pasta.save"))"#)
        .eval()
        .unwrap();
    assert_eq!(save_type, "table");

    // act.save should reference pasta.save
    lua.load(
        r#"
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local act = ACT.new({})
        
        -- They should be the same reference
        assert(act.save == SAVE, "act.save should reference pasta.save")
        
        -- Changes in one should reflect in the other
        act.save.test_key = "test_value"
        assert(SAVE.test_key == "test_value", "Changes should reflect")
    "#,
    )
    .exec()
    .unwrap();
}

/// Test STORE.save is deprecated (removed)
#[test]
fn test_store_save_deprecated() {
    let lua = common::e2e_helpers::create_runtime_with_finalize().unwrap();

    let save_value: mlua::Value = lua
        .load(r#"return require("pasta.store").save"#)
        .eval()
        .unwrap();
    assert!(save_value.is_nil(), "STORE.save should be nil (deprecated)");
}

// ============================================================================
// Lua モジュール境界のエラーパス（review-improvement-loop セル 3.16 / G1 追加）
//
// src/runtime/persistence.rs の #[cfg(test)] は load_from_file/save_to_file の
// Rust 関数レベルのエラーと、モジュール経由の正常系のみを検証している。
// ここでは load()/save() Lua 関数の劣化動作（壊れたファイル → 空テーブル、
// 保存失敗 → (nil, err)）とモジュール経由の gzip 往復を検証する。
// ============================================================================

/// 指定した file_path・obfuscate でモジュールを登録した (Lua, base_dir) を作る。
fn register_module_in_temp(
    temp_dir: &TempDir,
    file_path: &str,
    obfuscate: bool,
) -> (mlua::Lua, mlua::Table) {
    let config = PersistenceConfig {
        obfuscate,
        file_path: file_path.to_string(),
        debug_mode: false,
    };
    let lua = mlua::Lua::new();
    let module = persistence::register(&lua, &config, temp_dir.path()).unwrap();
    (lua, module)
}

/// 壊れた JSON ファイルでも load() は空テーブルを返す（エラーにしない）。
#[test]
fn test_persistence_load_corrupted_json_returns_empty_table() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("save.json"), "{ not valid json !!").unwrap();

    let (_lua, module) = register_module_in_temp(&temp_dir, "save.json", false);
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();

    assert_eq!(result.len().unwrap(), 0, "corrupted file should degrade to empty table");
}

/// オブジェクト以外の JSON（スカラー）でも load() は空テーブルを返す。
#[test]
fn test_persistence_load_non_object_json_returns_empty_table() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("save.json"), r#""just a string""#).unwrap();

    let (_lua, module) = register_module_in_temp(&temp_dir, "save.json", false);
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();

    assert_eq!(
        result.len().unwrap(),
        0,
        "non-object JSON should degrade to empty table"
    );
}

/// gzip マジック付きの壊れたデータでも load() は空テーブルを返す。
#[test]
fn test_persistence_load_corrupted_gzip_returns_empty_table() {
    let temp_dir = TempDir::new().unwrap();
    // gzip マジック (0x1f 0x8b) + 不正データ → GzDecoder がエラー
    std::fs::write(
        temp_dir.path().join("save.dat"),
        [0x1f, 0x8b, 0xff, 0xff, 0x00, 0x12, 0x34],
    )
    .unwrap();

    let (_lua, module) = register_module_in_temp(&temp_dir, "save.dat", true);
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();

    assert_eq!(
        result.len().unwrap(),
        0,
        "corrupted gzip should degrade to empty table"
    );
}

/// モジュール経由の obfuscate=true 往復: save() が gzip で書き、load() が復元する。
#[test]
fn test_persistence_module_obfuscated_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let (lua, module) = register_module_in_temp(&temp_dir, "save.dat", true);

    let data: mlua::Table = lua.create_table().unwrap();
    data.set("secret", "hidden").unwrap();
    data.set("count", 3).unwrap();

    let save_fn: mlua::Function = module.get("save").unwrap();
    let (ok, err): (Option<bool>, Option<String>) = save_fn.call(data).unwrap();
    assert_eq!(ok, Some(true), "save should succeed: {err:?}");

    // ファイルは gzip マジックで始まる（平文 JSON ではない）
    let raw = std::fs::read(temp_dir.path().join("save.dat")).unwrap();
    assert!(raw.len() >= 2);
    assert_eq!(&raw[0..2], &[0x1f, 0x8b], "file should be gzip compressed");

    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();
    let secret: String = result.get("secret").unwrap();
    let count: i64 = result.get("count").unwrap();
    assert_eq!(secret, "hidden");
    assert_eq!(count, 3);
}

/// 保存先パスが書き込み不能（同名ディレクトリが存在）の場合、
/// save() は (nil, エラーメッセージ) を返し、panic も Lua エラーもしない。
#[test]
fn test_persistence_save_failure_returns_nil_and_message() {
    let temp_dir = TempDir::new().unwrap();
    // save.json の位置にディレクトリを作り、rename を失敗させる
    std::fs::create_dir(temp_dir.path().join("save.json")).unwrap();

    let (lua, module) = register_module_in_temp(&temp_dir, "save.json", false);
    let data: mlua::Table = lua.create_table().unwrap();
    data.set("k", "v").unwrap();

    let save_fn: mlua::Function = module.get("save").unwrap();
    let (ok, err): (Option<bool>, Option<String>) = save_fn.call(data).unwrap();

    assert_eq!(ok, None, "save should report failure");
    let err = err.expect("error message should be present");
    assert!(
        err.contains("Failed to save"),
        "error message should describe the failure: {err}"
    );
}
