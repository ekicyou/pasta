//! 動的参照（`＠＄x`・`＠＄＊x`・`＠＄０`・`＠＄f（…）`）のセマンティックトークンテスト
//!
//! アクション行・代入の右辺・式の中の動的参照が WORD トークンになり、
//! 全角半角混在のマーカー列（`＠$x`・`@＄＊x`）でも位置が合い、
//! 診断が出ないことを確認する（dynamic-word-reference 6.3, 7.10）。

use pasta_lsp::analysis::{AnalysisEngine, AnalysisResult, token_type};

/// Decode delta-encoded tokens into (line, start_char, length, token_type, modifiers).
fn decode_tokens(result: &AnalysisResult) -> Vec<(u32, u32, u32, u32, u32)> {
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
        decoded.push((
            line,
            char_offset,
            t.length,
            t.token_type,
            t.token_modifiers_bitset,
        ));
    }
    decoded
}

/// 行 1（0 始まり）の、暗黙の SCENE を除くトークン。診断なしも併せて確認する。
fn line1_tokens(source: &str) -> Vec<(u32, u32, u32, u32, u32)> {
    let result = AnalysisEngine::analyze(source);
    assert!(
        result.diagnostics.is_empty(),
        "診断なし: {:?}",
        result.diagnostics
    );
    decode_tokens(&result)
        .into_iter()
        .filter(|t| t.0 == 1 && t.3 != token_type::SCENE)
        .collect()
}

/// WORD トークンだけを (start_char, length) で取り出す。
fn words(tokens: &[(u32, u32, u32, u32, u32)]) -> Vec<(u32, u32)> {
    tokens
        .iter()
        .filter(|t| t.3 == token_type::WORD)
        .map(|t| (t.1, t.2))
        .collect()
}

// ============================================================================
// アクション行
// ============================================================================

#[test]
fn test_action_dynamic_word_ref_is_word() {
    // 　さくら：こんにちは＠＄x — ＠ は col10、＠＄x で len3
    let vt = line1_tokens("＊挨拶\n　さくら：こんにちは＠＄x\n");
    assert_eq!(words(&vt), vec![(10, 3)], "{:?}", vt);
}

#[test]
fn test_action_dynamic_word_ref_marker_mixes() {
    // 半角 $ ・半角 @ と全角 ＄＊ ・引数番号の混在でも同じ位置・長さ
    for (src, expected) in [
        ("＊挨拶\n　さくら：こんにちは＠$x\n", (10, 3)),
        ("＊挨拶\n　さくら：こんにちは@＄＊x\n", (10, 4)),
        ("＊挨拶\n　さくら：こんにちは@$*x\n", (10, 4)),
        ("＊挨拶\n　さくら：こんにちは＠＄０\n", (10, 3)),
    ] {
        let vt = line1_tokens(src);
        assert_eq!(words(&vt), vec![expected], "{src}: {:?}", vt);
    }
}

#[test]
fn test_action_dynamic_fn_call_is_word() {
    // 静的 FnCall と同じく呼び出し全体（＠＄f（１））を 1 WORD トークンにする
    let vt = line1_tokens("＊挨拶\n　さくら：＠＄f（１）\n");
    assert_eq!(words(&vt), vec![(5, 6)], "{:?}", vt);
}

// ============================================================================
// 代入の右辺
// ============================================================================

#[test]
fn test_var_set_rhs_dynamic_word_ref() {
    // 　＄y＝＠＄x — ＠ は col4、＠＄x で len3（静的 ＄y＝＠x と同じ分類）
    for (src, expected) in [
        ("＊挨拶\n　＄y＝＠＄x\n", (4, 3)),
        ("＊挨拶\n　＄y＝＠$x\n", (4, 3)),
        ("＊挨拶\n　＄y＝@＄＊x\n", (4, 4)),
        ("＊挨拶\n　＄y＝@$*x\n", (4, 4)),
        ("＊挨拶\n　＄y＝＠＄０\n", (4, 3)),
    ] {
        let vt = line1_tokens(src);
        assert_eq!(words(&vt), vec![expected], "{src}: {:?}", vt);
    }
}

#[test]
fn test_var_set_rhs_dynamic_word_ref_utf16_position() {
    // サロゲートペアの変数名（𠮷 = UTF-16 で 2 単位）の後ろでも位置が合う
    let vt = line1_tokens("＊挨拶\n　＄𠮷＝@＄＊x\n");
    assert_eq!(words(&vt), vec![(5, 4)], "{:?}", vt);
}

// ============================================================================
// 式の中
// ============================================================================

#[test]
fn test_expr_dynamic_fn_call_with_args() {
    // ＄z＝＠＄f（１、２） — WORD + 括弧 OPERATOR + 引数 NUMBER（静的 ＠乱数（１、２）と同形）
    let vt = line1_tokens("＊挨拶\n　＄z＝＠＄f（１、２）\n");
    assert_eq!(
        &vt[3..],
        &[
            (1, 4, 3, token_type::WORD, 0),
            (1, 7, 1, token_type::OPERATOR, 0),
            (1, 8, 1, token_type::NUMBER, 0),
            (1, 10, 1, token_type::NUMBER, 0),
            (1, 11, 1, token_type::OPERATOR, 0),
        ],
        "{:?}",
        vt
    );
}

#[test]
fn test_expr_dynamic_fn_call_in_binary_mixed_markers() {
    // ＄z＝１＋＠$f（２） — 二項演算の右項にある動的呼び出しと引数
    let vt = line1_tokens("＊挨拶\n　＄z＝１＋＠$f（２）\n");
    assert_eq!(
        &vt[3..],
        &[
            (1, 4, 1, token_type::NUMBER, 0),
            (1, 5, 1, token_type::OPERATOR, 0),
            (1, 6, 3, token_type::WORD, 0),
            (1, 9, 1, token_type::OPERATOR, 0),
            (1, 10, 1, token_type::NUMBER, 0),
            (1, 11, 1, token_type::OPERATOR, 0),
        ],
        "{:?}",
        vt
    );
}

#[test]
fn test_expr_dynamic_fn_call_as_static_fn_arg() {
    // ＄z＝＠乱数（@＄＊g（）） — 静的呼び出しの引数の中の動的呼び出し
    let vt = line1_tokens("＊挨拶\n　＄z＝＠乱数（@＄＊g（））\n");
    assert_eq!(words(&vt), vec![(4, 3), (8, 4)], "{:?}", vt);
}
