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
    /// Makes `base_dir` absolute **without resolving 8.3 short names or
    /// symlinks**, so the form is preserved exactly.
    ///
    /// # Why not `canonicalize()`
    ///
    /// `base_dir` flows into `package.path`, which determines the chunk name Lua
    /// reports for every `require`d scene (`@<package.path with ? filled>`). The
    /// debugger keys breakpoints on that chunk name via
    /// [`canonicalize_chunk_name`](crate::debug::source_map::canonicalize_chunk_name)
    /// (separator + case normalization only — it does NOT resolve 8.3 short
    /// names). The cache paths, the aggregated source map, and the breakpoint
    /// store all derive from the **raw** `base_dir` the loader was given.
    ///
    /// `std::fs::canonicalize()` resolves Windows 8.3 short names to their long
    /// form (e.g. a temp dir handed in as `C:\Users\RUNNER~1\...` becomes
    /// `C:\Users\runneradmin\...`) and symlinks. That made `package.path`
    /// (→ runtime chunk names) diverge from the cache/source-map/breakpoint form,
    /// so `should_pause` never matched and breakpoints silently never fired when
    /// the base path differed from its canonical form — reproducible on CI (whose
    /// `%TEMP%` is an 8.3 short name) and possible for any real ghost installed
    /// under an 8.3-generating path. `std::path::absolute` makes the path absolute
    /// (resolving `.`/`..`) while preserving the exact short/long form, keeping
    /// every path identity consistent. It also never emits the `\\?\`
    /// extended-length prefix, so no stripping is needed.
    pub fn from_config(base_dir: &Path, config: &PastaConfig) -> Self {
        let abs_base = std::path::absolute(base_dir).unwrap_or_else(|_| base_dir.to_path_buf());

        Self {
            base_dir: abs_base,
            lua_search_paths: config.loader.lua_search_paths.clone(),
            custom_fields: config.custom_fields.clone(),
        }
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

    #[test]
    fn test_generate_package_path_init_patterns_in_order() {
        // Each search path yields BOTH "?.lua" and "?/init.lua", in declared order.
        let ctx = LoaderContext::new(
            "/ghost/master",
            vec!["scripts".to_string(), "lib".to_string()],
            toml::Table::new(),
        );

        let path = ctx.generate_package_path();
        assert_eq!(
            path,
            "/ghost/master/scripts/?.lua;/ghost/master/scripts/?/init.lua;\
             /ghost/master/lib/?.lua;/ghost/master/lib/?/init.lua"
        );
    }

    #[test]
    fn test_generate_package_path_normalizes_backslashes() {
        // Backslash separators (Windows style) are normalized to "/" for Lua.
        let ctx = LoaderContext::new(
            r"C:\ghost\master",
            vec!["scripts".to_string()],
            toml::Table::new(),
        );

        let path = ctx.generate_package_path();
        assert!(!path.contains('\\'), "no backslashes expected: {}", path);
        assert!(path.contains("C:/ghost/master/scripts/?.lua"));
        assert!(path.contains("C:/ghost/master/scripts/?/init.lua"));
    }

    #[test]
    fn test_generate_package_path_empty_search_paths() {
        let ctx = LoaderContext::new("/ghost/master", Vec::new(), toml::Table::new());
        assert_eq!(ctx.generate_package_path(), "");
        assert!(ctx.absolute_search_paths().is_empty());
    }

    #[test]
    fn test_from_config_existing_dir_is_absolute_without_extended_prefix() {
        // Existing dir -> absolute path with NO \\?\ extended-length prefix
        // (`std::path::absolute` never emits it, so Lua's package.path stays clean).
        let temp = tempfile::TempDir::new().unwrap();
        let config = PastaConfig::default();
        let ctx = LoaderContext::from_config(temp.path(), &config);

        assert!(ctx.base_dir.is_absolute());
        assert!(
            !ctx.base_dir.to_string_lossy().starts_with(r"\\?\"),
            "extended-length prefix must not be present: {}",
            ctx.base_dir.display()
        );
    }

    #[test]
    fn test_from_config_preserves_path_form_without_resolving_short_names() {
        // `std::path::absolute` must NOT resolve 8.3 short names or symlinks (the
        // regression guard for the breakpoint chunk-identity mismatch): the base
        // must remain byte-for-byte the input's form once absolute. An already
        // absolute input is therefore returned unchanged.
        let temp = tempfile::TempDir::new().unwrap();
        let config = PastaConfig::default();
        let ctx = LoaderContext::from_config(temp.path(), &config);
        assert_eq!(
            ctx.base_dir,
            std::path::absolute(temp.path()).unwrap(),
            "base form must be preserved (no 8.3/symlink resolution)"
        );
    }

    #[test]
    fn test_from_config_makes_relative_dir_absolute() {
        // A relative (here nonexistent) path is made absolute lexically; unlike
        // `canonicalize()` this does not require the path to exist.
        let config = PastaConfig::default();
        let input = Path::new("definitely_nonexistent_loader_ctx_dir/sub");
        let ctx = LoaderContext::from_config(input, &config);
        assert!(ctx.base_dir.is_absolute());
        assert!(ctx.base_dir.ends_with("definitely_nonexistent_loader_ctx_dir/sub"));
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
