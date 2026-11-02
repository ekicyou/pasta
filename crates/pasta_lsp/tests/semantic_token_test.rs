//! セマンティックトークン識別テスト (Task 7.1)
//!
//! 14トークンタイプ個別識別テスト、AST→トークン変換の正確性テスト

use pasta_lsp::analysis::{AnalysisEngine, token_type};

// ============================================================================
// Global Scene (NAMESPACE)
// ============================================================================

#[test]
fn test_token_global_scene_marker() {
    // ＊ マーカーはNAMESPACEトークンタイプ
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(!result.tokens.is_empty(), "トークンが生成される");
    // First token should be the global scene marker (NAMESPACE = 1)
    assert_eq!(result.tokens[0].token_type, token_type::NAMESPACE);
}

// ============================================================================
// Local Scene (scene)
// ============================================================================

#[test]
fn test_token_local_scene_marker() {
    // Anonymous local scene (name=None) should NOT emit a SCENE token.
    // Named local scene (with ・ or - marker) should emit one.
    let source = "＊挨拶\n  Alice：こんにちは\n  ・ランダム\n    Bob：OK\n";
    let result = AnalysisEngine::analyze(source);
    // Should contain scene token type (2) for the named local scene
    let has_scene = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::SCENE);
    assert!(has_scene, "名前付きローカルシーンのトークンが含まれる");
}

// ============================================================================
// Action Line - Actor Name (actorName) and Operator (OPERATOR)
// ============================================================================

#[test]
fn test_token_actor_name_and_colon() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);

    // Should contain actorName token (8)
    let has_actor_name = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::ACTOR_NAME);
    assert!(has_actor_name, "アクター名トークンが含まれる");

    // Should contain operator token (13) for colon
    let has_operator = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::OPERATOR);
    assert!(has_operator, "コロン区切りのオペレータートークンが含まれる");
}

// ============================================================================
// Talk Text (TALK)
// ============================================================================

#[test]
fn test_token_talk_text() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);

    let has_talk = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::TALK);
    assert!(has_talk, "Talk文字列トークンが含まれる");
}

// ============================================================================
// File Attribute (DECORATOR)
// ============================================================================

#[test]
fn test_token_file_attr() {
    let source = "＆author：Test\n＊挨拶\n  Alice：OK\n";
    let result = AnalysisEngine::analyze(source);

    let has_decorator = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::DECORATOR);
    assert!(has_decorator, "属性マーカーのDECORATORトークンが含まれる");
}

// ============================================================================
// Multiple token types in one document
// ============================================================================

#[test]
fn test_multiple_token_types_in_document() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "エラーなし");
    assert!(
        result.tokens.len() >= 3,
        "最低でもNAMESPACE + SCENE + ACTOR_NAME/STRING等が必要: got {}",
        result.tokens.len()
    );
}

// ============================================================================
// Empty / Edge Cases
// ============================================================================

#[test]
fn test_token_empty_source() {
    let result = AnalysisEngine::analyze("");
    // Empty source should produce empty tokens
    assert!(result.tokens.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_token_syntax_error_generates_diagnostics() {
    // Intentionally malformed source
    let source = "これは構文的に壊れています\n壊れた行\n";
    let result = AnalysisEngine::analyze(source);
    // Should generate diagnostics
    assert!(
        !result.diagnostics.is_empty(),
        "構文エラー時にDiagnosticsが生成される"
    );
}

#[test]
fn test_tokens_are_delta_encoded() {
    // Verify delta encoding property: delta_line >= 0
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    // All delta_lines must be >= 0 (u32 guarantees this), but
    // also check that first token has reasonable values
    if !result.tokens.is_empty() {
        // First token's delta_line should be the absolute line (since prev is 0)
        let first = &result.tokens[0];
        assert!(first.length > 0, "First token has positive length");
    }
}

// ============================================================================
// 宣言の行の名前・属性・行末のコメント（要件 4.1・4.2）
// ============================================================================

/// 誤りの無い文字の、指定した行（0 始まり）のトークン（桁・長さ・種類）
fn line_tokens(source: &str, target: u32) -> Vec<(u32, u32, u32)> {
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "誤りなし: {source:?}");
    let (mut line, mut start) = (0u32, 0u32);
    let mut out = Vec::new();
    for t in &result.tokens {
        if t.delta_line == 0 {
            start += t.delta_start;
        } else {
            line += t.delta_line;
            start = t.delta_start;
        }
        if line == target {
            out.push((start, t.length, t.token_type));
        }
    }
    out
}

use token_type::{ACTOR, COMMENT, DECORATOR, NAMESPACE, SCENE};

#[test]
fn test_decl_global_scene_trailing_comment() {
    let source = "＊OnTest ＃ メモ\n  Alice：OK\n";
    assert_eq!(
        line_tokens(source, 0),
        vec![(0, 7, NAMESPACE), (8, 4, COMMENT)]
    );
}

#[test]
fn test_decl_global_scene_attr() {
    let source = "＊挨拶 ＆朝：1\n  Alice：OK\n";
    assert_eq!(
        line_tokens(source, 0),
        vec![(0, 3, NAMESPACE), (4, 4, DECORATOR)]
    );
}

#[test]
fn test_decl_local_scene_attr_without_indent() {
    let source = "＊挨拶\n  Alice：OK\n  ・ローカル ＆朝：1\n    Bob：OK\n";
    assert_eq!(
        line_tokens(source, 2),
        vec![(2, 5, SCENE), (8, 4, DECORATOR)]
    );
}

#[test]
fn test_decl_actor_trailing_comment() {
    let source = "％さくら ＃ メモ\n";
    assert_eq!(line_tokens(source, 0), vec![(0, 4, ACTOR), (5, 4, COMMENT)]);
}

#[test]
fn test_decl_continue_scene_trailing_comment() {
    let source = "＊挨拶\n  Alice：OK\n＊ ＃ 続き\n  Bob：OK\n";
    assert_eq!(
        line_tokens(source, 2),
        vec![(0, 1, NAMESPACE), (2, 4, COMMENT)]
    );
}

/// 引用の中の `＃` はコメントにしない。属性ごとに DECORATOR を出す
#[test]
fn test_decl_attrs_with_quoted_hash_and_comment() {
    let source = "＊挨拶 ＆朝：「a＃b」 ＆夜：2 ＃ c\n  Alice：OK\n";
    assert_eq!(
        line_tokens(source, 0),
        vec![
            (0, 3, NAMESPACE),
            (4, 8, DECORATOR),
            (13, 4, DECORATOR),
            (18, 3, COMMENT)
        ]
    );
}

/// 値の頭でない `「` は引用の始まりでない（文法の attr_string）。その後の `＃` はコメント
#[test]
fn test_decl_attr_mid_value_quote_is_plain() {
    let source = "＊挨拶 ＆朝：a「x ＃ y」\n  Alice：OK\n";
    assert_eq!(
        line_tokens(source, 0),
        vec![(0, 3, NAMESPACE), (4, 6, DECORATOR), (11, 4, COMMENT)]
    );
}

/// 行末のコメントの無い宣言の行は、印（字下げを除く）から名前の終わりまで 1 つ
#[test]
fn test_decl_without_comment_unchanged() {
    let source = "＊挨拶\n  Alice：OK\n  ・ランダム\n    Bob：OK\n％さくら\n";
    assert_eq!(line_tokens(source, 0), vec![(0, 3, NAMESPACE)]);
    assert_eq!(line_tokens(source, 2), vec![(2, 5, SCENE)]);
    assert_eq!(line_tokens(source, 4), vec![(0, 4, ACTOR)]);
}
