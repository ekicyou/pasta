use super::*;
use crate::registry::random::MockRandomSelector;
use super::scene_table_candidate_tests::create_test_scene_info;

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
