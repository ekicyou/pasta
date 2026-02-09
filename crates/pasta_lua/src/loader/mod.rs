//! Pasta Loader - Startup sequence orchestration.
//!
//! This module provides the integrated startup sequence for pasta_lua,
//! from directory discovery to runtime initialization.
//!
//! # Architecture
//!
//! - `PastaLoader` - Main entry point for startup sequence
//! - `PastaConfig` / `LoaderConfig` - Configuration file handling
//! - `LoaderContext` - Runtime initialization context
//! - `LoaderError` - Error types for startup sequence
//!
//! # Example
//!
//! ```rust,ignore
//! use pasta_lua::loader::PastaLoader;
//!
//! // Load from startup directory
//! let runtime = PastaLoader::load("ghost/master/")?;
//!
//! // Execute a scene
//! let result = runtime.exec("some_scene_call()")?;
//! ```

mod cache;
mod config;
mod context;
mod discovery;
mod error;

pub use cache::CacheManager;
pub use config::{
    LoaderConfig, LoggingConfig, LuaConfig, PastaConfig, PersistenceConfig, TalkConfig,
    default_libs,
};
pub use context::LoaderContext;
pub use error::{LoaderError, TranspileFailure};

use crate::context::TranspileContext;
use crate::runtime::{PastaLuaRuntime, RuntimeConfig};
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Process statistics for logging.
struct ProcessStats {
    transpiled: usize,
    skipped: usize,
    failed: usize,
    copied: usize,
}

/// Pasta Loader - Unified startup sequence API.
///
/// Orchestrates the complete startup sequence:
/// 1. Load configuration from pasta.toml
/// 2. Prepare profile directories and cache with version check
/// 3. Discover .pasta files in dic/*/*.pasta
/// 4. Incremental transpile (only changed files)
/// 5. Generate scene_dic.lua
/// 6. Initialize PastaLuaRuntime and load scene_dic
pub struct PastaLoader;

impl PastaLoader {
    /// Load runtime from startup directory with default configuration.
    ///
    /// This is the main entry point for the startup sequence.
    /// Executes all phases: config → discovery → transpile → runtime.
    ///
    /// # Arguments
    /// * `base_dir` - Startup directory path (ghost/master/ equivalent)
    ///
    /// # Returns
    /// * `Ok(PastaLuaRuntime)` - Initialized runtime ready for execution
    /// * `Err(LoaderError)` - Startup sequence failed
    pub fn load(base_dir: impl AsRef<Path>) -> Result<PastaLuaRuntime, LoaderError> {
        Self::load_with_config(base_dir, RuntimeConfig::new())
    }

