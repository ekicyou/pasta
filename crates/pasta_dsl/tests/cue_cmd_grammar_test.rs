//! キューコマンド行（!行）の AST 型と PEG 文法ルールのテスト

use pasta_dsl::parser::*;
use pest::Parser as PestParser;

// ============================================================================
// AST Type Construction Tests
// ============================================================================

#[test]
fn test_cue_command_node_construction() {
    let node = CueCommandNode {
        command: "emote".to_string(),
        scope: None,
        args: vec![],
        span: Span::default(),
    };
    assert_eq!(node.command, "emote");
    assert!(node.scope.is_none());
    assert!(node.args.is_empty());
}

#[test]
fn test_cue_command_node_with_scope_and_args() {
    let scope = ScopedName {
        actor: None,
        name: "笑顔".to_string(),
        span: Span::default(),
    };
    let node = CueCommandNode {
        command: "emote".to_string(),
        scope: Some(scope),
        args: vec![CueArgToken::Ident("normal".to_string())],
        span: Span::default(),
    };
    assert_eq!(node.command, "emote");
    assert!(node.scope.is_some());
    let s = node.scope.unwrap();
    assert!(s.actor.is_none());
    assert_eq!(s.name, "笑顔");
    assert_eq!(node.args.len(), 1);
}

#[test]
fn test_scoped_name_simple() {
    let name = ScopedName {
        actor: None,
        name: "笑顔".to_string(),
        span: Span::default(),
    };
    assert!(name.actor.is_none());
    assert_eq!(name.name, "笑顔");
}

#[test]
fn test_scoped_name_with_actor() {
    let name = ScopedName {
        actor: Some("さくら".to_string()),
        name: "笑顔".to_string(),
        span: Span::default(),
    };
    assert_eq!(name.actor, Some("さくら".to_string()));
    assert_eq!(name.name, "笑顔");
}

#[test]
fn test_scoped_name_equality() {
    let a = ScopedName {
        actor: None,
        name: "test".to_string(),
        span: Span::default(),
    };
    let b = ScopedName {
        actor: None,
        name: "test".to_string(),
        span: Span::default(),
    };
    assert_eq!(a, b);
}

#[test]
fn test_scoped_name_inequality_different_actor() {
    let a = ScopedName {
        actor: Some("actor1".to_string()),
        name: "test".to_string(),
        span: Span::default(),
    };
    let b = ScopedName {
        actor: Some("actor2".to_string()),
        name: "test".to_string(),
        span: Span::default(),
    };
    assert_ne!(a, b);
}

#[test]
fn test_cue_arg_token_ident() {
    let token = CueArgToken::Ident("normal".to_string());
    assert_eq!(token, CueArgToken::Ident("normal".to_string()));
}

#[test]
fn test_cue_arg_token_string_literal() {
    let token = CueArgToken::StringLiteral("hello".to_string());
    assert_eq!(token, CueArgToken::StringLiteral("hello".to_string()));
}

#[test]
fn test_cue_arg_token_integer() {
    let token = CueArgToken::Integer(42);
    assert_eq!(token, CueArgToken::Integer(42));
}

#[test]
fn test_cue_arg_token_float() {
    let token = CueArgToken::Float(10.5);
    assert_eq!(token, CueArgToken::Float(10.5));
}

#[test]
fn test_cue_arg_token_at_ref() {
    let token = CueArgToken::AtRef("name".to_string());
    assert_eq!(token, CueArgToken::AtRef("name".to_string()));
}

#[test]
fn test_local_scene_item_cue_command_variant() {
    let node = CueCommandNode {
        command: "clear".to_string(),
        scope: None,
        args: vec![],
        span: Span::default(),
    };
    let item = LocalSceneItem::CueCommand(node);
    assert!(matches!(item, LocalSceneItem::CueCommand(_)));
}

// ============================================================================
// PEG Grammar Rule Parsing Tests
// ============================================================================

#[test]
fn test_pest_cue_cmd_line_simple() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !clear\n");
    assert!(
        result.is_ok(),
        "Simple cue command should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_with_scope() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@笑顔\n");
    assert!(
        result.is_ok(),
        "Cue command with scope should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_with_args() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !yield(10.0)\n");
    assert!(
        result.is_ok(),
        "Cue command with args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_with_scope_and_args() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@普通(normal)\n");
    assert!(
        result.is_ok(),
        "Cue command with scope and args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_fullwidth_marker() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  ！clear\n");
    assert!(
        result.is_ok(),
        "Full-width cue marker should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_scoped_with_actor() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@さくら:笑顔(smile)\n");
    assert!(
        result.is_ok(),
        "Scoped ident with actor:name should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_multiple_args() {
    let result = PastaParser2::parse(
        Rule::cue_cmd_line,
        "  !choice@はい(yes, 「はい、行きましょう！」)\n",
    );
    assert!(
        result.is_ok(),
        "Multiple args including string literal should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_at_ref_arg() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !seek(@名前, 1.0)\n");
    assert!(result.is_ok(), "AtRef arg should parse: {:?}", result.err());
}

