//! 回帰テスト: 部分パース経路でアクタースコープ内・文字列リテラル RHS の
//! var_set をトークン化する際に、span のバイトオフセット不整合で
//! バイトスライスがパニックしないこと。
//!
//! # 背景（真因）
//! `AnalysisEngine::analyze` は完全パース失敗時に `parse_str_partial` へ
//! フォールバックする。部分パースはソースをスコープ境界で分割した
//! 「チャンク」を個別にパースするため、返される AST の span は
//! チャンク相対（バイト/行オフセットが 0 始まり）になる。
//! それをフルソースに対してトークン化すると span が別の行に当たり、
//! `span_line_window` 由来の span_text が空になったり、行内バイト
//! オフセットがマルチバイト文字の途中に落ちたりして、
//! `&str` のバイトスライスがパニックしていた。
//!
//! 具体的な再現入力では `tokenize_var_set_text` のマーカー検出が
//! 空 span_text でも `$` を選んで cursor を進め、`span_text[1..]`
//! （長さ 0）で `start byte index 1 is out of bounds` パニックを起こす。

use pasta_lsp::analysis::AnalysisEngine;

/// 非先頭チャンクのアクタースコープ + 全角文字列リテラル RHS。
/// 先頭の不正行 `＠` で完全パースを失敗させ、部分パース経路を強制する。
#[test]
fn test_partial_parse_actor_string_rhs_no_panic() {
    let source = "＠\n％さくら\n　＄名前＝「アリス」\n";
    // パニックせず完走すること（パニックすればテストプロセスごと落ちる）。
    let result = AnalysisEngine::analyze(source);
    // 部分パースなので診断は出るが、何らかの出力（トークンまたは診断）が返る。
    assert!(
        !result.tokens.is_empty() || !result.diagnostics.is_empty(),
        "部分パースで何らかの出力が返ること"
    );
}

/// ASCII アクター名版（報告書の元入力に近い形）。
#[test]
fn test_partial_parse_ascii_actor_string_rhs_no_panic() {
    let source = "＠\n％Alice\n　＄名前＝「アリス」\n";
    let result = AnalysisEngine::analyze(source);
    assert!(!result.tokens.is_empty() || !result.diagnostics.is_empty());
}

/// 完全パースが成功する通常経路は従来どおり panic せずトークンを返す
/// （アクタースコープ + 文字列 RHS の組み合わせ自体は健全であることの確認）。
#[test]
fn test_full_parse_actor_string_rhs_ok() {
    let source = "％さくら\n　＄名前＝「アリス」\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "完全パース成功（診断なし）");
    assert!(!result.tokens.is_empty(), "トークンが返る");
}
