//! Phase A: analysis.rs のインラインテストを外部化
//!
//! get_line_text, line_byte_offset: pub に昇格

use pasta_lsp::analysis::{
    AnalysisEngine, RawToken, encode_tokens, get_line_text, line_byte_offset, token_type,
    utf8_offset_to_utf16,
};

#[test]
fn test_utf8_offset_to_utf16_ascii() {
    let text = "hello";
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 3), 3);
    assert_eq!(utf8_offset_to_utf16(text, 5), 5);
}

#[test]
fn test_utf8_offset_to_utf16_japanese() {
    let text = "＊挨拶";
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 3), 1);
    assert_eq!(utf8_offset_to_utf16(text, 6), 2);
    assert_eq!(utf8_offset_to_utf16(text, 9), 3);
}

#[test]
fn test_utf8_offset_to_utf16_emoji() {
    let text = "a😀b";
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 1), 1);
    assert_eq!(utf8_offset_to_utf16(text, 5), 3);
    assert_eq!(utf8_offset_to_utf16(text, 6), 4);
}

#[test]
fn test_encode_tokens_empty() {
    let mut raw: Vec<RawToken> = vec![];
    assert!(encode_tokens(&mut raw).is_empty());
}

#[test]
fn test_encode_tokens_single() {
    let mut raw = vec![RawToken {
        line: 0,
        start_char: 0,
        length: 3,
        token_type: token_type::COMMENT,
        modifiers: 0,
    }];
    let encoded = encode_tokens(&mut raw);
    assert_eq!(encoded.len(), 1);
    assert_eq!(encoded[0].delta_line, 0);
    assert_eq!(encoded[0].delta_start, 0);
    assert_eq!(encoded[0].length, 3);
}

#[test]
fn test_encode_tokens_delta() {
    let mut raw = vec![
        RawToken {
            line: 0,
            start_char: 0,
            length: 3,
            token_type: 0,
            modifiers: 0,
        },
        RawToken {
            line: 0,
            start_char: 5,
            length: 2,
            token_type: 1,
            modifiers: 0,
        },
        RawToken {
            line: 2,
            start_char: 1,
            length: 4,
            token_type: 2,
            modifiers: 0,
        },
    ];
    let encoded = encode_tokens(&mut raw);
    assert_eq!(encoded.len(), 3);
    assert_eq!(encoded[0].delta_line, 0);
    assert_eq!(encoded[0].delta_start, 0);
    assert_eq!(encoded[1].delta_line, 0);
    assert_eq!(encoded[1].delta_start, 5);
    assert_eq!(encoded[2].delta_line, 2);
    assert_eq!(encoded[2].delta_start, 1);
}

#[test]
fn test_analyze_simple_scene() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(!result.tokens.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_analyze_crlf_does_not_panic() {
    // CRLF line endings must not cause panics (e.g., byte offset miscalculation)
    let source = "＃コメント\r\n＊挨拶\r\n  Alice：こんにちは\r\n";
    let result = AnalysisEngine::analyze(source);
    // Should produce tokens without panicking
    assert!(
        !result.tokens.is_empty(),
        "CRLF input should produce tokens"
    );
}

#[test]
fn test_analyze_crlf_actor_scope() {
    let source = "＃アクター辞書\r\n％さくら\r\n　＠通常：\\s[0]\r\n";
    let result = AnalysisEngine::analyze(source);
    // Must not panic; may produce tokens or diagnostics
    let _total = result.tokens.len() + result.diagnostics.len();
}

#[test]
fn test_line_byte_offset_lf() {
    let source = "abc\ndef\nghi";
    assert_eq!(line_byte_offset(source, 1), 0);
    assert_eq!(line_byte_offset(source, 2), 4);
    assert_eq!(line_byte_offset(source, 3), 8);
}

#[test]
fn test_line_byte_offset_crlf() {
    let source = "abc\r\ndef\r\nghi";
    assert_eq!(line_byte_offset(source, 1), 0);
    assert_eq!(line_byte_offset(source, 2), 5); // "abc\r\n" = 5 bytes
    assert_eq!(line_byte_offset(source, 3), 10); // "abc\r\ndef\r\n" = 10 bytes
}

#[test]
fn test_get_line_text_strips_cr() {
    let source = "hello\r\nworld\r\n";
    assert_eq!(get_line_text(source, 1), "hello");
    assert_eq!(get_line_text(source, 2), "world");
}

#[test]
fn test_comment_line_fullwidth_emits_comment_token() {
    // scan_comment_lines: ＃ 行は AST に現れないため行スキャンで COMMENT トークンを生成する
    let source = "＃コメント\n＊挨拶\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty());

    // 先頭トークン（行0・列0）が COMMENT で、行全長（5 UTF-16 ユニット）をカバーする
    let first = &result.tokens[0];
    assert_eq!(first.delta_line, 0);
    assert_eq!(first.delta_start, 0);
    assert_eq!(first.token_type, token_type::COMMENT);
    assert_eq!(first.length, 5, "＃コメント = 5 UTF-16 units");
}

#[test]
fn test_comment_line_halfwidth_with_leading_whitespace() {
    // 半角 # ＋ 先行空白: COMMENT トークンは列0から行全長で生成される
    let source = "  # comment\n＊挨拶\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty());

    let first = &result.tokens[0];
    assert_eq!(first.delta_line, 0);
    assert_eq!(first.delta_start, 0);
    assert_eq!(first.token_type, token_type::COMMENT);
    assert_eq!(first.length, 11, "行全体（先行空白込み）= 11 units");
}

#[test]
fn test_analyze_without_trailing_newline_matches_with_newline() {
    // analyze は末尾改行なし入力（VSCode 等）に内部で改行を補完する（Cow::Owned 経路）。
    // 末尾改行あり入力と同一のトークン列・診断になることを固定する。
    let without_nl = "＊挨拶\n　Alice：こんにちは";
    let with_nl = "＊挨拶\n　Alice：こんにちは\n";
    let r1 = AnalysisEngine::analyze(without_nl);
    let r2 = AnalysisEngine::analyze(with_nl);

    assert!(!r1.tokens.is_empty(), "末尾改行なしでもトークン生成");
    assert_eq!(
        r1.tokens, r2.tokens,
        "トークン列が末尾改行の有無に依存しない"
    );
    assert_eq!(r1.diagnostics.len(), r2.diagnostics.len());
}
