//! End-to-end test for runtime label resolution with actual SceneTable

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta::PastaEngine;

#[test]
fn test_label_resolution_with_prefix_match() {
    // Test prefix matching: "会話" should match "会話_1::__start__"
    let script = r#"
＊会話
    さくら：こんにちは
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // This should execute via scene_selector -> select_scene_to_id -> resolve_label_id
    let result = engine.execute_label("会話");
    assert!(
        result.is_ok(),
        "Label resolution should work with prefix match"
    );

    let events = result.unwrap();
    assert!(
        !events.is_empty(),
        "Should have events from label execution"
    );
}

#[test]
fn test_label_resolution_with_multiple_labels() {
    // Test multiple labels with same name (random selection)
    let script = r#"
＊挨拶
    さくら：おはよう

＊挨拶
    さくら：こんにちは

＊挨拶
    さくら：こんばんは
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // Execute multiple times - should potentially get different labels
    for _ in 0..5 {
        let result = engine.execute_label("挨拶");
        assert!(
            result.is_ok(),
            "Label resolution should work with multiple labels"
        );
    }
}

#[test]
fn test_label_resolution_sequential_consumption() {
    // Test sequential label consumption (no repeat until exhausted)
    let script = r#"
＊カウント
    さくら：1回目

＊カウント
    さくら：2回目

＊カウント
    さくら：3回目
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // Note: Current execute_label implementation doesn't use the full resolution
    // This test documents the expected behavior for when full integration is complete
    let result = engine.execute_label("カウント");
    assert!(result.is_ok(), "First label should execute");
}
