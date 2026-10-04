//! 二項連鎖のセマンティックトークン生成テスト（string-concat-operator 7.6）
//!
//! 連鎖を平らにして式のテキストを左から 1 回走査し、演算子を正しい位置の
//! `OPERATOR` トークンとして出すことを確かめる。

use pasta_lsp::analysis::{AnalysisEngine, token_type};

/// Decode delta-encoded tokens into (line, start_char, length, token_type).
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

/// `　＄ｘ＝{rhs}` を解析し、右辺（列 4 以降）のトークンを返す。
fn rhs_tokens(rhs: &str) -> Vec<(u32, u32, u32)> {
    let source = format!("＊挨拶\n　＄ｘ＝{rhs}\n　Alice：や\n");
    let result = AnalysisEngine::analyze(&source);
    assert!(
        result.diagnostics.is_empty(),
        "パースエラーなし: {:?}",
        result.diagnostics
    );
    decode_tokens(&result)
        .into_iter()
        .filter(|t| t.0 == 1 && t.1 >= 4)
        .map(|t| (t.1, t.2, t.3))
        .collect()
}

/// 右辺の OPERATOR トークンの列位置。
fn operator_cols(rhs: &str) -> Vec<u32> {
    rhs_tokens(rhs)
        .into_iter()
        .filter(|t| t.2 == token_type::OPERATOR)
        .map(|t| t.0)
        .collect()
}

#[test]
fn test_same_op_chain() {
    // １＋２＋３: 両方の ＋ が演算子、３ つの数値も各項として出る。
    let vt = rhs_tokens("１＋２＋３");
    assert_eq!(
        vt,
        vec![
            (4, 1, token_type::NUMBER),
            (5, 1, token_type::OPERATOR),
            (6, 1, token_type::NUMBER),
            (7, 1, token_type::OPERATOR),
            (8, 1, token_type::NUMBER),
        ]
    );
}

#[test]
fn test_op_inside_string_literal_is_not_operator() {
    // 「A＋B」＋＄ｙ: 文字列の中の ＋ は演算子でない。
    assert_eq!(operator_cols("「A＋B」＋＄ｙ"), vec![9]);
}

#[test]
fn test_global_marker_is_not_operator() {
    // ＄＊ｇ＊２: ＄ の直後の ＊ はマーカー。
    let vt = rhs_tokens("＄＊ｇ＊２");
    assert_eq!(
        vt,
        vec![
            (4, 3, token_type::VARIABLE),
            (7, 1, token_type::OPERATOR),
            (8, 1, token_type::NUMBER),
        ]
    );
}

#[test]
fn test_unary_minus_belongs_to_term() {
    // １－－２: 2 つ目の － は項の先頭（負号）。
    assert_eq!(operator_cols("１－－２"), vec![5]);
}

#[test]
fn test_multiply_divide_signs() {
    // ２×３÷４: 文法の × ÷ も演算子。
    assert_eq!(operator_cols("２×３÷４"), vec![5, 7]);
}

#[test]
fn test_mixed_ops_with_paren() {
    // （１＋２）＊３－４: 括弧内の ＋ も括弧内の連鎖として出る。
    assert_eq!(operator_cols("（１＋２）＊３－４"), vec![4, 6, 8, 9, 11]);
}

#[test]
fn test_count_mismatch_falls_back_to_single_token() {
    // ＄％a(＋１: プロパティ名の括弧が閉じず ＋ が深さ 1 になり、演算子が
    // 見つからない。落ちずに式全体を 1 トークンで出す。
    assert_eq!(rhs_tokens("＄％a(＋１"), vec![(4, 6, token_type::VARIABLE)]);
}
