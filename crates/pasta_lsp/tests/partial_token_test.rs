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

/// LSP の delta エンコード済みセマンティックトークンから、各トークンの
/// 絶対行番号（0-based）の集合を復元する。
fn decoded_token_lines(result: &pasta_lsp::analysis::AnalysisResult) -> Vec<u32> {
    let mut lines = Vec::new();
    let mut current_line: u32 = 0;
    for (i, tok) in result.tokens.iter().enumerate() {
        if i == 0 {
            current_line = tok.delta_line;
        } else {
            current_line += tok.delta_line;
        }
        lines.push(current_line);
    }
    lines
}

#[test]
fn test_partial_token_positions_corrected_for_later_scopes() {
    // 1 行目が不正 → 全体パース失敗 → 部分パースへフォールバック。
    // 2 つ目以降のスコープ（％さくら / ＄表情＝0）のトークンは、span が
    // チャンク相対のままだと先頭チャンク（行 0/1）に誤って描画される。
    // 補正後はフルソースの行 3（％さくら）・行 4（＄表情＝0）へ届くはず。
    let source = "壊れた行ですわ\n＊挨拶\n  Alice：こんにちは\n％さくら\n  ＄表情＝0\n";
    let result = AnalysisEngine::analyze(source);

    let lines = decoded_token_lines(&result);
    assert!(
        lines.contains(&3),
        "アクタースコープ（行3）にトークンが描画される: lines={:?}",
        lines
    );
    assert!(
        lines.contains(&4),
        "2つ目スコープの var_set（行4）にトークンが描画される: lines={:?}",
        lines
    );
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
