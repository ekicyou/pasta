//! 選択肢行（＠？行）の PEG 文法ルールテスト
//!
//! Task 1.1: choice_line 文法規則のパーステスト
//! Task 1.2: AST 構築テスト（ChoiceNode / parse_choice_line）

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

// ============================================================================
// Task 1.2: AST-level parse tests (ChoiceNode / parse_choice_line)
// ============================================================================

/// Parse source and extract ChoiceNodes from the first global scene's local scenes.
fn parse_and_extract_choices(source: &str) -> Vec<ChoiceNode> {
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let mut choices = Vec::new();
    for item in &file.items {
        if let FileItem::GlobalSceneScope(scene) = item {
            for local in &scene.local_scenes {
                for li in &local.items {
                    if let LocalSceneItem::Choice(node) = li {
                        choices.push(node.clone());
                    }
                }
            }
        }
    }
    choices
}

#[test]
fn test_ast_choice_node_shorthand() {
    let source = "\
＊メニュー
  ＠？挨拶
";
    let choices = parse_and_extract_choices(source);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].target, "挨拶");
    assert_eq!(choices[0].label, None);
}

#[test]
fn test_ast_choice_node_with_label() {
    let source = "\
＊メニュー
  ＠？挨拶「こんにちはを選ぶ」
";
    let choices = parse_and_extract_choices(source);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].target, "挨拶");
    assert_eq!(choices[0].label, Some("こんにちはを選ぶ".to_string()));
}

#[test]
fn test_ast_choice_node_halfwidth() {
    let source = "\
＊メニュー
  @?greeting
";
    let choices = parse_and_extract_choices(source);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].target, "greeting");
    assert_eq!(choices[0].label, None);
}

#[test]
fn test_ast_choice_node_halfwidth_with_label() {
    let source = "\
＊メニュー
  @?greeting「say hello」
";
    let choices = parse_and_extract_choices(source);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].target, "greeting");
    assert_eq!(choices[0].label, Some("say hello".to_string()));
}

#[test]
fn test_ast_multiple_choices_in_scene() {
    let source = "\
＊メニュー
  さくら：選んでね。
  ＠？挨拶
  ＠？雑談「雑談する」
  ＠？終了
";
    let choices = parse_and_extract_choices(source);
    assert_eq!(choices.len(), 3);
    assert_eq!(choices[0].target, "挨拶");
    assert_eq!(choices[0].label, None);
    assert_eq!(choices[1].target, "雑談");
    assert_eq!(choices[1].label, Some("雑談する".to_string()));
    assert_eq!(choices[2].target, "終了");
    assert_eq!(choices[2].label, None);
}

#[test]
fn test_ast_choice_in_local_scene() {
    let source = concat!(
        "＊メニュー\n",
        "    さくら：こんにちは\n",
        "    ・サブメニュー\n",
        "    ＠？戻る\n",
        "    ＠？進む「次へ進む」\n",
    );
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };

    // local_scenes[0] = start scene, local_scenes[1] = named local scene
    assert_eq!(scene.local_scenes.len(), 2);
    let named_local = &scene.local_scenes[1];
    assert_eq!(named_local.name, Some("サブメニュー".to_string()));

    let choices: Vec<_> = named_local
        .items
        .iter()
        .filter_map(|item| {
            if let LocalSceneItem::Choice(node) = item {
                Some(node)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].target, "戻る");
    assert_eq!(choices[0].label, None);
    assert_eq!(choices[1].target, "進む");
    assert_eq!(choices[1].label, Some("次へ進む".to_string()));
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
