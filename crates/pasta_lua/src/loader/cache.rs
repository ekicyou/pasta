//! Cache Manager for incremental transpilation.
//!
//! This module provides the CacheManager struct which manages the lifecycle
//! of transpiled Lua cache files, including version checking, timestamp
//! comparison, and scene_dic.lua generation.

use super::LoaderError;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Cache version file name.
const CACHE_VERSION_FILE: &str = ".cache_version";

/// Current cache version from Cargo.toml.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

            if cached_version.trim() != CURRENT_VERSION {
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
            } else {
                debug!(version = %CURRENT_VERSION, "Cache version matches, preserving cache");
                // Ensure scene directory exists
                let scene_dir = self.cache_dir.join("pasta/scene");
                if !scene_dir.exists() {
                    fs::create_dir_all(&scene_dir)
                        .map_err(|e| LoaderError::cache_directory(&scene_dir, e))?;
                }
                return Ok(());
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

        // Create parent directories if needed
        if let Some(parent) = cache_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| LoaderError::cache_write(&cache_path, e))?;
            }
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
    fn get_relative_path(&self, source_path: &Path) -> String {
        source_path
            .strip_prefix(&self.base_dir)
            .unwrap_or(source_path)
            .to_string_lossy()
            .to_string()
    }

    /// Generate scene_dic.lua from module names.
    ///
    /// Creates a Lua file that requires all scene modules and calls
    /// finalize_scene() at the end.
    ///
    /// # Arguments
    /// * `module_names` - List of all module names to require
    ///
    /// # Returns
    /// * `Ok(path)` - Path to generated scene_dic.lua
    /// * `Err(LoaderError::SceneDicGenerationError)` - Generation failed
    pub fn generate_scene_dic(&self, module_names: &[String]) -> Result<PathBuf, LoaderError> {
        let scene_dic_path = self.cache_dir.join("pasta/scene_dic.lua");

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
                let path = entry.path();
                if path.is_dir() {
                    Self::walk_lua_files(&path, expected, orphans);
                } else if path.extension().map_or(false, |e| e == "lua") {
                    // Skip scene_dic.lua
                    if path.file_name().map_or(false, |n| n == "scene_dic.lua") {
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
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache_manager() -> (TempDir, CacheManager) {
        let temp = TempDir::new().unwrap();
        let manager = CacheManager::new(temp.path().to_path_buf(), "profile/pasta/cache/lua");
        (temp, manager)
    }

    // ========================================================================
    // Version Management Tests (Task 1.2)
    // ========================================================================

    #[test]
    fn test_prepare_cache_dir_creates_version_file() {
        let (temp, manager) = create_test_cache_manager();

        // First call should create version file
        manager.prepare_cache_dir().unwrap();

        let version_file = temp.path().join("profile/pasta/cache/lua/.cache_version");
        assert!(version_file.exists());
        assert_eq!(fs::read_to_string(&version_file).unwrap(), CURRENT_VERSION);
    }

    #[test]
    fn test_prepare_cache_dir_preserves_cache_on_version_match() {
        let (temp, manager) = create_test_cache_manager();

        // Create initial cache
        manager.prepare_cache_dir().unwrap();

        // Create a test file in cache
        let test_file = temp
            .path()
            .join("profile/pasta/cache/lua/pasta/scene/test.lua");
        fs::write(&test_file, "-- test content").unwrap();

        // Call again - should preserve
        manager.prepare_cache_dir().unwrap();

        // Test file should still exist
        assert!(test_file.exists());
        assert_eq!(fs::read_to_string(&test_file).unwrap(), "-- test content");
    }

    #[test]
    fn test_prepare_cache_dir_clears_on_version_mismatch() {
        let (temp, manager) = create_test_cache_manager();

        // Create initial cache with old version
        let cache_dir = temp.path().join("profile/pasta/cache/lua");
        fs::create_dir_all(cache_dir.join("pasta/scene")).unwrap();
        fs::write(cache_dir.join(".cache_version"), "0.0.0-old").unwrap();

        // Create a test file
        let test_file = cache_dir.join("pasta/scene/test.lua");
        fs::write(&test_file, "-- old content").unwrap();

        // Call prepare - should clear
        manager.prepare_cache_dir().unwrap();

        // Test file should be deleted
        assert!(!test_file.exists());

        // Version file should be updated
        let version_file = cache_dir.join(".cache_version");
        assert_eq!(fs::read_to_string(&version_file).unwrap(), CURRENT_VERSION);
    }

    // ========================================================================
    // Change Detection Tests (Task 2.2)
    // ========================================================================

    #[test]
    fn test_needs_transpile_no_cache() {
        let (temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        // Create source file
        let dic_dir = temp.path().join("dic");
        fs::create_dir_all(&dic_dir).unwrap();
        let source = dic_dir.join("test.pasta");
        fs::write(&source, "# test").unwrap();

        // No cache exists
        assert!(manager.needs_transpile(&source).unwrap());
    }

    #[test]
    fn test_needs_transpile_cache_older() {
        let (temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        // Create source file
        let dic_dir = temp.path().join("dic");
        fs::create_dir_all(&dic_dir).unwrap();
        let source = dic_dir.join("test.pasta");

        // Create cache first (older)
        let cache_path = manager.source_to_cache_path(&source);
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        fs::write(&cache_path, "-- cached").unwrap();

        // Wait a bit and create source (newer)
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&source, "# updated").unwrap();

        // Source is newer
        assert!(manager.needs_transpile(&source).unwrap());
    }

    #[test]
    fn test_needs_transpile_cache_newer() {
        let (temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        // Create source file first
        let dic_dir = temp.path().join("dic");
        fs::create_dir_all(&dic_dir).unwrap();
        let source = dic_dir.join("test.pasta");
        fs::write(&source, "# test").unwrap();

        // Wait a bit and create cache (newer)
        std::thread::sleep(std::time::Duration::from_millis(50));
        let cache_path = manager.source_to_cache_path(&source);
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        fs::write(&cache_path, "-- cached").unwrap();

        // Cache is newer
        assert!(!manager.needs_transpile(&source).unwrap());
    }

    // ========================================================================
    // Path Conversion Tests (Task 2.3)
    // ========================================================================

    #[test]
    fn test_source_to_module_name_simple() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/system.pasta");
        assert_eq!(manager.source_to_module_name(&source), "pasta.scene.system");
    }

    #[test]
    fn test_source_to_module_name_nested() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/baseware/greet.pasta");
        assert_eq!(
            manager.source_to_module_name(&source),
            "pasta.scene.baseware.greet"
        );
    }

    #[test]
    fn test_source_to_module_name_deep_nested() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/dialog/npc/shopkeeper.pasta");
        assert_eq!(
            manager.source_to_module_name(&source),
            "pasta.scene.dialog.npc.shopkeeper"
        );
    }

    #[test]
    fn test_source_to_module_name_with_hyphen() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/my-scene.pasta");
        assert_eq!(
            manager.source_to_module_name(&source),
            "pasta.scene.my_scene"
        );
    }

    #[test]
    fn test_source_to_module_name_japanese() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/挨拶/朝.pasta");
        assert_eq!(
            manager.source_to_module_name(&source),
            "pasta.scene.挨拶.朝"
        );
    }

    #[test]
    fn test_source_to_cache_path_simple() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/system.pasta");
        let expected = temp
            .path()
            .join("profile/pasta/cache/lua/pasta/scene/system.lua");
        assert_eq!(manager.source_to_cache_path(&source), expected);
    }

    #[test]
    fn test_source_to_cache_path_nested() {
        let (temp, manager) = create_test_cache_manager();
        let source = temp.path().join("dic/baseware/greet.pasta");
        let expected = temp
            .path()
            .join("profile/pasta/cache/lua/pasta/scene/baseware/greet.lua");
        assert_eq!(manager.source_to_cache_path(&source), expected);
    }

    // ========================================================================
    // Cache Save Tests (Task 2.4)
    // ========================================================================

    #[test]
    fn test_save_cache_creates_file() {
        let (temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        let source = temp.path().join("dic/test.pasta");
        let lua_code = "-- generated lua code";

        let module_name = manager.save_cache(&source, lua_code).unwrap();

        assert_eq!(module_name, "pasta.scene.test");

        let cache_path = manager.source_to_cache_path(&source);
        assert!(cache_path.exists());
        assert_eq!(fs::read_to_string(&cache_path).unwrap(), lua_code);
    }

    #[test]
    fn test_save_cache_creates_nested_dirs() {
        let (temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        let source = temp.path().join("dic/deep/nested/path/scene.pasta");
        let lua_code = "-- nested lua";

        manager.save_cache(&source, lua_code).unwrap();

        let cache_path = manager.source_to_cache_path(&source);
        assert!(cache_path.exists());
    }

    // ========================================================================
    // scene_dic.lua Generation Tests (Task 2.5)
    // ========================================================================

    #[test]
    fn test_generate_scene_dic() {
        let (_temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        let modules = vec![
            "pasta.scene.system".to_string(),
            "pasta.scene.baseware.greet".to_string(),
        ];

        let path = manager.generate_scene_dic(&modules).unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();

        // Check header
        assert!(content.contains("Auto-generated by pasta_lua CacheManager"));

        // Check sorted requires
        assert!(content.contains("require(\"pasta.scene.baseware.greet\")"));
        assert!(content.contains("require(\"pasta.scene.system\")"));

        // Check finalize_scene call
        assert!(content.contains("require(\"pasta\").finalize_scene()"));

        // Check sorting (baseware should come before system)
        let pos_baseware = content.find("pasta.scene.baseware.greet").unwrap();
        let pos_system = content.find("pasta.scene.system").unwrap();
        assert!(pos_baseware < pos_system);
    }

    #[test]
    fn test_generate_scene_dic_empty() {
        let (_temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        let modules: Vec<String> = vec![];
        let path = manager.generate_scene_dic(&modules).unwrap();

        let content = fs::read_to_string(&path).unwrap();

        // Should still have header and finalize_scene
        assert!(content.contains("Auto-generated"));
        assert!(content.contains("require(\"pasta\").finalize_scene()"));
    }

    // ========================================================================
    // Orphan Detection Tests
    // ========================================================================

    #[test]
    fn test_find_orphaned_caches() {
        let (temp, manager) = create_test_cache_manager();
        manager.prepare_cache_dir().unwrap();

        // Create some cache files
        let scene_dir = temp.path().join("profile/pasta/cache/lua/pasta/scene");
        fs::write(scene_dir.join("active.lua"), "-- active").unwrap();
        fs::write(scene_dir.join("orphan.lua"), "-- orphan").unwrap();

        // Only active.pasta exists
        let source_paths = vec![temp.path().join("dic/active.pasta")];

        let orphans = manager.find_orphaned_caches(&source_paths);

        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].ends_with("orphan.lua"));
    }
}
