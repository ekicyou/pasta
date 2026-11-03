use super::scene_table_candidate_tests::create_test_scene_info;
use super::*;
use crate::registry::SceneRegistry;
use crate::registry::random::MockRandomSelector;

// ======================================================================
// Tests for resolve_scene_id_unified error paths and attribute filtering (G1)
// ======================================================================

#[test]
fn test_resolve_scene_id_empty_key_is_invalid_scene() {
    // Empty search_key must return InvalidScene (not SceneNotFound)
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(SceneRegistry::new(), selector);

    let result = table.resolve_scene_id_unified("", "", None);
    match result {
        Err(SceneTableError::InvalidScene { scene }) => assert_eq!(scene, ""),
        other => panic!("Expected InvalidScene, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn test_resolve_scene_id_not_found() {
    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let mut table = SceneTable::from_scene_registry(SceneRegistry::new(), selector);

    let result = table.resolve_scene_id_unified("", "存在しない", None);
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "存在しない"),
        other => panic!("Expected SceneNotFound, got {:?}", other.map(|_| ())),
    }
}

fn season(value: &str) -> crate::registry::SceneAttr {
    crate::registry::SceneAttr::Decl {
        key: "季節".to_string(),
        value: crate::registry::SceneAttrValue::String(value.to_string()),
    }
}

/// 宣言の項目が全部等しい候補だけ満たす述語（同じ名前の宣言は後の値）
fn decl_eq(pairs: &[(&str, &str)]) -> impl FnMut(&SceneInfo) -> bool {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |scene| {
        pairs.iter().all(|(key, value)| {
            scene
                .attributes
                .iter()
                .rev()
                .find_map(|a| match a {
                    crate::registry::SceneAttr::Decl { key: k, value: v } if k == key => Some(v),
                    _ => None,
                })
                .is_some_and(|v| v.to_string() == *value)
        })
    }
}

fn build_filtered_table() -> SceneTable {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("挨拶", vec![season("春")]);
    registry.register_global("挨拶", vec![season("冬")]);

    let selector = Box::new(MockRandomSelector::new(vec![]));
    SceneTable::from_scene_registry(registry, selector)
}

#[test]
fn test_resolve_scene_id_attribute_filter_selects_matching() {
    let mut table = build_filtered_table();

    let mut winter = decl_eq(&[("季節", "冬")]);

    // Only the winter scene matches; repeated calls keep returning it
    for _ in 0..3 {
        let id = table
            .resolve_scene_id_unified("", "挨拶", Some(&mut winter))
            .unwrap();
        let scene = table.get_scene(id).unwrap();
        assert_eq!(scene.attributes, vec![season("冬")]);
    }
}

#[test]
fn test_resolve_scene_id_unified_attribute_filter_no_match() {
    let mut table = build_filtered_table();

    let mut summer = decl_eq(&[("季節", "夏")]);

    // Candidates exist (prefix matches) but the predicate rejects all of them
    let result = table.resolve_scene_id_unified("", "挨拶", Some(&mut summer));
    match result {
        Err(SceneTableError::NoMatchingScene { scene }) => assert_eq!(scene, "挨拶"),
        other => panic!("Expected NoMatchingScene, got {:?}", other.map(|_| ())),
    }
}

/// 宣言の項目だけで AND 一致する。同じ名前の条件・フラグは比べない。
/// 整数になった小数は整数の表記（`2`）で比べる（2.5・7.3・4.2）
#[test]
fn test_resolve_scene_id_attribute_filter_and_over_decl_items() {
    use crate::registry::{CmpOp, SceneAttr, SceneAttrValue};

    let level = |f: f64| SceneAttr::Decl {
        key: "段".to_string(),
        value: SceneAttrValue::from_f64(f),
    };
    let mut registry = SceneRegistry::new();
    // 0: 季節=冬 だけ（段が無い）
    registry.register_global("挨拶", vec![season("冬")]);
    // 1: 季節=冬・段=2（小数 2.0 から）→ 一致
    registry.register_global("挨拶", vec![season("冬"), level(2.0)]);
    // 2: 季節=春・段=2
    registry.register_global("挨拶", vec![season("春"), level(2.0)]);
    // 3: 段 は条件とフラグだけ（宣言ではない）
    registry.register_global(
        "挨拶",
        vec![
            season("冬"),
            SceneAttr::Cond {
                key: "段".to_string(),
                op: CmpOp::Eq,
                value: SceneAttrValue::Integer(2),
            },
            SceneAttr::Flag {
                key: "段".to_string(),
            },
        ],
    );

    let mut cond = decl_eq(&[("季節", "冬"), ("段", "2")]);

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::from_scene_registry(registry, selector);
    for _ in 0..3 {
        let id = table
            .resolve_scene_id_unified("", "挨拶", Some(&mut cond))
            .unwrap();
        assert_eq!(id, SceneId(1));
    }
}

