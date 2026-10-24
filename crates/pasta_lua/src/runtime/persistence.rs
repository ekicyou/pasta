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
use std::path::{Component, Path, PathBuf};
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

    /// TOML として読めない（UTF-8 でない中身もここ）。
    #[error("TOML parse error: {0}")]
    TomlParseError(#[from] toml::de::Error),

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

    // Validate file path to prevent directory traversal
    let effective_path = config.effective_file_path();
    let relative = Path::new(&effective_path);
    if relative.is_absolute()
        || relative.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(mlua::Error::RuntimeError(format!(
            "Invalid persistence file path (directory traversal detected): {}",
            effective_path
        )));
    }

    // Create state for closures
    let file_path = base_dir.join(&effective_path);
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
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
        tracing::debug!(path = %parent.display(), "Created persistence directory");
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

// ---------------------------------------------------------------------------
// SaveCodec（Lua の値 → TOML の値）
// ---------------------------------------------------------------------------

/// セーブの入れ子の上限（根の表を 1 段目と数える）。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で save_table が使い始めたら外す
pub(crate) const SAVE_MAX_DEPTH: usize = 32;

/// 保存しなかったキー 1 件。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で警告に使い始めたら外す
#[derive(Debug)]
pub(crate) struct DroppedKey {
    /// 根からの道筋（例: "flags.secret"・"inventory[1].f"）。
    pub path: String,
    /// 書けない理由。
    pub reason: String,
}

/// セーブの根の表を TOML の文書にする。書けないキーは外して一覧に入れる。失敗しない。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で save_table が使い始めたら外す
pub(crate) fn lua_table_to_toml_document(
    lua: &Lua,
    root: &Table,
) -> (toml::Table, Vec<DroppedKey>) {
    let mut dropped = Vec::new();
    let mut ancestors = vec![root.to_pointer()];
    // 根は必ずキーのある表として読む（文字列でないキーは convert_keyed が 1 つずつ外す）
    let doc = convert_keyed(
        lua,
        raw_entries(root),
        1,
        SAVE_MAX_DEPTH,
        &mut ancestors,
        "",
        Some(&mut dropped),
    )
    .unwrap_or_default(); // 外すモードの convert_keyed は Err を返さない
    (doc, dropped)
}

/// 値を TOML の値にする。どこか 1 つでも書けなければ Err(理由)。ログが使う。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 3.3 で log.rs が使い始めたら外す
pub(crate) fn lua_value_to_toml_strict(
    lua: &Lua,
    value: &Value,
    max_depth: usize,
) -> Result<toml::Value, String> {
    convert(lua, value, 1, max_depth, &mut Vec::new(), "", None)
}

/// 表の中身を生のたどり方で読む（`__pairs` を呼ばない）。
fn raw_entries(table: &Table) -> Vec<(Value, Value)> {
    // Value → Value の変換は失敗しない
    table.pairs::<Value, Value>().flatten().collect()
}

/// 値 1 つを変換する。`dropped` が `None` なら厳しいモード。
/// `depth` はこの値が表のときの段（根の表が 1）。`ancestors` はいまたどっている祖先の表。
fn convert(
    lua: &Lua,
    value: &Value,
    depth: usize,
    max_depth: usize,
    ancestors: &mut Vec<*const std::ffi::c_void>,
    path: &str,
    dropped: Option<&mut Vec<DroppedKey>>,
) -> Result<toml::Value, String> {
    match value {
        Value::String(s) => s
            .to_str()
            .map(|s| toml::Value::String(s.to_owned()))
            .map_err(|_| "string is not valid UTF-8".to_string()),
        Value::Boolean(b) => Ok(toml::Value::Boolean(*b)),
        Value::Integer(i) => Ok(toml::Value::Integer(*i)),
        Value::Number(f) => Ok(toml::Value::Float(*f)),
        Value::Table(t) => {
            let ptr = t.to_pointer();
            if ancestors.contains(&ptr) {
                return Err("recursive table".into());
            }
            if depth > max_depth {
                return Err(format!("nested deeper than {max_depth} levels"));
            }
            ancestors.push(ptr);
            let result = convert_table(lua, t, depth, max_depth, ancestors, path, dropped);
            ancestors.pop();
            result
        }
        other => Err(format!("unsupported value type ({})", other.type_name())),
    }
}

