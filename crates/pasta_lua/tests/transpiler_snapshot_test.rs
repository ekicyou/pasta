//! Transpiler Snapshot Tests (Golden Tests)
//!
//! Uses insta crate to verify Lua output against stored snapshots.
//! This ensures transpiler output stability across changes.

mod common;

use common::e2e_helpers::transpile;
use insta::assert_snapshot;
use std::fs;
use std::path::PathBuf;

/// Get the path to test fixtures
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

// ============================================================================
// Basic Scene Snapshots
// ============================================================================

/// Minimal scene snapshot
#[test]
fn test_minimal_scene_snapshot() {
    let source = r#"
＊メイン
  さくら：「こんにちは」
"#;

    let lua_code = transpile(source);
    assert_snapshot!("minimal_scene", lua_code);
}

/// Scene with word reference snapshot
#[test]
fn test_scene_with_word_snapshot() {
    let source = r#"
＠挨拶：おはよう、こんにちは、こんばんは

＊メイン
  さくら：「＠挨拶！」
"#;

    let lua_code = transpile(source);
    assert_snapshot!("scene_with_word", lua_code);
}

/// Scene with variables snapshot
#[test]
fn test_scene_with_variables_snapshot() {
    let source = r#"
＊メイン
  ＄カウンタ＝「10」
  ＄＊グローバル＝「永続値」
  さくら：「カウンタは＄カウンタ、グローバルは＄＊グローバル」
"#;

    let lua_code = transpile(source);
    assert_snapshot!("scene_with_variables", lua_code);
}

/// Scene with Call statement snapshot
#[test]
fn test_scene_with_call_snapshot() {
    let source = r#"
＊メイン
  さくら：「サブルーチンを呼びます」
  ＞サブ

  ・サブ
    うにゅう：「サブルーチンです」
"#;

    let lua_code = transpile(source);
    assert_snapshot!("scene_with_call", lua_code);
}

/// Scene with attributes snapshot
#[test]
fn test_scene_with_attributes_snapshot() {
    let source = r#"
＆天気：晴れ
＆場所：東京

＊メイン
  ＆時間帯：朝
  さくら：「今日の天気は晴れです」
"#;

    let lua_code = transpile(source);
    assert_snapshot!("scene_with_attributes", lua_code);
}

// ============================================================================
// Multiple Scenes Snapshot
// ============================================================================

/// Multiple scenes with same prefix snapshot
#[test]
fn test_multiple_scenes_snapshot() {
    let source = r#"
＊挨拶
  さくら：「おはようございます」

＊挨拶
  さくら：「こんにちは」

＊挨拶朝
  さくら：「おはよう！朝だね」

＊メイン
  ＞挨拶
"#;

    let lua_code = transpile(source);
    assert_snapshot!("multiple_scenes", lua_code);
}

// ============================================================================
// Actor Word Definition Snapshot
// ============================================================================

/// Actor with word definitions snapshot
#[test]
fn test_actor_word_definition_snapshot() {
    let source = r#"
％さくら
  ＠一人称：私、わたし、あたし

％うにゅう
  ＠一人称：僕、ぼく

＊メイン
  さくら：「＠一人称は元気です」
  うにゅう：「＠一人称も元気だよ」
"#;

    let lua_code = transpile(source);
    assert_snapshot!("actor_word_definition", lua_code);
}

// ============================================================================
// Tail Call Optimization Snapshot
// ============================================================================

/// Tail call vs normal call distinction snapshot
#[test]
fn test_tail_call_optimization_snapshot() {
    let fixture_path = fixtures_path().join("tail_call_optimization.pasta");
    let source = fs::read_to_string(&fixture_path).expect("Failed to read fixture file");

    let lua_code = transpile(&source);
    assert_snapshot!("tail_call_optimization", lua_code);
}
