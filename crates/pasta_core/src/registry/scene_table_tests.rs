//! SceneTable テスト (#[path] パターン)
//!
//! Phase A: registry/scene_table.rs のインラインテストを分離
//! 例外理由: SceneTableの全privateフィールド（labels, prefix_index, cache, random_selector, shuffle_enabled）
//! への直接アクセスが構造的に必要なため、#[path]パターンで分離

use super::*;
use crate::registry::random::MockRandomSelector;

fn create_test_scene_info(id: usize, name: &str, fn_name: &str) -> SceneInfo {
    SceneInfo {
        id: SceneId(id),
        name: name.to_string(),
        scope: SceneScope::Global,
        attributes: HashMap::new(),
        fn_name: fn_name.to_string(),
        parent: None,
    }
}

#[test]
fn test_resolve_scene_id_basic() {
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable {
        labels: vec![create_test_scene_info(0, "test", "test_1::__start__")],
        prefix_index: {
            let mut map = RadixMap::new();
            map.insert(b"test_1::__start__", vec![SceneId(0)]);
            map
        },
        cache: HashMap::new(),
        random_selector: selector,
        shuffle_enabled: false,
    };

    let result = table.resolve_scene_id("test", &HashMap::new());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SceneId(0));
}

// ======================================================================
// Tests for collect_scene_candidates (Task 5.1)
// ======================================================================

fn create_test_local_scene_info(
    id: usize,
    name: &str,
    fn_name: &str,
    parent: &str,
) -> SceneInfo {
    SceneInfo {
        id: SceneId(id),
        name: name.to_string(),
        scope: SceneScope::Local,
        attributes: HashMap::new(),
        fn_name: fn_name.to_string(),
        parent: Some(parent.to_string()),
    }
}

#[test]
fn test_collect_scene_candidates_local_search() {
    // Test: Local search should find scenes with :module:prefix pattern
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add local scene with :parent:local key format
    table.labels.push(create_test_local_scene_info(
        0,
        "選択肢",
        "会話_1::選択肢_1",
        "会話_1",
    ));
    table
        .prefix_index
        .insert(":会話_1:選択肢".as_bytes(), vec![SceneId(0)]);

    // Search from 会話_1 module
    let result = table.collect_scene_candidates("会話_1", "選択肢");
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], SceneId(0));
}

#[test]
fn test_collect_scene_candidates_global_search() {
    // Test: Global search (empty module_name) should find scenes without : prefix
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add global scene with simple key
    table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
    table
        .prefix_index
        .insert("挨拶".as_bytes(), vec![SceneId(0)]);

    // Search with empty module_name - should get global
    let result = table.collect_scene_candidates("", "挨拶");
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], SceneId(0));
}

#[test]
fn test_collect_scene_candidates_local_only() {
    // Test: Local search only - no global fallback
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add global scene
    table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
    table
        .prefix_index
        .insert("挨拶".as_bytes(), vec![SceneId(0)]);

    // Add local scene with same prefix
    table.labels.push(create_test_local_scene_info(
        1,
        "挨拶",
        "会話_1::挨拶_1",
        "会話_1",
    ));
    table
        .prefix_index
        .insert(":会話_1:挨拶".as_bytes(), vec![SceneId(1)]);

    // Search from 会話_1 module - should get ONLY local
    let result = table.collect_scene_candidates("会話_1", "挨拶");
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(candidates.contains(&SceneId(1))); // local only
    assert!(!candidates.contains(&SceneId(0))); // global NOT included
}

#[test]
fn test_collect_scene_candidates_local_not_found_no_fallback() {
    // Test: No local found, should NOT fall back to global - return error
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add global scene only
    table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
    table
        .prefix_index
        .insert("挨拶".as_bytes(), vec![SceneId(0)]);

    // Search from 会話_1 module - no local, should NOT fall back to global
    let result = table.collect_scene_candidates("会話_1", "挨拶");
    assert!(result.is_err());
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "挨拶"),
        _ => panic!("Expected SceneNotFound error"),
    }
}

