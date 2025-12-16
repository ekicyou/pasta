//! Common test utilities for Pasta integration tests.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

/// Get a temporary directory for test script storage.
/// This directory persists for the duration of the test suite.
#[allow(dead_code)]
pub fn get_test_script_dir() -> PathBuf {
    static TEMP_DIR: OnceLock<Mutex<TempDir>> = OnceLock::new();

    let temp_dir = TEMP_DIR
        .get_or_init(|| Mutex::new(TempDir::new().expect("Failed to create temp dir for scripts")));

    temp_dir.lock().unwrap().path().to_path_buf()
}

/// Get a temporary directory for test persistence storage.
/// This directory persists for the duration of the test suite.
///
/// NOTE: For concurrent tests, use `create_unique_persistence_dir()` instead
/// to avoid sharing the same directory across threads.
pub fn get_test_persistence_dir() -> PathBuf {
    static TEMP_DIR: OnceLock<Mutex<TempDir>> = OnceLock::new();

    let temp_dir = TEMP_DIR.get_or_init(|| {
        Mutex::new(TempDir::new().expect("Failed to create temp dir for persistence"))
    });

    temp_dir.lock().unwrap().path().to_path_buf()
}

/// Create a unique temporary directory for persistence storage.
///
/// This creates a new temporary directory for each call, which is necessary
/// for concurrent tests where each thread needs its own persistence directory.
/// The directory is leaked to prevent cleanup during test execution.
#[allow(dead_code)]
pub fn create_unique_persistence_dir() -> std::io::Result<PathBuf> {
    use tempfile::TempDir;

    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().to_path_buf();

    // Leak the temp directory to prevent cleanup
    std::mem::forget(temp_dir);

    Ok(path)
}

/// Write a script to a temporary file and return the directory path.
///
/// This function creates a unique temporary directory for each test with:
/// - dic/main.pasta file containing the script content (loaded by DirectoryLoader)
/// - main.rn file (empty, required by PastaEngine)
/// - dic/ directory (required by PastaEngine)
///
/// The directory is leaked to prevent cleanup during test execution.
pub fn create_test_script(script_content: &str) -> std::io::Result<PathBuf> {
    use std::fs;
    use tempfile::TempDir;

    // Create a unique temporary directory for this test
    let temp_dir = TempDir::new()?;
    let script_dir = temp_dir.path().to_path_buf();

    // Create dic directory (required by PastaEngine)
    let dic_dir = script_dir.join("dic");
    fs::create_dir(&dic_dir)?;

    // Write main.pasta in dic/ directory (DirectoryLoader loads from dic/)
    let script_file = dic_dir.join("main.pasta");
    fs::write(&script_file, script_content)?;

    // Write main.rn (required by PastaEngine) with actor definitions
    let main_rune = script_dir.join("main.rn");
    let main_rn_content = r#"// Actor definitions
pub mod actors {
    pub const さくら = #{
        name: "さくら",
        id: "sakura",
    };

    pub const うにゅう = #{
        name: "うにゅう",
        id: "unyuu",
    };

    pub const ななこ = #{
        name: "ななこ",
        id: "nanako",
    };
}

pub fn main() {}
"#;
    fs::write(&main_rune, main_rn_content)?;

    // Leak the temp directory to prevent cleanup
    std::mem::forget(temp_dir);

    Ok(script_dir)
}
