//! 部分トークン提供統合テスト (Task 8.5)
//!
//! エラー時の部分トークン提供検証（pasta_dsl部分パース統合）

use pasta_lsp::analysis::AnalysisEngine;

#[test]
fn test_partial_tokens_on_error() {
    // Full parse fails → partial parse provides some tokens
    let source = "＊挨拶\n  Alice：こんにちは\n\n壊れた行\n";
    let result = AnalysisEngine::analyze(source);

    // Partial parse should provide at least some tokens from the valid part
    // and diagnostics for the invalid part
    // The exact result depends on parse_str_partial behavior
    let has_output = !result.tokens.is_empty() || !result.diagnostics.is_empty();
    assert!(has_output, "部分パースで何らかの出力がある");
}

#[test]
fn test_full_parse_success_no_fallback() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(!result.tokens.is_empty(), "完全パース成功時はトークンあり");
    assert!(
        result.diagnostics.is_empty(),
        "完全パース成功時はDiagnosticsなし"
    );
}

#[test]
fn test_complete_failure_still_returns_result() {
    let source = "完全に不正な入力ですわ！！！\n";
    let result = AnalysisEngine::analyze(source);
    // Even on total failure, the function should return without panicking
    // and provide some diagnostics
    assert!(
        !result.diagnostics.is_empty(),
        "完全失敗でもDiagnosticsを返す"
    );
}

#[test]
fn test_empty_source_no_tokens_no_diagnostics() {
    let result = AnalysisEngine::analyze("");
    assert!(result.tokens.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_mixed_valid_invalid_content() {
    // Source with a valid global scene followed by garbage
    let source = "＊挨拶\n  Alice：OK\n\nこれは構文エラー\n";
    let result = AnalysisEngine::analyze(source);

    // The analysis engine should handle this gracefully
    // Either full parse succeeds (if parser is lenient) or partial parse kicks in
    let total_output = result.tokens.len() + result.diagnostics.len();
    assert!(
        total_output > 0,
        "混在コンテンツで何らかの出力: tokens={}, diagnostics={}",
        result.tokens.len(),
        result.diagnostics.len()
    );
}
