//! `scene_join` のファイルごとの突き合わせの単体テスト（scene-identity-format・4.1/4.3/4.5）。
//!
//! `build_scene_index` 本体は runtime（シーン表・関数の定義元チャンク）を要するため統合
//! テスト（`tests/scene_identity_index_test.rs`）で観測する。ここでは Lua を使わずに、
//! ファイルごとの実行時の登録名と記録から索引を作る [`join_records`] を固定する。

use std::collections::HashMap;

use super::*;
use crate::debug::source_map::SceneIdentity;

fn records(list: &[(&str, &str, u32)]) -> HashMap<String, Vec<(String, u32)>> {
    let mut m: HashMap<String, Vec<(String, u32)>> = HashMap::new();
    for (file, key, line) in list {
        m.entry(file.to_string())
            .or_default()
            .push((key.to_string(), *line));
    }
    m
}

fn names(list: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
    list.iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect()
}

fn ident(scene_id: &str, parent: Option<&str>) -> Option<SceneIdentity> {
    Some(SceneIdentity {
        scene_id: scene_id.to_string(),
        parent: parent.map(str::to_string),
    })
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

/// `＊章` を 11 個と `＊章11` を 1 個: 末尾の数字を推測しないので取り違えない（4.1）。
#[test]
fn trailing_digit_names_join_without_confusion() {
    let mut list: Vec<(&str, String, u32)> = (1..=11)
        .map(|k| ("a.pasta", format!("G:章#{k}"), k as u32 * 10))
        .collect();
    list.push(("a.pasta", "G:章11#1".to_string(), 200));
    let list: Vec<(&str, &str, u32)> = list.iter().map(|(f, k, l)| (*f, k.as_str(), *l)).collect();

    let mut runtime: Vec<String> = (1..=11).map(|k| format!("章_{k}")).collect();
    runtime.push("章11_1".to_string());
    let runtime: Vec<&str> = runtime.iter().map(String::as_str).collect();

    let index = join_records(
        &records(&list),
        &names(&[("a.pasta", &runtime)]),
        &HashMap::new(),
    );

    assert_eq!(index.scene_at("a.pasta", 200), ident("章11_1", None));
    assert_eq!(index.scene_at("a.pasta", 110), ident("章_11", None));
    assert_eq!(index.scene_at("a.pasta", 10), ident("章_1", None));
}

/// 実行時に無い記録は索引に入らない（4.3）。分けられない名前・ファイルの引けない
/// シーンは突き合わせに使わない。
#[test]
fn records_without_runtime_scene_are_not_indexed() {
    let index = join_records(
        &records(&[
            ("a.pasta", "G:会話#1", 1),
            ("a.pasta", "G:会話#2", 5),
            ("b.pasta", "G:雑談#1", 1),
        ]),
        // a.pasta の実行時には 会話 が 1 つだけ。b.pasta は定義元が引けない（無い）。
        &names(&[("a.pasta", &["会話_1", "加算ループ"])]),
        &HashMap::new(),
    );

    assert_eq!(index.scene_at("a.pasta", 1), ident("会話_1", None));
    // 2 つ目の 会話 は索引に無い（1 つ目は 4 行目で終わり、下方にも無い）。
    assert_eq!(index.scene_at("a.pasta", 5), None);
    assert_eq!(index.scene_at("b.pasta", 1), None);
}

/// 2 ファイルの記録がどちらも `G:雑談#1` のとき、それぞれ自分のファイルの登録名に
/// 解決する（4.5）。ローカルの親も自分のファイルの登録名になる。
#[test]
fn same_name_in_two_files_resolves_to_own_file() {
    let index = join_records(
        &records(&[
            ("a.pasta", "G:雑談#1", 1),
            ("a.pasta", "L:雑談#1:__start__", 1),
            ("a.pasta", "L:雑談#1:挨拶_1", 3),
            ("b.pasta", "G:雑談#1", 1),
            ("b.pasta", "L:雑談#1:__start__", 1),
            ("b.pasta", "L:雑談#1:挨拶_1", 3),
        ]),
        &names(&[("a.pasta", &["雑談_1"]), ("b.pasta", &["雑談_2"])]),
        &names(&[
            ("雑談_1", &["__start__", "挨拶_1"]),
            ("雑談_2", &["__start__", "挨拶_1"]),
        ]),
    );

    assert_eq!(index.scene_at("a.pasta", 1), ident("雑談_1", None));
    assert_eq!(index.scene_at("b.pasta", 1), ident("雑談_2", None));
    assert_eq!(
        index.scene_at("a.pasta", 3),
        ident("挨拶_1", Some("雑談_1"))
    );
    assert_eq!(
        index.scene_at("b.pasta", 3),
        ident("挨拶_1", Some("雑談_2"))
    );
}

/// ローカルは親の配下に同じ関数名があるときだけ採る。`__start__` は索引に入れない。
#[test]
fn local_requires_function_under_parent() {
    let index = join_records(
        &records(&[
            ("a.pasta", "G:会話#1", 1),
            ("a.pasta", "L:会話#1:__start__", 1),
            ("a.pasta", "L:会話#1:不在_1", 3),
        ]),
        &names(&[("a.pasta", &["会話_1"])]),
        &names(&[("会話_1", &["__start__"])]),
    );

    // __start__ はグローバル identity が覆う。不在_1 は索引に無く、グローバルに含まれる。
    assert_eq!(index.scene_at("a.pasta", 1), ident("会話_1", None));
    assert_eq!(index.scene_at("a.pasta", 3), ident("会話_1", None));
}
