//! external-control-events の結合テスト（task 3）。
//!
//! デバッグを無効にしたゴーストを一時フォルダに作って `PastaLoader::load_with_config` で読み込み、
//! `SHIORI.request` に GET のリクエスト（SecurityLevel: local）を渡して確かめる。
//! 既定のハンドラの読み込み（`event/init.lua` の 1 行）が効いていることも、このテストで確かめる。
//!
//! - 2 ビートのシーンで 200 の Value が 1 ビート目だけで、続きが `STORE.co_scene` に置かれる（1.5・2.1・5.1）
//! - 前方一致と既定の別名で当たる（1.1・1.2）
//! - 引数が文字列のまま渡り、出力を積まない呼び先で 204（1.4・1.6・1.7）
//! - `自動トーク停止` で 204 と停止の状態（1.6）
//! - `"01"` を代入した後に話すシーンで `01` が出る（3.2・3.6）
//! - 無い名前で表記付きの 200（4.2・4.6）、エラーを投げる関数で止まるエラーの応答（2.3）
//!
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use std::path::{Path, PathBuf};

use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};

/// 結合フィクスチャ
const FIXTURE: &str = "\
＃ external-control-events 結合フィクスチャ
＊初回起動
　さくら：「一拍目」
　＞チェイントーク
　さくら：「二拍目」

＊挨拶朝
　さくら：「おはよう」

＊OnTalk
　さくら：「ふだんの会話」

＊OnPastaCallFunction
　さくら：「フォールバック」

＊会員番号を話す
　さくら：「番号は＄＊会員番号　です」

＊関数置き場
```lua
GLOBAL[\"好感度を足す\"] = function(act, ...)
    _G.__received = { n = select(\"#\", ...), ... }
end
GLOBAL[\"壊れる関数\"] = function(act)
    error(\"わざと壊した\")
end
```
　さくら：「置き場」
";

/// 辞書のファイルの、ghost/master/ からの相対パス
const REL_PATH: &str = "dic/test/external_control.pasta";

/// デバッグを無効にした一時ゴーストを作って読み込む。
fn load_ghost(base: &Path) -> PastaLuaRuntime {
    let pasta_file = base.join(REL_PATH);
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, FIXTURE).unwrap();
    std::fs::write(base.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let dst = base.join(sub);
        std::fs::create_dir_all(&dst).unwrap();
        common::copy_dir_recursive(&crate_root.join(sub), &dst).unwrap();
    }

    let runtime =
        PastaLoader::load_with_config(base, RuntimeConfig::new()).expect("ghost must load");
    assert!(!runtime.debug_enabled(), "debug must stay disabled");
    runtime
}

/// `SHIORI.request` に GET のリクエスト（SecurityLevel: local）を渡し、応答の文字列を返す。
fn request(runtime: &PastaLuaRuntime, id: &str, refs: &[&str]) -> String {
    let lua = runtime.lua();
    let req = lua.create_table().unwrap();
    req.set("id", id).unwrap();
    req.set("method", "get").unwrap();
    req.set("version", 30).unwrap();
    req.set("security_level", "local").unwrap();
    req.set("dic", lua.create_table().unwrap()).unwrap();
    let reference = lua.create_table().unwrap();
    for (i, value) in refs.iter().enumerate() {
        reference.set(i, *value).unwrap();
    }
    req.set("reference", reference).unwrap();

    let shiori: Table = lua
        .load(r#"return require("pasta.shiori.entry")"#)
        .eval()
        .unwrap();
    shiori
        .get::<Function>("request")
        .unwrap()
        .call::<String>(req)
        .expect("SHIORI.request must return a response string")
}

/// 200 OK であることを確かめ、Value の行の値を返す。
fn value_of_200(response: &str) -> String {
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 200 OK"),
        "200 expected: {response:?}"
    );
    response
        .lines()
        .find_map(|l| l.strip_prefix("Value: "))
        .unwrap_or_else(|| panic!("Value line expected: {response:?}"))
        .to_string()
}

fn assert_204(response: &str) {
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 204 No Content"),
        "204 expected: {response:?}"
    );
}

/// Lua の式を評価して真偽を返す。
fn lua_bool(runtime: &PastaLuaRuntime, code: &str) -> bool {
    runtime.lua().load(code).eval::<bool>().unwrap()
}

