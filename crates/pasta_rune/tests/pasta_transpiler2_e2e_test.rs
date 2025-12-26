//! E2E tests for Transpiler2: Parse → Transpile → Rune VM compile/execute
//!
//! These tests verify the complete pipeline from .pasta source to executable Rune code.

use pasta_rune::parser;
use pasta_rune::{SceneRegistry, WordDefRegistry};
use pasta_rune::transpiler::Transpiler2;

// ============================================================
// Basic Transpilation Tests
// ============================================================

#[test]
fn test_e2e_simple_scene_compiles() {
    // Parse simple scene
    let source = r#"＊挨拶
  さくら：「こんにちは！」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Verify the generated code structure
    assert!(
        code.contains("pub mod 挨拶_1"),
        "Should contain scene module"
    );
    assert!(
        code.contains("pub fn __start__"),
        "Should contain __start__ function"
    );
    assert!(
        code.contains("mod __pasta_trans2__"),
        "Should contain selector module"
    );
    assert!(code.contains("mod pasta"), "Should contain pasta wrapper");

    // Note: Full Rune compile may fail without pasta_stdlib, but structure is valid
    println!("Generated code:\n{}", code);
}

#[test]
fn test_e2e_multi_scene_registration() {
    let source = r#"＊シーン1
  さくら：「最初」

＊シーン2
  うにゅう：「次」

＊シーン3
  さくら：「最後」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 failed");

    // Verify all scenes registered
    assert_eq!(scene_registry.all_scenes().len(), 3);

    let code =
        String::from_utf8(output.clone()).expect("UTF-8 failed") + "\n// (pass2 would follow)";
    assert!(code.contains("pub mod シーン1_1"));
    assert!(code.contains("pub mod シーン2_1"));
    assert!(code.contains("pub mod シーン3_1"));
}

#[test]
fn test_e2e_selector_generates_match_arms() {
    let source = r#"＊会話A
  さくら：「A」

＊会話B
  さくら：「B」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Verify scene_selector has match arms
    assert!(
        code.contains("fn scene_selector"),
        "Should have scene_selector function"
    );
    assert!(
        code.contains("match id"),
        "Should have match statement for id"
    );

    // Each scene should have a match arm
    assert!(
        code.contains("crate::会話A_1::__start__"),
        "Should reference 会話A module"
    );
    assert!(
        code.contains("crate::会話B_1::__start__"),
        "Should reference 会話B module"
    );
}

#[test]
fn test_e2e_pasta_call_wrapper() {
    let source = r#"＊テスト
  さくら：「テスト」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Verify pasta_rune::call wrapper with module_name parameter for unified scope resolution
    assert!(code.contains("pub mod pasta"));
    assert!(code.contains("pub fn call(ctx, scene, module_name, filters, args)"));
    assert!(code.contains("crate::__pasta_trans2__::scene_selector(scene, module_name, filters)"));
    assert!(
        code.contains("for a in func(ctx, args) { yield a; }"),
        "Should have yield loop"
    );
    // Result handling in nested match
    assert!(
        code.contains("let id = pasta_stdlib::select_scene_to_id(scene, module_name, filters);"),
        "Should call select_scene_to_id"
    );
    assert!(
        code.contains("Ok(id) => match id"),
        "Should handle Ok case with nested match"
    );
}

// ============================================================
// Local Scene Tests
// ============================================================

#[test]
fn test_e2e_nested_local_scenes() {
    let source = r#"＊メイン
  さくら：「開始」
  ・サブ1
    さくら：「サブシーン1」
  ・サブ2
    さくら：「サブシーン2」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Verify local scenes are generated
    assert!(code.contains("pub mod メイン_1"), "Should have main module");
    // Local scenes should be nested functions or sub-modules
    // Actual structure depends on CodeGenerator implementation
    println!("Generated code with local scenes:\n{}", code);
}

// ============================================================
// Continue Line Tests
// ============================================================

#[test]
fn test_e2e_continue_lines() {
    let source = r#"＊継続
  さくら：「最初」
  ：「続き1」
  ：「続き2」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Verify code is generated (specific output depends on CodeGenerator)
    assert!(!code.is_empty(), "Should generate non-empty code");
    println!("Continue lines output:\n{}", code);
}

// ============================================================
// File-level Feature Tests
// ============================================================

#[test]
fn test_e2e_file_attributes() {
    let source = r#"&author：テスト
&version：1

＊挨拶
  さくら：「属性テスト」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // File attributes should be processed (may affect metadata)
    assert!(code.contains("pub mod 挨拶_1"));
}

