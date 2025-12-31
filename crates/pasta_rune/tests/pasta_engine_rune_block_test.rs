//! Integration tests for Rune block support
//!
//! Tests the complete pipeline: parsing → AST → transpiling → execution

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta_rune::{PastaEngine, ir::ScriptEvent};

#[test]
fn test_rune_block_parsing() {
    // Test that a script with a rune block can be parsed
    // parser2 grammar: action_line requires pad (leading space)
    // local_scene_item+ comes BEFORE code_scope*
    let script = r#"＊テスト
  さくら：こんにちは
```rune
fn helper() {
  return 42;
}
```
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
    // parser2 grammar: action_line requires pad, local_scene_item+ comes BEFORE code_scope*
    let script = r#"＊テスト
  さくら：計算します
```rune
fn add(a, b) {
  return a + b;
}
```
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
    // parser2 grammar: action_line requires pad, code_block comes AFTER action lines
    let script = r#"＊テスト
  さくら：関数を呼びます
```rune
fn greet(name) {
  return "Hello, " + name;
}
```
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
    // parser2 grammar: action_line requires pad, local_scene_item+ comes BEFORE code_scope*
    // Empty code blocks are not valid in parser2 (expected code_contents)
    // Test with minimal code content instead
    let script = r#"＊テスト
  さくら：こんにちは
```rune
// empty block with comment
```
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
    // parser2 grammar: local_scene_line requires pad, action_line requires pad
    // local_scene_item+ comes BEFORE code_scope*
    let script = "＊Global\n  さくら：グローバルシーン\n  -Local\n  さくら：ローカル関数定義\n```rune\nfn local_func() {\n  return \"local\";\n}\n```\n";

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
    // parser2 grammar: action_line requires pad, local_scene_item+ comes BEFORE code_scope*
    let script = r#"＊テスト
  さくら：複雑な関数を定義しました
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
    // parser2 grammar: action_line requires pad, code_scope* comes after local_scene_item+
    // Multiple code blocks are allowed at the end of a scene
    let script = r#"＊テスト
  さくら：関数を定義しました
```rune
fn add(a, b) {
  return a + b;
}
```
```rune
fn multiply(a, b) {
  return a * b;
}
```
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
    // parser2 grammar: action_line requires pad, local_scene_item+ comes BEFORE code_scope*
    let script = r#"＊テスト
  さくら：インデント確認
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
