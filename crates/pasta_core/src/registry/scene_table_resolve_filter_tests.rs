use super::scene_table_candidate_tests::create_test_scene_info;
use super::*;
use crate::registry::random::MockRandomSelector;

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
    assert!(matches!(result, Err(SceneTableError::SceneNotFound { .. })));
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
    assert_eq!(
        after,
        SceneId(0),
        "cache was not cleared by replace_selector"
    );

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

// ======================================================================
// Tests for resolve_scene_id_exact（完全一致で 1 つ選ぶ）
// ======================================================================

fn build_table(globals: &[&str], selector: Vec<usize>) -> SceneTable {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    for name in globals {
        registry.register_global(name, HashMap::new());
    }
    SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(selector))).unwrap()
}

fn scene_name(table: &SceneTable, id: SceneId) -> String {
    table.get_scene(id).unwrap().name.clone()
}

#[test]
fn test_resolve_scene_id_exact_ignores_longer_names_with_same_head() {
    // 「つっつき反応胸」で「つっつき反応胸右」を選ばない
    let mut table = build_table(&["つっつき反応胸右", "つっつき反応胸"], vec![0]);
    for _ in 0..4 {
        let id = table
            .resolve_scene_id_exact("", "つっつき反応胸", &HashMap::new())
            .unwrap();
        assert_eq!(scene_name(&table, id), "つっつき反応胸");
    }

    // 頭だけの名前では見つからない（前方一致しない）
    let result = table.resolve_scene_id_exact("", "つっつき反応", &HashMap::new());
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "つっつき反応"),
        other => panic!("Expected SceneNotFound, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn test_resolve_scene_id_exact_consumes_same_name_scenes_per_cycle() {
    // 同じ名前 3 つ（＋頭が同じ別の名前 1 つ）。一巡で 1 回ずつ、その後また一巡する
    let mut table = build_table(
        &["撫で反応頭", "撫で反応頭右", "撫で反応頭", "撫で反応頭"],
        vec![2, 0, 1],
    );
    let expected: Vec<SceneId> = table
        .labels_iter()
        .filter(|s| s.name == "撫で反応頭")
        .map(|s| s.id)
        .collect();
    assert_eq!(expected.len(), 3);

    for _cycle in 0..2 {
        let mut picked: Vec<SceneId> = (0..3)
            .map(|_| {
                table
                    .resolve_scene_id_exact("", "撫で反応頭", &HashMap::new())
                    .unwrap()
            })
            .collect();
        // MockRandomSelector の並べ替え（位置 2,0,1）に従う
        assert_eq!(picked, vec![expected[2], expected[0], expected[1]]);
        picked.sort_by_key(|id| id.0);
        assert_eq!(picked, expected, "一巡で 3 つが 1 回ずつ選ばれる");
    }
}

#[test]
fn test_resolve_scene_id_exact_cache_does_not_mix_with_prefix() {
    // 同じ文字列「会話」でも、前方一致（会話・会話続き）と完全一致（会話）のキャッシュは別
    let mut table = build_table(&["会話", "会話続き"], vec![0]);
    table.set_shuffle_enabled(false);
    let empty = HashMap::new();

    let first = table.resolve_scene_id_unified("", "会話", &empty).unwrap();
    assert_eq!(scene_name(&table, first), "会話");

    for _ in 0..2 {
        let id = table.resolve_scene_id_exact("", "会話", &empty).unwrap();
        assert_eq!(scene_name(&table, id), "会話");
    }

    // 完全一致の呼び出しが前方一致の巡回を進めていない
    let second = table.resolve_scene_id_unified("", "会話", &empty).unwrap();
    assert_eq!(scene_name(&table, second), "会話続き");
}

#[test]
fn test_resolve_scene_id_exact_local_scope() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    let (_, counter) = registry.register_global("会話", HashMap::new());
    registry.register_local("選択肢続き", "会話", counter, 1, HashMap::new());
    registry.register_local("選択肢", "会話", counter, 2, HashMap::new());
    registry.register_global("選択肢", HashMap::new());
    let mut table =
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![0])))
            .unwrap();
    let empty = HashMap::new();

    // ローカルの範囲: 頭が同じローカルも、同じ名前のグローバルも選ばない
    for _ in 0..3 {
        let id = table
            .resolve_scene_id_exact("会話_1", "選択肢", &empty)
            .unwrap();
        let scene = table.get_scene(id).unwrap();
        assert_eq!(scene.name, "選択肢");
        assert_eq!(scene.scope, SceneScope::Local);
    }

    // グローバルの範囲: ローカルを選ばない
    let id = table.resolve_scene_id_exact("", "選択肢", &empty).unwrap();
    assert_eq!(table.get_scene(id).unwrap().scope, SceneScope::Global);

    // ローカルに無い名前はグローバルへ落ちない
    let result = table.resolve_scene_id_exact("会話_1", "会話", &empty);
    assert!(matches!(result, Err(SceneTableError::SceneNotFound { .. })));
}