#[test]
fn test_collect_scene_candidates_global_via_empty_module_name() {
    // Test: Empty module_name should search global only
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add global scene
    table.labels.push(create_test_scene_info(0, "挨拶", "挨拶"));
    table
        .prefix_index
        .insert("挨拶".as_bytes(), vec![SceneId(0)]);

    // Search with empty module_name - should get global
    let result = table.collect_scene_candidates("", "挨拶");
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(candidates.contains(&SceneId(0)));
}

#[test]
fn test_collect_scene_candidates_prefix_match() {
    // Test: Prefix matching should work correctly
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add scenes with common prefix
    table
        .labels
        .push(create_test_scene_info(0, "挨拶_朝", "挨拶_朝"));
    table
        .labels
        .push(create_test_scene_info(1, "挨拶_昼", "挨拶_昼"));
    table
        .labels
        .push(create_test_scene_info(2, "挨拶_夜", "挨拶_夜"));

    // Insert with proper UTF-8 bytes
    table
        .prefix_index
        .insert("挨拶_朝".as_bytes(), vec![SceneId(0)]);
    table
        .prefix_index
        .insert("挨拶_昼".as_bytes(), vec![SceneId(1)]);
    table
        .prefix_index
        .insert("挨拶_夜".as_bytes(), vec![SceneId(2)]);

    // Prefix search for "挨拶" should find all three
    let result = table.collect_scene_candidates("", "挨拶");
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert_eq!(candidates.len(), 3);
}

#[test]
fn test_collect_scene_candidates_not_found() {
    // Test: Should return error when no candidates found
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = SceneTable::new(selector);

    let result = table.collect_scene_candidates("会話_1", "存在しないシーン");
    assert!(result.is_err());
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => {
            assert_eq!(scene, "存在しないシーン");
        }
        _ => panic!("Expected SceneNotFound error"),
    }
}

#[test]
fn test_collect_scene_candidates_empty_prefix_error() {
    // Test: Empty prefix should return error
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = SceneTable::new(selector);

    let result = table.collect_scene_candidates("会話_1", "");
    assert!(result.is_err());
    match result {
        Err(SceneTableError::InvalidScene { scene }) => {
            assert_eq!(scene, "");
        }
        _ => panic!("Expected InvalidScene error"),
    }
}

#[test]
fn test_collect_scene_candidates_exclude_local_from_global() {
    // Test: Global search should exclude local keys (starting with :)
    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::new(selector);

    // Add a local scene
    table.labels.push(create_test_local_scene_info(
        0,
        "選択肢",
        "他モジュール::選択肢_1",
        "他モジュール",
    ));
    table
        .prefix_index
        .insert(":他モジュール:選択肢".as_bytes(), vec![SceneId(0)]);

    // Search from different module - should NOT find the local scene of another module
    let result = table.collect_scene_candidates("会話_1", "選択肢");
    assert!(result.is_err()); // No candidates
}

// ======================================================================
// Tests for fn_name_to_search_key and prefix_index conversion (Task 5.3)
// ======================================================================

#[test]
fn test_fn_name_to_search_key_local_scene() {
    // Local scene: "会話_1::選択肢_1" → ":会話_1:選択肢_1"
    let result = SceneTable::fn_name_to_search_key("会話_1::選択肢_1", true);
    assert_eq!(result, ":会話_1:選択肢_1");
}

#[test]
fn test_fn_name_to_search_key_global_scene() {
    // Global scene: "会話_1::__start__" → "会話_1"
    let result = SceneTable::fn_name_to_search_key("会話_1::__start__", false);
    assert_eq!(result, "会話_1");
}

#[test]
fn test_fn_name_to_search_key_global_scene_no_suffix() {
    // Global scene without ::__start__: "挨拶" → "挨拶"
    let result = SceneTable::fn_name_to_search_key("挨拶", false);
    assert_eq!(result, "挨拶");
}