/// 述語なしは属性があっても候補を減らさない（2.5・7.3）
#[test]
fn test_resolve_scene_id_empty_filter_keeps_all_candidates() {
    let mut registry = SceneRegistry::new();
    registry.register_global("挨拶", vec![season("春")]);
    registry.register_global("挨拶", Vec::new());
    registry.register_global("挨拶", vec![season("冬")]);

    let selector = Box::new(MockRandomSelector::new(vec![0, 0, 0]));
    let mut table = SceneTable::from_scene_registry(registry, selector);
    let mut seen: Vec<SceneId> = (0..3)
        .map(|_| table.resolve_scene_id_unified("", "挨拶", None).unwrap())
        .collect();
    seen.sort_by_key(|id| id.0);
    assert_eq!(seen, vec![SceneId(0), SceneId(1), SceneId(2)]);
}

#[test]
fn test_get_scene_out_of_range_returns_none() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    registry.register_global("会話", Vec::new());

    let selector = Box::new(MockRandomSelector::new(vec![0]));
    let table = SceneTable::from_scene_registry(registry, selector);

    assert!(table.get_scene(SceneId(0)).is_some());
    assert!(table.get_scene(SceneId(99)).is_none());
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
        prefix_index: BTreeMap::from([("OnTalk".to_string(), vec![SceneId(0), SceneId(1)])]),
        cache: HashMap::new(),
        random_selector: selector,
    };

    // Consume the first candidate of the cycle
    let first = table.resolve_scene_id_unified("", "OnTalk", None).unwrap();
    assert_eq!(first, SceneId(0));

    // Replace selector — cache must be cleared, cycle restarts at the beginning
    table.replace_selector(Box::new(MockRandomSelector::new(vec![0])));
    let after = table.resolve_scene_id_unified("", "OnTalk", None).unwrap();
    assert_eq!(
        after,
        SceneId(0),
        "cache was not cleared by replace_selector"
    );

    // Labels and prefix_index stay intact
    assert_eq!(table.labels.len(), 2);
}

#[test]
fn test_from_scene_registry_groups_same_name_scenes() {
    use crate::registry::SceneRegistry;

    // Two scenes with the same name share the search-key prefix and both
    // are reachable via sequential consumption
    let mut registry = SceneRegistry::new();
    registry.register_global("会話", Vec::new()); // 会話_1
    registry.register_global("会話", Vec::new()); // 会話_2

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::from_scene_registry(registry, selector);

    let id1 = table.resolve_scene_id_unified("", "会話", None).unwrap();
    let id2 = table.resolve_scene_id_unified("", "会話", None).unwrap();
    assert_ne!(id1, id2, "sequential consumption must visit both scenes");
}

#[test]
fn test_resolve_scene_id_unified_cache_key_includes_module() {
    use crate::registry::SceneRegistry;

    // Create registry with same-named local scenes in different modules
    let mut registry = SceneRegistry::new();
    let (_, counter1) = registry.register_global("会話A", Vec::new());
    registry.register_local("選択肢", "会話A", counter1, 1, Vec::new());
    let (_, counter2) = registry.register_global("会話B", Vec::new());
    registry.register_local("選択肢", "会話B", counter2, 1, Vec::new());

    let selector = Box::new(MockRandomSelector::new(vec![]));
    let mut table = SceneTable::from_scene_registry(registry, selector);

    // Resolve from 会話A_1
    let result_a = table.resolve_scene_id_unified("会話A_1", "選択肢", None);
    assert!(result_a.is_ok());

    // Resolve from 会話B_1 - should use different cache key
    let result_b = table.resolve_scene_id_unified("会話B_1", "選択肢", None);
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
        registry.register_global(name, Vec::new());
    }
    SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(selector)))
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
            .resolve_scene_id_exact("", "つっつき反応胸", None)
            .unwrap();
        assert_eq!(scene_name(&table, id), "つっつき反応胸");
    }

    // 頭だけの名前では見つからない（前方一致しない）
    let result = table.resolve_scene_id_exact("", "つっつき反応", None);
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
        .labels
        .iter()
        .filter(|s| s.name == "撫で反応頭")
        .map(|s| s.id)
        .collect();
    assert_eq!(expected.len(), 3);

    for _cycle in 0..2 {
        let mut picked: Vec<SceneId> = (0..3)
            .map(|_| {
                table
                    .resolve_scene_id_exact("", "撫で反応頭", None)
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
    let mut table = build_table(&["会話", "会話続き"], vec![]);

    let first = table.resolve_scene_id_unified("", "会話", None).unwrap();
    assert_eq!(scene_name(&table, first), "会話");

    for _ in 0..2 {
        let id = table.resolve_scene_id_exact("", "会話", None).unwrap();
        assert_eq!(scene_name(&table, id), "会話");
    }

    // 完全一致の呼び出しが前方一致の巡回を進めていない
    let second = table.resolve_scene_id_unified("", "会話", None).unwrap();
    assert_eq!(scene_name(&table, second), "会話続き");
}

#[test]
fn test_resolve_scene_id_exact_local_scope() {
    use crate::registry::SceneRegistry;

    let mut registry = SceneRegistry::new();
    let (_, counter) = registry.register_global("会話", Vec::new());
    registry.register_local("選択肢続き", "会話", counter, 1, Vec::new());
    registry.register_local("選択肢", "会話", counter, 2, Vec::new());
    registry.register_global("選択肢", Vec::new());
    let mut table =
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![0])));

    // ローカルの範囲: 頭が同じローカルも、同じ名前のグローバルも選ばない
    for _ in 0..3 {
        let id = table
            .resolve_scene_id_exact("会話_1", "選択肢", None)
            .unwrap();
        let scene = table.get_scene(id).unwrap();
        assert_eq!(scene.name, "選択肢");
        assert_eq!(scene.scope, SceneScope::Local);
    }

    // グローバルの範囲: ローカルを選ばない
    let id = table.resolve_scene_id_exact("", "選択肢", None).unwrap();
    assert_eq!(table.get_scene(id).unwrap().scope, SceneScope::Global);

    // ローカルに無い名前はグローバルへ落ちない
    let result = table.resolve_scene_id_exact("会話_1", "会話", None);
    assert!(matches!(result, Err(SceneTableError::SceneNotFound { .. })));
}

