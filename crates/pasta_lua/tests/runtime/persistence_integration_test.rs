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

/// Test TOML format load/save roundtrip via persistence module
#[test]
fn test_persistence_toml_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.toml");

    // Save data
    let data: toml::Table = toml::from_str(
        r#"
player_name = "Alice"
score = 1000
[flags]
tutorial_complete = true
"#,
    )
    .unwrap();
    persistence::save_to_file(&data, &file_path, false).unwrap();

    // Load and verify
    let loaded = persistence::load_from_file(&file_path).unwrap();
    assert_eq!(loaded["player_name"].as_str(), Some("Alice"));
    assert_eq!(loaded["score"].as_integer(), Some(1000));
    assert_eq!(loaded["flags"]["tutorial_complete"].as_bool(), Some(true));
}

/// Test obfuscated (gzip) format load/save roundtrip
#[test]
fn test_persistence_obfuscated_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.dat");

    // Save with obfuscation
    let data: toml::Table = toml::from_str(
        r#"
secret = "hidden_value"
inventory = ["sword", "shield", "potion"]
"#,
    )
    .unwrap();
    persistence::save_to_file(&data, &file_path, true).unwrap();

    // Verify file is compressed (starts with gzip magic header)
    let raw = std::fs::read(&file_path).unwrap();
    assert!(raw.len() >= 2);
    assert_eq!(raw[0], 0x1f);
    assert_eq!(raw[1], 0x8b);

    // Load and verify
    let loaded = persistence::load_from_file(&file_path).unwrap();
    assert_eq!(loaded["secret"].as_str(), Some("hidden_value"));
    assert_eq!(loaded["inventory"][0].as_str(), Some("sword"));
}

