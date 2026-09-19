//! Module registration functions for PastaLuaRuntime.
//!
//! This file contains the `impl PastaLuaRuntime` methods that register
//! various Lua modules (split impl pattern).

use super::PastaLuaRuntime;
use super::enc;
use super::log;
use super::persistence;
use super::renderer_injection::RendererInjection;
use super::searcher::install_module_searcher;
use crate::loader::{LoaderContext, PastaConfig};
use mlua::{IntoLua, Lua, Result as LuaResult, Table, Value};
use std::path::{Path, PathBuf};

/// Register a module into `package.loaded` under the given name.
///
/// This is the shared helper that eliminates the repeated
/// `package` → `loaded` → `set` → `tracing::debug!` boilerplate
/// found in every `register_*_module` function.
fn set_loaded_module(lua: &Lua, name: &str, module: impl IntoLua) -> LuaResult<()> {
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set(name, module)?;
    tracing::debug!("Registered {name} module");
    Ok(())
}

impl PastaLuaRuntime {
    /// Setup package.path for Lua module resolution.
    ///
    /// Sets the package.path to include all search paths from LoaderContext
    /// in priority order (first path has highest priority).
    ///
    /// 検索パス設定は [`LoaderContext::generate_package_path`] の UTF-8 文字列を
    /// **無変換で**設定する（要件 2.1・設計 PackagePathSetup）。ANSI 変換は行わない
    /// ため、非 ANSI の設置パスで設定が失敗する経路を持たない。ASCII パスでは
    /// 変更前とバイト同一である。設定は従来どおり上書きで、`package.cpath` は
    /// 変更しない。
    ///
    /// あわせて [`install_module_searcher`] を設置し、`require` のファイル解決を
    /// narrow `fopen` から Rust std（wide API）へ移す。これにより長パス（要件 1.4）と
    /// 非 ASCII のモジュール名・設置パス（要件 3.8）が解決できるようになる。
    pub(crate) fn setup_package_path(lua: &Lua, loader_context: &LoaderContext) -> LuaResult<()> {
        let path = loader_context.generate_package_path();

        let package: Table = lua.globals().get("package")?;
        package.set("path", path.as_str())?;

        // 検索パスを UTF-8 として解釈する searcher へ置換する（他のどの Lua コードよりも前）。
        install_module_searcher(lua)?;

        tracing::debug!(path = %path, "Set package.path");
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

        set_loaded_module(lua, "@pasta_config", config_table)
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
        set_loaded_module(lua, "@enc", enc_table)
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
        set_loaded_module(lua, "@pasta_persistence", persistence_table)
    }

    /// Register @pasta_sakura_script module via an adapter-injected renderer.
    ///
    /// 設計の **SakuraRenderer（アダプタ注入）** Service Interface
    /// (`register_sakura_script_module(lua, config, renderer)`)。コアはさくら描画を
    /// **無条件 hard-code 起点としては保持せず**、注入された [`RendererInjection`] を
    /// 通じてのみレンダラを登録する（R3.1/R3.2/R3.3）。
    ///
    /// `renderer` の既定（[`RendererInjection::default`]）は SHIORI さくらレンダラ
    /// （= 既存 `crate::sakura_script::register` 経路）であり、`talk_to_script` 出力は
    /// ゴールデンとバイト不変（R3.5/R3.6）。実レンダリングは VM 内 Lua に維持される
    /// （Lua 集約死守・物理移動なし）。
    pub(crate) fn register_sakura_script_module(
        lua: &Lua,
        config: &Option<PastaConfig>,
        renderer: RendererInjection,
    ) -> LuaResult<()> {
        // Get talk config, use defaults if not specified
        let talk_config = config.as_ref().and_then(|c| c.talk());

        // アダプタ注入レンダラ経由で登録（既定 = SHIORI さくらレンダラ = 既存挙動）。
        let sakura_module = renderer.register_module(lua, talk_config.as_ref())?;
        set_loaded_module(lua, "@pasta_sakura_script", sakura_module)
    }

    /// Register @pasta_log module for Lua logging bridge.
    ///
    /// Provides trace/debug/info/warn/error functions that bridge to Rust tracing.
    /// Always available, independent of RuntimeConfig.libs.
    pub(crate) fn register_log_module(lua: &Lua) -> LuaResult<()> {
        let log_table = log::register(lua)?;
        set_loaded_module(lua, "@pasta_log", log_table)
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
