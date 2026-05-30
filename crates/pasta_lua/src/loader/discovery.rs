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
///
/// # Arguments
/// * `base_dir` - Base directory to search from
/// * `patterns` - Glob patterns (e.g., ["dic/*/*.pasta"])
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of discovered files (may be empty)
/// * `Err(LoaderError)` - Directory not found or pattern error
pub(crate) fn discover_files(base_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>, LoaderError> {
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

    #[test]
    fn test_discover_rejects_parent_dir_traversal() {
        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);

        // Pattern with ".." should be silently skipped
        let patterns = vec!["../../../etc/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_discover_rejects_traversal_preserves_valid() {
        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);

        // Mix of valid and traversal patterns — valid should still work
        let patterns = vec!["../secret/*.pasta".to_string(), "dic/*/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_contains_traversal() {
        assert!(contains_traversal("../foo/*.pasta"));
        assert!(contains_traversal("foo/../../bar/*.pasta"));
        assert!(!contains_traversal("dic/*/*.pasta"));
        assert!(!contains_traversal("**/*.pasta"));
        assert!(!contains_traversal("extra/*.pasta"));
    }

    #[test]
    fn test_is_within_base_dir() {
        let temp = TempDir::new().unwrap();
        let base_dir = temp.path().join("base");
        let child = base_dir.join("dic/test.pasta");
        let outside = temp.path().join("outside/test.pasta");

        assert!(is_within_base_dir(&base_dir, &child));
        assert!(!is_within_base_dir(&base_dir, &outside));
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_skips_symlinked_file() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);
        let external = temp.path().join("external.pasta");
        fs::write(&external, "# external").unwrap();
        unix_fs::symlink(&external, base_dir.join("dic/greeting/link.pasta")).unwrap();

        let patterns = vec!["dic/*/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();
        let file_names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(files.len(), 3);
        assert!(!file_names.contains(&"link.pasta".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_skips_symlinked_directory() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let base_dir = create_test_structure(&temp);
        let external_dir = temp.path().join("external");
        fs::create_dir_all(&external_dir).unwrap();
        fs::write(external_dir.join("secret.pasta"), "# secret").unwrap();
        unix_fs::symlink(&external_dir, base_dir.join("dic/linked")).unwrap();

        let patterns = vec!["dic/*/*.pasta".to_string()];
        let files = discover_files(&base_dir, &patterns).unwrap();
        let file_names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(files.len(), 3);
        assert!(!file_names.contains(&"secret.pasta".to_string()));
    }
}
