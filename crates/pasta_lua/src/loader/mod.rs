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
pub use config::{LoaderConfig, LoggingConfig, PastaConfig, PersistenceConfig};
pub use context::LoaderContext;
pub use error::{LoaderError, TranspileFailure};

use crate::context::TranspileContext;
use crate::runtime::{PastaLuaRuntime, RuntimeConfig};
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Transpile statistics for logging.
struct TranspileStats {
    transpiled: usize,
    skipped: usize,
    failed: usize,
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
        debug!("Phase 3: Discovering pasta files");
        let files = discovery::discover_files(base_dir, &config.loader.pasta_patterns)?;
        if files.is_empty() {
            warn!(path = %base_dir.display(), "No .pasta files found");
        } else {
            info!(count = files.len(), "Found pasta files");
        }

        // Phase 4: Incremental transpile
        debug!("Phase 4: Incremental transpilation");
        let (context, module_names, stats) =
            Self::transpile_incremental(base_dir, &files, &cache_manager)?;

        // Log statistics in debug mode
        if config.loader.debug_mode {
            info!(
                transpiled = stats.transpiled,
                skipped = stats.skipped,
                failed = stats.failed,
                "Transpilation statistics"
            );
        }

        // Check for orphaned caches
        let orphans = cache_manager.find_orphaned_caches(&files);
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

    /// Incremental transpilation - only transpile changed files.
    ///
    /// Uses CacheManager to check timestamps and skip unchanged files.
    fn transpile_incremental(
        _base_dir: &Path,
        files: &[std::path::PathBuf],
        cache_manager: &CacheManager,
    ) -> Result<(TranspileContext, Vec<String>, TranspileStats), LoaderError> {
        let transpiler = LuaTranspiler::default();
        let mut combined_context = TranspileContext::new();
        let mut module_names = Vec::with_capacity(files.len());
        let mut failures = Vec::new();
        let mut stats = TranspileStats {
            transpiled: 0,
            skipped: 0,
            failed: 0,
        };

        for file_path in files {
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
            let pasta_file = match pasta_core::parse_str(&content, &filename) {
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

        // Report failures if any
        if !failures.is_empty() {
            warn!(failed = stats.failed, "Some files failed to transpile");
            for failure in &failures {
                warn!(
                    path = %failure.source_path.display(),
                    error = %failure.error,
                    "Transpile failure"
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
