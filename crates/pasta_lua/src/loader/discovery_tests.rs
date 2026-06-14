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
fn test_contains_traversal_absolute_paths() {
    // RootDir component (absolute path) must be rejected
    assert!(contains_traversal("/etc/*.pasta"));
    // Windows drive prefix must be rejected (Prefix component on Windows)
    #[cfg(windows)]
    assert!(contains_traversal(r"C:\secret\*.pasta"));
}

#[test]
fn test_discover_base_dir_is_file() {
    // base_dir exists but is a regular file -> DirectoryNotFound
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("not_a_dir");
    fs::write(&file_path, "plain file").unwrap();

    let patterns = vec!["dic/*/*.pasta".to_string()];
    let result = discover_files(&file_path, &patterns);

    match result {
        Err(LoaderError::DirectoryNotFound(path)) => {
            assert_eq!(path, file_path);
        }
        other => panic!("Expected DirectoryNotFound, got: {:?}", other),
    }
}

#[test]
fn test_discover_invalid_glob_pattern() {
    // Unclosed character class is an invalid glob pattern -> GlobPattern error
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);

    let patterns = vec!["dic/[/*.pasta".to_string()];
    let result = discover_files(&base_dir, &patterns);

    assert!(matches!(result, Err(LoaderError::GlobPattern(_))));
}

