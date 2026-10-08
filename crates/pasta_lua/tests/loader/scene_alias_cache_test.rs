//! scene-name-alias task 6.3: pasta.toml の別名表だけを変えて読み直すキャッシュの結合テスト。
//!
//! 同じゴーストディレクトリ（キャッシュが残る）を、`.pasta` を触らずに pasta.toml だけを
//! 「既定 → 空の `[scene.alias]` → 削除」と変えて 3 回読み直し、各段で登録名と
//! `SEARCH:search_scene` の結果の両方が新しい表に切り替わること、`.scene_alias` マーカーが
//! 新しい表の指紋に更新されることを確かめる（requirements 6.1〜6.3 / 10.4）。
//! PASTA_DEBUG の中和は `tests/common/mod.rs` の `#[ctor]` ガードが担う。

use crate::common;

use common::create_temp_with_pasta;
use pasta_core::SceneAliasTable;
use pasta_lua::loader::PastaLoader;
use pasta_lua::mlua;
use std::path::Path;

const ALIAS_ONLY_DIC: &str = "＊会話\n　さくら：「こんにちは」\n";
const BASE_TOML: &str = "[loader]\ndebug_mode = true\n";

fn search_global(lua: &mlua::Lua, name: &str) -> Option<String> {
    lua.load(
        r#"
        local SEARCH = require "@pasta_search"
        local g, l = SEARCH:search_scene(..., nil)
        return g
    "#,
    )
    .call::<Option<String>>(name)
    .expect("search_scene must not error")
}

/// 読み込み、登録名・検索結果・マーカーを確かめる。`expected` は `＊会話` の登録名、
/// `other` は登録されてはならない側の検索名（見つからないこと）。
fn load_and_check(base: &Path, step: &str, table: &SceneAliasTable, expected: &str, other: &str) {
    let runtime = PastaLoader::load(base).unwrap_or_else(|e| panic!("[{step}] load: {e}"));
    let lua = runtime.lua();

    let scenes = pasta_lua::runtime::finalize::collect_scenes(lua).expect("collect_scenes");
    let globals: Vec<&str> = scenes.iter().map(|(g, _)| g.as_str()).collect();
    assert!(
        globals.contains(&expected),
        "[{step}] ＊会話 は {expected} として登録される: {globals:?}"
    );
    assert!(
        !globals.iter().any(|g| g.starts_with(other)),
        "[{step}] {other}_N は登録されない: {globals:?}"
    );

    assert_eq!(
        search_global(lua, "会話").as_deref(),
        Some(expected),
        "[{step}] search_scene(\"会話\")"
    );
    assert_eq!(
        search_global(lua, other),
        None,
        "[{step}] search_scene(\"{other}\") は見つからない"
    );

    let marker = std::fs::read_to_string(base.join("profile/pasta/cache/lua/.scene_alias"))
        .unwrap_or_else(|e| panic!("[{step}] marker: {e}"));
    assert_eq!(marker, table.fingerprint(), "[{step}] .scene_alias の指紋");
}

/// pasta.toml の別名表だけを変えて同じディレクトリを読み直すと、`.pasta` が古いままの
/// キャッシュが残らず、宣言と検索の両方が新しい表で切り替わる（6.1・6.2・6.3・10.4）。
#[test]
fn changing_only_alias_table_in_pasta_toml_switches_declaration_and_search() {
    let temp = create_temp_with_pasta(ALIAS_ONLY_DIC);
    let base = temp.path();
    let toml = base.join("pasta.toml");
    let pasta = base.join("dic/test/hello.pasta");
    let pasta_mtime = std::fs::metadata(&pasta).unwrap().modified().unwrap();

    // (1) 既定の表: ＊会話 → OnTalk_1
    load_and_check(
        base,
        "1:default",
        &SceneAliasTable::builtin_default(),
        "OnTalk_1",
        "会話_",
    );

    // (2) 空の [scene.alias] を足す: ＊会話 → 会話_1
    std::fs::write(&toml, format!("{BASE_TOML}\n[scene.alias]\n")).unwrap();
    load_and_check(
        base,
        "2:empty",
        &SceneAliasTable::empty(),
        "会話_1",
        "OnTalk",
    );

    // (3) [scene.alias] を削除: 既定へ戻り ＊会話 → OnTalk_1
    std::fs::write(&toml, BASE_TOML).unwrap();
    load_and_check(
        base,
        "3:removed",
        &SceneAliasTable::builtin_default(),
        "OnTalk_1",
        "会話_",
    );

    // .pasta は一度も触っていない
    assert_eq!(std::fs::read_to_string(&pasta).unwrap(), ALIAS_ONLY_DIC);
    assert_eq!(
        std::fs::metadata(&pasta).unwrap().modified().unwrap(),
        pasta_mtime
    );
}
