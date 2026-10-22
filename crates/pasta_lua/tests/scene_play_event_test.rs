//! scene-play-event の結合テスト（task 4.1）。
//!
//! デバッグを有効にしたゴーストを一時フォルダに作って `PastaLoader::load_with_config` で読み込み、
//! `SHIORI.request` に GET のリクエスト（`OnPastaScene`・`OnPastaSceneAt`）を渡して確かめる。
//!
//! - 名前で大域・ローカルを再生し、200 の Value が 1 ビート目だけで、続きが
//!   `STORE.co_scene` に置かれる（要件 1.1・1.2・3.1・3.2・3.3）
//! - 同じ名前の大域が 2 つあり片方だけがローカルを持つとき、毎回再生できる（要件 1.8）
//! - 相対パスと絶対パスで位置から再生できる（要件 2.1・2.3・2.4）
//! - 待ち受けが立たなくてもソースマップが残り、位置で再生できる（要件 4.2・4.3、task 4.2）
//! - デバッグが無効なら 204、見つからないときは 200 で失敗の表記（要件 4.4・5.1・5.3・7.1・7.4、task 4.2）
//!
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従い、待ち受けのポートは `port = 0` にする。

mod common;

use std::path::{Path, PathBuf};

use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};

/// 結合フィクスチャ。行番号は `LINE_*` が依存するので、編集するときは合わせること。
const FIXTURE: &str = "\
＃ scene-play-event 4.1 結合フィクスチャ
＊再生大域
　さくら：「大域の声」

＊再生親
　さくら：「親の声」

　・再生子
　　さくら：「子の声」

＊二本
　さくら：「二本の一本目」

＊二本
　さくら：「二本の二本目」

　・だけ
　　さくら：「片方のローカル」

＊二拍
　さくら：「一拍目」
　＞チェイントーク
　さくら：「二拍目」
";

/// `＊再生大域` の本体の行
const LINE_GLOBAL_BODY: u32 = 3;
/// `・再生子` の本体の行
const LINE_LOCAL_BODY: u32 = 9;
/// 2 つ目の `＊二本` の本体の行
const LINE_SECOND_TWIN_BODY: u32 = 15;

/// 辞書のファイルの、ghost/master/ からの相対パス
const REL_PATH: &str = "dic/test/scene_play.pasta";

/// デバッグを有効にした一時ゴーストを作って読み込む。
fn load_debug_ghost(base: &Path) -> PastaLuaRuntime {
    load_ghost(
        base,
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\nport = 0\n",
    )
}

/// 指定の pasta.toml で一時ゴーストを作って読み込む。
fn load_ghost(base: &Path, pasta_toml: &str) -> PastaLuaRuntime {
    let pasta_file = base.join(REL_PATH);
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, FIXTURE).unwrap();
    std::fs::write(base.join("pasta.toml"), pasta_toml).unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let dst = base.join(sub);
        std::fs::create_dir_all(&dst).unwrap();
        common::copy_dir_recursive(&crate_root.join(sub), &dst).unwrap();
    }

    PastaLoader::load_with_config(base, RuntimeConfig::new()).expect("ghost must load")
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

/// Lua の式を評価して真偽を返す。
fn lua_bool(runtime: &PastaLuaRuntime, code: &str) -> bool {
    runtime.lua().load(code).eval::<bool>().unwrap()
}

/// 1.1・1.2・3.1・3.2・3.3: 名前で大域とローカルを再生し、1 ビート目だけを返し、続きを場面の続きに置く。
#[test]
fn on_pasta_scene_plays_global_and_local_by_name() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    // 1.1: 大域だけ（Reference1 なし）
    let value = value_of_200(&request(&runtime, "OnPastaScene", &["再生大域"]));
    assert!(value.contains("大域の声"), "global scene played: {value}");

    // 1.2: 大域の中のローカル（大域の本体は流さない）
    let value = value_of_200(&request(&runtime, "OnPastaScene", &["再生親", "再生子"]));
    assert!(value.contains("子の声"), "local scene played: {value}");
    assert!(
        !value.contains("親の声"),
        "global body must not play: {value}"
    );

    // 3.1: 進行中の場面の続きを中断し、元に戻さない
    runtime
        .exec(
            r#"
            local STORE = require("pasta.store")
            _G.__old_co = coroutine.create(function() coroutine.yield("古い続き") end)
            STORE.co_scene = _G.__old_co
            "#,
        )
        .unwrap();

    // 3.2: 2 ビートのシーンは 1 ビート目だけを返す
    let value = value_of_200(&request(&runtime, "OnPastaScene", &["二拍"]));
    assert!(value.contains("一拍目"), "beat 1 delivered: {value}");
    assert!(
        !value.contains("二拍目"),
        "beat 2 must not be delivered yet: {value}"
    );

    // 3.3: 続きは STORE.co_scene に置かれ、古い続きは戻らない
    assert!(
        lua_bool(
            &runtime,
            r#"
            local STORE = require("pasta.store")
            return type(STORE.co_scene) == "thread"
                and STORE.co_scene ~= _G.__old_co
                and coroutine.status(STORE.co_scene) == "suspended"
            "#,
        ),
        "continuation must be placed in STORE.co_scene, replacing the old one"
    );

    // 続きを、ふだんの続き（OnTalk の時機に STORE.co_scene を返す）と同じ形で流すと 2 ビート目が出る
    runtime
        .exec(
            r#"
            local REG = require("pasta.shiori.event.register")
            REG.OnTestContinue = function() return require("pasta.store").co_scene end
            "#,
        )
        .unwrap();
    let value = value_of_200(&request(&runtime, "OnTestContinue", &[]));
    assert!(
        value.contains("二拍目"),
        "beat 2 delivered from continuation: {value}"
    );
    assert!(!value.contains("一拍目"), "beat 1 must not repeat: {value}");
}

