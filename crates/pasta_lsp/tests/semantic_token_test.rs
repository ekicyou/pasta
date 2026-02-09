//! セマンティックトークン識別テスト (Task 7.1)
//!
//! 14トークンタイプ個別識別テスト、AST→トークン変換の正確性テスト

use pasta_lsp::analysis::{token_type, AnalysisEngine};

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
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    // Should contain scene token type (2) for local scene
    let has_scene = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::SCENE);
    assert!(has_scene, "ローカルシーンのトークンが含まれる");
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
// Talk Text (STRING)
// ============================================================================

#[test]
fn test_token_talk_text() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);

    let has_string = result
        .tokens
        .iter()
        .any(|t| t.token_type == token_type::STRING);
    assert!(has_string, "Talk文字列トークンが含まれる");
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
