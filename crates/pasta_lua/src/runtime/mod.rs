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

use crate::context::TranspileContext;
use crate::loader::{LoaderContext, TranspileResult};
use crate::logging::PastaLogger;
use mlua::{Lua, Result as LuaResult, StdLib, Table, Value};
use std::path::Path;
use std::sync::Arc;

/// Configuration for which standard libraries to enable in the Lua runtime.
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    /// Enable Lua standard libraries (base, table, string, math, etc.)
    /// Default: true
    pub enable_std_libs: bool,
    /// Enable mlua-stdlib assertions module (@assertions)
    /// Provides assertion functions for testing and validation.
    /// Default: true
    pub enable_assertions: bool,
    /// Enable mlua-stdlib testing module (@testing)
    /// Provides a testing framework with hooks and reporting.
    /// Default: true
    pub enable_testing: bool,
    /// Enable mlua-stdlib env module (@env)
    /// Provides environment variable and filesystem path access.
    /// Default: false (security-sensitive, opt-in)
    pub enable_env: bool,
    /// Enable mlua-stdlib regex module (@regex)
    /// Default: true
    pub enable_regex: bool,
    /// Enable mlua-stdlib json module (@json)
    /// Default: true
    pub enable_json: bool,
    /// Enable mlua-stdlib yaml module (@yaml)
    /// Default: true
    pub enable_yaml: bool,
}

impl RuntimeConfig {
    /// Create a new configuration with all safe features enabled (default).
    ///
    /// Note: `enable_env` is disabled by default for security reasons.
    pub fn new() -> Self {
        Self {
            enable_std_libs: true,
            enable_assertions: true,
            enable_testing: true,
            enable_env: false, // Disabled by default for security
            enable_regex: true,
            enable_json: true,
            enable_yaml: true,
        }
    }

    /// Create a configuration with all features enabled, including security-sensitive ones.
    pub fn full() -> Self {
        Self {
            enable_std_libs: true,
            enable_assertions: true,
            enable_testing: true,
            enable_env: true,
            enable_regex: true,
            enable_json: true,
            enable_yaml: true,
        }
    }

