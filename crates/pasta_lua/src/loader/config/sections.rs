//! Custom-field configuration sections for pasta.toml.
//!
//! These typed sections back the `PastaConfig` accessor methods
//! (`logging()`, `persistence()`, `lua()`, `talk()`, `debug()`) and the
//! SHIORI-profile `[ghost]` defaults applied at config-construction time.
//! They are split out of `config/mod.rs` purely to keep each file small;
//! the public surface is unchanged (re-exported via `pub use sections::*`).

use serde::Deserialize;

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

/// Ghost configuration defaults for the `[ghost]` section in pasta.toml.
///
/// Acts as the single source of truth (SSOT) for the SHIORI default values of
/// the `[ghost]` section. The defaults (`talk_interval_min = 180`,
/// `talk_interval_max = 300`, `hour_margin = 30`, `spot_newlines = 1.5`) match
/// the current Lua literal fallbacks so that omitting `[ghost]` keeps the
/// historical behavior (requirements 1.2 / 1.3 / 3.3).
///
/// This struct is a **value supply source only**: it provides the default
/// values for the later `apply_shiori_defaults` completion step. It is
/// intentionally NOT wired into [`PastaConfig::parse`] and does not extract
/// `[ghost]` out of `custom_fields` — the `[ghost]` section keeps flowing
/// through to Lua via `custom_fields` (the `@pasta_config` exposure path stays
/// unchanged).
#[derive(Debug, Clone)]
pub struct GhostConfig {
    /// Minimum interval between random talks, in seconds.
    /// Default: 180
    pub talk_interval_min: i64,

    /// Maximum interval between random talks, in seconds.
    /// Default: 300
    pub talk_interval_max: i64,

    /// Margin (in seconds) around the top of the hour during which a random
    /// talk is suppressed in favor of the hourly event.
    /// Default: 30
    pub hour_margin: i64,

    /// Number of blank lines inserted on actor/spot switch (as a ratio).
    /// Default: 1.5
    pub spot_newlines: f64,
}

impl Default for GhostConfig {
    fn default() -> Self {
        Self {
            talk_interval_min: default_talk_interval_min(),
            talk_interval_max: default_talk_interval_max(),
            hour_margin: default_hour_margin(),
            spot_newlines: default_spot_newlines(),
        }
    }
}

/// Default minimum random-talk interval in seconds (`180`).
pub const fn default_talk_interval_min() -> i64 {
    180
}

/// Default maximum random-talk interval in seconds (`300`).
pub const fn default_talk_interval_max() -> i64 {
    300
}

/// Default top-of-hour suppression margin in seconds (`30`).
pub const fn default_hour_margin() -> i64 {
    30
}

/// Default spot/actor switch blank-line ratio (`1.5`).
pub const fn default_spot_newlines() -> f64 {
    1.5
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