/// 表を分類して、空の表・空の配列・配列・キーのある表のどれかにする。
fn convert_table(
    lua: &Lua,
    table: &Table,
    depth: usize,
    max_depth: usize,
    ancestors: &mut Vec<*const std::ffi::c_void>,
    path: &str,
    mut dropped: Option<&mut Vec<DroppedKey>>,
) -> Result<toml::Value, String> {
    let entries = raw_entries(table);
    if entries.is_empty() {
        // メタテーブルは mlua の配列の印だけを見る
        let is_array = table
            .metatable()
            .is_some_and(|mt| mt.to_pointer() == lua.array_metatable().to_pointer());
        return Ok(if is_array {
            toml::Value::Array(Vec::new())
        } else {
            toml::Value::Table(toml::Table::new())
        });
    }

    let (mut strings, mut ints, mut max_int, mut has_one) = (0usize, 0i64, 0i64, false);
    for (key, _) in &entries {
        match key {
            Value::String(_) => strings += 1,
            Value::Integer(i) if *i >= 1 => {
                ints += 1;
                max_int = max_int.max(*i);
                has_one |= *i == 1;
            }
            other => return Err(format!("non-string key ({})", other.type_name())),
        }
    }
    if strings > 0 && ints > 0 {
        return Err("table mixes array items and named keys".into());
    }
    if strings > 0 {
        return convert_keyed(lua, entries, depth, max_depth, ancestors, path, dropped)
            .map(toml::Value::Table);
    }
    if max_int != ints {
        return Err(if has_one {
            "hole in array".to_string()
        } else {
            "non-string key (integer)".to_string()
        });
    }

    // 1 から穴なし: 配列。要素が 1 つでも書けなければ配列ごと書けない
    let mut items = Vec::with_capacity(entries.len());
    for i in 1..=ints {
        let item: Value = table.raw_get(i).map_err(|e| e.to_string())?;
        let item_path = format!("{path}[{i}]");
        items.push(convert(
            lua,
            &item,
            depth + 1,
            max_depth,
            ancestors,
            &item_path,
            dropped.as_deref_mut(),
        )?);
    }
    Ok(toml::Value::Array(items))
}

