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

    /// Get logging configuration from [logging] section.
    ///
    /// # Returns
    /// * `Some(LoggingConfig)` - If [logging] section exists and is valid
    /// * `None` - If [logging] section is missing or invalid
    pub fn logging(&self) -> Option<LoggingConfig> {
        self.custom_fields
            .get("logging")
            .and_then(|v| v.clone().try_into().ok())
    }

    /// Get persistence configuration from [persistence] section.
    ///
    /// # Returns
    /// * `Some(PersistenceConfig)` - If [persistence] section exists and is valid
    /// * `None` - If [persistence] section is missing or invalid
    pub fn persistence(&self) -> Option<PersistenceConfig> {
        self.custom_fields
            .get("persistence")
            .and_then(|v| v.clone().try_into().ok())
    }

    /// Get Lua library configuration from [lua] section.
    ///
    /// # Returns
    /// * `Some(LuaConfig)` - If [lua] section exists and is valid
    /// * `None` - If [lua] section is missing or invalid
    pub fn lua(&self) -> Option<LuaConfig> {
        self.custom_fields
            .get("lua")
            .and_then(|v| v.clone().try_into().ok())
    }

    /// Get talk configuration from [talk] section.
    ///
    /// # Returns
    /// * `Some(TalkConfig)` - If [talk] section exists and is valid
    /// * `None` - If [talk] section is missing or invalid
    pub fn talk(&self) -> Option<TalkConfig> {
        self.custom_fields
            .get("talk")
            .and_then(|v| v.clone().try_into().ok())
    }

    /// Create from TOML string (for testing).
    #[cfg(test)]
    fn from_str(s: &str) -> Result<Self, toml::de::Error> {
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

fn default_lua_search_paths() -> Vec<String> {
    vec![
        "profile/pasta/save/lua".to_string(),
        "user_scripts".to_string(),
        "scripts".to_string(),
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
    /// Default: "debug"
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

fn default_log_file_path() -> String {
    "profile/pasta/logs/pasta.log".to_string()
}

fn default_rotation_days() -> usize {
    7
}

fn default_log_level() -> String {
    "debug".to_string()
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
    /// - `std_utf8`, `std_math`, `std_package`, `std_debug`
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PastaConfig::default();
        assert_eq!(config.loader.pasta_patterns, vec!["dic/*/*.pasta"]);
        assert_eq!(
            config.loader.lua_search_paths,
            vec![
                "profile/pasta/save/lua",
                "user_scripts",
                "scripts",
                "profile/pasta/cache/lua",
                "scriptlibs"
            ]
        );
        assert_eq!(
            config.loader.transpiled_output_dir,
            "profile/pasta/cache/lua"
        );
        assert!(config.loader.debug_mode);
        assert!(config.custom_fields.is_empty());
    }

    #[test]
    fn test_deserialize_minimal_config() {
        let toml_str = r#"
[loader]
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        assert_eq!(config.loader.pasta_patterns, vec!["dic/*/*.pasta"]);
        assert!(config.loader.debug_mode);
    }

    #[test]
    fn test_deserialize_full_config() {
        let toml_str = r#"
[loader]
pasta_patterns = ["custom/*.pasta"]
lua_search_paths = ["lib", "src"]
transpiled_output_dir = "cache"
debug_mode = false
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        assert_eq!(config.loader.pasta_patterns, vec!["custom/*.pasta"]);
        assert_eq!(config.loader.lua_search_paths, vec!["lib", "src"]);
        assert_eq!(config.loader.transpiled_output_dir, "cache");
        assert!(!config.loader.debug_mode);
    }

    #[test]
    fn test_deserialize_with_custom_fields() {
        // Note: In TOML, keys after [section] belong to that section.
        // To have top-level keys with a section, put them before the section.
        let toml_str = r#"
ghost_name = "TestGhost"
version = "1.0.0"

[loader]
debug_mode = true

[user_data]
key1 = "value1"
key2 = 42
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        assert!(config.loader.debug_mode);

        // Debug: print custom_fields
        println!("custom_fields: {:?}", config.custom_fields);
        println!(
            "custom_fields keys: {:?}",
            config.custom_fields.keys().collect::<Vec<_>>()
        );

        // Check custom fields (loader should be excluded)
        assert!(!config.custom_fields.contains_key("loader"));
        assert_eq!(
            config.custom_fields.get("ghost_name"),
            Some(&toml::Value::String("TestGhost".to_string()))
        );
        assert_eq!(
            config.custom_fields.get("version"),
            Some(&toml::Value::String("1.0.0".to_string()))
        );

        // Check nested table
        let user_data = config.custom_fields.get("user_data").unwrap();
        if let toml::Value::Table(t) = user_data {
            assert_eq!(
                t.get("key1"),
                Some(&toml::Value::String("value1".to_string()))
            );
            assert_eq!(t.get("key2"), Some(&toml::Value::Integer(42)));
        } else {
            panic!("Expected table for user_data");
        }
    }

    #[test]
    fn test_deserialize_without_loader_section() {
        let toml_str = r#"
ghost_name = "NoLoaderGhost"
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        // Should use defaults for loader
        assert_eq!(config.loader.pasta_patterns, vec!["dic/*/*.pasta"]);
        assert_eq!(
            config.custom_fields.get("ghost_name"),
            Some(&toml::Value::String("NoLoaderGhost".to_string()))
        );
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.file_path, "profile/pasta/logs/pasta.log");
        assert_eq!(config.rotation_days, 7);
        assert_eq!(config.level, "debug");
        assert!(config.filter.is_none());
    }

    #[test]
    fn test_logging_config_to_filter_directive_with_filter() {
        // filter優先: filterが設定されている場合はfilterを返す
        let config = LoggingConfig {
            file_path: default_log_file_path(),
            rotation_days: 7,
            level: "info".to_string(),
            filter: Some("debug,pasta_shiori=trace".to_string()),
        };
        assert_eq!(config.to_filter_directive(), "debug,pasta_shiori=trace");
    }

    #[test]
    fn test_logging_config_to_filter_directive_with_level_only() {
        // filterなし: levelを返す
        let config = LoggingConfig {
            file_path: default_log_file_path(),
            rotation_days: 7,
            level: "warn".to_string(),
            filter: None,
        };
        assert_eq!(config.to_filter_directive(), "warn");
    }

    #[test]
    fn test_logging_config_to_filter_directive_default() {
        // デフォルト: "debug"
        let config = LoggingConfig::default();
        assert_eq!(config.to_filter_directive(), "debug");
    }

    #[test]
    fn test_logging_config_from_toml() {
        let toml_str = r#"
[loader]
debug_mode = true

[logging]
file_path = "profile/custom/logs/my.log"
rotation_days = 14
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let logging = config.logging().expect("logging section should exist");
        assert_eq!(logging.file_path, "profile/custom/logs/my.log");
        assert_eq!(logging.rotation_days, 14);
        assert_eq!(logging.level, "debug"); // default
        assert!(logging.filter.is_none());
    }

    #[test]
    fn test_logging_config_from_toml_with_level() {
        let toml_str = r#"
[logging]
level = "info"
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let logging = config.logging().expect("logging section should exist");
        assert_eq!(logging.level, "info");
        assert!(logging.filter.is_none());
        assert_eq!(logging.to_filter_directive(), "info");
    }

    #[test]
    fn test_logging_config_from_toml_with_filter() {
        let toml_str = r#"
[logging]
level = "info"
filter = "debug,pasta_shiori=warn,pasta_lua::runtime=trace"
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let logging = config.logging().expect("logging section should exist");
        assert_eq!(logging.level, "info");
        assert_eq!(
            logging.filter,
            Some("debug,pasta_shiori=warn,pasta_lua::runtime=trace".to_string())
        );
        // filter優先
        assert_eq!(
            logging.to_filter_directive(),
            "debug,pasta_shiori=warn,pasta_lua::runtime=trace"
        );
    }

    #[test]
    fn test_logging_config_defaults_when_partial() {
        let toml_str = r#"
[logging]
file_path = "profile/pasta/logs/custom.log"
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let logging = config.logging().expect("logging section should exist");
        assert_eq!(logging.file_path, "profile/pasta/logs/custom.log");
        assert_eq!(logging.rotation_days, 7); // default
    }

    #[test]
    fn test_logging_config_none_when_missing() {
        let toml_str = r#"
[loader]
debug_mode = true
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        assert!(config.logging().is_none());
    }

    #[test]
    fn test_persistence_config_default() {
        let config = PersistenceConfig::default();
        assert!(!config.obfuscate);
        assert_eq!(config.file_path, "profile/pasta/save/save.json");
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_persistence_config_from_toml() {
        let toml_str = r#"
[persistence]
obfuscate = true
file_path = "profile/custom/save.dat"
debug_mode = true
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let persistence = config
            .persistence()
            .expect("persistence section should exist");
        assert!(persistence.obfuscate);
        assert_eq!(persistence.file_path, "profile/custom/save.dat");
        assert!(persistence.debug_mode);
    }

    #[test]
    fn test_persistence_config_defaults_when_partial() {
        let toml_str = r#"
[persistence]
obfuscate = true
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let persistence = config
            .persistence()
            .expect("persistence section should exist");
        assert!(persistence.obfuscate);
        assert_eq!(persistence.file_path, "profile/pasta/save/save.json"); // default
        assert!(!persistence.debug_mode); // default
    }

    #[test]
    fn test_persistence_config_none_when_missing() {
        let toml_str = r#"
[loader]
debug_mode = true
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        assert!(config.persistence().is_none());
    }

    #[test]
    fn test_persistence_effective_file_path() {
        // Non-obfuscated: keep original path
        let config = PersistenceConfig::default();
        assert_eq!(config.effective_file_path(), "profile/pasta/save/save.json");

        // Obfuscated with .json: change to .dat
        let config = PersistenceConfig {
            obfuscate: true,
            file_path: "profile/pasta/save/save.json".to_string(),
            debug_mode: false,
        };
        assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");

        // Obfuscated with .dat: keep as-is
        let config = PersistenceConfig {
            obfuscate: true,
            file_path: "profile/pasta/save/save.dat".to_string(),
            debug_mode: false,
        };
        assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");
    }

    // ========================================
    // LuaConfig tests
    // ========================================

    #[test]
    fn test_lua_config_default() {
        let config = LuaConfig::default();
        assert_eq!(
            config.libs,
            vec!["std_all", "assertions", "testing", "regex", "json", "yaml"]
        );
    }

    #[test]
    fn test_lua_config_from_toml() {
        let toml_str = r#"
[lua]
libs = ["std_string", "std_table", "testing"]
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let lua = config.lua().expect("lua section should exist");
        assert_eq!(lua.libs, vec!["std_string", "std_table", "testing"]);
    }

    #[test]
    fn test_lua_config_with_subtraction() {
        let toml_str = r#"
[lua]
libs = ["std_all", "-std_debug", "testing"]
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let lua = config.lua().expect("lua section should exist");
        assert_eq!(lua.libs, vec!["std_all", "-std_debug", "testing"]);
    }

    #[test]
    fn test_lua_config_empty_array() {
        let toml_str = r#"
[lua]
libs = []
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let lua = config.lua().expect("lua section should exist");
        assert!(lua.libs.is_empty());
    }

    #[test]
    fn test_lua_config_defaults_when_libs_omitted() {
        let toml_str = r#"
[lua]
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        let lua = config.lua().expect("lua section should exist");
        // libs should use default when omitted
        assert_eq!(
            lua.libs,
            vec!["std_all", "assertions", "testing", "regex", "json", "yaml"]
        );
    }

    #[test]
    fn test_lua_config_none_when_section_missing() {
        let toml_str = r#"
[loader]
debug_mode = true
"#;
        let config = PastaConfig::from_str(toml_str).unwrap();
        assert!(config.lua().is_none());
    }

    // ========================================
    // Lua Search Path tests (lua-module-path-resolution spec)
    // ========================================

    #[test]
    fn test_default_lua_search_paths_contains_user_scripts() {
        // Requirement 1.2: user_scripts should be at priority 2 (second position)
        let paths = default_lua_search_paths();
        assert_eq!(paths.len(), 5, "Should have 5 search paths");
        assert_eq!(
            paths,
            vec![
                "profile/pasta/save/lua",
                "user_scripts",
                "scripts",
                "profile/pasta/cache/lua",
                "scriptlibs",
            ],
            "Search paths should be in correct priority order"
        );
    }

    #[test]
    fn test_default_lua_search_paths_user_scripts_priority() {
        // Requirement 1.3: user_scripts (index 1) should come before scripts (index 2)
        let paths = default_lua_search_paths();
        let user_scripts_pos = paths.iter().position(|p| p == "user_scripts");
        let scripts_pos = paths.iter().position(|p| p == "scripts");
        assert!(
            user_scripts_pos.is_some(),
            "user_scripts should be in search paths"
        );
        assert!(scripts_pos.is_some(), "scripts should be in search paths");
        assert!(
            user_scripts_pos.unwrap() < scripts_pos.unwrap(),
            "user_scripts should come before scripts for override functionality"
        );
    }

    #[test]
    fn test_loader_config_default_includes_user_scripts() {
        // Verify LoaderConfig::default() uses the new search paths
        let config = LoaderConfig::default();
        assert!(
            config
                .lua_search_paths
                .contains(&"user_scripts".to_string()),
            "Default LoaderConfig should include user_scripts"
        );
    }
}