/// Test PersistenceConfig effective_file_path conversion (3.1〜3.4)
#[test]
fn test_persistence_config_effective_path() {
    // Default (non-obfuscated): save.toml
    let config = PersistenceConfig::default();
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.toml");

    // Obfuscated default: trailing .toml -> .dat
    let config = PersistenceConfig {
        obfuscate: true,
        ..PersistenceConfig::default()
    };
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");

    // Obfuscated with .json: not special, .dat is appended
    let config = PersistenceConfig {
        obfuscate: true,
        file_path: "profile/pasta/save/save.json".to_string(),
        debug_mode: false,
    };
    assert_eq!(
        config.effective_file_path(),
        "profile/pasta/save/save.json.dat"
    );

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
        .join("save.toml");

    let data: toml::Table = toml::from_str("test = true").unwrap();
    persistence::save_to_file(&data, &file_path, false).unwrap();

    assert!(file_path.exists());
    let loaded = persistence::load_from_file(&file_path).unwrap();
    assert_eq!(loaded["test"].as_bool(), Some(true));
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

    assert_eq!(
        result.len().unwrap(),
        0,
        "corrupted file should degrade to empty table"
    );
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

    // ファイルは gzip マジックで始まる（平文 TOML ではない）
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

/// 保存先パスが書き込み不能（親が通常のファイル）の場合、
/// save() は (nil, エラーメッセージ) を返し、panic も Lua エラーもしない。
#[test]
fn test_persistence_save_failure_returns_nil_and_message() {
    let temp_dir = TempDir::new().unwrap();
    // 親フォルダの位置に通常のファイルを置き、フォルダの作成か読み込みと退避を失敗させる
    std::fs::write(temp_dir.path().join("blocker"), "not a directory").unwrap();

    let (lua, module) = register_module_in_temp(&temp_dir, "blocker/save.toml", false);
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

// ============================================================================
// TOML のセーブの結合テスト（toml-serialization タスク 4）
// ============================================================================

/// モジュールの `save(data)` を呼び、成功を確かめる。
fn call_save(module: &mlua::Table, data: mlua::Table) {
    let save_fn: mlua::Function = module.get("save").unwrap();
    let (ok, err): (Option<bool>, Option<String>) = save_fn.call(data).unwrap();
    assert_eq!(ok, Some(true), "save should succeed: {err:?}");
}

/// Lua から save() した素の TOML のファイルを load_from_file で読み戻す（1.1・1.3・4.5）。
#[test]
fn test_lua_save_writes_plain_toml_file() {
    let temp_dir = TempDir::new().unwrap();
    let (lua, module) = register_module_in_temp(&temp_dir, "save.toml", false);
    let data: mlua::Table = lua
        .load(r#"return { name = "あい", count = 42, ratio = 0.5, items = {"a", "b"}, flags = { on = true } }"#)
        .eval()
        .unwrap();
    call_save(&module, data);

    let path = temp_dir.path().join("save.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let parsed: toml::Table = toml::from_str(&text).expect("file must be plain TOML text");
    assert_eq!(parsed, persistence::load_from_file(&path).unwrap());
    assert_eq!(parsed["name"].as_str(), Some("あい"));
    assert_eq!(parsed["count"].as_integer(), Some(42));
    assert_eq!(parsed["ratio"].as_float(), Some(0.5));
    assert_eq!(parsed["items"][1].as_str(), Some("b"));
    assert_eq!(parsed["flags"]["on"].as_bool(), Some(true));
}

/// Lua から save() した gzip のファイルを load_from_file で読み戻す（1.2・1.3・2.2）。
#[test]
fn test_lua_save_writes_gzip_toml_file() {
    let temp_dir = TempDir::new().unwrap();
    let (lua, module) = register_module_in_temp(&temp_dir, "save.dat", true);
    let data: mlua::Table = lua
        .load(r#"return { name = "あい", items = {1, 2, 3} }"#)
        .eval()
        .unwrap();
    call_save(&module, data);

    let path = temp_dir.path().join("save.dat");
    let raw = std::fs::read(&path).unwrap();
    assert_eq!(&raw[0..2], &[0x1f, 0x8b], "file should be gzip compressed");
    let mut text = String::new();
    std::io::Read::read_to_string(&mut flate2::read::GzDecoder::new(&raw[..]), &mut text).unwrap();
    let parsed: toml::Table = toml::from_str(&text).expect("gzip content must be TOML text");
    assert_eq!(parsed, persistence::load_from_file(&path).unwrap());
    assert_eq!(parsed["name"].as_str(), Some("あい"));
    assert_eq!(parsed["items"][2].as_integer(), Some(3));
}

/// obfuscate の設定と逆の形のファイルも load() で読める（1.4）。
#[test]
fn test_load_reads_file_of_opposite_form() {
    let data: toml::Table = toml::from_str(r#"name = "逆""#).unwrap();
    for (file_path, obfuscate) in [("save.dat", true), ("save.toml", false)] {
        let temp_dir = TempDir::new().unwrap();
        // 設定と逆の形で書いておく
        persistence::save_to_file(&data, &temp_dir.path().join(file_path), !obfuscate).unwrap();

        let (_lua, module) = register_module_in_temp(&temp_dir, file_path, obfuscate);
        let load_fn: mlua::Function = module.get("load").unwrap();
        let result: mlua::Table = load_fn.call(()).unwrap();
        let name: String = result.get("name").unwrap();
        assert_eq!(
            name, "逆",
            "obfuscate={obfuscate} should read the opposite form"
        );
    }
}

/// `file_path = "save.json"` の古い JSON は save.json.broken に移り、次の保存で save.json に TOML が書かれる（2.3・5.1・5.2）。
#[test]
fn test_old_json_at_configured_path_is_moved_aside_then_toml_written() {
    let temp_dir = TempDir::new().unwrap();
    let old = r#"{"name": "old", "count": 1}"#;
    std::fs::write(temp_dir.path().join("save.json"), old).unwrap();

    let (lua, module) = register_module_in_temp(&temp_dir, "save.json", false);
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();
    assert!(result.get::<mlua::Value>("name").unwrap().is_nil());

    let broken = temp_dir.path().join("save.json.broken");
    assert_eq!(
        std::fs::read_to_string(&broken).unwrap(),
        old,
        "old content must be kept"
    );
    assert!(!temp_dir.path().join("save.json").exists());

    let data: mlua::Table = lua.load(r#"return { name = "new" }"#).eval().unwrap();
    call_save(&module, data);
    let text = std::fs::read_to_string(temp_dir.path().join("save.json")).unwrap();
    let parsed: toml::Table = toml::from_str(&text).expect("save.json must now hold TOML");
    assert_eq!(parsed["name"].as_str(), Some("new"));
    assert_eq!(std::fs::read_to_string(&broken).unwrap(), old);
}

/// 既定の設定では古い save.json は読まれず、中身も名前も変わらない（2.1・2.2）。
#[test]
fn test_default_config_ignores_old_save_json() {
    let temp_dir = TempDir::new().unwrap();
    let save_dir = temp_dir.path().join("profile/pasta/save");
    std::fs::create_dir_all(&save_dir).unwrap();
    let old = r#"{"name": "old"}"#;
    std::fs::write(save_dir.join("save.json"), old).unwrap();

    let lua = mlua::Lua::new();
    let module =
        persistence::register(&lua, &PersistenceConfig::default(), temp_dir.path()).unwrap();
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();
    assert!(
        result.get::<mlua::Value>("name").unwrap().is_nil(),
        "save.json must not be read"
    );

    let data: mlua::Table = lua.load(r#"return { name = "new" }"#).eval().unwrap();
    call_save(&module, data);

    assert_eq!(
        std::fs::read_to_string(save_dir.join("save.json")).unwrap(),
        old
    );
    assert!(!save_dir.join("save.json.broken").exists());
    let saved = persistence::load_from_file(&save_dir.join("save.toml")).unwrap();
    assert_eq!(saved["name"].as_str(), Some("new"));
}

/// obfuscate = true の古い gzip の JSON の save.dat は save.dat.broken に移る（5.1・5.2）。
#[test]
fn test_old_gzip_json_save_dat_is_moved_aside() {
    use std::io::Write;
    let temp_dir = TempDir::new().unwrap();
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(br#"{"name": "old"}"#).unwrap();
    let old = enc.finish().unwrap();
    std::fs::write(temp_dir.path().join("save.dat"), &old).unwrap();

    let (_lua, module) = register_module_in_temp(&temp_dir, "save.dat", true);
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: mlua::Table = load_fn.call(()).unwrap();
    assert!(result.get::<mlua::Value>("name").unwrap().is_nil());

    assert!(!temp_dir.path().join("save.dat").exists());
    assert_eq!(
        std::fs::read(temp_dir.path().join("save.dat.broken")).unwrap(),
        old
    );
}

/// 混ざった表は外され、警告が出る。ほかのキーは保存される（3.4・4.6）。
#[tracing_test::traced_test]
#[test]
fn test_mixed_table_key_is_dropped_with_warning() {
    let temp_dir = TempDir::new().unwrap();
    let (lua, module) = register_module_in_temp(&temp_dir, "save.toml", false);
    let data: mlua::Table = lua
        .load(r#"return { name = "ok", ["混ざり"] = { 1, 2, x = 3 } }"#)
        .eval()
        .unwrap();
    call_save(&module, data);

    assert!(logs_contain(
        "key '混ざり' was not saved: table mixes array items and named keys"
    ));
    let saved = persistence::load_from_file(&temp_dir.path().join("save.toml")).unwrap();
    assert_eq!(saved["name"].as_str(), Some("ok"));
    assert!(!saved.contains_key("混ざり"));
}

/// ゴーストを読み込んだランタイムを落とすと save.toml に TOML が書かれ、書けない値のキーだけが外れる（4.9）。
#[test]
fn test_drop_writes_toml_and_drops_only_unwritable_key() {
    let temp = common::create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let runtime = pasta_lua::loader::PastaLoader::load(temp.path()).unwrap();
    runtime
        .exec(
            r#"
            local SAVE = require("pasta.save")
            SAVE.name = "落とす前"
            SAVE.count = 7
            SAVE.f = function() end
            "#,
        )
        .unwrap();
    drop(runtime);

    let path = temp.path().join("profile/pasta/save/save.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let parsed: toml::Table = toml::from_str(&text).expect("drop must write TOML text");
    assert_eq!(parsed["name"].as_str(), Some("落とす前"));
    assert_eq!(parsed["count"].as_integer(), Some(7));
    assert!(
        !parsed.contains_key("f"),
        "the function value must be dropped"
    );
}
