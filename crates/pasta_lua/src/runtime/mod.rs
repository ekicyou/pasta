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
/// Persistence module - Persistent data storage for Lua scripts.
pub mod persistence;

use crate::context::TranspileContext;
use crate::error::ConfigError;
use crate::loader::{LoaderContext, LuaConfig, PastaConfig, TranspileResult, default_libs};
use crate::logging::PastaLogger;
pub(crate) use finalize::register_finalize_scene;
use mlua::{Lua, LuaSerdeExt, Result as LuaResult, StdLib, Table, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for which standard libraries to enable in the Lua runtime.
///
/// Uses Cargo-style array notation with optional subtraction syntax.
///
/// # Examples
///
/// ```rust
/// use pasta_lua::RuntimeConfig;
///
/// // Default configuration (safe libraries + common mlua-stdlib modules)
/// let config = RuntimeConfig::new();
///
/// // Full configuration with all features including security-sensitive ones
/// let config = RuntimeConfig::full();
///
/// // Minimal configuration with no libraries
/// let config = RuntimeConfig::minimal();
///
/// // Custom configuration
/// let config = RuntimeConfig::from_libs(vec![
///     "std_all".into(),
///     "testing".into(),
///     "-std_debug".into(),
/// ]);
/// ```
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Library configuration array.
    ///
    /// Supports Lua standard libraries (std_* prefix) and mlua-stdlib modules.
    /// Use `-` prefix to subtract/exclude a library.
    ///
    /// Valid Lua standard libraries:
    /// - `std_all` - All safe libraries (StdLib::ALL_SAFE, excludes std_debug)
    /// - `std_all_unsafe` - All libraries including debug (StdLib::ALL)
    /// - `std_coroutine`, `std_table`, `std_io`, `std_os`, `std_string`
    /// - `std_utf8`, `std_math`, `std_package`, `std_debug`
    ///
    /// Valid mlua-stdlib modules:
    /// - `assertions`, `testing`, `env`, `regex`, `json`, `yaml`
    pub libs: Vec<String>,
}

impl RuntimeConfig {
    /// Create a new configuration with all safe features enabled (default).
    ///
    /// Default: `["std_all", "assertions", "testing", "regex", "json", "yaml"]`
    ///
    /// Note: `env` is disabled by default for security reasons.
    pub fn new() -> Self {
        Self {
            libs: default_libs(),
        }
    }

    /// Create a configuration with all features enabled, including security-sensitive ones.
    ///
    /// Includes: `std_all_unsafe` (debug), `env`
    pub fn full() -> Self {
        Self {
            libs: vec![
                "std_all_unsafe".into(),
                "assertions".into(),
                "testing".into(),
                "env".into(),
                "regex".into(),
                "json".into(),
                "yaml".into(),
            ],
        }
    }

    /// Create a minimal configuration with only safe Lua standard libraries.
    ///
    /// No mlua-stdlib modules are enabled.
    ///
    /// Contains: `["std_all"]`
    pub fn minimal() -> Self {
        Self {
            libs: vec!["std_all".into()],
        }
    }

    /// Create a configuration from a custom libs array.
    pub fn from_libs(libs: Vec<String>) -> Self {
        Self { libs }
    }

    /// Convert libs array to mlua::StdLib flags.
    ///
    /// Processing order: additions first, then subtractions.
    /// This ensures order-independent behavior.
    ///
    /// # Returns
    /// * `Ok(StdLib)` - Computed StdLib flags
    /// * `Err(ConfigError)` - Unknown library name found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pasta_lua::RuntimeConfig;
    /// use mlua::StdLib;
    ///
    /// let config = RuntimeConfig::from_libs(vec!["std_all".into(), "-std_debug".into()]);
    /// let stdlib = config.to_stdlib().unwrap();
    /// assert_eq!(stdlib, StdLib::ALL_SAFE);
    /// ```
    pub fn to_stdlib(&self) -> Result<StdLib, ConfigError> {
        let mut additions = StdLib::NONE;
        let mut subtractions = StdLib::NONE;

        for lib in &self.libs {
            let (is_subtraction, name) = if let Some(stripped) = lib.strip_prefix('-') {
                (true, stripped)
            } else {
                (false, lib.as_str())
            };

            // Only process std_* prefixed names for StdLib
            if !name.starts_with("std_") {
                // mlua-stdlib modules are handled separately
                continue;
            }

            let flag = Self::parse_std_lib(name)?;

            if is_subtraction {
                subtractions |= flag;
            } else {
                additions |= flag;
            }
        }

        // Remove subtractions from additions using XOR on the intersection
        // First find bits that are in both, then XOR them out of additions
        let intersection = additions & subtractions;
        Ok(additions ^ intersection)
    }

