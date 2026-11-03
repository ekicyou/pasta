//! Pasta Lua Runtime - Lua VM host for Pasta scripts.
//!
//! This module provides the PastaLuaRuntime struct which hosts a Lua VM
//! and integrates pasta modules for script execution.
//!
//! # Example
//!
//! ```rust,ignore
//! use pasta_lua::{LuaTranspiler, PastaLuaRuntime};
//!
//! let transpiler = LuaTranspiler::default();
//! let mut output = Vec::new();
//! let context = transpiler.transpile(&pasta_file, &mut output)?;
//!
//! let runtime = PastaLuaRuntime::new(context)?;
//! let result = runtime.exec("return 1 + 1")?;
//! ```

mod enc;
mod exec;
mod factory;
/// `ffi.load` の置き換え（UTF-16 で先に読む・探索のフォルダ・同名の確かめ）。
#[cfg(windows)]
mod ffi_load;
/// Finalize module - Collects Lua-side registries and builds SearchContext.
pub mod finalize;
mod lifecycle;
/// Log module - Lua logging bridge to Rust tracing infrastructure.
pub mod log;
mod module_registry;
/// Persistence module - Persistent data storage for Lua scripts.
pub mod persistence;
mod runtime_config;
/// Searcher module - Rust std backed Lua file searcher for `package.loaders[2]`.
mod searcher;

pub use runtime_config::RuntimeConfig;
pub use runtime_config::default_libs;
pub use runtime_config::lua_require;
pub use searcher::install_module_searcher;

use crate::context::TranspileContext;
use crate::debug::source_map::SourceMap;
use crate::loader::PastaConfig;
use crate::logging::PastaLogger;
pub(crate) use finalize::register_finalize_scene;
use mlua::{Function, Lua, Result as LuaResult, Table};
use std::path::PathBuf;
use std::sync::Arc;

/// Pasta Lua Runtime - hosts a Lua VM with pasta modules.
///
/// Each instance owns an independent Lua VM and SearchContext.
/// Multiple instances can coexist without interference.
pub struct PastaLuaRuntime {
    lua: Lua,
    /// Instance-specific logger (optional).
    /// Held so the logger lives as long as the runtime; tracing output reaches
    /// it through the GlobalLoggerRegistry entry, not through this field.
    /// Wrapped in Arc for sharing with GlobalLoggerRegistry.
    logger: Option<Arc<PastaLogger>>,
    /// Configuration for persistence and other runtime settings.
    /// Used for Drop-time auto-save and other Rust-side operations.
    config: Option<PastaConfig>,
    /// Base directory for resolving relative paths (persistence file, etc.).
    base_dir: Option<PathBuf>,
    /// シーンマップ。`Some` なのはローダがデバッグを有効と解決して作ったときだけで、
    /// その有無が「デバッグが有効か」の唯一の値になる。ローダを通らない VM はいつも `None`。
    source_map: Option<Arc<SourceMap>>,
}

impl PastaLuaRuntime {
    /// Create a new runtime from a TranspileContext with default configuration.
    ///
    /// Initializes a Lua VM with standard libraries enabled and registers:
    /// - `@pasta_search` module with scene and word registries
    /// - `@pasta_log` module for the Lua→Rust tracing log bridge
    ///   (always registered, independent of `RuntimeConfig::libs`)
    /// - `@toml` module for TOML decoding (always registered)
    ///
    /// # Arguments
    /// * `context` - TranspileContext from LuaTranspiler::transpile()
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized successfully
    /// * `Err(e)` - Lua VM or module registration failed
    pub fn new(context: TranspileContext) -> LuaResult<Self> {
        Self::with_config(context, RuntimeConfig::new())
    }

    /// Create a new runtime from a TranspileContext with custom configuration.
    ///
    /// # Arguments
    /// * `context` - TranspileContext from LuaTranspiler::transpile()
    /// * `config` - Runtime configuration for library loading
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized successfully
    /// * `Err(e)` - Lua VM or module registration failed
    pub fn with_config(context: TranspileContext, config: RuntimeConfig) -> LuaResult<Self> {
        // シーンマップを作るのはローダだけなので、この入口の VM はいつもデバッグが無効。
        Self::with_config_and_source_map(context, config, None)
    }

    /// Like [`with_config`](Self::with_config) but holds the scene map the
    /// loader built (only when debugging is enabled) as-is.
    pub(crate) fn with_config_and_source_map(
        context: TranspileContext,
        config: RuntimeConfig,
        source_map: Option<Arc<SourceMap>>,
    ) -> LuaResult<Self> {
        // Validate configuration and emit warnings
        config.validate_and_warn();

        // Required libraries must be present before any VM is built.
        config
            .ensure_libs()
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Convert libs array to StdLib flags
        let std_lib = config
            .to_stdlib()
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Create Lua VM with appropriate standard libraries
        // SAFETY: `Lua::unsafe_new_with` is required because mlua's safe `new` does not
        // accept custom StdLib flags. The safety invariants are upheld because:
        //  1. `std_lib` is constructed from validated `RuntimeConfig` via `to_stdlib()`,
        //     which only maps known library names to mlua `StdLib` flags.
        //  2. `validate_and_warn()` above alerts on dangerous libraries (std_debug /
        //     std_all_unsafe).
        //  3. Default LuaOptions are used, so no custom allocator or hook is involved.
        //  4. The returned `Lua` handle is used in a single-threaded context and is not
        //     shared across threads.
        let lua = unsafe { Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) };

        // LuaJIT の乱数は固定の種で始まるため、起動ごとに同じ列になる。
        // math がある構成でだけ、時刻とプロセス ID から種を与える。
        if let Ok(math) = lua.globals().get::<Table>("math") {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            // f64 で誤差なく表せる 53 ビットに収める。
            let seed = (nanos ^ u64::from(std::process::id())) & ((1u64 << 53) - 1);
            math.get::<Function>("randomseed")?
                .call::<()>(seed as f64)?;
        }

        // Extract registries from context
        let scene_registry = context.scene_registry;
        let word_registry = context.word_registry;

        // Register @pasta_search module (the same alias table is handed to the
        // finalize_scene re-registration via `register_finalize_scene`)
        crate::search::register(
            &lua,
            scene_registry,
            word_registry,
            config.scene_aliases.clone(),
        )?;

        // Register @pasta_log module (always available, independent of RuntimeConfig.libs)
        Self::register_log_module(&lua)?;

        // Register @toml module (always available, independent of RuntimeConfig.libs)
        Self::register_toml_module(&lua)?;

        Ok(Self {
            lua,
            logger: None,
            config: None,
            base_dir: None,
            source_map,
        })
    }
}
