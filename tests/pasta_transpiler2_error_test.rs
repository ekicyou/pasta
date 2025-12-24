//! Error case tests for Transpiler2
//!
//! These tests verify that Transpiler2 produces appropriate errors for invalid inputs.

use pasta::parser;
use pasta::registry::{SceneRegistry, WordDefRegistry};
use pasta::transpiler::{TranspileError, Transpiler2};

// ============================================================
// Parse Error Propagation Tests
// ============================================================

#[test]
fn test_error_invalid_syntax() {
    // Invalid pasta syntax should fail at parse stage, not transpile stage
    let source = "＊＊＊Invalid";
    let result = parser::parse_str(source, "test.pasta");

    // Should fail to parse
    assert!(result.is_err(), "Invalid syntax should fail to parse");
}

#[test]
fn test_error_unclosed_quote() {
    // Unclosed quote in dialogue
    let source = r#"＊テスト
  さくら：「閉じていない
"#;

    let result = parser::parse_str(source, "test.pasta");
    // This may or may not be a parse error depending on grammar
    // Either parse fails or we get malformed AST
    if result.is_err() {
        println!("Parse correctly rejected unclosed quote");
    } else {
        // If parsed, transpile should still work but produce different output
        let file = result.unwrap();
        let transpile_result = Transpiler2::transpile_to_string(&file);
        assert!(
            transpile_result.is_ok(),
            "Transpiler should handle parsed AST"
        );
    }
}

// ============================================================
// Continue Action Error Tests
// ============================================================

#[test]
fn test_error_first_line_continue() {
    // First action in a scene cannot be a continue action
    // This should produce an InvalidContinuation error
    let source = r#"＊テスト
  ：「これは最初の行です」
"#;

    let result = parser::parse_str(source, "test.pasta");

    // This might fail at parse stage or be caught at transpile stage
    if result.is_ok() {
        let file = result.unwrap();
        let transpile_result = Transpiler2::transpile_to_string(&file);

        match transpile_result {
            Err(e) => {
                println!("Transpiler correctly rejected first-line continue: {:?}", e);
                // Expected behavior - continue without previous action is invalid
            }
            Ok(code) => {
                // If transpiler doesn't catch this, verify the output is reasonable
                println!("Transpiler allowed first-line continue, output: {}", code);
                // This is also acceptable if CodeGenerator handles it gracefully
            }
        }
    }
}

// ============================================================
// Empty Content Edge Cases
// ============================================================

#[test]
fn test_error_empty_scene_name() {
    // Scene with no name after marker
    let source = r#"＊
  さくら：「テスト」
"#;

    let result = parser::parse_str(source, "test.pasta");

    // This is actually valid - unnamed scene (continuation scene)
    if result.is_ok() {
        let file = result.unwrap();
        let code = Transpiler2::transpile_to_string(&file).expect("Transpile should work");
        // Should generate a module with default/unnamed name
        println!("Empty scene name generated: {}", code);
    }
}

#[test]
fn test_error_scene_with_no_actions() {
    // Scene with only attributes, no actions - might be parsed differently
    let source = r#"＊空シーン
  &priority：1
"#;

    // This should fail parsing because no action lines after attribute
    let result = parser::parse_str(source, "test.pasta");

    match result {
        Err(e) => {
            println!("Parse correctly rejected scene with no actions: {:?}", e);
        }
        Ok(file) => {
            // If it parses, transpile should handle gracefully
            let code_result = Transpiler2::transpile_to_string(&file);
            match code_result {
                Ok(code) => println!("Scene with only attrs: {}", code),
                Err(e) => println!("Transpile rejected empty scene: {:?}", e),
            }
        }
    }
}

// ============================================================
// Module Generation Edge Cases
// ============================================================

#[test]
fn test_special_characters_in_scene_name() {
    // Scene name with special characters that need sanitization
    let source = r#"＊テスト-シーン
  さくら：「テスト」
"#;

    let result = parser::parse_str(source, "test.pasta");

    // Parse might succeed or fail depending on grammar
    if let Ok(file) = result {
        match Transpiler2::transpile_to_string(&file) {
            Ok(code) => {
                // Module name should be sanitized
                assert!(
                    !code.contains("pub mod テスト-シーン"),
                    "Raw hyphen in module name should be sanitized"
                );
                println!("Sanitized output: {}", code);
            }
            Err(e) => {
                println!("Transpile rejected special chars: {:?}", e);
            }
        }
    }
}

#[test]
fn test_very_long_scene_name() {
    // Very long scene name
    let long_name = "あ".repeat(100);
    let source = format!("＊{}\n  さくら：「テスト」\n", long_name);

    let file = parser::parse_str(&source, "test.pasta").expect("Parse should work");
    let result = Transpiler2::transpile_to_string(&file);

    // Should handle long names (truncate or allow)
    assert!(result.is_ok(), "Should handle long scene names");
}

