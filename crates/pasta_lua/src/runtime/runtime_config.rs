//! Runtime configuration for Lua standard library and module selection.
//!
//! RuntimeConfig controls which Lua standard libraries are enabled in the
//! PastaLuaRuntime.

use crate::error::ConfigError;
use mlua::{Function, Lua, Result as LuaResult, StdLib, Value};
use pasta_core::registry::SceneAliasTable;

/// Default libs configuration used by [`RuntimeConfig::new`].
///
/// Returns: `["std_all"]`
pub fn default_libs() -> Vec<String> {
    vec!["std_all".into()]
}

/// Libraries the VM cannot be built without (Rust-side module registration
/// uses them). Checked by [`RuntimeConfig::ensure_libs`].
const REQUIRED_LIBS: &[&str] = &["std_package"];

/// Configuration for which standard libraries to enable in the Lua runtime.
///
/// Uses Cargo-style array notation with optional subtraction syntax.
///
/// `std_package` is required: a configuration without it (including one that
/// subtracts it with `-std_package`) fails with
/// [`ConfigError::MissingRequiredLibrary`] before the VM is built.
///
/// # Examples
///
/// ```rust
/// use pasta_lua::RuntimeConfig;
///
/// // Default configuration (safe Lua standard libraries)
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
///     "-std_debug".into(),
/// ]);
/// ```
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// Library configuration array.
    ///
    /// Only Lua standard library names (std_* prefix) have meaning; other
    /// names are silently ignored. Use `-` prefix to subtract/exclude a library.
    ///
    /// Valid Lua standard libraries:
    /// - `std_all` - All safe libraries (StdLib::ALL_SAFE, excludes std_debug)
    /// - `std_all_unsafe` - All libraries including debug (StdLib::ALL)
    /// - `std_coroutine`, `std_table`, `std_io`, `std_os`, `std_string`
    /// - `std_math`, `std_package`, `std_debug`, `std_jit`, `std_ffi`, `std_bit`
    ///
    /// Required: `std_package` (`std_all` / `std_all_unsafe` include it).
    /// Without it the runtime returns [`ConfigError::MissingRequiredLibrary`]
    /// and builds no VM.
    ///
    /// The framework scripts (`pasta_scripts`, loaded via the loader) also use
    /// `std_string`, `std_table`, `std_math` and `std_os`. These are not
    /// checked; if one is missing, the Lua error names it
    /// (e.g. `attempt to index global 'os' (a nil value)`).
    pub libs: Vec<String>,

    /// Global scene name alias table (scene-name-alias).
    ///
    /// The runtime hands this ONE table to both `@pasta_search` registration
    /// points: the initial registration at VM init and the re-registration by
    /// `finalize_scene`. Defaults to the **empty** table (no aliases), so every
    /// existing constructor keeps the pre-alias search behavior. It must match
    /// the table the transpiler used for the same dictionary (the loader
    /// guarantees this).
    pub scene_aliases: SceneAliasTable,
}

impl RuntimeConfig {
    /// Create a new configuration with all safe libraries enabled (default).
    ///
    /// Default: `["std_all"]`
    pub fn new() -> Self {
        Self {
            libs: default_libs(),
            scene_aliases: SceneAliasTable::empty(),
        }
    }

    /// Create a configuration with all libraries enabled, including `std_debug`.
    ///
    /// Contains: `["std_all_unsafe"]`
    pub fn full() -> Self {
        Self {
            libs: vec!["std_all_unsafe".into()],
            scene_aliases: SceneAliasTable::empty(),
        }
    }

    /// Create a minimal configuration with only safe Lua standard libraries.
    ///
    /// Contains: `["std_all"]`
    pub fn minimal() -> Self {
        Self {
            libs: vec!["std_all".into()],
            scene_aliases: SceneAliasTable::empty(),
        }
    }

    /// Create a configuration from a custom libs array.
    ///
    /// `libs` must include `std_package` (see [`libs`](Self::libs)); the
    /// framework scripts loaded via the loader also use `std_string`,
    /// `std_table`, `std_math` and `std_os`.
    pub fn from_libs(libs: Vec<String>) -> Self {
        Self {
            libs,
            scene_aliases: SceneAliasTable::empty(),
        }
    }

