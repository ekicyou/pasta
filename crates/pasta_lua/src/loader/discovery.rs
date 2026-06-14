//! File discovery for Pasta Loader.
//!
//! This module provides file discovery functionality using glob patterns.

use glob::glob;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::LoaderError;

/// Check if a pattern contains directory traversal components.
///
/// Rejects patterns containing `..`, absolute paths, or Windows drive prefixes
/// to prevent file discovery outside the intended base directory.
fn contains_traversal(pattern: &str) -> bool {
    let path = Path::new(pattern);
    path.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

fn is_within_base_dir(base_dir: &Path, path: &Path) -> bool {
    path.strip_prefix(base_dir).is_ok()
}

fn has_symlink_component(base_dir: &Path, path: &Path) -> std::io::Result<bool> {
    let Ok(relative) = path.strip_prefix(base_dir) else {
        return Ok(false);
    };

    let mut current = base_dir.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Discover pasta files matching the given patterns.
///
/// Files in `profile/` directory are excluded from discovery.
/// Patterns containing directory traversal (`..`, absolute paths) are rejected.
/// Matches outside `base_dir` and paths containing a symlinked component
/// (including Windows junctions) are skipped.
///
/// # Arguments
/// * `base_dir` - Base directory to search from
/// * `patterns` - Glob patterns (e.g., ["dic/*/*.pasta"])
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of discovered files (may be empty)
/// * `Err(LoaderError)` - Directory not found or pattern error
pub(crate) fn discover_files(
    base_dir: &Path,
    patterns: &[String],
) -> Result<Vec<PathBuf>, LoaderError> {
    // Verify base directory exists
    if !base_dir.exists() {
        return Err(LoaderError::directory_not_found(base_dir));
    }

    if !base_dir.is_dir() {
        return Err(LoaderError::directory_not_found(base_dir));
    }

    let mut files = Vec::new();

    for pattern in patterns {
        // Reject patterns with directory traversal components
        if contains_traversal(pattern) {
            tracing::warn!(
                pattern = %pattern,
                "Rejecting pattern with directory traversal"
            );
            continue;
        }

        let full_pattern = base_dir.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        tracing::debug!(pattern = %pattern_str, "Searching for files");

        for entry in glob(&pattern_str)? {
            let path = entry?;

            if !is_within_base_dir(base_dir, &path) {
                tracing::warn!(
                    path = %path.display(),
                    base_dir = %base_dir.display(),
                    "Skipping match outside base directory"
                );
                continue;
            }

            match has_symlink_component(base_dir, &path) {
                Ok(true) => {
                    tracing::debug!(path = %path.display(), "Skipping symlinked path");
                    continue;
                }
                Err(error) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %error,
                        "Skipping path with unreadable metadata"
                    );
                    continue;
                }
                Ok(false) => {}
            }

            // Skip files in profile/ directory
            if is_in_profile_dir(base_dir, &path) {
                tracing::debug!(path = %path.display(), "Skipping profile file");
                continue;
            }

            files.push(path);
        }
    }

    if files.is_empty() {
        tracing::warn!(
            base_dir = %base_dir.display(),
            patterns = ?patterns,
            "No .pasta files found"
        );
    }

    Ok(files)
}

/// Check if a path is inside the profile/ directory.
fn is_in_profile_dir(base_dir: &Path, path: &Path) -> bool {
    let profile_dir = base_dir.join("profile");
    path.starts_with(&profile_dir)
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
