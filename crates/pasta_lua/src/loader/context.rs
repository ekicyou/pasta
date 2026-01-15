//! Loader context for runtime initialization.
//!
//! This module provides the LoaderContext structure that transfers
//! configuration from PastaLoader to PastaLuaRuntime.

use std::path::{Path, PathBuf};

use super::config::PastaConfig;

/// Context for runtime initialization from PastaLoader.
///
/// Contains all information needed to initialize PastaLuaRuntime
/// from the loader context, including paths and custom configuration.
#[derive(Debug, Clone)]
pub struct LoaderContext {
    /// Base directory (absolute path)
    pub base_dir: PathBuf,

    /// Lua module search paths (relative to base_dir)
    pub lua_search_paths: Vec<String>,

    /// Custom configuration fields from pasta.toml
    /// (everything except [loader] section)
    pub custom_fields: toml::Table,
}

impl LoaderContext {
    /// Create a new LoaderContext.
    pub fn new(
        base_dir: impl Into<PathBuf>,
        lua_search_paths: Vec<String>,
        custom_fields: toml::Table,
    ) -> Self {
        Self {
            base_dir: base_dir.into(),
            lua_search_paths,
            custom_fields,
        }
    }

    /// Create LoaderContext from PastaConfig.
    ///
    /// Converts base_dir to absolute path and extracts relevant settings.
    pub fn from_config(base_dir: &Path, config: &PastaConfig) -> Self {
        let abs_base = base_dir
            .canonicalize()
            .unwrap_or_else(|_| base_dir.to_path_buf());

        // On Windows, canonicalize returns extended-length paths (\\?\C:\...)
        // Strip this prefix for Lua compatibility
        let abs_base = Self::strip_windows_prefix(&abs_base);

        Self {
            base_dir: abs_base,
            lua_search_paths: config.loader.lua_search_paths.clone(),
            custom_fields: config.custom_fields.clone(),
        }
    }

