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
/// Finalize module - Collects Lua-side registries and builds SearchContext.
pub mod finalize;
/// Log module - Lua logging bridge to Rust tracing infrastructure.
pub mod log;
mod module_registry;
/// Persistence module - Persistent data storage for Lua scripts.
pub mod persistence;
mod runtime_config;

pub use runtime_config::lua_require;
pub use runtime_config::RuntimeConfig;

use crate::context::TranspileContext;
use crate::loader::{LoaderContext, PastaConfig, TranspileResult};
use crate::logging::PastaLogger;
pub(crate) use finalize::register_finalize_scene;
use mlua::{Lua, LuaSerdeExt, Result as LuaResult, Table, Value};
use std::path::{Path, PathBuf};
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

        Ok(Self {
            lua,
            logger: None,
            config: None,
            base_dir: None,
        })
    }

    /// Execute a Lua script string.
    ///
    /// # Arguments
    /// * `script` - Lua code to execute
    ///
    /// # Returns
    /// * `Ok(Value)` - Execution result
    /// * `Err(e)` - Execution error
    ///
    /// # Safety (input source analysis)
    /// All callers of `exec()` pass either:
    /// - Transpiled Lua code generated by `LuaTranspiler` from validated `.pasta` files
    /// - Hardcoded Rust string literals in test code
    ///   No external user input is passed directly to this method.
    ///   Errors are propagated via `LuaResult`, not unwrapped.
    pub fn exec(&self, script: &str) -> LuaResult<Value> {
        self.lua.load(script).eval()
    }

    /// Execute a Lua script from a file.
    ///
    /// # Arguments
    /// * `path` - Path to the Lua script file
    ///
    /// # Returns
    /// * `Ok(Value)` - Execution result
    /// * `Err(e)` - File read or execution error
    ///
    /// # Safety (input source analysis)
    /// The file is read from a trusted local path supplied by Rust caller code.
    /// Delegates to `exec()` which propagates errors via `LuaResult`.
    pub fn exec_file(&self, path: &Path) -> LuaResult<Value> {
        let script =
            std::fs::read_to_string(path).map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
        self.exec(&script)
    }

    /// Get a reference to the internal Lua instance.
    ///
    /// This allows advanced operations on the Lua VM.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get a clone of the Arc-wrapped logger, if any.
    ///
    /// This allows sharing the logger with GlobalLoggerRegistry
    /// for log routing in multi-instance scenarios.
    pub fn logger(&self) -> Option<Arc<PastaLogger>> {
        self.logger.clone()
    }

    /// Get reference to PastaConfig if available.
    ///
    /// Returns the configuration loaded from pasta.toml during PastaLoader::load().
    ///
    /// # Returns
    /// * `Some(&PastaConfig)` - Config was set during load
    /// * `None` - Config not available (e.g., runtime created without loader)
    pub fn config(&self) -> Option<&PastaConfig> {
        self.config.as_ref()
    }

    /// Register a custom module with the runtime.
    ///
    /// # Arguments
    /// * `name` - Module name (e.g., "@my_module")
    /// * `module` - Module table
    ///
    /// # Returns
    /// * `Ok(())` - Module registered successfully
    /// * `Err(e)` - Registration failed
    pub fn register_module(&self, name: &str, module: Table) -> LuaResult<()> {
        let package: Table = self.lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set(name, module)?;
        Ok(())
    }

    /// Create a runtime from LoaderContext with transpiled code.
    ///
    /// This is the factory method used by PastaLoader to create a runtime
    /// with all configuration applied.
    ///
    /// # Arguments
    /// * `context` - TranspileContext with scene/word registries
    /// * `loader_context` - Configuration and paths from PastaLoader
    /// * `config` - Runtime configuration
    /// * `transpiled` - Transpiled Lua code to load
    /// * `logger` - Optional instance-specific logger (Arc-wrapped for sharing)
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized and code loaded
    /// * `Err(e)` - Initialization or code loading failed
    pub fn from_loader(
        context: TranspileContext,
        loader_context: LoaderContext,
        config: RuntimeConfig,
        transpiled: &[TranspileResult],
        logger: Option<Arc<PastaLogger>>,
    ) -> LuaResult<Self> {
        // Create base runtime
        let mut runtime = Self::with_config(context, config)?;

        // Set logger if provided
        runtime.logger = logger;

        // Setup package.path for module resolution
        Self::setup_package_path(&runtime.lua, &loader_context)?;

        // Register @pasta_config module
        Self::register_config_module(&runtime.lua, &loader_context.custom_fields)?;

        // Register @enc module for encoding conversion
        Self::register_enc_module(&runtime.lua)?;

        // Load transpiled code directly into memory
        // SAFETY(injection): `lua_code` is generated by LuaTranspiler from validated
        // .pasta source files, not from raw external input. `module_name` is derived
        // from file paths during transpilation. Errors are propagated via `?`.
        for result in transpiled {
            runtime
                .lua
                .load(&result.lua_code)
                .set_name(&result.module_name)
                .exec()?;
            tracing::debug!(module = %result.module_name, "Loaded transpiled module");
        }

        // Load entry.lua if exists (for SHIORI.load/SHIORI.request functions)
        // SAFETY(injection): Script is read from a deterministic local path
        // (base_dir/scripts/pasta/shiori/entry.lua), not from external input.
        // Errors are caught and logged as warnings.
        let entry_lua_path = loader_context
            .base_dir
            .join("scripts/pasta/shiori/entry.lua");
        if entry_lua_path.exists() {
            match std::fs::read_to_string(&entry_lua_path) {
                Ok(script) => {
                    if let Err(e) = runtime.lua.load(&script).set_name("entry.lua").exec() {
                        tracing::warn!(error = %e, "Failed to load entry.lua, continuing without SHIORI functions");
                    } else {
                        tracing::debug!("Loaded entry.lua");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to read entry.lua, continuing without SHIORI functions");
                }
            }
        }

        Ok(runtime)
    }

    /// Create a runtime from LoaderContext with scene_dic.lua loading.
    ///
    /// This is the new factory method used by PastaLoader with incremental transpilation.
    /// Instead of loading transpiled code directly, it loads scene_dic.lua which
    /// requires all cached scene modules.
    ///
    /// # Initialization Sequence (lua-module-path-resolution spec)
    /// 1. Setup package.path for module resolution
    /// 2. Register Rust modules (@pasta_config, @enc, @pasta_persistence, @pasta_sakura_script)
    /// 3. Register finalize_scene Rust binding
    /// 4. require("main") - User initialization (errors logged as warnings, continues)
    /// 5. require("pasta.shiori.entry") - SHIORI handlers (errors logged as warnings, continues)
    /// 6. require("pasta.scene_dic") - Scene loading and finalization
    ///
    /// # Arguments
    /// * `context` - TranspileContext with scene/word registries
    /// * `loader_context` - Configuration and paths from PastaLoader
    /// * `runtime_config` - Runtime configuration
    /// * `pasta_config` - Pasta configuration from pasta.toml
    /// * `logger` - Optional instance-specific logger (Arc-wrapped for sharing)
    /// * `scene_dic_path` - Path to the generated scene_dic.lua (used for backward compatibility check)
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized and scene_dic loaded
    /// * `Err(e)` - Initialization or scene_dic loading failed
    pub fn from_loader_with_scene_dic(
        context: TranspileContext,
        loader_context: LoaderContext,
        runtime_config: RuntimeConfig,
        pasta_config: Option<PastaConfig>,
        logger: Option<Arc<PastaLogger>>,
        _scene_dic_path: &Path,
    ) -> LuaResult<Self> {
        // Create base runtime
        let mut runtime = Self::with_config(context, runtime_config)?;

        // Set logger if provided
        runtime.logger = logger;

        // Store config and base_dir for Drop-time persistence save
        runtime.base_dir = Some(loader_context.base_dir.clone());
        runtime.config = pasta_config;

        // Setup package.path for module resolution
        Self::setup_package_path(&runtime.lua, &loader_context)?;

        // Register @pasta_config module
        Self::register_config_module(&runtime.lua, &loader_context.custom_fields)?;

        // Register @enc module for encoding conversion
        Self::register_enc_module(&runtime.lua)?;

        // Register @pasta_persistence module for persistent data storage
        Self::register_persistence_module(&runtime.lua, &runtime.config, &runtime.base_dir)?;

        // Register @pasta_log module for Lua logging bridge
        Self::register_log_module(&runtime.lua)?;

        // Register @pasta_sakura_script module for wait insertion
        Self::register_sakura_script_module(&runtime.lua, &runtime.config)?;

        // Register finalize_scene Rust binding to overwrite Lua stub (Requirement 4.3)
        // This must be done before loading scene_dic.lua which calls finalize_scene()
        register_finalize_scene(&runtime.lua)?;

        // ========================================
        // Module Loading Phase (all require-based)
        // ========================================

        // Step 4: require("main") - User initialization script
        // Runs before scene_dic finalization to allow dictionary registration
        // SAFETY(injection): All module names below are compile-time string literals.
        // Errors from "main" and "pasta.shiori.entry" are logged as warnings and
        // do not halt execution. "pasta.scene_dic" error is propagated via `?`.
        if let Err(e) = lua_require(&runtime.lua, "main") {
            tracing::warn!(error = %e, "Failed to load main.lua, continuing without user initialization");
        } else {
            tracing::debug!(module = "main", "Loaded module via require");
        }

        // Step 5: require("pasta.shiori.entry") - SHIORI handlers
        if let Err(e) = lua_require(&runtime.lua, "pasta.shiori.entry") {
            tracing::warn!(error = %e, "Failed to load pasta.shiori.entry, continuing without SHIORI functions");
        } else {
            tracing::debug!(module = "pasta.shiori.entry", "Loaded module via require");
        }

        // Step 6: require("pasta.scene_dic") - Scene loading and finalization
        // This triggers SearchContext construction from Lua-side registries
        lua_require(&runtime.lua, "pasta.scene_dic")?;
        tracing::debug!(module = "pasta.scene_dic", "Loaded module via require");

        Ok(runtime)
    }

    /// Load scene_dic.lua to initialize all scene modules.
    ///
    /// This method requires the scene_dic.lua file which in turn requires
    /// all cached scene modules and calls finalize_scene().
    ///
    /// # Arguments
    /// * `scene_dic_path` - Path to scene_dic.lua
    ///
    /// # Returns
    /// * `Ok(())` - All scenes loaded successfully
    /// * `Err(e)` - Scene loading failed
    pub fn load_scene_dic(&self, scene_dic_path: &Path) -> LuaResult<()> {
        // Read the scene_dic.lua file
        // SAFETY(injection): Script is read from a generated scene_dic.lua file
        // produced by the transpiler, not from external user input.
        // Errors are propagated via `?`.
        let script = std::fs::read_to_string(scene_dic_path)
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Execute the scene_dic.lua
        self.lua.load(&script).set_name("pasta.scene_dic").exec()?;

        tracing::info!(path = %scene_dic_path.display(), "Loaded scene_dic.lua");
        Ok(())
    }

    /// Save persistence data from ctx.save.
    ///
    /// Called automatically on Drop to save any modified persistent data.
    fn save_persistence_data(&self) -> Result<(), persistence::PersistenceError> {
        // Get persistence config
        let persistence_config = self
            .config
            .as_ref()
            .and_then(|c| c.persistence())
            .unwrap_or_default();

        let base_dir = self.base_dir.as_deref().unwrap_or(Path::new("."));
        let file_path = base_dir.join(persistence_config.effective_file_path());

        // Try to get save from Lua
        // SAFETY(injection): Module name is a compile-time string literal.
        // The match handles the error case gracefully (early return Ok).
        let save_table: Table = match self.lua.load(r#"require("pasta.save")"#).eval() {
            Ok(t) => t,
            Err(e) => {
                // save might not exist if runtime wasn't fully initialized
                tracing::debug!(error = %e, "Could not access pasta.save, skipping persistence save");
                return Ok(());
            }
        };

        // Convert Lua table to serde_json::Value
        let lua_value = Value::Table(save_table);
        let json_value: serde_json::Value = self
            .lua
            .from_value(lua_value)
            .map_err(|e| persistence::PersistenceError::LuaConversionError(e.to_string()))?;

        // Save to file
        persistence::save_to_file(&json_value, &file_path, persistence_config.obfuscate)?;

        if persistence_config.debug_mode {
            tracing::debug!(path = %file_path.display(), "Saved persistence data on drop");
        }

        Ok(())
    }
}

impl Drop for PastaLuaRuntime {
    fn drop(&mut self) {
        // Save persistence data (errors are logged, not propagated)
        if let Err(e) = self.save_persistence_data() {
            tracing::error!(error = %e, "Failed to save persistence data on drop");
        }
    }
}

