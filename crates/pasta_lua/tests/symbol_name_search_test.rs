//! 記号を含むシーン名の Call と検索の結合テスト（scene-search-key-normalization）。
//!
//! テスト用の `.pasta` で一時ゴーストを作り、`PastaLoader::load` で実物のランタイムを起こす。
//! シーンの出力は `EVENT.fire` の応答文字列で、警告は `tracing_test` の捕捉で確かめる。
//! `mod common` の宣言により `PASTA_DEBUG` 中和ガード（`#[ctor]`）が適用される。

mod common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;
use tracing_test::traced_test;

/// 記号を含むシーン名を Call・検索する入口シーンを集めたゴースト。
const SYMBOL_SCENES_PASTA: &str = "\
＊静的呼び出し
  さくら：前置き
  ＞会話・朝
  さくら：後置き

＊文字列呼び出し
  ＞「会話・朝」

＊変数呼び出し
  ＄行き先＝「会話・朝」
  ＞＄行き先

＊会話・朝
  さくら：朝の会話です

＊選択テスト
  ＞選択・A

  ・選択・A
    さくら：Aを選びました

＊不明呼び出し
  ＞存在しない・朝
  さくら：次の行です

＊OnSymbolCall
  ＞会話・朝

＊選択肢表示
  さくら：どれにする？
  ＠？選択・A「Aにする」

  ・選択・A
    さくら：選択肢からAに来ました
";

/// テスト用ゴーストのアクター設定（pasta.toml へ追記する）。
const ACTOR_TOML: &str = "[actor.\"さくら\"]
spot = 0
";

/// 一時ゴーストを作って実物のランタイムを読み込む。`extra_toml` は pasta.toml へ追記する。
/// `TempDir` はランタイムより長く保持すること。
fn load_ghost(pasta: &str, extra_toml: &str) -> (TempDir, PastaLuaRuntime) {
    let temp = create_temp_with_pasta(pasta);
    let toml = temp.path().join("pasta.toml");
    let base = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(&toml, format!("{base}{extra_toml}")).unwrap();
    let runtime = PastaLoader::load(temp.path()).expect("ゴーストの読み込みに失敗");
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

fn assert_ok_with(response: &str, text: &str) {
    assert!(
        response.starts_with("SHIORI/3.0 200 OK"),
        "応答が 200 でない: {response:?}"
    );
    assert!(
        response.contains(text),
        "応答に {text:?} が無い: {response:?}"
    );
}

/// 要件 1.1・7.1: `＞会話・朝` で記号を含むグローバルシーンを Call でき、警告が出ない。
#[traced_test]
#[test]
fn static_call_reaches_symbol_global_scene_without_warning() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    let response = fire(&runtime, "静的呼び出し");

    assert_ok_with(&response, "前置き");
    assert_ok_with(&response, "朝の会話です");
    assert_ok_with(&response, "後置き");
    assert!(!logs_contain("handler not found"));
}

/// 要件 1.2: 動的ターゲット（文字列・変数）で `会話・朝` を指しても同じシーンが実行される。
#[traced_test]
#[test]
fn dynamic_call_reaches_symbol_global_scene() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    assert_ok_with(&fire(&runtime, "文字列呼び出し"), "朝の会話です");
    assert_ok_with(&fire(&runtime, "変数呼び出し"), "朝の会話です");
    assert!(!logs_contain("handler not found"));
}

/// 要件 1.3: `SCENE.search` と `@pasta_search` の `search_scene` が記号を含む名前で結果を返す。
#[test]
fn scene_search_apis_find_symbol_global_scene() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    let value = runtime
        .exec(
            r#"local SCENE = require "pasta.scene"
local SEARCH = require "@pasta_search"
local found = SCENE.search("会話・朝")
local global_name, local_name = SEARCH:search_scene("会話・朝")
return tostring(found and found.global_name) .. "|" .. tostring(global_name) .. "|" .. tostring(local_name)"#,
        )
        .unwrap();
    let result = value_as_str(&value).unwrap();

    let parts: Vec<&str> = result.split('|').collect();
    assert_ne!(parts[0], "nil", "SCENE.search が見つけない: {result}");
    assert_ne!(parts[1], "nil", "search_scene が見つけない: {result}");
    assert_eq!(parts[0], parts[1], "2 つの API の結果が食い違う: {result}");
    assert_eq!(
        parts[2], "__start__",
        "グローバルシーンの入口でない: {result}"
    );
}

/// 要件 2.1・2.2・7.2: グローバルシーンの中の `・選択・A` を `＞選択・A` で Call でき、
/// `SCENE.search("選択・A", 登録名)` が結果を返す。
#[traced_test]
#[test]
fn symbol_local_scene_is_callable_and_searchable() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    assert_ok_with(&fire(&runtime, "選択テスト"), "Aを選びました");
    assert!(!logs_contain("handler not found"));

    let value = runtime
        .exec(
            r#"local SCENE = require "pasta.scene"
local parent = SCENE.search("選択テスト")
local found = parent and SCENE.search("選択・A", parent.global_name)
return found and (found.global_name .. "|" .. found.local_name) or "nil""#,
        )
        .unwrap();
    let result = value_as_str(&value).unwrap();

    assert_ne!(result, "nil", "ローカルシーンが見つからない");
    let (_, local_name) = result.split_once('|').unwrap();
    assert_ne!(local_name, "__start__", "ローカルシーンでない: {result}");
}

/// 要件 3.5: 存在しない記号つきの名前を Call すると次の行へ進み、警告に元の名前が出る。
#[traced_test]
#[test]
fn missing_symbol_call_warns_with_raw_name_and_continues() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    assert_ok_with(&fire(&runtime, "不明呼び出し"), "次の行です");
    assert!(logs_contain("handler not found: key='存在しない・朝'"));
}

/// 要件 1.5: SHIORI イベントのシーンが記号を含むグローバルシーンを Call すると、
/// 応答が 200 でシーンの出力を含む（204 にならない）。
#[test]
fn event_scene_calling_symbol_global_scene_responds_200() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    let response = fire(&runtime, "OnSymbolCall");

    assert!(
        !response.starts_with("SHIORI/3.0 204"),
        "204 になった: {response:?}"
    );
    assert_ok_with(&response, "朝の会話です");
}

/// 要件 2.4・7.2: 選択肢行 `＠？選択・A「Aにする」` を出したあと、選択 ID `選択・A` の
/// `OnChoiceSelectEx` を起こすと、記号を含むローカルシーンが実行される（204 にならない）。
#[test]
fn choice_select_jumps_to_symbol_local_scene() {
    let (_temp, runtime) = load_ghost(SYMBOL_SCENES_PASTA, ACTOR_TOML);

    assert_ok_with(&fire(&runtime, "選択肢表示"), r"\q[Aにする,選択・A]");

    let value = runtime
        .exec(
            r#"local EVENT = require "pasta.shiori.event"
return EVENT.fire({
    id = "OnChoiceSelectEx", method = "get", version = 30,
    reference = { [0] = "Aにする", [1] = "選択・A" },
})"#,
        )
        .expect("EVENT.fire に失敗");
    let response = value_as_str(&value).expect("応答が文字列でない");

    assert!(
        !response.starts_with("SHIORI/3.0 204"),
        "204 になった: {response:?}"
    );
    assert_ok_with(&response, "選択肢からAに来ました");
}
