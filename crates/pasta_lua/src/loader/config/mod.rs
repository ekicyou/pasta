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
        let mut config = Self {
            loader,
            custom_fields,
        };

        // Single SHIORI-defaults completion choke point: applied exactly once,
        // here at the sole config-construction site, so both the Rust consumers
        // and the Lua exposure path (`@pasta_config` via `custom_fields`) observe
        // the same post-completion state (requirements 3.1/3.3). Do NOT call
        // `apply_shiori_defaults` anywhere else.
        config.apply_shiori_defaults();

        Ok(config)
    }

    /// Fill missing SHIORI-profile defaults into `custom_fields`.
    ///
    /// This is the single post-load completion choke point for SHIORI-profile
    /// custom-field sections. Currently it normalizes the `[ghost]` section by
    /// filling **only the missing keys** from [`GhostConfig::default()`] (the
    /// SSOT) without ever overwriting values the author wrote explicitly
    /// (requirements 3.1/3.2/3.4). If the `[ghost]` table is absent it is
    /// created and all four keys are filled (so `@pasta_config.ghost.*` always
    /// exists — requirement 3.3).
    ///
    /// The operation is **idempotent**: applying it twice yields the same
    /// result. Engine-profile-only sections (`[package]`) and every non-`[ghost]`
    /// section are left untouched. The `[ghost]` section keeps flowing through
    /// `custom_fields` to Lua — it is not extracted into a typed field.
    ///
    /// Invoked exactly once from [`PastaConfig::parse`] (the sole config
    /// construction site), just before returning, so it is the single completion
    /// choke point. It must not be called from anywhere else.
    fn apply_shiori_defaults(&mut self) {
        let defaults = GhostConfig::default();

        // Ensure the `ghost` entry exists as a table, then fill only missing
        // keys. `entry(..).or_insert_with(..)` keeps an existing table (and its
        // explicit values) intact, guaranteeing idempotence and non-override.
        let ghost = self
            .custom_fields
            .entry("ghost")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));

        if let Some(ghost) = ghost.as_table_mut() {
            ghost
                .entry("talk_interval_min")
                .or_insert_with(|| toml::Value::Integer(defaults.talk_interval_min));
            ghost
                .entry("talk_interval_max")
                .or_insert_with(|| toml::Value::Integer(defaults.talk_interval_max));
            ghost
                .entry("hour_margin")
                .or_insert_with(|| toml::Value::Integer(defaults.hour_margin));
            ghost
                .entry("spot_newlines")
                .or_insert_with(|| toml::Value::Float(defaults.spot_newlines));
        }

        // `[actor]` is the single SHIORI-required section (requirement 2.1). Its
        // value (name/spot) is ghost-specific and cannot be defaulted, so we do
        // NOT block startup when it is missing — we emit ONE lightweight warning
        // so the author can tell the omission apart (requirement 2.3). Mirrors
        // `RuntimeConfig::validate_and_warn` in tone. The message is intentionally
        // generic: no file paths, no secrets (Security Considerations).
        let has_actor_table = self
            .custom_fields
            .get("actor")
            .is_some_and(toml::Value::is_table);
        if !has_actor_table {
            tracing::warn!(
                "No [actor] section is defined in pasta.toml. \
                 At least one [actor] is required for the ghost to act as a SHIORI; \
                 without it the ghost may not function correctly."
            );
        }
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
    /// Pasta file discovery patterns (default: ["dic/**/*.pasta"])
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
    vec!["dic/**/*.pasta".to_string()]
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

/// Typed custom-field sections (`[logging]`/`[persistence]`/`[lua]`/`[talk]`/
/// `[ghost]`/`[debug]`) split out to keep this hub small. Re-exported so the
/// public surface (and `loader::mod` re-exports) stays unchanged.
mod sections;
pub use sections::*;

#[cfg(test)]
#[path = "../config_tests.rs"]
mod tests;
