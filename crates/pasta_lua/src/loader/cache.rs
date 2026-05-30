//! Cache Manager for incremental transpilation.
//!
//! This module provides the CacheManager struct which manages the lifecycle
//! of transpiled Lua cache files, including version checking, timestamp
//! comparison, and scene_dic.lua generation.

use super::LoaderError;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Cache version file name.
const CACHE_VERSION_FILE: &str = ".cache_version";

/// Current cache version from Cargo.toml.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cache Manager for incremental transpilation.
///
/// Manages the lifecycle of transpiled Lua cache files:
/// - Version checking and cache invalidation
/// - Timestamp comparison for change detection
/// - Cache file saving with directory structure
/// - scene_dic.lua generation
pub struct CacheManager {
    /// Project base directory (ghost/master/).
    base_dir: PathBuf,
    /// Cache output directory (e.g., profile/pasta/cache/lua).
    cache_dir: PathBuf,
    /// Relative path from base_dir to dic directory.
    dic_prefix: String,
}

impl CacheManager {
    /// Create a new CacheManager.
    ///
    /// # Arguments
    /// * `base_dir` - Project root directory
    /// * `output_dir` - Cache output directory relative to base_dir (e.g., "profile/pasta/cache/lua")
    pub fn new(base_dir: PathBuf, output_dir: &str) -> Self {
        let cache_dir = base_dir.join(output_dir);
        Self {
            base_dir,
            cache_dir,
            dic_prefix: "dic".to_string(),
        }
    }

    /// Prepare the cache directory with version checking.
    ///
    /// If the pasta_lua version has changed, all cache files are cleared.
    /// Otherwise, existing cache is preserved for incremental updates.
    ///
    /// # Returns
    /// * `Ok(())` - Directory prepared successfully
    /// * `Err(LoaderError::CacheDirectoryError)` - Directory operation failed
    pub fn prepare_cache_dir(&self) -> Result<(), LoaderError> {
        let version_file = self.cache_dir.join(CACHE_VERSION_FILE);

        // Check version if cache directory exists
        if version_file.exists() {
            let cached_version = fs::read_to_string(&version_file)
                .map_err(|e| LoaderError::cache_directory(&version_file, e))?;

            if cached_version.trim() == CURRENT_VERSION {
                debug!(version = %CURRENT_VERSION, "Cache version matches, preserving cache");
                // Ensure scene directory exists (idempotent)
                let scene_dir = self.cache_dir.join("pasta/scene");
                fs::create_dir_all(&scene_dir)
                    .map_err(|e| LoaderError::cache_directory(&scene_dir, e))?;
                return Ok(());
            }

            // Version mismatch → clear all cache
            info!(
                old_version = %cached_version.trim(),
                new_version = %CURRENT_VERSION,
                "Cache version mismatch, clearing all cache"
            );
            if self.cache_dir.exists() {
                fs::remove_dir_all(&self.cache_dir)
                    .map_err(|e| LoaderError::cache_directory(&self.cache_dir, e))?;
            }
        }

        // Create directory structure (idempotent)
        let scene_dir = self.cache_dir.join("pasta/scene");
        fs::create_dir_all(&scene_dir).map_err(|e| LoaderError::cache_directory(&scene_dir, e))?;

        // Write version file
        fs::write(&version_file, CURRENT_VERSION)
            .map_err(|e| LoaderError::cache_directory(&version_file, e))?;

        debug!(version = %CURRENT_VERSION, "Created cache directory with version file");
        Ok(())
    }

    /// Check if a source file needs transpilation.
    ///
    /// Compares the source file's modification time with the cache file's.
    /// Returns true if:
    /// - Cache file doesn't exist
    /// - Source file is newer than cache file
    ///
    /// # Arguments
    /// * `source_path` - Path to the .pasta source file
    ///
    /// # Returns
    /// * `Ok(true)` - Transpilation needed
    /// * `Ok(false)` - Cache is up-to-date
    /// * `Err(LoaderError::MetadataError)` - Failed to get file metadata
    pub fn needs_transpile(&self, source_path: &Path) -> Result<bool, LoaderError> {
        let cache_path = self.source_to_cache_path(source_path);

        // If cache doesn't exist, need transpile
        if !cache_path.exists() {
            debug!(source = %source_path.display(), "Cache not found, needs transpile");
            return Ok(true);
        }

        // Get source modification time
        let source_mtime = Self::get_mtime(source_path)?;

        // Get cache modification time
        let cache_mtime = Self::get_mtime(&cache_path)?;

        // Compare timestamps
        let needs = source_mtime > cache_mtime;
        if needs {
            debug!(
                source = %source_path.display(),
                "Source newer than cache, needs transpile"
            );
        } else {
            debug!(
                source = %source_path.display(),
                "Cache up-to-date, skipping"
            );
        }

        Ok(needs)
    }

    /// Get file modification time.
    fn get_mtime(path: &Path) -> Result<SystemTime, LoaderError> {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .map_err(|e| LoaderError::metadata(path, e))
    }