    /// Create a minimal configuration with only pasta modules.
    pub fn minimal() -> Self {
        Self {
            enable_std_libs: false,
            enable_assertions: false,
            enable_testing: false,
            enable_env: false,
            enable_regex: false,
            enable_json: false,
            enable_yaml: false,
        }
    }
}

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
        // Create Lua VM with appropriate standard libraries
        let lua = if config.enable_std_libs {
            // Load all safe standard libraries (excluding debug and ffi)
            // SAFETY: ALL_SAFE only loads safe standard libraries
            unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) }
        } else {
            Lua::new()
        };

        // Extract registries from context
        let scene_registry = context.scene_registry;
        let word_registry = context.word_registry;

        // Register @pasta_search module
        crate::search::register(&lua, scene_registry, word_registry)?;

        // Register mlua-stdlib core modules based on configuration
        if config.enable_assertions {
            mlua_stdlib::assertions::register(&lua, None)?;
        }
        if config.enable_testing {
            mlua_stdlib::testing::register(&lua, None)?;
        }
        if config.enable_env {
            mlua_stdlib::env::register(&lua, None)?;
        }

        // Register mlua-stdlib feature-gated modules
        if config.enable_regex {
            mlua_stdlib::regex::register(&lua, None)?;
        }
        if config.enable_json {
            mlua_stdlib::json::register(&lua, None)?;
        }
        if config.enable_yaml {
            mlua_stdlib::yaml::register(&lua, None)?;
        }

        Ok(Self { lua, logger: None })
    }

    /// Execute a Lua script string.
    ///
    /// # Arguments
    /// * `script` - Lua code to execute
    ///
    /// # Returns
    /// * `Ok(Value)` - Execution result
    /// * `Err(e)` - Execution error
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
        for result in transpiled {
            runtime
                .lua
                .load(&result.lua_code)
                .set_name(&result.module_name)
                .exec()?;
            tracing::debug!(module = %result.module_name, "Loaded transpiled module");
        }

        // Load main.lua if exists (for SHIORI.load/SHIORI.request functions)
        let main_lua_path = loader_context
            .base_dir
            .join("scripts/pasta/shiori/main.lua");
        if main_lua_path.exists() {
            match std::fs::read_to_string(&main_lua_path) {
                Ok(script) => {
                    if let Err(e) = runtime.lua.load(&script).set_name("main.lua").exec() {
                        tracing::warn!(error = %e, "Failed to load main.lua, continuing without SHIORI functions");
                    } else {
                        tracing::debug!("Loaded main.lua");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to read main.lua, continuing without SHIORI functions");
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
    /// # Arguments
    /// * `context` - TranspileContext with scene/word registries
    /// * `loader_context` - Configuration and paths from PastaLoader
    /// * `config` - Runtime configuration
    /// * `logger` - Optional instance-specific logger (Arc-wrapped for sharing)
    /// * `scene_dic_path` - Path to the generated scene_dic.lua
    ///
    /// # Returns
    /// * `Ok(Self)` - Runtime initialized and scene_dic loaded
    /// * `Err(e)` - Initialization or scene_dic loading failed
    pub fn from_loader_with_scene_dic(
        context: TranspileContext,
        loader_context: LoaderContext,
        config: RuntimeConfig,
        logger: Option<Arc<PastaLogger>>,
        scene_dic_path: &Path,
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

        // Load main.lua first (for SHIORI.load/SHIORI.request functions)
        let main_lua_path = loader_context
            .base_dir
            .join("scripts/pasta/shiori/main.lua");
        if main_lua_path.exists() {
            match std::fs::read_to_string(&main_lua_path) {
                Ok(script) => {
                    if let Err(e) = runtime.lua.load(&script).set_name("main.lua").exec() {
                        tracing::warn!(error = %e, "Failed to load main.lua, continuing without SHIORI functions");
                    } else {
                        tracing::debug!("Loaded main.lua");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to read main.lua, continuing without SHIORI functions");
                }
            }
        }

        // Load scene_dic.lua to require all cached scene modules
        tracing::debug!(path = %scene_dic_path.display(), "Loading scene_dic.lua");
        runtime.load_scene_dic(scene_dic_path)?;

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
        let script = std::fs::read_to_string(scene_dic_path)
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        // Execute the scene_dic.lua
        self.lua.load(&script).set_name("pasta.scene_dic").exec()?;

        tracing::info!(path = %scene_dic_path.display(), "Loaded scene_dic.lua");
        Ok(())
    }

    /// Setup package.path for Lua module resolution.
    ///
    /// Sets the package.path to include all search paths from LoaderContext
    /// in priority order (first path has highest priority).
    ///
    /// On Windows, the path is converted to ANSI encoding to ensure
    /// Lua's file I/O functions (which use fopen) can resolve non-ASCII paths.
    fn setup_package_path(lua: &Lua, loader_context: &LoaderContext) -> LuaResult<()> {
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
    fn register_config_module(lua: &Lua, custom_fields: &toml::Table) -> LuaResult<()> {
        let config_table = Self::toml_to_lua(lua, &toml::Value::Table(custom_fields.clone()))?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@pasta_config", config_table)?;

        tracing::debug!("Registered @pasta_config module");
        Ok(())
    }

    /// Register @enc module for encoding conversion.
    ///
    /// Provides UTF-8 <-> ANSI conversion functions for Lua scripts.
    fn register_enc_module(lua: &Lua) -> LuaResult<()> {
        let enc_table = enc::register(lua)?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@enc", enc_table)?;

        tracing::debug!("Registered @enc module");
        Ok(())
    }

    /// Convert toml::Value to mlua::Value.
    ///
    /// Recursively converts TOML structures to Lua tables.
    fn toml_to_lua(lua: &Lua, value: &toml::Value) -> LuaResult<Value> {
        match value {
            toml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
            toml::Value::Integer(i) => Ok(Value::Integer(*i)),
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
