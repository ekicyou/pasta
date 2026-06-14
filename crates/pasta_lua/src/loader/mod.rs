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
mod extract;
mod process;
mod source_map_build;

pub use cache::{CURRENT_VERSION, CacheManager};
pub use config::{
    DebugFileConfig, GhostConfig, LoaderConfig, LoggingConfig, LuaConfig, PastaConfig,
    PersistenceConfig, TalkConfig, default_debug_port, default_hour_margin, default_libs,
    default_log_file_path, default_lua_search_paths, default_spot_newlines,
    default_talk_interval_max, default_talk_interval_min,
};
pub use context::LoaderContext;
pub use error::{LoaderError, TranspileFailure};

use crate::runtime::{PastaLuaRuntime, RuntimeConfig};

use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Pasta Loader - Unified startup sequence API.
///
/// Orchestrates the complete startup sequence:
/// 1. Load configuration from pasta.toml
///    1.5. Create logger with config, register, and update tracing filter
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

        // Stage 1.5: Create logger with config and update tracing filter
        debug!("Stage 1.5: Applying logging configuration");
        let logger = Self::create_and_register_logger(base_dir, &config)?;

        // Phase 2: Prepare directories and cache (with version check)
        debug!("Phase 2: Preparing directories and cache");
        Self::prepare_directories(base_dir, &config.loader)?;
        let cache_manager =
            CacheManager::new(base_dir.to_path_buf(), &config.loader.transpiled_output_dir);
        cache_manager.prepare_cache_dir()?;

        // Phase 2.5: Self-deploy framework scripts (non-fatal)
        // base_dir is finalized by Phase 2; sync the embedded canonical pasta_scripts
        // to disk BEFORE Phase 3 / package.path construction (Req 6.1). Failure must
        // NOT abort startup (Req 3.2): log ERROR and continue with existing scripts.
        debug!("Phase 2.5: Self-deploying framework scripts");
        match extract::sync_pasta_scripts(base_dir) {
            Ok(_outcome) => {
                // Outcome already logged inside (DEBUG on skip / INFO on deploy).
            }
            Err(e) => {
                error!(
                    path = %base_dir.join("profile/pasta/pasta_scripts").display(),
                    error = %e,
                    "Self-deploy failed; continuing startup with existing scripts (version drift unresolved)"
                );
            }
        }

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
            Self::process_incremental(&pasta_files, &lua_files, &cache_manager)?;

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

        // Phase 5.5: Build the aggregated `.pasta`↔`.lua` source map (task 4.4).
        //
        // Gate on the SAME resolved debug decision the runtime uses
        // (`DebugConfig::from_env` over the pasta.toml `[debug]` section + the
        // `PASTA_DEBUG*` environment), so the build happens iff debugging is
        // enabled. On the disabled (default) path we do NOT call
        // `build_source_map`: no sink is attached, no `SourceMap` is allocated, and
        // the generated `.lua` from Phase 4 stays byte-invariant (requirements 3.1
        // / 7.1). The map is built AFTER transpile (Phase 4) so the ordering is
        // transpile → build map → enable-with-map (requirement 3.1).
        let debug_file = config.debug();
        let resolved_debug = crate::debug::DebugConfig::from_env(debug_file.as_ref());
        let debug_enabled = resolved_debug.enabled;
        let source_map = if debug_enabled {
            // Pass the resolved `source_map_sidecar` flag (task 4.1; env > file >
            // 既定 false) so the build path also writes the optional on-disk
            // `<lua>.map` sidecar next to each chunk (3.2). Disabled (default) →
            // no sidecar is written and the in-memory map is the sole path (3.1).
            let map = Self::build_source_map(
                &pasta_files,
                &cache_manager,
                resolved_debug.source_map_sidecar,
            );
            debug!("Phase 5.5: Built source map for debug session");
            Some(map)
        } else {
            None
        };

        // Phase 6: Initialize runtime and load scene_dic
        debug!("Phase 6: Initializing runtime");
        let loader_context = LoaderContext::from_config(base_dir, &config);
        let runtime = PastaLuaRuntime::from_loader_with_scene_dic(
            context,
            loader_context,
            runtime_config,
            Some(config),
            logger,
            &scene_dic_path,
            source_map,
        )?;

        info!(path = %base_dir.display(), "Startup sequence completed");
        Ok(runtime)
    }

    /// Create an instance-specific logger, register it with the global registry,
    /// and update the tracing filter with config from pasta.toml.
    ///
    /// Called at Stage 1.5 (after Phase 1 config load).
    fn create_and_register_logger(
        base_dir: &Path,
        config: &PastaConfig,
    ) -> Result<Option<std::sync::Arc<crate::logging::PastaLogger>>, LoaderError> {
        let logging_config = config.logging();

        match crate::logging::PastaLogger::new(base_dir, logging_config.as_ref()) {
            Ok(logger) => {
                let logger = std::sync::Arc::new(logger);
                info!(path = %logger.log_path().display(), "Created instance logger");

                // Register with global registry (overwrites Stage 1 default writer)
                crate::logging::GlobalLoggerRegistry::instance()
                    .register(base_dir.to_path_buf(), logger.clone());

                // Update tracing filter with config
                if let Some(ref cfg) = logging_config {
                    crate::logging::update_tracing_filter(cfg);
                }

                Ok(Some(logger))
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
