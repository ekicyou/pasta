//! Common test utilities for pasta_shiori integration tests.

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Copy fixture directory to a temporary location for isolated testing.
///
/// This function copies both the fixture files and the support files
/// (scripts and scriptlibs) needed for SHIORI operation.
///
/// **Important**: Support files are copied first, then fixture files.
/// This ensures fixture-specific overrides (e.g., main.lua) take precedence.
///
/// # Arguments
/// * `fixture_name` - Name of fixture under tests/fixtures/
///
/// # Returns
/// TempDir containing copied fixture with all necessary support files
pub fn copy_fixture_to_temp(fixture_name: &str) -> TempDir {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp = TempDir::new().expect("Failed to create temp directory");

    // Copy support files first (scripts, scriptlibs)
    // These provide the base runtime environment
    let support_root = manifest_dir.join("tests/support");
    copy_support_dirs(&support_root, temp.path());

    // Copy fixture files last (overrides support files if needed)
    // This allows fixture-specific main.lua or other customizations
    let fixture_src = manifest_dir.join("tests/fixtures").join(fixture_name);
    if fixture_src.exists() {
        copy_dir_recursive(&fixture_src, temp.path()).expect("Failed to copy fixture");
    }

    temp
}

/// Copy hello-pasta ghost to a temporary directory for isolated testing.
///
/// This function copies the complete hello-pasta ghost from pasta_sample_ghost
/// to a temporary directory, enabling integration tests with real ghost definitions.
/// Uses the existing `copy_dir_recursive()` function which automatically skips
/// profile/ directories.
///
/// # Returns
/// TempDir containing the copied hello-pasta ghost
///
/// # Panics
/// Panics if the ghost directory doesn't exist or copy fails.
pub fn copy_sample_ghost_to_temp() -> TempDir {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp = TempDir::new().expect("Failed to create temp directory");

    // Navigate to pasta_sample_ghost crate
    let sample_ghost_dir = manifest_dir
        .parent()
        .expect("Failed to get parent directory")
        .join("pasta_sample_ghost/ghosts/hello-pasta/ghost/master");

    // Copy ghost files using existing recursive copy (auto-skips profile/)
    copy_dir_recursive(&sample_ghost_dir, temp.path()).expect("Failed to copy hello-pasta ghost");

    temp
}

/// Recursively copy a directory and its contents.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            // Skip profile directories
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Copy support directories (scripts, scriptlibs) to destination.
fn copy_support_dirs(support_root: &Path, dst: &Path) {
    // Copy scripts directory
    let scripts_src = support_root.join("scripts");
    let scripts_dst = dst.join("scripts");
    if scripts_src.exists() {
        std::fs::create_dir_all(&scripts_dst).ok();
        copy_dir_recursive(&scripts_src, &scripts_dst).ok();
    }

    // Copy scriptlibs directory
    let scriptlibs_src = support_root.join("scriptlibs");
    let scriptlibs_dst = dst.join("scriptlibs");
    if scriptlibs_src.exists() {
        std::fs::create_dir_all(&scriptlibs_dst).ok();
        copy_dir_recursive(&scriptlibs_src, &scriptlibs_dst).ok();
    }
}
