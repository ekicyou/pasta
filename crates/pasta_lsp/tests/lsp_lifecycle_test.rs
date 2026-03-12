//! LSPライフサイクル統合テスト (Task 8.1)
//!
//! DocumentManager + AnalysisEngine の連携フロー検証:
//! open → analyze → get tokens → close

use pasta_lsp::analysis::AnalysisEngine;
use pasta_lsp::document::DocumentManager;

const TEST_URI: &str = "file:///test.pasta";

#[test]
fn test_lifecycle_open_analyze_close() {
    let mut dm = DocumentManager::new();
    let source = "＊挨拶\n  Alice：こんにちは\n";

    // 1. Open document
    dm.open(TEST_URI.to_string(), source.to_string(), 1);
    let doc = dm.get(TEST_URI).unwrap();
    assert_eq!(doc.version, 1);
    assert!(doc.analysis.is_none(), "解析前はキャッシュなし");

    // 2. Analyze
    let result = AnalysisEngine::analyze(&doc.text);
    assert!(!result.tokens.is_empty(), "トークンが生成される");
    assert!(result.diagnostics.is_empty(), "エラーなし");

    // 3. Cache analysis result
    dm.get_mut(TEST_URI).unwrap().analysis = Some(result.clone());

    // 4. Verify cached tokens
    let cached = dm.get(TEST_URI).unwrap().analysis.as_ref().unwrap();
    assert_eq!(cached.tokens.len(), result.tokens.len());

    // 5. Close
    dm.close(TEST_URI);
    assert!(dm.get(TEST_URI).is_none(), "Close後はドキュメントなし");
}

#[test]
fn test_lifecycle_capabilities_legend() {
    // Verify the legend contains all expected token types
    let legend = pasta_lsp::analysis::semantic_tokens_legend();
    assert_eq!(legend.token_types.len(), 17, "17トークンタイプ");
    assert_eq!(legend.token_modifiers.len(), 3, "3モディファイア");
}

#[test]
fn test_lifecycle_open_then_semantic_tokens_full() {
    let mut dm = DocumentManager::new();
    let source = "＊挨拶\n  Alice：こんにちは\n";

    dm.open(TEST_URI.to_string(), source.to_string(), 1);

    // Simulate semanticTokens/full: analyze and cache
    let text = dm.get(TEST_URI).unwrap().text.clone();
    let result = AnalysisEngine::analyze(&text);
    dm.get_mut(TEST_URI).unwrap().analysis = Some(result);

    // Retrieve cached tokens
    let tokens = dm
        .get(TEST_URI)
        .and_then(|d| d.analysis.as_ref())
        .map(|a| a.tokens.clone())
        .unwrap_or_default();

    assert!(!tokens.is_empty(), "キャッシュ済みトークンが取得できる");
}

#[test]
fn test_lifecycle_unregistered_document_returns_empty() {
    let dm = DocumentManager::new();
    let tokens = dm
        .get("file:///nonexistent.pasta")
        .and_then(|d| d.analysis.as_ref())
        .map(|a| a.tokens.clone())
        .unwrap_or_default();

    assert!(tokens.is_empty(), "未登録ドキュメントは空トークン");
}
