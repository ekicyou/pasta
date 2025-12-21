// Test PastaEngine with two-pass transpiler

use pasta::PastaEngine;
use tempfile::TempDir;

#[test]
fn test_engine_with_simple_project() {
    let script_root = std::env::current_dir()
        .unwrap()
        .join("tests/fixtures/simple-test");
    let temp_dir = TempDir::new().unwrap();

    // Create engine with simple test project
    let result = PastaEngine::new(&script_root, temp_dir.path());

    match result {
        Ok(mut engine) => {
            println!("Engine created successfully!");

            // Test scene existence by execution
            assert!(
                engine.execute_label("会話").is_ok(),
                "Label '会話' should exist"
            );
        }
        Err(e) => {
            panic!("Failed to create engine: {:?}", e);
        }
    }
}

#[test]
fn test_engine_with_test_project() {
    let script_root = std::env::current_dir()
        .unwrap()
        .join("tests/fixtures/test-project");
    let temp_dir = TempDir::new().unwrap();

    // Create engine with test project
    let result = PastaEngine::new(&script_root, temp_dir.path());

    match result {
        Ok(mut engine) => {
            println!("Engine created successfully!");

            // Test scene existence by execution (use actual Japanese scene name)
            assert!(
                engine.execute_label("挨拶").is_ok(),
                "Label '挨拶' should exist"
            );
        }
        Err(e) => {
            panic!("Failed to create engine: {:?}", e);
        }
    }
}

#[test]
fn test_engine_execute_label() {
    let script_root = std::env::current_dir()
        .unwrap()
        .join("tests/fixtures/test-project");
    let temp_dir = TempDir::new().unwrap();

    let mut engine = PastaEngine::new(&script_root, temp_dir.path()).unwrap();

    // Execute a scene (use actual Japanese scene name)
    let events = engine.execute_label("挨拶").unwrap();

    // Verify events
    assert!(!events.is_empty(), "Should generate events");
}
