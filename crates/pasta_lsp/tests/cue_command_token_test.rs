//! キューコマンド行のセマンティックトークン生成テスト (lsp-spec-conformance)
//!
//! TDD: RED → GREEN → REFACTOR

use pasta_lsp::analysis::{AnalysisEngine, token_type};

// ============================================================================
// Helper
// ============================================================================

fn decode_tokens(result: &pasta_lsp::analysis::AnalysisResult) -> Vec<(u32, u32, u32, u32)> {
    let mut decoded = Vec::new();
    let mut line = 0u32;
    let mut char_offset = 0u32;
    for t in &result.tokens {
        if t.delta_line > 0 {
            line += t.delta_line;
            char_offset = t.delta_start;
        } else {
            char_offset += t.delta_start;
        }
        decoded.push((line, char_offset, t.length, t.token_type));
    }
    decoded
}

/// Filter tokens on a given line, excluding SCENE tokens from implicit local scene scope
fn cue_tokens_on_line(decoded: &[(u32, u32, u32, u32)], line: u32) -> Vec<(u32, u32, u32, u32)> {
    decoded
        .iter()
        .filter(|t| t.0 == line && t.3 != token_type::SCENE)
        .copied()
        .collect()
}

// ============================================================================
// 4.1: 基本 4 形式のトークン生成テスト (R4.1)
// ============================================================================

#[test]
fn test_cue_simple_command() {
    let source = "＊シーン\n  !clear\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(ct.len() >= 2, "!clear: 2 tokens, got: {:?}", ct);
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
}

#[test]
fn test_cue_command_with_scope() {
    let source = "＊シーン\n  !emote@笑顔\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(ct.len() >= 3, "!emote@笑顔: 3 tokens, got: {:?}", ct);
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
    assert_eq!(ct[2].3, token_type::WORD);
}

#[test]
fn test_cue_command_with_args() {
    let source = "＊シーン\n  !choice(yes, no)\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(ct.len() >= 6, "!choice(yes, no): 6 tokens, got: {:?}", ct);
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
    assert_eq!(ct[2].3, token_type::OPERATOR);
    assert_eq!(ct[3].3, token_type::CUE_COMMAND); // yes
    assert_eq!(ct[4].3, token_type::CUE_COMMAND); // no
    assert_eq!(ct[5].3, token_type::OPERATOR);
}

#[test]
fn test_cue_command_full() {
    let source = "＊シーン\n  !emote@さくら:笑顔(normal)\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(
        ct.len() >= 6,
        "!emote@さくら:笑顔(normal): 6 tokens, got: {:?}",
        ct
    );
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
    assert_eq!(ct[2].3, token_type::WORD);
    assert_eq!(ct[3].3, token_type::OPERATOR);
    assert_eq!(ct[4].3, token_type::CUE_COMMAND); // normal
    assert_eq!(ct[5].3, token_type::OPERATOR);
}

// ============================================================================
// 4.1 (R4.2): 全角/半角マーカー同値テスト
// ============================================================================

#[test]
fn test_cue_fullwidth_marker() {
    let source_half = "＊シーン\n  !clear\n";
    let source_full = "＊シーン\n  ！clear\n";

    let result_half = AnalysisEngine::analyze(source_half);
    let result_full = AnalysisEngine::analyze(source_full);

    let dh = decode_tokens(&result_half);
    let df = decode_tokens(&result_full);
    let th = cue_tokens_on_line(&dh, 1);
    let tf = cue_tokens_on_line(&df, 1);

    assert_eq!(th.len(), tf.len(), "half={:?}, full={:?}", th, tf);
    for (h, f) in th.iter().zip(tf.iter()) {
        assert_eq!(h.3, f.3, "half={:?}, full={:?}", h, f);
    }
}

// ============================================================================
// 4.2: 引数タイプ別のトークン生成テスト (R1.4, R1.5, R1.6)
// ============================================================================

#[test]
fn test_cue_string_literal_arg() {
    let source = "＊シーン\n  !msg(「こんにちは」)\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(ct.len() >= 5, "5 tokens, got: {:?}", ct);
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
    assert_eq!(ct[2].3, token_type::OPERATOR);
    assert_eq!(ct[3].3, token_type::TALK, "string → TALK: {:?}", ct[3]);
    assert_eq!(ct[4].3, token_type::OPERATOR);
}

#[test]
fn test_cue_number_arg() {
    let source = "＊シーン\n  !yield(10.0)\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(ct.len() >= 5, "5 tokens, got: {:?}", ct);
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
    assert_eq!(ct[2].3, token_type::OPERATOR);
    assert_eq!(ct[3].3, token_type::NUMBER, "number → NUMBER: {:?}", ct[3]);
    assert_eq!(ct[4].3, token_type::OPERATOR);
}

#[test]
fn test_cue_at_ref_arg() {
    let source = "＊シーン\n  !bind(@name)\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);
    let ct = cue_tokens_on_line(&decoded, 1);

    assert!(ct.len() >= 5, "5 tokens, got: {:?}", ct);
    assert_eq!(ct[0].3, token_type::CUE_MARKER);
    assert_eq!(ct[1].3, token_type::CUE_COMMAND);
    assert_eq!(ct[2].3, token_type::OPERATOR);
    assert_eq!(ct[3].3, token_type::WORD, "@ref → WORD: {:?}", ct[3]);
    assert_eq!(ct[4].3, token_type::OPERATOR);
}

// ============================================================================
// 4.3: 混在ドキュメントと Diagnostics テスト (R4.3, R4.4)
// ============================================================================

#[test]
fn test_cue_mixed_scene() {
    let source = "＊シーン\n  !emote@笑顔\n  さくら：こんにちは\n  !clear\n";
    let result = AnalysisEngine::analyze(source);
    let decoded = decode_tokens(&result);

    let l1 = cue_tokens_on_line(&decoded, 1);
    assert!(l1.len() >= 3, "line 1: 3 tokens, got: {:?}", l1);
    assert_eq!(l1[0].3, token_type::CUE_MARKER);
    assert_eq!(l1[1].3, token_type::CUE_COMMAND);

    let l2 = cue_tokens_on_line(&decoded, 2);
    assert!(!l2.is_empty(), "line 2: action tokens, got: {:?}", l2);

    let l3 = cue_tokens_on_line(&decoded, 3);
    assert!(l3.len() >= 2, "line 3: 2 tokens, got: {:?}", l3);
    assert_eq!(l3[0].3, token_type::CUE_MARKER);
    assert_eq!(l3[1].3, token_type::CUE_COMMAND);
}

#[test]
fn test_cue_parse_error_diagnostics() {
    let source = "＊シーン\n  !cmd(unclosed\n";
    let result = AnalysisEngine::analyze(source);
    assert!(
        !result.diagnostics.is_empty(),
        "diagnostics: {:?}",
        result.diagnostics
    );
}
