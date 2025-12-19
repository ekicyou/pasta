//! Comprehensive tests for sakura script compatibility (Task 6.3)
//!
//! Tests Requirements 3.1-3.6:
//! - 3.1: Basic sakura script commands - parsing and IR output
//! - 3.2: Surface switching (`\s[n]`)
//! - 3.3: Wait commands (`\w[n]`, `\_w[n]`)
//! - 3.4: Speaker switching (`\0`, `\1`)
//! - 3.5: Newline commands (`\n`)
//! - 3.6: Custom extension commands
//!
//! Note: Current implementation generates separate Talk events for each content part.
//! This is acceptable for Task 6.1-6.3. Future optimization (separate task) will
//! combine them into single Talk events with multiple ContentParts.

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta::{
    ir::{ContentPart, ScriptEvent},
    PastaEngine,
};

/// Helper function to extract all ContentParts from a sequence of events
fn extract_content_parts(events: &[ScriptEvent]) -> Vec<&ContentPart> {
    events
        .iter()
        .filter_map(|e| match e {
            ScriptEvent::Talk { content, .. } => Some(content.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Test basic sakura script escape parsing with ASCII backslash (Requirement 3.1)
#[test]
fn test_basic_sakura_script_ascii() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：こんにちは\s[0]
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    // Verify we have a ChangeSpeaker event
    assert!(events
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "さくら")));

    // Extract all content parts
    let parts = extract_content_parts(&events);

    // Should have Text + SakuraScript
    assert_eq!(parts.len(), 2);
    assert!(matches!(parts[0], ContentPart::Text(t) if t == "こんにちは"));
    assert!(matches!(parts[1], ContentPart::SakuraScript(s) if s == "s[0]"));

    Ok(())
}

/// Test basic sakura script escape parsing with full-width backslash (Requirement 3.1)
#[test]
fn test_basic_sakura_script_fullwidth() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：こんにちは＼ｓ［０］
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert_eq!(parts.len(), 2);
    assert!(matches!(parts[0], ContentPart::Text(t) if t == "こんにちは"));
    assert!(matches!(parts[1], ContentPart::SakuraScript(s) if s == "ｓ［０］"));

    Ok(())
}

/// Test surface switching command (Requirement 3.2)
#[test]
fn test_surface_switching() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：笑顔です\s[1]
    さくら：怒った顔です\s[2]
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);

    // Should find both surface commands
    let sakura_parts: Vec<_> = parts
        .iter()
        .filter_map(|p| match p {
            ContentPart::SakuraScript(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    assert!(sakura_parts.contains(&"s[1]"), "Expected \\s[1] command");
    assert!(sakura_parts.contains(&"s[2]"), "Expected \\s[2] command");

    Ok(())
}

/// Test wait commands (Requirement 3.3)
#[test]
fn test_wait_commands() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：こんにちは\w8お元気ですか
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);

    // Should have: Text + SakuraScript(w8) + Text
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::Text(t) if t == "こんにちは")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "w8")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::Text(t) if t == "お元気ですか")));

    Ok(())
}

/// Test quick wait command (Requirement 3.3)
#[test]
fn test_quick_wait() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：素早く\_w[50]表示します
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "_w[50]")));

    Ok(())
}

/// Test speaker switching commands (Requirement 3.4)
#[test]
fn test_speaker_switching_commands() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：\0私が話します
    さくら：\1次はうにゅうです
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "0")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "1")));

    Ok(())
}

/// Test newline commands (Requirement 3.5)
#[test]
fn test_newline_commands() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：1行目\n[半分改行]\n[half]もっと半分
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    // Check that we have n[...] style commands
    let sakura_parts: Vec<_> = parts
        .iter()
        .filter_map(|p| match p {
            ContentPart::SakuraScript(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        sakura_parts.iter().any(|&s| s.starts_with("n[")),
        "Expected \\n[...] commands"
    );

    Ok(())
}

/// Test multiple sakura script commands in one line
#[test]
fn test_multiple_commands() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：\s[0]こんにちは\w8元気ですか\s[1]
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    let sakura_parts: Vec<_> = parts
        .iter()
        .filter_map(|p| match p {
            ContentPart::SakuraScript(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    assert!(sakura_parts.contains(&"s[0]"), "Expected \\s[0] command");
    assert!(sakura_parts.contains(&"w8"), "Expected \\w8 command");
    assert!(sakura_parts.contains(&"s[1]"), "Expected \\s[1] command");

    Ok(())
}

/// Test custom extension commands (Requirement 3.6)
#[test]
fn test_custom_commands() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：カスタム\custom[arg1,arg2]コマンドです
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "custom[arg1,arg2]")));

    Ok(())
}

/// Test sakura script across multiple speech lines
#[test]
fn test_sakura_script_multiple_lines() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：最初の行です\s[0]
    さくら：2番目の行です\w5
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "s[0]")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "w5")));

    Ok(())
}

/// Test edge case: sakura script at start of line
#[test]
fn test_sakura_script_at_start() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：\s[0]最初から表情変更
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "s[0]")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::Text(t) if t == "最初から表情変更")));

    Ok(())
}

/// Test edge case: sakura script at end of line
#[test]
fn test_sakura_script_at_end() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：最後に表情変更\s[0]
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::Text(t) if t == "最後に表情変更")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "s[0]")));

    Ok(())
}

/// Test edge case: only sakura script, no text
#[test]
fn test_only_sakura_script() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：\s[0]\w5\n
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "s[0]")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "w5")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "n")));

    Ok(())
}

/// Test complex sakura script with brackets and special characters
#[test]
fn test_complex_sakura_commands() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：\s[10]複雑\![raise,OnTest,arg1,arg2]なコマンド
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "s[10]")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "![raise,OnTest,arg1,arg2]")));

    Ok(())
}

/// Test mixed ASCII and full-width backslashes
#[test]
fn test_mixed_backslashes() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：ASCII\s[0]とfull-width＼ｓ［１］混在
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);
    let sakura_parts: Vec<_> = parts
        .iter()
        .filter_map(|p| match p {
            ContentPart::SakuraScript(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    assert!(sakura_parts.contains(&"s[0]"), "Expected ASCII \\s[0]");
    assert!(
        sakura_parts.contains(&"ｓ［１］"),
        "Expected full-width ＼ｓ［１］"
    );

    Ok(())
}

/// Test that sakura scripts are preserved alongside regular text
#[test]
fn test_sakura_with_text() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：\s[0]こんにちはお元気ですか\w5
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;
    let events = engine.execute_label("test")?;

    let parts = extract_content_parts(&events);

    // Should have sakura scripts and text
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "s[0]")));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::Text(t) if t.contains("こんにちは"))));
    assert!(parts
        .iter()
        .any(|p| matches!(p, ContentPart::SakuraScript(s) if s == "w5")));

    Ok(())
}
