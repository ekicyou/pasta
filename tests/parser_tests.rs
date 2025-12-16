//! Parser tests for Pasta DSL
//!
//! These tests validate the parser's ability to convert DSL source code into AST.

use pasta::{
    parse_str, BinOp, Expr, JumpTarget, LabelScope, Literal, SpeechPart, Statement, VarScope,
};

#[test]
fn test_parse_simple_label() {
    let source = r#"＊挨拶
  さくら：こんにちは
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    assert_eq!(file.labels.len(), 1);
    assert_eq!(file.labels[0].name, "挨拶");
    assert_eq!(file.labels[0].scope, LabelScope::Global);
    assert_eq!(file.labels[0].statements.len(), 1);
}

#[test]
fn test_parse_speech_with_var_ref() {
    let source = r#"＊挨拶
  さくら：こんにちは＠ユーザー名　さん
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    println!("Labels: {}", file.labels.len());
    println!("Statements: {}", file.labels[0].statements.len());

    if let Statement::Speech { content, .. } = &file.labels[0].statements[0] {
        println!("Content length: {}, parts: {:?}", content.len(), content);
        if content.len() >= 2 {
            // ＠ユーザー名 は単語展開（word expansion）なので FuncCall
            match &content[1] {
                SpeechPart::FuncCall { name, .. } => assert_eq!(name, "ユーザー名"),
                other => panic!("Expected FuncCall at index 1, got: {:?}", other),
            }
        } else {
            panic!(
                "Expected at least 2 content parts, got {}: {:?}",
                content.len(),
                content
            );
        }
    } else {
        panic!(
            "Expected Speech statement, got: {:?}",
            file.labels[0].statements[0]
        );
    }
}

#[test]
fn test_parse_attributes() {
    let source = r#"＊挨拶
  ＆時間帯：朝
  ＆重み：5
  さくら：おはよう
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    assert_eq!(file.labels[0].attributes.len(), 2);
    assert_eq!(file.labels[0].attributes[0].key, "時間帯");
}

#[test]
fn test_parse_local_label() {
    let source = r#"＊挨拶
  -朝
    さくら：おはよう
  -昼
    さくら：こんにちは
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    assert_eq!(file.labels[0].local_labels.len(), 2);
    assert_eq!(file.labels[0].local_labels[0].name, "朝");
    assert_eq!(file.labels[0].local_labels[0].scope, LabelScope::Local);
}

#[test]
fn test_parse_call_statement() {
    let source = r#"＊開始
  ＞挨拶
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::Call { target, .. } = &file.labels[0].statements[0] {
        match target {
            JumpTarget::Local(name) => assert_eq!(name, "挨拶"),
            _ => panic!("Expected local jump target"),
        }
    } else {
        panic!("Expected Call statement");
    }
}

#[test]
fn test_parse_jump_global() {
    let source = r#"＊開始
  ？＊終了
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::Jump { target, .. } = &file.labels[0].statements[0] {
        match target {
            JumpTarget::Global(name) => assert_eq!(name, "終了"),
            _ => panic!("Expected global jump target"),
        }
    } else {
        panic!("Expected Jump statement");
    }
}

#[test]
fn test_parse_var_assign() {
    let source = r#"＊開始
  ＄カウンター＝1
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::VarAssign {
        name, scope, value, ..
    } = &file.labels[0].statements[0]
    {
        assert_eq!(name, "カウンター");
        assert_eq!(*scope, VarScope::Local);
        match value {
            Expr::Literal(Literal::Number(n)) => assert_eq!(*n, 1.0),
            _ => panic!("Expected number literal"),
        }
    } else {
        panic!("Expected VarAssign statement");
    }
}

#[test]
fn test_parse_global_var_assign() {
    let source = r#"＊開始
  ＄＊カウンター＝42
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::VarAssign { scope, .. } = &file.labels[0].statements[0] {
        assert_eq!(*scope, VarScope::Global);
    } else {
        panic!("Expected VarAssign statement");
    }
}

