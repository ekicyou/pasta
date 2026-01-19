//! Common test utilities for pasta_shiori integration tests.

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Copy fixture directory to a temporary location for isolated testing.
///
/// This function copies both the fixture files and the support files
/// (scripts and scriptlibs) needed for SHIORI operation.
///
/// # Arguments
/// * `fixture_name` - Name of fixture under tests/fixtures/
///
/// # Returns
/// TempDir containing copied fixture with all necessary support files
pub fn copy_fixture_to_temp(fixture_name: &str) -> TempDir {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Copy fixture files
    let fixture_src = manifest_dir.join("tests/fixtures").join(fixture_name);
    let temp = TempDir::new().expect("Failed to create temp directory");

    if fixture_src.exists() {
        copy_dir_recursive(&fixture_src, temp.path()).expect("Failed to copy fixture");
    }

    // Copy support files (scripts, scriptlibs)
    let support_root = manifest_dir.join("tests/support");
    copy_support_dirs(&support_root, temp.path());

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