    /// Set the global scene name alias table (builder).
    ///
    /// The same table is passed to both `@pasta_search` registrations (VM init
    /// and `finalize_scene`). Returns `self` for chaining.
    pub fn with_scene_aliases(mut self, scene_aliases: SceneAliasTable) -> Self {
        self.scene_aliases = scene_aliases;
        self
    }

    /// `[loader] ffi` の反映（builder）。`enabled` が真なら `libs` に
    /// `"std_ffi"` を 1 つ足し、偽なら何もしない（`default_libs()` は変えない）。
    pub fn with_ffi(mut self, enabled: bool) -> Self {
        if enabled {
            self.libs.push("std_ffi".into());
        }
        self
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
                // Non-std_ names are silently ignored
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

    /// Return [`ConfigError::MissingRequiredLibrary`] if this configuration
    /// lacks any library in `REQUIRED_LIBS` (judged on the [`to_stdlib`]
    /// flags, so `-std_package` subtraction counts as missing).
    ///
    /// [`to_stdlib`]: Self::to_stdlib
    pub(crate) fn ensure_libs(&self) -> Result<(), ConfigError> {
        let std_lib = self.to_stdlib()?;
        let missing: Vec<&str> = REQUIRED_LIBS
            .iter()
            .copied()
            .filter(|name| Self::parse_std_lib(name).is_ok_and(|flag| !std_lib.contains(flag)))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::MissingRequiredLibrary(missing.join(", ")))
        }
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

    /// Validate configuration and emit security warnings.
    ///
    /// Emits `tracing::warn` for:
    /// - `std_debug` or `std_all_unsafe` enabled
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

        // Log enabled libraries at debug level
        tracing::debug!(libs = ?self.libs, "Lua library configuration");
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod ensure_libs_tests {
    use super::*;

    fn libs(names: &[&str]) -> RuntimeConfig {
        RuntimeConfig::from_libs(names.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn standard_configs_pass() {
        assert_eq!(RuntimeConfig::new().ensure_libs(), Ok(()));
        assert_eq!(RuntimeConfig::minimal().ensure_libs(), Ok(()));
        assert_eq!(RuntimeConfig::full().ensure_libs(), Ok(()));
        assert_eq!(libs(&["std_all", "-std_math"]).ensure_libs(), Ok(()));
    }

    #[test]
    fn missing_std_package_is_named() {
        for cfg in [libs(&["std_string"]), libs(&["std_all", "-std_package"])] {
            match cfg.ensure_libs() {
                Err(ConfigError::MissingRequiredLibrary(names)) => {
                    assert!(names.contains("std_package"), "{names}")
                }
                other => panic!(
                    "expected MissingRequiredLibrary for {:?}, got {other:?}",
                    cfg.libs
                ),
            }
        }
    }
}

#[cfg(test)]
mod with_ffi_tests {
    use super::*;
    use crate::context::TranspileContext;
    use crate::runtime::PastaLuaRuntime;

    /// 偽なら何もしない（1.3・8.1）。既定のライブラリの組み合わせも変えない（8.4）。
    #[test]
    fn with_ffi_false_leaves_libs_unchanged() {
        assert_eq!(RuntimeConfig::new().with_ffi(false).libs, default_libs());
        assert!(!default_libs().iter().any(|l| l == "std_ffi"));
    }

    /// 真なら `std_ffi` を足し、StdLib の FFI が立つ（1.2）。
    #[test]
    fn with_ffi_true_adds_std_ffi() {
        let config = RuntimeConfig::new().with_ffi(true);
        assert!(config.libs.iter().any(|l| l == "std_ffi"));
        assert!(config.to_stdlib().unwrap().contains(StdLib::FFI));
    }

    /// 真で作った VM はグローバルの `ffi` と `require "ffi"` が同じ表、
    /// 偽なら今までどおり `nil`（1.2・1.3）。
    #[test]
    fn vm_has_ffi_global_only_when_enabled() {
        let on = PastaLuaRuntime::with_config(
            TranspileContext::new(),
            RuntimeConfig::new().with_ffi(true),
        )
        .unwrap();
        let same: bool = on
            .lua()
            .load("return type(ffi) == 'table' and ffi == require('ffi')")
            .eval()
            .unwrap();
        assert!(same);

        let off = PastaLuaRuntime::with_config(
            TranspileContext::new(),
            RuntimeConfig::new().with_ffi(false),
        )
        .unwrap();
        let is_nil: bool = off.lua().load("return ffi == nil").eval().unwrap();
        assert!(is_nil);
    }
}