#[test]
fn test_resolve_scene_id_exact_empty_name_is_invalid_scene() {
    let mut table = build_table(&["会話"], vec![0]);
    for module in ["", "会話_1"] {
        let result = table.resolve_scene_id_exact(module, "", &HashMap::new());
        match result {
            Err(SceneTableError::InvalidScene { scene }) => assert_eq!(scene, ""),
            other => panic!("Expected InvalidScene, got {:?}", other.map(|_| ())),
        }
    }
}

#[test]
fn test_resolve_scene_id_exact_attribute_filter() {
    let mut table = build_filtered_table();

    let mut winter = HashMap::new();
    winter.insert("季節".to_string(), "冬".to_string());
    let id = table.resolve_scene_id_exact("", "挨拶", &winter).unwrap();
    assert_eq!(
        table.get_scene(id).unwrap().attributes.get("季節"),
        Some(&"冬".to_string())
    );

    let mut summer = HashMap::new();
    summer.insert("季節".to_string(), "夏".to_string());
    let result = table.resolve_scene_id_exact("", "挨拶", &summer);
    assert!(matches!(
        result,
        Err(SceneTableError::NoMatchingScene { .. })
    ));
}

// ======================================================================
// Tests for resolve_global_owning_local_exact（そのローカルを持つ大域を順送りで選ぶ）
// ======================================================================

/// `会話_1`（挨拶あり）・`会話_2`（なし）・`会話_3`（あり）と、ローカルを持たない `雑談_1`
fn build_owning_table() -> SceneTable {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    let (_, c1) = registry.register_global("会話", HashMap::new());
    registry.register_local("挨拶", "会話", c1, 1, HashMap::new());
    let (_, c2) = registry.register_global("会話", HashMap::new());
    registry.register_local("挨拶続き", "会話", c2, 1, HashMap::new());
    let (_, c3) = registry.register_global("会話", HashMap::new());
    registry.register_local("挨拶", "会話", c3, 1, HashMap::new());
    registry.register_global("雑談", HashMap::new());
    let mut table =
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![0])))
            .unwrap();
    table.set_shuffle_enabled(false);
    table
}

fn owner_name(table: &SceneTable, id: SceneId) -> String {
    let scene = table.get_scene(id).unwrap();
    assert_eq!(scene.scope, SceneScope::Global);
    scene.fn_name.split_once("::").unwrap().0.to_string()
}

#[test]
fn test_resolve_global_owning_local_exact_picks_only_owners_per_cycle() {
    let mut table = build_owning_table();
    let empty = HashMap::new();

    for _cycle in 0..2 {
        let mut picked: Vec<String> = (0..2)
            .map(|_| {
                let id = table
                    .resolve_global_owning_local_exact("会話", "挨拶", &empty)
                    .unwrap()
                    .expect("挨拶を持つ大域がある");
                owner_name(&table, id)
            })
            .collect();
        picked.sort();
        assert_eq!(
            picked,
            vec!["会話_1", "会話_3"],
            "一巡で 1 と 3 が 1 回ずつ"
        );
    }
}

#[test]
fn test_resolve_global_owning_local_exact_no_global_and_no_owner() {
    let mut table = build_owning_table();
    let empty = HashMap::new();

    // 大域が無い
    let result = table.resolve_global_owning_local_exact("存在しない", "挨拶", &empty);
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "存在しない"),
        other => panic!("Expected SceneNotFound, got {:?}", other),
    }

    // 大域はあるがどれも持たない（頭が同じローカル「挨拶続き」も数えない）
    assert_eq!(
        table
            .resolve_global_owning_local_exact("会話", "挨", &empty)
            .unwrap(),
        None
    );
    assert_eq!(
        table
            .resolve_global_owning_local_exact("雑談", "挨拶", &empty)
            .unwrap(),
        None
    );

    // 名前が空
    for (name, local) in [("", "挨拶"), ("会話", "")] {
        let result = table.resolve_global_owning_local_exact(name, local, &empty);
        assert!(
            matches!(result, Err(SceneTableError::InvalidScene { .. })),
            "({name:?}, {local:?}) は不正なシーン"
        );
    }
}

#[test]
fn test_resolve_global_owning_local_exact_cache_independent_of_exact() {
    // ふつうの完全一致（1,2,3 の順送り）と状態が混ざらない
    let mut table = build_owning_table();
    let empty = HashMap::new();

    let mut owning = Vec::new();
    let mut exact = Vec::new();
    for _ in 0..3 {
        let id = table
            .resolve_global_owning_local_exact("会話", "挨拶", &empty)
            .unwrap()
            .unwrap();
        owning.push(owner_name(&table, id));
        let id = table.resolve_scene_id_exact("", "会話", &empty).unwrap();
        exact.push(owner_name(&table, id));
    }
    assert_eq!(owning, vec!["会話_1", "会話_3", "会話_1"]);
    assert_eq!(exact, vec!["会話_1", "会話_2", "会話_3"]);
}
