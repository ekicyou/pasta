//! touch-poke-shortcuts: 実ローダーで一時ゴーストを読み込み、つつき・撫でのショートカットを確かめる。
//!
//! PASTA_DEBUG の中和は `tests/common/mod.rs` の `#[ctor]` ガードが担う。

use crate::common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;

/// 一時ゴーストを作って実物のランタイムを読み込む。`TempDir` はランタイムより長く保持すること。
fn load_ghost(pasta: &str) -> (TempDir, PastaLuaRuntime) {
    load_ghost_with_main(pasta, None)
}

/// `load_ghost` に加えて、`main_lua` があれば起動用の `scripts/main.lua` として書いてから読み込む。
fn load_ghost_with_main(pasta: &str, main_lua: Option<&str>) -> (TempDir, PastaLuaRuntime) {
    let temp = create_temp_with_pasta(pasta);
    if let Some(main_lua) = main_lua {
        let scripts = temp.path().join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(scripts.join("main.lua"), main_lua).unwrap();
    }
    let runtime =
        PastaLoader::load(temp.path()).unwrap_or_else(|e| panic!("ゴーストの読み込みに失敗: {e}"));
    (temp, runtime)
}

/// `EVENT.fire` でイベント（未登録ならイベント名と同じ名前のシーン）を起こし、応答文字列を返す。
fn fire(runtime: &PastaLuaRuntime, id: &str) -> String {
    let script = format!(
        r#"local EVENT = require "pasta.shiori.event"
return EVENT.fire({{ id = "{id}", method = "get", version = 30 }})"#
    );
    let value = runtime.exec(&script).expect("EVENT.fire に失敗");
    value_as_str(&value).expect("応答が文字列でない")
}

/// 範囲を省略した `SEARCH:search_scene(name)` の戻り（登録名・ローカル名）。
fn search_global(runtime: &PastaLuaRuntime, name: &str) -> Option<(String, String)> {
    runtime
        .lua()
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            local g, l = SEARCH:search_scene(..., nil)
            if g == nil then return nil end
            return g .. "|" .. l
        "#,
        )
        .call::<Option<String>>(name)
        .expect("search_scene must not error")
        .map(|s| {
            let (g, l) = s.split_once('|').unwrap();
            (g.to_string(), l.to_string())
        })
}

// ============================================================================
// 空のシーン（要件 7.1）
// ============================================================================

/// 本文の無い `＊つっつき反応Head` と、本文のある `＊つっつき反応その他` を置いた辞書。
const EMPTY_HEAD_DIC: &str = "\
＊つっつき反応Head

＊つっつき反応その他
　さくら：その他の反応です
";

/// 本文の無いグローバルシーンも登録され（`__start__` を持つ）、実行すると何も出力しない（204）。
#[test]
fn empty_global_scene_is_registered_and_yields_no_output() {
    let (_temp, runtime) = load_ghost(EMPTY_HEAD_DIC);

    let scenes =
        pasta_lua::runtime::finalize::collect_scenes(runtime.lua()).expect("collect_scenes");
    assert!(
        scenes.iter().any(|(g, _)| g == "つっつき反応Head_1"),
        "本文の無いシーンが登録されていない: {scenes:?}"
    );

    assert_eq!(
        search_global(&runtime, "つっつき反応Head"),
        Some(("つっつき反応Head_1".to_string(), "__start__".to_string())),
        "本文の無いシーンが検索で見つからない"
    );

    // 実行すると出力が空 → 204（本文のある隣のシーンは呼ばれない）
    let response = fire(&runtime, "つっつき反応Head");
    assert!(
        response.starts_with("SHIORI/3.0 204"),
        "本文の無いシーンの応答が 204 でない: {response:?}"
    );
    assert!(!response.contains("その他の反応です"), "{response:?}");

    // 対照: 本文のあるシーンは同じ経路で 200 を返す（経路が生きていることの確認）
    let response = fire(&runtime, "つっつき反応その他");
    assert!(
        response.starts_with("SHIORI/3.0 200") && response.contains("その他の反応です"),
        "本文のあるシーンの応答: {response:?}"
    );
}

// ============================================================================
// 既存の既定ハンドラはゴーストの登録を上書きしない（要件 8.7）
// ============================================================================

/// 既定ハンドラを持つ 3 つのイベント。
const DEFAULT_HANDLER_EVENTS: [&str; 3] = ["OnBoot", "OnChoiceSelectEx", "OnSecondChange"];

/// `REG.{event}` に、イベント名入りの固定の応答を返すゴーストのハンドラを登録する Lua。
fn ghost_handler_lua(event: &str) -> String {
    format!(
        r#"local REG = require "pasta.shiori.event.register"
REG.{event} = function(act) return "ゴーストの{event}" end
"#
    )
}

/// 起動用 Lua（イベント層より先に読まれる）で登録しても、既定に上書きされずゴーストのハンドラが呼ばれる。
#[test]
fn ghost_handler_registered_before_event_layer_is_kept() {
    for event in DEFAULT_HANDLER_EVENTS {
        let (_temp, runtime) = load_ghost_with_main("", Some(&ghost_handler_lua(event)));
        let response = fire(&runtime, event);
        assert!(
            response.contains(&format!("ゴーストの{event}")),
            "{event}: 先に登録したハンドラが既定に上書きされた: {response:?}"
        );
    }
}

/// 辞書の Lua ブロック（イベント層より後に読まれる）で登録すると、今までどおりゴーストのハンドラが呼ばれる。
#[test]
fn ghost_handler_registered_in_dic_lua_block_is_used() {
    for event in DEFAULT_HANDLER_EVENTS {
        let dic = format!("＊登録\n```lua\n{}```\n", ghost_handler_lua(event));
        let (_temp, runtime) = load_ghost(&dic);
        let response = fire(&runtime, event);
        assert!(
            response.contains(&format!("ゴーストの{event}")),
            "{event}: 後に登録したハンドラが呼ばれない: {response:?}"
        );
    }
}

/// 先に `require("pasta.shiori.event")` してから既定を取っておき、包んだハンドラから既定を呼べる。
#[test]
fn ghost_can_wrap_default_handler_after_requiring_event_layer() {
    const DIC: &str = "\
＊OnBoot
　さくら：起動の既定です

＊OnChoiceSelectEx
　さくら：選択の既定です
";
    // OnSecondChange の既定は初回の呼び出しで何も返さない（トークのシーンが無い）
    let expected = [
        ("OnBoot", "起動の既定です"),
        ("OnChoiceSelectEx", "選択の既定です"),
        ("OnSecondChange", "既定は応答なし"),
    ];
    for (event, text) in expected {
        let main_lua = format!(
            r#"require("pasta.shiori.event")
local REG = require "pasta.shiori.event.register"
local default = REG.{event}
REG.{event} = function(act)
    local r = default(act)
    if r ~= nil then return r end
    return "既定は応答なし"
end
"#
        );
        let (_temp, runtime) = load_ghost_with_main(DIC, Some(&main_lua));
        let response = fire(&runtime, event);
        assert!(
            response.starts_with("SHIORI/3.0 200") && response.contains(text),
            "{event}: 包んだ既定の応答: {response:?}"
        );
    }
}
