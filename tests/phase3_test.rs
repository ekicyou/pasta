// Phase 3: 基本的なcall/jump/local label テスト

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;

#[test]
fn test_global_and_local_labels() {
    let pasta_content = r#"
＊メイン
　さくら：最初の発言
　
　-選択肢１
　さくら：選択肢１の発言
　
　-選択肢２
　さくら：選択肢２の発言
"#;

    let ast = parse_str(pasta_content, "test.pasta").expect("Failed to parse");
    let result = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled ===\n{}", result);

    // Global label module
    assert!(result.contains("pub mod メイン_1"));
    assert!(result.contains("pub fn __start__(ctx, args)"));

    // Local labels
    assert!(result.contains("pub fn 選択肢１_1(ctx, args)"));
    assert!(result.contains("pub fn 選択肢２_1(ctx, args)"));

    // Local functions should be in the module
    // (Simplified: just check they exist in the output)
    assert!(result.contains("メイン_1") && result.contains("選択肢１_1"));
    assert!(result.contains("メイン_1") && result.contains("選択肢２_1"));
}

#[test]
fn test_call_statement() {
    let pasta_content = r#"
＊メイン
　さくら：メイン発言
　＞サブ

＊サブ
　さくら：サブ発言
"#;

    let ast = parse_str(pasta_content, "test.pasta").expect("Failed to parse");
    let result = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled ===\n{}", result);

    // Call statement should generate pasta::call()
    assert!(result.contains("pasta::call"));
}

#[test]
fn test_jump_statement() {
    let pasta_content = r#"
＊メイン
　さくら：メイン発言
　？サブ

＊サブ
　さくら：サブ発言
"#;

    let ast = parse_str(pasta_content, "test.pasta").expect("Failed to parse");
    let result = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled ===\n{}", result);

    // Jump statement should generate pasta::jump()
    assert!(result.contains("pasta::jump"));
}
