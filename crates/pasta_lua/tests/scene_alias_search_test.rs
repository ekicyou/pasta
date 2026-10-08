//! scene-name-alias task 6.2: 既定・作者定義・空の 3 表で、宣言と検索の全入口が同じシーンに解決される結合テスト。
//!
//! テスト用の `.pasta` と pasta.toml の `[scene.alias]` で一時ゴーストを作り、`PastaLoader::load` で
//! 実物のランタイムを起こす。シーンの出力は `EVENT.fire` の応答文字列で、登録名は `collect_scenes`
//! と `@pasta_search` の戻り値で、警告は `tracing_test` の捕捉で確かめる
//! （requirements 1.2〜1.4・3.6・3.7・4.3〜4.8・5.3・5.4・5.6・8.1・8.3・10.1・10.3）。
//! `mod common` の宣言により `PASTA_DEBUG` 中和ガード（`#[ctor]`）が適用される。

mod common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;
use tracing_test::traced_test;

/// 有効な別名表の 3 通り。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Table {
    /// `[scene.alias]` 無し（既定の `OnTalk = ["会話"]`）。
    Default,
    /// 作者定義の表（既定とはマージしない）。
    Author,
    /// 見出しだけの空の表（別名なし＝導入前と同じ）。
    Empty,
}

impl Table {
    const ALL: [Table; 3] = [Table::Default, Table::Author, Table::Empty];

    /// pasta.toml へ追記する内容。
    fn toml(self) -> &'static str {
        match self {
            Table::Default => "",
            Table::Author => {
                "\n[scene.alias]\nOnTalk = [\"会話\", \"雑談\"]\nOnBoot = [\"起動\"]\n"
            }
            Table::Empty => "\n[scene.alias]\n",
        }
    }

    /// `＊会話` の登録名（置き換え後の名前から作られる・8.3）。
    fn kaiwa_id(self) -> &'static str {
        match self {
            Table::Default | Table::Author => "OnTalk_1",
            Table::Empty => "会話_1",
        }
    }

    /// `会話` が別名として置き換わるか。
    fn aliases_kaiwa(self) -> bool {
        self != Table::Empty
    }
}

/// 宣言と、グローバルシーンを名前で探す入口・置き換えない名前を集めたゴースト。
const MAIN_PASTA: &str = "\
＠会話：単語の会話です

％会話
  ＠通常：会話アクターの通常顔

＊会話
  さくら：会話のシーンです

＊静的呼び出し
  ＞会話

＊文字列呼び出し
  ＞「会話」

＊変数呼び出し
  ＄行き先＝「会話」
  ＞＄行き先

＊選択肢表示
  さくら：どうする？
  ＠？会話「話す」

＊起動
  さくら：起動のシーンです

＊ローカル親
  ＞会話

  ・会話
    さくら：ローカルの会話です

  ・OnTalk
    さくら：ローカルのOnTalkです

＊単語参照
  さくら：＠会話

＊アクター参照
  会話：＠通常
";

/// Lua 公開 API（`SCENE.co_exec`・`act:call`）を範囲省略の名前「会話」で呼ぶイベントハンドラ。
const API_MAIN_LUA: &str = r#"local REG = require "pasta.shiori.event.register"
local SCENE = require "pasta.scene"
REG.OnApiCoExec = function(act)
    return SCENE.co_exec(act, "会話", nil, nil)
end
REG.OnApiActCall = function(act)
    return coroutine.create(function(a)
        a:call(nil, "会話", nil)
        return a:build()
    end)
end
return {}
"#;

/// テスト用ゴーストのアクター設定。
const ACTOR_TOML: &str = "[actor.\"さくら\"]\nspot = 0\n";

