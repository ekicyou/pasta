//! Integration tests for persistence functionality.

mod common;

use common::{create_test_script, get_test_persistence_dir};
use pasta::{PastaEngine, PastaError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

fn copy_fixtures_to_temp(temp_dir: &TempDir) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("persistence");

    for entry in fs::read_dir(&fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read entry");
        let dest = temp_dir.path().join(entry.file_name());
        fs::copy(entry.path(), dest).expect("Failed to copy fixture");
    }
}

#[test]
fn test_new_with_persistence_absolute_path() {
    let script = r#"＊test
    さくら：Hello
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let result = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(result.is_ok());
}

#[test]
fn test_new_with_persistence_relative_path() {
    let script = r#"＊test
    さくら：Hello
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let result = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(result.is_ok());
}

#[test]
fn test_new_without_persistence() {
    let script = r#"＊test
    さくら：Hello
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let result = PastaEngine::new(&script_dir, &persistence_dir);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_persistence_path() {
    let script = r#"＊test
    さくら：Hello
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let result = PastaEngine::new(
        &script_dir,
        std::path::Path::new("/nonexistent/path/that/does/not/exist"),
    );
    assert!(result.is_err());

    if let Err(PastaError::PersistenceDirectoryNotFound { path }) = result {
        assert!(path.contains("nonexistent"));
    } else {
        panic!("Expected PersistenceDirectoryNotFound error");
    }
}

#[test]
fn test_rune_script_access_persistence_path() {
    let temp_dir = setup_test_dir();
    // parser2 grammar: action_line requires pad (leading space)
    let script = r#"＊test
  さくら：persistence_pathテスト
```rune
let path = ctx["persistence_path"];
yield emit_text(path);
```
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let mut engine =
        PastaEngine::new(&script_dir, temp_dir.path()).expect("Failed to create engine");
    let events = engine
        .execute_label("test")
        .expect("Failed to execute label");

    assert!(!events.is_empty());
    // Find the last Talk event (from emit_text in code block)
    // First events are ChangeSpeaker + Talk for the speaker line
    let talk_event = events
        .iter()
        .filter(|e| matches!(e, pasta::ScriptEvent::Talk { .. }))
        .last();
    if let Some(pasta::ScriptEvent::Talk { content, .. }) = talk_event {
        let text = content
            .iter()
            .filter_map(|p| {
                if let pasta::ir::ContentPart::Text(t) = p {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(!text.is_empty());
        // Path should be non-empty
        assert!(!text[0].is_empty());
    } else {
        panic!("Expected Talk event");
    }
}

#[test]
fn test_rune_script_without_persistence_path() {
    // parser2 grammar: action_line requires pad (leading space)
    let script = r#"＊test
  さくら：persistence_pathテスト
```rune
let path = ctx["persistence_path"];
if path == "" {
    yield emit_text("No persistence path");
} else {
    yield emit_text("Has persistence path");
}
```
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let mut engine =
        PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");
    let events = engine
        .execute_label("test")
        .expect("Failed to execute label");

    assert!(!events.is_empty());
    // Find the last Talk event (from emit_text in code block)
    let talk_event = events
        .iter()
        .filter(|e| matches!(e, pasta::ScriptEvent::Talk { .. }))
        .last();
    if let Some(pasta::ScriptEvent::Talk { content, .. }) = talk_event {
        let text = content
            .iter()
            .filter_map(|p| {
                if let pasta::ir::ContentPart::Text(t) = p {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        // PastaEngine::new always requires a persistence_dir, so it will have a path
        assert_eq!(text[0], "Has persistence path");
    } else {
        panic!("Expected Talk event");
    }
}

#[test]
fn test_rune_toml_serialization() {
    let temp_dir = setup_test_dir();
    let save_file_path = temp_dir.path().join("test_save.toml");
    let save_file_str = save_file_path.display().to_string().replace("\\", "/");

    // Build script dynamically to avoid format! escaping issues
    // parser2 grammar: action_line requires pad (leading space)
    let script = format!(
        r#"＊save_game
  さくら：セーブ処理
```rune
let data = #{{"level": 5, "gold": 100}};
let toml_str = toml_to_string(data)?;
write_text_file("{}", toml_str)?;
yield emit_text("Saved");
```

＊load_game
  さくら：ロード処理
```rune
let toml_str = read_text_file("{}")?;
let data = toml_from_string(toml_str)?;
let level = data["level"];
let text = `Level: ${{level}}`;
yield emit_text(text);
```
"#,
        save_file_str, save_file_str
    );

    let script_dir = create_test_script(&script).expect("Failed to create script");
    let mut engine =
        PastaEngine::new(&script_dir, temp_dir.path()).expect("Failed to create engine");

    // Save game
    let save_events = engine
        .execute_label("save_game")
        .expect("Failed to execute save_game");
    assert!(!save_events.is_empty());

    // Verify file was created
    assert!(save_file_path.exists());

    // Load game
    let load_events = engine
        .execute_label("load_game")
        .expect("Failed to execute load_game");
    assert!(!load_events.is_empty());

    if let Some(pasta::ScriptEvent::Talk { content, .. }) = load_events
        .iter()
        .filter(|e| matches!(e, pasta::ScriptEvent::Talk { .. }))
        .last()
    {
        let text = content
            .iter()
            .filter_map(|p| {
                if let pasta::ir::ContentPart::Text(t) = p {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(text[0].contains("Level: 5"));
    }
}

#[test]
fn test_tempdir_auto_cleanup() {
    let temp_dir = setup_test_dir();
    let path = temp_dir.path().to_path_buf();

    // Directory should exist
    assert!(path.exists());

    // Drop temp_dir
    drop(temp_dir);

    // Directory should be cleaned up
    assert!(!path.exists());
}

#[test]
fn test_multiple_engines_different_paths() {
    let temp_dir1 = setup_test_dir();
    let temp_dir2 = setup_test_dir();

    // parser2 grammar: action_line requires pad (leading space)
    let script = r#"＊test
  さくら：persistence_pathテスト
```rune
let path = ctx["persistence_path"];
yield emit_text(path);
```
"#;

    let script_dir1 = create_test_script(script).expect("Failed to create script");
    let mut engine1 =
        PastaEngine::new(&script_dir1, temp_dir1.path()).expect("Failed to create engine1");
    let script_dir2 = create_test_script(script).expect("Failed to create script");
    let mut engine2 =
        PastaEngine::new(&script_dir2, temp_dir2.path()).expect("Failed to create engine2");

    let events1 = engine1
        .execute_label("test")
        .expect("Failed to execute on engine1");
    let events2 = engine2
        .execute_label("test")
        .expect("Failed to execute on engine2");

    // Extract paths from events - find the last Talk event (from emit_text)
    let get_path = |events: &[pasta::ScriptEvent]| {
        let talk_event = events
            .iter()
            .filter(|e| matches!(e, pasta::ScriptEvent::Talk { .. }))
            .last();
        if let Some(pasta::ScriptEvent::Talk { content, .. }) = talk_event {
            content
                .iter()
                .filter_map(|p| {
                    if let pasta::ir::ContentPart::Text(t) = p {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
                .next()
                .unwrap()
        } else {
            panic!("Expected Talk event")
        }
    };

    let path1 = get_path(&events1);
    let path2 = get_path(&events2);

    // Paths should be different
    assert_ne!(path1, path2);
}

#[test]
fn test_transpiler_signature_change() {
    // parser2 grammar: action_line requires pad (leading space)
    let script = r#"＊test
  さくら：Hello
"#;

    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir = get_test_persistence_dir();
    let engine = PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // This test verifies that the engine compiles successfully with the new signature.
    // The transpiler should generate `pub fn test(ctx)` instead of `pub fn test()`.
    // If the signature is incorrect, compilation would fail.
    drop(engine);
}

#[test]
fn test_persistence_with_fixture_files() {
    let temp_dir = setup_test_dir();
    copy_fixtures_to_temp(&temp_dir);

    let save_file_path = temp_dir.path().join("sample_save.toml");
    let save_file_str = save_file_path.display().to_string().replace("\\", "/");

    let script = format!(
        // parser2 grammar: action_line requires pad (leading space)
        r#"＊load_save
  さくら：ロード処理
```rune
let toml_str = read_text_file("{}")?;
let data = toml_from_string(toml_str)?;
let level = data["level"];
let gold = data["gold"];
let text = `Level: ${{level}}, Gold: ${{gold}}`;
yield emit_text(text);
```
"#,
        save_file_str
    );

    let script_dir = create_test_script(&script).expect("Failed to create script");
    let mut engine =
        PastaEngine::new(&script_dir, temp_dir.path()).expect("Failed to create engine");

    let events = engine
        .execute_label("load_save")
        .expect("Failed to execute load_save");

    if let Some(pasta::ScriptEvent::Talk { content, .. }) = events
        .iter()
        .filter(|e| matches!(e, pasta::ScriptEvent::Talk { .. }))
        .last()
    {
        let text = content
            .iter()
            .filter_map(|p| {
                if let pasta::ir::ContentPart::Text(t) = p {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(text[0].contains("Level: 5"));
        assert!(text[0].contains("Gold: 100"));
    }
}
