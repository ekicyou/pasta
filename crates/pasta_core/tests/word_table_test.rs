//! 単語テーブルの外部テスト
//!

use pasta_core::WordTableError;
use pasta_core::registry::{MockRandomSelector, WordDefRegistry, WordTable};

fn create_test_registry() -> WordDefRegistry {
    let mut registry = WordDefRegistry::new();
    registry.register_global(
        "挨拶",
        vec!["こんにちは".to_string(), "おはよう".to_string()],
    );
    registry.register_global("場所", vec!["東京".to_string()]);
    registry.register_global("場所_日本", vec!["大阪".to_string(), "京都".to_string()]);
    registry.register_local("会話_1", "挨拶", vec!["やあ".to_string()]);
    registry.register_local("会話_1", "場所", vec!["ここ".to_string()]);
    registry
}

#[test]
fn test_from_word_def_registry() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = WordTable::from_word_def_registry(registry, selector);

    // 5 つのキーがどれも引ける
    for (module, key) in [
        ("", "挨拶"),
        ("", "場所"),
        ("", "場所_日本"),
        ("会話_1", "挨拶"),
        ("会話_1", "場所"),
    ] {
        assert!(
            table.collect_word_candidates(module, key).is_ok(),
            "key not found: {module}:{key}"
        );
    }
}

#[test]
fn test_collect_word_candidates_global_exact() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    let candidates = table.collect_word_candidates("", "挨拶").unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&"こんにちは".to_string()));
    assert!(candidates.contains(&"おはよう".to_string()));
}

#[test]
fn test_collect_word_candidates_global_prefix() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    let candidates = table.collect_word_candidates("", "場所").unwrap();
    assert_eq!(candidates.len(), 3);
    assert!(candidates.contains(&"東京".to_string()));
    assert!(candidates.contains(&"大阪".to_string()));
    assert!(candidates.contains(&"京都".to_string()));
}

#[test]
fn test_collect_word_candidates_local_only() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    // Local search only - does NOT include global words
    let candidates = table.collect_word_candidates("会話_1", "挨拶").unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(candidates.contains(&"やあ".to_string()));
    assert!(!candidates.contains(&"こんにちは".to_string()));
    assert!(!candidates.contains(&"おはよう".to_string()));
}

#[test]
fn test_collect_word_candidates_local_not_found_no_fallback() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    // Should NOT fall back to global - should return error
    let result = table.collect_word_candidates("別のモジュール", "場所");
    assert!(result.is_err());
    match result {
        Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "場所"),
        _ => panic!("Expected WordNotFound error"),
    }
}

#[test]
fn test_collect_word_candidates_global_only() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    // Empty module_name means global only search
    let candidates = table.collect_word_candidates("", "挨拶").unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&"こんにちは".to_string()));
    assert!(candidates.contains(&"おはよう".to_string()));
}

#[test]
fn test_collect_word_candidates_duplicate_entries_merge() {
    let mut registry = WordDefRegistry::new();
    registry.register_global(
        "挨拶",
        vec!["おはよう".to_string(), "こんにちわ".to_string()],
    );
    registry.register_global(
        "挨拶",
        vec!["はろー".to_string(), "ぐっもーにん".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    let candidates = table.collect_word_candidates("", "挨拶").unwrap();
    assert_eq!(candidates.len(), 4);
    assert!(candidates.contains(&"おはよう".to_string()));
    assert!(candidates.contains(&"こんにちわ".to_string()));
    assert!(candidates.contains(&"はろー".to_string()));
    assert!(candidates.contains(&"ぐっもーにん".to_string()));
}

#[test]
fn test_collect_word_candidates_not_found() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(registry, selector);

    let result = table.collect_word_candidates("", "存在しない");
    assert!(result.is_err());
    match result {
        Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "存在しない"),
        _ => panic!("Expected WordNotFound error"),
    }
}

#[test]
fn test_search_word_global_exact() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("", "挨拶");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "こんにちは");
}

#[test]
fn test_search_word_global_prefix() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("", "場所");
    assert!(result.is_ok());
    let word = result.unwrap();
    assert!(word == "東京" || word == "大阪" || word == "京都");
}

#[test]
fn test_search_word_local() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("会話_1", "挨拶");
    assert!(result.is_ok());
    let word = result.unwrap();
    assert_eq!(word, "やあ");
}

#[test]
fn test_search_word_not_found() {
    let registry = create_test_registry();
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("", "存在しない");
    assert!(result.is_err());
    match result {
        Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "存在しない"),
        _ => panic!("Expected WordNotFound error"),
    }
}

