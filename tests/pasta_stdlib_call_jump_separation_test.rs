//! Task 5.3: Verification that Call/Jump statements do not access the word dictionary.
//!
//! This test verifies the design principle that:
//! - `select_label_to_id()` only accesses `LabelTable`
//! - Word references use `word()` function which only accesses `WordTable`
//! - The two dictionaries are completely separate

use pasta::runtime::labels::LabelTable;
use pasta::runtime::random::DefaultRandomSelector;
use pasta::runtime::words::WordTable;
use pasta::transpiler::{LabelRegistry, WordDefRegistry};
use std::collections::HashMap;

/// Helper to create a test LabelTable with labels
fn create_test_label_table_with_labels(labels: Vec<&str>) -> LabelTable {
    let mut registry = LabelRegistry::new();
    for label in labels {
        registry.register_global(label, HashMap::new());
    }
    
    let selector = Box::new(DefaultRandomSelector::new());
    LabelTable::from_label_registry(registry, selector).unwrap()
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

/// Verify that LabelTable and WordTable are separate data structures.
/// This test documents the design requirement that Call/Jump use LabelTable,
/// not WordTable.
#[test]
fn test_label_table_does_not_contain_word_definitions() {
    let mut label_table = create_test_label_table_with_labels(vec!["test_label", "another_label"]);

    // Verify LabelTable has labels
    let result = label_table.resolve_label_id("test_label", &HashMap::new());
    assert!(result.is_ok());

    // LabelTable has no API to access words - this is by design
    // The struct only contains label_prefix_map, no word-related fields
}

/// Verify that WordTable does not contain label definitions.
/// This test documents the design requirement that word expansion uses WordTable,
/// not LabelTable.
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

    // WordTable has no API to access labels - this is by design
    // The struct only contains entries (RadixMap), no label-related fields
}

/// Verify that words prefixed with common label patterns are not confused with labels.
/// For example, a word named "＊ラベル" should not be treated as a label.
#[test]
fn test_word_with_label_like_name_stays_in_word_table() {
    let mut word_table = create_test_word_table_with_words(vec![
        ("＊ラベル風", vec!["単語です"]),
    ]);

    // The word should be accessible from WordTable
    let result = word_table.search_word("", "＊ラベル風", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "単語です");
}

/// Verify that labels with common word patterns are not confused with words.
/// For example, a label named "挨拶ラベル" should not be treated as a word.
#[test]
fn test_label_with_word_like_name_stays_in_label_table() {
    let mut label_table = create_test_label_table_with_labels(vec!["挨拶ラベル"]);

    // The label should be accessible from LabelTable
    let result = label_table.resolve_label_id("挨拶ラベル", &HashMap::new());
    assert!(result.is_ok());
}

/// Integration test: Verify that a script with both labels and words
/// keeps them in separate dictionaries.
#[test]
fn test_separate_dictionaries_integration() {
    // This test verifies the high-level design:
    // - Pass 1 collects LabelRegistry (for labels) and WordDefRegistry (for words)
    // - They are converted to LabelTable and WordTable respectively
    // - At runtime, Call/Jump use LabelTable, word references use WordTable

    // Simulate labels collected in Pass 1
    let mut label_table = create_test_label_table_with_labels(vec!["会話ラベル"]);

    // Simulate words collected in Pass 1
    let mut word_table = create_test_word_table_with_words(vec![
        ("場所", vec!["東京", "大阪"]),
    ]);

    // Labels are ONLY in LabelTable
    assert!(label_table.resolve_label_id("会話ラベル", &HashMap::new()).is_ok());
    assert!(word_table.search_word("", "会話ラベル", &[]).is_err());

    // Words are ONLY in WordTable
    assert!(word_table.search_word("", "場所", &[]).is_ok());
    assert!(label_table.resolve_label_id("場所", &HashMap::new()).is_err());
}
