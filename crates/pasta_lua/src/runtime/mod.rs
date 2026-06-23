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
/// Finalize module - Collects Lua-side registries and builds SearchContext.
pub mod finalize;
mod lifecycle;
/// Log module - Lua logging bridge to Rust tracing infrastructure.
pub mod log;
mod module_registry;
/// Persistence module - Persistent data storage for Lua scripts.
pub mod persistence;
/// Renderer injection seam - adapter-originated `@pasta_sakura_script` registration.
pub mod renderer_injection;
mod runtime_config;

pub use renderer_injection::{RendererInjection, SakuraRenderBoundary, default_sakura_renderer};
pub use runtime_config::RuntimeConfig;
pub use runtime_config::lua_require;

use crate::context::TranspileContext;
use crate::debug::source_map::SourceMap;
use crate::loader::PastaConfig;
use crate::logging::PastaLogger;
pub(crate) use finalize::register_finalize_scene;
use mlua::{Lua, Result as LuaResult};
use std::path::PathBuf;
use std::sync::Arc;

/// Pasta Lua Runtime - hosts a Lua VM with pasta modules.
///
/// Each instance owns an independent Lua VM and SearchContext.
/// Multiple instances can coexist without interference.
pub struct PastaLuaRuntime {
    lua: Lua,
    /// Instance-specific logger (optional).
    /// If set, this logger is used for tracing output.
    /// Wrapped in Arc for sharing with GlobalLoggerRegistry.
    logger: Option<Arc<PastaLogger>>,
    /// Configuration for persistence and other runtime settings.
    /// Used for Drop-time auto-save and other Rust-side operations.
    config: Option<PastaConfig>,
    /// Base directory for resolving relative paths (persistence file, etc.).
    base_dir: Option<PathBuf>,
    /// Runtime-scope debug backend handle (task 4.2).
    ///
    /// `Some` only when debugging was enabled at VM init (`config.debug.enabled`);
    /// the hook is installed exactly ONCE there and this handle owns the bridge
    /// threads + transport + the runtime-scope `DebugSession`/`BreakpointSet`. It
    /// is `None` on the disabled (default) zero-cost path — no hook, no port, no
    /// `std_debug` (R5.2 / R5.3 / R5.5).
    ///
    /// Held at RUNTIME scope so breakpoint/session state set in one request
    /// PERSISTS across subsequent requests (design "セッションライフサイクル"). Its
    /// `Drop` (run after this struct's explicit `Drop` saves persistence) signals
    /// `terminated` to any connected client and tears the transport/threads down
    /// non-blockingly.
    debug_handle: Option<crate::debug::DebugHandle>,
    /// Aggregated `.pasta`↔`.lua` source map held at runtime scope (task 4.4).
    ///
    /// `Some` only when debugging was ENABLED at VM init AND the loader built the
    /// map (after transpile) and handed it to [`with_config`]→[`crate::debug::enable`]
    /// (requirements 3.1: メモリ内保持 ＋ enable へ到達). The same `Arc` was cloned
    /// into the debug backend's wiring/session by `enable`; this field keeps a
    /// runtime-scope clone so the map outlives a single request and is observable
    /// (e.g. for diagnostics / sidecar). On the disabled (default) zero-cost path
    /// it is `None` — the loader does not build a map and nothing is held
    /// (requirements 7.1, byte-invariant).
    source_map: Option<Arc<SourceMap>>,
}