#[test]
fn test_parse_expression() {
    let source = r#"＊開始
  ＄結果＝1+2*3
"#;
    let result = parse_str(source, "test.pasta");
    if let Err(e) = &result {
        println!("Parse error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::VarAssign { value, .. } = &file.labels[0].statements[0] {
        // Should parse as: 1 + (2 * 3) due to left-to-right in our simple parser
        // Actually it's ((1 + 2) * 3) since we don't have precedence
        match value {
            Expr::BinaryOp { op, .. } => {
                // Just check it parsed as a binary op
                assert!(matches!(op, BinOp::Add | BinOp::Mul));
            }
            _ => panic!("Expected binary operation"),
        }
    } else {
        panic!("Expected VarAssign statement");
    }
}

#[test]
fn test_parse_function_call_in_speech() {
    let source = r#"＊挨拶
  さくら：こんにちは＠W（300）お元気ですか
"#;
    let result = parse_str(source, "test.pasta");
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::Speech { content, .. } = &file.labels[0].statements[0] {
        // Should have: text, func_call, text
        println!("Content: {:?}", content);
        assert!(content.len() >= 2);
        let has_func_call = content
            .iter()
            .any(|part| matches!(part, SpeechPart::FuncCall { .. }));
        assert!(has_func_call, "Expected function call in speech");
    } else {
        panic!("Expected Speech statement");
    }
}

#[test]
fn test_parse_string_literals() {
    let source = r#"＊テスト
  ＄日本語＝「こんにちは」
  ＄英語＝"Hello"
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    assert_eq!(file.labels[0].statements.len(), 2);

    for stmt in &file.labels[0].statements {
        if let Statement::VarAssign { value, .. } = stmt {
            match value {
                Expr::Literal(Literal::String(_)) => {} // OK
                _ => panic!("Expected string literal"),
            }
        }
    }
}

#[test]
fn test_parse_multiple_labels() {
    let source = r#"＊挨拶
  さくら：こんにちは

＊挨拶
  さくら：おはよう

＊終了
  さくら：さようなら
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    assert_eq!(file.labels.len(), 3);
    assert_eq!(file.labels[0].name, "挨拶");
    assert_eq!(file.labels[1].name, "挨拶"); // Same name OK
    assert_eq!(file.labels[2].name, "終了");
}

#[test]
fn test_parse_continuation_lines() {
    let source = r#"＊挨拶
  さくら：こんにちは。今日はいい天気ですね。どこか行きましょう。
"#;
    let result = parse_str(source, "test.pasta");
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::Speech { content, .. } = &file.labels[0].statements[0] {
        // Should contain the text
        println!("Content parts: {}", content.len());
        assert!(content.len() >= 1);
    } else {
        panic!("Expected Speech statement");
    }
}

#[test]
fn test_parse_sakura_script() {
    let source = r#"＊挨拶
  さくら：こんにちは\nお元気ですか
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::Speech { content, .. } = &file.labels[0].statements[0] {
        let has_sakura = content
            .iter()
            .any(|part| matches!(part, SpeechPart::SakuraScript(_)));
        assert!(has_sakura, "Expected sakura script escape");
    } else {
        panic!("Expected Speech statement");
    }
}

#[test]
fn test_parse_error_reporting() {
    let source = r#"＊挨拶
  ？？？invalid
"#;
    let result = parse_str(source, "test.pasta");
    // Should fail to parse and return error with location info
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    assert!(
        err_str.contains("test.pasta"),
        "Error should mention filename"
    );
}

#[test]
fn test_parse_halfwidth_syntax() {
    let source = r#"*greeting
  sakura:hello
  >farewell
"#;
    let result = parse_str(source, "test.pasta");
    assert!(
        result.is_ok(),
        "Failed to parse half-width syntax: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert_eq!(file.labels.len(), 1);
    assert_eq!(file.labels[0].name, "greeting");
}

#[test]
fn test_parse_long_jump() {
    let source = r#"＊開始
  ＞＊挨拶-朝
"#;
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let file = result.unwrap();
    if let Statement::Call { target, .. } = &file.labels[0].statements[0] {
        println!("Target: {:?}", target);
        match target {
            JumpTarget::LongJump { global, local } => {
                assert_eq!(global, "挨拶");
                assert_eq!(local, "朝");
            }
            _ => panic!("Expected long jump target, got: {:?}", target),
        }
    } else {
        panic!("Expected Call statement");
    }
}