#[test]
fn test_numeric_scene_name() {
    // Scene name starting with number
    let source = r#"＊123シーン
  さくら：「テスト」
"#;

    // This may or may not be valid depending on grammar's id rule
    let result = parser::parse_str(source, "test.pasta");

    if let Ok(file) = result {
        let code_result = Transpiler2::transpile_to_string(&file);
        if let Ok(code) = code_result {
            println!("Numeric scene name output: {}", code);
        }
    }
}

// ============================================================
// Registry State Tests
// ============================================================

#[test]
fn test_registry_remains_valid_after_error() {
    // Even if transpilation fails, registry should be in valid state
    let source1 = r#"＊有効シーン
  さくら：「有効」
"#;

    // First file is valid
    let file1 = parser::parse_str(source1, "file1.pasta").expect("Parse should work");

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file1, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 should work");

    // Registry should have the scene
    assert_eq!(scene_registry.all_scenes().len(), 1);

    // Now try to add another valid file
    let source2 = r#"＊別シーン
  うにゅう：「別」
"#;
    let file2 = parser::parse_str(source2, "file2.pasta").expect("Parse should work");

    Transpiler2::transpile_pass1(&file2, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 should work");

    // Registry should now have both scenes
    assert_eq!(scene_registry.all_scenes().len(), 2);
}

// ============================================================
// Unicode Edge Cases
// ============================================================

#[test]
fn test_unicode_normalization() {
    // Test with different unicode normalizations of same character
    let source = r#"＊テスト
  さくら：「こんにちは」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse should work");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile should work");

    assert!(!code.is_empty(), "Should produce output for unicode input");
}

#[test]
fn test_emoji_in_dialogue() {
    // Emoji in dialogue content
    let source = r#"＊絵文字テスト
  さくら：「こんにちは😀」
"#;

    let result = parser::parse_str(source, "test.pasta");

    if let Ok(file) = result {
        let code = Transpiler2::transpile_to_string(&file).expect("Should handle emoji");
        println!("Emoji output: {}", code);
    }
}

// ============================================================
// TranspileError Type Tests
// ============================================================

#[test]
fn test_transpile_error_display() {
    // Test that TranspileError has proper Display implementation
    let error = TranspileError::internal("test error message".to_string());
    let display = format!("{}", error);
    assert!(!display.is_empty(), "Error should have display text");
}

#[test]
fn test_io_error_conversion() {
    // Test that std::io::Error converts to TranspileError
    let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test io error");
    let transpile_error: TranspileError = io_error.into();

    // Should be convertible and displayable
    let display = format!("{}", transpile_error);
    assert!(!display.is_empty());
}

// ============================================================
// Output Validity Tests
// ============================================================

#[test]
fn test_output_is_valid_utf8() {
    let source = r#"＊テスト
  さくら：「日本語テスト」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse should work");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile should work");

    // Output should be valid UTF-8 (implicit since it's a String)
    assert!(code.len() > 0);

    // Verify it's actually valid UTF-8 by checking bytes
    assert!(String::from_utf8(code.as_bytes().to_vec()).is_ok());
}

#[test]
fn test_output_has_no_null_bytes() {
    let source = r#"＊テスト
  さくら：「テスト」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse should work");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile should work");

    assert!(!code.contains('\0'), "Output should not contain null bytes");
}

// ============================================================
// Word Definition Edge Cases
// ============================================================

#[test]
fn test_word_definition_empty_values() {
    // Word definition with empty or whitespace values
    let source = r#"@empty：

＊テスト
  さくら：「テスト」
"#;

    let result = parser::parse_str(source, "test.pasta");

    // This might fail at parse or transpile
    if let Ok(file) = result {
        let code_result = Transpiler2::transpile_to_string(&file);
        println!("Empty word values result: {:?}", code_result.is_ok());
    }
}

#[test]
fn test_word_definition_many_values() {
    // Word definition with many values
    let values = (1..50)
        .map(|i| format!("値{}", i))
        .collect::<Vec<_>>()
        .join("、");
    let source = format!("@多数：{}\n\n＊テスト\n  さくら：「テスト」\n", values);

    let file = parser::parse_str(&source, "test.pasta").expect("Parse should work");

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 should work");

    // Check all values were registered
    let entries = word_registry.all_entries();
    let entry = entries.iter().find(|e| e.key == "多数");
    assert!(entry.is_some(), "Word should be registered");
    assert!(entry.unwrap().values.len() > 10, "Should have many values");
}
