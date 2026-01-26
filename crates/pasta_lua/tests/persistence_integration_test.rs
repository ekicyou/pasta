//! Persistence Integration Tests
//!
//! Tests for the store-save-persistence feature:
//! - Runtime persistence load/save roundtrip
//! - Drop-time auto-save
//! - pasta.toml configuration integration

use pasta_lua::loader::PersistenceConfig;
use pasta_lua::runtime::persistence;
use tempfile::TempDir;

mod common;

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

    // ctx.save should reference pasta.save
    lua.load(
        r#"
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local ctx = CTX.new()
        
        -- They should be the same reference
        assert(ctx.save == SAVE, "ctx.save should reference pasta.save")
        
        -- Changes in one should reflect in the other
        ctx.save.test_key = "test_value"
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
