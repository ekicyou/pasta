//! scene-name-alias task 5: ローダーが同じ別名表を宣言と検索の両方へ配線する。
//!
//! `＊会話` だけの辞書を `PastaLoader` で読み込み、既定の別名表（`OnTalk = ["会話"]`）が
//! 宣言（トランスパイラ）・検索（`@pasta_search`）・デバッグのソースマップの 3 か所へ
//! 同じ値で届いていることを確かめる（requirements 2.7 / 6.2 / 7.1〜7.3）。
//! PASTA_DEBUG の中和は `tests/common/mod.rs` の `#[ctor]` ガードが担う。

use crate::common;

use common::create_temp_with_pasta;
use pasta_lua::debug::source_map::SceneIdentity;
use pasta_lua::loader::PastaLoader;
use pasta_lua::mlua;

/// 行 2 が `＊会話` の本体（行 1 はグローバル宣言）。
const ALIAS_ONLY_DIC: &str = "＊会話\n　さくら：「こんにちは」\n";
const LINE_BODY: u32 = 2;

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

/// 既定表のもとで `＊会話` は `OnTalk_1` として登録され、`会話`・`OnTalk` の
/// どちらで検索しても同じシーンに解決する（宣言と検索が同じ表を使う・6.2）。
#[test]
fn alias_only_dic_registers_ontalk_and_both_names_resolve_to_it() {
    let temp = create_temp_with_pasta(ALIAS_ONLY_DIC);
    let runtime = PastaLoader::load(temp.path()).expect("alias-only dic must load");
    let lua = runtime.lua();

    let scenes = pasta_lua::runtime::finalize::collect_scenes(lua).expect("collect_scenes");
    assert!(
        scenes.iter().any(|(g, _)| g == "OnTalk_1"),
        "＊会話 は OnTalk_1 として登録される: {scenes:?}"
    );
    assert!(
        !scenes.iter().any(|(g, _)| g.starts_with("会話")),
        "会話_N は登録されない: {scenes:?}"
    );

    assert_eq!(search_global(lua, "会話").as_deref(), Some("OnTalk_1"));
    assert_eq!(search_global(lua, "OnTalk").as_deref(), Some("OnTalk_1"));
}

/// デバッグ有効時、ソースマップ側のトランスパイラも同じ表を使うため、突合キーが
/// `G:OnTalk#1` となり `＊会話` の本体行が runtime の `OnTalk_1` へ対応づく（7.1〜7.3）。
#[test]
fn debug_source_map_join_uses_aliased_name() {
    let temp = create_temp_with_pasta(ALIAS_ONLY_DIC);
    std::fs::write(
        temp.path().join("pasta.toml"),
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\nport = 0\n",
    )
    .unwrap();
    let pasta_key = temp
        .path()
        .join("dic/test/hello.pasta")
        .to_string_lossy()
        .to_string();

    let runtime = PastaLoader::load(temp.path()).expect("debug-enabled runtime must load");
    let source_map = runtime
        .debug_source_map()
        .expect("enabled debug runtime holds the source map");

    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_BODY),
        Some(SceneIdentity {
            scene_id: "OnTalk_1".to_string(),
            parent: None,
        }),
        "＊会話 の本体行は OnTalk_1 の identity へ解決する"
    );
}

/// 有効な別名表（出どころと各行）が info で 1 回だけ記録される（2.7）。
#[tracing_test::traced_test]
#[test]
fn effective_alias_table_is_logged_once() {
    let temp = create_temp_with_pasta(ALIAS_ONLY_DIC);
    let _runtime = PastaLoader::load(temp.path()).expect("alias-only dic must load");

    logs_assert(|lines: &[&str]| {
        let hits: Vec<_> = lines
            .iter()
            .filter(|l| l.contains("Scene alias table"))
            .collect();
        match hits.as_slice() {
            [line]
                if line.contains("INFO")
                    && line.contains("BuiltinDefault")
                    && line.contains("OnTalk <- 会話") =>
            {
                Ok(())
            }
            _ => Err(format!(
                "expected exactly one INFO alias-table line, got {hits:?}"
            )),
        }
    });
}
