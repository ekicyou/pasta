//! 登録名の形式（`{名前}_{通し番号}`）の統合テスト（scene-identity-format）。
//!
//! - `＊A1` と `＊A`×11 が 12 個の別々のシーンとして登録され、`A1_1` と `A_11` の両方が
//!   実行できる（1.3・8.1）。旧形式（`A` ＋ `11` = `A11`）では `A1` の 1 本目と衝突する。
//! - 実行時のシーン表の関数名（`グローバル::ローカル`）の集合が、トランスパイル時のレジストリが
//!   Rust の規則（`SceneRegistry::registered_name`）で組み立てた集合と一致する（6.3・2.12）。
//!   グローバルの登録名は Lua（`SCENE.create_scene`）が、ローカルの関数名は Rust の生成器が作るので、
//!   ここが Rust と Lua の形式、およびレジストリと生成器の通し番号の突き合わせになる。
//! - 照合用の名前が重なるローカルシーン（`・挨拶・1` と `・挨拶_1`）が別々の関数になる（1.7）。

use std::collections::BTreeSet;

use crate::common;
use common::e2e_helpers::create_runtime_with_finalize;
use mlua::Lua;
use pasta_core::registry::{SceneRegistry, WordDefRegistry};
use pasta_dsl::parser::parse_str;
use pasta_lua::{LuaTranspiler, SearchContext};

const FIXTURE: &str = include_str!("../fixtures/scene_identity_format.pasta");

/// フィクスチャをトランスパイル→実行→`finalize_scene` し、ランタイムと、
/// トランスパイル時のシーンレジストリを返す。
fn load_fixture() -> (Lua, SceneRegistry) {
    load_source(FIXTURE)
}

/// `.pasta` のソースをトランスパイル→実行→`finalize_scene` する（`load_fixture` の本体）。
fn load_source(source: &str) -> (Lua, SceneRegistry) {
    let file = parse_str(source, "scene_identity_format.pasta").expect("fixture must parse");
    let mut out = Vec::new();
    let ctx = LuaTranspiler::default()
        .transpile(&file, &mut out)
        .expect("fixture must transpile");

    let lua = create_runtime_with_finalize().unwrap();
    lua.load(String::from_utf8(out).unwrap()).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    (lua, ctx.scene_registry)
}

/// トランスパイル時のレジストリの関数名（`登録名::__start__`・`親の登録名::ローカルの登録名`）の集合。
fn registry_fn_names(registry: &SceneRegistry) -> BTreeSet<String> {
    registry
        .all_scenes()
        .iter()
        .map(|s| s.fn_name.clone())
        .collect()
}

/// 確定後のシーン表（`collect_scenes`）の関数名の集合。
fn runtime_fn_names(lua: &Lua) -> BTreeSet<String> {
    pasta_lua::runtime::finalize::collect_scenes(lua)
        .unwrap()
        .into_iter()
        .map(|(g, l)| format!("{g}::{l}"))
        .collect()
}

