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
    load_source_with(source, "")
}

/// `load_source` に、辞書確定の前に実行する Lua（`WORD.create_local` 等）を足したもの。
fn load_source_with(source: &str, before_finalize: &str) -> (Lua, SceneRegistry) {
    let file = parse_str(source, "scene_identity_format.pasta").expect("fixture must parse");
    let mut out = Vec::new();
    let ctx = LuaTranspiler::default()
        .transpile(&file, &mut out)
        .expect("fixture must transpile");

    let lua = create_runtime_with_finalize().unwrap();
    lua.load(String::from_utf8(out).unwrap()).exec().unwrap();
    lua.load(before_finalize).exec().unwrap();
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

/// 呼ぶ回数。候補が混ざっていれば、シャッフル＆順次消費で必ず 1 巡する数より多くする。
const REPEAT: usize = 30;

/// 8.2（2.2・2.3・2.4・2.10）: `＞A1`・`search_scene("A1")`・`SCENE.search("A1")` を繰り返しても
/// `＊A` は 1 度も選ばれない。`＊章`／`＊章・1`、`＊会話・朝` の中の `・挨拶`／`・挨拶・1` も同様。
#[test]
fn call_and_search_never_pick_scene_named_by_shorter_prefix() {
    let (lua, _) = load_fixture();
    let search: mlua::Function = lua
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            local SCENE = require "pasta.scene"
            return function(name, parent)
                local g, l = SEARCH:search_scene(name, parent)
                local r = SCENE.search(name, parent)
                return g, l, r and r.global_name, r and r.local_name
            end
        "#,
        )
        .eval()
        .unwrap();
    // (検索する名前, 親の登録名, 選ばれるべき (グローバル, ローカル))
    let queries = [
        ("A1", None, ("A1_1", "__start__")),
        ("章・1", None, ("章_1_1", "__start__")),
        ("挨拶・1", Some("会話_朝_1"), ("会話_朝_1", "挨拶_1_1")),
    ];
    for (name, parent, (g, l)) in queries {
        let expected = Some((g.to_string(), l.to_string()));
        for _ in 0..REPEAT {
            let (sg, sl, rg, rl): (
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
            ) = search.call((name, parent)).unwrap();
            assert_eq!(sg.zip(sl), expected, "search_scene({name}, {parent:?})");
            assert_eq!(rg.zip(rl), expected, "SCENE.search({name}, {parent:?})");
        }
    }

    // DSL の Call（呼ぶ側のシーン, 呼ぶ側のローカル, 呼ばれる側の出力。「」も本文に含まれる）
    let calls = [
        ("呼出A1_1", "__start__", "「A1本体」"),
        ("呼出章_1_1", "__start__", "「章・1本体」"),
        ("会話_朝_1", "呼出挨拶_1", "「挨拶・1本体」"),
    ];
    for (g, l, out) in calls {
        for _ in 0..REPEAT {
            assert_eq!(run_scene(&lua, g, l), out, "{g}::{l} の Call");
        }
    }
}

/// 2.5: 登録名の形の名前（`メイン_1`）を `search_scene` の第 1 引数に渡すと、作者が書いた名前として
/// 扱われ、`＊メイン` の 1 つ目（`メイン_1`）ではなく `＊メイン・1`（`メイン_1_1`）だけが返る。
#[test]
fn registered_name_as_search_name_does_not_point_to_first_scene() {
    let source = "＊メイン\n　さくら：「メイン本体1」\n\n\
                  ＊メイン\n　さくら：「メイン本体2」\n\n\
                  ＊メイン・1\n　さくら：「メイン・1本体」\n";
    let (lua, _) = load_source(source);
    let picked: Vec<String> = lua
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            local picked = {}
            for _ = 1, 10 do
                local g = SEARCH:search_scene("メイン_1", nil)
                picked[#picked + 1] = tostring(g)
            end
            return picked
        "#,
        )
        .eval()
        .unwrap();
    assert!(
        picked.iter().all(|g| g == "メイン_1_1"),
        "メイン_1 は ＊メイン・1 だけを指す: {picked:?}"
    );
}

