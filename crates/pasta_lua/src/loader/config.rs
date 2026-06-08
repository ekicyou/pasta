//! Configuration management for Pasta Loader.
//!
//! This module provides configuration file parsing and default values
//! for the pasta loader startup sequence.

use serde::Deserialize;
use std::fs;
use std::path::Path;

use super::LoaderError;

/// Main configuration structure for pasta.toml.
///
/// Contains loader-specific settings and custom user fields.
#[derive(Debug, Clone)]
pub struct PastaConfig {
    /// Loader-specific configuration ([loader] section)
    pub loader: LoaderConfig,

    /// All other fields/sections (custom user configuration)
    /// Note: The [loader] section is explicitly excluded.
    pub custom_fields: toml::Table,
}

impl Default for PastaConfig {
    fn default() -> Self {
        Self {
            loader: LoaderConfig::default(),
            custom_fields: toml::Table::new(),
        }
    }
}

impl PastaConfig {
    /// Load configuration from pasta.toml in the base directory.
    ///
    /// Returns an error if pasta.toml doesn't exist.
    ///
    /// # Arguments
    /// * `base_dir` - Base directory to look for pasta.toml
    ///
    /// # Returns
    /// * `Ok(PastaConfig)` - Configuration loaded successfully
    /// * `Err(LoaderError)` - File not found, read error, or parse error
    pub fn load(base_dir: &Path) -> Result<Self, LoaderError> {
        let config_path = base_dir.join("pasta.toml");

        if !config_path.exists() {
            return Err(LoaderError::config_not_found(&config_path));
        }

        let content =
            fs::read_to_string(&config_path).map_err(|e| LoaderError::io(&config_path, e))?;

        Self::parse(&content).map_err(|e| LoaderError::config(&config_path, e))
    }

    /// Parse configuration from TOML string.
    fn parse(content: &str) -> Result<Self, toml::de::Error> {
        // Parse as a raw TOML table first
        let mut table: toml::Table = toml::from_str(content)?;

        // Extract and deserialize [loader] section
        let loader = if let Some(loader_value) = table.remove("loader") {
            loader_value.try_into()?
        } else {
            LoaderConfig::default()
        };

        // Everything else becomes custom_fields
        let custom_fields = table;

        tracing::debug!("Parsed configuration");
        Ok(Self {
            loader,
            custom_fields,
        })
    }

    /// Get a custom configuration section by key.
    ///
    /// Deserializes a TOML section into the target type.
    /// Returns `None` if the section is missing or cannot be deserialized.
    fn get_custom_config<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.custom_fields
            .get(key)
            .and_then(|v| v.clone().try_into().ok())
    }

    /// Get logging configuration from [logging] section.
    pub fn logging(&self) -> Option<LoggingConfig> {
        self.get_custom_config("logging")
    }

    /// Get persistence configuration from [persistence] section.
    pub fn persistence(&self) -> Option<PersistenceConfig> {
        self.get_custom_config("persistence")
    }

    /// Get Lua library configuration from [lua] section.
    pub fn lua(&self) -> Option<LuaConfig> {
        self.get_custom_config("lua")
    }

    /// Get talk configuration from [talk] section.
    pub fn talk(&self) -> Option<TalkConfig> {
        self.get_custom_config("talk")
    }

    /// Get debug configuration from [debug] section.
    ///
    /// Returns `None` if the `[debug]` section is absent. When present, missing
    /// fields fall back to their serde defaults (`enabled = false`, `port = 9276`).
    pub fn debug(&self) -> Option<DebugFileConfig> {
        self.get_custom_config("debug")
    }

    /// Create from TOML string.
    #[allow(
        clippy::should_implement_trait,
        reason = "Public API stability: keep the existing inherent from_str constructor name without renaming"
    )]
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        Self::parse(s)
    }
}

/// Loader-specific configuration ([loader] section).
#[derive(Debug, Clone, Deserialize)]
pub struct LoaderConfig {
    /// Pasta file discovery patterns (default: ["dic/*/*.pasta"])
    #[serde(default = "default_pasta_patterns")]
    pub pasta_patterns: Vec<String>,

    /// Lua module search paths in priority order
    #[serde(default = "default_lua_search_paths")]
    pub lua_search_paths: Vec<String>,

    /// Directory for transpiled output (default: "profile/pasta/cache/lua")
    #[serde(default = "default_transpiled_output_dir")]
    pub transpiled_output_dir: String,

    /// Debug mode - save transpiled files (default: true)
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            pasta_patterns: default_pasta_patterns(),
            lua_search_paths: default_lua_search_paths(),
            transpiled_output_dir: default_transpiled_output_dir(),
            debug_mode: default_debug_mode(),
        }
    }
}

