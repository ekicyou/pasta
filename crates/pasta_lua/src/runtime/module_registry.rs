//! Module registration functions for PastaLuaRuntime.
//!
//! This file contains the `impl PastaLuaRuntime` methods that register
//! various Lua modules (split impl pattern).

use super::PastaLuaRuntime;
use super::enc;
use super::log;
use super::persistence;
use super::searcher::{install_module_searcher, register_chunk_table};
use crate::debug::playscene::resolve_position;
use crate::debug::source_map::SourceMap;
use crate::loader::{LoaderContext, PastaConfig};
use mlua::{IntoLua, Lua, Result as LuaResult, Table, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
    ///
    /// Windows では、FFI を開いた VM（`[loader] ffi = true`）に限り `ffi.load` を
    /// 包むラッパを設置する（`ffi_load::install`）。DLL をゴーストの探索のフォルダから
    /// wide API で先に読み、元の `ffi.load` には ANSI のファイル名を渡す。書き換えるのは
    /// `ffi` の表の `load` だけで、`package.*` と `.lua` の searcher には触らない。
    pub(crate) fn setup_package_path(lua: &Lua, loader_context: &LoaderContext) -> LuaResult<()> {
        let path = loader_context.generate_package_path();

        let package: Table = lua.globals().get("package")?;
        package.set("path", path.as_str())?;

        // 検索パスを UTF-8 として解釈する searcher へ置換する（他のどの Lua コードよりも前）。
        install_module_searcher(lua)?;
        // FFI を開いた VM では `ffi.load` を置き換える（開いていなければ何もしない）。
        #[cfg(windows)]
        super::ffi_load::install(lua, loader_context)?;
        // 読み込んだファイルをエラー文の形から引ける表（`@pasta_chunks`）を用意する。
        register_chunk_table(
            lua,
            &loader_context.base_dir,
            &loader_context.transpiled_output_dir,
            &loader_context.pasta_sources,
        )?;

        tracing::debug!(path = %path, "Set package.path");
        Ok(())
    }

    /// Register @pasta_config module with custom fields.
    ///
    /// Creates a plain (writable, not read-only) Lua table from the TOML
    /// custom_fields and registers it as the @pasta_config module.
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

        for pair in actor_table.pairs::<mlua::LuaString, Value>() {
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

    /// Register @pasta_sakura_script module.
    pub(crate) fn register_sakura_script_module(
        lua: &Lua,
        config: &Option<PastaConfig>,
    ) -> LuaResult<()> {
        // Get talk config, use defaults if not specified
        let talk_config = config.as_ref().and_then(|c| c.talk());
        let sakura_module = crate::sakura_script::register(lua, talk_config.as_ref())?;
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

    /// Register @toml module (`decode` only).
    ///
    /// Reads a TOML string into a Lua table with the same conversion as
    /// `@pasta_config`. Unreadable input returns `nil, message`; a non-string
    /// argument is a Lua error. Always available, independent of RuntimeConfig.libs.
    pub(crate) fn register_toml_module(lua: &Lua) -> LuaResult<()> {
        let table = lua.create_table()?;
        // Value で受ける: mlua の String は数を文字列へ変えてしまう（要件 2.7）。
        let decode = lua.create_function(|lua, text: Value| {
            let Value::String(text) = text else {
                return Err(mlua::Error::runtime(format!(
                    "bad argument #1 to 'decode' (string expected, got {})",
                    text.type_name()
                )));
            };
            let parsed = std::str::from_utf8(&text.as_bytes())
                .map_err(|e| e.to_string())
                .and_then(|s| toml::from_str::<toml::Table>(s).map_err(|e| e.to_string()));
            match parsed {
                Ok(t) => Ok((Self::toml_to_lua(lua, &toml::Value::Table(t))?, None)),
                Err(msg) => Ok((Value::Nil, Some(msg))),
            }
        })?;
        table.set("decode", decode)?;
        set_loaded_module(lua, "@toml", table)
    }

    /// Register the internal `@pasta_debug` module（scene-play-event 2.1/4.1/4.2）。
    ///
    /// 関数は 4 つ: `enabled()` はシーンマップの有無（`debug_enabled()` と同じ値）、
    /// `scene_at(file, line)` は位置を登録名の組 `(大域, ローカル)` に引く。マップが無い、
    /// 行が `1..=u32::MAX` の整数でない、見つからないときはエラーにせず nil を返す。
    /// `dump_variables(save)` は表を `profile/pasta/debug/variables.toml` にテキストの TOML で書く。
    /// 書けなければ警告を 1 行書くだけでエラーを上げない。
    /// `is_local_var_name(s)` は `s` の全体が `＄名前` のローカル変数名の決まりに当たるか（scene-play-locals 4.2）。
    /// 有効かは見ない（門は scene_play.lua）。
    /// 本番の入口（`from_loader_with_scene_dic`）だけが呼ぶ。
    pub(crate) fn register_debug_module(
        lua: &Lua,
        enabled: bool,
        source_map: Option<Arc<SourceMap>>,
        base_dir: &Path,
    ) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.set("enabled", lua.create_function(move |_, ()| Ok(enabled))?)?;
        let dump_path = base_dir.join("profile/pasta/debug/variables.toml");
        let dump_variables = lua.create_function(move |lua, save: Table| {
            if let Err(e) = persistence::dump_table(lua, &save, &dump_path) {
                let path = dump_path.display();
                tracing::warn!(%path, error = %e, "OnPastaDumpVariables: could not write {path}: {e}");
            }
            Ok(())
        })?;
        table.set("dump_variables", dump_variables)?;
        let base_dir = base_dir.to_path_buf();
        let scene_at = lua.create_function(move |_, (file, line): (String, Value)| {
            let line = match line {
                Value::Integer(i) => u32::try_from(i).ok(),
                Value::Number(n) if n.fract() == 0.0 && n >= 0.0 && n <= u32::MAX as f64 => {
                    Some(n as u32)
                }
                _ => None,
            };
            let found = match (&source_map, line) {
                (Some(map), Some(line)) if line >= 1 => {
                    resolve_position(map, &base_dir, &file, line)
                }
                _ => None,
            };
            Ok(found.unzip())
        })?;
        table.set("scene_at", scene_at)?;
        let is_local_var_name =
            lua.create_function(|_, s: String| Ok(pasta_dsl::parser::is_id(&s)))?;
        table.set("is_local_var_name", is_local_var_name)?;
        set_loaded_module(lua, "@pasta_debug", table)
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
