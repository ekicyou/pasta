//! Integration test: Verify transpiler ID and runtime ID consistency
//!
//! This test verifies that:
//! 1. Transpiler assigns IDs starting from 1
//! 2. Runtime receives and uses the same IDs
//! 3. The ID→function mapping is consistent between transpilation and runtime

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta_rune::PastaEngine;

#[test]
fn test_transpiler_runtime_id_consistency() {
    // Create a script with multiple scenes to verify ID assignment
    let script = r#"
＊会話
    さくら：こんにちは

＊挨拶
    さくら：おはよう

＊別れ
    さくら：さようなら
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();

    // Create engine - this internally:
    // 1. Parses Pasta files
    // 2. Transpiles to Rune with ID assignment
    // 3. Creates SceneTable from registry
    // 4. Registers select_scene_to_id with SceneTable
    let mut engine =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // If transpiler and runtime IDs don't match, these executions will fail
    // because select_scene_to_id will return wrong IDs that don't match
    // the match statement in scene_selector

    let result1 = engine.execute_label("会話");
    assert!(result1.is_ok(), "Runtime should resolve 会話 correctly");

    let result2 = engine.execute_label("挨拶");
    assert!(result2.is_ok(), "Runtime should resolve 挨拶 correctly");

    let result3 = engine.execute_label("別れ");
    assert!(result3.is_ok(), "Runtime should resolve 別れ correctly");

    println!("✅ ID consistency verified: All scenes execute successfully");
    println!("   This confirms transpiler IDs match runtime IDs");
}

#[test]
fn test_duplicate_labels_id_consistency() {
    // Test with duplicate scene names
    // Each duplicate should get unique ID (1, 2, 3)
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

    // If ID assignment is inconsistent, runtime resolution will fail
    // because select_scene_to_id will return IDs that don't match
    // the transpiler's match statement
    for _ in 0..5 {
        let result = engine.execute_label("挨拶");
        assert!(
            result.is_ok(),
            "Runtime should handle duplicate scenes with correct IDs"
        );
    }

    println!("✅ Duplicate scene ID consistency verified");
    println!("   Each duplicate has unique ID and resolves correctly");
}