fn default_pasta_patterns() -> Vec<String> {
    vec!["dic/*/*.pasta".to_string()]
}

pub fn default_lua_search_paths() -> Vec<String> {
    vec![
        "profile/pasta/save/lua".to_string(),
        "scripts".to_string(),
        "profile/pasta/pasta_scripts".to_string(),
        "profile/pasta/cache/lua".to_string(),
        "scriptlibs".to_string(),
    ]
}

fn default_transpiled_output_dir() -> String {
    "profile/pasta/cache/lua".to_string()
}

fn default_debug_mode() -> bool {
    true
}

/// Logging configuration from [logging] section in pasta.toml.
///
/// Configures instance-specific logging with file rotation and log level filtering.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log file path relative to load_dir.
    /// Default: "profile/pasta/logs/pasta.log"
    #[serde(default = "default_log_file_path")]
    pub file_path: String,

    /// Number of days to retain log files.
    /// Default: 7
    #[serde(default = "default_rotation_days")]
    pub rotation_days: usize,

    /// Default log level.
    /// Default: "info"
    /// Valid: "error", "warn", "info", "debug", "trace"
    #[serde(default = "default_log_level")]
    pub level: String,

    /// EnvFilter directive string.
    /// When set, takes precedence over `level`.
    /// Example: "debug,pasta_shiori=info,pasta_lua=warn"
    #[serde(default)]
    pub filter: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            file_path: default_log_file_path(),
            rotation_days: default_rotation_days(),
            level: default_log_level(),
            filter: None,
        }
    }
}

impl LoggingConfig {
    /// Build EnvFilter directive string.
    /// Priority: filter > level > default ("debug")
    pub fn to_filter_directive(&self) -> String {
        if let Some(ref filter) = self.filter {
            filter.clone()
        } else {
            self.level.clone()
        }
    }
}

pub fn default_log_file_path() -> String {
    "profile/pasta/logs/pasta.log".to_string()
}

fn default_rotation_days() -> usize {
    7
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Persistence configuration from [persistence] section in pasta.toml.
///
/// Configures persistent data storage with optional obfuscation.
#[derive(Debug, Clone, Deserialize)]
pub struct PersistenceConfig {
    /// Enable obfuscation (gzip compression) for saved data.
    /// Default: false
    #[serde(default)]
    pub obfuscate: bool,

    /// Save file path relative to load_dir.
    /// Default: "profile/pasta/save/save.json" (or .dat if obfuscate=true)
    #[serde(default = "default_persistence_file_path")]
    pub file_path: String,

    /// Enable debug logging for persistence operations.
    /// Default: false
    #[serde(default)]
    pub debug_mode: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            obfuscate: false,
            file_path: default_persistence_file_path(),
            debug_mode: false,
        }
    }
}

fn default_persistence_file_path() -> String {
    "profile/pasta/save/save.json".to_string()
}

impl PersistenceConfig {
    /// Get the effective file path based on obfuscate setting.
    ///
    /// If obfuscate is true and file_path ends with .json, changes extension to .dat.
    pub fn effective_file_path(&self) -> String {
        if self.obfuscate && self.file_path.ends_with(".json") {
            self.file_path.replace(".json", ".dat")
        } else if self.obfuscate && !self.file_path.ends_with(".dat") {
            format!("{}.dat", self.file_path)
        } else {
            self.file_path.clone()
        }
    }
}

/// Lua library configuration from [lua] section in pasta.toml.
///
/// Configures which Lua standard libraries and mlua-stdlib modules to enable.
/// Uses Cargo-style array notation with optional subtraction syntax.
///
/// # Examples
///
/// ```toml
/// [lua]
/// # Default: all safe libraries + common mlua-stdlib modules
/// libs = ["std_all", "assertions", "testing", "regex", "json", "yaml"]
///
/// # Minimal configuration
/// libs = []
///
/// # Subtraction syntax
/// libs = ["std_all", "testing", "-std_debug"]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct LuaConfig {
    /// Library configuration array.
    ///
    /// Supports Lua standard libraries (std_* prefix) and mlua-stdlib modules.
    /// Use `-` prefix to subtract/exclude a library.
    ///
    /// Valid Lua standard libraries:
    /// - `std_all` - All safe libraries (StdLib::ALL_SAFE)
    /// - `std_all_unsafe` - All libraries including debug (StdLib::ALL)
    /// - `std_coroutine`, `std_table`, `std_io`, `std_os`, `std_string`
    /// - `std_math`, `std_package`, `std_debug`, `std_jit`, `std_ffi`, `std_bit`
    ///
    /// Valid mlua-stdlib modules:
    /// - `assertions`, `testing`, `env`, `regex`, `json`, `yaml`
    #[serde(default = "default_libs")]
    pub libs: Vec<String>,
}

