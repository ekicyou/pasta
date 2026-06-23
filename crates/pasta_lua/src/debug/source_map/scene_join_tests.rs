//! `scene_join` の純パーサ・突合ヘルパーの単体テスト（task 2.2）。
//!
//! `build_scene_index` 本体は runtime（`collect_scenes`）を要するため統合テスト
//! （`tests/scene_identity_index_test.rs`）で観測する。ここでは join_key パースと
//! runtime global_name 分解の境界を固定する。

use super::*;

#[test]
fn split_runtime_global_strips_trailing_counter() {
    assert_eq!(split_runtime_global("会話1"), Some(("会話", 1)));
    assert_eq!(split_runtime_global("メイン12"), Some(("メイン", 12)));
    // 末尾が数字でない → None。
    assert_eq!(split_runtime_global("会話"), None);
    // 純数字（base 非空条件に反する）→ None。
    assert_eq!(split_runtime_global("123"), None);
    // 空文字 → None。
    assert_eq!(split_runtime_global(""), None);
}

#[test]
fn parse_base_counter_splits_on_hash() {
    assert_eq!(parse_base_counter("会話#1"), ("会話", Some(1)));
    assert_eq!(parse_base_counter("メイン#42"), ("メイン", Some(42)));
    // `#` 無し → counter None。
    assert_eq!(parse_base_counter("会話"), ("会話", None));
    // counter が非数値 → None。
    assert_eq!(parse_base_counter("会話#x"), ("会話", None));
}

#[test]
fn parse_and_join_global_matches_runtime_identity() {
    let mut globals: std::collections::HashMap<(String, u32), String> = Default::default();
    globals.insert(("会話".to_string(), 1), "会話1".to_string());
    let locals: std::collections::HashMap<String, Vec<String>> = Default::default();

    let r = parse_and_join("G:会話#1", 3, &globals, &locals);
    assert_eq!(r.level, 0);
    assert_eq!(r.start_line, 3);
    assert_eq!(r.scene_id.as_deref(), Some("会話1"));
    assert_eq!(r.parent, None);

    // 対応 runtime identity が無い global は突合失敗（scene_id None）。
    let miss = parse_and_join("G:不在#1", 3, &globals, &locals);
    assert_eq!(miss.scene_id, None);
}

#[test]
fn parse_and_join_local_matches_runtime_identity_and_parent() {
    let mut globals: std::collections::HashMap<(String, u32), String> = Default::default();
    globals.insert(("会話".to_string(), 1), "会話1".to_string());
    let mut locals: std::collections::HashMap<String, Vec<String>> = Default::default();
    locals.insert(
        "会話1".to_string(),
        vec!["__start__".to_string(), "挨拶_1".to_string()],
    );

    let r = parse_and_join("L:会話#1:挨拶_1", 5, &globals, &locals);
    assert_eq!(r.level, 1);
    assert_eq!(r.start_line, 5);
    assert_eq!(r.scene_id.as_deref(), Some("挨拶_1"));
    assert_eq!(r.parent.as_deref(), Some("会話1"));

    // __start__ は索引へ入れない（scene_id None）: グローバル本体領域はグローバル
    // identity が覆い、global kick が __start__ を再生する（kick 決定ノート）。親は解決可。
    let start = parse_and_join("L:会話#1:__start__", 3, &globals, &locals);
    assert_eq!(start.scene_id, None);
    assert_eq!(start.parent.as_deref(), Some("会話1"));

    // 親配下に無い fn_name は突合失敗（scene_id None・親は解決できても本体不一致）。
    let miss = parse_and_join("L:会話#1:不在_1", 7, &globals, &locals);
    assert_eq!(miss.scene_id, None);
    assert_eq!(miss.parent.as_deref(), Some("会話1"));
}