#[test]
fn test_discover_rejects_absolute_pattern() {
    // Absolute patterns are silently skipped (traversal guard), result is empty
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);

    let patterns = vec!["/etc/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_discover_profile_prefix_dir_not_excluded() {
    // Only the exact "profile" directory is excluded. A sibling directory
    // whose name merely starts with "profile" (e.g. "profile2") must NOT
    // be excluded — exclusion works on path components, not string prefix.
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);
    fs::create_dir_all(base_dir.join("profile2/inner")).unwrap();
    fs::write(base_dir.join("profile2/inner/kept.pasta"), "# kept").unwrap();

    let patterns = vec!["**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(file_names.contains(&"kept.pasta".to_string()));
    // The real profile/ file stays excluded
    assert!(!file_names.contains(&"cached.pasta".to_string()));
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

#[cfg(windows)]
#[test]
fn test_discover_skips_junction_directory() {
    // Windows junctions (mount-point reparse points) can redirect discovery
    // outside base_dir exactly like symlinks. Rust std reports them via
    // `FileType::is_symlink()` (name-surrogate reparse tag), so
    // `has_symlink_component` must skip them — this test pins that behavior.
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);
    let external_dir = temp.path().join("external");
    fs::create_dir_all(&external_dir).unwrap();
    fs::write(external_dir.join("secret.pasta"), "# secret").unwrap();

    // Create junction without elevation: cmd /C mklink /J <link> <target>
    let junction = base_dir.join("dic").join("linked");
    let status = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(&junction)
        .arg(&external_dir)
        .status()
        .expect("failed to spawn cmd for mklink");
    assert!(status.success(), "mklink /J failed");

    let patterns = vec!["dic/*/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert_eq!(files.len(), 3);
    assert!(!file_names.contains(&"secret.pasta".to_string()));
}

// --- Recursive default pattern (`dic/**/*.pasta`) tests ---
//
// Task 1.2 changed the default discovery glob to `dic/**/*.pasta` (recursive).
// These tests pin the recursive-glob behavior for flat / one-level /
// multi-level placements, fix the existing nested layout as a
// non-destructive regression, and confirm exclusion rules (profile/,
// traversal, symlink/junction) remain invariant under the recursive pattern.

/// Flat: a file directly under `dic/` IS discovered with `dic/**/*.pasta`.
/// (The old one-level pattern `dic/*/*.pasta` excludes it — see
/// `test_discover_excludes_root_dic` — but the recursive default includes it.)
#[test]
fn test_discover_recursive_includes_flat_root() {
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);

    let patterns = vec!["dic/**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    // dic/root.pasta sits directly under dic/ and must now be found.
    assert!(
        file_names.contains(&"root.pasta".to_string()),
        "recursive pattern must include flat dic/root.pasta, got {file_names:?}"
    );
}

/// One-level: files under `dic/<sub>/*.pasta` are discovered with the
/// recursive pattern (non-destructive: the existing nested layout still
/// loads exactly as it did under `dic/*/*.pasta`).
#[test]
fn test_discover_recursive_includes_one_level_nested() {
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);

    let patterns = vec!["dic/**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    // The pre-existing one-level nested files remain discovered (regression).
    assert!(file_names.contains(&"hello.pasta".to_string()));
    assert!(file_names.contains(&"goodbye.pasta".to_string()));
    assert!(file_names.contains(&"chat.pasta".to_string()));
}

/// Multi-level: a file deeper than one level (`dic/a/b/c.pasta`) IS
/// discovered with `dic/**/*.pasta`. The old one-level default
/// (`dic/*/*.pasta`) would NOT match this — pin the new capability.
#[test]
fn test_discover_recursive_includes_multi_level_nested() {
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);
    fs::create_dir_all(base_dir.join("dic/a/b")).unwrap();
    fs::write(base_dir.join("dic/a/b/c.pasta"), "# deep").unwrap();

    let patterns = vec!["dic/**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    // Deep file must be found by the recursive pattern.
    assert!(
        file_names.contains(&"c.pasta".to_string()),
        "recursive pattern must include multi-level dic/a/b/c.pasta, got {file_names:?}"
    );

    // Sanity: the old one-level pattern would NOT reach the deep file,
    // confirming this is genuinely new recursive capability.
    let one_level = discover_files(&base_dir, &["dic/*/*.pasta".to_string()]).unwrap();
    let one_level_names: Vec<_> = one_level
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(!one_level_names.contains(&"c.pasta".to_string()));
}

/// Full default-pattern picture: with `dic/**/*.pasta` the discovered set is
/// exactly the flat file plus the one-level nested files (additive vs. the
/// old `dic/*/*.pasta` result, which had 3 files and excluded root).
#[test]
fn test_discover_recursive_default_full_set() {
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);

    let patterns = vec!["dic/**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let mut file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    file_names.sort();

    assert_eq!(
        file_names,
        vec![
            "chat.pasta".to_string(),
            "goodbye.pasta".to_string(),
            "hello.pasta".to_string(),
            "root.pasta".to_string(),
        ],
        "recursive default must additively include flat + nested files"
    );
}

/// Exclusion invariant: `profile/` files stay excluded under the recursive
/// `dic/**/*.pasta` pattern as well. We place a `.pasta` inside a `profile/`
/// dir nested under `dic/` to ensure the profile exclusion still wins even
/// when reached via the recursive glob.
#[test]
fn test_discover_recursive_excludes_profile() {
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);
    // profile/ directly under base — reachable via the broad recursive glob.
    fs::create_dir_all(base_dir.join("profile/nested")).unwrap();
    fs::write(base_dir.join("profile/nested/secret.pasta"), "# secret").unwrap();

    // Use a base-rooted recursive pattern so a profile match is even possible,
    // then confirm exclusion still holds.
    let patterns = vec!["**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(!file_names.contains(&"secret.pasta".to_string()));
    assert!(!file_names.contains(&"cached.pasta".to_string()));
    // dic content is still present.
    assert!(file_names.contains(&"hello.pasta".to_string()));
    assert!(file_names.contains(&"root.pasta".to_string()));
}

/// Exclusion invariant: directory-traversal patterns are still rejected when
/// combined with the recursive default — the valid recursive pattern keeps
/// working, the `..` pattern contributes nothing.
#[test]
fn test_discover_recursive_rejects_traversal_preserves_valid() {
    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);

    let patterns = vec![
        "../secret/**/*.pasta".to_string(),
        "dic/**/*.pasta".to_string(),
    ];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    // Recursive default yields flat + nested (4 files); traversal added none.
    assert_eq!(files.len(), 4);
    assert!(file_names.contains(&"root.pasta".to_string()));
    assert!(file_names.contains(&"hello.pasta".to_string()));
}

/// Exclusion invariant (Unix): a symlinked file matched via the recursive
/// `dic/**/*.pasta` pattern is still skipped.
#[cfg(unix)]
#[test]
fn test_discover_recursive_skips_symlinked_file() {
    use std::os::unix::fs as unix_fs;

    let temp = TempDir::new().unwrap();
    let base_dir = create_test_structure(&temp);
    let external = temp.path().join("external.pasta");
    fs::write(&external, "# external").unwrap();
    unix_fs::symlink(&external, base_dir.join("dic/greeting/link.pasta")).unwrap();

    let patterns = vec!["dic/**/*.pasta".to_string()];
    let files = discover_files(&base_dir, &patterns).unwrap();
    let file_names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(!file_names.contains(&"link.pasta".to_string()));
    // Real files (including the flat root) remain discovered.
    assert!(file_names.contains(&"root.pasta".to_string()));
    assert!(file_names.contains(&"hello.pasta".to_string()));
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
