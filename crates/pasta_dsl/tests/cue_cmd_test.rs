//! キューコマンド行（!行）の AST 型と PEG 文法ルールのテスト
//!
//! Task 1: AST 型定義テスト（CueCommandNode, ScopedName, CueArgToken）
//! Task 2: PEG 文法ルールテスト（cue_cmd_line 関連ルール）
//! Task 3: パースロジックテスト（parse_cue_cmd_line による AST 構築）
//! Task 4: 行種推定・キューコマンド検出ヘルパーテスト

use pasta_dsl::parser::*;
use pasta_dsl::partial::infer_rule_from_line;
use pest::Parser as PestParser;

// ============================================================================
// Helper: extract CueCommandNodes from parsed source
// ============================================================================

/// Parse source and extract CueCommandNodes from the first global scene's first local scene.
fn parse_and_extract_cue_commands(source: &str) -> Vec<CueCommandNode> {
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let mut cue_commands = Vec::new();
    for item in &file.items {
        if let FileItem::GlobalSceneScope(scene) = item {
            for local in &scene.local_scenes {
                for li in &local.items {
                    if let LocalSceneItem::CueCommand(node) = li {
                        cue_commands.push(node.clone());
                    }
                }
            }
        }
    }
    cue_commands
}

// ============================================================================
// Task 1: AST Type Construction Tests
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
// Task 2: PEG Grammar Rule Parsing Tests
// ============================================================================

