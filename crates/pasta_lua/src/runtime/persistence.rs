//! Persistence module for Lua.
//!
//! Provides the `@pasta_persistence` module with functions to load and save
//! persistent data to files with optional gzip compression (obfuscation).
//!
//! # Example
//! ```lua
//! local persistence = require "@pasta_persistence"
//!
//! -- Load data (returns empty table if file not found)
//! local data = persistence.load()
//!
//! -- Modify data
//! data.player_name = "Alice"
//! data.play_count = 42
//!
//! -- Save data (explicit save)
//! local ok, err = persistence.save(data)
//! if not ok then
//!     print("Save failed:", err)
//! end
//! ```

use crate::loader::PersistenceConfig;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use mlua::{Lua, LuaSerdeExt, Result as LuaResult, Table, Value};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Module version.
const VERSION: &str = "0.1.0";

/// Module description.
const DESCRIPTION: &str = "Persistent data storage (JSON/gzip)";

/// Gzip magic header bytes.
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// Persistence error types.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Lua value conversion error.
    #[error("Lua conversion error: {0}")]
    LuaConversionError(String),

    /// Configuration not found.
    #[error("Persistence configuration not found")]
    ConfigNotFound,

    /// Lua VM access error.
    #[error("Lua VM access error: {0}")]
    LuaAccessError(String),

    /// Invalid file format.
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
}

impl From<mlua::Error> for PersistenceError {
    fn from(e: mlua::Error) -> Self {
        PersistenceError::LuaConversionError(e.to_string())
    }
}

/// Internal state for the persistence module.
/// Stored as upvalue in Lua closures.
#[derive(Debug, Clone)]
struct PersistenceState {
    /// Absolute path to the persistence file.
    file_path: PathBuf,
    /// Whether to use gzip compression (obfuscation).
    obfuscate: bool,
    /// Enable debug logging.
    debug_mode: bool,
}

/// Register the @pasta_persistence module with the Lua state.
///
/// Creates a module table with:
/// - `_VERSION` - Module version string
/// - `_DESCRIPTION` - Module description
/// - `load()` - Load data from persistence file
/// - `save(data)` - Save data to persistence file
///
/// # Arguments
/// * `lua` - The Lua state to register the module with
/// * `config` - Persistence configuration
/// * `base_dir` - Base directory for resolving relative paths
///
/// # Returns
/// * `Ok(Table)` - The module table
/// * `Err(e)` - Registration failed
pub fn register(lua: &Lua, config: &PersistenceConfig, base_dir: &Path) -> LuaResult<Table> {
    let module = lua.create_table()?;

    // Set module metadata
    module.set("_VERSION", VERSION)?;
    module.set("_DESCRIPTION", DESCRIPTION)?;

    // Create state for closures
    let file_path = base_dir.join(config.effective_file_path());
    let state = PersistenceState {
        file_path: file_path.clone(),
        obfuscate: config.obfuscate,
        debug_mode: config.debug_mode,
    };

    if config.debug_mode {
        tracing::debug!(
            path = %file_path.display(),
            obfuscate = config.obfuscate,
            "Persistence module initialized"
        );
    }

    // Register load function
    let load_state = state.clone();
    module.set(
        "load",
        lua.create_function(move |lua, ()| load_impl(lua, &load_state))?,
    )?;

    // Register save function
    let save_state = state;
    module.set(
        "save",
        lua.create_function(move |lua, data: Table| save_impl(lua, &save_state, data))?,
    )?;

    Ok(module)
}