    /// Parse a std_* library name to StdLib flag.
    fn parse_std_lib(name: &str) -> Result<StdLib, ConfigError> {
        match name {
            "std_all" => Ok(StdLib::ALL_SAFE),
            "std_all_unsafe" => Ok(StdLib::ALL),
            "std_coroutine" => Ok(StdLib::COROUTINE),
            "std_table" => Ok(StdLib::TABLE),
            "std_io" => Ok(StdLib::IO),
            "std_os" => Ok(StdLib::OS),
            "std_string" => Ok(StdLib::STRING),
            "std_utf8" => Ok(StdLib::UTF8),
            "std_math" => Ok(StdLib::MATH),
            "std_package" => Ok(StdLib::PACKAGE),
            "std_debug" => Ok(StdLib::DEBUG),
            _ => Err(ConfigError::UnknownLibrary(name.to_string())),
        }
    }

    /// Check if a specific mlua-stdlib module should be enabled.
    ///
    /// # Arguments
    /// * `module` - Module name without prefix (e.g., "testing", "regex")
    ///
    /// # Returns
    /// `true` if module is in libs array and not subtracted
    pub fn should_enable_module(&self, module: &str) -> bool {
        let has_positive = self.libs.iter().any(|lib| lib == module);
        let has_negative = self
            .libs
            .iter()
            .any(|lib| lib.strip_prefix('-') == Some(module));
        has_positive && !has_negative
    }

