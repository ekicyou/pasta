//! キューコマンド行（!行）のパースロジックと行種推定のテスト
//!
//! Task 3: パースロジックテスト（parse_cue_cmd_line による AST 構築）
//! Task 4: 行種推定・キューコマンド検出ヘルパーテスト

use pasta_dsl::parser::*;
use pasta_dsl::partial::infer_rule_from_line;

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