/// 1.5・2.1・5.1: デバッグが無効でも、2 ビートのシーンの 1 ビート目だけを返し、続きを置く。
#[test]
fn call_function_plays_first_beat_and_places_continuation() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let value = value_of_200(&request(&runtime, "OnPastaCallFunction", &["初回起動"]));
    assert!(value.contains("一拍目"), "beat 1 delivered: {value}");
    assert!(!value.contains("二拍目"), "beat 2 must wait: {value}");
    assert!(
        lua_bool(
            &runtime,
            r#"
            local STORE = require("pasta.store")
            return type(STORE.co_scene) == "thread"
                and coroutine.status(STORE.co_scene) == "suspended"
            "#,
        ),
        "continuation must be placed in STORE.co_scene"
    );
}

/// 1.1・1.2: 前方一致と既定の別名で当たる。
#[test]
fn call_function_finds_by_prefix_and_alias() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let value = value_of_200(&request(&runtime, "OnPastaCallFunction", &["挨拶"]));
    assert!(value.contains("おはよう"), "prefix match: {value}");

    let value = value_of_200(&request(&runtime, "OnPastaCallFunction", &["会話"]));
    assert!(value.contains("ふだんの会話"), "default alias: {value}");
}

/// 1.4・1.6・1.7: 引数は文字列のまま渡り、出力を積まない呼び先は 204。
#[test]
fn call_function_passes_string_args_and_returns_204() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    assert_204(&request(
        &runtime,
        "OnPastaCallFunction",
        &["好感度を足す", "10", ""],
    ));
    assert!(
        lua_bool(
            &runtime,
            r#"
            local r = _G.__received
            return r.n == 2 and r[1] == "10" and type(r[1]) == "string" and r[2] == ""
            "#,
        ),
        "args must arrive as strings"
    );
}

/// 1.6: `自動トーク停止` で 204 と停止の状態。
#[test]
fn call_function_pauses_auto_talk() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    assert_204(&request(
        &runtime,
        "OnPastaCallFunction",
        &["自動トーク停止"],
    ));
    assert!(
        lua_bool(
            &runtime,
            r#"return require("pasta.store").auto_talk_paused == true"#
        ),
        "auto talk must be paused"
    );
}

/// 3.2・3.6: `"01"` を代入した後に話すシーンで `01` が出て、値は文字列のまま。
#[test]
fn set_save_variable_then_scene_speaks_the_string() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    assert_204(&request(
        &runtime,
        "OnPastaSetSaveVariable",
        &["会員番号", "\"01\""],
    ));
    assert!(
        lua_bool(
            &runtime,
            r#"return require("pasta.save")["会員番号"] == "01""#
        ),
        "save must hold the string \"01\""
    );

    let value = value_of_200(&request(
        &runtime,
        "OnPastaCallFunction",
        &["会員番号を話す"],
    ));
    assert!(value.contains("番号は01"), "assigned value spoken: {value}");
}

/// 4.2・4.6・4.8・5.4: 無い名前で 200 と表記。待っている続きは残り、
/// `＊OnPastaCallFunction` へのフォールバックも起きない。
#[test]
fn call_function_reports_not_found() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    request(&runtime, "OnPastaCallFunction", &["初回起動"]);
    runtime
        .lua()
        .load("_G.__co = require(\"pasta.store\").co_scene")
        .exec()
        .unwrap();

    let value = value_of_200(&request(&runtime, "OnPastaCallFunction", &["無い名前"]));
    assert!(
        value.contains("【シーンが見つからない：OnPastaCallFunction ＞無い名前】"),
        "not found notice: {value}"
    );
    assert!(
        !value.contains("フォールバック"),
        "no fallback scene: {value}"
    );
    assert!(
        lua_bool(
            &runtime,
            r#"
            local STORE = require("pasta.store")
            return STORE.co_scene == _G.__co
                and coroutine.status(STORE.co_scene) == "suspended"
            "#,
        ),
        "continuation must be kept on failure"
    );
}

/// 2.3: エラーを投げる関数で、ふだんのイベントと同じ止まるエラーの応答になる。
#[test]
fn call_function_error_gives_stop_response() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let response = request(&runtime, "OnPastaCallFunction", &["壊れる関数"]);
    let value = value_of_200(&response);
    assert!(
        value.contains("【スクリプトのエラー："),
        "stop notice: {value}"
    );
    assert!(
        response.contains("X-Error-Reason: ") && response.contains("わざと壊した"),
        "error reason: {response:?}"
    );
}
