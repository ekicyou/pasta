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

pub use cache::{CURRENT_VERSION, CacheManager};
pub use config::{
    DebugFileConfig, GhostConfig, LoaderConfig, LoggingConfig, LuaConfig, PastaConfig,
    PersistenceConfig, TalkConfig, default_debug_port, default_hour_margin, default_libs,
    default_log_file_path, default_lua_search_paths, default_spot_newlines,
    default_talk_interval_max, default_talk_interval_min,
};
pub use context::LoaderContext;
pub use error::{LoaderError, TranspileFailure};

use crate::context::TranspileContext;
use crate::debug::source_map::{MapBuilderSink, SourceMap};
use crate::runtime::{PastaLuaRuntime, RuntimeConfig};
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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
            if let Some(file_name) = file.file_name().and_then(|n| n.to_str())
                && (file_name == "init.lua" || file_name == "init.pasta")
            {
                return Err(LoaderError::invalid_file_name(file));
            }
        }

        // Build HashSet of .pasta module names for conflict detection
        let pasta_module_names: std::collections::HashSet<String> = pasta_files
            .iter()
            .map(|f| module_key(base_dir, f))
            .collect();

        // Filter out conflicting .lua files
        let mut filtered_lua_files = Vec::new();
        for lua_file in &lua_files {
            let module_key = module_key(base_dir, lua_file);

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
                    error!(
                        file = %file_path.display(),
                        error = %e,
                        "Failed to read .lua file"
                    );
                    failures.push(TranspileFailure {
                        source_path: file_path.clone(),
                        error: format!("Read error: {}", e),
                    });
                    stats.failed += 1;
                    continue;
                }
            };

            // Save to cache (direct copy)
            if let Err(e) = cache_manager.save_cache(file_path, &content) {
                error!(
                    file = %file_path.display(),
                    error = %e,
                    "Failed to copy .lua file to cache"
                );
                failures.push(TranspileFailure {
                    source_path: file_path.clone(),
                    error: format!("Cache write error: {}", e),
                });
                stats.failed += 1;
                continue;
            }

            stats.copied += 1;
            debug!(file = %file_path.display(), module = %module_name, "Copied .lua");
        }

        // Abort on any failures
        if !failures.is_empty() {
            for failure in &failures {
                error!(
                    path = %failure.source_path.display(),
                    error = %failure.error,
                    "Process failure"
                );
            }
            let succeeded = stats.transpiled + stats.skipped + stats.copied;
            return Err(LoaderError::partial_transpile(
                succeeded,
                stats.failed,
                failures,
            ));
        }

        Ok((combined_context, module_names, stats))
    }

    /// Build and aggregate the multi-chunk `.pasta`↔`.lua` [`SourceMap`] for the
    /// discovered `.pasta` files (task 4.3 — `SourceMapBuilder`).
    ///
    /// This is the loader-side source-map build orchestration (design "Flow 1",
    /// File Structure Plan `loader/mod.rs` line 166). For EACH `.pasta` file it:
    ///
    /// 1. Re-parses the source and creates a [`MapBuilderSink::new(pasta_file,
    ///    chunk_name)`], where `chunk_name` is derived from the SAME loader cache
    ///    path construction the runtime require/hook path uses
    ///    ([`CacheManager::source_to_cache_path`] — task 1.1's confirmed naming
    ///    strategy; `set_name` is NOT needed because the hook source and this key
    ///    match after [`crate::debug::source_map::canonicalize_chunk_name`]).
    /// 2. Transpiles with the sink attached via
    ///    [`LuaTranspiler::transpile_with_source_map`], which records pre-normalize
    ///    `(.lua line -> .pasta span)` correspondences (Requirement 1.1) and returns
    ///    the normalize [`crate::normalize::LineShift`].
    /// 3. Calls `sink.finish(&shift)` to rebase the records onto the final `.lua`
    ///    line numbers (Requirement 2.1), yielding a per-chunk
    ///    [`crate::debug::source_map::ChunkSourceMap`].
    /// 4. Inserts the chunk into the aggregate [`SourceMap`] keyed by chunk name
    ///    ([`SourceMap::insert_chunk`] canonicalizes the key internally — task 3.4).
    ///
    /// The result is held as an `Arc<SourceMap>` (immutable shared reference, design
    /// "Architecture" `Arc<SourceMap>` 不変共有 — Requirement 3.1, in-memory, no
    /// mandatory intermediate file). Task 4.4 threads this `Arc` into
    /// [`crate::debug::enable`].
    ///
    /// # Debug gating (Requirements 3.1, 7.1)
    ///
    /// This is invoked by the loader ONLY when debugging is enabled. On the disabled
    /// (default) path the loader does NOT call this, so no sink is attached during
    /// transpilation, no [`SourceMap`] is allocated, and the generated `.lua` bytes
    /// stay byte-invariant (the production transpile in
    /// [`process_incremental`](Self::process_incremental) uses the sink-less
    /// [`LuaTranspiler::transpile`]).
    ///
    /// Files that fail to read/parse/transpile are skipped with a warning (the map
    /// is best-effort and non-fatal; the primary transpile path in
    /// `process_incremental` is the one that surfaces hard failures).
    ///
    /// # Optional disk sidecar (task 6.1 — Requirement 3.2)
    ///
    /// When `sidecar` is `true` (the resolved `source_map_sidecar` flag — task 4.1,
    /// `env > file > 既定 false`), each per-chunk [`ChunkSourceMap`] is ALSO written
    /// to a `<generated.lua>.map` sidecar next to its generated `.lua`
    /// ([`crate::debug::source_map::write_sidecar`]). The sidecar write is
    /// NON-FATAL (design Error 611, 616 / Requirement 3.1): an I/O failure is logged
    /// via `tracing::warn!` and the in-memory `SourceMap` build continues unaffected.
    /// When `sidecar` is `false` (default) no `.map` file is written and the
    /// in-memory map is the sole path.
    pub fn build_source_map(
        pasta_files: &[std::path::PathBuf],
        cache_manager: &CacheManager,
        sidecar: bool,
    ) -> Arc<SourceMap> {
        Arc::new(build_source_map_inner(pasta_files, cache_manager, sidecar))
    }
}

