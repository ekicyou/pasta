//! パーサークラッシュ回復テスト (Task 8.4)
//!
//! パニック発生時のサーバー継続動作検証

use pasta_lsp::analysis::AnalysisEngine;
use pasta_lsp::document::DocumentManager;

#[test]
fn test_catch_unwind_protects_from_panic() {
    // Simulate the crash protection pattern used in PastaLangServer::analyze_and_publish
    let source = "＊挨拶\n  Alice：こんにちは\n";

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        AnalysisEngine::analyze(source)
    }));

    assert!(result.is_ok(), "正常なソースではパニックしない");
    let analysis = result.unwrap();
    assert!(!analysis.tokens.is_empty());
}

#[test]
fn test_catch_unwind_with_malformed_input() {
    // Even with unusual input, analyze should not panic
    let large_a = "a".repeat(10000);
    let large_star = "＊".repeat(100);
    let sources: Vec<&str> = vec![
        "",
        "\0",
        "\n\n\n",
        &large_a,
        &large_star,
    ];

    for source in &sources {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            AnalysisEngine::analyze(source)
        }));
        assert!(
            result.is_ok(),
            "入力 {:?}... でパニックしない",
            &source[..source.len().min(20)]
        );
    }
}

#[test]
fn test_other_documents_unaffected_after_error() {
    let mut dm = DocumentManager::new();
    let uri1 = "file:///good.pasta";
    let uri2 = "file:///bad.pasta";

    let good_source = "＊挨拶\n  Alice：OK\n";
    let bad_source = "壊れたソース\n";

    dm.open(uri1.to_string(), good_source.to_string(), 1);
    dm.open(uri2.to_string(), bad_source.to_string(), 1);

    // Analyze good document
    let good_result = AnalysisEngine::analyze(good_source);
    dm.get_mut(uri1).unwrap().analysis = Some(good_result);

    // Analyze bad document (should not affect good)
    let _bad_result = AnalysisEngine::analyze(bad_source);

    // Good document's analysis should be unaffected
    let cached = dm.get(uri1).unwrap().analysis.as_ref().unwrap();
    assert!(!cached.tokens.is_empty(), "良好なドキュメントの解析結果は影響なし");
}

#[test]
fn test_server_continues_after_error_document() {
    let mut dm = DocumentManager::new();

    // 1. Open and analyze a bad document
    let bad_source = "壊れた構文\n";
    dm.open("file:///bad.pasta".to_string(), bad_source.to_string(), 1);
    let _ = AnalysisEngine::analyze(bad_source);

    // 2. Open and analyze a good document AFTER the error
    let good_source = "＊挨拶\n  Alice：OK\n";
    dm.open("file:///good.pasta".to_string(), good_source.to_string(), 1);
    let good_result = AnalysisEngine::analyze(good_source);

    assert!(
        !good_result.tokens.is_empty(),
        "エラードキュメント後も正常にトークン生成"
    );
    assert!(
        good_result.diagnostics.is_empty(),
        "エラードキュメント後も正常なソースはエラーなし"
    );
}
