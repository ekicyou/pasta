//! 選択肢行（＠？行）の PEG 文法ルールテスト
//!
//! Task 1.1: choice_line 文法規則のパーステスト

use pasta_dsl::parser::*;
use pest::Parser as PestParser;

// ============================================================================
// PEG-level parse tests for choice_line rule
// ============================================================================

#[test]
fn test_choice_line_shorthand_fullwidth() {
    // 省略形、全角マーカー
    let result = PastaParser2::parse(Rule::choice_line, "  ＠？target\n");
    assert!(result.is_ok(), "省略形（全角）がパースできること: {:?}", result.err());
}

#[test]
fn test_choice_line_with_label_fullwidth() {
    // 括弧形、全角マーカー
    let result = PastaParser2::parse(Rule::choice_line, "  ＠？target「表示テキスト」\n");
    assert!(result.is_ok(), "括弧形（全角）がパースできること: {:?}", result.err());
}

#[test]
fn test_choice_line_shorthand_halfwidth() {
    // 省略形、半角マーカー
    let result = PastaParser2::parse(Rule::choice_line, "  @?target\n");
    assert!(result.is_ok(), "省略形（半角）がパースできること: {:?}", result.err());
}

#[test]
fn test_choice_line_with_label_halfwidth() {
    // 括弧形、半角マーカー
    let result = PastaParser2::parse(Rule::choice_line, "  @?target「display text」\n");
    assert!(result.is_ok(), "括弧形（半角）がパースできること: {:?}", result.err());
}

#[test]
fn test_choice_line_missing_target() {
    // target 欠落 — パースエラーになること
    let result = PastaParser2::parse(Rule::choice_line, "  ＠？\n");
    assert!(result.is_err(), "target 欠落時にパースエラーになること");
}

#[test]
fn test_choice_line_with_comment() {
    // コメント付き
    let result = PastaParser2::parse(Rule::choice_line, "  ＠？target # コメント\n");
    assert!(result.is_ok(), "コメント付きがパースできること: {:?}", result.err());
}

// ============================================================================
// Parse tree structure tests
// ============================================================================

#[test]
fn test_choice_line_parse_tree_shorthand() {
    let pairs = PastaParser2::parse(Rule::choice_line, "  ＠？target\n").unwrap();
    let choice = pairs.into_iter().next().unwrap();
    assert_eq!(choice.as_rule(), Rule::choice_line);

    let inner: Vec<_> = choice.into_inner().collect();
    // choice_line should contain: id (and optionally choice_label)
    assert_eq!(inner.len(), 1, "省略形は id のみ");
    assert_eq!(inner[0].as_rule(), Rule::id);
    assert_eq!(inner[0].as_str(), "target");
}

#[test]
fn test_choice_line_parse_tree_with_label() {
    let pairs = PastaParser2::parse(Rule::choice_line, "  ＠？target「表示テキスト」\n").unwrap();
    let choice = pairs.into_iter().next().unwrap();
    assert_eq!(choice.as_rule(), Rule::choice_line);

    let inner: Vec<_> = choice.into_inner().collect();
    // choice_line should contain: id, choice_label
    assert_eq!(inner.len(), 2, "括弧形は id + choice_label");
    assert_eq!(inner[0].as_rule(), Rule::id);
    assert_eq!(inner[0].as_str(), "target");
    assert_eq!(inner[1].as_rule(), Rule::choice_label);
    assert_eq!(inner[1].as_str(), "「表示テキスト」");
}

// ============================================================================
// Full file context test (choice_line inside a scene)
// ============================================================================

#[test]
fn test_choice_line_in_scene_context() {
    let source = "\
＊メニュー
  さくら：こんにちは。選んでね。
  ＠？挨拶
  ＠？挨拶「こんにちはを選ぶ」
";
    let result = PastaParser2::parse(Rule::file, source);
    assert!(result.is_ok(), "シーン内の選択肢行がパースできること: {:?}", result.err());
}

#[test]
fn test_choice_line_in_local_scene_context() {
    let source = "\
＊メニュー
  ・サブ
    さくら：選んでね。
    ＠？挨拶「挨拶する」
";
    let result = PastaParser2::parse(Rule::file, source);
    assert!(result.is_ok(), "ローカルシーン内の選択肢行がパースできること: {:?}", result.err());
}