/// シーン表から完全一致で引いたシーンを最小の act で実行し、トークの本文をつなげて返す。
fn run_scene(lua: &Lua, global: &str, local: &str) -> String {
    lua.load(
        r#"
        local g, l = ...
        local fn = require("pasta.scene").get(g, l)
        if type(fn) ~= "function" then error("scene not found: " .. g .. "::" .. l) end
        local act = require("pasta.act").new({ ["さくら"] = { name = "さくら" } })
        fn(act)
        local texts = {}
        for _, t in ipairs(act.token) do
            if t.type == "talk" then texts[#texts + 1] = t.text end
        end
        return table.concat(texts)
    "#,
    )
    .call((global, local))
    .unwrap()
}

/// 1.3・8.1: `＊A1`＋`＊A`×11 が 12 個の別々のシーンになり、`A1_1` と `A_11` の両方が実行できる。
#[test]
fn a1_and_eleven_a_scenes_are_twelve_distinct_runnable_scenes() {
    let (lua, _) = load_fixture();
    let scenes = pasta_lua::runtime::finalize::collect_scenes(&lua).unwrap();
    let a_globals: BTreeSet<&str> = scenes
        .iter()
        .map(|(g, _)| g.as_str())
        .filter(|g| g.starts_with('A'))
        .collect();
    assert_eq!(
        a_globals.len(),
        12,
        "＊A1 と ＊A×11 は 12 個のシーンになる: {a_globals:?}"
    );

    assert!(run_scene(&lua, "A1_1", "__start__").contains("A1本体"));
    assert!(run_scene(&lua, "A_11", "__start__").contains("A本体11"));
}

/// 6.3: 実行時の関数名の集合（グローバル・ローカルとも）が、定義から Rust の規則で組み立てた
/// トランスパイル時のレジストリの集合と一致する。
#[test]
fn runtime_registered_names_match_rust_registered_name_rule() {
    let (lua, registry) = load_fixture();
    assert_eq!(
        runtime_fn_names(&lua),
        registry_fn_names(&registry),
        "Lua の登録名・生成器の関数名と Rust の registered_name の形式が一致する"
    );
}

/// 1.7・8.1: 同じグローバルシーンの中の `・挨拶・1` と `・挨拶_1` は照合用の名前（`挨拶_1`）が
/// 重なるが、別々の関数（`挨拶_1_1`・`挨拶_1_2`）になり、どちらも実行できる。
#[test]
fn locals_with_overlapping_sanitized_names_are_distinct_runnable_functions() {
    let source = [
        "＊会話",
        "　さくら：「会話本体」",
        "",
        "　・挨拶・1",
        "　　さくら：「中黒本体」",
        "",
        "　・挨拶_1",
        "　　さくら：「下線本体」",
        "",
    ]
    .join("\n");
    let (lua, registry) = load_source(&source);
    assert!(run_scene(&lua, "会話_1", "挨拶_1_1").contains("中黒本体"));
    assert!(run_scene(&lua, "会話_1", "挨拶_1_2").contains("下線本体"));
    assert_eq!(runtime_fn_names(&lua), registry_fn_names(&registry));
}

/// 2.12・6.3: 開始シーン＋名前の異なるローカルシーン 2 つ＋同名のローカルシーン 2 つを持つ
/// 1 ファイルで、辞書確定前のレジストリと確定後のシーン表の関数名の集合が一致し、
/// 決まった順に選ぶ設定の下で検索の結果も同じになる。
#[test]
fn transpile_time_registry_matches_finalized_scene_table() {
    let source = [
        "＊会話",
        "　さくら：「会話本体」",
        "",
        "　・選択A",
        "　　さくら：「選択A本体」",
        "",
        "　・選択B",
        "　　さくら：「選択B本体」",
        "",
        "　・挨拶",
        "　　さくら：「挨拶本体1」",
        "",
        "　・挨拶",
        "　　さくら：「挨拶本体2」",
        "",
    ]
    .join("\n");
    let (lua, registry) = load_source(&source);
    assert_eq!(
        runtime_fn_names(&lua),
        registry_fn_names(&registry),
        "辞書確定前と確定後の関数名の集合が一致する"
    );

    // (検索する名前, 親の登録名, 呼ぶ回数)
    let queries: [(&str, Option<&str>, usize); 4] = [
        ("会話", None, 1),
        ("選択A", Some("会話_1"), 1),
        ("選択", Some("会話_1"), 2),
        ("挨拶", Some("会話_1"), 2),
    ];

    let mut transpiled = SearchContext::new(registry, WordDefRegistry::new()).unwrap();
    transpiled.set_scene_selector(Some(vec![0])).unwrap();
    let finalized: mlua::Function = lua
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            SEARCH:set_scene_selector(0)
            return function(name, parent) return SEARCH:search_scene(name, parent) end
        "#,
        )
        .eval()
        .unwrap();

    let (mut before, mut after) = (Vec::new(), Vec::new());
    for (name, parent, n) in queries {
        for _ in 0..n {
            before.push(transpiled.search_scene(name, parent).unwrap());
            let (g, l): (Option<String>, Option<String>) = finalized.call((name, parent)).unwrap();
            after.push(g.zip(l));
        }
    }

    assert_eq!(after, before, "辞書確定前と確定後で検索の結果が同じ");
    assert!(
        before.iter().all(Option::is_some),
        "どの検索も候補が見つかる: {before:?}"
    );
}

/// 2.9: 決まった順に選ぶ設定（`set_scene_selector(0)`）の下で、`＊メイン`×10 は
/// `メイン_1`・`メイン_2`・…・`メイン_10` の順に返る（辞書確定の登録順と検索キーの統合確認）。
#[test]
fn deterministic_selector_returns_scenes_in_counter_order() {
    let source: String = (1..=10)
        .map(|i| format!("＊メイン\n　さくら：「メイン本体{i}」\n\n"))
        .collect();
    let (lua, _) = load_source(&source);
    let picked: Vec<String> = lua
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            SEARCH:set_scene_selector(0)
            local picked = {}
            for _ = 1, 10 do
                local g = SEARCH:search_scene("メイン", nil)
                picked[#picked + 1] = g
            end
            return picked
        "#,
        )
        .eval()
        .unwrap();
    let expected: Vec<String> = (1..=10).map(|i| format!("メイン_{i}")).collect();
    assert_eq!(picked, expected, "通し番号の順に返る");
}