/// Implementation of `persistence.load()`.
///
/// Loads data from the persistence file.
/// Returns empty table if file doesn't exist or is corrupted.
fn load_impl(lua: &Lua, state: &PersistenceState) -> LuaResult<Table> {
    match load_from_file(&state.file_path) {
        Ok(value) => {
            if state.debug_mode {
                tracing::debug!(path = %state.file_path.display(), "Loaded persistence data");
            }
            // Convert serde_json::Value to Lua Value, then extract table
            let lua_value: Value = lua.to_value(&value)?;
            match lua_value {
                Value::Table(t) => Ok(t),
                _ => {
                    tracing::warn!(
                        path = %state.file_path.display(),
                        "Persistence data is not an object, using empty table"
                    );
                    lua.create_table()
                }
            }
        }
        Err(PersistenceError::IoError(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            // File not found is expected on first run
            tracing::warn!(path = %state.file_path.display(), "Persistence file not found, using empty table");
            lua.create_table()
        }
        Err(e) => {
            // Other errors: log warning and return empty table
            tracing::warn!(error = %e, path = %state.file_path.display(), "Failed to load persistence data, using empty table");
            lua.create_table()
        }
    }
}

/// Implementation of `persistence.save(data)`.
///
/// Saves data to the persistence file.
/// Returns (true, nil) on success, (nil, error_message) on error.
fn save_impl(
    lua: &Lua,
    state: &PersistenceState,
    data: Table,
) -> LuaResult<(Option<bool>, Option<String>)> {
    // Convert Lua table to serde_json::Value
    let lua_value = Value::Table(data);
    let json_value: serde_json::Value = match lua.from_value(lua_value) {
        Ok(v) => v,
        Err(e) => {
            let err_msg = format!("Failed to convert Lua value: {}", e);
            tracing::warn!(error = %err_msg, "Persistence save conversion error");
            return Ok((None, Some(err_msg)));
        }
    };

    // Save to file
    match save_to_file(&json_value, &state.file_path, state.obfuscate) {
        Ok(()) => {
            if state.debug_mode {
                tracing::debug!(path = %state.file_path.display(), "Saved persistence data");
            }
            Ok((Some(true), None))
        }
        Err(e) => {
            let err_msg = format!("Failed to save: {}", e);
            tracing::error!(error = %err_msg, path = %state.file_path.display(), "Persistence save error");
            Ok((None, Some(err_msg)))
        }
    }
}

/// Load data from a persistence file.
///
/// Automatically detects format (JSON or gzip) based on file content.
///
/// # Arguments
/// * `path` - Path to the persistence file
///
/// # Returns
/// * `Ok(Value)` - Loaded JSON value
/// * `Err(e)` - Load failed
pub fn load_from_file(path: &Path) -> Result<serde_json::Value, PersistenceError> {
    let data = fs::read(path)?;

    if data.is_empty() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }

    // Detect format by magic header
    if data.len() >= 2 && data[0] == GZIP_MAGIC[0] && data[1] == GZIP_MAGIC[1] {
        // Gzip compressed
        let mut decoder = GzDecoder::new(&data[..]);
        let mut json_bytes = Vec::new();
        decoder.read_to_end(&mut json_bytes)?;
        Ok(serde_json::from_slice(&json_bytes)?)
    } else {
        // Plain JSON
        Ok(serde_json::from_slice(&data)?)
    }
}

/// Save data to a persistence file.
///
/// Uses atomic write (temp file + rename) to prevent corruption.
///
/// # Arguments
/// * `data` - JSON value to save
/// * `path` - Path to the persistence file
/// * `obfuscate` - Whether to use gzip compression
///
/// # Returns
/// * `Ok(())` - Save successful
/// * `Err(e)` - Save failed
pub fn save_to_file(
    data: &serde_json::Value,
    path: &Path,
    obfuscate: bool,
) -> Result<(), PersistenceError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            tracing::debug!(path = %parent.display(), "Created persistence directory");
        }
    }

    // Serialize data
    let bytes = if obfuscate {
        // Gzip compressed
        let json_bytes = serde_json::to_vec(data)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json_bytes)?;
        encoder.finish()?
    } else {
        // Pretty-printed JSON
        serde_json::to_vec_pretty(data)?
    };

    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    let mut file = File::create(&temp_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);

    // Rename to final path
    if let Err(e) = fs::rename(&temp_path, path) {
        // Cleanup temp file on failure
        let _ = fs::remove_file(&temp_path);
        return Err(PersistenceError::IoError(e));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
}
