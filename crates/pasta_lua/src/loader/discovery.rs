//! File discovery for Pasta Loader.
//!
//! This module provides file discovery functionality using glob patterns.

use glob::glob;
use std::path::{Path, PathBuf};

use super::LoaderError;

/// Discover pasta files matching the given patterns.
///
/// Files in `profile/` directory are excluded from discovery.
///
/// # Arguments
/// * `base_dir` - Base directory to search from
/// * `patterns` - Glob patterns (e.g., ["dic/*/*.pasta"])
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of discovered files (may be empty)
/// * `Err(LoaderError)` - Directory not found or pattern error
pub fn discover_files(base_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>, LoaderError> {
    // Verify base directory exists
    if !base_dir.exists() {
        return Err(LoaderError::directory_not_found(base_dir));
    }

    if !base_dir.is_dir() {
        return Err(LoaderError::directory_not_found(base_dir));
    }

    let mut files = Vec::new();

    for pattern in patterns {
        let full_pattern = base_dir.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        tracing::debug!(pattern = %pattern_str, "Searching for files");

        for entry in glob(&pattern_str)? {
            let path = entry?;

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
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_structure(temp: &TempDir) -> PathBuf {
        let base = temp.path();

        // Create dic structure
        fs::create_dir_all(base.join("dic/greeting")).unwrap();
        fs::create_dir_all(base.join("dic/conversation")).unwrap();
        fs::write(base.join("dic/greeting/hello.pasta"), "# hello").unwrap();
        fs::write(base.join("dic/greeting/goodbye.pasta"), "# goodbye").unwrap();
        fs::write(base.join("dic/conversation/chat.pasta"), "# chat").unwrap();

        // Create file directly in dic (should be ignored by dic/*/*.pasta)
        fs::write(base.join("dic/root.pasta"), "# root").unwrap();

        // Create profile structure (should be excluded)
        fs::create_dir_all(base.join("profile/pasta/cache/lua")).unwrap();
        fs::write(
            base.join("profile/pasta/cache/lua/cached.pasta"),
            "# cached",
        )
        .unwrap();

        base.to_path_buf()
    }

    #[test]
    fn test_discover_default_pattern() {
        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);

        let patterns = vec!["dic/*/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();

        assert_eq!(files.len(), 3);
        let file_names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(file_names.contains(&"hello.pasta".to_string()));
        assert!(file_names.contains(&"goodbye.pasta".to_string()));
        assert!(file_names.contains(&"chat.pasta".to_string()));
    }

    #[test]
    fn test_discover_excludes_root_dic() {
        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);

        let patterns = vec!["dic/*/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();

        // Should not include dic/root.pasta
        let file_names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(!file_names.contains(&"root.pasta".to_string()));
    }

    #[test]
    fn test_discover_excludes_profile() {
        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);

        // Even with a pattern that would match profile, it should be excluded
        let patterns = vec!["**/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();

        let file_names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(!file_names.contains(&"cached.pasta".to_string()));
    }

    #[test]
    fn test_discover_nonexistent_directory() {
        let temp = TempDir::new().unwrap();
        let nonexistent = temp.path().join("nonexistent");

        let patterns = vec!["dic/*/*.pasta".to_string()];
        let result = discover_files(&nonexistent, &patterns);

        assert!(result.is_err());
        match result {
            Err(LoaderError::DirectoryNotFound(_)) => {}
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp = TempDir::new().unwrap();
        let base_dir = temp.path();

        // Create empty dic structure
        fs::create_dir_all(base_dir.join("dic/empty")).unwrap();

        let patterns = vec!["dic/*/*.pasta".to_string()];
        let files = discover_files(base_dir, &patterns).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_discover_multiple_patterns() {
        let temp = TempDir::new().unwrap();
        let base_dir = temp.path();

        // Create structures for multiple patterns
        fs::create_dir_all(base_dir.join("dic/sub")).unwrap();
        fs::create_dir_all(base_dir.join("extra")).unwrap();
        fs::write(base_dir.join("dic/sub/a.pasta"), "# a").unwrap();
        fs::write(base_dir.join("extra/b.pasta"), "# b").unwrap();

        let patterns = vec!["dic/*/*.pasta".to_string(), "extra/*.pasta".to_string()];
        let files = discover_files(base_dir, &patterns).unwrap();

        assert_eq!(files.len(), 2);
    }
}