/// 2.7: `search_scene` が返したグローバルの登録名を第 2 引数に渡すと、そのシーンのローカルシーンが返る。
#[test]
fn returned_registered_name_finds_local_scenes() {
    let (lua, _) = load_fixture();
    let (g, lg, ll): (String, Option<String>, Option<String>) = lua
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            local g = SEARCH:search_scene("会話・朝", nil)
            local lg, ll = SEARCH:search_scene("挨拶", g)
            return g, lg, ll
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(g, "会話_朝_1");
    assert_eq!(lg.as_deref(), Some("会話_朝_1"));
    let ll = ll.expect("ローカルシーンが返る");
    assert!(ll.starts_with("挨拶_"), "・挨拶 のどれか: {ll}");
}

/// 2.8: 動的ターゲット（`＞＄変数`）と SHIORI イベントの経路（`SCENE.co_exec`）は、`＞シーン名` と
/// 同じ候補から選ぶ。`章・1` は `＊章・1` だけ、`章` は `＊章`×11・`＊章11`・`＊章_2`・`＊章・1` の 14 個。
#[test]
fn dynamic_target_and_event_choose_from_same_candidates_as_call() {
    let (lua, _) = load_fixture();
    let event: mlua::Function = lua
        .load(
            r#"
            local SCENE = require "pasta.scene"
            return function(name)
                local act = require("pasta.act").new({ ["さくら"] = { name = "さくら" } })
                local texts = {}
                act.build = function(self)
                    for _, t in ipairs(self.token) do
                        if t.type == "talk" then texts[#texts + 1] = t.text end
                    end
                end
                local co = SCENE.co_exec(act, name)
                if not co then error("event scene not found: " .. name) end
                assert(coroutine.resume(co, act))
                return table.concat(texts)
            end
        "#,
        )
        .eval()
        .unwrap();

    // (検索する名前, 候補の数, `＞名前` のシーン, `＞＄変数` のシーン)
    let targets = [
        ("章・1", 1, "呼出章_1_1", "動的呼出章_1_1"),
        ("章", 14, "呼出章_1", "動的呼出章_1"),
    ];
    for (name, n, static_caller, dynamic_caller) in targets {
        // 候補の数ちょうど呼ぶと、シャッフル＆順次消費で候補を 1 巡する（3 経路は同じキャッシュを共有するが、各経路が 1 巡の長さずつ引くので、どの経路も巡の頭から引き始める）。
        let collect = |f: &dyn Fn() -> String| (0..n).map(|_| f()).collect::<BTreeSet<_>>();
        let by_call = collect(&|| run_scene(&lua, static_caller, "__start__"));
        let by_dynamic = collect(&|| run_scene(&lua, dynamic_caller, "__start__"));
        let by_event = collect(&|| event.call::<String>(name).unwrap());
        assert_eq!(by_call.len(), n, "{name}: ＞{name} の候補 {by_call:?}");
        assert!(by_call.contains("「章・1本体」"), "{name}: {by_call:?}");
        assert_eq!(by_dynamic, by_call, "{name}: 動的ターゲット");
        assert_eq!(by_event, by_call, "{name}: SHIORI イベント");
    }
}

/// 3.2〜3.4: `WORD.create_local("メイン_1", キー)` の単語は `メイン_1` の実行中に参照でき、
/// `search_word(キー, "メイン_1")` で見つかる。旧形式 `メイン1` では見つからず、エラーにもならない。
#[test]
fn local_words_use_new_registered_name() {
    let source = "＊メイン\n　さくら：「＠ローカル語」\n\n\
                  ＊メイン\n　さくら：「＠旧形式語」\n";
    let (lua, _) = load_source_with(
        source,
        r#"
        local WORD = require "pasta.word"
        WORD.create_local("メイン_1", "ローカル語"):entry("新形式の単語")
        WORD.create_local("メイン1", "旧形式語"):entry("旧形式の単語")
        "#,
    );
    let (new_hit, old_miss, old_key_miss): (Option<String>, Option<String>, Option<String>) = lua
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            return SEARCH:search_word("ローカル語", "メイン_1"),
                SEARCH:search_word("ローカル語", "メイン1"),
                SEARCH:search_word("旧形式語", "メイン_2")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(new_hit.as_deref(), Some("新形式の単語"));
    assert_eq!(old_miss, None, "旧形式の登録名では見つからない");
    assert_eq!(
        old_key_miss, None,
        "メイン1 に登録した単語は メイン_2 にも無い"
    );

    assert_eq!(run_scene(&lua, "メイン_1", "__start__"), "「新形式の単語」");
    assert!(
        !run_scene(&lua, "メイン_2", "__start__").contains("旧形式の単語"),
        "メイン1 に登録した単語はどのシーンの実行中にも参照されない"
    );
}
