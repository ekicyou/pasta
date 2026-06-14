//! Loader file discovery and incremental processing.
//!
//! Sibling submodule of [`super`] holding the discovery + incremental
//! transpile/copy responsibilities split out of `loader/mod.rs` (C3).

use super::discovery;
use super::{CacheManager, LoaderError, PastaLoader, TranspileFailure};
use crate::context::TranspileContext;
use crate::transpiler::LuaTranspiler;

use std::fs;
use std::path::Path;
use tracing::{debug, error, warn};

/// Process statistics for logging.
pub(super) struct ProcessStats {
    pub(super) transpiled: usize,
    pub(super) skipped: usize,
    pub(super) failed: usize,
    pub(super) copied: usize,
}

impl PastaLoader {
    /// Discover all files (.pasta and .lua) with conflict checking.
    ///
    /// Returns (pasta_files, lua_files) where lua_files has conflicts removed.
    /// Also checks for invalid filenames (init.lua, init.pasta).
    pub(super) fn discover_all_files(
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
    pub(super) fn process_incremental(
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
