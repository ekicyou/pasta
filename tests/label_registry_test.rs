use pasta::transpiler::LabelRegistry;
use std::collections::HashMap;

#[test]
fn test_label_registry_basic() {
    let mut registry = LabelRegistry::new();

    // Register global labels
    let (id1, counter1) = registry.register_global("会話", HashMap::new());
    let (id2, counter2) = registry.register_global("別会話", HashMap::new());

    assert_eq!(id1, 1);
    assert_eq!(counter1, 1);
    assert_eq!(id2, 2);
    assert_eq!(counter2, 1);

    // Verify labels
    let label1 = registry.get_label(id1).unwrap();
    assert_eq!(label1.name, "会話");
    assert_eq!(label1.fn_path, "crate::会話_1::__start__");

    let label2 = registry.get_label(id2).unwrap();
    assert_eq!(label2.name, "別会話");
    assert_eq!(label2.fn_path, "crate::別会話_1::__start__");

    // Check all_labels
    let all = registry.all_labels();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_label_registry_with_local_labels() {
    let mut registry = LabelRegistry::new();

    // Register parent
    let (parent_id, parent_counter) = registry.register_global("メイン", HashMap::new());

    // Register local labels
    let (local1_id, _local1_counter) =
        registry.register_local("自己紹介", "メイン", parent_counter, HashMap::new());

    let (local2_id, _local2_counter) =
        registry.register_local("趣味紹介", "メイン", parent_counter, HashMap::new());

    assert_eq!(parent_id, 1);
    assert_eq!(local1_id, 2);
    assert_eq!(local2_id, 3);

    // Verify parent
    let parent = registry.get_label(parent_id).unwrap();
    assert_eq!(parent.name, "メイン");
    assert_eq!(parent.fn_path, "crate::メイン_1::__start__");
    assert_eq!(parent.parent, None);

    // Verify local labels
    let local1 = registry.get_label(local1_id).unwrap();
    assert_eq!(local1.name, "自己紹介");
    assert_eq!(local1.fn_path, "crate::メイン_1::自己紹介_1");
    assert_eq!(local1.parent, Some("メイン".to_string()));

    let local2 = registry.get_label(local2_id).unwrap();
    assert_eq!(local2.name, "趣味紹介");
    assert_eq!(local2.fn_path, "crate::メイン_1::趣味紹介_1");
    assert_eq!(local2.parent, Some("メイン".to_string()));
}

#[test]
fn test_label_registry_duplicate_names() {
    let mut registry = LabelRegistry::new();

    // Register duplicate global labels
    let (id1, counter1) = registry.register_global("会話", HashMap::new());
    let (id2, counter2) = registry.register_global("会話", HashMap::new());
    let (id3, counter3) = registry.register_global("会話", HashMap::new());

    assert_eq!(counter1, 1);
    assert_eq!(counter2, 2);
    assert_eq!(counter3, 3);

    let label1 = registry.get_label(id1).unwrap();
    let label2 = registry.get_label(id2).unwrap();
    let label3 = registry.get_label(id3).unwrap();

    assert_eq!(label1.fn_path, "crate::会話_1::__start__");
    assert_eq!(label2.fn_path, "crate::会話_2::__start__");
    assert_eq!(label3.fn_path, "crate::会話_3::__start__");
}
