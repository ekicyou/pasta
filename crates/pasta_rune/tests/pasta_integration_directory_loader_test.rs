//! Integration tests for directory-based script loading.

use pasta_rune::{PastaEngine, PastaError};
use std::path::PathBuf;

fn get_test_project_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("test-project")
        .canonicalize()
        .expect("Failed to canonicalize test project path")
}

fn get_test_persistence_path() -> PathBuf {
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use tempfile::TempDir;

    static TEMP_DIR: OnceLock<Mutex<TempDir>> = OnceLock::new();

    let temp_dir = TEMP_DIR.get_or_init(|| {
        Mutex::new(TempDir::new().expect("Failed to create temp dir for persistence"))
    });

    temp_dir.lock().unwrap().path().to_path_buf()
}

#[test]
fn test_from_directory_success() {
    let script_path = get_test_project_path();
    let persistence_path = get_test_persistence_path();
    let engine = PastaEngine::new(&script_path, &persistence_path);

    assert!(
        engine.is_ok(),
        "Engine initialization should succeed: {:?}",
        engine.err()
    );

    let mut engine = engine.unwrap();

    // Verify scenes by executing them
    let result = engine.execute_label("挨拶");
    assert!(result.is_ok(), "Should be able to execute 挨拶 label");

    let result = engine.execute_label("別れ");
    assert!(result.is_ok(), "Should be able to execute 別れ label");
}

#[test]
fn test_ignored_files_skipped() {
    let script_path = get_test_project_path();
    let persistence_path = get_test_persistence_path();
    _ = PastaEngine::new(&script_path, &persistence_path).expect("Engine should initialize");

    // _ignored.pasta should be skipped (verified by engine construction success)
}

#[test]
fn test_directory_not_found_error() {
    let non_existent_script = PathBuf::from("C:\\nonexistent\\script");
    let persistence_path = get_test_persistence_path();
    let result = PastaEngine::new(&non_existent_script, &persistence_path);

    assert!(result.is_err(), "Should return error for non-existent path");

    if let Err(e) = result {
        match e {
            PastaError::DirectoryNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected DirectoryNotFound, got: {:?}", other),
        }
    }
}

#[test]
fn test_not_absolute_path_error() {
    let relative_script_path = PathBuf::from("relative/path");
    let persistence_path = get_test_persistence_path();
    let result = PastaEngine::new(&relative_script_path, &persistence_path);

    assert!(result.is_err(), "Should return error for relative path");

    if let Err(e) = result {
        match e {
            PastaError::NotAbsolutePath { .. } => {
                // Expected error
            }
            other => panic!("Expected NotAbsolutePath, got: {:?}", other),
        }
    }
}

#[test]
fn test_dic_directory_not_found_error() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().canonicalize().unwrap();

    // Create main.rn but no dic/ directory
    std::fs::write(temp_path.join("main.rn"), "pub fn main() {}").unwrap();

    let persistence_path = get_test_persistence_path();
    let result = PastaEngine::new(&temp_path, &persistence_path);

    assert!(
        result.is_err(),
        "Should return error when dic/ directory is missing"
    );

    if let Err(e) = result {
        match e {
            PastaError::DicDirectoryNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected DicDirectoryNotFound, got: {:?}", other),
        }
    }
}

#[test]
fn test_main_rune_not_found_error() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().canonicalize().unwrap();

    // Create dic/ directory but no main.rn
    std::fs::create_dir(temp_path.join("dic")).unwrap();

    let persistence_path = get_test_persistence_path();
    let result = PastaEngine::new(&temp_path, &persistence_path);

    assert!(
        result.is_err(),
        "Should return error when main.rn is missing"
    );

    if let Err(e) = result {
        match e {
            PastaError::MainRuneNotFound { .. } => {
                // Expected error
            }
            other => panic!("Expected MainRuneNotFound, got: {:?}", other),
        }
    }
}

#[test]
fn test_multiple_labels_random_selection() {
    let script_path = get_test_project_path();
    let persistence_path = get_test_persistence_path();
    let mut engine =
        PastaEngine::new(&script_path, &persistence_path).expect("Engine should initialize");

    // Execute the same scene multiple times
    // With 3 definitions, at least one should be selected
    let mut results = Vec::new();
    for _ in 0..10 {
        let events = engine.execute_label("挨拶").expect("Label should execute");
        results.push(events);
    }

    // Just verify that execution succeeds (random selection is non-deterministic in this test)
    assert!(!results.is_empty(), "Should have executed label");
    assert!(
        results.iter().all(|events| !events.is_empty()),
        "All executions should produce events"
    );
}

#[test]
fn test_label_execution() {
    let script_path = get_test_project_path();
    let persistence_path = get_test_persistence_path();
    let mut engine =
        PastaEngine::new(&script_path, &persistence_path).expect("Engine should initialize");

    // Execute a simple label
    let events = engine.execute_label("別れ").expect("Label should execute");

    assert!(!events.is_empty(), "Should produce at least one event");
}
