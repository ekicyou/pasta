//! Module registration functions for PastaLuaRuntime.
//!
//! This file contains the `impl PastaLuaRuntime` methods that register
//! various Lua modules (split impl pattern).

use super::PastaLuaRuntime;
use super::enc;
use super::log;
use super::persistence;
use crate::loader::{LoaderContext, PastaConfig};
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl PastaLuaRuntime {
    /// Setup package.path for Lua module resolution.
    ///
    /// Sets the package.path to include all search paths from LoaderContext
    /// in priority order (first path has highest priority).
    ///
    /// On Windows, the path is converted to ANSI encoding to ensure
    /// Lua's file I/O functions (which use fopen) can resolve non-ASCII paths.
    pub(crate) fn setup_package_path(lua: &Lua, loader_context: &LoaderContext) -> LuaResult<()> {
        // Get path bytes in system encoding (ANSI on Windows, UTF-8 on Unix)
        let path_bytes = loader_context
            .generate_package_path_bytes()
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Create Lua string from raw bytes
        let lua_path_string = lua.create_string(&path_bytes)?;

        let package: Table = lua.globals().get("package")?;
        package.set("path", lua_path_string)?;

        // Log the path (interpret as UTF-8 if possible, otherwise show byte count)
        let path_display = String::from_utf8_lossy(&path_bytes);
        tracing::debug!(path = %path_display, "Set package.path");
        Ok(())
    }

    /// Register @pasta_config module with custom fields.
    ///
    /// Creates a read-only Lua table from the TOML custom_fields and
    /// registers it as the @pasta_config module.
    pub(crate) fn register_config_module(lua: &Lua, custom_fields: &toml::Table) -> LuaResult<()> {
        let config_table = Self::toml_to_lua(lua, &toml::Value::Table(custom_fields.clone()))?;

        // [actor] サブテーブルの各エントリに name = キー名 を注入
        Self::inject_actor_names(lua, &config_table)?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@pasta_config", config_table)?;

        tracing::debug!("Registered @pasta_config module");
        Ok(())
    }

    /// Inject `name` field into each actor sub-table under `config_table["actor"]`.
    ///
    /// For each sub-table entry in `config_table["actor"]`, sets `name = key_name`.
    /// This ensures CONFIG-derived actors have their name field set at the data source,
    /// so `BUILDER.build()` can resolve `actor.name` correctly for `actor_spots` lookup.
    ///
    /// If `config_table["actor"]` does not exist or is not a table, this is a no-op.
    /// Existing `name` fields are overwritten (TOML key is authoritative).
    fn inject_actor_names(_lua: &Lua, config_table: &Value) -> LuaResult<()> {
        let config = match config_table {
            Value::Table(t) => t,
            _ => return Ok(()),
        };

        let actor_section: Value = config.get("actor")?;
        let actor_table = match actor_section {
            Value::Table(t) => t,
            _ => return Ok(()),
        };

        for pair in actor_table.pairs::<mlua::String, Value>() {
            let (key, value) = pair?;
            if let Value::Table(sub_table) = value {
                sub_table.set("name", key.clone())?;
            }
        }

        Ok(())
    }

    /// Register @enc module for encoding conversion.
    ///
    /// Provides UTF-8 <-> ANSI conversion functions for Lua scripts.
    pub(crate) fn register_enc_module(lua: &Lua) -> LuaResult<()> {
        let enc_table = enc::register(lua)?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@enc", enc_table)?;

        tracing::debug!("Registered @enc module");
        Ok(())
    }

    /// Register @pasta_persistence module for persistent data storage.
    ///
    /// Provides load/save functions for persisting Lua tables to files.
    pub(crate) fn register_persistence_module(
        lua: &Lua,
        config: &Option<PastaConfig>,
        base_dir: &Option<PathBuf>,
    ) -> LuaResult<()> {
        // Get persistence config, use defaults if not specified
        let persistence_config = config
            .as_ref()
            .and_then(|c| c.persistence())
            .unwrap_or_default();

        let base = base_dir.as_deref().unwrap_or(Path::new("."));

        let persistence_table = persistence::register(lua, &persistence_config, base)?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@pasta_persistence", persistence_table)?;

        tracing::debug!("Registered @pasta_persistence module");
        Ok(())
    }

    /// Register @pasta_sakura_script module for wait insertion.
    ///
    /// Provides talk_to_script function for natural conversation tempo.
    pub(crate) fn register_sakura_script_module(
        lua: &Lua,
        config: &Option<PastaConfig>,
    ) -> LuaResult<()> {
        // Get talk config, use defaults if not specified
        let talk_config = config.as_ref().and_then(|c| c.talk());

        let sakura_module = crate::sakura_script::register(lua, talk_config.as_ref())?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@pasta_sakura_script", sakura_module)?;

        tracing::debug!("Registered @pasta_sakura_script module");
        Ok(())
    }

    /// Register @pasta_log module for Lua logging bridge.
    ///
    /// Provides trace/debug/info/warn/error functions that bridge to Rust tracing.
    /// Always available, independent of RuntimeConfig.libs.
    pub(crate) fn register_log_module(lua: &Lua) -> LuaResult<()> {
        let log_table = log::register(lua)?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@pasta_log", log_table)?;

        tracing::debug!("Registered @pasta_log module");
        Ok(())
    }

    /// Convert toml::Value to mlua::Value.
    ///
    /// Recursively converts TOML structures to Lua tables.
    fn toml_to_lua(lua: &Lua, value: &toml::Value) -> LuaResult<Value> {
        match value {
            toml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
            toml::Value::Integer(i) => Ok(Value::Number(*i as f64)),
            toml::Value::Float(f) => Ok(Value::Number(*f)),
            toml::Value::Boolean(b) => Ok(Value::Boolean(*b)),
            toml::Value::Datetime(dt) => Ok(Value::String(lua.create_string(dt.to_string())?)),
            toml::Value::Array(arr) => {
                let table = lua.create_table()?;
                for (i, v) in arr.iter().enumerate() {
                    table.set(i + 1, Self::toml_to_lua(lua, v)?)?;
                }
                Ok(Value::Table(table))
            }
            toml::Value::Table(t) => {
                let table = lua.create_table()?;
                for (k, v) in t {
                    table.set(k.as_str(), Self::toml_to_lua(lua, v)?)?;
                }
                Ok(Value::Table(table))
            }
        }
    }
}