#[test]
fn test_resolve_scene_id_exact_empty_name_is_invalid_scene() {
    let mut table = build_table(&["会話"], vec![0]);
    for module in ["", "会話_1"] {
        let result = table.resolve_scene_id_exact(module, "", None);
        match result {
            Err(SceneTableError::InvalidScene { scene }) => assert_eq!(scene, ""),
            other => panic!("Expected InvalidScene, got {:?}", other.map(|_| ())),
        }
    }
}

#[test]
fn test_resolve_scene_id_exact_attribute_filter() {
    let mut table = build_filtered_table();

    let mut winter = decl_eq(&[("季節", "冬")]);
    let id = table
        .resolve_scene_id_exact("", "挨拶", Some(&mut winter))
        .unwrap();
    assert_eq!(table.get_scene(id).unwrap().attributes, vec![season("冬")]);

    let mut summer = decl_eq(&[("季節", "夏")]);
    let result = table.resolve_scene_id_exact("", "挨拶", Some(&mut summer));
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
    let (_, c1) = registry.register_global("会話", Vec::new());
    registry.register_local("挨拶", "会話", c1, 1, Vec::new());
    let (_, c2) = registry.register_global("会話", Vec::new());
    registry.register_local("挨拶続き", "会話", c2, 1, Vec::new());
    let (_, c3) = registry.register_global("会話", Vec::new());
    registry.register_local("挨拶", "会話", c3, 1, Vec::new());
    registry.register_global("雑談", Vec::new());
    SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![])))
}

fn owner_name(table: &SceneTable, id: SceneId) -> String {
    let scene = table.get_scene(id).unwrap();
    assert_eq!(scene.scope, SceneScope::Global);
    scene.fn_name.split_once("::").unwrap().0.to_string()
}

#[test]
fn test_resolve_global_owning_local_exact_picks_only_owners_per_cycle() {
    let mut table = build_owning_table();

    for _cycle in 0..2 {
        let mut picked: Vec<String> = (0..2)
            .map(|_| {
                let id = table
                    .resolve_global_owning_local_exact("会話", "挨拶", None)
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

    // 大域が無い
    let result = table.resolve_global_owning_local_exact("存在しない", "挨拶", None);
    match result {
        Err(SceneTableError::SceneNotFound { scene }) => assert_eq!(scene, "存在しない"),
        other => panic!("Expected SceneNotFound, got {:?}", other),
    }

    // 大域はあるがどれも持たない（頭が同じローカル「挨拶続き」も数えない）
    assert_eq!(
        table
            .resolve_global_owning_local_exact("会話", "挨", None)
            .unwrap(),
        None
    );
    assert_eq!(
        table
            .resolve_global_owning_local_exact("雑談", "挨拶", None)
            .unwrap(),
        None
    );

    // 名前が空
    for (name, local) in [("", "挨拶"), ("会話", "")] {
        let result = table.resolve_global_owning_local_exact(name, local, None);
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

    let mut owning = Vec::new();
    let mut exact = Vec::new();
    for _ in 0..3 {
        let id = table
            .resolve_global_owning_local_exact("会話", "挨拶", None)
            .unwrap()
            .unwrap();
        owning.push(owner_name(&table, id));
        let id = table.resolve_scene_id_exact("", "会話", None).unwrap();
        exact.push(owner_name(&table, id));
    }
    assert_eq!(owning, vec!["会話_1", "会話_3", "会話_1"]);
    assert_eq!(exact, vec!["会話_1", "会話_2", "会話_3"]);
}