#[test]
fn test_pest_cue_cmd_line_simple() {
    // !clear (command only, no scope, no args)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !clear\n");
    assert!(
        result.is_ok(),
        "Simple cue command should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_with_scope() {
    // !emote@笑顔 (command + scope, no args)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@笑顔\n");
    assert!(
        result.is_ok(),
        "Cue command with scope should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_with_args() {
    // !yield(10.0) (command + args, no scope)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !yield(10.0)\n");
    assert!(
        result.is_ok(),
        "Cue command with args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_with_scope_and_args() {
    // !emote@普通(normal) (command + scope + args)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@普通(normal)\n");
    assert!(
        result.is_ok(),
        "Cue command with scope and args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_fullwidth_marker() {
    // ！clear (full-width exclamation mark)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  ！clear\n");
    assert!(
        result.is_ok(),
        "Full-width cue marker should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_scoped_with_actor() {
    // !emote@さくら:笑顔(smile) (scoped ident with actor:name)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@さくら:笑顔(smile)\n");
    assert!(
        result.is_ok(),
        "Scoped ident with actor:name should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_multiple_args() {
    // !choice@はい(yes, 「はい、行きましょう！」) (multiple args with string literal)
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
    // !seek(@名前, 1.0) (at-ref argument)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !seek(@名前, 1.0)\n");
    assert!(result.is_ok(), "AtRef arg should parse: {:?}", result.err());
}

#[test]
fn test_pest_cue_cmd_line_with_comment() {
    // !clear # comment (inline comment)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !clear # コメント\n");
    assert!(
        result.is_ok(),
        "Cue command with inline comment should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_fullwidth_parens() {
    // ！選択待ち（30） (full-width parens and marker)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  ！選択待ち（30）\n");
    assert!(
        result.is_ok(),
        "Full-width parens should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_empty_args() {
    // !select() (empty argument list)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !select()\n");
    assert!(
        result.is_ok(),
        "Empty args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_arg_with_colons() {
    // !route_add(shell, actor:さくら:shell) (arg containing colons)
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
    // !select(30) (integer argument)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !select(30)\n");
    assert!(
        result.is_ok(),
        "Integer arg should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pest_cue_cmd_line_negative_number_arg() {
    // !offset(-5.0) (negative number argument)
    let result = PastaParser2::parse(Rule::cue_cmd_line, "  !offset(-5.0)\n");
    assert!(
        result.is_ok(),
        "Negative number arg should parse: {:?}",
        result.err()
    );
}

// ============================================================================
// Task 2: Full Script Integration Tests
// ============================================================================

#[test]
fn test_full_script_with_cue_cmd_parses() {
    // Full script with ! lines inside a scene (requirements sample)
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
    // Scene without cue commands should parse as before
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
    // Scene with both action lines and cue command lines
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
    // Cue command inside a local scene
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
// Task 2: Grammar Rule Token Structure Tests
// ============================================================================

#[test]
fn test_pest_cue_cmd_line_inner_structure_simple() {
    // Verify the inner token structure of a simple cue command
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !clear\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    assert_eq!(cmd_line.as_rule(), Rule::cue_cmd_line);

    let inner: Vec<_> = cmd_line.into_inner().collect();
    // Should have cue_cmd_name
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

    // Should have cue_cmd_name and cue_cmd_scope
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
    // Verify cue_scoped_ident captures the full text as atomic
    let pairs = PastaParser2::parse(Rule::cue_cmd_line, "  !emote@普通(normal)\n").unwrap();
    let cmd_line = pairs.into_iter().next().unwrap();
    let scope_pair = cmd_line
        .into_inner()
        .find(|p| p.as_rule() == Rule::cue_cmd_scope)
        .unwrap();

    // cue_cmd_scope = { at ~ cue_scoped_ident }
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

// ============================================================================
// Task 3: Parse Logic Tests — CueCommandNode AST Construction
// ============================================================================

#[test]
fn test_parse_cue_cmd_simple_command_only() {
    // !clear → command="clear", scope=None, args=[]
    let source = "＊テスト\n    !clear\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1, "Should find 1 cue command");
    assert_eq!(cmds[0].command, "clear");
    assert!(cmds[0].scope.is_none());
    assert!(cmds[0].args.is_empty());
}

#[test]
fn test_parse_cue_cmd_with_scope_no_actor() {
    // !emote@笑顔 → command="emote", scope=ScopedName{actor:None, name:"笑顔"}, args=[]
    let source = "＊テスト\n    !emote@笑顔\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "emote");
    let scope = cmds[0].scope.as_ref().expect("Should have scope");
    assert!(scope.actor.is_none());
    assert_eq!(scope.name, "笑顔");
    assert!(cmds[0].args.is_empty());
}

#[test]
fn test_parse_cue_cmd_with_scope_and_actor() {
    // !emote@さくら:笑顔 → scope=ScopedName{actor:Some("さくら"), name:"笑顔"}
    let source = "＊テスト\n    !emote@さくら:笑顔\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "emote");
    let scope = cmds[0].scope.as_ref().expect("Should have scope");
    assert_eq!(scope.actor, Some("さくら".to_string()));
    assert_eq!(scope.name, "笑顔");
}

#[test]
fn test_parse_cue_cmd_with_float_arg() {
    // !yield(10.0) → args=[Float(10.0)]
    let source = "＊テスト\n    !yield(10.0)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "yield");
    assert!(cmds[0].scope.is_none());
    assert_eq!(cmds[0].args.len(), 1);
    assert_eq!(cmds[0].args[0], CueArgToken::Float(10.0));
}

#[test]
fn test_parse_cue_cmd_with_integer_arg() {
    // !select(30) → args=[Integer(30)]
    let source = "＊テスト\n    !select(30)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "select");
    assert_eq!(cmds[0].args.len(), 1);
    assert_eq!(cmds[0].args[0], CueArgToken::Integer(30));
}

#[test]
fn test_parse_cue_cmd_with_ident_arg() {
    // !emote@普通(normal) → args=[Ident("normal")]
    let source = "＊テスト\n    !emote@普通(normal)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "emote");
    assert_eq!(cmds[0].args.len(), 1);
    assert_eq!(cmds[0].args[0], CueArgToken::Ident("normal".to_string()));
}

#[test]
fn test_parse_cue_cmd_with_string_literal_arg() {
    // !choice@はい(yes, 「はい、行きましょう！」) → args=[Ident("yes"), StringLiteral("はい、行きましょう！")]
    let source = "＊テスト\n    !choice@はい(yes, 「はい、行きましょう！」)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "choice");
    let scope = cmds[0].scope.as_ref().expect("Should have scope");
    assert_eq!(scope.name, "はい");
    assert_eq!(cmds[0].args.len(), 2);
    assert_eq!(cmds[0].args[0], CueArgToken::Ident("yes".to_string()));
    assert_eq!(
        cmds[0].args[1],
        CueArgToken::StringLiteral("はい、行きましょう！".to_string())
    );
}

#[test]
fn test_parse_cue_cmd_with_at_ref_arg() {
    // !seek(@名前, 1.0) → args=[AtRef("名前"), Float(1.0)]
    let source = "＊テスト\n    !seek(@名前, 1.0)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "seek");
    assert!(cmds[0].scope.is_none());
    assert_eq!(cmds[0].args.len(), 2);
    assert_eq!(cmds[0].args[0], CueArgToken::AtRef("名前".to_string()));
    assert_eq!(cmds[0].args[1], CueArgToken::Float(1.0));
}

#[test]
fn test_parse_cue_cmd_with_negative_float() {
    // !offset(-5.0) → args=[Float(-5.0)]
    let source = "＊テスト\n    !offset(-5.0)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "offset");
    assert_eq!(cmds[0].args.len(), 1);
    assert_eq!(cmds[0].args[0], CueArgToken::Float(-5.0));
}

#[test]
fn test_parse_cue_cmd_empty_args() {
    // !select() → args=[]
    let source = "＊テスト\n    !select()\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "select");
    assert!(cmds[0].args.is_empty());
}

#[test]
fn test_parse_cue_cmd_fullwidth_marker_and_parens() {
    // ！選択待ち（30） → command="選択待ち", args=[Integer(30)]
    let source = "＊テスト\n    ！選択待ち（30）\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "選択待ち");
    assert_eq!(cmds[0].args.len(), 1);
    assert_eq!(cmds[0].args[0], CueArgToken::Integer(30));
}

#[test]
fn test_parse_cue_cmd_colon_in_arg() {
    // !route_add(shell, actor:さくら:shell) → args=[Ident("shell"), Ident("actor:さくら:shell")]
    let source = "＊テスト\n    !route_add(shell, actor:さくら:shell)\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "route_add");
    assert_eq!(cmds[0].args.len(), 2);
    assert_eq!(cmds[0].args[0], CueArgToken::Ident("shell".to_string()));
    assert_eq!(
        cmds[0].args[1],
        CueArgToken::Ident("actor:さくら:shell".to_string())
    );
}

#[test]
fn test_parse_cue_cmd_span_is_valid() {
    // Verify CueCommandNode has a valid span
    let source = "＊テスト\n    !clear\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    assert!(
        cmds[0].span.is_valid(),
        "CueCommandNode span should be valid"
    );
}

#[test]
fn test_parse_cue_cmd_scope_span_is_valid() {
    // Verify ScopedName has a valid span
    let source = "＊テスト\n    !emote@笑顔\n";
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 1);
    let scope = cmds[0].scope.as_ref().expect("Should have scope");
    assert!(scope.span.is_valid(), "ScopedName span should be valid");
}

#[test]
fn test_parse_multiple_cue_commands_in_scene() {
    // Multiple cue commands in a single scene
    let source = concat!(
        "＊テスト\n",
        "    !emote@普通(normal)\n",
        "    !mark@挨拶後\n",
        "    !yield(10.0)\n",
        "    !clear\n",
    );
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 4);
    assert_eq!(cmds[0].command, "emote");
    assert_eq!(cmds[1].command, "mark");
    assert_eq!(cmds[2].command, "yield");
    assert_eq!(cmds[3].command, "clear");
}

#[test]
fn test_parse_cue_cmd_mixed_with_action_lines() {
    // Cue commands mixed with action lines
    let source = concat!(
        "＊テスト\n",
        "    !emote@普通(normal)\n",
        "    Alice：こんにちは\n",
        "    !mark@挨拶後\n",
        "    ：続きです\n",
        "    !clear\n",
    );
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };
    let items = &scene.local_scenes[0].items;

    // Verify the ordering: CueCommand, ActionLine, CueCommand, ContinueAction, CueCommand
    assert!(matches!(items[0], LocalSceneItem::CueCommand(_)));
    assert!(matches!(items[1], LocalSceneItem::ActionLine(_)));
    assert!(matches!(items[2], LocalSceneItem::CueCommand(_)));
    assert!(matches!(items[3], LocalSceneItem::ContinueAction(_)));
    assert!(matches!(items[4], LocalSceneItem::CueCommand(_)));
}

#[test]
fn test_parse_cue_cmd_in_named_local_scene() {
    // Cue command inside a named local scene (not start scene)
    let source = concat!(
        "＊テスト\n",
        "    Alice：こんにちは\n",
        "    ・サブシーン\n",
        "    !mark@ここ\n",
        "    Bob：やあ\n",
    );
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };

    // local_scenes[0] = start scene, local_scenes[1] = named local scene
    assert_eq!(scene.local_scenes.len(), 2);
    let named_local = &scene.local_scenes[1];
    assert_eq!(named_local.name, Some("サブシーン".to_string()));

    let cue_cmds: Vec<_> = named_local
        .items
        .iter()
        .filter_map(|item| {
            if let LocalSceneItem::CueCommand(node) = item {
                Some(node)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cue_cmds.len(), 1);
    assert_eq!(cue_cmds[0].command, "mark");
}

#[test]
fn test_parse_full_sample_scene_ast_construction() {
    // Full requirements sample scene - verify all CueCommandNodes
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
    let cmds = parse_and_extract_cue_commands(source);
    assert_eq!(cmds.len(), 7);

    // !emote@普通(normal)
    assert_eq!(cmds[0].command, "emote");
    assert_eq!(cmds[0].scope.as_ref().unwrap().name, "普通");
    assert_eq!(cmds[0].args, vec![CueArgToken::Ident("normal".to_string())]);

    // !emote@笑顔(smile)
    assert_eq!(cmds[1].command, "emote");
    assert_eq!(cmds[1].scope.as_ref().unwrap().name, "笑顔");
    assert_eq!(cmds[1].args, vec![CueArgToken::Ident("smile".to_string())]);

    // !choice@はい(yes, 「はい、行きましょう！」)
    assert_eq!(cmds[2].command, "choice");
    assert_eq!(cmds[2].scope.as_ref().unwrap().name, "はい");
    assert_eq!(cmds[2].args.len(), 2);
    assert_eq!(cmds[2].args[0], CueArgToken::Ident("yes".to_string()));
    assert_eq!(
        cmds[2].args[1],
        CueArgToken::StringLiteral("はい、行きましょう！".to_string())
    );

    // !mark@挨拶後
    assert_eq!(cmds[3].command, "mark");
    assert_eq!(cmds[3].scope.as_ref().unwrap().name, "挨拶後");
    assert!(cmds[3].args.is_empty());

    // !yield(10.0)
    assert_eq!(cmds[4].command, "yield");
    assert!(cmds[4].scope.is_none());
    assert_eq!(cmds[4].args, vec![CueArgToken::Float(10.0)]);

    // !clear
    assert_eq!(cmds[5].command, "clear");
    assert!(cmds[5].scope.is_none());
    assert!(cmds[5].args.is_empty());

    // !select(30.0)
    assert_eq!(cmds[6].command, "select");
    assert!(cmds[6].scope.is_none());
    assert_eq!(cmds[6].args, vec![CueArgToken::Float(30.0)]);
}

// ============================================================================
// Task 4: Line Type Inference Tests
// ============================================================================

#[test]
fn test_infer_rule_halfwidth_exclamation() {
    // !command → Rule::cue_cmd_line
    let result = infer_rule_from_line("!command");
    assert_eq!(result, Some(Rule::cue_cmd_line));
}

#[test]
fn test_infer_rule_fullwidth_exclamation() {
    // ！コマンド → Rule::cue_cmd_line
    let result = infer_rule_from_line("！コマンド");
    assert_eq!(result, Some(Rule::cue_cmd_line));
}

#[test]
fn test_infer_rule_exclamation_with_leading_whitespace() {
    // Leading whitespace should be ignored
    let result = infer_rule_from_line("    !emote@笑顔(normal)");
    assert_eq!(result, Some(Rule::cue_cmd_line));
}

#[test]
fn test_infer_rule_exclamation_fullwidth_with_whitespace() {
    let result = infer_rule_from_line("  ！選択待ち（30）");
    assert_eq!(result, Some(Rule::cue_cmd_line));
}

// ============================================================================
// Task 4: has_cue_commands() Helper Tests
// ============================================================================

#[test]
fn test_has_cue_commands_local_scene_with_cue() {
    // Scene with cue commands → has_cue_commands() == true
    let source = concat!("＊テスト\n", "    !clear\n", "    Alice：こんにちは\n",);
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };
    assert!(
        scene.local_scenes[0].has_cue_commands(),
        "LocalSceneScope with !clear should have cue commands"
    );
}

#[test]
fn test_has_cue_commands_local_scene_without_cue() {
    // Scene without cue commands → has_cue_commands() == false
    let source = "＊テスト\n    Alice：こんにちは\n    ：続き\n";
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };
    assert!(
        !scene.local_scenes[0].has_cue_commands(),
        "LocalSceneScope without cue commands should return false"
    );
}

#[test]
fn test_has_cue_commands_global_scene_with_cue() {
    // GlobalSceneScope::has_cue_commands() delegates to local scenes
    let source = concat!("＊テスト\n", "    !mark@start\n", "    Alice：こんにちは\n",);
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };
    assert!(
        scene.has_cue_commands(),
        "GlobalSceneScope should detect cue commands in local scenes"
    );
}

#[test]
fn test_has_cue_commands_global_scene_without_cue() {
    let source = "＊テスト\n    Alice：こんにちは\n";
    let file = parse_str(source, "test.pasta").expect("Parse should succeed");
    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        _ => panic!("Expected GlobalSceneScope"),
    };
    assert!(
        !scene.has_cue_commands(),
        "GlobalSceneScope without cue commands should return false"
    );
}

#[test]
fn test_has_cue_commands_constructed_local_scene_scope() {
    // Construct LocalSceneScope manually and test has_cue_commands
    let mut scope = LocalSceneScope::start();
    assert!(!scope.has_cue_commands());

    scope.items.push(LocalSceneItem::CueCommand(CueCommandNode {
        command: "clear".to_string(),
        scope: None,
        args: vec![],
        span: Span::default(),
    }));
    assert!(scope.has_cue_commands());
}

#[test]
fn test_has_cue_commands_constructed_global_scene_scope() {
    // Construct GlobalSceneScope manually and test has_cue_commands
    let mut global = GlobalSceneScope::new("test".to_string());
    assert!(!global.has_cue_commands());

    let mut local = LocalSceneScope::start();
    local.items.push(LocalSceneItem::CueCommand(CueCommandNode {
        command: "mark".to_string(),
        scope: None,
        args: vec![],
        span: Span::default(),
    }));
    global.local_scenes.push(local);
    assert!(global.has_cue_commands());
}
