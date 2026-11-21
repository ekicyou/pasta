//! `SceneIdentityIndex` の単体テスト。

use super::*;

fn idx() -> SceneIdentityIndexBuilder {
    SceneIdentityIndex::builder()
}

fn ident(scene_id: &str, parent: Option<&str>) -> SceneIdentity {
    SceneIdentity {
        scene_id: scene_id.to_string(),
        parent: parent.map(|p| p.to_string()),
    }
}

#[test]
fn containment_single_scene_resolves_that_scene() {
    let mut b = idx();
    b.add_scene("a.pasta", "会話_1", None, 10, 20, 0);
    let index = b.finish();

    assert_eq!(index.scene_at("a.pasta", 10), Some(ident("会話_1", None)));
    assert_eq!(index.scene_at("a.pasta", 15), Some(ident("会話_1", None)));
    assert_eq!(index.scene_at("a.pasta", 20), Some(ident("会話_1", None)));
}

#[test]
fn innermost_local_preferred_over_enclosing_global() {
    let mut b = idx();
    b.add_scene("a.pasta", "会話_1", None, 10, 40, 0);
    b.add_scene("a.pasta", "挨拶_1", Some("会話_1"), 20, 30, 1);
    let index = b.finish();

    assert_eq!(index.scene_at("a.pasta", 15), Some(ident("会話_1", None)));
    assert_eq!(
        index.scene_at("a.pasta", 20),
        Some(ident("挨拶_1", Some("会話_1")))
    );
    assert_eq!(
        index.scene_at("a.pasta", 25),
        Some(ident("挨拶_1", Some("会話_1")))
    );
    assert_eq!(
        index.scene_at("a.pasta", 30),
        Some(ident("挨拶_1", Some("会話_1")))
    );
    assert_eq!(index.scene_at("a.pasta", 35), Some(ident("会話_1", None)));
}

#[test]
fn backward_fallback_to_nearest_scene_at_or_below() {
    // ヘッダ部（行 1..9）はどのシーンにも属さない。
    let mut b = idx();
    b.add_scene("a.pasta", "会話_1", None, 10, 20, 0);
    b.add_scene("a.pasta", "会話_2", None, 30, 40, 0);
    let index = b.finish();

    assert_eq!(index.scene_at("a.pasta", 5), Some(ident("会話_1", None)));
    assert_eq!(index.scene_at("a.pasta", 10), Some(ident("会話_1", None)));
    // シーン間の隙間（行 25・どこにも包含されない）→ 下方最近接 会話_2。
    assert_eq!(index.scene_at("a.pasta", 25), Some(ident("会話_2", None)));
}

#[test]
fn not_found_after_last_scene() {
    let mut b = idx();
    b.add_scene("a.pasta", "会話_1", None, 10, 20, 0);
    let index = b.finish();

    assert_eq!(index.scene_at("a.pasta", 21), None);
    assert_eq!(index.scene_at("a.pasta", 100), None);
}

#[test]
fn empty_file_not_found() {
    let index = idx().finish();
    assert_eq!(index.scene_at("a.pasta", 1), None);
    assert_eq!(index.scene_at("a.pasta", 50), None);
}

#[test]
fn unknown_file_not_found() {
    let mut b = idx();
    b.add_scene("a.pasta", "会話_1", None, 10, 20, 0);
    let index = b.finish();
    assert_eq!(index.scene_at("b.pasta", 15), None);
}
