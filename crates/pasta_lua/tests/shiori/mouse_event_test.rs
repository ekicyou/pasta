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
    load_ghost_full(pasta, main_lua, "")
}

/// `load_ghost_with_main` に加えて、`toml_extra` を `pasta.toml` の後ろに足してから読み込む。
fn load_ghost_full(
    pasta: &str,
    main_lua: Option<&str>,
    toml_extra: &str,
) -> (TempDir, PastaLuaRuntime) {
    let temp = create_temp_with_pasta(pasta);
    if !toml_extra.is_empty() {
        let toml = temp.path().join("pasta.toml");
        let base = std::fs::read_to_string(&toml).unwrap();
        std::fs::write(&toml, base + toml_extra).unwrap();
    }
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

// ============================================================================
// 完全一致でグローバルシーンをコルーチンにする（要件 6.2）
// ============================================================================

/// 頭が同じ長い名前のシーンと、ローカルシーン・`GLOBAL` の関数を置いた辞書。
const EXACT_DIC: &str = "\
＊つっつき反応Head
　さくら：頭の反応です

＊つっつき反応Head右
　さくら：頭右の反応です

＊親
　さくら：親です

　・ローカルだけ
　　さくら：ローカルです
";

/// `SCENE.co_exec_exact` を呼ぶハンドラと、同名の `GLOBAL` 関数を登録する起動用 Lua。
const EXACT_MAIN_LUA: &str = r#"local REG = require "pasta.shiori.event.register"
local SCENE = require "pasta.scene"
local GLOBAL = require "pasta.global"
GLOBAL["グローバル関数だけ"] = function(act) act.sakura:talk("関数です") end
REG.OnExactHead = function(act)
    return SCENE.co_exec_exact("つっつき反応Head")
end
"#;

/// `SCENE.co_exec_exact(name)` が nil を返すか。
fn co_exec_exact_is_nil(runtime: &PastaLuaRuntime, name: &str) -> bool {
    runtime
        .lua()
        .load(r#"return require("pasta.scene").co_exec_exact(...) == nil"#)
        .call::<bool>(name)
        .expect("co_exec_exact must not error")
}

/// 完全一致の名前で見つかったシーンのコルーチンを再開すると出力が返り、
/// 頭が同じ長い名前・前方一致だけの名前・無い名前・ローカルシーン・`GLOBAL` の関数では nil。
#[test]
fn co_exec_exact_runs_only_exactly_named_global_scene() {
    let (_temp, runtime) = load_ghost_with_main(EXACT_DIC, Some(EXACT_MAIN_LUA));

    // 頭が同じ `つっつき反応Head右` は選ばれない（同名は 1 つなので何度でも頭の反応）
    for _ in 0..8 {
        let response = fire(&runtime, "OnExactHead");
        assert!(
            response.starts_with("SHIORI/3.0 200") && response.contains("頭の反応です"),
            "完全一致のシーンの応答: {response:?}"
        );
        assert!(!response.contains("頭右の反応です"), "{response:?}");
    }

    assert!(
        !co_exec_exact_is_nil(&runtime, "つっつき反応Head右"),
        "長い名前もそれ自体は完全一致で見つかる"
    );
    for name in [
        "つっつき反応",    // 前方一致だけ
        "つっつき反応Hea", // 前方一致だけ
        "無いシーン",
        "ローカルだけ",       // ローカルシーンは見ない
        "グローバル関数だけ", // GLOBAL の関数は見ない
    ] {
        assert!(co_exec_exact_is_nil(&runtime, name), "{name}: nil でない");
    }
}

// ============================================================================
// つつき・撫での既定のハンドラ（要件 1.3・3.3・8.1）
// ============================================================================

/// `extra` を足したリクエストで `EVENT.fire` を呼ぶ（`extra` は Lua の表の中身。例: `reference = { [3] = "0" }`）。
fn fire_with(runtime: &PastaLuaRuntime, id: &str, extra: &str) -> String {
    let script = format!(
        r#"local EVENT = require "pasta.shiori.event"
return EVENT.fire({{ id = "{id}", method = "get", version = 30, {extra} }})"#
    );
    let value = runtime.exec(&script).expect("EVENT.fire に失敗");
    value_as_str(&value).expect("応答が文字列でない")
}

/// `＊つっつき反応その他` だけを置いたゴーストにつつきを送ると、そのシーンの応答が返る。
#[test]
fn poke_runs_shortcut_scene() {
    let (_temp, runtime) = load_ghost("＊つっつき反応その他\n　さくら：つつかれました\n");
    let response = fire_with(
        &runtime,
        "OnMouseDoubleClick",
        r#"reference = { [3] = "0", [4] = "Head" }"#,
    );
    assert!(
        response.starts_with("SHIORI/3.0 200") && response.contains("つつかれました"),
        "つつきのショートカットの応答: {response:?}"
    );
}

/// ショートカットが無ければ、つつき・移動は今までどおりイベント名のシーンへ（要件 8.1）。
#[test]
fn mouse_events_fall_back_to_event_name_scene() {
    let (_temp, runtime) = load_ghost(
        "＊OnMouseDoubleClick\n　さくら：ダブルクリック\n\n＊OnMouseMove\n　さくら：移動\n",
    );
    let refs = r#"reference = { [3] = "0", [4] = "Head" }"#;
    let response = fire_with(&runtime, "OnMouseDoubleClick", refs);
    assert!(response.contains("ダブルクリック"), "{response:?}");
    let response = fire_with(&runtime, "OnMouseMove", refs);
    assert!(response.contains("移動"), "{response:?}");
    // どれも無いイベントは 204（要件 1.3）
    let (_temp, runtime) = load_ghost("");
    let response = fire_with(&runtime, "OnMouseDoubleClick", refs);
    assert!(response.starts_with("SHIORI/3.0 204"), "{response:?}");
}

// ============================================================================
// つつきの段々・完全一致・抽選・空のシーン・チェイントーク（要件 1・4.4・6.2・6.3・7・8.1）
// ============================================================================

/// アクター名を引けるように `pasta.toml` に足すアクター（スポット 0・1）。
const ACTORS_TOML: &str = "\n[actor.\"女の子\"]\nspot = 0\n\n[actor.\"男の子\"]\nspot = 1\n";

/// 女の子（スポット 0）の `Head` をつついたときの 9 段の名前（上の段から）。
const POKE_RUNGS: [&str; 9] = [
    "つっつき反応女の子Head",
    "OnPoke女の子Head",
    "つっつき反応女の子その他",
    "OnPoke女の子Other",
    "つっつき反応Head",
    "OnPokeHead",
    "つっつき反応その他",
    "OnPokeOther",
    "OnMouseDoubleClick",
];

/// 段 `rungs`（1 始まり）のシーンだけを置いた辞書。各シーンは「段{n}です」と話す。
fn rung_dic(rungs: impl IntoIterator<Item = usize>) -> String {
    rungs
        .into_iter()
        .map(|n| format!("＊{}\n　女の子：段{n}です\n\n", POKE_RUNGS[n - 1]))
        .collect()
}

/// つつき（OnMouseDoubleClick）を Reference3・4 付きで送る。`scope` が None なら Reference3 を付けない。
fn poke(runtime: &PastaLuaRuntime, scope: Option<&str>, part: &str) -> String {
    let scope = scope.map_or(String::new(), |s| format!("[3] = \"{s}\", "));
    fire_with(
        runtime,
        "OnMouseDoubleClick",
        &format!("reference = {{ {scope}[4] = \"{part}\" }}"),
    )
}

/// 応答が 200 で、段 `n` のシーンの出力だけを含むか。
fn assert_rung(response: &str, n: usize, ctx: &str) {
    assert!(
        response.starts_with("SHIORI/3.0 200") && response.contains(&format!("段{n}です")),
        "{ctx}: 段{n}が選ばれない: {response:?}"
    );
    for other in (1..=9).filter(|&m| m != n) {
        assert!(
            !response.contains(&format!("段{other}です")),
            "{ctx}: {response:?}"
        );
    }
}

/// 段 k から下だけを置いたゴーストで、いつも段 k（書いた中で一番上）が選ばれる（要件 1.1）。
#[test]
fn poke_picks_highest_written_rung() {
    for top in 1..=9 {
        let (_temp, runtime) = load_ghost_full(&rung_dic(top..=9), None, ACTORS_TOML);
        let response = poke(&runtime, Some("0"), "Head");
        assert_rung(&response, top, &format!("段{top}から下"));
    }
    // 書いた段 1 つだけのゴーストでも、その段が選ばれる
    for only in 1..=9 {
        let (_temp, runtime) = load_ghost_full(&rung_dic([only]), None, ACTORS_TOML);
        let response = poke(&runtime, Some("0"), "Head");
        assert_rung(&response, only, &format!("段{only}だけ"));
    }
}

/// 全段を置いたゴーストで、部位が空なら部位の段（1・2・5・6）を、
/// アクター名が決まらなければアクターの段（1〜4）を飛ばす（要件 1.4・1.5・4.4）。
#[test]
fn poke_skips_rungs_without_part_or_actor() {
    let (_temp, runtime) = load_ghost_full(&rung_dic(1..=9), None, ACTORS_TOML);
    let cases: [(Option<&str>, &str, usize); 6] = [
        (Some("0"), "Head", 1),
        (Some("0"), "", 3),     // 部位が空 → 女の子その他
        (Some("5"), "Head", 5), // 未登録のスコープ番号 → 部位の段
        (None, "Head", 5),      // スコープ番号なし
        (Some("5"), "", 7),     // どちらも無い → その他
        (Some("abc"), "", 7),   // 数でないスコープ番号
    ];
    for (scope, part, expected) in cases {
        let response = poke(&runtime, scope, part);
        assert_rung(
            &response,
            expected,
            &format!("scope={scope:?} part={part:?}"),
        );
    }
    // スコープ番号そのものを入れた名前（つっつき反応0Head）は探さない（要件 4.4）
    let (_temp, runtime) = load_ghost_full(
        "＊つっつき反応0Head\n　女の子：番号の段です\n",
        None,
        ACTORS_TOML,
    );
    let response = poke(&runtime, Some("0"), "Head");
    assert!(response.starts_with("SHIORI/3.0 204"), "{response:?}");
}

/// `＊OnMouseDoubleClick` だけのゴーストは、既定のハンドラが無かったとき（イベント名のシーンを
/// そのまま探す `EVENT.no_entry`）と同じ応答を返す（要件 6.3・8.1）。
#[test]
fn poke_with_only_event_scene_matches_previous_behavior() {
    const DIC: &str = "＊OnMouseDoubleClick\n　女の子：ダブルクリックです\n";
    const OLD_MAIN: &str = r#"local REG = require "pasta.shiori.event.register"
REG.OnMouseDoubleClick = function(act) return require("pasta.shiori.event").no_entry(act) end
"#;
    let (_t1, new_rt) = load_ghost_full(DIC, None, ACTORS_TOML);
    let (_t2, old_rt) = load_ghost_full(DIC, Some(OLD_MAIN), ACTORS_TOML);
    for (scope, part) in [(Some("0"), "Head"), (Some("1"), ""), (None, "")] {
        let new = poke(&new_rt, scope, part);
        let old = poke(&old_rt, scope, part);
        assert!(new.contains("ダブルクリックです"), "{new:?}");
        assert_eq!(new, old, "scope={scope:?} part={part:?}");
    }
}

/// `＊つっつき反応胸右` だけのゴーストで部位 `胸` をつついても `胸右` は選ばれない（要件 6.2・1.3）。
#[test]
fn poke_does_not_pick_longer_part_name() {
    let (_temp, runtime) = load_ghost_full(
        "＊つっつき反応胸右\n　女の子：胸右です\n",
        None,
        ACTORS_TOML,
    );
    let response = poke(&runtime, Some("0"), "胸");
    assert!(response.starts_with("SHIORI/3.0 204"), "{response:?}");
    // 対照: 部位 `胸右` なら選ばれる
    let response = poke(&runtime, Some("0"), "胸右");
    assert!(response.contains("胸右です"), "{response:?}");
}

/// 同じ名前の 2 シーンは、2 回のつつきで 1 回ずつ出る（要件 1.2）。
#[test]
fn poke_draws_same_named_scenes_in_turn() {
    let (_temp, runtime) = load_ghost_full(
        "＊つっつき反応その他\n　女の子：甲です\n\n＊つっつき反応その他\n　女の子：乙です\n",
        None,
        ACTORS_TOML,
    );
    for round in 0..3 {
        let a = poke(&runtime, Some("0"), "Head");
        let b = poke(&runtime, Some("0"), "Head");
        let kou = [&a, &b].iter().filter(|r| r.contains("甲です")).count();
        let otsu = [&a, &b].iter().filter(|r| r.contains("乙です")).count();
        assert_eq!((kou, otsu), (1, 1), "{round} 巡目: {a:?} / {b:?}");
    }
}

/// 本文の無い `＊つっつき反応Head` があると部位 `Head` のつつきは 204 で、
/// `その他` は呼ばれない。ほかの部位は `その他` に届く（要件 7.1）。
#[test]
fn empty_shortcut_scene_stops_lower_rungs() {
    let (_temp, runtime) = load_ghost_full(EMPTY_HEAD_DIC, None, ACTORS_TOML);
    let response = poke(&runtime, Some("0"), "Head");
    assert!(response.starts_with("SHIORI/3.0 204"), "{response:?}");
    assert!(!response.contains("その他の反応です"), "{response:?}");
    let response = poke(&runtime, Some("0"), "Face");
    assert!(response.contains("その他の反応です"), "{response:?}");
}

/// ショートカットのシーンの中で `＞transfer_req_to_var` が Reference を変数へ移す（要件 7.2）。
#[test]
fn transfer_req_to_var_works_in_shortcut_scene() {
    let (_temp, runtime) = load_ghost_full(
        "＊つっつき反応その他\n　＞transfer_req_to_var\n　女の子：番号＄ｒ３　部位＄ｒ４　です\n",
        None,
        ACTORS_TOML,
    );
    let response = poke(&runtime, Some("1"), "Face");
    assert!(response.contains("番号1"), "{response:?}");
    assert!(response.contains("部位Face"), "{response:?}");
}

/// つつきの後の継続（`STORE.co_scene`）を調べ、残っていれば再開してその出力を返す Lua。
const RESUME_CONTINUATION: &str = r#"local STORE = require "pasta.store"
local co = STORE.co_scene
if co == nil then return "継続なし" end
local before = coroutine.status(co)
local ok, value = require("pasta.shiori.event").drive(co, nil)
return before .. "|" .. tostring(ok) .. "|" .. tostring(value) .. "|" .. tostring(STORE.co_scene)
"#;

/// チェイントークを含むショートカットのシーンの、応答と継続の扱いが
/// `＊OnMouseDoubleClick` で書いたときと同じ（要件 7.4）。
#[test]
fn chain_talk_after_shortcut_matches_event_scene() {
    const BODY: &str = "　女の子：前半です\n　＞チェイントーク\n　女の子：後半です\n";
    let mut results = Vec::new();
    for name in ["つっつき反応その他", "OnMouseDoubleClick"] {
        let (_temp, runtime) = load_ghost_full(&format!("＊{name}\n{BODY}"), None, ACTORS_TOML);
        let first = poke(&runtime, Some("0"), "Head");
        let value = runtime.exec(RESUME_CONTINUATION).expect("継続の再開に失敗");
        let cont = value_as_str(&value).expect("文字列でない");
        assert!(
            first.contains("前半です") && !first.contains("後半です"),
            "{name}: {first:?}"
        );
        assert!(
            cont.starts_with("suspended|true|") && cont.contains("後半です"),
            "{name}: 継続: {cont:?}"
        );
        results.push((first, cont));
    }
    assert_eq!(
        results[0], results[1],
        "ショートカットとイベント名のシーンで扱いが違う"
    );
}

// ============================================================================
// 撫でと抑止（要件 2.1・2.6・3.1・3.3・5.1・5.3・5.4・5.5・8.1・8.5）
// ============================================================================

/// 撫でのシーンとイベント名のシーンを置いた辞書。
const STROKE_DIC: &str = "\
＊さわり反応その他
　女の子：撫でられました

＊OnMouseMove
　女の子：移動です
";

/// 時刻 `t`（秒・小数つき）と Status を付けて、女の子（スポット 0）の `Head` の上の移動を送る。
fn mouse_move(runtime: &PastaLuaRuntime, t: f64, status: &str) -> String {
    let unix = t.floor() as i64;
    let ns = ((t - t.floor()) * 1e9).round() as i64;
    fire_with(
        runtime,
        "OnMouseMove",
        &format!(
            r#"reference = {{ [3] = "0", [4] = "Head" }}, status = "{status}", date = {{ unix = {unix}, ns = {ns} }}"#
        ),
    )
}

/// 応答が 200 で、撫でのシーンなら `stroke = true`、イベント名のシーンなら false の出力だけを含むか。
fn assert_move(response: &str, stroke: bool, ctx: &str) {
    let (want, other) = if stroke {
        ("撫でられました", "移動です")
    } else {
        ("移動です", "撫でられました")
    };
    assert!(
        response.starts_with("SHIORI/3.0 200")
            && response.contains(want)
            && !response.contains(other),
        "{ctx}: {want} だけが返らない: {response:?}"
    );
}

/// 0.5 秒おきの移動で、2 秒に届くまでは `＊OnMouseMove`、届いた移動で撫でのシーンが 1 回だけ返る。
#[test]
fn stroke_fires_after_two_seconds_of_moves() {
    let (_temp, runtime) = load_ghost_full(STROKE_DIC, None, ACTORS_TOML);
    for (i, t) in [1000.0, 1000.5, 1001.0, 1001.5].into_iter().enumerate() {
        assert_move(
            &mouse_move(&runtime, t, "balloon(0=0)"),
            false,
            &format!("{i} 回目 t={t}"),
        );
    }
    assert_move(
        &mouse_move(&runtime, 1002.0, "balloon(0=0)"),
        true,
        "2 秒に届いた移動",
    );
    // 撫で続けても 2 回目は出ない（1 回だけ）
    for t in [1002.5, 1003.0, 1004.5] {
        assert_move(
            &mouse_move(&runtime, t, "balloon(0=0)"),
            false,
            &format!("撫でた後 t={t}"),
        );
    }
}

/// `talking` の間は撫でのシーンが返らず `＊OnMouseMove` が返る。
/// `balloon(0=0)` だけになると、そこから数え始めて 2 秒で撫でのシーンが返る。
#[test]
fn stroke_is_blocked_while_talking() {
    let (_temp, runtime) = load_ghost_full(STROKE_DIC, None, ACTORS_TOML);
    for t in [2000.0, 2000.5, 2001.0, 2001.5, 2002.0, 2002.5, 2003.0] {
        assert_move(
            &mouse_move(&runtime, t, "talking,balloon(0=0)"),
            false,
            &format!("トーク中 t={t}"),
        );
    }
    // 抑止が解けた最初の移動から数え始める（2003.5 → 2005.5 で撫でた）
    for t in [2003.5, 2004.0, 2004.5, 2005.0] {
        assert_move(
            &mouse_move(&runtime, t, "balloon(0=0)"),
            false,
            &format!("解けた後 t={t}"),
        );
    }
    assert_move(
        &mouse_move(&runtime, 2005.5, "balloon(0=0)"),
        true,
        "解けてから 2 秒",
    );
}

/// 撫でのショートカットが無いゴーストでは、何秒撫でても毎回 `＊OnMouseMove` が返る。
#[test]
fn stroke_without_shortcut_always_runs_event_scene() {
    let (_temp, runtime) =
        load_ghost_full("＊OnMouseMove\n　女の子：移動です\n", None, ACTORS_TOML);
    for i in 0..10 {
        let t = 3000.0 + f64::from(i) * 0.5;
        assert_move(
            &mouse_move(&runtime, t, "balloon(0=0)"),
            false,
            &format!("t={t}"),
        );
    }
}

// ============================================================================
// 自前のハンドラは登録の前後にかかわらず優先する（要件 8.2・8.3・8.6）
// ============================================================================

/// つつき・撫での両方のショートカットのシーンを置いたゴーストで、自前のハンドラを
/// (a) 起動用 Lua（既定より先）と (b) 辞書の Lua ブロック（既定より後）で登録すると、
/// どちらでも自前のハンドラの応答が返り、ショートカットのシーンは選ばれず、撫での判定の状態も初期のまま。
#[test]
fn ghost_mouse_handlers_win_regardless_of_registration_order() {
    const SHORTCUTS: &str = "\
＊つっつき反応その他
　女の子：つつかれました

＊さわり反応その他
　女の子：撫でられました
";
    let handlers = ghost_handler_lua("OnMouseDoubleClick") + &ghost_handler_lua("OnMouseMove");
    let in_dic = format!("{SHORTCUTS}\n＊登録\n```lua\n{handlers}```\n");
    let cases = [
        ("起動用 Lua", SHORTCUTS, Some(handlers.as_str())),
        ("辞書の Lua ブロック", in_dic.as_str(), None),
    ];
    for (ctx, dic, main_lua) in cases {
        let (_temp, runtime) = load_ghost_full(dic, main_lua, ACTORS_TOML);
        let response = fire_with(
            &runtime,
            "OnMouseDoubleClick",
            r#"reference = { [3] = "0", [4] = "Head" }"#,
        );
        assert!(
            response.contains("ゴーストのOnMouseDoubleClick")
                && !response.contains("つつかれました"),
            "{ctx}: つつきで自前のハンドラが呼ばれない: {response:?}"
        );
        // 既定の受け方なら 2 秒で撫でのシーンが返る移動
        for i in 0..7 {
            let t = 1000.0 + f64::from(i) * 0.5;
            let response = mouse_move(&runtime, t, "balloon(0=0)");
            assert!(
                response.contains("ゴーストのOnMouseMove") && !response.contains("撫でられました"),
                "{ctx}: t={t} 移動で自前のハンドラが呼ばれない: {response:?}"
            );
        }
        let state = runtime
            .exec(
                r#"local s = require("pasta.shiori.event.mouse")._get_state()
return table.concat({ tostring(s.target), tostring(s.start), tostring(s.last), tostring(s.fired) }, ",")"#,
            )
            .expect("撫での状態を読めない");
        assert_eq!(
            value_as_str(&state).as_deref(),
            Some("nil,0,0,false"),
            "{ctx}: 撫での判定の状態が初期のままでない"
        );
    }
}