    /// Strip Windows extended-length path prefix (\\?\) if present.
    ///
    /// Windows `canonicalize()` returns paths like `\\?\C:\path` which
    /// cause issues with Lua's package.path. This function removes the
    /// prefix to get a normal path like `C:\path`.
    #[cfg(windows)]
    fn strip_windows_prefix(path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
            PathBuf::from(stripped)
        } else {
            path.to_path_buf()
        }
    }

    #[cfg(not(windows))]
    fn strip_windows_prefix(path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    /// Generate absolute Lua module search paths.
    ///
    /// Converts relative search paths to absolute paths based on base_dir.
    pub fn absolute_search_paths(&self) -> Vec<PathBuf> {
        self.lua_search_paths
            .iter()
            .map(|p| self.base_dir.join(p))
            .collect()
    }

    /// Generate package.path string for Lua.
    ///
    /// Creates a semicolon-separated path string in Lua format:
    /// `/path/to/dir/?.lua;/path/to/dir/?/init.lua;/next/path/?.lua;...`
    ///
    /// Each search path generates two patterns:
    /// - `?.lua` for direct module files
    /// - `?/init.lua` for directory modules (like `pasta/init.lua`)
    pub fn generate_package_path(&self) -> String {
        self.lua_search_paths
            .iter()
            .flat_map(|p| {
                let abs_path = self.base_dir.join(p);
                // Normalize path separators to forward slashes for Lua
                let path_str = abs_path.to_string_lossy().replace('\\', "/");
                // Return both patterns for each search path
                vec![
                    format!("{}/?.lua", path_str),
                    format!("{}/?/init.lua", path_str),
                ]
            })
            .collect::<Vec<_>>()
            .join(";")
    }

    /// Generate package.path bytes for Lua.
    ///
    /// Creates a semicolon-separated path string and converts it to
    /// system-native encoding bytes (ANSI on Windows, UTF-8 on Unix).
    ///
    /// This is the preferred method for setting `package.path` in Lua
    /// because Lua's file I/O functions (fopen) use ANSI encoding on Windows.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Encoded path bytes ready for Lua
    /// * `Err(std::io::Error)` - If encoding conversion fails
    ///
    /// # Example
    /// ```rust,ignore
    /// let bytes = loader_context.generate_package_path_bytes()?;
    /// let lua_string = lua.create_string(&bytes)?;
    /// package.set("path", lua_string)?;
    /// ```
    pub fn generate_package_path_bytes(&self) -> std::io::Result<Vec<u8>> {
        let path_str = self.generate_package_path();
        crate::encoding::to_ansi_bytes(&path_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ctx = LoaderContext::new(
            "/ghost/master",
            vec!["scripts".to_string(), "lib".to_string()],
            toml::Table::new(),
        );

        assert_eq!(ctx.base_dir, PathBuf::from("/ghost/master"));
        assert_eq!(ctx.lua_search_paths, vec!["scripts", "lib"]);
        assert!(ctx.custom_fields.is_empty());
    }

    #[test]
    fn test_absolute_search_paths() {
        let ctx = LoaderContext::new(
            "/ghost/master",
            vec!["profile/pasta/save/lua".to_string(), "scripts".to_string()],
            toml::Table::new(),
        );

        let paths = ctx.absolute_search_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[0],
            PathBuf::from("/ghost/master/profile/pasta/save/lua")
        );
        assert_eq!(paths[1], PathBuf::from("/ghost/master/scripts"));
    }

    #[test]
    fn test_generate_package_path() {
        let ctx = LoaderContext::new(
            "/ghost/master",
            vec!["scripts".to_string(), "lib".to_string()],
            toml::Table::new(),
        );

        let path = ctx.generate_package_path();
        assert!(path.contains("/ghost/master/scripts/?.lua"));
        assert!(path.contains("/ghost/master/lib/?.lua"));
        assert!(path.contains(";"));
    }

    #[test]
    fn test_from_config() {
        let config = PastaConfig::default();
        let ctx = LoaderContext::from_config(Path::new("/ghost/master"), &config);

        assert_eq!(ctx.lua_search_paths, config.loader.lua_search_paths);
        assert!(ctx.custom_fields.is_empty());
    }

    #[test]
    fn test_from_config_with_custom_fields() {
        let mut custom = toml::Table::new();
        custom.insert(
            "ghost_name".to_string(),
            toml::Value::String("Test".to_string()),
        );

        let config = PastaConfig {
            loader: super::super::config::LoaderConfig::default(),
            custom_fields: custom.clone(),
        };

        let ctx = LoaderContext::from_config(Path::new("/ghost/master"), &config);
        assert_eq!(ctx.custom_fields, custom);
    }

    #[test]
    fn test_generate_package_path_bytes_ascii() {
        let ctx = LoaderContext::new(
            "/ghost/master",
            vec!["scripts".to_string()],
            toml::Table::new(),
        );

        let bytes = ctx.generate_package_path_bytes().unwrap();
        // ASCII paths should be unchanged
        let expected = ctx.generate_package_path();
        assert_eq!(bytes, expected.as_bytes());
    }

    #[test]
    fn test_generate_package_path_bytes_not_empty() {
        let ctx = LoaderContext::new(
            "/ghost/master",
            vec!["scripts".to_string(), "lib".to_string()],
            toml::Table::new(),
        );

        let bytes = ctx.generate_package_path_bytes().unwrap();
        assert!(!bytes.is_empty());
        // Should contain semicolon separator
        assert!(bytes.contains(&b';'));
    }

    #[cfg(windows)]
    #[test]
    fn test_generate_package_path_bytes_japanese() {
        let ctx = LoaderContext::new(
            "C:\\ユーザー\\テスト",
            vec!["scripts".to_string()],
            toml::Table::new(),
        );

        let bytes = ctx.generate_package_path_bytes().unwrap();
        // On Windows with Japanese locale, bytes should be ANSI encoded
        assert!(!bytes.is_empty());
        // The result should not be the same as UTF-8 bytes
        let utf8_bytes = ctx.generate_package_path().into_bytes();
        assert_ne!(bytes, utf8_bytes);
    }
}
