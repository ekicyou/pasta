//! Runtime configuration for Lua standard library and module selection.
//!
//! RuntimeConfig controls which Lua standard libraries and mlua-stdlib modules
//! are enabled in the PastaLuaRuntime.

use crate::debug::kick::KickSink;
use crate::debug::{DebugConfig, DebugFileConfig};
use crate::error::ConfigError;
use mlua::{Function, Lua, Result as LuaResult, StdLib, Value};
use pasta_core::registry::SceneAliasTable;

/// Default libs configuration used by [`RuntimeConfig::new`].
///
/// Returns: ["std_all", "assertions", "testing", "regex", "json", "yaml"]
/// Note: `env` is excluded by default for security (filesystem access).
pub fn default_libs() -> Vec<String> {
    vec![
        "std_all".into(),
        "assertions".into(),
        "testing".into(),
        "regex".into(),
        "json".into(),
        "yaml".into(),
    ]
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
#[derive(Clone)]
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

    /// Resolved debug backend configuration (task 4.2 — single enable choke point).
    ///
    /// This is the ONE place the runtime VM init reads to decide whether to call
    /// [`crate::debug::enable`]. It defaults to **disabled** (`enabled = false`,
    /// `listen = None`), so every existing `RuntimeConfig` constructor
    /// (`new`/`minimal`/`full`/`from_libs`) is zero-cost: no
    /// hook, no port, no `std_debug` exposure (R5.2 / R5.3 / R5.5). The loader
    /// path overrides this via [`with_debug`](Self::with_debug) after resolving
    /// pasta.toml `[debug]` + the `PASTA_DEBUG`/`PASTA_DEBUG_PORT` environment.
    pub debug: DebugConfig,

    /// Optional, host-injected scene-kick sink (`KickSinkSeam`, requirements
    /// 2.4 / 2.6).
    ///
    /// `pasta_lua` holds this opaquely: an outer host (`pasta_shiori`) binds a
    /// [`KickSink`] closure via [`with_kick_sink`](Self::with_kick_sink) before
    /// VM construction so an inbound `playScene` debug request can be delivered
    /// to the actor. It defaults to **`None`** (未注入), so every existing
    /// constructor leaves the kick path inactive by type and by default value
    /// (R2.6). Wiring the sink through `enable` into the socket-bridge is a later
    /// task (2.x); this field is only the injection slot.
    pub kick_sink: Option<KickSink>,

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

impl std::fmt::Debug for RuntimeConfig {
    // Hand-written because `kick_sink: Option<KickSink>` wraps a `dyn Fn` trait
    // object, which is not `Debug`. The sink is summarised as present/absent so
    // the rest of the config still prints (mirrors the derived layout).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeConfig")
            .field("libs", &self.libs)
            .field("debug", &self.debug)
            .field("kick_sink", &self.kick_sink.as_ref().map(|_| "<sink>"))
            .field("scene_aliases", &self.scene_aliases)
            .finish()
    }
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
            kick_sink: None,
            scene_aliases: SceneAliasTable::empty(),
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
            kick_sink: None,
            scene_aliases: SceneAliasTable::empty(),
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
            kick_sink: None,
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
            debug: DebugConfig::default(),
            kick_sink: None,
            scene_aliases: SceneAliasTable::empty(),
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

    /// Attach (or clear) the host-injected scene-kick sink (builder).
    ///
    /// `KickSinkSeam` (requirements 2.4 / 2.6): an outer host (`pasta_shiori`)
    /// binds a [`KickSink`] closure here before VM construction so an inbound
    /// `playScene` debug request can be delivered to the actor. Passing `None`
    /// leaves (or restores) the default — the kick path stays inactive.
    /// `pasta_lua` holds the sink opaquely and never inspects its body. Returns
    /// `self` for chaining.
    pub fn with_kick_sink(mut self, kick_sink: Option<KickSink>) -> Self {
        self.kick_sink = kick_sink;
        self
    }

    /// Set the global scene name alias table (builder).
    ///
    /// The same table is passed to both `@pasta_search` registrations (VM init
    /// and `finalize_scene`). Returns `self` for chaining.
    pub fn with_scene_aliases(mut self, scene_aliases: SceneAliasTable) -> Self {
        self.scene_aliases = scene_aliases;
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
mod kick_sink_tests {
    use super::*;
    use crate::debug::kick::KickRequest;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn default_runtime_config_has_no_kick_sink() {
        // Requirement 2.6: debug 無効が既定 ⇒ キック経路は型・既定値で非活性。
        // 全コンストラクタで sink 未注入（None）であること。
        assert!(RuntimeConfig::new().kick_sink.is_none());
        assert!(RuntimeConfig::full().kick_sink.is_none());
        assert!(RuntimeConfig::minimal().kick_sink.is_none());
        assert!(
            RuntimeConfig::from_libs(vec!["std_all".into()])
                .kick_sink
                .is_none()
        );
        assert!(RuntimeConfig::default().kick_sink.is_none());
    }

    #[test]
    fn with_kick_sink_holds_some_and_is_invocable() {
        // Requirement 2.4: 注入された汎用 sink を RuntimeConfig が保持する。
        let called = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&called);
        let config =
            RuntimeConfig::new().with_kick_sink(Some(Arc::new(move |req: KickRequest| {
                assert_eq!(req.scene, "OnTest");
                flag.store(true, Ordering::SeqCst);
            })));

        let sink = config.kick_sink.as_ref().expect("sink must be Some");
        sink(KickRequest {
            scene: "OnTest".into(),
        });
        assert!(
            called.load(Ordering::SeqCst),
            "injected sink must be called"
        );
    }

    #[test]
    fn with_kick_sink_none_clears_sink() {
        let config = RuntimeConfig::new()
            .with_kick_sink(Some(Arc::new(|_req: KickRequest| {})))
            .with_kick_sink(None);
        assert!(config.kick_sink.is_none());
    }
}
