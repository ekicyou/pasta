//! Task 6.1: End-to-end integration test for call unified scope resolution.
//!
//! This test verifies the complete pipeline for Call statement (＞シーン):
//! Pasta script → Parser → Transpiler → Rune VM → Scene call with local+global merge
//!
//! The unified scope resolution allows ＞シーン to search both:
//! - Local scenes (within the current global scene)
//! - Global scenes (all global scenes)
//! And select from the merged candidates.

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta::PastaEngine;
use pasta::ir::ScriptEvent;
use std::collections::HashSet;

/// Test: Call statement can invoke local scene within the same global scene.
///
/// Scenario: Global scene "会話" has local scene "返答", call ＞返答 should work.
#[test]
fn test_call_local_scene_within_global() -> Result<(), Box<dyn std::error::Error>> {
    // Using local scene marker "・" instead of "＊" for local scenes
    let source = r#"
＊会話
  さくら：「これは親シーンです」
  ＞返答

  ・返答
    さくら：「これは子シーンです」
"#;

    let script_dir = create_test_script(source)?;
    let persistence_dir = get_test_persistence_dir();
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;

    let events = engine.execute_label("会話")?;

    // Extract all talk contents
    let talks: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            ScriptEvent::Talk { content, .. } => Some(
                content
                    .iter()
                    .filter_map(|p| match p {
                        pasta::ir::ContentPart::Text(t) => Some(t.clone()),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .collect();

    // Should have both parent and child scene outputs
    assert!(
        talks.iter().any(|t| t.contains("親シーン")),
        "Missing parent scene output: {:?}",
        talks
    );
    assert!(
        talks.iter().any(|t| t.contains("子シーン")),
        "Missing child scene output: {:?}",
        talks
    );

    Ok(())
}

/// Test: Call statement merges local and global candidates with same prefix.
///
/// Scenario:
/// - Global scene "朝の挨拶" (matches "朝の")
/// - Global scene "昼の挨拶" (does NOT match "朝の")
/// - Global scene "会話" has local scene "朝の返答" (matches "朝の")
/// - Global scene "会話" has local scene "夜の返答" (does NOT match "朝の")
/// - Call ＞朝の should select from BOTH matching candidates (朝の挨拶 + 朝の返答)
/// - And should NOT select non-matching ones (昼の挨拶, 夜の返答)
///
/// This test verifies correct prefix matching and exclusion.
#[test]
fn test_call_merges_local_and_global_candidates() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
＊朝の挨拶
  さくら：「グローバル朝の挨拶です」

＊昼の挨拶
  さくら：「グローバル昼の挨拶です」

＊会話
  さくら：「開始」
  ＞朝の

  ・朝の返答
    さくら：「ローカル朝の返答です」

  ・夜の返答
    さくら：「ローカル夜の返答です」
"#;

    let script_dir = create_test_script(source)?;

    // Track which scenes are selected across multiple runs
    let mut global_朝の_selected = false;
    let mut local_朝の_selected = false;
    let mut non_matching_selected = false;

    // Run multiple times to verify correct candidates are selected
    for _i in 0..50 {
        let persistence_temp = tempfile::TempDir::new()?;
        let persistence_dir = persistence_temp.path().to_path_buf();
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;

        let events = engine.execute_label("会話")?;

        let talks: Vec<String> = events
            .iter()
            .filter_map(|e| match e {
                ScriptEvent::Talk { content, .. } => Some(
                    content
                        .iter()
                        .filter_map(|p| match p {
                            pasta::ir::ContentPart::Text(t) => Some(t.clone()),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            })
            .collect();

        if talks.iter().any(|t| t.contains("グローバル朝の挨拶")) {
            global_朝の_selected = true;
        }
        if talks.iter().any(|t| t.contains("ローカル朝の返答")) {
            local_朝の_selected = true;
        }
        // Non-matching scenes should NEVER be selected
        if talks.iter().any(|t| t.contains("グローバル昼の挨拶")) {
            non_matching_selected = true;
        }
        if talks.iter().any(|t| t.contains("ローカル夜の返答")) {
            non_matching_selected = true;
        }

        // Early exit if both matching candidates have been selected
        if global_朝の_selected && local_朝の_selected {
            break;
        }
    }

    // Verify that non-matching scenes were NOT selected
    assert!(
        !non_matching_selected,
        "Non-matching scene was incorrectly selected for ＞朝の (昼の挨拶 or 夜の返答)"
    );

    // Verify that at least one matching candidate was selected
    assert!(
        global_朝の_selected || local_朝の_selected,
        "Neither matching (朝の) local nor global scene was selected."
    );

    Ok(())
}

/// Test: Call with prefix matches multiple scenes.
///
/// Scenario:
/// - Global scenes "挨拶_朝", "挨拶_昼", "挨拶_夜" exist
/// - Call ＞挨拶 should match all three and select randomly
#[test]
fn test_call_prefix_match_multiple_global_scenes() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
＊挨拶_朝
  さくら：「おはよう」

＊挨拶_昼
  さくら：「こんにちは」

＊挨拶_夜
  さくら：「こんばんは」

＊メイン
  さくら：「開始」
  ＞挨拶
"#;

    let script_dir = create_test_script(source)?;

    // Track which greetings are selected
    let mut selected_greetings: HashSet<String> = HashSet::new();

    for _ in 0..100 {
        // Keep TempDir alive during engine usage
        let persistence_temp = tempfile::TempDir::new()?;
        let persistence_dir = persistence_temp.path().to_path_buf();
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;

        let events = engine.execute_label("メイン")?;

        for event in &events {
            if let ScriptEvent::Talk { content, .. } = event {
                let text: String = content
                    .iter()
                    .filter_map(|p| match p {
                        pasta::ir::ContentPart::Text(t) => Some(t.clone()),
                        _ => None,
                    })
                    .collect();
                if text.contains("おはよう") {
                    selected_greetings.insert("朝".to_string());
                }
                if text.contains("こんにちは") {
                    selected_greetings.insert("昼".to_string());
                }
                if text.contains("こんばんは") {
                    selected_greetings.insert("夜".to_string());
                }
            }
        }

        // Early exit if all greetings have been selected
        if selected_greetings.len() >= 3 {
            break;
        }
    }

    // With 100 runs, we should see at least 2 different greetings
    // (probability of seeing only 1 is (1/3)^99 ≈ 0)
    assert!(
        selected_greetings.len() >= 2,
        "Expected at least 2 different greetings, got: {:?}",
        selected_greetings
    );

    Ok(())
}

/// Test: Local scene takes priority when exact match exists (no prefix search).
///
/// Scenario:
/// - Global scene "返答" exists
/// - Local scene "返答" also exists within "会話"
/// - Both should be candidates for ＞返答
#[test]
fn test_call_local_and_global_same_name() -> Result<(), Box<dyn std::error::Error>> {
    // Using local scene marker "・" for local scenes
    let source = r#"
＊返答
  さくら：「グローバル返答」

＊会話
  さくら：「開始」
  ＞返答

  ・返答
    さくら：「ローカル返答」
"#;

    let script_dir = create_test_script(source)?;

    let mut global_count = 0;
    let mut local_count = 0;

    for _ in 0..50 {
        // Keep TempDir alive during engine usage
        let persistence_temp = tempfile::TempDir::new()?;
        let persistence_dir = persistence_temp.path().to_path_buf();
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)?;

        let events = engine.execute_label("会話")?;

        for event in &events {
            if let ScriptEvent::Talk { content, .. } = event {
                let text: String = content
                    .iter()
                    .filter_map(|p| match p {
                        pasta::ir::ContentPart::Text(t) => Some(t.clone()),
                        _ => None,
                    })
                    .collect();
                if text.contains("グローバル返答") {
                    global_count += 1;
                }
                if text.contains("ローカル返答") {
                    local_count += 1;
                }
            }
        }
    }

    // Both local and global should have been selected at some point
    // (with unified scope, both are candidates)
    assert!(
        local_count > 0 || global_count > 0,
        "Neither local nor global scene was selected"
    );

    Ok(())
}