/// Compute the bare scene module key for a source file under `base_dir`.
///
/// Same derivation as `CacheManager::source_to_module_name`, without the
/// `pasta.scene.` prefix: strip `base_dir` and the leading `dic` component,
/// drop the extension, then map path separators to `.` and `-` to `_`.
fn module_key(base_dir: &Path, file: &Path) -> String {
    let relative = file
        .strip_prefix(base_dir)
        .unwrap_or(file)
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
}

/// Core multi-chunk source-map build.
///
/// See [`PastaLoader::build_source_map`] for the contract. Kept as a private free
/// function so the build logic stays separate from the `Arc` wrapping; tests
/// exercise it through [`PastaLoader::build_source_map`].
fn build_source_map_inner(
    pasta_files: &[std::path::PathBuf],
    cache_manager: &CacheManager,
    sidecar: bool,
) -> SourceMap {
    let transpiler = LuaTranspiler::default();
    let mut source_map = SourceMap::new();
    let mut chunk_count = 0usize;

    for file_path in pasta_files {
        // Read the .pasta source.
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (read error)");
                continue;
            }
        };

        let filename = file_path.to_string_lossy().to_string();
        let pasta_file = match pasta_dsl::parse_str(&content, &filename) {
            Ok(pf) => pf,
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (parse error)");
                continue;
            }
        };

        // Chunk name = SAME loader cache path construction the runtime require/hook
        // path reports (task 1.1). `insert_chunk` canonicalizes this internally so
        // it matches the `@<absolute .lua path>` hook source after normalization.
        let chunk_name = cache_manager
            .source_to_cache_path(file_path)
            .to_string_lossy()
            .to_string();

        // Attach the recording sink and transpile, capturing the normalize shift.
        let mut sink = MapBuilderSink::new(filename.clone(), chunk_name.clone());
        let mut output = Vec::new();
        let shift = match transpiler.transpile_with_source_map(
            &pasta_file,
            &mut output,
            Some(&mut sink),
        ) {
            Ok((_ctx, shift)) => shift,
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "source-map: skip (transpile error)");
                continue;
            }
        };

        // Rebase pre-normalize records onto final `.lua` lines (2.1) and aggregate
        // the per-chunk map by chunk name (3.4).
        let chunk_map = sink.finish(&shift);

        // Optional disk sidecar (task 6.1 / Requirement 3.2): when enabled, write a
        // `<generated.lua>.map` next to the generated `.lua` (the same cache path the
        // chunk name was derived from). NON-FATAL (design Error 611, 616 / 3.1): an
        // I/O failure is logged and the in-memory map build continues unchanged.
        if sidecar {
            let lua_path = cache_manager.source_to_cache_path(file_path);
            if let Err(e) =
                crate::debug::source_map::write_sidecar(&lua_path, &filename, &chunk_map)
            {
                warn!(
                    file = %lua_path.display(),
                    error = %e,
                    "source-map: sidecar write failed; continuing with in-memory map (non-fatal, 3.2/3.1)"
                );
            } else {
                debug!(file = %lua_path.display(), "source-map: wrote sidecar (.lua.map)");
            }
        }

        source_map.insert_chunk(chunk_name, filename, chunk_map);
        chunk_count += 1;
    }

    debug!(
        chunks = chunk_count,
        files = pasta_files.len(),
        "source-map: built aggregate map"
    );
    source_map
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