#[test]
fn test_search_word_cache_sequential() {
    let mut registry = WordDefRegistry::new();
    registry.register_global(
        "test",
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let r1 = table.search_word("", "test").unwrap();
    let r2 = table.search_word("", "test").unwrap();
    let r3 = table.search_word("", "test").unwrap();

    assert_eq!(r1, "a");
    assert_eq!(r2, "b");
    assert_eq!(r3, "c");
}

#[test]
fn test_search_word_cache_reshuffle() {
    let mut registry = WordDefRegistry::new();
    registry.register_global("test", vec!["a".to_string(), "b".to_string()]);

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let _ = table.search_word("", "test").unwrap();
    let _ = table.search_word("", "test").unwrap();

    let r3 = table.search_word("", "test").unwrap();
    assert_eq!(r3, "a"); // Back to first (no shuffle)
}

#[test]
fn test_cache_key_separation() {
    let mut registry = WordDefRegistry::new();
    registry.register_global("word", vec!["global".to_string()]);
    registry.register_local("mod1", "word", vec!["local1".to_string()]);
    registry.register_local("mod2", "word", vec!["local2".to_string()]);

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let r1 = table.search_word("mod1", "word").unwrap();
    let r2 = table.search_word("mod2", "word").unwrap();

    // Fallback strategy: local found, global NOT included
    // mod1 search: ONLY local1 (fallback)
    // mod2 search: ONLY local2 (fallback)
    assert_eq!(r1, "local1");
    assert_eq!(r2, "local2");
}

#[test]
fn test_local_does_not_match_global_prefix() {
    let mut registry = WordDefRegistry::new();
    registry.register_global("abc", vec!["global_abc".to_string()]);
    registry.register_local("mod", "abc", vec!["local_abc".to_string()]);

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("", "abc").unwrap();
    assert_eq!(result, "global_abc");
}

#[test]
fn test_search_word_merge_duplicate_entries() {
    let mut registry = WordDefRegistry::new();

    registry.register_global(
        "挨拶",
        vec!["おはよう".to_string(), "こんにちわ".to_string()],
    );
    registry.register_global(
        "挨拶",
        vec!["はろー".to_string(), "ぐっもーにん".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("", "挨拶");
    assert!(result.is_ok());

    let word = result.unwrap();
    assert!(
        word == "おはよう" || word == "こんにちわ" || word == "はろー" || word == "ぐっもーにん",
        "Unexpected word: {}",
        word
    );
}

#[test]
fn test_search_word_merge_duplicate_entries_all_words_reachable() {
    let mut registry = WordDefRegistry::new();

    registry.register_global(
        "挨拶",
        vec!["おはよう".to_string(), "こんにちわ".to_string()],
    );
    registry.register_global(
        "挨拶",
        vec!["はろー".to_string(), "ぐっもーにん".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let r1 = table.search_word("", "挨拶").unwrap();
    let r2 = table.search_word("", "挨拶").unwrap();
    let r3 = table.search_word("", "挨拶").unwrap();
    let r4 = table.search_word("", "挨拶").unwrap();

    let results = [r1, r2, r3, r4];

    assert!(results.contains(&"おはよう".to_string()));
    assert!(results.contains(&"こんにちわ".to_string()));
    assert!(results.contains(&"はろー".to_string()));
    assert!(results.contains(&"ぐっもーにん".to_string()));
}

#[test]
fn test_search_word_merge_duplicate_local_entries() {
    let mut registry = WordDefRegistry::new();

    registry.register_local("会話", "挨拶", vec!["やあ".to_string(), "よう".to_string()]);
    registry.register_local(
        "会話",
        "挨拶",
        vec!["おす".to_string(), "ういっす".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let result = table.search_word("会話", "挨拶");
    assert!(result.is_ok());

    let word = result.unwrap();
    assert!(
        word == "やあ" || word == "よう" || word == "おす" || word == "ういっす",
        "Unexpected word: {}",
        word
    );
}

#[test]
fn test_new_word_table_is_empty() {
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = WordTable::from_word_def_registry(WordDefRegistry::new(), selector);

    // Any search on an empty table is WordNotFound
    let result = table.search_word("", "anything");
    assert!(matches!(result, Err(WordTableError::WordNotFound { .. })));
}

#[test]
fn test_replace_selector_clears_cache_keeps_entries() {
    let mut registry = WordDefRegistry::new();
    registry.register_global("test", vec!["a".to_string(), "b".to_string()]);

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    assert_eq!(table.search_word("", "test").unwrap(), "a");

    // Replace the selector — cache must be cleared, entries preserved
    table.replace_selector(Box::new(MockRandomSelector::new(vec![0])));
    assert_eq!(
        table.search_word("", "test").unwrap(),
        "a",
        "cache was not cleared by replace_selector"
    );
}

#[test]
fn test_search_word_empty_key_matches_all_global() {
    // Empty key is a prefix of every key: global search merges all global words
    let mut registry = WordDefRegistry::new();
    registry.register_global("alpha", vec!["a".to_string()]);
    registry.register_global("beta", vec!["b".to_string()]);
    registry.register_local("mod", "gamma", vec!["c".to_string()]);

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = WordTable::from_word_def_registry(registry, selector);

    // Documents current behavior: empty key matches all GLOBAL entries,
    // local entries (':' prefix) are excluded
    let candidates = table.collect_word_candidates("", "").unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&"a".to_string()));
    assert!(candidates.contains(&"b".to_string()));
    assert!(!candidates.contains(&"c".to_string()));
}

#[test]
fn test_collect_word_candidates_entry_with_empty_values() {
    // An entry that exists but has zero values must yield WordNotFound,
    // not an empty list (collect_word_candidates' second guard)
    let mut registry = WordDefRegistry::new();
    registry.register_global("empty", vec![]);

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = WordTable::from_word_def_registry(registry, selector);

    let result = table.collect_word_candidates("", "empty");
    match result {
        Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "empty"),
        other => panic!("Expected WordNotFound, got {:?}", other),
    }
}

#[test]
fn test_collect_word_candidates_local_entry_with_empty_values() {
    // Same guard on the local search path
    let mut registry = WordDefRegistry::new();
    registry.register_local("mod", "empty", vec![]);

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = WordTable::from_word_def_registry(registry, selector);

    let result = table.collect_word_candidates("mod", "empty");
    match result {
        Err(WordTableError::WordNotFound { key }) => assert_eq!(key, "empty"),
        other => panic!("Expected WordNotFound, got {:?}", other),
    }
}

#[test]
fn test_search_word_merge_duplicate_with_global() {
    // Pattern: search for global, should get all 5 words (2 + 3)
    let mut registry = WordDefRegistry::new();

    registry.register_global("word", vec!["a".to_string(), "b".to_string()]);
    registry.register_global(
        "word",
        vec!["c".to_string(), "d".to_string(), "e".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let results: Vec<_> = (0..5)
        .map(|_| table.search_word("", "word").unwrap())
        .collect();

    assert_eq!(results.len(), 5);
    assert!(results.contains(&"a".to_string()));
    assert!(results.contains(&"b".to_string()));
    assert!(results.contains(&"c".to_string()));
    assert!(results.contains(&"d".to_string()));
    assert!(results.contains(&"e".to_string()));
}

#[test]
fn test_search_word_mock_sequence_applies_each_round() {
    // Mock [1]: the 2nd candidate comes first, the rest
    // follow in candidate order, and the next round applies [1] again.
    let mut registry = WordDefRegistry::new();
    registry.register_global(
        "test",
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );

    let selector = Box::new(MockRandomSelector::new(vec![1]));
    let mut table = WordTable::from_word_def_registry(registry, selector);

    let results: Vec<_> = (0..4)
        .map(|_| table.search_word("", "test").unwrap())
        .collect();

    assert_eq!(results, ["b", "a", "c", "b"]);
}

// 候補の順を固めるテスト（U2）：キーのバイト列の辞書順、同じキーの中は登録順。
fn create_order_registry() -> WordDefRegistry {
    let mut registry = WordDefRegistry::new();
    let w = |s: &str| vec![s.to_string()];
    registry.register_global("挨拶_夜", w("夜"));
    registry.register_global("挨拶", vec!["a1".to_string(), "a2".to_string()]);
    registry.register_local("会話_1", "挨拶", w("l1"));
    registry.register_global("挨拶A", w("A"));
    registry.register_global("挨拶", w("a3"));
    registry.register_global("挨拶1", w("1"));
    registry.register_global("挨拶あ", w("あ"));
    registry.register_global("Zebra", w("z"));
    registry.register_global("0番", w("0"));
    registry.register_global("雑談", w("x"));
    registry.register_local("会話_1", "挨拶_夜", w("l2"));
    registry
}

#[test]
fn test_collect_word_candidates_order_global_prefix() {
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(create_order_registry(), selector);

    let candidates = table.collect_word_candidates("", "挨拶").unwrap();
    assert_eq!(candidates, ["a1", "a2", "a3", "1", "A", "夜", "あ"]);
}

#[test]
fn test_collect_word_candidates_order_global_empty_key() {
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(create_order_registry(), selector);

    // ':'（0x3A）で始まるローカルのキーは `0番` と `Zebra` の間に並ぶが、大域では飛ばす
    let candidates = table.collect_word_candidates("", "").unwrap();
    assert_eq!(
        candidates,
        ["0", "z", "a1", "a2", "a3", "1", "A", "夜", "あ", "x"]
    );
}

#[test]
fn test_collect_word_candidates_order_local_prefix() {
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = WordTable::from_word_def_registry(create_order_registry(), selector);

    let candidates = table.collect_word_candidates("会話_1", "挨拶").unwrap();
    assert_eq!(candidates, ["l1", "l2"]);
}

#[test]
fn test_search_word_mock_sequence_picks_by_candidate_position() {
    // 指定列 [2, 0]：7 つの候補を [2, 0, 1, 3, 4, 5, 6] の位置の順に選ぶ
    let selector = Box::new(MockRandomSelector::new(vec![2, 0]));
    let mut table = WordTable::from_word_def_registry(create_order_registry(), selector);

    let results: Vec<_> = (0..7)
        .map(|_| table.search_word("", "挨拶").unwrap())
        .collect();
    assert_eq!(results, ["a3", "a1", "a2", "1", "A", "夜", "あ"]);
}