    /// Load runtime from startup directory with custom runtime configuration.
    ///
    /// # Arguments
    /// * `base_dir` - Startup directory path
    /// * `runtime_config` - Custom runtime configuration
    ///
    /// # Returns
    /// * `Ok(PastaLuaRuntime)` - Initialized runtime
    /// * `Err(LoaderError)` - Startup sequence failed
    pub fn load_with_config(
        base_dir: impl AsRef<Path>,
        runtime_config: RuntimeConfig,
    ) -> Result<PastaLuaRuntime, LoaderError> {
        let base_dir = base_dir.as_ref();

        // Check if base directory exists
        if !base_dir.exists() {
            return Err(LoaderError::DirectoryNotFound(base_dir.to_path_buf()));
        }

        info!(path = %base_dir.display(), "Starting pasta loader");

        // Phase 1: Load configuration
        debug!("Phase 1: Loading configuration");
        let config = PastaConfig::load(base_dir)?;

        // Phase 2: Prepare directories and cache (with version check)
        debug!("Phase 2: Preparing directories and cache");
        Self::prepare_directories(base_dir, &config.loader)?;
        let cache_manager =
            CacheManager::new(base_dir.to_path_buf(), &config.loader.transpiled_output_dir);
        cache_manager.prepare_cache_dir()?;

        // Phase 3: Discover files
        debug!("Phase 3: Discovering pasta and lua files");
        let (pasta_files, lua_files) =
            Self::discover_all_files(base_dir, &config.loader.pasta_patterns)?;
        let total_files = pasta_files.len() + lua_files.len();
        if total_files == 0 {
            warn!(path = %base_dir.display(), "No .pasta or .lua files found");
        } else {
            info!(
                pasta = pasta_files.len(),
                lua = lua_files.len(),
                "Found files"
            );
        }

        // Phase 4: Incremental process (transpile .pasta, copy .lua)
        debug!("Phase 4: Incremental processing");
        let (context, module_names, stats) =
            Self::process_incremental(base_dir, &pasta_files, &lua_files, &cache_manager)?;

        // Log statistics in debug mode
        if config.loader.debug_mode {
            info!(
                transpiled = stats.transpiled,
                skipped = stats.skipped,
                failed = stats.failed,
                copied = stats.copied,
                "Processing statistics"
            );
        }

        // Check for orphaned caches
        let all_source_files: Vec<_> = pasta_files
            .iter()
            .chain(lua_files.iter())
            .cloned()
            .collect();
        let orphans = cache_manager.find_orphaned_caches(&all_source_files);
        if !orphans.is_empty() && config.loader.debug_mode {
            for orphan in &orphans {
                warn!(path = %orphan.display(), "Orphaned cache file detected");
            }
        }

        // Phase 5: Generate scene_dic.lua
        debug!("Phase 5: Generating scene_dic.lua");
        let scene_dic_path = cache_manager.generate_scene_dic(&module_names)?;

        // Phase 6: Create logger
        debug!("Phase 6: Creating instance logger");
        let logger = Self::create_logger(base_dir, &config)?;

        // Phase 7: Initialize runtime and load scene_dic
        debug!("Phase 7: Initializing runtime");
        let loader_context = LoaderContext::from_config(base_dir, &config);
        let runtime = PastaLuaRuntime::from_loader_with_scene_dic(
            context,
            loader_context,
            runtime_config,
            Some(config),
            logger,
            &scene_dic_path,
        )?;

        info!(path = %base_dir.display(), "Startup sequence completed");
        Ok(runtime)
    }

    /// Create an instance-specific logger from configuration.
    ///
    /// Returns None if logging directory cannot be created (optional feature).
    /// Logger is wrapped in Arc for sharing with GlobalLoggerRegistry.
    fn create_logger(
        base_dir: &Path,
        config: &PastaConfig,
    ) -> Result<Option<std::sync::Arc<crate::logging::PastaLogger>>, LoaderError> {
        let logging_config = config.logging();

        match crate::logging::PastaLogger::new(base_dir, logging_config.as_ref()) {
            Ok(logger) => {
                info!(path = %logger.log_path().display(), "Created instance logger");
                Ok(Some(std::sync::Arc::new(logger)))
            }
            Err(e) => {
                // Log warning but don't fail startup
                warn!(error = %e, "Failed to create instance logger, logging disabled");
                Ok(None)
            }
        }
    }

    /// Prepare profile directories.
    ///
    /// Note: Cache directory management (including version-based clearing)
    /// is now handled by CacheManager::prepare_cache_dir().
    fn prepare_directories(base_dir: &Path, config: &LoaderConfig) -> Result<(), LoaderError> {
        let dirs = [
            "profile/pasta/save",
            "profile/pasta/save/lua",
            "profile/pasta/cache",
            &config.transpiled_output_dir,
        ];

        for dir in &dirs {
            let path = base_dir.join(dir);
            if !path.exists() {
                fs::create_dir_all(&path).map_err(|e| LoaderError::io(&path, e))?;
                debug!(path = %path.display(), "Created directory");
            }
        }

        // Note: Cache clearing is now handled by CacheManager with version checking
        // We no longer unconditionally delete cache/lua directory

        Ok(())
    }

