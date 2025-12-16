//! Directory-based script loader for Pasta engine.
//!
//! This module provides functionality to load Pasta scripts from a directory structure
//! following the areka-P0-script-engine convention (dic/ + main.rn).

use crate::error::{ParseErrorInfo, PastaError, Result};
use std::path::{Path, PathBuf};

/// Directory loader for Pasta scripts.
///
/// Loads scripts from a directory following the areka-P0-script-engine convention:
/// - `dic/` subdirectory containing `.pasta` files
/// - `main.rn` file at the script root
pub struct DirectoryLoader;

/// Loaded files from a script directory.
#[derive(Debug, Clone)]
pub struct LoadedFiles {
    /// Script root directory (absolute path).
    pub script_root: PathBuf,
    /// Collected .pasta file paths.
    pub pasta_files: Vec<PathBuf>,
    /// main.rn file path.
    pub main_rune: PathBuf,
}

impl DirectoryLoader {
    /// Load scripts from a directory.
    ///
    /// # Arguments
    ///
    /// * `script_root` - Script root directory (must be absolute path)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Path is not absolute (`NotAbsolutePath`)
    /// - Directory does not exist (`DirectoryNotFound`)
    /// - Path is not a directory (`NotADirectory`)
    /// - Read permission is denied (`PermissionDenied`)
    /// - `dic/` directory not found (`DicDirectoryNotFound`)
    /// - `main.rn` not found (`MainRuneNotFound`)
    pub fn load(script_root: &Path) -> Result<LoadedFiles> {
        // Step 1: Validate directory
        Self::validate_directory(script_root)?;

        // Step 2: Check main.rn and dic/ directory
        Self::check_main_rn(script_root)?;
        let dic_path = script_root.join("dic");
        if !dic_path.exists() {
            return Err(PastaError::DicDirectoryNotFound {
                script_root: script_root.display().to_string(),
            });
        }

        // Step 3: Collect .pasta files
        let pasta_files = Self::collect_pasta_files(&dic_path)?;

        // Step 4: Log warning if no .pasta files found
        if pasta_files.is_empty() {
            tracing::warn!(
                script_root = %script_root.display(),
                "No .pasta files found in dic/ directory"
            );
        }

        let main_rune = script_root.join("main.rn");

        Ok(LoadedFiles {
            script_root: script_root.to_path_buf(),
            pasta_files,
            main_rune,
        })
    }

    /// Validate directory path.
    fn validate_directory(path: &Path) -> Result<()> {
        // Check absolute path
        if !path.is_absolute() {
            return Err(PastaError::NotAbsolutePath {
                path: path.display().to_string(),
            });
        }

        // Check existence
        if !path.exists() {
            return Err(PastaError::DirectoryNotFound {
                path: path.display().to_string(),
            });
        }

        // Check directory type
        if !path.is_dir() {
            return Err(PastaError::NotADirectory {
                path: path.display().to_string(),
            });
        }

        // Check read permission (by attempting to read directory)
        std::fs::read_dir(path).map_err(|_| PastaError::PermissionDenied {
            path: path.display().to_string(),
        })?;

        Ok(())
    }

    /// Check if main.rn exists.
    fn check_main_rn(script_root: &Path) -> Result<()> {
        let main_rune_path = script_root.join("main.rn");
        if !main_rune_path.exists() {
            return Err(PastaError::MainRuneNotFound {
                script_root: script_root.display().to_string(),
            });
        }
        Ok(())
    }

    /// Collect .pasta files from dic/ directory recursively.
    fn collect_pasta_files(dic_path: &Path) -> Result<Vec<PathBuf>> {
        let pattern = dic_path.join("**/*.pasta");
        let pattern_str = pattern.to_string_lossy();

        let mut pasta_files = Vec::new();

        for entry in glob::glob(&pattern_str).map_err(|e| {
            PastaError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Glob pattern error: {}", e),
            ))
        })? {
            let path = entry.map_err(|e| {
                PastaError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Glob error: {}", e),
                ))
            })?;

            // Skip files starting with '_'
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if name.starts_with('_') {
                    continue;
                }
                // Skip hidden files (starting with '.')
                if name.starts_with('.') {
                    continue;
                }
            }

            // Case-insensitive .pasta extension check
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "pasta" {
                    pasta_files.push(path);
                }
            }
        }

        Ok(pasta_files)
    }
}

/// Error log writer for parse errors.
pub(crate) struct ErrorLogWriter;

impl ErrorLogWriter {
    /// Log parse errors using tracing.
    ///
    /// # Arguments
    ///
    /// * `script_root` - Script root directory
    /// * `errors` - Parse errors to log
    pub fn log(script_root: &Path, errors: &[ParseErrorInfo]) {
        for error in errors {
            tracing::info!(
                "パースエラー: {}:{}:{} - {}",
                error.file,
                error.line,
                error.column,
                error.message
            );
        }

        tracing::info!(
            "合計 {} 件のパースエラー (script_root: {})",
            errors.len(),
            script_root.display()
        );
    }
}