#[test]
fn test_pest_cue_cmd_line_with_comment() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !clear # コメント\n");
    assert!(
        result.is_ok(),
        "Cue command with inline comment should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_fullwidth_parens() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  ！選択待ち（30）\n");
    assert!(
        result.is_ok(),
        "Full-width parens should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_empty_args() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !select()\n");
    assert!(
        result.is_ok(),
        "Empty args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_arg_with_colons() {
    let result = PastaParser2::parse(
        Rule::cue_cmd_line,
        "  !route_add(shell, actor:さくら:shell)\n",
    );
    assert!(
        result.is_ok(),
        "Args with colons should parse as cue_arg_id: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_integer_arg() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !select(30)\n");
    assert!(
        result.is_ok(),
        "Integer arg should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_negative_number_arg() {
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !offset(-5.0)\n");
    assert!(
        result.is_ok(),
        "Negative number arg should parse: {:?}",
        result.err()
    );
}

// ============================================================================
// Full Script Integration Tests
// ============================================================================

#[test]
fn test_full_script_with_cue_cmd_parses() {
    let source = concat!(
        "\u{ff0a}起動挨拶\n",
        "\n",
        "    %さくら\n",
        "\n",
        "    !emote@普通(normal)\n",
        "    !emote@笑顔(smile)\n",
        "    !choice@はい(yes, 「はい、行きましょう！」)\n",
        "\n",
        "    さくら：こんにちは\n",
        "\n",
        "    !mark@挨拶後\n",
        "\n",
        "    さくら：お散歩でも行きませんか？\n",
        "\n",
        "    !yield(10.0)\n",
        "    !clear\n",
        "    !select(30.0)\n",
    );
    let result = parse_str(source, "test.pasta");
    assert!(
        result.is_ok(),
        "Full script with cue commands should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_cue_cmd_does_not_break_existing_scenes() {
    let source = "＊挨拶\n  Alice：こんにちは\n  ：続きです\n";
    let result = parse_str(source, "test.pasta");
    assert!(
        result.is_ok(),
        "Existing scene without cue commands should still parse"
    );
    let file = result.unwrap();
    let scenes: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| {
            if let FileItem::GlobalSceneScope(s) = item {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].name, "挨拶");
}

#[test]
fn test_mixed_scene_cue_and_action_lines() {
    let source = concat!(
        "＊混在テスト\n",
        "    !emote@普通(normal)\n",
        "    Alice：こんにちは\n",
        "    !mark@pos1\n",
        "    ：続きの台詞\n",
        "    !clear\n",
    );
    let result = parse_str(source, "test.pasta");
    assert!(
        result.is_ok(),
        "Mixed scene with cue and action lines should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_cue_cmd_in_local_scene() {
    let source = concat!(
        "＊テスト\n",
        "    Alice：こんにちは\n",
        "    ・サブシーン\n",
        "    !mark@ここ\n",
        "    Bob：やあ\n",
    );
    let result = parse_str(source, "test.pasta");
    assert!(
        result.is_ok(),
        "Cue command in local scene should parse: {:?}",
        result.err()
    );
}

// ============================================================================
// Grammar Rule Token Structure Tests
// ============================================================================

#[test]
fn test_pest_cue_cmd_line_inner_structure_simple() {
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !clear\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    assert_eq!(cmd_line.as_rule(), Rule::cue_cmd_line);

    let inner: Vec<_> = cmd_line.into_inner().collect();
    assert!(
        inner.iter().any(|p| p.as_rule() == Rule::cue_cmd_name),
        "Should contain cue_cmd_name"
    );
    let name_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::cue_cmd_name)
        .unwrap();
    assert_eq!(name_pair.as_str(), "clear");
}

#[test]
fn test_pest_cue_cmd_line_inner_structure_with_scope() {
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@笑顔\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    let inner: Vec<_> = cmd_line.into_inner().collect();

    let name_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::cue_cmd_name)
        .unwrap();
    assert_eq!(name_pair.as_str(), "emote");

    assert!(
        inner.iter().any(|p| p.as_rule() == Rule::cue_cmd_scope),
        "Should contain cue_cmd_scope"
    );
}

#[test]
fn test_pest_cue_cmd_line_inner_structure_with_args() {
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !yield(10.0)\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    let inner: Vec<_> = cmd_line.into_inner().collect();

    let name_pair = inner
        .iter()
        .find(|p| p.as_rule() == Rule::cue_cmd_name)
        .unwrap();
    assert_eq!(name_pair.as_str(), "yield");

    assert!(
        inner.iter().any(|p| p.as_rule() == Rule::cue_cmd_args),
        "Should contain cue_cmd_args"
    );
}

#[test]
fn test_pest_cue_scoped_ident_simple() {
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@普通(normal)\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    let scope_pair = cmd_line
        .into_inner()
        .find(|p| p.as_rule() == Rule::cue_cmd_scope)
        .unwrap();

    let scoped_ident = scope_pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::cue_scoped_ident)
        .unwrap();
    assert_eq!(scoped_ident.as_str(), "普通");
}

#[test]
fn test_pest_cue_scoped_ident_with_actor() {
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@さくら:笑顔\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    let scope_pair = cmd_line
        .into_inner()
        .find(|p| p.as_rule() == Rule::cue_cmd_scope)
        .unwrap();

    let scoped_ident = scope_pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::cue_scoped_ident)
        .unwrap();
    // Atomic rule: full text including colon
    assert_eq!(scoped_ident.as_str(), "さくら:笑顔");
}
