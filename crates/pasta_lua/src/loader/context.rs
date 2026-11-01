//! Loader context for runtime initialization.
//!
//! This module provides the LoaderContext structure that transfers
//! configuration from PastaLoader to PastaLuaRuntime.

use std::path::{Path, PathBuf};

use super::config::PastaConfig;
use super::error::LoaderError;

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

    /// 変換した Lua のキャッシュの置き場所（base_dir 相対・`[loader] transpiled_output_dir`）。
    /// 探索関数が読み込んだファイルを辞書のキャッシュと見分けるのに使う。
    pub transpiled_output_dir: String,
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
            transpiled_output_dir: super::config::LoaderConfig::default().transpiled_output_dir,
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
            transpiled_output_dir: config.loader.transpiled_output_dir.clone(),
        }
    }

    /// Reject ';' and '?' in the paths that go into `package.path`
    /// (the ghost folder, then each `lua_search_paths` entry in written order).
    /// Lua reads ';' as the template separator and '?' as the module-name marker.
    pub fn check_package_path_chars(&self) -> Result<(), LoaderError> {
        let find = |s: &str| s.chars().find(|c| matches!(c, ';' | '?'));
        let path = self.base_dir.to_string_lossy();
        if let Some(ch) = find(&path) {
            return Err(LoaderError::GhostDirPathChar {
                ch,
                path: path.into_owned(),
            });
        }
        for entry in &self.lua_search_paths {
            if let Some(ch) = find(entry) {
                return Err(LoaderError::LuaSearchPathChar {
                    ch,
                    entry: entry.clone(),
                });
            }
        }
        Ok(())
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
            ..PastaConfig::default()
        };

        let ctx = LoaderContext::from_config(Path::new("/ghost/master"), &config);
        assert_eq!(ctx.custom_fields, custom);
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
        assert!(
            ctx.base_dir
                .ends_with("definitely_nonexistent_loader_ctx_dir/sub")
        );
    }

    /// 文字列だけで組む（ファイルシステムに触らない）。
    fn ctx_with(base_dir: &str, lua_search_paths: &[&str]) -> LoaderContext {
        LoaderContext {
            base_dir: PathBuf::from(base_dir),
            lua_search_paths: lua_search_paths.iter().map(|s| s.to_string()).collect(),
            custom_fields: toml::Table::new(),
            transpiled_output_dir: "profile/pasta/cache/lua".to_string(),
        }
    }

    #[test]
    fn check_package_path_chars_rejects_ghost_dir_chars() {
        for (dir, want) in [
            (r"C:\SSP\ghost\a?b\ghost\master", '?'),
            ("/ghost/a;b/master", ';'),
        ] {
            match ctx_with(dir, &["scripts"]).check_package_path_chars() {
                Err(LoaderError::GhostDirPathChar { ch, path }) => {
                    assert_eq!(ch, want);
                    assert_eq!(path, dir);
                }
                other => panic!("expected GhostDirPathChar for {dir}: {other:?}"),
            }
        }
    }

    #[test]
    fn check_package_path_chars_rejects_search_path_entries_in_order() {
        // 最初の誤った項目の、最初に現れた文字を報告する。
        let ctx = ctx_with("/ghost/master", &["scripts", "lib?;x", "a;b"]);
        match ctx.check_package_path_chars() {
            Err(LoaderError::LuaSearchPathChar { ch, entry }) => {
                assert_eq!(ch, '?');
                assert_eq!(entry, "lib?;x");
            }
            other => panic!("expected LuaSearchPathChar: {other:?}"),
        }
    }

    #[test]
    fn check_package_path_chars_ghost_dir_first() {
        let err = ctx_with("/ghost/a;b", &["lib?"])
            .check_package_path_chars()
            .unwrap_err();
        assert!(matches!(err, LoaderError::GhostDirPathChar { ch: ';', .. }));
    }

    #[test]
    fn check_package_path_chars_accepts_other_chars() {
        let ctx = ctx_with(
            r"C:\Users\RUNNER~1\AppData\Local\Temp\ゴースト 置き場\ghost\master",
            &["profile/pasta/save/lua", "スクリプト 集", "scripts"],
        );
        assert!(ctx.check_package_path_chars().is_ok());
    }
}
