//! Tests for PastaEngine instance independence.
//!
//! These tests verify that multiple PastaEngine instances are completely independent
//! and do not share state with each other.

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta::{PastaEngine, ScriptEvent};

#[test]
fn test_independent_execution() {
    // Test that two engines can run different scripts independently
    let script1 = r#"
＊test1
    さくら：エンジン1
"#;
    let script2 = r#"
＊test2
    うにゅう：エンジン2
"#;

    let script_dir1 = create_test_script(script1).expect("Failed to create script1");
    let script_dir2 = create_test_script(script2).expect("Failed to create script2");
    let persistence_dir = get_test_persistence_dir();

    let mut engine1 =
        PastaEngine::new(&script_dir1, &persistence_dir).expect("Failed to create engine1");
    let mut engine2 =
        PastaEngine::new(&script_dir2, &persistence_dir).expect("Failed to create engine2");

    // Execute both engines
    let events1 = engine1
        .execute_label("test1")
        .expect("Failed to execute test1");
    let events2 = engine2
        .execute_label("test2")
        .expect("Failed to execute test2");

    // Verify engine1 has さくら speaker
    let has_sakura = events1
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "さくら"));
    assert!(has_sakura, "Engine1 should have さくら speaker");

    // Verify engine2 has うにゅう speaker
    let has_unyuu = events2
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "うにゅう"));
    assert!(has_unyuu, "Engine2 should have うにゅう speaker");

    // Verify engines don't interfere with each other by executing labels
    assert!(engine1.execute_label("test1").is_ok());
    assert!(engine1.execute_label("test2").is_err());
    assert!(engine2.execute_label("test2").is_ok());
    assert!(engine2.execute_label("test1").is_err());
}

#[test]
fn test_global_variable_isolation() {
    // Test that global variables are independent between engines
    // Note: Currently PastaEngine doesn't expose variable manipulation directly,
    // but the internal VariableManager should be independent per instance.
    // This test verifies structural independence.

    let script = r#"
＊test
    さくら：テスト
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();

    let mut engine1 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine1");
    let mut engine2 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine2");

    // Both engines should execute independently
    let events1 = engine1.execute_label("test").expect("Failed on engine1");
    let events2 = engine2.execute_label("test").expect("Failed on engine2");

    // Both should succeed and produce events
    assert!(!events1.is_empty());
    assert!(!events2.is_empty());
}

#[test]
fn test_independent_parsing() {
    // Test that multiple engines can parse the same script independently
    let script = r#"
＊greeting
    さくら：こんにちは
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();

    // Create three engines from the same script
    let mut engine1 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine1");
    let mut engine2 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine2");
    let mut engine3 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine3");

    // All should be able to execute independently
    let events1 = engine1.execute_label("greeting").unwrap();
    let events2 = engine2.execute_label("greeting").unwrap();
    let events3 = engine3.execute_label("greeting").unwrap();

    // All should produce the same output
    assert!(!events1.is_empty());
    assert!(!events2.is_empty());
    assert!(!events3.is_empty());
}

#[test]
fn test_random_selector_independence() {
    // Test that each engine has its own RandomSelector
    let script = r#"
＊choice
    さくら：選択肢1
    
＊choice
    さくら：選択肢2
    
＊choice
    さくら：選択肢3
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();

    let mut engine1 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine1");
    let mut engine2 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine2");

    // Execute multiple times - each engine should be able to select randomly
    for _ in 0..5 {
        let events1 = engine1.execute_label("choice").unwrap();
        let events2 = engine2.execute_label("choice").unwrap();

        // Both should produce valid events
        assert!(!events1.is_empty());
        assert!(!events2.is_empty());
    }
}

#[test]
fn test_drop_independence() {
    // Test that dropping one engine doesn't affect another
    let script1 = r#"
＊test1
    さくら：エンジン1
"#;
    let script2 = r#"
＊test2
    うにゅう：エンジン2
"#;

    let script_dir1 = create_test_script(script1).expect("Failed to create script1");
    let script_dir2 = create_test_script(script2).expect("Failed to create script2");
    let persistence_dir = get_test_persistence_dir();

    let mut engine1 =
        PastaEngine::new(&script_dir1, &persistence_dir).expect("Failed to create engine1");
    let mut engine2 =
        PastaEngine::new(&script_dir2, &persistence_dir).expect("Failed to create engine2");

    // Verify both work
    assert!(engine1.execute_label("test1").is_ok());
    assert!(engine2.execute_label("test2").is_ok());

    // Drop engine1
    drop(engine1);

    // Engine2 should still work
    let events = engine2
        .execute_label("test2")
        .expect("Engine2 should still work after engine1 is dropped");
    assert!(!events.is_empty());
}

#[test]
fn test_concurrent_parsing() {
    // Test that multiple engines can be created with the same script simultaneously
    let script = r#"
＊test
    さくら：こんにちは
    うにゅう：やあ
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();

    // Create multiple engines at the same time
    let engines: Vec<_> = (0..10)
        .map(|_| PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine"))
        .collect();

    // All should be valid
    assert_eq!(engines.len(), 10);
    // All engines are valid (verified by successful construction)
}

#[test]
fn test_independent_label_execution() {
    // Test that label execution state doesn't leak between engines
    let script = r#"
＊label_a
    さくら：ラベルA

＊label_b
    うにゅう：ラベルB
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();

    let mut engine1 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine1");
    let mut engine2 =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine2");

    // Execute different labels on each engine
    let events1_a = engine1.execute_label("label_a").unwrap();
    let events2_b = engine2.execute_label("label_b").unwrap();

    // Verify correct speakers
    let has_sakura = events1_a
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "さくら"));
    let has_unyuu = events2_b
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "うにゅう"));

    assert!(has_sakura, "Engine1 should execute label_a with さくら");
    assert!(has_unyuu, "Engine2 should execute label_b with うにゅう");

    // Now execute opposite labels
    let events1_b = engine1.execute_label("label_b").unwrap();
    let events2_a = engine2.execute_label("label_a").unwrap();

    // Both engines should be able to execute both labels independently
    assert!(!events1_b.is_empty());
    assert!(!events2_a.is_empty());
}

#[test]
fn test_engine_with_different_scripts() {
    // Test engines with completely different script structures
    let simple_script = r#"
＊simple
    さくら：シンプル
"#;

    let complex_script = r#"
＊complex
    さくら：複雑なスクリプト
    うにゅう：ネストあり
    
    ＊local
        さくら：ローカルラベル
"#;

    let simple_dir = create_test_script(simple_script).expect("Failed to create simple script");
    let complex_dir = create_test_script(complex_script).expect("Failed to create complex script");
    let persistence_dir = get_test_persistence_dir();

    let mut simple_engine =
        PastaEngine::new(&simple_dir, &persistence_dir).expect("Failed to create simple engine");
    let mut complex_engine =
        PastaEngine::new(&complex_dir, &persistence_dir).expect("Failed to create complex engine");

    // Both should work independently
    assert!(simple_engine.execute_label("simple").is_ok());
    assert!(complex_engine.execute_label("complex").is_ok());

    // Verify label isolation by executing
    assert!(simple_engine.execute_label("simple").is_ok());
    assert!(simple_engine.execute_label("complex").is_err());
    assert!(complex_engine.execute_label("complex").is_ok());
    assert!(complex_engine.execute_label("simple").is_err());
}
