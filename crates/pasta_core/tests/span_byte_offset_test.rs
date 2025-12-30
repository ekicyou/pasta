//! Span byte offset extension tests.
//!
//! TDD: Tests for Span structure extension with byte offsets.
//! These tests verify the Span byte offset functionality specified in
//! ast-source-span-mapping requirements.

use pasta_core::parser::{FileItem, GlobalSceneScope, Span, SpanError, parse_str};

/// Helper to get global scene scopes from PastaFile items
fn get_global_scene_scopes(file: &pasta_core::parser::PastaFile) -> Vec<&GlobalSceneScope> {
    file.items
        .iter()
        .filter_map(|item| {
            if let FileItem::GlobalSceneScope(scene) = item {
                Some(scene)
            } else {
                None
            }
        })
        .collect()
}

// ============================================================================
// Task 1.1: Span バイトオフセットフィールドのテスト
// ============================================================================

#[test]
fn test_span_new_with_6_arguments() {
    // Span::new() が 6 フィールドを正しく設定することを確認
    let span = Span::new(1, 5, 2, 10, 4, 20);

    assert_eq!(span.start_line, 1);
    assert_eq!(span.start_col, 5);
    assert_eq!(span.end_line, 2);
    assert_eq!(span.end_col, 10);
    assert_eq!(span.start_byte, 4);
    assert_eq!(span.end_byte, 20);
}

#[test]
fn test_span_default_initializes_all_to_zero() {
    // Default trait 実装で全フィールドを 0 に初期化
    let span = Span::default();

    assert_eq!(span.start_line, 0);
    assert_eq!(span.start_col, 0);
    assert_eq!(span.end_line, 0);
    assert_eq!(span.end_col, 0);
    assert_eq!(span.start_byte, 0);
    assert_eq!(span.end_byte, 0);
}

// ============================================================================
// Task 1.3: ソース参照 API のテスト
// ============================================================================

#[test]
fn test_span_extract_source_returns_correct_slice() {
    // extract_source() が正確なスライスを返すことを確認
    let source = "Hello, World!";
    let span = Span::new(1, 1, 1, 6, 0, 5);

    let result = span.extract_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_span_extract_source_utf8_multibyte() {
    // UTF-8 マルチバイト文字でオフセット正確性を確認
    let source = "こんにちは、世界！";
    // "こん" = 6 bytes (2 chars × 3 bytes each)
    let span = Span::new(1, 1, 1, 3, 0, 6);

    let result = span.extract_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "こん");
}

#[test]
fn test_span_extract_source_out_of_bounds_error() {
    // バイトオフセット範囲検証（範囲外でエラー）
    let source = "Hello";
    let span = Span::new(1, 1, 1, 20, 0, 100);

    let result = span.extract_source(source);
    assert!(result.is_err());
    match result.unwrap_err() {
        SpanError::OutOfBounds { .. } => {}
        _ => panic!("Expected OutOfBounds error"),
    }
}

#[test]
fn test_span_extract_source_invalid_utf8_boundary() {
    // UTF-8 文字境界検証（無効境界でエラー）
    let source = "こんにちは";
    // Start at byte 1 (middle of "こ") - invalid UTF-8 boundary
    let span = Span::new(1, 1, 1, 2, 1, 4);

    let result = span.extract_source(source);
    assert!(result.is_err());
    match result.unwrap_err() {
        SpanError::InvalidUtf8Boundary { .. } => {}
        _ => panic!("Expected InvalidUtf8Boundary error"),
    }
}

#[test]
fn test_span_extract_source_invalid_span() {
    // 無効な Span（start_byte == end_byte == 0）でエラー
    let source = "Hello";
    let span = Span::default();

    let result = span.extract_source(source);
    assert!(result.is_err());
    match result.unwrap_err() {
        SpanError::InvalidSpan => {}
        _ => panic!("Expected InvalidSpan error"),
    }
}

#[test]
fn test_span_is_valid() {
    // is_valid() が default Span を無効と判定することを確認
    let default_span = Span::default();
    assert!(!default_span.is_valid());

    let valid_span = Span::new(1, 1, 1, 5, 0, 5);
    assert!(valid_span.is_valid());

    // All zeros except line info still considered invalid if byte range has end == 0
    let partial_span = Span::new(1, 1, 1, 5, 0, 0);
    assert!(!partial_span.is_valid());
}

#[test]
fn test_span_byte_len() {
    // byte_len() がバイト長を取得
    let span = Span::new(1, 1, 2, 10, 5, 25);
    assert_eq!(span.byte_len(), 20);

    let empty_span = Span::default();
    assert_eq!(empty_span.byte_len(), 0);
}

// ============================================================================
// Task 6.2: パーサー統合テストの拡張
// ============================================================================

#[test]
fn test_parser_span_has_byte_offsets_ascii() {
    // ASCII スクリプトでバイトオフセット検証
    let source = "＊挨拶\n  Alice：Hello\n";
    let ast = parse_str(source, "test.pasta").unwrap();

    // ファイル全体の Span を確認
    assert!(ast.span.start_byte < ast.span.end_byte);
    assert_eq!(ast.span.start_byte, 0);
    assert!(ast.span.end_byte > 0);
}

#[test]
fn test_parser_span_byte_offset_utf8() {
    // UTF-8 マルチバイト文字でオフセット正確性を確認
    let source = "＊挨拶\n  太郎：こんにちは\n";
    let ast = parse_str(source, "test.pasta").unwrap();

    // シーンの Span を確認
    let scenes = get_global_scene_scopes(&ast);
    assert!(!scenes.is_empty());

    let scene = &scenes[0];
    assert!(scene.span.start_byte < scene.span.end_byte);
}

#[test]
fn test_parser_span_extract_source_matches_original() {
    // パース結果の Span で元ソースを抽出できることを確認
    let source = "＊挨拶\n  Alice：Hello\n";
    let ast = parse_str(source, "test.pasta").unwrap();

    // ファイル全体の抽出
    let extracted = ast.span.extract_source(source);
    assert!(extracted.is_ok());
    // 抽出結果は元ソース全体と一致
    assert_eq!(extracted.unwrap(), source);
}
