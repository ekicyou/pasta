//! Integration tests for transpiler2 module.
//!
//! These tests verify the complete transpilation flow:
//! - Pass 1 → Pass 2 integration
//! - Multiple files and scenes registration
//! - Attribute inheritance patterns

use pasta_rune::parser;
use pasta_rune::registry::{SceneRegistry, WordDefRegistry};
use pasta_rune::transpiler::Transpiler2;
use std::fs;
use std::path::PathBuf;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/transpiler2")
}

fn read_fixture(name: &str) -> String {
    let path = fixtures_path().join(name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read fixture: {:?}", path))
}

// ====================
// Pass 1 → Pass 2 Integration Tests
// ====================

#[test]
fn test_pass1_pass2_complete_flow() {
    let source = read_fixture("simple_scene.pasta");
    let file = parser::parse_str(&source, "simple_scene.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    // Pass 1
    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    // Verify Pass 1 results
    assert!(!scene_registry.all_scenes().is_empty());

    // Pass 2
    Transpiler2::transpile_pass2(&scene_registry, &mut output).unwrap();

    let result = String::from_utf8(output).unwrap();

    // Verify complete output structure
    assert!(result.contains("pub mod"));
    assert!(result.contains("pub mod __pasta_trans2__"));
    assert!(result.contains("scene_selector"));
    assert!(result.contains("pub mod pasta"));
}

#[test]
fn test_transpile_to_string_complete() {
    let source = read_fixture("simple_scene.pasta");
    let file = parser::parse_str(&source, "simple_scene.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify generated Rune code structure
    assert!(result.contains("pub mod 挨拶_1"));
    assert!(result.contains("pub fn __start__"));
    assert!(result.contains("yield change_speaker"));
    assert!(result.contains("yield Talk"));
}

// ====================
// Multiple Scenes Tests
// ====================

#[test]
fn test_multi_scene_registration() {
    let source = read_fixture("multi_scene.pasta");
    let file = parser::parse_str(&source, "multi_scene.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let scenes = scene_registry.all_scenes();
    assert_eq!(scenes.len(), 3); // 挨拶、会話、別れ

    // Verify scene names
    let names: Vec<_> = scenes.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"挨拶"));
    assert!(names.contains(&"会話"));
    assert!(names.contains(&"別れ"));
}

#[test]
fn test_multi_scene_selector_generation() {
    let source = read_fixture("multi_scene.pasta");
    let file = parser::parse_str(&source, "multi_scene.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify all scene modules are generated
    assert!(result.contains("pub mod 挨拶_1"));
    assert!(result.contains("pub mod 会話_1"));
    assert!(result.contains("pub mod 別れ_1"));

    // Verify selector matches all scenes
    assert!(result.contains("1 =>"));
    assert!(result.contains("2 =>"));
    assert!(result.contains("3 =>"));
}

// ====================
// Nested Scenes Tests
// ====================

#[test]
fn test_nested_scene_structure() {
    let source = read_fixture("nested_scene.pasta");
    let file = parser::parse_str(&source, "nested_scene.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify global module and local scene functions
    assert!(result.contains("pub mod メイン_1"));
    assert!(result.contains("pub fn __start__"));

    // Local named scenes should be generated as numbered functions
    // (exact format depends on implementation)
}

// ====================
// Word Definition Tests
// ====================

#[test]
fn test_word_definitions_registration() {
    let source = read_fixture("word_definitions.pasta");
    let file = parser::parse_str(&source, "word_definitions.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let entries = word_registry.all_entries();

    // Should have both global and local word definitions
    assert!(!entries.is_empty());

    // Verify global word is registered
    let global_word = entries.iter().find(|e| e.key == "挨拶");
    assert!(global_word.is_some());
}

// ====================
// Attribute Inheritance Tests
// ====================

#[test]
fn test_file_level_attribute_inheritance() {
    let source = read_fixture("attribute_inheritance.pasta");
    let file = parser::parse_str(&source, "attribute_inheritance.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let scenes = scene_registry.all_scenes();

    // First scene should inherit file attrs and have its own
    let greeting_scene = scenes.iter().find(|s| s.name == "挨拶");
    if let Some(scene) = greeting_scene {
        // Should have inherited 天気 from file level
        assert!(scene.attributes.contains_key("天気"));
    }
}

// ====================
// Variable Scope Tests
// ====================

#[test]
fn test_variable_scope_generation() {
    let source = read_fixture("variable_scope.pasta");
    let file = parser::parse_str(&source, "variable_scope.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify local variable assignment
    assert!(result.contains("ctx.local"));

    // Verify global variable assignment
    assert!(result.contains("ctx.global"));
}

// ====================
// Continue Lines Tests
// ====================

#[test]
fn test_continue_lines_generation() {
    let source = read_fixture("continue_lines.pasta");
    let file = parser::parse_str(&source, "continue_lines.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify speaker change is generated
    assert!(result.contains("yield change_speaker"));

    // Verify multiple Talk actions
    let talk_count = result.matches("yield Talk").count();
    assert!(talk_count >= 2, "Expected multiple Talk actions");
}

// ====================
// Code Blocks Tests
// ====================

#[test]
fn test_code_blocks_processing() {
    let source = read_fixture("code_blocks.pasta");
    let file = parser::parse_str(&source, "code_blocks.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Code blocks should be included in output
    // (exact format depends on CodeBlock processing implementation)
    assert!(result.contains("pub mod"));
}

// ====================
// Error Handling Tests
// ====================

#[test]
fn test_empty_file_produces_valid_output() {
    let file = pasta::parser::PastaFile::new(PathBuf::from("empty.pasta"));

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Empty file should still produce valid selector structure
    assert!(result.contains("__pasta_trans2__"));
    assert!(result.contains("pasta"));
}

#[test]
fn test_multiple_files_share_registry() {
    let source1 = "＊シーンA\n  actor：hello\n";
    let source2 = "＊シーンB\n  actor：world\n";

    let file1 = parser::parse_str(source1, "file1.pasta").unwrap();
    let file2 = parser::parse_str(source2, "file2.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    // Process both files with shared registry
    Transpiler2::transpile_pass1(&file1, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();
    Transpiler2::transpile_pass1(&file2, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    // Both scenes should be registered
    let scenes = scene_registry.all_scenes();
    assert_eq!(scenes.len(), 2);

    // Generate selector with all scenes
    Transpiler2::transpile_pass2(&scene_registry, &mut output).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("1 =>"));
    assert!(result.contains("2 =>"));
}

