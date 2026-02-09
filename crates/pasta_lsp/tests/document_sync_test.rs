//! ドキュメント同期統合テスト (Task 8.2)
//!
//! didChange（増分更新・全文置換）→ 再解析 → トークン更新フロー検証

use pasta_lsp::analysis::AnalysisEngine;
use pasta_lsp::document::{ContentChange, DocumentManager, TextRange};

const TEST_URI: &str = "file:///sync_test.pasta";

#[test]
fn test_full_content_change_triggers_reanalysis() {
    let mut dm = DocumentManager::new();
    let source1 = "＊挨拶\n  Alice：こんにちは\n";
    let source2 = "＊別れ\n  Bob：さようなら\n";

    // Open
    dm.open(TEST_URI.to_string(), source1.to_string(), 1);
    let result1 = AnalysisEngine::analyze(source1);
    dm.get_mut(TEST_URI).unwrap().analysis = Some(result1.clone());

    // Change (full replacement)
    dm.change(
        TEST_URI,
        &[ContentChange {
            range: None,
            text: source2.to_string(),
        }],
        2,
    );

    // Analysis cache invalidated
    let doc = dm.get(TEST_URI).unwrap();
    assert!(doc.analysis.is_none(), "変更後キャッシュ無効化");
    assert_eq!(doc.version, 2, "バージョン更新");
    assert_eq!(doc.text, source2, "テキスト更新");

    // Re-analyze
    let result2 = AnalysisEngine::analyze(&doc.text);
    assert!(!result2.tokens.is_empty(), "再解析でトークン生成");
}

#[test]
fn test_incremental_change_updates_text() {
    let mut dm = DocumentManager::new();
    let source = "hello\nworld\n";

    dm.open(TEST_URI.to_string(), source.to_string(), 1);

    // Replace "hello" with "HELLO" (incremental)
    dm.change(
        TEST_URI,
        &[ContentChange {
            range: Some(TextRange {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 5,
            }),
            text: "HELLO".to_string(),
        }],
        2,
    );

    let doc = dm.get(TEST_URI).unwrap();
    assert_eq!(doc.text, "HELLO\nworld\n");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_version_tracking() {
    let mut dm = DocumentManager::new();

    dm.open(TEST_URI.to_string(), "v1".to_string(), 1);
    assert_eq!(dm.get(TEST_URI).unwrap().version, 1);

    dm.change(
        TEST_URI,
        &[ContentChange {
            range: None,
            text: "v2".to_string(),
        }],
        2,
    );
    assert_eq!(dm.get(TEST_URI).unwrap().version, 2);

    dm.change(
        TEST_URI,
        &[ContentChange {
            range: None,
            text: "v3".to_string(),
        }],
        5,
    );
    assert_eq!(dm.get(TEST_URI).unwrap().version, 5);
}

#[test]
fn test_multiple_documents_independent() {
    let mut dm = DocumentManager::new();
    let uri1 = "file:///doc1.pasta";
    let uri2 = "file:///doc2.pasta";

    dm.open(uri1.to_string(), "doc1 content".to_string(), 1);
    dm.open(uri2.to_string(), "doc2 content".to_string(), 1);

    dm.change(
        uri1,
        &[ContentChange {
            range: None,
            text: "doc1 changed".to_string(),
        }],
        2,
    );

    assert_eq!(dm.get(uri1).unwrap().text, "doc1 changed");
    assert_eq!(dm.get(uri2).unwrap().text, "doc2 content");
}
