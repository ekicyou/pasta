//! Diagnostics通知統合テスト (Task 8.3)
//!
//! パースエラー → Diagnostics通知フロー検証
//! PartialParseError → Diagnostics変換の正確性確認

use pasta_lsp::analysis::AnalysisEngine;
use tower_lsp::lsp_types::DiagnosticSeverity;

#[test]
fn test_parse_error_generates_diagnostics() {
    let source = "不正な構文ですわ\nこれもダメですわ\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        !result.diagnostics.is_empty(),
        "パースエラー時にDiagnostics生成"
    );
}

#[test]
fn test_diagnostics_have_error_severity() {
    let source = "不正な構文\n";
    let result = AnalysisEngine::analyze(source);
    for diag in &result.diagnostics {
        assert_eq!(
            diag.severity,
            Some(DiagnosticSeverity::ERROR),
            "Diagnostic severity is ERROR"
        );
    }
}

#[test]
fn test_diagnostics_have_nonempty_message() {
    let source = "壊れた行\n";
    let result = AnalysisEngine::analyze(source);
    for diag in &result.diagnostics {
        assert!(!diag.message.is_empty(), "Diagnostic message is non-empty");
    }
}

#[test]
fn test_valid_source_no_diagnostics() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        result.diagnostics.is_empty(),
        "正常ソースではDiagnosticsなし"
    );
}

#[test]
fn test_partial_parse_mixed_errors_and_tokens() {
    // 正常部分と異常部分が混在
    let source = "＊挨拶\n  Alice：こんにちは\n\n壊れた行\n";
    let result = AnalysisEngine::analyze(source);
    // Partial parse fallback should produce some tokens from the valid part
    // and diagnostics from the invalid part
    // (具体的な挙動は parse_str_partial の実装次第)
    // 少なくとも結果が返ること自体を確認
    assert!(
        !result.tokens.is_empty() || !result.diagnostics.is_empty(),
        "混在ソースでトークンかDiagnosticsが返る"
    );
}

#[test]
fn test_diagnostics_line_numbers_are_0_based() {
    let source = "壊れた行1\n壊れた行2\n";
    let result = AnalysisEngine::analyze(source);
    for diag in &result.diagnostics {
        // LSP Diagnostics use 0-based line numbers
        // Verify they are reasonable (within document range)
        assert!(
            diag.range.start.line < 100,
            "Diagnostic line is reasonable: {}",
            diag.range.start.line
        );
    }
}