/// キーのある表を変換する。外すモードでは書けない値のキーだけを外す。
/// 文字列でないキーは根でだけ届く（ほかの段では convert_table が先に断る）。
fn convert_keyed(
    lua: &Lua,
    entries: Vec<(Value, Value)>,
    depth: usize,
    max_depth: usize,
    ancestors: &mut Vec<*const std::ffi::c_void>,
    path: &str,
    mut dropped: Option<&mut Vec<DroppedKey>>,
) -> Result<toml::Table, String> {
    let join = |key: &str| {
        if path.is_empty() {
            key.to_string()
        } else {
            format!("{path}.{key}")
        }
    };
    let mut out = toml::Table::new();
    for (key, value) in entries {
        let (key_path, reason) = match &key {
            Value::String(s) => match s.to_str() {
                Ok(k) => {
                    let key_path = join(&k);
                    let mark = dropped.as_ref().map(|d| d.len());
                    match convert(
                        lua,
                        &value,
                        depth + 1,
                        max_depth,
                        ancestors,
                        &key_path,
                        dropped.as_deref_mut(),
                    ) {
                        Ok(v) => {
                            out.insert(k.to_owned(), v);
                            continue;
                        }
                        Err(reason) => {
                            // 配列ごと外すときは、その中で先に足した件を取り消す
                            if let (Some(d), Some(mark)) = (dropped.as_deref_mut(), mark) {
                                d.truncate(mark);
                            }
                            (key_path, reason)
                        }
                    }
                }
                Err(_) => (
                    join(&s.to_string_lossy()),
                    "key is not valid UTF-8".to_string(),
                ),
            },
            other => {
                let label = match other {
                    Value::Integer(i) => i.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    _ => other.type_name().to_string(),
                };
                (
                    format!("{path}[{label}]"),
                    format!("non-string key at root ({})", other.type_name()),
                )
            }
        };
        match dropped.as_deref_mut() {
            Some(d) => d.push(DroppedKey {
                path: key_path,
                reason,
            }),
            None => return Err(reason),
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// SaveCodec（TOML の値 → Lua の値）
// ---------------------------------------------------------------------------

/// TOML の値を Lua の値にする（日時は文字列、配列には配列の印）。
/// 設定の読み込みの変換（module_registry）は呼ばず、ここに小さな写しを持つ。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で load が使い始めたら外す
fn toml_to_lua(lua: &Lua, value: &toml::Value) -> LuaResult<Value> {
    Ok(match value {
        toml::Value::String(s) => Value::String(lua.create_string(s)?),
        toml::Value::Integer(i) => Value::Integer(*i),
        toml::Value::Float(f) => Value::Number(*f),
        toml::Value::Boolean(b) => Value::Boolean(*b),
        toml::Value::Datetime(dt) => Value::String(lua.create_string(dt.to_string())?),
        toml::Value::Array(arr) => {
            let table = lua.create_table_with_capacity(arr.len(), 0)?;
            for (i, v) in arr.iter().enumerate() {
                table.raw_set(i + 1, toml_to_lua(lua, v)?)?;
            }
            // 空の配列が空の配列のまま書き戻るように印を付ける
            table.set_metatable(Some(lua.array_metatable()))?;
            Value::Table(table)
        }
        toml::Value::Table(t) => {
            let table = lua.create_table_with_capacity(0, t.len())?;
            for (k, v) in t {
                table.raw_set(k.as_str(), toml_to_lua(lua, v)?)?;
            }
            Value::Table(table)
        }
    })
}

// ---------------------------------------------------------------------------
// SaveStore（読み込み・退避・SaveGate）
// ---------------------------------------------------------------------------

/// 1 つのランタイムで、セーブファイルを読んだ結果（Lua の app_data）。無い = まだ読んでいない。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で save_table が使い始めたら外す
#[derive(Debug, Clone, PartialEq)]
enum SaveGate {
    /// 読めた・無かった・退避できた。保存してよい。
    Open,
    /// 退避できなかった。理由を持ち、このランタイムでは保存しない。
    Closed(String),
}

/// gzip の印を見て展開し、TOML の文書として読む。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で load_from_file に置き換えたら外す
fn load_toml_from_file(path: &Path) -> Result<toml::Table, PersistenceError> {
    let data = fs::read(path)?;
    if data.starts_with(&GZIP_MAGIC) {
        let mut toml_bytes = Vec::new();
        GzDecoder::new(&data[..]).read_to_end(&mut toml_bytes)?;
        Ok(toml::from_slice(&toml_bytes)?)
    } else {
        Ok(toml::from_slice(&data)?)
    }
}

/// 印を置く。`Closed` を `Open` で上書きしない。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で read_or_quarantine が使い始めたら外す
fn set_gate(lua: &Lua, gate: SaveGate) {
    // 借りたまま set_app_data を呼ばないよう、判定の借用は先に返す
    let closed = matches!(
        lua.app_data_ref::<SaveGate>().as_deref(),
        Some(SaveGate::Closed(_))
    );
    if !closed {
        lua.set_app_data(gate);
    }
}

/// 読む・退避する・印を置く。どの結果でも表を返す（失敗は空の表）。
#[cfg_attr(not(test), allow(dead_code))] // shortcut: 2.2 で load_impl が使い始めたら外す
fn read_or_quarantine(lua: &Lua, path: &Path) -> toml::Table {
    match load_toml_from_file(path) {
        Ok(doc) => {
            set_gate(lua, SaveGate::Open);
            doc
        }
        Err(PersistenceError::IoError(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(path = %path.display(), "Persistence file not found, using empty table");
            set_gate(lua, SaveGate::Open);
            toml::Table::new()
        }
        Err(e) => {
            match quarantine(path) {
                Ok(to) => {
                    tracing::warn!(
                        from = %path.display(),
                        to = %to.display(),
                        error = %e,
                        "Unreadable persistence file moved aside, using empty table"
                    );
                    set_gate(lua, SaveGate::Open);
                }
                Err(rename_error) => {
                    tracing::error!(
                        path = %path.display(),
                        error = %e,
                        rename_error = %rename_error,
                        "Unreadable persistence file could not be moved aside; saving is disabled for this runtime"
                    );
                    set_gate(
                        lua,
                        SaveGate::Closed(format!(
                            "{} could not be read ({e}) or moved aside ({rename_error})",
                            path.display()
                        )),
                    );
                }
            }
            toml::Table::new()
        }
    }
}

/// 空いている .broken の名前（`.broken`、`.broken.1`、…）を選んで移す。移した先を返す。
fn quarantine(path: &Path) -> std::io::Result<PathBuf> {
    let name = |suffix: String| {
        let mut s = path.as_os_str().to_owned();
        s.push(suffix);
        PathBuf::from(s)
    };
    // rename は Windows でも行き先を置き換えるので、先に有無を確かめる
    let to = std::iter::once(name(".broken".into()))
        .chain((1..).map(|n| name(format!(".broken.{n}"))))
        .find(|p| !p.exists())
        .expect("unbounded candidates");
    fs::rename(path, &to)?;
    Ok(to)
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod tests;
