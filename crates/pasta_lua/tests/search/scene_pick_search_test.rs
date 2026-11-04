//! 順番が使えずシャッフルへ戻ったときの注意のログと、番号の重なりの読み込みエラー
//! （scene-pick-modes 要件 3.1・3.3）。
//!
//! - 混ざり・前方一致の番号の重なりで、`pasta_lua::search` の warn に探した名前と理由が入り、
//!   乱数の差し替え（シャッフル）のとおりに選ばれる。
//! - 同じ名前のシーンの番号の重なりは、シーンの確定で読み込みを止める。

use crate::common::create_temp_with_pasta;
use pasta_core::registry::{SceneAttr, SceneAttrValue, SceneRegistry, WordDefRegistry};
use pasta_lua::loader::PastaLoader;
use pasta_lua::search::SearchContext;
use tracing_test::traced_test;

fn order(n: i64) -> Vec<SceneAttr> {
    vec![SceneAttr::Decl {
        key: "順番".to_string(),
        value: SceneAttrValue::Integer(n),
    }]
}

/// `scenes` を登録し、並びを `sequence` で決める検索の部品
fn context(scenes: Vec<(&str, Vec<SceneAttr>)>, sequence: Vec<usize>) -> SearchContext {
    let mut registry = SceneRegistry::new();
    for (name, attrs) in scenes {
        registry.register_global(name, attrs);
    }
    let mut ctx = SearchContext::new(registry, WordDefRegistry::new());
    ctx.set_scene_selector(Some(sequence)).unwrap();
    ctx
}

#[traced_test]
#[test]
fn test_order_mixed_warns_and_shuffles() {
    // 会話_1（＆順番：10）と会話_2（順番なし）。並びは 2 つ目を先に
    let mut ctx = context(vec![("会話", order(10)), ("会話", vec![])], vec![1, 0]);
    let picked = ctx.search_scene("会話", None).unwrap().unwrap();
    assert_eq!(picked.0, "会話_2", "シャッフルの並びのとおりに選ぶ");
    assert!(logs_contain("pasta_lua::search"));
    assert!(logs_contain("name=会話"));
    assert!(logs_contain("順番の有るものと無いものが混ざっている"));
}

#[traced_test]
#[test]
fn test_prefix_duplicate_order_warns_and_shuffles() {
    // 前方一致で 会話・会話A（どちらも 10）・会話B（5）が集まる。番号の順なら 会話B_1
    let mut ctx = context(
        vec![
            ("会話", order(10)),
            ("会話A", order(10)),
            ("会話B", order(5)),
        ],
        vec![0],
    );
    let picked = ctx.search_scene("会話", None).unwrap().unwrap();
    assert_eq!(picked.0, "会話_1", "番号の順でなくシャッフルで選ぶ");
    assert!(logs_contain("name=会話"));
    assert!(logs_contain("同じ番号 10 が 2 つ以上ある"));
}

#[traced_test]
#[test]
fn test_order_mixed_warns_on_local_exact_owner_pick() {
    // 大域 会話 の 2 つ（1 つだけ ＆順番）がどちらもローカル 選択 を持つ
    let mut registry = SceneRegistry::new();
    let (_, c1) = registry.register_global("会話", order(10));
    registry.register_local("選択", "会話", c1, 1, Vec::new());
    let (_, c2) = registry.register_global("会話", Vec::new());
    registry.register_local("選択", "会話", c2, 1, Vec::new());
    let mut ctx = SearchContext::new(registry, WordDefRegistry::new());
    ctx.set_scene_selector(Some(vec![1, 0])).unwrap();

    ctx.search_local_scene_exact("会話", "選択").unwrap();
    assert!(logs_contain("name=会話"));
    assert!(logs_contain("順番の有るものと無いものが混ざっている"));
}

#[traced_test]
#[test]
fn test_all_ordered_does_not_warn() {
    let mut ctx = context(vec![("会話", order(20)), ("会話", order(10))], vec![0]);
    let picked = ctx.search_scene("会話", None).unwrap().unwrap();
    assert_eq!(picked.0, "会話_2", "番号の小さいほうから");
    assert!(!logs_contain("順番の有るものと無いもの"));
    assert!(!logs_contain("2 つ以上ある"));
}

#[test]
fn test_check_scene_orders_reports_first_duplicate() {
    let ctx = context(
        vec![
            ("会話", order(10)),
            ("会話", order(10)),
            ("会話", order(20)),
        ],
        vec![],
    );
    let err = ctx.check_scene_orders().unwrap_err().to_string();
    assert!(err.contains("`＆順番` の番号 10"), "{err}");
    assert!(err.contains("会話_1::__start__"), "{err}");
    assert!(err.contains("会話_2::__start__"), "{err}");
    assert!(!err.contains("会話_3"), "{err}");

    let ok = context(vec![("会話", order(10)), ("会話", order(20))], vec![]);
    assert!(ok.check_scene_orders().is_ok());
}

#[test]
fn test_duplicate_order_stops_loading() {
    let temp = create_temp_with_pasta(
        "＊挨拶\n  ＆順番：１０\n  さくら：あ\n\n＊挨拶\n  ＆順番：１０\n  さくら：い\n",
    );
    let err = match PastaLoader::load(temp.path()) {
        Ok(_) => panic!("番号の重なりは読み込みエラーになるべし"),
        Err(e) => format!("{e:?}"),
    };
    assert!(err.contains("pasta.scene_dic"), "{err}");
    assert!(
        err.contains(r#"DuplicateOrder { scene: "挨拶", order: 10"#),
        "{err}"
    );
    assert!(err.contains("挨拶_2::__start__"), "{err}");
}