#[test]
fn test_from_scene_registry_key_conversion() {
    use crate::registry::SceneRegistry;

    // Create a registry with global and local scenes
    let mut registry = SceneRegistry::new();
    let (_, counter) = registry.register_global("会話", HashMap::new());
    registry.register_local("選択肢", "会話", counter, 1, HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let table = SceneTable::from_scene_registry(registry, selector).unwrap();

    // Verify search key conversion works with collect_scene_candidates
    // Global scene search
    let global_result = table.collect_scene_candidates("", "会話");
    assert!(global_result.is_ok());
    assert_eq!(global_result.unwrap().len(), 1);

    // Local scene search (from 会話_1 module)
    let local_result = table.collect_scene_candidates("会話_1", "選択肢");
    assert!(local_result.is_ok());
    assert_eq!(local_result.unwrap().len(), 1);

    // Local scene should not be found from different module
    let cross_module_result = table.collect_scene_candidates("他のモジュール", "選択肢");
    assert!(cross_module_result.is_err());
}

// ======================================================================
// Tests for resolve_scene_id_unified (Task 5.2)
// ======================================================================

#[test]
fn test_resolve_scene_id_unified_local_scene() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    let (_, counter) = registry.register_global("会話", HashMap::new());
    registry.register_local("選択肢", "会話", counter, 1, HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // Resolve local scene from parent module
    let result = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
    assert!(result.is_ok());
    let scene_id = result.unwrap();
    let scene = table.get_scene(scene_id).unwrap();
    assert!(scene.name.contains("選択肢"));
}

#[test]
fn test_resolve_scene_id_unified_global_scene() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("挨拶", HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // Resolve global scene - must use empty module_name for global search
    let result = table.resolve_scene_id_unified("", "挨拶", &HashMap::new());
    assert!(result.is_ok());
}

#[test]
fn test_resolve_scene_id_unified_global_scene_no_fallback() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("挨拶", HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // Try to resolve global scene from a module - should fail (no fallback)
    let result = table.resolve_scene_id_unified("任意のモジュール", "挨拶", &HashMap::new());
    assert!(result.is_err());
}

#[test]
fn test_resolve_scene_id_unified_local_found() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    // Global scene "挨拶" (should not be found when searching from module)
    registry.register_global("挨拶", HashMap::new());
    // Local scene "挨拶" in module 会話_1
    let (_, counter) = registry.register_global("会話", HashMap::new());
    registry.register_local("挨拶", "会話", counter, 1, HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // Resolve from 会話_1 - should find ONLY local (no fallback to global)
    let result = table.resolve_scene_id_unified("会話_1", "挨拶", &HashMap::new());
    assert!(result.is_ok());
    let scene_id = result.unwrap();
    let scene = table.get_scene(scene_id).unwrap();
    // Should be local scene (has parent)
    assert!(scene.parent.is_some());

    // Call again - should succeed due to cycling reset (1 local candidate cycles)
    let result2 = table.resolve_scene_id_unified("会話_1", "挨拶", &HashMap::new());
    assert!(result2.is_ok()); // Cycling reset returns the same scene
    let scene_id2 = result2.unwrap();
    let scene2 = table.get_scene(scene_id2).unwrap();
    assert!(scene2.parent.is_some()); // Still a local scene
}

// ======================================================================
// Tests for cycling reset (Task 3.1〜3.4)
// ======================================================================

#[test]
fn test_resolve_scene_id_cycling() {
    // Task 3.1: 3件のシーンを登録、4回目の呼び出しが成功することを検証
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable {
        labels: vec![
            create_test_scene_info(0, "OnTalk1", "OnTalk"),
            create_test_scene_info(1, "OnTalk2", "OnTalk"),
            create_test_scene_info(2, "OnTalk3", "OnTalk"),
        ],
        prefix_index: {
            let mut map = RadixMap::new();
            map.insert(b"OnTalk", vec![SceneId(0), SceneId(1), SceneId(2)]);
            map
        },
        cache: HashMap::new(),
        random_selector: selector,
        shuffle_enabled: false,
    };

    // 1回目〜3回目: 候補を順番に消費
    let r1 = table.resolve_scene_id("OnTalk", &HashMap::new());
    assert!(r1.is_ok());
    let r2 = table.resolve_scene_id("OnTalk", &HashMap::new());
    assert!(r2.is_ok());
    let r3 = table.resolve_scene_id("OnTalk", &HashMap::new());
    assert!(r3.is_ok());

    // 4回目: 循環リセットが発生し、成功すること
    let r4 = table.resolve_scene_id("OnTalk", &HashMap::new());
    assert!(
        r4.is_ok(),
        "4回目の呼び出しが失敗: 循環リセットが動作していない"
    );

    // 返却されたSceneIdが候補のいずれかであること
    let id = r4.unwrap();
    assert!(
        id == SceneId(0) || id == SceneId(1) || id == SceneId(2),
        "返却されたSceneId {:?} が候補に含まれていない",
        id
    );

    // さらに続行できることも検証（10回以上）
    for i in 5..=12 {
        let r = table.resolve_scene_id("OnTalk", &HashMap::new());
        assert!(r.is_ok(), "{}回目の呼び出しが失敗", i);
    }
}

#[test]
fn test_resolve_scene_id_cycling_reshuffles() {
    // Task 3.2: シャッフル有効時のリセット後再シャッフル検証
    // DefaultRandomSelectorを使用して実際のシャッフルが発生することを確認
    use crate::registry::random::DefaultRandomSelector;

    let selector = Box::new(DefaultRandomSelector::with_seed(42));
    let mut table = SceneTable {
        labels: vec![
            create_test_scene_info(0, "OnTalk1", "OnTalk"),
            create_test_scene_info(1, "OnTalk2", "OnTalk"),
            create_test_scene_info(2, "OnTalk3", "OnTalk"),
        ],
        prefix_index: {
            let mut map = RadixMap::new();
            map.insert(b"OnTalk", vec![SceneId(0), SceneId(1), SceneId(2)]);
            map
        },
        cache: HashMap::new(),
        random_selector: selector,
        shuffle_enabled: true,
    };

    // 1周目の順序を記録
    let mut first_cycle = Vec::new();
    for _ in 0..3 {
        let r = table.resolve_scene_id("OnTalk", &HashMap::new());
        first_cycle.push(r.unwrap());
    }

    // 2周目（リセット後）の順序を記録
    let mut second_cycle = Vec::new();
    for _ in 0..3 {
        let r = table.resolve_scene_id("OnTalk", &HashMap::new());
        assert!(r.is_ok());
        second_cycle.push(r.unwrap());
    }

    // 同じ候補セットだが、シャッフルにより順序が異なる可能性がある
    let first_set: std::collections::HashSet<_> = first_cycle.iter().collect();
    let second_set: std::collections::HashSet<_> = second_cycle.iter().collect();
    assert_eq!(first_set, second_set, "候補セットが変化している");
}

#[test]
fn test_resolve_scene_id_cycling_preserves_candidates() {
    // Task 3.3: リセット前後で候補リスト（ID集合）が保持されることを検証
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable {
        labels: vec![
            create_test_scene_info(0, "OnTalk1", "OnTalk"),
            create_test_scene_info(1, "OnTalk2", "OnTalk"),
            create_test_scene_info(2, "OnTalk3", "OnTalk"),
        ],
        prefix_index: {
            let mut map = RadixMap::new();
            map.insert(b"OnTalk", vec![SceneId(0), SceneId(1), SceneId(2)]);
            map
        },
        cache: HashMap::new(),
        random_selector: selector,
        shuffle_enabled: false,
    };

    // 1周目: 全候補を収集
    let mut first_cycle: std::collections::HashSet<SceneId> = std::collections::HashSet::new();
    for _ in 0..3 {
        first_cycle.insert(table.resolve_scene_id("OnTalk", &HashMap::new()).unwrap());
    }
    assert_eq!(first_cycle.len(), 3);

    // 2周目: リセット後の全候補を収集
    let mut second_cycle: std::collections::HashSet<SceneId> = std::collections::HashSet::new();
    for _ in 0..3 {
        second_cycle.insert(table.resolve_scene_id("OnTalk", &HashMap::new()).unwrap());
    }
    assert_eq!(second_cycle.len(), 3);

    // 候補セットが同一であること
    assert_eq!(
        first_cycle, second_cycle,
        "リセット後に候補リストが変化している"
    );
}

#[test]
fn test_resolve_scene_id_unified_cycling() {
    // Task 3.4: unified版での循環リセット動作検証
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    let (_, counter) = registry.register_global("会話", HashMap::new());
    registry.register_local("選択肢A", "会話", counter, 1, HashMap::new());
    registry.register_local("選択肢B", "会話", counter, 2, HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // 1周目: 2件のローカル候補を消費
    let r1 = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
    assert!(r1.is_ok());
    let r2 = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
    assert!(r2.is_ok());

    // 3回目: 循環リセットが発生し、成功すること
    let r3 = table.resolve_scene_id_unified("会話_1", "選択肢", &HashMap::new());
    assert!(
        r3.is_ok(),
        "unified版で3回目の呼び出しが失敗: 循環リセットが動作していない"
    );

    // 返却されたSceneIdが有効なローカルシーンであること
    let scene = table.get_scene(r3.unwrap()).unwrap();
    assert!(scene.parent.is_some(), "返却されたシーンがローカルでない");
}

// ======================================================================
// Tests for resolve_scene_id error paths and attribute filtering (G1)
// ======================================================================

#[test]
fn test_resolve_scene_id_empty_key_is_invalid_scene() {
    // Empty search_key must return InvalidScene (not SceneNotFound)
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::new(selector);

    let result = table.resolve_scene_id("", &HashMap::new());
    match result {
        Err(SceneTableError::InvalidScene { scene }) => assert_eq!(scene, ""),
        other => panic!("Expected InvalidScene, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn test_resolve_scene_id_not_found() {
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::new(selector);

    let result = table.resolve_scene_id("存在しない", &HashMap::new());
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "存在しない"),
        other => panic!("Expected SceneNotFound, got {:?}", other.map(|_| ())),
    }
}

fn build_filtered_table() -> SceneTable {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    let mut spring = HashMap::new();
    spring.insert("季節".to_string(), "春".to_string());
    let mut winter = HashMap::new();
    winter.insert("季節".to_string(), "冬".to_string());
    registry.register_global("挨拶", spring);
    registry.register_global("挨拶", winter);

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);
    table
}

#[test]
fn test_resolve_scene_id_attribute_filter_selects_matching() {
    let mut table = build_filtered_table();

    let mut filters = HashMap::new();
    filters.insert("季節".to_string(), "冬".to_string());

    // Only the winter scene matches; repeated calls keep returning it
    for _ in 0..3 {
        let id = table.resolve_scene_id("挨拶", &filters).unwrap();
        let scene = table.get_scene(id).unwrap();
        assert_eq!(scene.attributes.get("季節"), Some(&"冬".to_string()));
    }
}

#[test]
fn test_resolve_scene_id_attribute_filter_no_match() {
    let mut table = build_filtered_table();

    let mut filters = HashMap::new();
    filters.insert("季節".to_string(), "夏".to_string());

    let result = table.resolve_scene_id("挨拶", &filters);
    match result {
        Err(SceneTableError::NoMatchingScene { scene, filters: f }) => {
            assert_eq!(scene, "挨拶");
            assert_eq!(f.get("季節"), Some(&"夏".to_string()));
        }
        other => panic!("Expected NoMatchingScene, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn test_resolve_scene_id_unified_attribute_filter_no_match() {
    let mut table = build_filtered_table();

    let mut filters = HashMap::new();
    filters.insert("季節".to_string(), "夏".to_string());

    // Candidates exist (prefix matches) but filters reject all of them
    let result = table.resolve_scene_id_unified("", "挨拶", &filters);
    match result {
        Err(SceneTableError::NoMatchingScene { scene, .. }) => assert_eq!(scene, "挨拶"),
        other => panic!("Expected NoMatchingScene, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn test_find_scene_returns_fn_name() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("会話", HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // Legacy method returns the fn_name string of the resolved scene
    let fn_name = table.find_scene("会話", &HashMap::new()).unwrap();
    assert_eq!(fn_name, "会話_1::__start__");
}

#[test]
fn test_find_scene_not_found() {
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::new(selector);

    let result = table.find_scene("存在しない", &HashMap::new());
    assert!(matches!(
        result,
        Err(SceneTableError::SceneNotFound { .. })
    ));
}

#[test]
fn test_get_scene_out_of_range_returns_none() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("会話", HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = SceneTable::from_scene_registry(registry, selector).unwrap();

    assert!(table.get_scene(SceneId(0)).is_some());
    assert!(table.get_scene(SceneId(99)).is_none());
}

#[test]
fn test_labels_iter_yields_all_scenes() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("会話", HashMap::new());
    registry.register_global("挨拶", HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = SceneTable::from_scene_registry(registry, selector).unwrap();

    let names: Vec<_> = table.labels_iter().map(|s| s.name.clone()).collect();
    assert_eq!(names, vec!["会話", "挨拶"]);
}

#[test]
fn test_replace_selector_clears_cache() {
    // After replace_selector, sequential consumption restarts from a fresh cycle
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable {
        labels: vec![
            create_test_scene_info(0, "OnTalk1", "OnTalk"),
            create_test_scene_info(1, "OnTalk2", "OnTalk"),
        ],
        prefix_index: {
            let mut map = RadixMap::new();
            map.insert(b"OnTalk", vec![SceneId(0), SceneId(1)]);
            map
        },
        cache: HashMap::new(),
        random_selector: selector,
        shuffle_enabled: false,
    };

    // Consume the first candidate of the cycle
    let first = table.resolve_scene_id("OnTalk", &HashMap::new()).unwrap();
    assert_eq!(first, SceneId(0));

    // Replace selector — cache must be cleared, cycle restarts at the beginning
    table.replace_selector(Box::new(MockRandomSelector::new(vec![0])));
    let after = table.resolve_scene_id("OnTalk", &HashMap::new()).unwrap();
    assert_eq!(after, SceneId(0), "cache was not cleared by replace_selector");

    // Labels and prefix_index stay intact
    assert_eq!(table.labels_iter().count(), 2);
}

#[test]
fn test_from_scene_registry_groups_same_name_scenes() {
    use crate::registry::SceneRegistry;

    // Two scenes with the same name share the search-key prefix and both
    // are reachable via sequential consumption
    let mut registry = SceneRegistry::new();
    registry.register_global("会話", HashMap::new()); // 会話_1
    registry.register_global("会話", HashMap::new()); // 会話_2

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    let id1 = table.resolve_scene_id("会話", &HashMap::new()).unwrap();
    let id2 = table.resolve_scene_id("会話", &HashMap::new()).unwrap();
    assert_ne!(id1, id2, "sequential consumption must visit both scenes");
}

#[test]
fn test_resolve_scene_id_unified_cache_key_includes_module() {
    use crate::registry::SceneRegistry;

    // Create registry with same-named local scenes in different modules
    let mut registry = SceneRegistry::new();
    let (_, counter1) = registry.register_global("会話A", HashMap::new());
    registry.register_local("選択肢", "会話A", counter1, 1, HashMap::new());
    let (_, counter2) = registry.register_global("会話B", HashMap::new());
    registry.register_local("選択肢", "会話B", counter2, 1, HashMap::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(registry, selector).unwrap();
    table.set_shuffle_enabled(false);

    // Resolve from 会話A_1
    let result_a = table.resolve_scene_id_unified("会話A_1", "選択肢", &HashMap::new());
    assert!(result_a.is_ok());

    // Resolve from 会話B_1 - should use different cache key
    let result_b = table.resolve_scene_id_unified("会話B_1", "選択肢", &HashMap::new());
    assert!(result_b.is_ok());

    // Both should succeed (different cache keys)
    // The scenes should be different
    let scene_a = table.get_scene(result_a.unwrap()).unwrap();
    let scene_b = table.get_scene(result_b.unwrap()).unwrap();
    assert_ne!(scene_a.fn_name, scene_b.fn_name);
}
