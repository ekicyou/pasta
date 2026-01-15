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

mod config;
mod context;
mod discovery;
mod error;

pub use config::{LoaderConfig, LoggingConfig, PastaConfig};
pub use context::LoaderContext;
pub use error::LoaderError;

use crate::context::TranspileContext;
use crate::runtime::{PastaLuaRuntime, RuntimeConfig};
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Pasta Loader - Unified startup sequence API.
///
/// Orchestrates the complete startup sequence:
/// 1. Load configuration from pasta.toml
/// 2. Prepare profile directories
/// 3. Discover .pasta files in dic/*/*.pasta
/// 4. Transpile all files to Lua
/// 5. Initialize PastaLuaRuntime
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

        // Phase 2: Prepare directories
        debug!("Phase 2: Preparing directories");
        Self::prepare_directories(base_dir, &config.loader)?;

        // Phase 3: Discover files
        debug!("Phase 3: Discovering pasta files");
        let files = discovery::discover_files(base_dir, &config.loader.pasta_patterns)?;
        if files.is_empty() {
            warn!(path = %base_dir.display(), "No .pasta files found");
        } else {
            info!(count = files.len(), "Found pasta files");
        }

        // Phase 4: Transpile
        debug!("Phase 4: Transpiling pasta files");
        let (context, transpiled) = Self::transpile_all(base_dir, &files)?;

        // Save cache files if debug mode
        if config.loader.debug_mode {
            Self::save_cache_files(base_dir, &config.loader.transpiled_output_dir, &transpiled)?;
        }

        // Phase 5: Create logger
        debug!("Phase 5: Creating instance logger");
        let logger = Self::create_logger(base_dir, &config)?;

        // Phase 6: Initialize runtime
        debug!("Phase 6: Initializing runtime");
        let loader_context = LoaderContext::from_config(base_dir, &config);
        let runtime = PastaLuaRuntime::from_loader(
            context,
            loader_context,
            runtime_config,
            &transpiled,
            logger,
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

        // Clear and recreate cache/lua directory
        let cache_lua = base_dir.join(&config.transpiled_output_dir);
        if cache_lua.exists() {
            fs::remove_dir_all(&cache_lua).map_err(|e| LoaderError::io(&cache_lua, e))?;
            fs::create_dir_all(&cache_lua).map_err(|e| LoaderError::io(&cache_lua, e))?;
            debug!(path = %cache_lua.display(), "Cleared cache directory");
        }

        Ok(())
    }

    /// Transpile all discovered pasta files.
    fn transpile_all(
        base_dir: &Path,
        files: &[std::path::PathBuf],
    ) -> Result<(TranspileContext, Vec<TranspileResult>), LoaderError> {
        let transpiler = LuaTranspiler::default();
        let mut combined_context = TranspileContext::new();
        let mut results = Vec::with_capacity(files.len());

        for file_path in files {
            let content =
                fs::read_to_string(file_path).map_err(|e| LoaderError::io(file_path, e))?;

            // Generate module name from file path
            let relative = file_path.strip_prefix(base_dir).unwrap_or(file_path);
            let module_name = Self::path_to_module_name(relative);
            let filename = file_path.to_string_lossy().to_string();

            // Parse the pasta file
            let pasta_file = pasta_core::parse_str(&content, &filename)
                .map_err(|e| LoaderError::parse_with_source(file_path, e.to_string(), e))?;

            // Transpile to Lua
            let mut output = Vec::new();
            let file_context = transpiler
                .transpile(&pasta_file, &mut output)
                .map_err(LoaderError::from)?;

            // Merge registries into combined context
            combined_context.merge_from(file_context);

            let lua_code = String::from_utf8(output).map_err(|e| {
                LoaderError::parse(
                    file_path,
                    format!("Invalid UTF-8 in transpiled output: {}", e),
                )
            })?;

            results.push(TranspileResult {
                module_name,
                lua_code,
                source_path: file_path.clone(),
            });

            debug!(file = %file_path.display(), "Transpiled");
        }

        Ok((combined_context, results))
    }

    /// Save transpiled results to cache directory.
    fn save_cache_files(
        base_dir: &Path,
        output_dir: &str,
        results: &[TranspileResult],
    ) -> Result<(), LoaderError> {
        let cache_dir = base_dir.join(output_dir);

        for result in results {
            let cache_file = cache_dir.join(format!("{}.lua", result.module_name));
            fs::write(&cache_file, &result.lua_code)
                .map_err(|e| LoaderError::io(&cache_file, e))?;
            debug!(file = %cache_file.display(), "Saved cache");
        }

        Ok(())
    }

    /// Convert file path to module name.
    ///
    /// Example: `dic/baseware/system.pasta` → `dic_baseware_system`
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