/// 一時ゴーストを作って実物のランタイムを読み込む。`TempDir` はランタイムより長く保持すること。
fn load_ghost(pasta: &str, table: Table, main_lua: Option<&str>) -> (TempDir, PastaLuaRuntime) {
    let temp = create_temp_with_pasta(pasta);
    let toml = temp.path().join("pasta.toml");
    let base = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(&toml, format!("{base}{ACTOR_TOML}{}", table.toml())).unwrap();
    if let Some(main_lua) = main_lua {
        let scripts = temp.path().join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(scripts.join("main.lua"), main_lua).unwrap();
    }
    let runtime = PastaLoader::load(temp.path())
        .unwrap_or_else(|e| panic!("{table:?}: ゴーストの読み込みに失敗: {e}"));
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

/// `SEARCH:search_scene(name, scope)` の戻り（登録名・ローカル名）。見つからなければ `None`。
fn search(runtime: &PastaLuaRuntime, name: &str, scope: Option<&str>) -> Option<(String, String)> {
    runtime
        .lua()
        .load(
            r#"
            local SEARCH = require "@pasta_search"
            local name, scope = ...
            local g, l = SEARCH:search_scene(name, scope)
            if g == nil then return nil end
            return g .. "|" .. l
        "#,
        )
        .call::<Option<String>>((name, scope))
        .expect("search_scene must not error")
        .map(|s| {
            let (g, l) = s.split_once('|').unwrap();
            (g.to_string(), l.to_string())
        })
}

/// 範囲を省略した `SEARCH:search_scene(name)` の登録名。
fn search_global(runtime: &PastaLuaRuntime, name: &str) -> Option<String> {
    search(runtime, name, None).map(|(g, l)| {
        assert_eq!(l, "__start__", "グローバル検索は __start__ を返す");
        g
    })
}

fn assert_ok_with(table: Table, what: &str, response: &str, text: &str) {
    assert!(
        response.starts_with("SHIORI/3.0 200 OK") && response.contains(text),
        "{table:?} / {what}: 応答に {text:?} が無い: {response:?}"
    );
}

/// 4.3〜4.6・1.2・8.3・5.6: 3 表それぞれで、`＊会話` の登録名と、Call（静的・「」・変数）・選択肢の飛び先・
/// Lua の検索 API（範囲省略の `SEARCH:search_scene`・`SCENE.co_exec`・`act:call`）がすべて宣言と同じシーンに
/// 解決される。空の表では導入前と同じく `会話_1` として登録・解決される。
#[traced_test]
#[test]
fn every_entry_resolves_to_declared_scene_under_each_table() {
    for table in Table::ALL {
        let (_temp, runtime) = load_ghost(MAIN_PASTA, table, Some(API_MAIN_LUA));
        let expected = "会話のシーンです";

        // 登録名（8.3）。既定・作者表では会話_N のグローバルシーンは無い
        let scenes = pasta_lua::runtime::finalize::collect_scenes(runtime.lua()).unwrap();
        assert!(
            scenes.iter().any(|(g, _)| g == table.kaiwa_id()),
            "{table:?}: {} が登録されていない: {scenes:?}",
            table.kaiwa_id()
        );
        let other = if table.aliases_kaiwa() {
            "会話_1"
        } else {
            "OnTalk_1"
        };
        assert!(
            !scenes.iter().any(|(g, _)| g == other),
            "{table:?}: {other} は登録されない: {scenes:?}"
        );

        // 範囲を省略した検索 API（4.6）
        assert_eq!(
            search_global(&runtime, "会話").as_deref(),
            Some(table.kaiwa_id()),
            "{table:?}: search_scene(\"会話\")"
        );
        // 置き換え後の名前で探しても同じシーン（空の表では OnTalk のグローバルシーンは無い）
        let ontalk = table.aliases_kaiwa().then_some("OnTalk_1");
        assert_eq!(
            search_global(&runtime, "OnTalk").as_deref(),
            ontalk,
            "{table:?}"
        );

        // Call: 静的名・「」囲み・変数の動的ターゲット（4.3）
        for scene in ["静的呼び出し", "文字列呼び出し", "変数呼び出し"] {
            assert_ok_with(table, scene, &fire(&runtime, scene), expected);
        }

        // Lua API: SCENE.co_exec・act:call（4.6）
        assert_ok_with(table, "co_exec", &fire(&runtime, "OnApiCoExec"), expected);
        assert_ok_with(table, "act:call", &fire(&runtime, "OnApiActCall"), expected);

        // 選択肢の飛び先（4.4）: 選択肢を出したシーンにローカル「会話」は無いのでグローバル検索に進む
        let shown = fire(&runtime, "選択肢表示");
        assert_ok_with(table, "選択肢表示", &shown, r"\q[話す,会話,選択肢表示_1]");
        let value = runtime
            .exec(
                r#"local EVENT = require "pasta.shiori.event"
return EVENT.fire({
    id = "OnChoiceSelectEx", method = "get", version = 30,
    reference = { [0] = "話す", [1] = "会話", [2] = "選択肢表示_1" },
})"#,
            )
            .expect("EVENT.fire に失敗");
        let chosen = value_as_str(&value).unwrap();
        assert_ok_with(table, "選択肢の飛び先", &chosen, expected);
    }
    assert!(!logs_contain("handler not found"));
}

/// 4.5・2.4: SHIORI イベントのシーン探索。作者定義の `OnBoot = ["起動"]` で `＊起動` が OnBoot で実行され、
/// 既定表・作者表では OnTalk のイベントで `＊会話` が実行される。空の表では導入前と同じく実行されない。
#[test]
fn shiori_event_scene_lookup_uses_declared_alias() {
    for table in Table::ALL {
        let (_temp, runtime) = load_ghost(MAIN_PASTA, table, None);

        let boot = fire(&runtime, "OnBoot");
        let boot_id = search_global(&runtime, "起動");
        if table == Table::Author {
            assert_ok_with(table, "OnBoot", &boot, "起動のシーンです");
            assert_eq!(boot_id.as_deref(), Some("OnBoot_1"), "{table:?}");
        } else {
            assert!(
                !boot.contains("起動のシーンです"),
                "{table:?}: 起動 は OnBoot の別名でない: {boot:?}"
            );
            assert_eq!(boot_id.as_deref(), Some("起動_1"), "{table:?}");
        }

        let talk = fire(&runtime, "OnTalk");
        if table.aliases_kaiwa() {
            assert_ok_with(table, "OnTalk", &talk, "会話のシーンです");
        } else {
            assert!(
                talk.starts_with("SHIORI/3.0 204"),
                "{table:?}: 空の表では OnTalk のシーンが無い: {talk:?}"
            );
        }

        // 作者の表は既定とマージせず、書いた別名だけを適用する（2.2・2.4）
        let zatsudan = (table == Table::Author).then_some("OnTalk_1");
        assert_eq!(
            search_global(&runtime, "雑談").as_deref(),
            zatsudan,
            "{table:?}"
        );
    }
}

/// 4.7・4.8・10.3: ローカルシーン `・会話`・単語 `＠会話`・アクター `％会話` は置き換わらず、範囲を指定した
/// 検索と登録名を直接受け取る入口（`SCENE.get_start`・キック）も置き換えない。
#[test]
fn names_outside_global_scene_search_are_not_replaced() {
    for table in Table::ALL {
        let (_temp, runtime) = load_ghost(MAIN_PASTA, table, None);

        // ローカルシーンの宣言名は置き換わらず、親の中の `＞会話` は 2 段目でローカルの「会話」に当たる
        let scenes = pasta_lua::runtime::finalize::collect_scenes(runtime.lua()).unwrap();
        assert!(
            scenes
                .iter()
                .any(|(g, l)| g == "ローカル親_1" && l.starts_with("会話")),
            "{table:?}: ローカルの 会話 が登録されていない: {scenes:?}"
        );
        assert_ok_with(
            table,
            "ローカル親",
            &fire(&runtime, "ローカル親"),
            "ローカルの会話です",
        );

        // 範囲を指定した検索は置き換えない（置き換えればローカルの OnTalk に当たってしまう）
        let (g, l) = search(&runtime, "会話", Some("ローカル親_1")).expect("scoped 会話");
        assert_eq!(g, "ローカル親_1", "{table:?}");
        assert!(l.starts_with("会話"), "{table:?}: scoped 会話 → {l}");
        let (_, l) = search(&runtime, "OnTalk", Some("ローカル親_1")).expect("scoped OnTalk");
        assert!(l.starts_with("OnTalk"), "{table:?}: scoped OnTalk → {l}");

        // 単語・アクター名は置き換えない
        assert_ok_with(table, "単語", &fire(&runtime, "単語参照"), "単語の会話です");
        assert_ok_with(
            table,
            "アクター",
            &fire(&runtime, "アクター参照"),
            "会話アクターの通常顔",
        );

        // 登録名を受け取る入口は書かれた登録名をそのまま引く（`会話_1` を `OnTalk_1` に読み替えない）
        let value = runtime
            .exec(
                r#"local SCENE = require "pasta.scene"
local STORE = require "pasta.store"
local KICK = require "pasta.shiori.event.kick"
local out = {}
for _, id in ipairs({ "会話_1", "OnTalk_1" }) do
    local start = type(SCENE.get_start(id)) == "function"
    STORE.kick_pending = id
    local kicked = KICK.try_dispatch(nil) ~= nil
    out[#out + 1] = id .. "=" .. tostring(start) .. "/" .. tostring(kicked)
end
return table.concat(out, ",")"#,
            )
            .unwrap();
        let expected = if table.aliases_kaiwa() {
            "会話_1=false/false,OnTalk_1=true/true"
        } else {
            "会話_1=true/true,OnTalk_1=false/false"
        };
        assert_eq!(value_as_str(&value).unwrap(), expected, "{table:?}");
    }
}

/// 1.3・1.4・5.3: `＊会話` と `＊OnTalk` は既定表・作者表で OnTalk の同じ候補の集まりになり、
/// 「会話」と「OnTalk」の検索が 1 つの順次消費の記録を進める。空の表では別々のシーンのまま。
#[test]
fn kaiwa_and_ontalk_share_one_sequential_consumption_state() {
    const PASTA: &str = "\
＊会話
  さくら：会話側です

＊OnTalk
  さくら：OnTalk側です
";
    for table in Table::ALL {
        let (_temp, runtime) = load_ghost(PASTA, table, None);
        // シャッフルを固定（候補順のまま）。記録が名前ごとに別なら 2 回とも先頭の候補が返る
        runtime
            .exec(r#"require("@pasta_search"):set_scene_selector(0, 1)"#)
            .unwrap();

        let first = search_global(&runtime, "会話");
        let second = search_global(&runtime, "OnTalk");
        if table.aliases_kaiwa() {
            let (a, b) = (first.unwrap(), second.unwrap());
            assert_ne!(a, b, "{table:?}: 会話→OnTalk で同じ記録を進める");
            let mut got = [a, b];
            got.sort();
            assert_eq!(got, ["OnTalk_1", "OnTalk_2"], "{table:?}");
        } else {
            assert_eq!(first.as_deref(), Some("会話_1"), "{table:?}");
            assert_eq!(second.as_deref(), Some("OnTalk_1"), "{table:?}");
        }
    }
}

/// 3.6: 既定表のもとで `＞会話` は `OnTalk` の前方一致になり、`＊OnTalk朝` も候補になる。
#[test]
fn aliased_call_prefix_matches_ontalk_suffixed_scene() {
    const PASTA: &str = "\
＊OnTalk朝
  さくら：OnTalk朝です

＊呼び出し
  ＞会話
  さくら：次の行です
";
    let (_temp, runtime) = load_ghost(PASTA, Table::Default, None);
    assert_eq!(
        search_global(&runtime, "会話").as_deref(),
        Some("OnTalk朝_1")
    );
    assert_ok_with(
        Table::Default,
        "呼び出し",
        &fire(&runtime, "呼び出し"),
        "OnTalk朝です",
    );
}

/// 3.7・8.1・5.4・5.6: 別名と完全一致する名前の検索は置き換え前の名前で始まる `＊会話・朝` を候補にせず、
/// 見つからない Call の失敗表記は書いた名前「会話」になり、Rust 側の warn に両方の名前が出る。
/// `＊会話・朝` 自体は置き換わらない。空の表では導入前と同じく `＞会話` が `＊会話・朝` に当たる。
#[traced_test]
#[test]
fn missing_aliased_call_reports_written_name_and_skips_prefix_scene() {
    const PASTA: &str = "\
＊会話・朝
  さくら：朝の会話です

＊呼び出し
  ＞会話
  さくら：次の行です
";
    for table in Table::ALL {
        let (_temp, runtime) = load_ghost(PASTA, table, None);

        assert_eq!(
            search_global(&runtime, "会話・朝").as_deref(),
            Some("会話_朝_1"),
            "{table:?}: ＊会話・朝 は置き換わらない"
        );

        let response = fire(&runtime, "呼び出し");
        if table.aliases_kaiwa() {
            assert_eq!(search_global(&runtime, "会話"), None, "{table:?}");
            assert_ok_with(
                table,
                "呼び出し",
                &response,
                "【Call失敗：「会話」が見つからない】\\p[0]次の行です",
            );
            assert!(
                !response.contains("朝の会話です"),
                "{table:?}: {response:?}"
            );
        } else {
            assert_eq!(
                search_global(&runtime, "会話").as_deref(),
                Some("会話_朝_1"),
                "{table:?}"
            );
            assert_ok_with(table, "呼び出し", &response, "朝の会話です");
        }
    }
    assert!(logs_contain("Scene not found (alias applied)"));
    assert!(logs_contain("name=会話 resolved=OnTalk"));
    assert!(logs_contain("handler not found: key='会話'"));
}
