//! 抽出した候補からの取り出し（重みの並び）のテスト

use super::*;
use crate::registry::random::MockRandomSelector;
use crate::registry::{SceneAttr, SceneAttrValue, SceneRegistry};
use std::sync::{Arc, Mutex};

/// シャッフルに渡された並びを控える（並べ方は MockRandomSelector に任せる）
struct RecordingSelector {
    inner: MockRandomSelector,
    calls: Arc<Mutex<Vec<Vec<usize>>>>,
}

impl RandomSelector for RecordingSelector {
    fn shuffle_usize(&mut self, items: &mut [usize]) {
        self.calls.lock().unwrap().push(items.to_vec());
        self.inner.shuffle_usize(items);
    }
}

fn decl(key: &str, n: i64) -> SceneAttr {
    SceneAttr::Decl {
        key: key.to_string(),
        value: SceneAttrValue::Integer(n),
    }
}

/// 大域「話」を属性つきで並べる。SceneId は登録順
fn table_with(
    attrs: Vec<Vec<SceneAttr>>,
    sequence: Vec<usize>,
) -> (SceneTable, Arc<Mutex<Vec<Vec<usize>>>>) {
    let mut registry = SceneRegistry::new();
    for a in attrs {
        registry.register_global("話", a);
    }
    let calls = Arc::new(Mutex::new(Vec::new()));
    let selector = Box::new(RecordingSelector {
        inner: MockRandomSelector::new(sequence),
        calls: calls.clone(),
    });
    (SceneTable::from_scene_registry(registry, selector), calls)
}

fn pick_n(table: &mut SceneTable, n: usize) -> Vec<usize> {
    (0..n)
        .map(|_| table.resolve_scene_id_unified("", "話", None).unwrap().0)
        .collect()
}

/// 重み 3・2・1 の候補は 1 巡にそれぞれ 3・2・1 回出る（1.1・1.2）
#[test]
fn test_weights_put_scene_n_times_in_round() {
    let (mut table, calls) = table_with(
        vec![vec![decl("重み", 3)], vec![decl("weight", 2)], vec![]],
        vec![],
    );
    let mut round = pick_n(&mut table, 6);
    // 並び（シャッフル前）は ID を重みの数だけ並べたもの
    assert_eq!(calls.lock().unwrap().as_slice(), &[vec![0, 0, 0, 1, 1, 2]]);
    round.sort();
    assert_eq!(round, vec![0, 0, 0, 1, 1, 2]);
    // 7 回目で始め直す（巡の数え方は今のまま）
    pick_n(&mut table, 1);
    assert_eq!(calls.lock().unwrap().len(), 2);
}

/// 重みは `＆優先` の段とも組み合わさる: 優先の降順の安定な並べ替えは今のまま（9.3）
#[test]
fn test_weights_keep_priority_order() {
    let (mut table, _) = table_with(vec![vec![decl("重み", 2)], vec![decl("優先", 1)]], vec![]);
    assert_eq!(pick_n(&mut table, 3), vec![1, 0, 0]);
}

/// 重み 1 だけの並びは今と同じ配列・同じ乱数の消費（9.1）
#[test]
fn test_weight_one_keeps_todays_round_and_rng_use() {
    let (mut table, calls) = table_with(vec![vec![], vec![decl("重み", 1)], vec![]], vec![2, 0, 1]);
    assert_eq!(pick_n(&mut table, 6), vec![2, 0, 1, 2, 0, 1]);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[vec![0, 1, 2], vec![0, 1, 2]],
        "シャッフルは作るときと始め直すときだけ・渡す並びは候補の ID のまま"
    );
}

/// 条件の述語は重みで増えた札にも同じく効く（`ok` は集合）
#[test]
fn test_weights_with_condition() {
    let (mut table, calls) = table_with(vec![vec![decl("重み", 2)], vec![]], vec![]);
    let mut only_zero = |s: &SceneInfo| s.id.0 == 0;
    let first = table
        .resolve_scene_id_unified("", "話", Some(&mut only_zero))
        .unwrap();
    let mut only_zero = |s: &SceneInfo| s.id.0 == 0;
    let second = table
        .resolve_scene_id_unified("", "話", Some(&mut only_zero))
        .unwrap();
    assert_eq!((first.0, second.0), (0, 0));
    assert_eq!(
        calls.lock().unwrap().len(),
        1,
        "同じ巡の 2 枚目なので始め直さない"
    );
}
