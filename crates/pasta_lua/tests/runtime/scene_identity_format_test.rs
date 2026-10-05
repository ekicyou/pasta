//! 登録名の形式（`{名前}_{通し番号}`）の統合テスト（scene-identity-format）。
//!
//! - `＊A1` と `＊A`×11 が 12 個の別々のシーンとして登録され、`A1_1` と `A_11` の両方が
//!   実行できる（1.3・8.1）。旧形式（`A` ＋ `11` = `A11`）では `A1` の 1 本目と衝突する。
//! - 実行時のシーン表のグローバルの登録名の集合が、定義から Rust の規則
//!   （`SceneRegistry::registered_name`）で組み立てた集合と一致する（6.3）。グローバルの
//!   登録名は Lua（`SCENE.create_scene`）が作るので、ここが Rust と Lua の形式の突き合わせになる。
//!   ローカルの登録名は Rust の生成器が作って生成コードに埋め込むので Lua の形式とは関係しない
//!   （辞書確定前のレジストリのローカルの通し番号を生成器にそろえるのは 4.3 で行う）。

use std::collections::BTreeSet;

use crate::common;
use common::e2e_helpers::create_runtime_with_finalize;
use mlua::Lua;
use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;

const FIXTURE: &str = include_str!("../fixtures/scene_identity_format.pasta");

/// フィクスチャをトランスパイル→実行→`finalize_scene` し、ランタイムと、
/// トランスパイル時のシーンレジストリから作ったグローバルの登録名の集合を返す。
fn load_fixture() -> (Lua, BTreeSet<String>) {
    let file = parse_str(FIXTURE, "scene_identity_format.pasta").expect("fixture must parse");
    let mut out = Vec::new();
    let ctx = LuaTranspiler::default()
        .transpile(&file, &mut out)
        .expect("fixture must transpile");
    // グローバルの fn_name（`登録名::__start__`）は registered_name（名前ごとの通し番号）で組み立てられる。
    let expected = ctx
        .scene_registry
        .all_scenes()
        .iter()
        .filter(|s| s.parent.is_none())
        .map(|s| {
            s.fn_name
                .split_once("::")
                .expect("fn_name is global::local")
                .0
                .to_string()
        })
        .collect();

    let lua = create_runtime_with_finalize().unwrap();
    lua.load(String::from_utf8(out).unwrap()).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    (lua, expected)
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

/// 6.3: 実行時のグローバルの登録名の集合が、定義から Rust の規則で組み立てた集合と一致する。
#[test]
fn runtime_registered_names_match_rust_registered_name_rule() {
    let (lua, expected) = load_fixture();
    let actual: BTreeSet<String> = pasta_lua::runtime::finalize::collect_scenes(&lua)
        .unwrap()
        .into_iter()
        .map(|(g, _)| g)
        .collect();
    assert_eq!(
        actual, expected,
        "Lua の登録名と Rust の registered_name の形式が一致する"
    );
}
