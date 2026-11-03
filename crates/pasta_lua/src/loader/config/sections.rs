//! Custom-field configuration sections for pasta.toml.
//!
//! These typed sections back the `PastaConfig` accessor methods
//! (`logging()`, `persistence()`, `talk()`, `debug()`) and the
//! SHIORI-profile `[ghost]` defaults applied at config-construction time.
//! They are split out of `config/mod.rs` purely to keep each file small;
//! the public surface is unchanged (re-exported via `pub use sections::*`).

use serde::Deserialize;

/// Logging configuration from `[logging]` section in pasta.toml.
///
/// Configures the instance-specific log file path and log level filtering.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log file path relative to load_dir.
    /// Default: "profile/pasta/logs/pasta.log"
    #[serde(default = "default_log_file_path")]
    pub file_path: String,

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
            level: default_log_level(),
            filter: None,
        }
    }
}

impl LoggingConfig {
    /// Build EnvFilter directive string.
    /// Priority: filter > level (`level` defaults to "info")
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

fn default_log_level() -> String {
    "info".to_string()
}

/// Persistence configuration from `[persistence]` section in pasta.toml.
///
/// Configures persistent data storage with optional obfuscation.
#[derive(Debug, Clone, Deserialize)]
pub struct PersistenceConfig {
    /// Enable obfuscation (gzip compression) for saved data.
    /// Default: false
    #[serde(default)]
    pub obfuscate: bool,

    /// Save file path relative to load_dir.
    /// Default: "profile/pasta/save/save.toml" (save.dat if obfuscate=true)
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
    "profile/pasta/save/save.toml".to_string()
}

impl PersistenceConfig {
    /// Get the effective file path based on obfuscate setting.
    ///
    /// If obfuscate is true: a trailing ".toml" becomes ".dat", a trailing ".dat" is kept,
    /// and any other path gets ".dat" appended. Only the end of the path is touched
    /// (folder names are never rewritten), and ".json" is not special.
    pub fn effective_file_path(&self) -> String {
        if !self.obfuscate || self.file_path.ends_with(".dat") {
            self.file_path.clone()
        } else if let Some(stem) = self.file_path.strip_suffix(".toml") {
            format!("{stem}.dat")
        } else {
            format!("{}.dat", self.file_path)
        }
    }
}

/// Talk configuration from `[talk]` section in pasta.toml.
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
/// `talk_interval_max = 300`, `hour_margin = 30`, `spot_newlines = 1.5`,
/// `choice_newlines = 1.5`, `after_choice_newlines = 1.0`) match the current
/// Lua literal fallbacks so that
/// omitting `[ghost]` keeps the historical behavior (requirements 1.2 / 1.3 /
/// 3.3).
///
/// This struct is a **value supply source only**: it provides the default
/// values for the later `apply_shiori_defaults` completion step. It is
/// intentionally NOT wired into `PastaConfig::parse` and does not extract
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

    /// Width of the line break before a choice that follows text (as a ratio).
    /// Default: 1.5
    pub choice_newlines: f64,

    /// Width of the line break after a choice, before the next choice or text
    /// in the same balloon (as a ratio; `1.0` emits a plain `\n`).
    /// Default: 1.0
    pub after_choice_newlines: f64,
}

impl Default for GhostConfig {
    fn default() -> Self {
        Self {
            talk_interval_min: default_talk_interval_min(),
            talk_interval_max: default_talk_interval_max(),
            hour_margin: default_hour_margin(),
            spot_newlines: default_spot_newlines(),
            choice_newlines: default_choice_newlines(),
            after_choice_newlines: default_after_choice_newlines(),
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

/// Default `choice_newlines` (`1.5`).
pub const fn default_choice_newlines() -> f64 {
    1.5
}

/// Default `after_choice_newlines` (`1.0`).
pub const fn default_after_choice_newlines() -> f64 {
    1.0
}

/// pasta.toml の `[debug]` 節。
///
/// 有効の解決は [`crate::debug::resolve_enabled`]（`PASTA_DEBUG` ＞ `enabled` ＞ false）。
/// 知らないキー（古い `port`・`present_as`・`source_map_sidecar` など）は黙って読み飛ばす。
///
/// ```toml
/// [debug]
/// enabled = true # default: false
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DebugFileConfig {
    /// 開発用のイベントを許し、読み込みでシーンマップを作る。既定は false。
    pub enabled: bool,
}

/// `[scene]` の読み取り専用プローブ。`alias` 以外のキーは無視する
/// （`custom_fields` に書いたとおり残る）。要素は位置つきで受け、
/// 意味エラーの文言に行番号を入れる。
#[derive(Debug, Deserialize)]
pub(crate) struct SceneSection {
    pub alias: Option<std::collections::BTreeMap<String, Vec<toml::Spanned<String>>>>,
}

/// pasta.toml 全体から `[scene]` だけを読むプローブ。
#[derive(Debug, Deserialize)]
pub(crate) struct SceneProbe {
    pub scene: Option<SceneSection>,
}

/// 有効な別名表の出どころ（読み込み時のログ用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneAliasSource {
    /// pasta.toml に `[scene.alias]` が無いか行が 1 つも無く、内蔵の既定表を使う。
    BuiltinDefault,
    /// pasta.toml の `[scene.alias]` に書いた行を、キーごとに既定表へ重ねた表を使う。
    PastaToml,
}
