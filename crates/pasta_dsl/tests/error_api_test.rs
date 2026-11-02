//! ParseError API・parse_file・パースエラー経路の網羅テスト
//!
//! - error.rs: SyntaxError・IoError の形と Display 形式・From<io::Error>
//! - parser/mod.rs: parse_file（成功 / IO エラー）、parse_str の pest レベルエラー位置情報

use pasta_dsl::ParseError;
use pasta_dsl::parser::{FileItem, parse_file, parse_str};
use std::path::Path;

// ============================================================================
// ParseError の形と Display（parse_str が返す誤りで確かめる）
// ============================================================================

#[test]
fn test_syntax_error_shape_and_display() {
    let err = parse_str("こんにちは\n", "shape.pasta").expect_err("raw text is an error");
    match &err {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
        } => {
            assert_eq!(file, "shape.pasta");
            assert_eq!(*line, 1);
            assert_eq!(*column, 1);
            assert!(message.starts_with("Parse error in shape.pasta at 1:1: "));
        }
        other => panic!("Expected SyntaxError, got {:?}", other),
    }
    assert!(
        err.to_string()
            .starts_with("Parse error: shape.pasta:1:1: Parse error in shape.pasta at 1:1: ")
    );
}

#[test]
fn test_from_std_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let err: ParseError = io_err.into();
    match &err {
        ParseError::IoError(msg) => assert!(msg.contains("missing")),
        other => panic!("Expected IoError, got {:?}", other),
    }
    assert_eq!(err.to_string(), "IO error: missing");
}

#[test]
fn test_parse_error_is_cloneable() {
    let err = parse_str("こんにちは\n", "f.pasta").expect_err("raw text is an error");
    let cloned = err.clone();
    assert_eq!(err.to_string(), cloned.to_string());
}

// ============================================================================
// parse_str: pest レベルのエラー位置情報
// ============================================================================

#[test]
fn test_parse_str_raw_text_at_file_level_is_syntax_error() {
    // シーン外の生テキストは file ルールにマッチしない
    let result = parse_str("こんにちは\n", "raw.pasta");
    match result {
        Err(ParseError::SyntaxError {
            file,
            line,
            column,
            message,
        }) => {
            assert_eq!(file, "raw.pasta");
            assert_eq!(line, 1);
            assert_eq!(column, 1);
            assert!(message.contains("raw.pasta"), "message embeds filename");
        }
        other => panic!("Expected SyntaxError, got {:?}", other),
    }
}

#[test]
fn test_parse_str_error_reports_correct_line_and_column() {
    // 2行目の不正な var_set（右辺なしの ＄＝）は 2:4 で失敗する
    let result = parse_str("＊scene\n　＄＝\n", "pos.pasta");
    match result {
        Err(ParseError::SyntaxError { line, column, .. }) => {
            assert_eq!(line, 2);
            assert_eq!(column, 4);
        }
        other => panic!("Expected SyntaxError, got {:?}", other),
    }
}

#[test]
fn test_parse_str_bare_call_marker_in_scene_is_error() {
    // 呼び出しマーカーのみ（ターゲットなし）はエラー
    let result = parse_str("＊scene\n　＞\n", "call.pasta");
    assert!(matches!(result, Err(ParseError::SyntaxError { .. })));
}

#[test]
fn test_parse_str_bare_attr_marker_is_error() {
    // 属性マーカーのみ（id なし）はエラー
    let result = parse_str("＆\n", "attr.pasta");
    match result {
        Err(ParseError::SyntaxError { line, column, .. }) => {
            assert_eq!(line, 1);
            assert_eq!(column, 2);
        }
        other => panic!("Expected SyntaxError, got {:?}", other),
    }
}

// ============================================================================
// parse_str: 境界入力の正常系
// ============================================================================

#[test]
fn test_parse_str_comment_only_source() {
    let file = parse_str("＃comment only\n", "comment.pasta").expect("should parse");
    assert!(file.items.is_empty());
}

#[test]
fn test_parse_str_crlf_line_endings() {
    // CRLF 改行でも LF と同じ構造にパースされる
    let file = parse_str("＊scene\r\n　Alice：こんにちは\r\n", "crlf.pasta").expect("should parse");
    let scenes: Vec<_> = file
        .items
        .iter()
        .filter_map(|i| match i {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .collect();
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].name, "scene");
    assert_eq!(scenes[0].local_scenes.len(), 1);
}

#[test]
fn test_parse_str_blank_lines_only() {
    let file = parse_str("\n\n\n", "blank.pasta").expect("should parse");
    assert!(file.items.is_empty());
}

#[test]
fn test_parse_str_comments_only() {
    let file = parse_str("# コメント\n\n# もう一つ\n", "comment.pasta").expect("should parse");
    assert!(file.items.is_empty());
}

// ============================================================================
// parse_file: 成功 / IO エラー
// ============================================================================

#[test]
fn test_parse_file_success_sets_path() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("pasta_dsl_test_{}.pasta", std::process::id()));
    std::fs::write(&path, "＊挨拶\n　Alice：こんにちは\n").expect("write temp file");

    let result = parse_file(&path);
    std::fs::remove_file(&path).ok();

    let ast = result.expect("parse_file should succeed");
    assert_eq!(ast.path, path);
    let scene_count = ast
        .items
        .iter()
        .filter(|i| matches!(i, FileItem::GlobalSceneScope(_)))
        .count();
    assert_eq!(scene_count, 1);
}

#[test]
fn test_parse_file_nonexistent_path_is_io_error() {
    let result = parse_file(Path::new("Z:/nonexistent/dir/missing_pasta_dsl_test.pasta"));
    match result {
        Err(ParseError::IoError(msg)) => {
            assert!(!msg.is_empty());
        }
        other => panic!("Expected IoError, got {:?}", other),
    }
}
