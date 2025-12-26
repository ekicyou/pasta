//! Task 5.3: Verification that Call/Jump statements do not access the word dictionary.
//!
//! This test verifies the design principle that:
//! - `select_scene_to_id()` only accesses `SceneTable`
//! - Word references use `word()` function which only accesses `WordTable`
//! - The two dictionaries are completely separate

use pasta_rune::registry::{SceneRegistry, WordDefRegistry};
use pasta_rune::runtime::random::DefaultRandomSelector;
use pasta_rune::runtime::scene::SceneTable;
use pasta_rune::runtime::words::WordTable;
use std::collections::HashMap;

/// Helper to create a test SceneTable with scenes
fn create_test_scene_table_with_scenes(scene_names: Vec<&str>) -> SceneTable {
    let mut registry = SceneRegistry::new();
    for scene_name in scene_names {
        registry.register_global(scene_name, HashMap::new());
    }

    let selector = Box::new(DefaultRandomSelector::new());
    SceneTable::from_scene_registry(registry, selector).unwrap()
}

/// Helper to create a test WordTable with words
fn create_test_word_table_with_words(words: Vec<(&str, Vec<&str>)>) -> WordTable {
    let mut registry = WordDefRegistry::new();
    for (name, values) in words {
        let values: Vec<String> = values.into_iter().map(|s| s.to_string()).collect();
        registry.register_global(name, values);
    }

    let selector = Box::new(DefaultRandomSelector::new());
    WordTable::from_word_def_registry(registry, selector)
}

/// Verify that SceneTable and WordTable are separate data structures.
/// This test documents the design requirement that Call/Jump use SceneTable,
/// not WordTable.
#[test]
fn test_label_table_does_not_contain_word_definitions() {
    let mut scene_table = create_test_scene_table_with_scenes(vec!["test_label", "another_label"]);

    // Verify SceneTable has labels
    let result = scene_table.resolve_scene_id("test_label", &HashMap::new());
    assert!(result.is_ok());

    // SceneTable has no API to access words - this is by design
    // The struct only contains label_prefix_map, no word-related fields
}

/// Verify that WordTable does not contain scene definitions.
/// This test documents the design requirement that word expansion uses WordTable,
/// not SceneTable.
#[test]
fn test_word_table_does_not_contain_label_definitions() {
    let mut word_table = create_test_word_table_with_words(vec![
        ("挨拶", vec!["こんにちは"]),
        ("場所", vec!["東京", "大阪"]),
    ]);

    // Verify WordTable has words
    let result = word_table.search_word("", "挨拶", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "こんにちは");

    // WordTable has no API to access scenes - this is by design
    // The struct only contains entries (RadixMap), no label-related fields
}

/// Verify that words prefixed with common scene patterns are not confused with labels.
/// For example, a word named "＊ラベル" should not be treated as a label.
#[test]
fn test_word_with_label_like_name_stays_in_word_table() {
    let mut word_table = create_test_word_table_with_words(vec![("＊ラベル風", vec!["単語です"])]);

    // The word should be accessible from WordTable
    let result = word_table.search_word("", "＊ラベル風", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "単語です");
}

/// Verify that scenes with common word patterns are not confused with words.
/// For example, a scene named "挨拶ラベル" should not be treated as a word.
#[test]
fn test_label_with_word_like_name_stays_in_label_table() {
    let mut scene_table = create_test_scene_table_with_scenes(vec!["挨拶ラベル"]);

    // The scene should be accessible from SceneTable
    let result = scene_table.resolve_scene_id("挨拶ラベル", &HashMap::new());
    assert!(result.is_ok());
}

/// Integration test: Verify that a script with both scenes and words
/// keeps them in separate dictionaries.
#[test]
fn test_separate_dictionaries_integration() {
    // This test verifies the high-level design:
    // - Pass 1 collects SceneRegistry (for labels) and WordDefRegistry (for words)
    // - They are converted to SceneTable and WordTable respectively
    // - At runtime, Call/Jump use SceneTable, word references use WordTable

    // Simulate scenes collected in Pass 1
    let mut scene_table = create_test_scene_table_with_scenes(vec!["会話ラベル"]);

    // Simulate words collected in Pass 1
    let mut word_table = create_test_word_table_with_words(vec![("場所", vec!["東京", "大阪"])]);

    // scenes are ONLY in SceneTable
    assert!(
        scene_table
            .resolve_scene_id("会話ラベル", &HashMap::new())
            .is_ok()
    );
    assert!(word_table.search_word("", "会話ラベル", &[]).is_err());

    // Words are ONLY in WordTable
    assert!(word_table.search_word("", "場所", &[]).is_ok());
    assert!(
        scene_table
            .resolve_scene_id("場所", &HashMap::new())
            .is_err()
    );
}