    /// Save transpiled Lua code to cache.
    ///
    /// Creates necessary directory structure and writes the file.
    ///
    /// # Arguments
    /// * `source_path` - Original .pasta source file path
    /// * `lua_code` - Transpiled Lua code
    ///
    /// # Returns
    /// * `Ok(module_name)` - Module name (e.g., "pasta.scene.baseware.system")
    /// * `Err(LoaderError::CacheWriteError)` - Write failed
    pub fn save_cache(&self, source_path: &Path, lua_code: &str) -> Result<String, LoaderError> {
        let cache_path = self.source_to_cache_path(source_path);
        let module_name = self.source_to_module_name(source_path);

        // Validate cache path stays within cache directory (prevent directory traversal)
        if !cache_path.starts_with(&self.cache_dir) {
            return Err(LoaderError::cache_write(
                &cache_path,
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "Cache path escapes cache directory (directory traversal detected): {}",
                        cache_path.display()
                    ),
                ),
            ));
        }

        // Create parent directories if needed (idempotent)
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).map_err(|e| LoaderError::cache_write(&cache_path, e))?;
        }

        // Write UTF-8 encoded Lua code
        fs::write(&cache_path, lua_code).map_err(|e| LoaderError::cache_write(&cache_path, e))?;

        debug!(
            path = %cache_path.display(),
            module = %module_name,
            "Saved cache file"
        );

        Ok(module_name)
    }

    /// Convert source path to module name.
    ///
    /// # Example
    /// `dic/baseware/system.pasta` → `pasta.scene.baseware.system`
    /// `dic/dialog/npc/shopkeeper.pasta` → `pasta.scene.dialog.npc.shopkeeper`
    pub fn source_to_module_name(&self, source_path: &Path) -> String {
        let relative = self.get_relative_path(source_path);

        // Remove dic/ prefix and .pasta extension
        let without_prefix = relative.strip_prefix(&self.dic_prefix).unwrap_or(&relative);
        let without_prefix = without_prefix
            .strip_prefix('/')
            .or_else(|| without_prefix.strip_prefix('\\'))
            .unwrap_or(without_prefix);

        let stem = Path::new(without_prefix).with_extension("");
        let stem_str = stem.to_string_lossy();

        // Convert path separators to dots, hyphens to underscores
        let module_path = stem_str.replace(['/', '\\'], ".").replace('-', "_");

        format!("pasta.scene.{}", module_path)
    }

    /// Convert source path to cache file path.
    ///
    /// # Example
    /// `dic/baseware/system.pasta` → `{cache_dir}/pasta/scene/baseware/system.lua`
    pub fn source_to_cache_path(&self, source_path: &Path) -> PathBuf {
        let relative = self.get_relative_path(source_path);

        // Remove dic/ prefix
        let without_prefix = relative.strip_prefix(&self.dic_prefix).unwrap_or(&relative);
        let without_prefix = without_prefix
            .strip_prefix('/')
            .or_else(|| without_prefix.strip_prefix('\\'))
            .unwrap_or(without_prefix);

        // Replace extension and convert hyphens
        let lua_path = without_prefix.replace('-', "_");
        let lua_path = Path::new(&lua_path).with_extension("lua");

        self.cache_dir.join("pasta/scene").join(lua_path)
    }

    /// Get relative path from base_dir.
    ///
    /// Returns the path relative to base_dir. If the source path contains
    /// directory traversal components (`..`), they are preserved in the
    /// relative path string (caught later by `save_cache` boundary check).
    fn get_relative_path(&self, source_path: &Path) -> String {
        let relative = source_path
            .strip_prefix(&self.base_dir)
            .unwrap_or(source_path);

        // Warn if relative path contains traversal components
        if relative.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        }) {
            warn!(
                path = %source_path.display(),
                "Source path contains directory traversal components"
            );
        }

        relative.to_string_lossy().to_string()
    }

    /// Generate scene_dic.lua from module names.
    ///
    /// Creates a Lua file that requires all scene modules and calls
    /// finalize_scene() at the end.
    ///
    /// The generated file is placed at `cache/pasta/scene_dic.lua` to be
    /// resolved via `require("pasta.scene_dic")`.
    ///
    /// # Arguments
    /// * `module_names` - List of all module names to require
    ///
    /// # Returns
    /// * `Ok(path)` - Path to generated scene_dic.lua
    /// * `Err(LoaderError::SceneDicGenerationError)` - Generation failed
    pub fn generate_scene_dic(&self, module_names: &[String]) -> Result<PathBuf, LoaderError> {
        // Create pasta subdirectory in cache (lua-module-path-resolution spec)
        let pasta_dir = self.cache_dir.join("pasta");
        fs::create_dir_all(&pasta_dir).map_err(|e| {
            LoaderError::scene_dic_generation("Failed to create pasta directory", Some(e))
        })?;

        // Clean up old scene_dic.lua location (backward compatibility)
        // Old location: cache_dir/scene_dic.lua (without pasta/ prefix)
        let old_scene_dic_path = self.cache_dir.join("scene_dic.lua");
        if old_scene_dic_path.exists() {
            if let Err(e) = fs::remove_file(&old_scene_dic_path) {
                warn!(
                    path = %old_scene_dic_path.display(),
                    error = %e,
                    "Failed to remove old scene_dic.lua, continuing"
                );
            } else {
                debug!(path = %old_scene_dic_path.display(), "Removed old scene_dic.lua");
            }
        }

        // New location: cache_dir/pasta/scene_dic.lua
        let scene_dic_path = pasta_dir.join("scene_dic.lua");

        // Sort module names alphabetically
        let mut sorted_modules = module_names.to_vec();
        sorted_modules.sort();

        // Generate timestamp using SystemTime
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Generate content
        let mut content = String::new();
        content.push_str("-- Auto-generated by pasta_lua CacheManager\n");
        content.push_str("-- Do not edit manually\n");
        content.push_str(&format!(
            "-- Generated at: {} (unix timestamp)\n\n",
            timestamp
        ));

        for module in &sorted_modules {
            content.push_str(&format!("require(\"{}\")\n", module));
        }

        content.push_str("\nrequire(\"pasta\").finalize_scene()\n");

        // Write file
        fs::write(&scene_dic_path, &content).map_err(|e| {
            LoaderError::scene_dic_generation("Failed to write scene_dic.lua", Some(e))
        })?;

        info!(
            path = %scene_dic_path.display(),
            modules = sorted_modules.len(),
            "Generated scene_dic.lua"
        );

        Ok(scene_dic_path)
    }

    /// Scan for orphaned cache files (cache files without corresponding source).
    ///
    /// Returns list of orphaned cache file paths. Does not delete them.
    ///
    /// # Arguments
    /// * `source_paths` - List of current source file paths
    ///
    /// # Returns
    /// List of orphaned cache file paths
    pub fn find_orphaned_caches(&self, source_paths: &[PathBuf]) -> Vec<PathBuf> {
        let scene_dir = self.cache_dir.join("pasta/scene");
        if !scene_dir.exists() {
            return Vec::new();
        }

        // Build set of expected cache paths
        let expected_caches: std::collections::HashSet<PathBuf> = source_paths
            .iter()
            .map(|p| self.source_to_cache_path(p))
            .collect();

        // Walk cache directory and find orphans
        let mut orphans = Vec::new();
        Self::walk_lua_files(&scene_dir, &expected_caches, &mut orphans);

        if !orphans.is_empty() {
            warn!(
                count = orphans.len(),
                "Found orphaned cache files (source files deleted)"
            );
            for orphan in &orphans {
                warn!(path = %orphan.display(), "Orphaned cache file");
            }
        }

        orphans
    }

    /// Recursively walk directory for .lua files.
    fn walk_lua_files(
        dir: &Path,
        expected: &std::collections::HashSet<PathBuf>,
        orphans: &mut Vec<PathBuf>,
    ) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };

                if file_type.is_symlink() {
                    continue;
                }

                let path = entry.path();
                if file_type.is_dir() {
                    Self::walk_lua_files(&path, expected, orphans);
                } else if file_type.is_file() && path.extension().is_some_and(|e| e == "lua") {
                    // Skip scene_dic.lua
                    if path.file_name().is_some_and(|n| n == "scene_dic.lua") {
                        continue;
                    }
                    if !expected.contains(&path) {
                        orphans.push(path);
                    }
                }
            }
        }
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::CacheManager;
    #[cfg(unix)]
    use std::collections::HashSet;
    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use tempfile::TempDir;

    #[cfg(unix)]
    #[test]
    fn walk_lua_files_skips_symlinks() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("scene");
        let external = temp.path().join("external");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir_all(&external).unwrap();
        let real = root.join("real.lua");
        let nested = root.join("nested").join("nested.lua");
        let scene_dic = root.join("scene_dic.lua");
        let secret = external.join("secret.lua");
        fs::write(&real, "print('real')").unwrap();
        fs::write(&nested, "print('nested')").unwrap();
        fs::write(&scene_dic, "return {}").unwrap();
        fs::write(&secret, "print('secret')").unwrap();
        unix_fs::symlink(&real, root.join("link.lua")).unwrap();
        unix_fs::symlink(&external, root.join("linked_dir")).unwrap();

        let expected = HashSet::new();
        let mut orphans = Vec::new();
        CacheManager::walk_lua_files(&root, &expected, &mut orphans);

        assert_eq!(orphans.len(), 2);
        assert!(orphans.contains(&real));
        assert!(orphans.contains(&nested));
        assert!(!orphans.contains(&scene_dic));
        assert!(!orphans.iter().any(|path| path.ends_with("link.lua")));
        assert!(!orphans.iter().any(|path| path.ends_with("secret.lua")));
    }
}
