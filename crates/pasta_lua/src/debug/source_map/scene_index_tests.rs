//! `SceneIdentityIndex` の単体テスト（task 1.1・requirements 2.1/2.2/2.5/2.6/3.2）。
//!
//! 観測対象（5 ケース）:
//! 1. 包含 1 件 → そのシーンを確定（2.1）。
//! 2. global ⊃ local 包含 → 最内（local）を選択（2.2）。
//! 3. 包含なし → クリック行と同じか下方の最近接シーン（後方フォールバック・2.5）。
//! 4. 最終シーンより後ろ → 未検出（2.6）。
//! 5. 空ファイル → 未検出。
//!
//! task 2.2 拡張: `scene_at` は **(scene_id, parent)**（[`SceneIdentity`]）を返す。
//! global は `parent=None`、local は `parent=Some(親グローバル)` を併せて表明する。

use super::*;

/// テスト用の索引を構築するヘルパー。
///
/// 入れ子レベル（global=0, local=1, ...）と宣言開始行を与え、end_line は
/// 「次の同レベル以上のシーン宣言の直前」を呼び出し側が明示して投入する。
fn idx() -> SceneIdentityIndexBuilder {
    let mut b = SceneIdentityIndex::builder();
    b.insert_file("a.pasta");
    b
}

/// `(scene_id, parent)` を簡潔に組む比較用ヘルパー。
fn ident(scene_id: &str, parent: Option<&str>) -> SceneIdentity {
    SceneIdentity {
        scene_id: scene_id.to_string(),
        parent: parent.map(|p| p.to_string()),
    }
}

#[test]
fn containment_single_scene_resolves_that_scene() {
    // global シーン 会話1 が 行 10..=20（end_line=20・parent=None）。
    let mut b = idx();
    b.add_scene("a.pasta", "会話1", None, 10, 20, 0);
    let index = b.finish();

    // 範囲内のクリック → 会話1（global なので parent=None）。
    assert_eq!(index.scene_at("a.pasta", 10), Some(ident("会話1", None)));
    assert_eq!(index.scene_at("a.pasta", 15), Some(ident("会話1", None)));
    assert_eq!(index.scene_at("a.pasta", 20), Some(ident("会話1", None)));
}

#[test]
fn innermost_local_preferred_over_enclosing_global() {
    // global 会話1: 行 10..=40（parent=None）。
    // その中に local 挨拶_1: 行 20..=30（parent=Some(会話1)）。
    let mut b = idx();
    b.add_scene("a.pasta", "会話1", None, 10, 40, 0);
    b.add_scene("a.pasta", "挨拶_1", Some("会話1"), 20, 30, 1);
    let index = b.finish();

    // local 範囲外（global のみ）→ global（parent=None）。
    assert_eq!(index.scene_at("a.pasta", 15), Some(ident("会話1", None)));
    // local 範囲内（両方包含）→ 最内 local 優先（parent=Some(会話1)）。
    assert_eq!(
        index.scene_at("a.pasta", 20),
        Some(ident("挨拶_1", Some("会話1")))
    );
    assert_eq!(
        index.scene_at("a.pasta", 25),
        Some(ident("挨拶_1", Some("会話1")))
    );
    assert_eq!(
        index.scene_at("a.pasta", 30),
        Some(ident("挨拶_1", Some("会話1")))
    );
    // local 範囲後・global 範囲内 → global へ戻る。
    assert_eq!(index.scene_at("a.pasta", 35), Some(ident("会話1", None)));
}

#[test]
fn backward_fallback_to_nearest_scene_at_or_below() {
    // ヘッダ部（行 1..9）はどのシーンにも属さない。
    // 会話1: 10..=20、会話2: 30..=40（いずれも global・parent=None）。
    let mut b = idx();
    b.add_scene("a.pasta", "会話1", None, 10, 20, 0);
    b.add_scene("a.pasta", "会話2", None, 30, 40, 0);
    let index = b.finish();

    // ヘッダ（行 5）→ 下方最近接 会話1。
    assert_eq!(index.scene_at("a.pasta", 5), Some(ident("会話1", None)));
    // 宣言行ちょうど（行 10）は包含で 会話1。
    assert_eq!(index.scene_at("a.pasta", 10), Some(ident("会話1", None)));
    // シーン間の隙間（行 25・どこにも包含されない）→ 下方最近接 会話2。
    assert_eq!(index.scene_at("a.pasta", 25), Some(ident("会話2", None)));
}

#[test]
fn not_found_after_last_scene() {
    // 会話1: 10..=20 のみ。最後のシーンより後ろは未検出。
    let mut b = idx();
    b.add_scene("a.pasta", "会話1", None, 10, 20, 0);
    let index = b.finish();

    // 最終シーンの終端より後ろ（下方に有効シーンなし）→ 未検出。
    assert_eq!(index.scene_at("a.pasta", 21), None);
    assert_eq!(index.scene_at("a.pasta", 100), None);
}

#[test]
fn empty_file_not_found() {
    // ファイルは登録されているがシーンが 1 件も無い。
    let index = idx().finish();
    assert_eq!(index.scene_at("a.pasta", 1), None);
    assert_eq!(index.scene_at("a.pasta", 50), None);
}

#[test]
fn unknown_file_not_found() {
    let mut b = idx();
    b.add_scene("a.pasta", "会話1", None, 10, 20, 0);
    let index = b.finish();
    // 登録外ファイルは未検出（誤解決しない）。
    assert_eq!(index.scene_at("b.pasta", 15), None);
}