    /// Discover all files (.pasta and .lua) with conflict checking.
    ///
    /// Returns (pasta_files, lua_files) where lua_files has conflicts removed.
    /// Also checks for invalid filenames (init.lua, init.pasta).
    fn discover_all_files(
        base_dir: &Path,
        pasta_patterns: &[String],
    ) -> Result<(Vec<std::path::PathBuf>, Vec<std::path::PathBuf>), LoaderError> {
        // Discover .pasta files
        let pasta_files = discovery::discover_files(base_dir, pasta_patterns)?;

        // Generate .lua patterns from .pasta patterns
        let lua_patterns: Vec<String> = pasta_patterns
            .iter()
            .filter_map(|p| {
                if let Some(stem) = p.strip_suffix(".pasta") {
                    Some(format!("{}.lua", stem))
                } else {
                    warn!(pattern = %p, "Cannot convert pattern to .lua, skipping");
                    None
                }
            })
            .collect();

        // Discover .lua files
        let lua_files = if lua_patterns.is_empty() {
            Vec::new()
        } else {
            discovery::discover_files(base_dir, &lua_patterns).unwrap_or_else(|e| {
                warn!(error = %e, "Failed to discover .lua files, skipping");
                Vec::new()
            })
        };

        // Check for invalid filenames (init.lua, init.pasta)
        for file in pasta_files.iter().chain(lua_files.iter()) {
            if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
                if file_name == "init.lua" || file_name == "init.pasta" {
                    return Err(LoaderError::invalid_file_name(file));
                }
            }
        }

        // Build HashSet of .pasta module names for conflict detection
        let pasta_module_names: std::collections::HashSet<String> = pasta_files
            .iter()
            .map(|f| {
                // Use inline module name computation (same as CacheManager)
                let relative = f
                    .strip_prefix(base_dir)
                    .unwrap_or(f)
                    .to_string_lossy()
                    .to_string();
                let without_prefix = relative
                    .strip_prefix("dic")
                    .unwrap_or(&relative)
                    .trim_start_matches(['/', '\\'])
                    .to_string();
                let stem = std::path::Path::new(&without_prefix)
                    .with_extension("")
                    .to_string_lossy()
                    .to_string();
                stem.replace(['/', '\\'], ".").replace('-', "_")
            })
            .collect();

        // Filter out conflicting .lua files
        let mut filtered_lua_files = Vec::new();
        for lua_file in &lua_files {
            let relative = lua_file
                .strip_prefix(base_dir)
                .unwrap_or(lua_file)
                .to_string_lossy()
                .to_string();
            let without_prefix = relative
                .strip_prefix("dic")
                .unwrap_or(&relative)
                .trim_start_matches(['/', '\\'])
                .to_string();
            let stem = std::path::Path::new(&without_prefix)
                .with_extension("")
                .to_string_lossy()
                .to_string();
            let module_key = stem.replace(['/', '\\'], ".").replace('-', "_");

            if pasta_module_names.contains(&module_key) {
                warn!(
                    lua_file = %lua_file.display(),
                    module_name = %format!("pasta.scene.{}", module_key),
                    "Module name conflict: .pasta file takes priority, .lua file ignored"
                );
            } else {
                filtered_lua_files.push(lua_file.clone());
            }
        }

