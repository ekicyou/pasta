//! Runtime configuration for Lua standard library and module selection.
//!
//! RuntimeConfig controls which Lua standard libraries and mlua-stdlib modules
//! are enabled in the PastaLuaRuntime.

use crate::debug::{DebugConfig, DebugFileConfig};
use crate::error::ConfigError;
use crate::loader::{LuaConfig, default_libs};
use mlua::{Function, Lua, Result as LuaResult, StdLib, Value};

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
    /// - `std_math`, `std_package`, `std_debug`, `std_jit`, `std_ffi`, `std_bit`
    ///
    /// Valid mlua-stdlib modules:
    /// - `assertions`, `testing`, `env`, `regex`, `json`, `yaml`
    pub libs: Vec<String>,

    /// Resolved debug backend configuration (task 4.2 — single enable choke point).
    ///
    /// This is the ONE place the runtime VM init reads to decide whether to call
    /// [`crate::debug::enable`]. It defaults to **disabled** (`enabled = false`,
    /// `listen = None`), so every existing `RuntimeConfig` constructor
    /// (`new`/`minimal`/`full`/`from_libs`/`From<LuaConfig>`) is zero-cost: no
    /// hook, no port, no `std_debug` exposure (R5.2 / R5.3 / R5.5). The loader
    /// path overrides this via [`with_debug`](Self::with_debug) after resolving
    /// pasta.toml `[debug]` + the `PASTA_DEBUG`/`PASTA_DEBUG_PORT` environment.
    pub debug: DebugConfig,
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
            debug: DebugConfig::default(),
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
            debug: DebugConfig::default(),
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
            debug: DebugConfig::default(),
        }
    }

    /// Create a configuration from a custom libs array.
    pub fn from_libs(libs: Vec<String>) -> Self {
        Self {
            libs,
            debug: DebugConfig::default(),
        }
    }

    /// Attach a resolved [`DebugConfig`] to this configuration (builder).
    ///
    /// Used by the loader path (`PastaLuaRuntime::from_loader_with_scene_dic`) to
    /// set the debug gate from pasta.toml `[debug]` + the environment, and by
    /// tests to enable the backend. When `debug.enabled` is `false` this is a
    /// no-op relative to the zero-cost default. Returns `self` for chaining.
    pub fn with_debug(mut self, debug: DebugConfig) -> Self {
        self.debug = debug;
        self
    }

    /// Resolve and attach the debug gate from a pasta.toml `[debug]` section plus
    /// the process environment (`PASTA_DEBUG` / `PASTA_DEBUG_PORT`).
    ///
    /// This is the loader-side bridge the design calls `DebugConfig::from_runtime`
    /// (design "DebugConfig & Gate"): it funnels the file `[debug]` config and the
    /// environment through the single pure [`DebugConfig::resolve`] choke point and
    /// stores the result on `self.debug`. Precedence (env overrides file overrides
    /// defaults) and the `listen = None` when disabled invariant are owned by
    /// `resolve`. Returns `self` for chaining.
    pub fn with_debug_from_file_and_env(self, file: Option<&DebugFileConfig>) -> Self {
        self.with_debug(DebugConfig::from_env(file))
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
            "std_coroutine" => Ok(StdLib::NONE),
            "std_table" => Ok(StdLib::TABLE),
            "std_io" => Ok(StdLib::IO),
            "std_os" => Ok(StdLib::OS),
            "std_string" => Ok(StdLib::STRING),
            "std_math" => Ok(StdLib::MATH),
            "std_package" => Ok(StdLib::PACKAGE),
            "std_debug" => Ok(StdLib::DEBUG),
            "std_jit" => Ok(StdLib::JIT),
            "std_ffi" => Ok(StdLib::FFI),
            "std_bit" => Ok(StdLib::BIT),
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
        Self {
            libs: config.libs,
            debug: DebugConfig::default(),
        }
    }
}

/// Execute Lua `require()` from Rust.
///
/// This helper function calls Lua's standard `require()` function,
/// following `package.path` settings for module resolution.
///
/// # Arguments
/// * `lua` - Lua VM instance
/// * `module_name` - Module name to require (e.g., "main", "pasta.shiori.entry")
///
/// # Returns
/// * `Ok(Value)` - Return value from `require()` (usually a module table)
/// * `Err(LuaError)` - Module not found or loading error
///
/// # Example
/// ```rust,ignore
/// let result = lua_require(&lua, "main")?;
/// let result = lua_require(&lua, "pasta.shiori.entry")?;
/// ```
// SAFETY(injection): All call sites pass compile-time string literals as `module_name`
// ("main", "pasta.shiori.entry", "pasta.scene_dic"). No external user input reaches
// this function. Errors are propagated via `LuaResult`.
pub fn lua_require(lua: &Lua, module_name: &str) -> LuaResult<Value> {
    let require: Function = lua.globals().get("require")?;
    require.call(module_name)
}