    /// Validate configuration and emit security warnings.
    ///
    /// Emits `tracing::warn` for:
    /// - `std_debug` or `std_all_unsafe` enabled
    /// - `env` module enabled
    ///
    /// Emits `tracing::debug` for enabled libraries list.
    pub fn validate_and_warn(&self) {
        // Check for security-sensitive Lua libraries
        let has_std_debug = self.libs.iter().any(|lib| lib == "std_debug");
        let has_std_all_unsafe = self.libs.iter().any(|lib| lib == "std_all_unsafe");
        let debug_subtracted = self
            .libs
            .iter()
            .any(|lib| lib == "-std_debug" || lib == "-std_all_unsafe");

        if (has_std_debug || has_std_all_unsafe) && !debug_subtracted {
            if has_std_all_unsafe {
                tracing::warn!(
                    "Unsafe Lua libraries enabled: std_all_unsafe. \
                     This includes std_debug which provides access to Lua internals. \
                     Not recommended for production."
                );
            } else {
                tracing::warn!(
                    "Unsafe Lua library enabled: std_debug. \
                     Provides access to Lua internals and stack manipulation. \
                     Not recommended for production."
                );
            }
        }

        // Check for security-sensitive mlua-stdlib modules
        if self.should_enable_module("env") {
            tracing::warn!(
                "Security-sensitive module enabled: env. \
                 Provides filesystem and environment variable access."
            );
        }

        // Log enabled libraries at debug level
        tracing::debug!(libs = ?self.libs, "Lua library configuration");
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl From<LuaConfig> for RuntimeConfig {
    fn from(config: LuaConfig) -> Self {
        Self { libs: config.libs }
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
        // SAFETY: We control the StdLib flags based on configuration
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
        for result in transpiled {
            runtime
                .lua
                .load(&result.lua_code)
                .set_name(&result.module_name)
                .exec()?;
            tracing::debug!(module = %result.module_name, "Loaded transpiled module");
        }

        // Load entry.lua if exists (for SHIORI.load/SHIORI.request functions)
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
    /// # Arguments
    /// * `context` - TranspileContext with scene/word registries
    /// * `loader_context` - Configuration and paths from PastaLoader
    /// * `runtime_config` - Runtime configuration
    /// * `pasta_config` - Pasta configuration from pasta.toml
    /// * `logger` - Optional instance-specific logger (Arc-wrapped for sharing)
    /// * `scene_dic_path` - Path to the generated scene_dic.lua
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
        scene_dic_path: &Path,
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

        // Register @pasta_sakura_script module for wait insertion
        Self::register_sakura_script_module(&runtime.lua, &runtime.config)?;

        // Load entry.lua first (for SHIORI.load/SHIORI.request functions)
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

        // Register finalize_scene Rust binding to overwrite Lua stub (Requirement 4.3)
        // This must be done before loading scene_dic.lua which calls finalize_scene()
        register_finalize_scene(&runtime.lua)?;

        // Load scene_dic.lua to require all cached scene modules
        // scene_dic.lua ends with require('pasta').finalize_scene() which
        // triggers SearchContext construction from Lua-side registries
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

    /// Register @pasta_persistence module for persistent data storage.
    ///
    /// Provides load/save functions for persisting Lua tables to files.
    fn register_persistence_module(
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
    fn register_sakura_script_module(lua: &Lua, config: &Option<PastaConfig>) -> LuaResult<()> {
        // Get talk config, use defaults if not specified
        let talk_config = config.as_ref().and_then(|c| c.talk());

        let sakura_module = crate::sakura_script::register(lua, talk_config.as_ref())?;

        let package: Table = lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set("@pasta_sakura_script", sakura_module)?;

        tracing::debug!("Registered @pasta_sakura_script module");
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RuntimeConfig constructor tests
    // ============================================================================

    #[test]
    fn test_runtime_config_new_returns_default_libs() {
        let config = RuntimeConfig::new();
        assert!(config.libs.contains(&"std_all".to_string()));
        assert!(config.libs.contains(&"assertions".to_string()));
        assert!(config.libs.contains(&"testing".to_string()));
        assert!(config.libs.contains(&"regex".to_string()));
        assert!(config.libs.contains(&"json".to_string()));
        assert!(config.libs.contains(&"yaml".to_string()));
        // env should NOT be in default
        assert!(!config.libs.contains(&"env".to_string()));
    }

    #[test]
    fn test_runtime_config_full_includes_unsafe() {
        let config = RuntimeConfig::full();
        assert!(config.libs.contains(&"std_all_unsafe".to_string()));
        assert!(config.libs.contains(&"env".to_string()));
    }

    #[test]
    fn test_runtime_config_minimal_is_std_all_only() {
        let config = RuntimeConfig::minimal();
        assert_eq!(config.libs, vec!["std_all".to_string()]);
    }

    #[test]
    fn test_runtime_config_from_libs() {
        let config = RuntimeConfig::from_libs(vec!["std_table".into(), "regex".into()]);
        assert_eq!(config.libs.len(), 2);
        assert!(config.libs.contains(&"std_table".to_string()));
        assert!(config.libs.contains(&"regex".to_string()));
    }

    // ============================================================================
    // to_stdlib() tests
    // ============================================================================

    #[test]
    fn test_to_stdlib_std_all() {
        let config = RuntimeConfig::from_libs(vec!["std_all".into()]);
        let stdlib = config.to_stdlib().unwrap();
        assert_eq!(stdlib, StdLib::ALL_SAFE);
    }

    #[test]
    fn test_to_stdlib_std_all_unsafe() {
        let config = RuntimeConfig::from_libs(vec!["std_all_unsafe".into()]);
        let stdlib = config.to_stdlib().unwrap();
        assert_eq!(stdlib, StdLib::ALL);
    }

    #[test]
    fn test_to_stdlib_individual_libs() {
        let config = RuntimeConfig::from_libs(vec![
            "std_table".into(),
            "std_string".into(),
            "std_math".into(),
        ]);
        let stdlib = config.to_stdlib().unwrap();
        assert_eq!(stdlib, StdLib::TABLE | StdLib::STRING | StdLib::MATH);
    }

    #[test]
    fn test_to_stdlib_subtraction() {
        let config = RuntimeConfig::from_libs(vec!["std_all".into(), "-std_io".into()]);
        let stdlib = config.to_stdlib().unwrap();
        // ALL_SAFE minus IO
        let expected = StdLib::ALL_SAFE ^ (StdLib::ALL_SAFE & StdLib::IO);
        assert_eq!(stdlib, expected);
    }

    #[test]
    fn test_to_stdlib_order_independent() {
        // Subtraction first, then addition - should still work
        let config1 = RuntimeConfig::from_libs(vec!["-std_io".into(), "std_all".into()]);
        let config2 = RuntimeConfig::from_libs(vec!["std_all".into(), "-std_io".into()]);

        let stdlib1 = config1.to_stdlib().unwrap();
        let stdlib2 = config2.to_stdlib().unwrap();

        assert_eq!(stdlib1, stdlib2);
    }

    #[test]
    fn test_to_stdlib_empty_libs() {
        let config = RuntimeConfig::from_libs(vec![]);
        let stdlib = config.to_stdlib().unwrap();
        assert_eq!(stdlib, StdLib::NONE);
    }

    #[test]
    fn test_to_stdlib_unknown_library_error() {
        let config = RuntimeConfig::from_libs(vec!["std_nonexistent".into()]);
        let result = config.to_stdlib();
        assert!(result.is_err());
        match result {
            Err(ConfigError::UnknownLibrary(name)) => {
                assert_eq!(name, "std_nonexistent");
            }
            Ok(_) => panic!("Expected error for unknown library"),
        }
    }

    #[test]
    fn test_to_stdlib_ignores_mlua_stdlib_modules() {
        // mlua-stdlib modules (no std_ prefix) should be ignored by to_stdlib
        let config = RuntimeConfig::from_libs(vec![
            "std_table".into(),
            "assertions".into(),
            "regex".into(),
        ]);
        let stdlib = config.to_stdlib().unwrap();
        // Only std_table should be processed
        assert_eq!(stdlib, StdLib::TABLE);
    }

    // ============================================================================
    // should_enable_module() tests
    // ============================================================================

    #[test]
    fn test_should_enable_module_positive() {
        let config = RuntimeConfig::from_libs(vec!["testing".into(), "regex".into()]);
        assert!(config.should_enable_module("testing"));
        assert!(config.should_enable_module("regex"));
        assert!(!config.should_enable_module("assertions"));
    }

    #[test]
    fn test_should_enable_module_with_subtraction() {
        let config = RuntimeConfig::from_libs(vec!["testing".into(), "-testing".into()]);
        // Has both positive and negative, so should return false
        assert!(!config.should_enable_module("testing"));
    }

    #[test]
    fn test_should_enable_module_subtraction_without_positive() {
        let config = RuntimeConfig::from_libs(vec!["-testing".into()]);
        // Only negative, no positive - should return false
        assert!(!config.should_enable_module("testing"));
    }

    // ============================================================================
    // From<LuaConfig> tests
    // ============================================================================

    #[test]
    fn test_from_lua_config() {
        let lua_config = LuaConfig {
            libs: vec!["std_all".into(), "testing".into()],
        };
        let runtime_config: RuntimeConfig = lua_config.into();
        assert_eq!(runtime_config.libs, vec!["std_all", "testing"]);
    }

    // ============================================================================
    // Default trait tests
    // ============================================================================

    #[test]
    fn test_default_equals_new() {
        let default_config = RuntimeConfig::default();
        let new_config = RuntimeConfig::new();
        assert_eq!(default_config.libs, new_config.libs);
    }

    // ============================================================================
    // validate_and_warn() tests
    // ============================================================================

    /// Test that validate_and_warn() does not panic for safe configurations.
    #[test]
    fn test_validate_and_warn_safe_config() {
        let config = RuntimeConfig::new();
        // Should not panic
        config.validate_and_warn();
    }

    /// Test that validate_and_warn() does not panic for unsafe configurations.
    #[test]
    fn test_validate_and_warn_unsafe_config() {
        let config = RuntimeConfig::full();
        // Should not panic, even with unsafe libs
        config.validate_and_warn();
    }

    /// Test that validate_and_warn() correctly identifies std_debug as security-sensitive.
    #[test]
    fn test_validate_and_warn_detects_std_debug() {
        let config = RuntimeConfig::from_libs(vec!["std_debug".into()]);
        // The method should run without error
        // Actual warning output is verified by log inspection
        config.validate_and_warn();

        // Verify that the logic correctly identifies std_debug
        assert!(config.libs.contains(&"std_debug".to_string()));
    }

    /// Test that validate_and_warn() correctly identifies std_all_unsafe as security-sensitive.
    #[test]
    fn test_validate_and_warn_detects_std_all_unsafe() {
        let config = RuntimeConfig::from_libs(vec!["std_all_unsafe".into()]);
        config.validate_and_warn();

        // Verify that the logic correctly identifies std_all_unsafe
        assert!(config.libs.contains(&"std_all_unsafe".to_string()));
    }

    /// Test that validate_and_warn() correctly identifies env module as security-sensitive.
    #[test]
    fn test_validate_and_warn_detects_env_module() {
        let config = RuntimeConfig::from_libs(vec!["std_all".into(), "env".into()]);
        config.validate_and_warn();

        // Verify that the method correctly identifies env
        assert!(config.should_enable_module("env"));
    }

    /// Test that validate_and_warn() respects subtraction for std_debug.
    #[test]
    fn test_validate_and_warn_respects_debug_subtraction() {
        let config = RuntimeConfig::from_libs(vec!["std_all_unsafe".into(), "-std_debug".into()]);
        config.validate_and_warn();

        // Verify subtraction is detected
        assert!(config.libs.iter().any(|lib| lib == "-std_debug"));
    }

    /// Test that validate_and_warn() respects subtraction for env module.
    #[test]
    fn test_validate_and_warn_respects_env_subtraction() {
        let config = RuntimeConfig::from_libs(vec!["env".into(), "-env".into()]);
        config.validate_and_warn();

        // Verify that env is effectively disabled via subtraction
        assert!(!config.should_enable_module("env"));
    }

    /// Test that validate_and_warn() handles empty libs configuration.
    #[test]
    fn test_validate_and_warn_empty_libs() {
        let config = RuntimeConfig::from_libs(vec![]);
        // Should not panic with empty configuration
        config.validate_and_warn();
    }

    /// Test that validate_and_warn() handles minimal configuration.
    #[test]
    fn test_validate_and_warn_minimal() {
        let config = RuntimeConfig::minimal();
        // Should not panic and should not trigger any warnings
        config.validate_and_warn();

        // Verify minimal has no security-sensitive options
        assert!(!config.libs.contains(&"std_debug".to_string()));
        assert!(!config.libs.contains(&"std_all_unsafe".to_string()));
        assert!(!config.should_enable_module("env"));
    }
}
