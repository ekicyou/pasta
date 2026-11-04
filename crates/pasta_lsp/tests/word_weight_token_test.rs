//! 単語の値の後ろの重み `＠数` のセマンティックトークンテスト
//!
//! Requirements 7.1, 7.2

use pasta_lsp::analysis::{AnalysisEngine, token_type};

/// 1 行目のトークンを (start_char, length, token_type) で取り出す（暗黙の SCENE は除く）。
fn line0_tokens(source: &str) -> Vec<(u32, u32, u32)> {
    let result = AnalysisEngine::analyze(source);
    let mut out = Vec::new();
    let mut line = 0u32;
    let mut ch = 0u32;
    for t in &result.tokens {
        if t.delta_line > 0 {
            line += t.delta_line;
            ch = t.delta_start;
        } else {
            ch += t.delta_start;
        }
        if line == 0 && t.token_type != token_type::SCENE {
            out.push((ch, t.length, t.token_type));
        }
    }
    out
}

#[test]
fn weights_are_numbers_between_values_are_words() {
    // ＠w：a＠３、「b」@12、c
    // UTF-16 単位（全角も 1）。先頭の `＠` は今と同じくトークンに含めない
    let src = "＠w：a＠３、「b」@12、c\n";
    assert_eq!(
        line0_tokens(src),
        vec![
            (1, 3, token_type::WORD),    // w：a
            (4, 2, token_type::NUMBER),  // ＠３
            (6, 4, token_type::WORD),    // 、「b」
            (10, 3, token_type::NUMBER), // @12
            (13, 2, token_type::WORD),   // 、c
        ]
    );
}

#[test]
fn weight_at_end_of_line() {
    let src = "＠w：こんにちは＠３\n";
    assert_eq!(
        line0_tokens(src),
        vec![(1, 7, token_type::WORD), (8, 2, token_type::NUMBER)]
    );
}

#[test]
fn at_inside_value_stays_word() {
    // 値の途中の `＠` は重みではないので値の色のまま
    let src = "＠w：x＠y、メール＠２\n";
    assert_eq!(
        line0_tokens(src),
        vec![(1, 9, token_type::WORD), (10, 2, token_type::NUMBER)]
    );
}

#[test]
fn without_weight_is_single_word_token() {
    let src = "＠w：x＠y、b\n";
    assert_eq!(line0_tokens(src), vec![(1, 7, token_type::WORD)]);
}