/// 1.8: 同じ名前の大域が 2 つあり、片方だけがローカルを持つとき、ローカルを毎回再生できる。
#[test]
fn on_pasta_scene_plays_local_owned_by_one_of_twin_globals_every_time() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    for i in 0..6 {
        let value = value_of_200(&request(&runtime, "OnPastaScene", &["二本", "だけ"]));
        assert!(
            value.contains("片方のローカル"),
            "attempt {i}: local owned by one twin must play every time: {value}"
        );
    }
}

/// 2.1・2.3・2.4: 相対パスと絶対パスで位置から再生する。
#[test]
fn on_pasta_scene_at_plays_by_relative_and_absolute_path() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    // 2.3: ghost/master/ からの相対パス（ローカルの行）
    let line = LINE_LOCAL_BODY.to_string();
    let value = value_of_200(&request(&runtime, "OnPastaSceneAt", &[REL_PATH, &line]));
    assert!(
        value.contains("子の声"),
        "relative path plays local: {value}"
    );
    assert!(
        !value.contains("親の声"),
        "global body must not play: {value}"
    );

    // 2.4: 絶対パス（大域の行）
    let abs = temp.path().join(REL_PATH).to_string_lossy().to_string();
    let line = LINE_GLOBAL_BODY.to_string();
    let value = value_of_200(&request(&runtime, "OnPastaSceneAt", &[&abs, &line]));
    assert!(
        value.contains("大域の声"),
        "absolute path plays global: {value}"
    );

    // 2.1: 同じ名前の大域のうち、指した位置のものを毎回再生する
    let line = LINE_SECOND_TWIN_BODY.to_string();
    for i in 0..4 {
        let value = value_of_200(&request(&runtime, "OnPastaSceneAt", &[REL_PATH, &line]));
        assert!(
            value.contains("二本の二本目"),
            "attempt {i}: pointed twin must play: {value}"
        );
    }
}

/// 4.2・4.3: 待ち受けのポートを先に塞いでも、ソースマップが残り、位置で再生できる。
#[test]
fn on_pasta_scene_at_plays_when_listener_cannot_bind() {
    let temp = tempfile::TempDir::new().unwrap();
    // 空いたポートを先に塞ぐ（固定のポートは使わない）。読み込みが終わるまで持ち続ける
    let blocker = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = blocker.local_addr().unwrap().port();
    let runtime = load_ghost(
        temp.path(),
        &format!("[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\nport = {port}\n"),
    );

    assert!(!runtime.debug_enabled(), "listener must not be up");
    assert!(
        runtime.debug_source_map().is_some(),
        "source map must be kept without a listener"
    );

    let line = LINE_GLOBAL_BODY.to_string();
    let value = value_of_200(&request(&runtime, "OnPastaSceneAt", &[REL_PATH, &line]));
    assert!(
        value.contains("大域の声"),
        "plays by position without a listener: {value}"
    );
    drop(blocker);
}

/// 4.4・7.1: デバッグを無効にしたゴーストでは、2 つのイベントが 204 になる。
#[test]
fn scene_play_events_return_204_when_debug_disabled() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), "[loader]\ndebug_mode = true\n");

    let line = LINE_GLOBAL_BODY.to_string();
    for (id, refs) in [
        ("OnPastaScene", vec!["再生大域"]),
        ("OnPastaSceneAt", vec![REL_PATH, line.as_str()]),
    ] {
        let response = request(&runtime, id, &refs);
        assert_eq!(
            response.lines().next(),
            Some("SHIORI/3.0 204 No Content"),
            "{id} must be 204 when debug is disabled: {response:?}"
        );
        assert!(
            !response.contains("大域の声"),
            "{id} must not play: {response:?}"
        );
    }
}

/// 5.1・5.2・5.3: 見つからないときは 200 で、イベント名と探した名前か位置を含む失敗の表記が返る。
#[test]
fn scene_play_events_return_failure_notice_when_not_found() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    let value = value_of_200(&request(&runtime, "OnPastaScene", &["無い名前"]));
    assert!(
        value.contains("【シーンが見つからない：OnPastaScene ＊無い名前】"),
        "global not found: {value}"
    );

    let value = value_of_200(&request(&runtime, "OnPastaScene", &["再生親", "無い子"]));
    assert!(
        value.contains("【シーンが見つからない：OnPastaScene ＊再生親 ・無い子】"),
        "local not found: {value}"
    );

    let value = value_of_200(&request(
        &runtime,
        "OnPastaSceneAt",
        &["dic/test/無い.pasta", "1"],
    ));
    assert!(
        value.contains("【シーンが見つからない：OnPastaSceneAt dic/test/無い.pasta:1】"),
        "position not found: {value}"
    );
}
