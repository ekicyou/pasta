//! Task 3.3: Integration test for Pass 1 → Pass 2 word code generation flow
//!
//! Tests that word definitions collected in Pass 1 result in correct code generation in Pass 2.

use pasta::parser::parse_str;
use pasta::transpiler::{LabelRegistry, Transpiler, WordDefRegistry};

/// Test that word reference generates correct Rune code
#[test]
fn test_word_reference_code_generation() {
    // Use word reference alone to avoid parsing ambiguity
    let source = r#"
＊ラベル
　キャラ：＠場所
"#;
    let ast = parse_str(source, "test.pasta").expect("parse failed");

    // Use transpile_to_string which handles both passes internally
    let code = Transpiler::transpile_to_string(&ast).expect("transpile failed");

    // Verify word call includes empty module name for global context
    assert!(
        code.contains(r#"pasta_stdlib::word("", "場所", [])"#),
        "Expected word call with empty module name, got:\n{}",
        code
    );
}

/// Test that global word definition is collected in Pass 1
#[test]
fn test_global_word_definition_collection() {
    let source = r#"
＠場所：東京　大阪　名古屋
＊メイン
　キャラ：＠場所
"#;
    let ast = parse_str(source, "test.pasta").expect("parse failed");
    let mut word_registry = WordDefRegistry::new();
    let mut label_registry = LabelRegistry::new();
    let mut output = Vec::new();

    // Pass 1 should collect word definitions
    Transpiler::transpile_pass1(&ast, &mut label_registry, &mut word_registry, &mut output)
        .expect("pass1 failed");

    // Verify word was registered
    let entries = word_registry.into_entries();
    assert_eq!(entries.len(), 1, "Expected 1 word entry");
    assert_eq!(entries[0].key, "場所");
    assert_eq!(entries[0].values, vec!["東京", "大阪", "名古屋"]);
}

/// Test that global and local word definitions are collected with transpile_with_registry
#[test]
fn test_transpile_with_registry_returns_word_registry() {
    let source = r#"
＠グローバル場所：東京　大阪
＊メイン
　キャラ：＠グローバル場所
"#;
    let ast = parse_str(source, "test.pasta").expect("parse failed");

    let (code, _label_registry, word_registry) =
        Transpiler::transpile_with_registry(&ast).expect("transpile failed");

    // Verify word was collected
    let entries = word_registry.into_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "グローバル場所");

    // Verify code generation
    assert!(
        code.contains(r#"pasta_stdlib::word("", "グローバル場所", [])"#),
        "Expected word call in generated code, got:\n{}",
        code
    );
}

/// Test multiple word definitions
#[test]
fn test_multiple_global_word_definitions() {
    let source = r#"
＠場所：東京　大阪
＠時間：朝　昼　夜
＊メイン
　キャラ：行くよ
"#;
    let ast = parse_str(source, "test.pasta").expect("parse failed");
    let mut word_registry = WordDefRegistry::new();
    let mut label_registry = LabelRegistry::new();
    let mut output = Vec::new();

    Transpiler::transpile_pass1(&ast, &mut label_registry, &mut word_registry, &mut output)
        .expect("pass1 failed");

    let entries = word_registry.into_entries();
    assert_eq!(entries.len(), 2, "Expected 2 word entries");

    let place = entries.iter().find(|e| e.key == "場所");
    let time = entries.iter().find(|e| e.key == "時間");

    assert!(place.is_some(), "Place word not found");
    assert!(time.is_some(), "Time word not found");

    assert_eq!(place.unwrap().values, vec!["東京", "大阪"]);
    assert_eq!(time.unwrap().values, vec!["朝", "昼", "夜"]);
}

/// Test that Pass 2 generates Talk with word call
#[test]
fn test_word_generates_talk_yield() {
    let source = r#"
＊テスト
　太郎：＠挨拶
"#;
    let ast = parse_str(source, "test.pasta").expect("parse failed");

    let code = Transpiler::transpile_to_string(&ast).expect("transpile failed");

    // Verify the generated code structure
    assert!(code.contains("yield Talk("), "Expected yield Talk statement");
    assert!(
        code.contains("pasta_stdlib::word("),
        "Expected word function call"
    );
}

/// Test word reference only (without mixed content due to parser behavior)
#[test]
fn test_word_reference_standalone() {
    let source = r#"
＊テスト
　キャラ：＠名前
"#;
    let ast = parse_str(source, "test.pasta").expect("parse failed");

    let code = Transpiler::transpile_to_string(&ast).expect("transpile failed");

    // Word reference should generate pasta_stdlib::word call
    assert!(
        code.contains(r#"pasta_stdlib::word("", "名前", [])"#),
        "Expected word call, got:\n{}",
        code
    );
}
