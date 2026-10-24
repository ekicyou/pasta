use super::*;
use tempfile::TempDir;

fn create_test_config(temp_dir: &TempDir, obfuscate: bool) -> (PersistenceConfig, PathBuf) {
    let file_name = if obfuscate { "save.dat" } else { "save.json" };
    let config = PersistenceConfig {
        obfuscate,
        file_path: file_name.to_string(),
        debug_mode: true,
    };
    let base_dir = temp_dir.path().to_path_buf();
    (config, base_dir)
}

#[test]
fn test_save_load_json() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.json");

    let data = serde_json::json!({
        "player_name": "Alice",
        "play_count": 42,
        "flags": {
            "tutorial_complete": true
        }
    });

    // Save
    save_to_file(&data, &file_path, false).unwrap();

    // Load
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_save_load_obfuscated() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.dat");

    let data = serde_json::json!({
        "player_name": "Bob",
        "inventory": ["sword", "shield"]
    });

    // Save with obfuscation
    save_to_file(&data, &file_path, true).unwrap();

    // Verify file starts with gzip magic
    let raw = fs::read(&file_path).unwrap();
    assert!(raw.len() >= 2);
    assert_eq!(raw[0], GZIP_MAGIC[0]);
    assert_eq!(raw[1], GZIP_MAGIC[1]);

    // Load
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_load_nonexistent_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.json");

    let result = load_from_file(&file_path);
    assert!(matches!(result, Err(PersistenceError::IoError(_))));
}

#[test]
fn test_load_corrupted_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("corrupted.json");

    // Write invalid JSON
    fs::write(&file_path, "{ invalid json }").unwrap();

    let result = load_from_file(&file_path);
    assert!(matches!(result, Err(PersistenceError::JsonError(_))));
}

#[test]
fn test_auto_detect_format() {
    let temp_dir = TempDir::new().unwrap();

    let data = serde_json::json!({"key": "value"});

    // Save as JSON
    let json_path = temp_dir.path().join("test.json");
    save_to_file(&data, &json_path, false).unwrap();

    // Save as gzip
    let gzip_path = temp_dir.path().join("test.dat");
    save_to_file(&data, &gzip_path, true).unwrap();

    // Both should load correctly
    let loaded_json = load_from_file(&json_path).unwrap();
    let loaded_gzip = load_from_file(&gzip_path).unwrap();

    assert_eq!(loaded_json, data);
    assert_eq!(loaded_gzip, data);
}

#[test]
fn test_atomic_write_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir
        .path()
        .join("subdir")
        .join("nested")
        .join("save.json");

    let data = serde_json::json!({"test": true});

    // Should create directories automatically
    save_to_file(&data, &file_path, false).unwrap();

    assert!(file_path.exists());
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_nested_table_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nested.json");

    let data = serde_json::json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": 123,
                    "array": [1, 2, 3],
                    "bool": true,
                    "string": "nested"
                }
            }
        }
    });

    save_to_file(&data, &file_path, false).unwrap();
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_empty_file_returns_empty_object() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.json");

    // Create empty file
    fs::write(&file_path, "").unwrap();

    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, serde_json::json!({}));
}

#[test]
fn test_lua_module_load() {
    let temp_dir = TempDir::new().unwrap();
    let (config, base_dir) = create_test_config(&temp_dir, false);

    let lua = Lua::new();
    let module = register(&lua, &config, &base_dir).unwrap();

    // Load should return empty table when file doesn't exist
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: Table = load_fn.call(()).unwrap();

    // Should be an empty table
    assert_eq!(result.len().unwrap(), 0);
}

#[test]
fn test_lua_module_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let (config, base_dir) = create_test_config(&temp_dir, false);

    let lua = Lua::new();
    let module = register(&lua, &config, &base_dir).unwrap();

    // Create test data
    let data: Table = lua.create_table().unwrap();
    data.set("name", "Test").unwrap();
    data.set("count", 42).unwrap();

    // Save
    let save_fn: mlua::Function = module.get("save").unwrap();
    let (ok, err): (Option<bool>, Option<String>) = save_fn.call(data.clone()).unwrap();
    assert_eq!(ok, Some(true));
    assert!(err.is_none());

    // Load
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: Table = load_fn.call(()).unwrap();

    // Verify data
    let name: String = result.get("name").unwrap();
    let count: i32 = result.get("count").unwrap();
    assert_eq!(name, "Test");
    assert_eq!(count, 42);
}

#[test]
fn test_register_rejects_directory_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let config = PersistenceConfig {
        obfuscate: false,
        file_path: "../../etc/malicious.json".to_string(),
        debug_mode: false,
    };

    let lua = Lua::new();
    let result = register(&lua, &config, &base_dir);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("directory traversal"),
        "Expected traversal error, got: {}",
        err_msg
    );
}

#[test]
fn test_register_rejects_rooted_path() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // "/" prefix has RootDir component on all platforms
    let config = PersistenceConfig {
        obfuscate: false,
        file_path: "/etc/passwd".to_string(),
        debug_mode: false,
    };

    let lua = Lua::new();
    let result = register(&lua, &config, &base_dir);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("directory traversal"),
        "Expected traversal error, got: {}",
        err_msg
    );
}