        Ok((pasta_files, filtered_lua_files))
    }

    /// Incremental processing - transpile .pasta files and copy .lua files.
    ///
    /// Uses CacheManager to check timestamps and skip unchanged files.
    fn process_incremental(
        _base_dir: &Path,
        pasta_files: &[std::path::PathBuf],
        lua_files: &[std::path::PathBuf],
        cache_manager: &CacheManager,
    ) -> Result<(TranspileContext, Vec<String>, ProcessStats), LoaderError> {
        let transpiler = LuaTranspiler::default();
        let mut combined_context = TranspileContext::new();
        let total_count = pasta_files.len() + lua_files.len();
        let mut module_names = Vec::with_capacity(total_count);
        let mut failures = Vec::new();
        let mut stats = ProcessStats {
            transpiled: 0,
            skipped: 0,
            failed: 0,
            copied: 0,
        };

        // Process .pasta files (transpile)
        for file_path in pasta_files {
            // Check if transpilation is needed
            let needs_transpile = cache_manager.needs_transpile(file_path).unwrap_or(true);

            // Always collect module name for scene_dic.lua
            let module_name = cache_manager.source_to_module_name(file_path);
            module_names.push(module_name.clone());

            if !needs_transpile {
                stats.skipped += 1;
                debug!(file = %file_path.display(), "Skipped (cache up-to-date)");
                continue;
            }

            // Read and parse
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    failures.push(TranspileFailure {
                        source_path: file_path.clone(),
                        error: format!("Read error: {}", e),
                    });
                    stats.failed += 1;
                    continue;
                }
            };

            let filename = file_path.to_string_lossy().to_string();
            let pasta_file = match pasta_dsl::parse_str(&content, &filename) {
                Ok(pf) => pf,
                Err(e) => {
                    failures.push(TranspileFailure {
                        source_path: file_path.clone(),
                        error: format!("Parse error: {}", e),
                    });
                    stats.failed += 1;
                    continue;
                }
            };

            // Transpile
            let mut output = Vec::new();
            let file_context = match transpiler.transpile(&pasta_file, &mut output) {
                Ok(ctx) => ctx,
                Err(e) => {
                    failures.push(TranspileFailure {
                        source_path: file_path.clone(),
                        error: format!("Transpile error: {}", e),
                    });
                    stats.failed += 1;
                    continue;
                }
            };

            // Merge registries
            combined_context.merge_from(file_context);

            let lua_code = match String::from_utf8(output) {
                Ok(s) => s,
                Err(e) => {
                    failures.push(TranspileFailure {
                        source_path: file_path.clone(),
                        error: format!("UTF-8 error: {}", e),
                    });
                    stats.failed += 1;
                    continue;
                }
            };

            // Save to cache
            if let Err(e) = cache_manager.save_cache(file_path, &lua_code) {
                warn!(file = %file_path.display(), error = %e, "Failed to save cache");
                // Continue anyway - cache write failure is not fatal
            }

            stats.transpiled += 1;
            debug!(file = %file_path.display(), module = %module_name, "Transpiled");
        }

        // Process .lua files (copy passthrough)
        for file_path in lua_files {
            let needs_copy = cache_manager.needs_transpile(file_path).unwrap_or(true);

            // Always collect module name for scene_dic.lua
            let module_name = cache_manager.source_to_module_name(file_path);
            module_names.push(module_name.clone());

            if !needs_copy {
                stats.skipped += 1;
                debug!(file = %file_path.display(), "Skipped .lua (cache up-to-date)");
                continue;
            }

            // Read .lua file content directly (no parse/transpile)
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    warn!(
                        file = %file_path.display(),
                        error = %e,
                        "Failed to read .lua file, skipping"
                    );
                    stats.failed += 1;
                    continue;
                }
            };

            // Save to cache (direct copy)
            if let Err(e) = cache_manager.save_cache(file_path, &content) {
                warn!(
                    file = %file_path.display(),
                    error = %e,
                    "Failed to copy .lua file to cache, skipping"
                );
                stats.failed += 1;
                continue;
            }

            stats.copied += 1;
            debug!(file = %file_path.display(), module = %module_name, "Copied .lua");
        }

        // Report failures if any
        if !failures.is_empty() {
            warn!(failed = stats.failed, "Some files failed to process");
            for failure in &failures {
                warn!(
                    path = %failure.source_path.display(),
                    error = %failure.error,
                    "Process failure"
                );
            }
        }

        Ok((combined_context, module_names, stats))
    }

    /// Convert file path to module name (legacy - for backward compatibility).
    ///
    /// Example: `dic/baseware/system.pasta` → `dic_baseware_system`
    #[allow(dead_code)]
    fn path_to_module_name(path: &Path) -> String {
        let stem = path.with_extension("");
        let s = stem.to_string_lossy();
        s.replace(['/', '\\'], "_")
    }
}

/// Result of transpiling a single pasta file.
#[derive(Debug, Clone)]
pub struct TranspileResult {
    /// Module name derived from source path
    pub module_name: String,
    /// Generated Lua code
    pub lua_code: String,
    /// Original source file path
    pub source_path: std::path::PathBuf,
}