impl PastaLuaRuntime {
    /// Create a new runtime from a TranspileContext with default configuration.
    ///
    /// Initializes a Lua VM with standard libraries enabled and registers:
    /// - `@pasta_search` module with scene and word registries
    /// - `@assertions` module for testing and validation
    /// - `@testing` module for testing framework with hooks and reporting
    /// - `@regex` module for regular expression support
    /// - `@json` module for JSON encoding/decoding
    /// - `@yaml` module for YAML encoding/decoding
    /// - `@pasta_log` module for the Lua→Rust tracing log bridge
    ///   (always registered, independent of `RuntimeConfig::libs`)
    ///
    /// Note: `@env` module is disabled by default for security reasons.
    /// Use `RuntimeConfig::full()` or enable it explicitly to access environment variables.
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
        // No source map on the bare `with_config` entry point: callers that build
        // the aggregated map (the loader, task 4.4) go through
        // `with_config_and_source_map`. Passing `None` keeps the existing default
        // `.lua` behavior (requirements 6.2 / 7.2) for ad-hoc / test runtimes.
        Self::with_config_and_source_map(context, config, None)
    }

    /// Like [`with_config`](Self::with_config) but additionally HANDS the
    /// aggregated `.pasta`↔`.lua` source map to the debug enable choke point
    /// (task 4.4).
    ///
    /// This is the runtime side of the design "runtime/mod.rs 168" handoff: the
    /// loader builds the aggregated `Arc<SourceMap>` AFTER transpile (only when
    /// debugging is enabled — task 4.3) and passes it here; this function holds it
    /// at runtime scope and forwards it to [`crate::debug::enable`] together with
    /// `config.debug` (which carries the resolved `source_mode`). The order is
    /// therefore transpile/load → build map → enable-with-map (requirement 3.1).
    ///
    /// When `source_map` is `None` (debug disabled, or an ad-hoc runtime), this is
    /// byte-for-byte the previous behavior: `enable` receives `None`, builds no
    /// map wiring, and the disabled gate stays zero-cost (requirements 6.2 / 7.1 /
    /// 7.2).
    pub(crate) fn with_config_and_source_map(
        context: TranspileContext,
        config: RuntimeConfig,
        source_map: Option<Arc<SourceMap>>,
    ) -> LuaResult<Self> {
        // Validate configuration and emit warnings
        config.validate_and_warn();

        // Convert libs array to StdLib flags
        let std_lib = config
            .to_stdlib()
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Create Lua VM with appropriate standard libraries
        // SAFETY: `Lua::unsafe_new_with` is required because mlua's safe `new` does not
        // accept custom StdLib flags. The safety invariants are upheld because:
        //  1. `std_lib` is constructed from validated `RuntimeConfig` via `to_stdlib()`,
        //     which only maps known library names to mlua `StdLib` flags.
        //  2. `validate_and_warn()` above alerts on dangerous libraries (debug/ffi).
        //  3. Default LuaOptions are used, so no custom allocator or hook is involved.
        //  4. The returned `Lua` handle is used in a single-threaded context and is not
        //     shared across threads.
        let lua = unsafe { Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) };

        // Extract registries from context
        let scene_registry = context.scene_registry;
        let word_registry = context.word_registry;

        // Register @pasta_search module
        crate::search::register(&lua, scene_registry, word_registry)?;

        // Register mlua-stdlib modules based on configuration
        if config.should_enable_module("assertions") {
            mlua_stdlib::assertions::register(&lua, None)?;
        }
        if config.should_enable_module("testing") {
            mlua_stdlib::testing::register(&lua, None)?;
        }
        if config.should_enable_module("env") {
            mlua_stdlib::env::register(&lua, None)?;
        }
        if config.should_enable_module("regex") {
            mlua_stdlib::regex::register(&lua, None)?;
        }
        if config.should_enable_module("json") {
            mlua_stdlib::json::register(&lua, None)?;
        }
        if config.should_enable_module("yaml") {
            mlua_stdlib::yaml::register(&lua, None)?;
        }

        // Register @pasta_log module (always available, independent of RuntimeConfig.libs)
        Self::register_log_module(&lua)?;

        // Debug backend gate (task 4.2): the SINGLE enable choke point. After the
        // VM is constructed and all modules are registered, install the debug hook
        // exactly ONCE iff `config.debug.enabled`. When disabled (the default),
        // `enable` returns `Ok(None)`: no hook, no port, no thread, no `std_debug`
        // exposure, no `jit.off()` — true zero cost (R5.2 / R5.3 / R5.5). The
        // returned handle lives at runtime scope so the `DebugSession` /
        // `BreakpointSet` / transport persist across requests (design
        // "セッションライフサイクル"). `DebugError` (`!Send`-safe, stringified) is
        // mapped to `mlua::Error` at this boundary.
        // Task 4.4: HAND the aggregated `.pasta` source map (built by the loader
        // AFTER transpile, only when debugging is enabled) to the enable choke
        // point. `enable` clones the `Arc` into the backend wiring/session when
        // `config.debug.source_mode == Pasta`; the disabled gate / `Lua` mode /
        // `None` map keeps the default `.lua` behavior (requirements 6.1 / 6.2 /
        // 7.2). We keep a runtime-scope clone in `self.source_map` so the held map
        // outlives a single request (requirement 3.1).
        // pasta-scene-kick tasks 1.1 / 2.3: thread the host-injected kick sink
        // (`RuntimeConfig.kick_sink`) through to the debug backend so an inbound
        // `pasta/playScene` invokes it (R2.4). When debug is disabled or no sink
        // was bound, the kick path stays inert (R2.6). Cloning is a refcount bump.
        let debug_handle = crate::debug::enable(
            &lua,
            &config.debug,
            source_map.clone(),
            config.kick_sink.clone(),
        )
        .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Only hold the map when the backend is actually active (debug enabled).
        // On the disabled zero-cost path nothing is built nor held (7.1).
        let source_map = if debug_handle.is_some() {
            source_map
        } else {
            None
        };

        Ok(Self {
            lua,
            logger: None,
            config: None,
            base_dir: None,
            debug_handle,
            source_map,
        })
    }
}
