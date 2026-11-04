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

fn order(n: i64) -> Vec<SceneAttr> {
    vec![decl("順番", n)]
}

/// 全部に順番があれば番号の順に巡り、最初へ戻る。乱数を使わない（2.1・2.2・2.3・8.4）
#[test]
fn test_order_cycles_without_shuffle() {
    let (mut table, calls) = table_with(vec![order(10), order(20), order(30)], vec![2, 1, 0]);
    assert_eq!(pick_n(&mut table, 4), vec![0, 1, 2, 0]);
    assert!(calls.lock().unwrap().is_empty(), "並びを作らない");
    assert_eq!(table.take_pick_warning(), None);
}

/// 番号は飛んでいてよく、間に足した番号は間に入る。`index` も同じ意味（2.5・2.6）
#[test]
fn test_order_inserted_number_goes_between() {
    let (mut table, _) = table_with(
        vec![order(10), order(20), order(30), vec![decl("index", 15)]],
        vec![],
    );
    assert_eq!(pick_n(&mut table, 5), vec![0, 3, 1, 2, 0]);
}

/// 出す順は番号だけで決まり、登録の順に左右されない（2.7）
#[test]
fn test_order_ignores_registration_order() {
    let (mut table, _) = table_with(vec![order(30), order(10), order(20)], vec![]);
    assert_eq!(pick_n(&mut table, 4), vec![1, 2, 0, 1]);
}

/// 条件で外れた番号は飛ばす（2.8）
#[test]
fn test_order_skips_excluded_by_condition() {
    let (mut table, _) = table_with(vec![order(10), order(20), order(30)], vec![2, 1, 0]);
    let picks: Vec<usize> = (0..3)
        .map(|_| {
            let mut not_20 = |s: &SceneInfo| s.id.0 != 1;
            table
                .resolve_scene_id_unified("", "話", Some(&mut not_20))
                .unwrap()
                .0
        })
        .collect();
    assert_eq!(picks, vec![0, 2, 0]);
}

/// 一部だけに順番があれば理由を控えてシャッフルで選ぶ。控えは読むと空・選ぶたびに空（3.1）
#[test]
fn test_order_mixed_falls_back_to_shuffle() {
    let (mut table, calls) = table_with(vec![order(10), vec![]], vec![1, 0]);
    assert_eq!(pick_n(&mut table, 1), vec![1]);
    assert_eq!(calls.lock().unwrap().len(), 1, "シャッフルで選ぶ");
    assert_eq!(table.take_pick_warning(), Some(PickWarning::OrderMixed));
    assert_eq!(table.take_pick_warning(), None, "読むと空になる");

    // 注意を読まずに次の選択へ: 抽出が順番だけなら注意は残らない
    pick_n(&mut table, 1);
    let mut only_zero = |s: &SceneInfo| s.id.0 == 0;
    let id = table
        .resolve_scene_id_unified("", "話", Some(&mut only_zero))
        .unwrap();
    assert_eq!(id.0, 0);
    assert_eq!(table.take_pick_warning(), None, "選ぶたびに空にする");
}

/// 番号が重なれば理由（番号）を控えてシャッフルで選ぶ（3.1）
#[test]
fn test_order_duplicate_falls_back_to_shuffle() {
    let (mut table, calls) = table_with(vec![order(10), order(10), order(20)], vec![2, 0, 1]);
    assert_eq!(pick_n(&mut table, 1), vec![2]);
    assert_eq!(calls.lock().unwrap().len(), 1);
    assert_eq!(
        table.take_pick_warning(),
        Some(PickWarning::OrderDuplicate { order: 10 })
    );
}

/// どれにも順番が無ければ黙ってシャッフル（9.1）
#[test]
fn test_no_order_no_warning() {
    let (mut table, _) = table_with(vec![vec![], vec![]], vec![]);
    pick_n(&mut table, 1);
    assert_eq!(table.take_pick_warning(), None);
}

/// 乱数の差し替えで覚えた番号も消え、最初からに戻る（2.4）
#[test]
fn test_replace_selector_resets_order() {
    let (mut table, _) = table_with(vec![order(10), order(20), order(30)], vec![2, 1, 0]);
    assert_eq!(pick_n(&mut table, 2), vec![0, 1]);
    table.replace_selector(Box::new(MockRandomSelector::new(vec![2, 1, 0])));
    assert_eq!(pick_n(&mut table, 1), vec![0]);
}

/// 前方一致と完全一致は別に覚える（鍵は並びの控えと同じ）
#[test]
fn test_order_remembered_per_cache_key() {
    let (mut table, _) = table_with(vec![order(10), order(20)], vec![1, 0]);
    assert_eq!(pick_n(&mut table, 1), vec![0]);
    let exact = table.resolve_scene_id_exact("", "話", None).unwrap();
    assert_eq!(exact.0, 0);
    assert_eq!(pick_n(&mut table, 1), vec![1]);
}

/// 同じ名前の中の番号の重なりを、グローバル（別名を置き換えた後の名前）とローカルで組にして返す（3.3）
#[test]
fn test_duplicate_orders_global_and_local() {
    let mut registry = SceneRegistry::new();
    // 別名 `会話` は登録の前に `OnTalk` へ置き換わるので、どちらも `OnTalk` で登録される
    registry.register_global("OnTalk", order(10)); // 0: 会話 と書いたもの
    registry.register_global("OnTalk", order(10)); // 1: OnTalk と書いたもの
    registry.register_global("OnTalk", order(20)); // 2
    registry.register_global("挨拶", order(10)); // 3: 名前が違えば重ならない
    registry.register_local("選択", "話", 1, 1, order(5)); // 4
    registry.register_local("選択", "話", 1, 2, order(5)); // 5
    registry.register_local("選択", "話", 1, 3, order(5)); // 6
    registry.register_local("選択", "話", 2, 1, order(5)); // 7: 親が違えば重ならない
    let table =
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![])));
    assert_eq!(
        table.duplicate_orders(),
        vec![
            OrderDuplicate {
                key: ":話_1:選択".to_string(),
                order: 5,
                scenes: vec![SceneId(4), SceneId(5), SceneId(6)],
            },
            OrderDuplicate {
                key: "OnTalk".to_string(),
                order: 10,
                scenes: vec![SceneId(0), SceneId(1)],
            },
        ]
    );
}

/// 重なりが無ければ空。順番の無いシーンは数えない（3.3）
#[test]
fn test_duplicate_orders_empty_without_overlap() {
    let (table, _) = table_with(vec![order(10), order(20), vec![], vec![]], vec![]);
    assert_eq!(table.duplicate_orders(), vec![]);
}
