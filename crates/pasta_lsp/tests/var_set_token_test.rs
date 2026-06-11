//! 変数代入行（var_set）のセマンティックトークン生成テスト (3.40 G1)
//!
//! visitors.rs の visit_var_set / tokenize_var_set_text / tokenize_expr_recursive
//! および text_utils の式スキャンヘルパー（find_number_literal / find_binary_op /
//! find_open_paren / find_close_paren / find_arg_end）は本ファイル以前は
//! テスト未到達だった（tests/・src 内に ＄/$ を含むテストが 0 件）。

use pasta_lsp::analysis::{AnalysisEngine, token_type};

// ============================================================================
// Helper
// ============================================================================

/// Decode delta-encoded tokens into (line, start_char, length, token_type, modifiers).
fn decode_tokens(result: &pasta_lsp::analysis::AnalysisResult) -> Vec<(u32, u32, u32, u32, u32)> {
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

/// Tokens on a given line, excluding implicit SCENE tokens.
fn tokens_on_line(
    decoded: &[(u32, u32, u32, u32, u32)],
    line: u32,
) -> Vec<(u32, u32, u32, u32, u32)> {
    decoded
        .iter()
        .filter(|t| t.0 == line && t.3 != token_type::SCENE)
        .copied()
        .collect()
}

/// 定義モディファイア（TOKEN_MODIFIERS[1] = DEFINITION）
const MOD_DEFINITION: u32 = 1 << 1;
/// グローバルスコープモディファイア（TOKEN_MODIFIERS[2] = global）
const MOD_GLOBAL: u32 = 1 << 2;

// ============================================================================
// ローカル変数代入: マーカー・変数名・代入演算子・RHS
// ============================================================================

#[test]
fn test_var_set_local_number_tokens() {
    // 行1: 　＄好感度＝１００
    let source = "＊挨拶\n　＄好感度＝１００\n　Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    // マーカー ＄ (col1, len1, VARIABLE, modなし)
    assert_eq!(
        vt[0],
        (1, 1, 1, token_type::VARIABLE, 0),
        "＄マーカー: {:?}",
        vt
    );
    // 変数名 好感度 (col2, len3, VARIABLE, definition)
    assert_eq!(
        vt[1],
        (1, 2, 3, token_type::VARIABLE, MOD_DEFINITION),
        "変数名（definitionモディファイア付き）: {:?}",
        vt
    );
    // 代入演算子 ＝ (col5, len1, OPERATOR)
    assert_eq!(
        vt[2],
        (1, 5, 1, token_type::OPERATOR, 0),
        "＝演算子: {:?}",
        vt
    );
    // 数値リテラル １００ (col6, len3, NUMBER)
    assert_eq!(
        vt[3],
        (1, 6, 3, token_type::NUMBER, 0),
        "数値リテラル: {:?}",
        vt
    );
}

#[test]
fn test_var_set_global_marker_and_modifiers() {
    // ＄＊回数＝５ — グローバル変数代入
    let source = "＊挨拶\n　＄＊回数＝５\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    // マーカー ＄＊ は2コードユニット
    assert_eq!(
        vt[0],
        (1, 1, 2, token_type::VARIABLE, 0),
        "＄＊マーカー: {:?}",
        vt
    );
    // 変数名は definition + global の両モディファイア
    assert_eq!(
        vt[1],
        (1, 3, 2, token_type::VARIABLE, MOD_DEFINITION | MOD_GLOBAL),
        "グローバル変数名（definition|global）: {:?}",
        vt
    );
    assert_eq!(
        vt[2],
        (1, 5, 1, token_type::OPERATOR, 0),
        "＝演算子: {:?}",
        vt
    );
    assert_eq!(vt[3], (1, 6, 1, token_type::NUMBER, 0), "数値: {:?}", vt);
}

#[test]
fn test_var_set_none_skips_name_token() {
    // ＄＝１ — 変数名なし（var_set_none）: マーカー直後に＝、名前トークンは出ない
    let source = "＊挨拶\n　＄＝１\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    assert_eq!(
        vt[0],
        (1, 1, 1, token_type::VARIABLE, 0),
        "＄マーカー: {:?}",
        vt
    );
    assert_eq!(
        vt[1],
        (1, 2, 1, token_type::OPERATOR, 0),
        "＝が直後: {:?}",
        vt
    );
    assert_eq!(vt[2], (1, 3, 1, token_type::NUMBER, 0), "数値: {:?}", vt);
    // definition モディファイア付き変数名トークンが存在しない
    assert!(
        !vt.iter().any(|t| t.4 & MOD_DEFINITION != 0),
        "名前なし代入に definition トークンは出ない: {:?}",
        vt
    );
}

// ============================================================================
// RHS 式バリエーション
// ============================================================================

#[test]
fn test_var_set_binary_expr_rhs() {
    // ＄ｘ＝１＋２ — 二項演算: NUMBER OPERATOR NUMBER
    let source = "＊挨拶\n　＄ｘ＝１＋２\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    // [marker, name, =, lhs数値, ＋演算子, rhs数値]
    assert_eq!(vt[3], (1, 4, 1, token_type::NUMBER, 0), "lhs数値: {:?}", vt);
    assert_eq!(
        vt[4],
        (1, 5, 1, token_type::OPERATOR, 0),
        "＋演算子: {:?}",
        vt
    );
    assert_eq!(vt[5], (1, 6, 1, token_type::NUMBER, 0), "rhs数値: {:?}", vt);
}

