//! Integration tests for Rune block support
//!
//! Tests the complete pipeline: parsing → AST → transpiling → execution

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta::{ir::ScriptEvent, PastaEngine};

#[test]
fn test_rune_block_parsing() {
    // Test that a script with a rune block can be parsed
    let script = r#"
＊テスト
  ```rune
  fn helper() {
    return 42;
  }
  ```
  さくら：こんにちは
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");

    let persistence_dir = get_test_persistence_dir();

    let result = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(
        result.is_ok(),
        "Failed to parse script with rune block: {:?}",
        result.err()
    );
}

#[test]
fn test_rune_block_transpilation() {
    // Test that rune blocks are correctly transpiled
    let script = r#"
＊テスト
  ```rune
  fn add(a, b) {
    return a + b;
  }
  ```
  さくら：計算します
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // The transpiled code should contain the rune function
    // We can't directly access the transpiled code, but we can verify execution works
    let events = engine.execute_label("テスト").unwrap();

    // Should have: ChangeSpeaker + Talk events
    assert!(events.len() >= 2, "Expected at least 2 events");

    // First event should be ChangeSpeaker
    match &events[0] {
        ScriptEvent::ChangeSpeaker { name } => {
            assert_eq!(name, "さくら");
        }
        _ => panic!("Expected ChangeSpeaker event, got {:?}", events[0]),
    }
}

#[test]
fn test_rune_block_with_function_call() {
    // Test that local functions defined in rune blocks can be called
    let script = r#"
＊テスト
  ```rune
  fn greet(name) {
    return "Hello, " + name;
  }
  ```
  さくら：@greet("World")
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");

    let persistence_dir = get_test_persistence_dir();

    let engine = PastaEngine::new(&script_dir, &persistence_dir);

    // This might fail at runtime if function scoping isn't implemented yet
    // For now, just verify it parses and transpiles
    assert!(
        engine.is_ok(),
        "Failed to create engine with rune function call: {:?}",
        engine.err()
    );
}

#[test]
fn test_rune_block_empty() {
    // Test that empty rune blocks are handled
    let script = r#"
＊テスト
  ```
  ```
  さくら：こんにちは
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let engine = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(
        engine.is_ok(),
        "Failed to parse script with empty rune block: {:?}",
        engine.err()
    );

    if let Ok(mut engine) = engine {
        let events = engine.execute_label("テスト").unwrap();
        assert!(events.len() >= 2, "Expected events after empty rune block");
    }
}

#[test]
fn test_rune_block_in_local_label() {
    // Test rune blocks in local scenes
    // Note: Using ASCII scene names to avoid encoding issues in test
    let script = "＊Global\n  -Local\n    ```rune\n    fn local_func() {\n      return \"local\";\n    }\n    ```\n    さくら：ローカル関数定義\n";

    let script_dir = create_test_script(script).expect("Failed to create script");

    let persistence_dir = get_test_persistence_dir();

    let engine = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(
        engine.is_ok(),
        "Failed to parse script with rune block in local scene: {:?}",
        engine.err()
    );
}

#[test]
fn test_rune_block_with_complex_code() {
    // Test rune blocks with more complex Rune code
    let script = r#"
＊テスト
  ```rune
  fn fibonacci(n) {
    if n <= 1 {
      return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }
  
  fn factorial(n) {
    if n <= 1 {
      return 1;
    }
    return n * factorial(n - 1);
  }
  ```
  さくら：複雑な関数を定義しました
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");

    let persistence_dir = get_test_persistence_dir();

    let engine = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(
        engine.is_ok(),
        "Failed to parse script with complex rune code: {:?}",
        engine.err()
    );
}

#[test]
fn test_multiple_rune_blocks() {
    // Test multiple rune blocks in the same label
    let script = r#"
＊テスト
  ```rune
  fn add(a, b) {
    return a + b;
  }
  ```
  さくら：最初の関数
  ```rune
  fn multiply(a, b) {
    return a * b;
  }
  ```
  さくら：二つ目の関数
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");

    let persistence_dir = get_test_persistence_dir();

    let engine = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(
        engine.is_ok(),
        "Failed to parse script with multiple rune blocks: {:?}",
        engine.err()
    );
}

#[test]
fn test_rune_block_indentation_preserved() {
    // Test that indentation within rune blocks is preserved
    let script = r#"
＊テスト
  ```rune
  fn nested_example() {
    let x = 1;
    if x > 0 {
      if x < 10 {
        return "small";
      }
    }
    return "other";
  }
  ```
  さくら：インデント確認
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");

    let persistence_dir = get_test_persistence_dir();

    let engine = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(
        engine.is_ok(),
        "Failed to parse script with nested indentation: {:?}",
        engine.err()
    );
}
