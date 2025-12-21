// Phase 3: 基本的なcall/jump/local scene テスト

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;

#[test]
fn test_global_and_local_scenes() {
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

    // Global scene module
    assert!(result.contains("pub mod メイン_1"));
    assert!(result.contains("pub fn __start__(ctx, args)"));

    // local scenes
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

/// Phase 1 (REQ-BC-1): Jump statement is deprecated
/// This test verifies that Jump statement (？) is now rejected at parse time
#[test]
fn test_jump_statement_deprecated() {
    let pasta_content = r#"
＊メイン
　さくら：メイン発言
　？サブ

＊サブ
　さくら：サブ発言
"#;

    let result = parse_str(pasta_content, "test.pasta");

    // Phase 1: Jump statement should be rejected at parse time
    assert!(
        result.is_err(),
        "Phase 1: Jump statement (？) is deprecated. Use Call (＞) instead"
    );
}
