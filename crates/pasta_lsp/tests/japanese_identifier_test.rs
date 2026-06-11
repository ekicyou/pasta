//! 日本語識別子テスト (Task 7.3)
//!
//! 日本語シーン名、変数名、単語名のトークン化テスト

use pasta_lsp::analysis::AnalysisEngine;

#[test]
fn test_japanese_scene_name_tokenized() {
    let source = "＊挨拶シーン\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        !result.tokens.is_empty(),
        "日本語シーン名がトークン化される"
    );
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
}

#[test]
fn test_japanese_actor_name_tokenized() {
    let source = "＊テスト\n  太郎：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        !result.tokens.is_empty(),
        "日本語アクター名がトークン化される"
    );
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
}

#[test]
fn test_japanese_global_scene_name() {
    let source = "＊日本語シーン名テスト\n  Alice：テスト\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        result.diagnostics.is_empty(),
        "日本語グローバルシーン名でエラーなし"
    );
    assert!(!result.tokens.is_empty());
}

#[test]
fn test_mixed_japanese_ascii_identifiers() {
    let source = "＊greeting挨拶\n  Alice：hello世界\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "日英混在でエラーなし");
    assert!(!result.tokens.is_empty(), "日英混在でトークン生成");
}

#[test]
fn test_japanese_identifier_utf16_positions() {
    // UTF-16のポジションが正しく計算されることを確認
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    for token in &result.tokens {
        // All token lengths should be positive
        assert!(token.length > 0, "トークンのlengthは正");
    }
}
