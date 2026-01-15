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
}
