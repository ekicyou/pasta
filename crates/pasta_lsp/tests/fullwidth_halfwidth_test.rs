//! 全角/半角マーカー同等認識テスト (Task 7.2)
//!
//! 全角マーカーと半角マーカーが同等のトークンを生成することを検証

use pasta_lsp::analysis::{AnalysisEngine, token_type};

/// 特定のトークンタイプが含まれるか確認するヘルパー
fn has_token_type(source: &str, expected_type: u32) -> bool {
    let result = AnalysisEngine::analyze(source);
    result.tokens.iter().any(|t| t.token_type == expected_type)
}

/// 2つのソースから同じトークンタイプが生成されることを確認
fn assert_equivalent_tokens(
    fullwidth_source: &str,
    halfwidth_source: &str,
    token_type: u32,
    label: &str,
) {
    let has_fw = has_token_type(fullwidth_source, token_type);
    let has_hw = has_token_type(halfwidth_source, token_type);
    assert!(has_fw, "{}: 全角マーカーでトークン生成失敗", label);
    assert!(has_hw, "{}: 半角マーカーでトークン生成失敗", label);
}

// ============================================================================
// Global Scene: ＊ vs *
// ============================================================================

#[test]
fn test_global_scene_fullwidth_halfwidth_equivalence() {
    let fw = "＊挨拶\n  Alice：OK\n";
    let hw = "*greeting\n  Alice：OK\n";
    assert_equivalent_tokens(fw, hw, token_type::NAMESPACE, "グローバルシーン");
}

// ============================================================================
// Local Scene: ・ vs -
// ============================================================================

#[test]
fn test_local_scene_fullwidth_halfwidth_equivalence() {
    // Anonymous local scene (local_start_scene_scope) is required before named scenes.
    // Use sources with both an anonymous and a named local scene.
    let fw = "＊挨拶\n  Alice：こんにちは\n  ・ランダム\n    Bob：OK\n";
    let hw = "*greeting\n  Alice：hello\n  -random\n    Bob：OK\n";
    assert_equivalent_tokens(fw, hw, token_type::SCENE, "ローカルシーン");
}

// ============================================================================
// File Attribute: ＆ vs &
// ============================================================================

#[test]
fn test_file_attr_fullwidth_halfwidth_equivalence() {
    let fw = "＆author：test\n＊挨拶\n  Alice：OK\n";
    let hw = "&author：test\n*greeting\n  Alice：OK\n";
    assert_equivalent_tokens(fw, hw, token_type::DECORATOR, "属性");
}

// ============================================================================
// Action Line Actor Name: fullwidth colon vs halfwidth colon
// ============================================================================

#[test]
fn test_action_line_fullwidth_halfwidth_colon() {
    let fw = "＊挨拶\n  Alice：OK\n";
    let hw = "*greeting\n  Alice:OK\n";
    assert_equivalent_tokens(fw, hw, token_type::ACTOR_NAME, "アクター名");
}

// ============================================================================
// Multiple markers mixed
// ============================================================================

#[test]
fn test_mixed_markers_produce_tokens() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(!result.tokens.is_empty(), "混合マーカーでトークン生成");
    assert!(result.diagnostics.is_empty(), "構文エラーなし");
}
