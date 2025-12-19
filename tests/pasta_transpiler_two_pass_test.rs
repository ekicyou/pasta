// Task 3.2: 2パストランスパイラー統合テスト
// Writeトレイトへの出力と2パス処理を検証

use pasta::parser::parse_str;
use pasta::transpiler::{LabelRegistry, Transpiler};

#[test]
fn test_two_pass_transpiler_to_vec() {
    // Simple pasta script
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    let ast = parse_str(pasta_code, "test.pasta").unwrap();

    // Pass 1: Output to Vec<u8>
    let mut registry = LabelRegistry::new();
    let mut output = Vec::new();

    Transpiler::transpile_pass1(&ast, &mut registry, &mut output).unwrap();

    let pass1_output = String::from_utf8(output.clone()).unwrap();
    println!("Pass 1 output:\n{}", pass1_output);

    // Verify Pass 1 output contains module but not mod pasta
    assert!(pass1_output.contains("pub mod 会話_1"));
    assert!(pass1_output.contains("pub fn __start__(ctx, args)"));
    assert!(!pass1_output.contains("pub mod pasta")); // mod pasta not yet generated

    // Pass 2: Append mod pasta
    Transpiler::transpile_pass2(&registry, &mut output).unwrap();

    let final_output = String::from_utf8(output).unwrap();
    println!("\nFinal output:\n{}", final_output);

    // Verify final output contains __pasta_trans2__ module
    assert!(final_output.contains("pub mod __pasta_trans2__"));
    assert!(final_output.contains("pub fn label_selector(label, filters)"));

    // Verify pasta module calls label_selector
    assert!(final_output.contains("pub mod pasta"));
    assert!(
        final_output.contains("pub fn jump(ctx, label, filters, args)")
            || final_output.contains("pub fn call(ctx, label, filters, args)")
    );
    assert!(final_output
        .contains("let func = crate::__pasta_trans2__::label_selector(label, filters);"));
    assert!(final_output.contains("for a in func(ctx, args) { yield a; }"));

    // Verify match expression is in __pasta_trans2__ module (function pointer, not call)
    assert!(final_output.contains("1 => crate::会話_1::__start__,"));
}

#[test]
fn test_two_pass_transpiler_to_string() {
    // Simple pasta script
    let pasta_code = r#"
＊会話
　さくら：こんにちは

＊別会話
　うにゅう：やあ
"#;

    let ast = parse_str(pasta_code, "test.pasta").unwrap();

    // Pass 1: Output to String (via Vec<u8>)
    let mut registry = LabelRegistry::new();

    // String doesn't impl Write, so use Vec<u8>
    let mut buffer = Vec::new();
    Transpiler::transpile_pass1(&ast, &mut registry, &mut buffer).unwrap();
    let mut output = String::from_utf8(buffer).unwrap();

    // Verify labels are registered
    let labels = registry.all_labels();
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0].name, "会話");
    assert_eq!(labels[1].name, "別会話");

    // Pass 2
    let mut buffer = Vec::new();
    Transpiler::transpile_pass2(&registry, &mut buffer).unwrap();
    let pass2_output = String::from_utf8(buffer).unwrap();

    output.push_str(&pass2_output);

    println!("Full output:\n{}", output);

    // Verify both labels in __pasta_trans2__ module (function pointers)
    assert!(output.contains("1 => crate::会話_1::__start__,"));
    assert!(output.contains("2 => crate::別会話_1::__start__,"));

    // Verify pasta module structure
    assert!(output.contains("let func = crate::__pasta_trans2__::label_selector(label, filters);"));
    assert!(output.contains("for a in func(ctx, args) { yield a; }"));
}

#[test]
fn test_transpile_to_string_helper() {
    // Test the convenience helper for single-file testing
    let pasta_code = r#"
＊会話
　さくら：テストです
"#;

    let ast = parse_str(pasta_code, "test.pasta").unwrap();

    let output = Transpiler::transpile_to_string(&ast).unwrap();

    println!("Output:\n{}", output);

    // Should contain both Pass 1 and Pass 2 output
    assert!(output.contains("pub mod 会話_1"));
    assert!(output.contains("pub mod __pasta_trans2__"));
    assert!(output.contains("pub mod pasta"));

    // Verify correct structure (function pointer in __pasta_trans2__)
    assert!(output.contains("1 => crate::会話_1::__start__,"));
    assert!(output.contains("let func = crate::__pasta_trans2__::label_selector(label, filters);"));
}

#[test]
fn test_multiple_files_simulation() {
    // Simulate processing multiple pasta files
    let file1 = r#"
＊メイン
　さくら：メインです
"#;

    let file2 = r#"
＊サブ
　うにゅう：サブです
"#;

    let ast1 = parse_str(file1, "file1.pasta").unwrap();
    let ast2 = parse_str(file2, "file2.pasta").unwrap();

    // Shared registry across files
    let mut registry = LabelRegistry::new();
    let mut output = Vec::new();

    // Pass 1 for file1
    Transpiler::transpile_pass1(&ast1, &mut registry, &mut output).unwrap();

    // Pass 1 for file2
    Transpiler::transpile_pass1(&ast2, &mut registry, &mut output).unwrap();

    // Pass 2 once for all files
    Transpiler::transpile_pass2(&registry, &mut output).unwrap();

    let final_output = String::from_utf8(output).unwrap();
    println!("Multi-file output:\n{}", final_output);

    // Should contain both files' labels
    assert!(final_output.contains("pub mod メイン_1"));
    assert!(final_output.contains("pub mod サブ_1"));

    // __pasta_trans2__ module should have both labels (function pointers)
    assert!(final_output.contains("1 => crate::メイン_1::__start__,"));
    assert!(final_output.contains("2 => crate::サブ_1::__start__,"));

    // Verify pasta module wrapper structure
    assert!(final_output
        .contains("let func = crate::__pasta_trans2__::label_selector(label, filters);"));
}