#[test]
fn test_var_set_word_ref_rhs() {
    // ＄挨拶語＝＠挨拶 — 単語参照 RHS は WORD トークン（＠込み len3）
    let source = "＊挨拶\n　＄挨拶語＝＠挨拶\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    assert_eq!(
        vt[3],
        (1, 6, 3, token_type::WORD, 0),
        "＠挨拶 WORD: {:?}",
        vt
    );
}

#[test]
fn test_var_set_var_ref_rhs() {
    // ＄ｙ＝＄ｘ — 変数参照 RHS は VARIABLE トークン（＄込み len2、modなし）
    let source = "＊挨拶\n　＄ｙ＝＄ｘ\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    assert_eq!(
        vt[3],
        (1, 4, 2, token_type::VARIABLE, 0),
        "＄ｘ参照: {:?}",
        vt
    );
}

#[test]
fn test_var_set_string_rhs() {
    // ＄名前＝「さくら」 — 文字列リテラル RHS は中身のみ TALK トークン
    let source = "＊挨拶\n　＄名前＝「さくら」\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    // 「 は col5、中身 さくら は col6 len3
    assert_eq!(
        vt[3],
        (1, 6, 3, token_type::TALK, 0),
        "文字列中身 TALK: {:?}",
        vt
    );
}

#[test]
fn test_var_set_fn_call_with_args_rhs() {
    // ＄ｚ＝＠乱数（１、２） — 関数呼び出し: WORD + 括弧 OPERATOR + 引数 NUMBER
    let source = "＊挨拶\n　＄ｚ＝＠乱数（１、２）\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    assert_eq!(
        vt[3],
        (1, 4, 3, token_type::WORD, 0),
        "＠乱数 WORD: {:?}",
        vt
    );
    assert_eq!(
        vt[4],
        (1, 7, 1, token_type::OPERATOR, 0),
        "開き括弧: {:?}",
        vt
    );
    assert_eq!(vt[5], (1, 8, 1, token_type::NUMBER, 0), "第1引数: {:?}", vt);
    assert_eq!(
        vt[6],
        (1, 10, 1, token_type::NUMBER, 0),
        "第2引数: {:?}",
        vt
    );
    assert_eq!(
        vt[7],
        (1, 11, 1, token_type::OPERATOR, 0),
        "閉じ括弧: {:?}",
        vt
    );
}

#[test]
fn test_var_set_paren_expr_rhs_characterization() {
    // ＄ｗ＝（１＋２）＊３ — 特性化テスト。
    //
    // 既知の上流問題（pasta_dsl 申し送り・3.40 G1 ではプロダクション不変更）:
    // pasta_dsl parse_action.rs try_parse_expr の Rule::paren_expr 分岐は
    // 括弧内の最初の term しか AST 化せず、（１＋２）は Paren(Integer(1)) になる。
    // そのため括弧内の ＋ と ２ のトークンは現状生成されない。
    // 本テストは LSP 側の現在の振る舞い（括弧 OPERATOR・lhs 数値・外側の
    // ＊ OPERATOR・rhs 数値）を固定する。上流修正時は本テストの期待値を
    // ＋/２ トークン込みに更新すること。
    let source = "＊挨拶\n　＄ｗ＝（１＋２）＊３\n　Alice：や\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let vt = tokens_on_line(&decode_tokens(&result), 1);

    assert_eq!(
        vt[3],
        (1, 4, 1, token_type::OPERATOR, 0),
        "開き括弧: {:?}",
        vt
    );
    assert_eq!(
        vt[4],
        (1, 5, 1, token_type::NUMBER, 0),
        "括弧内lhs数値: {:?}",
        vt
    );
    assert_eq!(
        vt[5],
        (1, 8, 1, token_type::OPERATOR, 0),
        "閉じ括弧: {:?}",
        vt
    );
    assert_eq!(
        vt[6],
        (1, 9, 1, token_type::OPERATOR, 0),
        "＊演算子: {:?}",
        vt
    );
    assert_eq!(
        vt[7],
        (1, 10, 1, token_type::NUMBER, 0),
        "rhs数値: {:?}",
        vt
    );
}

// ============================================================================
// アクター辞書スコープ内の変数代入（visit_actor_scope → visit_var_set 経路）
// ============================================================================

#[test]
fn test_var_set_in_actor_scope() {
    let source = "％さくら\n　＄好感度＝１\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "パースエラーなし");
    let decoded = decode_tokens(&result);

    // 行0: ％さくら 全体が ACTOR トークン
    assert_eq!(
        decoded[0],
        (0, 0, 4, token_type::ACTOR, 0),
        "アクターマーカー行: {:?}",
        decoded
    );
    // 行1: 変数代入の細粒度トークン
    let vt = tokens_on_line(&decoded, 1);
    assert_eq!(
        vt[0],
        (1, 1, 1, token_type::VARIABLE, 0),
        "＄マーカー: {:?}",
        vt
    );
    assert_eq!(
        vt[1],
        (1, 2, 3, token_type::VARIABLE, MOD_DEFINITION),
        "変数名: {:?}",
        vt
    );
    assert_eq!(
        vt[2],
        (1, 5, 1, token_type::OPERATOR, 0),
        "＝演算子: {:?}",
        vt
    );
    assert_eq!(vt[3], (1, 6, 1, token_type::NUMBER, 0), "数値: {:?}", vt);
}