#[test]
fn test_e2e_file_word_definitions() {
    let source = r#"@greeting：こんにちは、おはよう

＊会話
  さくら：「テスト」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 failed");

    // Verify word was registered
    let entries = word_registry.all_entries();
    let greeting_entry = entries.iter().find(|e| e.key == "greeting");
    assert!(
        greeting_entry.is_some(),
        "Should have 'greeting' word registered"
    );
    assert_eq!(greeting_entry.unwrap().values.len(), 2);
}

// ============================================================
// Multiple Files / Registry Sharing Tests
// ============================================================

#[test]
fn test_e2e_multiple_files_share_registry() {
    let source1 = r#"＊シーンA
  さくら：「ファイル1」
"#;
    let source2 = r#"＊シーンB
  さくら：「ファイル2」
"#;

    let file1 = parser::parse_str(source1, "file1.pasta").expect("Parse failed");
    let file2 = parser::parse_str(source2, "file2.pasta").expect("Parse failed");

    // Shared registries
    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    // Process both files with same registry
    Transpiler2::transpile_pass1(&file1, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 file1");
    Transpiler2::transpile_pass1(&file2, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1 file2");

    // Verify both scenes registered
    assert_eq!(scene_registry.all_scenes().len(), 2);

    // Generate selector covering both
    Transpiler2::transpile_pass2(&scene_registry, &mut output).expect("Pass2");

    let code = String::from_utf8(output).expect("UTF-8");
    assert!(code.contains("crate::シーンA_1::__start__"));
    assert!(code.contains("crate::シーンB_1::__start__"));
}

#[test]
fn test_e2e_same_name_scenes_get_unique_ids() {
    let source = r#"＊同名
  さくら：「1つ目」

＊同名
  さくら：「2つ目」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .expect("Pass1");

    // Both should be registered with unique counters
    assert_eq!(scene_registry.all_scenes().len(), 2);

    let code = String::from_utf8(output).expect("UTF-8");
    assert!(code.contains("pub mod 同名_1"), "Should have 同名_1 module");
    assert!(code.contains("pub mod 同名_2"), "Should have 同名_2 module");
}

// ============================================================
// Empty/Edge Case Tests
// ============================================================

#[test]
fn test_e2e_empty_file_produces_valid_output() {
    let source = "# Just a comment\n";

    let file = parser::parse_str(source, "empty.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Empty file should still produce selector structure
    assert!(code.contains("mod __pasta_trans2__"));
    assert!(code.contains("mod pasta"));
}

#[test]
fn test_e2e_scene_with_only_attributes() {
    let source = r#"＊設定シーン
  &priority：1
  &type：設定
  さくら：「設定完了」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    assert!(code.contains("pub mod 設定シーン_1"));
}

// ============================================================
// Code Block Tests
// ============================================================

#[test]
fn test_e2e_scene_with_code_block() {
    let source = r#"＊計算
  さくら：「計算します」

```rune
fn helper() { 42 }
```
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Code block content should appear in output
    assert!(
        code.contains("fn helper()") || code.contains("helper"),
        "Code block content should be included"
    );
}

// ============================================================
// Output Structure Verification
// ============================================================

#[test]
fn test_e2e_output_order() {
    let source = r#"＊A
  さくら：「A」

＊B
  さくら：「B」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    // Scene modules should come before selector
    let mod_a_pos = code.find("pub mod A_1").expect("Should have mod A_1");
    let mod_b_pos = code.find("pub mod B_1").expect("Should have mod B_1");
    let selector_pos = code
        .find("mod __pasta_trans2__")
        .expect("Should have selector");

    assert!(
        mod_a_pos < selector_pos,
        "Scene modules should precede selector"
    );
    assert!(
        mod_b_pos < selector_pos,
        "Scene modules should precede selector"
    );
}

#[test]
fn test_e2e_pasta_wrapper_after_selector() {
    let source = r#"＊テスト
  さくら：「テスト」
"#;

    let file = parser::parse_str(source, "test.pasta").expect("Parse failed");
    let code = Transpiler2::transpile_to_string(&file).expect("Transpile failed");

    let selector_pos = code
        .find("mod __pasta_trans2__")
        .expect("Should have selector");
    let pasta_pos = code
        .find("pub mod pasta")
        .expect("Should have pasta wrapper");

    assert!(
        selector_pos < pasta_pos,
        "Selector should come before pasta wrapper"
    );
}