/// Default libs configuration.
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

impl Default for LuaConfig {
    fn default() -> Self {
        Self {
            libs: default_libs(),
        }
    }
}

/// Talk configuration from [talk] section in pasta.toml.
///
/// Configures sakura script wait insertion for natural conversation tempo.
///
/// # Examples
///
/// ```toml
/// [talk]
/// # Wait values (milliseconds)
/// script_wait_normal = 50
/// script_wait_period = 1000
/// script_wait_comma = 500
/// script_wait_strong = 500
/// script_wait_leader = 200
///
/// # Character sets
/// chars_period = "｡。．."
/// chars_comma = "、，,"
/// chars_strong = "？！!?"
/// chars_leader = "･・‥…"
/// chars_line_start_prohibited = "゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ"
/// chars_line_end_prohibited = "（［｛「『([{｢"
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TalkConfig {
    // Wait values (milliseconds)
    /// Wait for general characters (default: 50ms)
    pub script_wait_normal: i64,
    /// Wait for period characters (default: 1000ms)
    pub script_wait_period: i64,
    /// Wait for comma characters (default: 500ms)
    pub script_wait_comma: i64,
    /// Wait for strong emphasis characters (default: 500ms)
    pub script_wait_strong: i64,
    /// Wait for leader characters (default: 200ms)
    pub script_wait_leader: i64,

    // Character sets
    /// Period characters (default: "｡。．.")
    pub chars_period: String,
    /// Comma characters (default: "、，,")
    pub chars_comma: String,
    /// Strong emphasis characters (default: "？！!?")
    pub chars_strong: String,
    /// Leader characters (default: "･・‥…")
    pub chars_leader: String,
    /// Line start prohibited characters (行頭禁則)
    pub chars_line_start_prohibited: String,
    /// Line end prohibited characters (行末禁則)
    pub chars_line_end_prohibited: String,
}

impl Default for TalkConfig {
    fn default() -> Self {
        Self {
            script_wait_normal: 50,
            script_wait_period: 1000,
            script_wait_comma: 500,
            script_wait_strong: 500,
            script_wait_leader: 200,
            chars_period: "｡。．.".into(),
            chars_comma: "、，,".into(),
            chars_strong: "？！!?".into(),
            chars_leader: "･・‥…".into(),
            chars_line_start_prohibited: "゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ".into(),
            chars_line_end_prohibited: "（［｛「『([{｢".into(),
        }
    }
}

/// Debug configuration from `[debug]` section in pasta.toml.
///
/// Controls the Rust-hosted DAP debug backend embedded in pasta_lua.
/// All fields default conservatively so that omitting the section (or the
/// whole file) keeps debugging OFF and the production path zero-cost.
///
/// Runtime resolution combines this file config with the `PASTA_DEBUG` /
/// `PASTA_DEBUG_PORT` / `PASTA_DEBUG_SOURCE_MODE` /
/// `PASTA_DEBUG_SOURCE_MAP_SIDECAR` environment variables — see
/// [`crate::debug::DebugConfig`]. The precedence for the source-presentation
/// mode is `DAP attach 引数 > env > pasta.toml [debug] > 既定 .pasta`; for the
/// sidecar flag it is `env > pasta.toml [debug] > 既定 false`
/// (requirements 6.3 / 3.2).
///
/// # Examples
///
/// ```toml
/// [debug]
/// enabled = true            # default: false
/// port = 9276               # default: 9276
/// present_as = "lua"        # default: .pasta ("pasta" / "lua", case-insensitive)
/// source_map_sidecar = true # default: false
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DebugFileConfig {
    /// Enable the debug backend. Default: `false`.
    pub enabled: bool,

    /// TCP port the DAP listener binds to when enabled. Default: `9276`.
    pub port: u16,

    /// Source presentation mode for the debug session (`"pasta"` / `"lua"`,
    /// case-insensitive). `None` (the default, when the key is omitted) means
    /// "not specified by the file" so the env/default decides; an invalid value
    /// is tolerated by the resolver and falls back to the default `.pasta`
    /// (requirements 6.1 / 6.3). The string→[`crate::debug::SourceMode`] parse
    /// happens at resolution time (in `crate::debug`) to avoid a `loader`→`debug`
    /// dependency cycle here.
    pub present_as: Option<String>,

    /// Whether to additionally write the on-disk `.lua.map` sidecar (3.2).
    /// Default: `false`. The in-memory source map is always the primary path.
    pub source_map_sidecar: bool,
}

impl Default for DebugFileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_debug_port(),
            present_as: None,
            source_map_sidecar: false,
        }
    }
}

/// Default debug listener port.
pub const fn default_debug_port() -> u16 {
    9276
}
