//! scene-play-event の結合テスト（task 4.1）。
//!
//! デバッグを有効にしたゴーストを一時フォルダに作って `PastaLoader::load_with_config` で読み込み、
//! `SHIORI.request` に GET のリクエスト（`OnPastaScene`・`OnPastaSceneAt`）を渡して確かめる。
//!
//! - 名前で大域・ローカルを再生し、200 の Value が 1 ビート目だけで、続きが
//!   `STORE.co_scene` に置かれる（要件 1.1・1.2・3.1・3.2・3.3）
//! - 同じ名前の大域が 2 つあり片方だけがローカルを持つとき、毎回再生できる（要件 1.8）
//! - 相対パスと絶対パスで位置から再生できる（要件 2.1・2.3・2.4）
//! - 有効ならシーンマップがあり、`debug_enabled()` と `@pasta_debug.enabled()` が真になる（debug-mcp-runtime 7.6）
//! - デバッグが無効なら 204、見つからないときは 200 で失敗の表記（要件 4.4・5.1・5.3・7.1・7.4、task 4.2）
//! - `OnPastaDumpVariables` は有効なとき 204 で `variables.toml` を書き、無効・external なら
//!   204 と警告で書かない（debug-mcp-runtime 4.2・4.3・5.1〜5.3・5.6）
//! - リロードで有効・無効が切り替わり、JIT が働き、`[debug]` の古いキーは警告なく読み飛ばす
//!   （debug-mcp-runtime 1.2・3.2・3.3・3.8）
//!
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

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

＊天気を言う
　さくら：「今日は＄天気」

＊型を言う
```lua
function SCENE.次の回数(act)
    local save, var = act:init_scene(SCENE)
    return tostring(var.回数 + 1) .. \"、\" .. type(var.晴れ)
end
```
　さくら：「天気は＄天気、回数は＄回数、晴れは＄晴れ、日は＄日」
　さくら：＠次の回数（）

　・型のローカル
　　さくら：「ローカルでは＄天気、回数は＄回数」

＊呼ぶ元
　＞呼ばれ先

＊呼ばれ先
　さくら：「呼ばれ先で＄天気」

＊代入する
　＄回数＝10
　さくら：「代入の後は＄回数」

＊続きで言う
　さくら：「一言目」
　＞チェイントーク
　さくら：「続きで＄天気」

＊参照を言う
　さくら：「参照は＄r0」
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
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\n",
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
    request_at_level(runtime, id, refs, "local")
}

/// `SHIORI.request` に GET のリクエストを、指定の SecurityLevel で渡し、応答の文字列を返す。
fn request_at_level(runtime: &PastaLuaRuntime, id: &str, refs: &[&str], level: &str) -> String {
    let lua = runtime.lua();
    let req = lua.create_table().unwrap();
    req.set("id", id).unwrap();
    req.set("method", "get").unwrap();
    req.set("version", 30).unwrap();
    req.set("security_level", level).unwrap();
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

/// 4.2・4.3（debug-mcp-runtime 7.6）: 有効ならシーンマップがあり、`debug_enabled()` と
/// `@pasta_debug.enabled()` がどちらも真で、位置で再生できる。
#[test]
fn enabled_debug_holds_scene_map_and_reports_enabled() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    assert!(
        runtime.debug_source_map().is_some(),
        "scene map must be held"
    );
    assert!(runtime.debug_enabled(), "debug_enabled() must be true");
    assert!(
        lua_bool(&runtime, r#"return require("@pasta_debug").enabled()"#),
        "@pasta_debug.enabled() must be true"
    );

    let line = LINE_GLOBAL_BODY.to_string();
    let value = value_of_200(&request(&runtime, "OnPastaSceneAt", &[REL_PATH, &line]));
    assert!(value.contains("大域の声"), "plays by position: {value}");
}

/// 4.4・7.1: デバッグを無効にしたゴーストでは、2 つのイベントが 204 になる。
#[test]
fn scene_play_events_return_204_when_debug_disabled() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), "[loader]\ndebug_mode = true\n");
    assert!(
        runtime.debug_source_map().is_none(),
        "no scene map when disabled"
    );
    assert!(!runtime.debug_enabled(), "debug_enabled() must be false");
    assert!(
        !lua_bool(&runtime, r#"return require("@pasta_debug").enabled()"#),
        "@pasta_debug.enabled() must be false"
    );

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

/// debug-mcp-runtime 1.5: 有効なゴーストのログには、`Startup sequence completed` の直前に
/// 開発用のイベントを許したことの info が 1 行あり、無効なゴーストには無い。
#[test]
fn debug_enabled_startup_log_line_only_when_enabled() {
    const LINE: &str = "Debug enabled: development events are allowed";
    pasta_lua::init_tracing();

    let read_log = |enabled: bool| {
        let temp = tempfile::TempDir::new().unwrap();
        let runtime = if enabled {
            load_debug_ghost(temp.path())
        } else {
            load_ghost(temp.path(), "[loader]\ndebug_mode = true\n")
        };
        // ランタイムと登録簿の両方が手放すとロガーが破棄され、ログがフラッシュされる。
        drop(runtime);
        pasta_lua::GlobalLoggerRegistry::instance().unregister(temp.path());
        std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log")).unwrap()
    };

    let enabled_log = read_log(true);
    let lines: Vec<&str> = enabled_log.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.contains(LINE))
        .unwrap_or_else(|| panic!("enabled ghost must log {LINE:?}: {enabled_log}"));
    assert_eq!(
        lines.iter().filter(|l| l.contains(LINE)).count(),
        1,
        "exactly one line: {enabled_log}"
    );
    assert!(lines[at].contains("INFO"), "info level: {}", lines[at]);
    assert!(
        lines
            .get(at + 1)
            .is_some_and(|l| l.contains("Startup sequence completed")),
        "must come right before Startup sequence completed: {enabled_log}"
    );

    let disabled_log = read_log(false);
    assert!(
        disabled_log.contains("Startup sequence completed"),
        "{disabled_log}"
    );
    assert!(!disabled_log.contains(LINE), "{disabled_log}");
}

/// debug-mcp-runtime 3.2・1.2: 同じフォルダを有効 → 無効 → 有効で読み直すと、`debug_enabled()` と
/// `OnPastaScene` の 200・204 が切り替わり、どの VM でも JIT が働く。
#[test]
fn reload_toggles_debug_and_keeps_jit() {
    let temp = tempfile::TempDir::new().unwrap();
    for enabled in [true, false, true] {
        let runtime = load_ghost(
            temp.path(),
            &format!("[loader]\ndebug_mode = true\n\n[debug]\nenabled = {enabled}\n"),
        );
        assert_eq!(runtime.debug_enabled(), enabled, "debug_enabled()");
        let response = request(&runtime, "OnPastaScene", &["再生大域"]);
        if enabled {
            assert!(value_of_200(&response).contains("大域の声"), "{response:?}");
        } else {
            assert_204(&response, "disabled after reload");
        }
        assert!(
            lua_bool(&runtime, "return jit.status()"),
            "JIT must be on (enabled = {enabled})"
        );
        drop(runtime);
    }
}

/// debug-mcp-runtime 3.3・3.8: `[debug]` に古いキーが残っていても読み込みが通り、有効で、
/// 警告が無く、`.lua.map` を作らない。
#[test]
fn old_debug_keys_are_skipped_silently() {
    pasta_lua::init_tracing();
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(
        temp.path(),
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\nport = \"x\"\npresent_as = \"lua\"\nsource_map_sidecar = true\n",
    );
    assert!(runtime.debug_enabled(), "debug_enabled() must be true");

    // ランタイムと登録簿の両方が手放すとロガーが破棄され、ログがフラッシュされる。
    drop(runtime);
    pasta_lua::GlobalLoggerRegistry::instance().unregister(temp.path());
    let log = std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log")).unwrap();
    assert!(log.contains("Startup sequence completed"), "{log}");
    assert!(!log.contains("WARN"), "no warning expected: {log}");

    let pattern = format!("{}/**/*.lua.map", temp.path().display());
    let maps: Vec<_> = glob::glob(&pattern).unwrap().collect();
    assert!(maps.is_empty(), "no .lua.map expected: {maps:?}");
}

/// 書き出す先（ghost/master/ から見たパス）
const VARIABLES_TOML: &str = "profile/pasta/debug/variables.toml";

/// 204 であることを確かめる。
fn assert_204(response: &str, what: &str) {
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 204 No Content"),
        "{what}: {response:?}"
    );
}

/// セーブのファイル（`save.toml`・`save.dat`）が無いことを確かめる。
fn assert_no_save_file(base: &Path) {
    let files: Vec<_> = ["save.toml", "save.dat"]
        .map(|name| base.join("profile/pasta/save").join(name))
        .into_iter()
        .filter(|p| p.exists())
        .collect();
    assert!(files.is_empty(), "save file must not be written: {files:?}");
}

/// 5.1・5.2・5.3・5.6: 有効なとき 204 で、保存変数を要件 5 の例のとおりテキストの TOML に書く。
/// 2 回目で置き換わり、フォルダが無くても作り、`obfuscate = true` でもテキストで、
/// `save.toml` を書かず、保存変数と場面の続きを変えない。
#[test]
fn on_pasta_dump_variables_writes_variables_toml_when_enabled() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(
        temp.path(),
        "[loader]
debug_mode = true

[debug]
enabled = true

[persistence]
obfuscate = true
",
    );
    let path = temp.path().join(VARIABLES_TOML);
    assert!(
        !path.parent().unwrap().exists(),
        "debug folder must not exist before the dump"
    );

    runtime
        .exec(
            r#"
            local save = require("pasta.save")
            save.好感度 = 3
            save.名前 = "まさお"
            local STORE = require("pasta.store")
            _G.__co = coroutine.create(function() coroutine.yield("続き") end)
            STORE.co_scene = _G.__co
            "#,
        )
        .unwrap();

    assert_204(
        &request(&runtime, "OnPastaDumpVariables", &[]),
        "dump must be 204",
    );
    let text = std::fs::read_to_string(&path).expect("variables.toml must be written as text");
    let doc: toml::Table = text.parse().unwrap();
    assert_eq!(
        doc.get("好感度").and_then(|v| v.as_integer()),
        Some(3),
        "{text}"
    );
    assert_eq!(
        doc.get("名前").and_then(|v| v.as_str()),
        Some("まさお"),
        "{text}"
    );
    assert_eq!(doc.len(), 2, "only the save variables: {text}");

    // 2 回目はそのときの値で置き換わる
    runtime.exec(r#"require("pasta.save").好感度 = 4"#).unwrap();
    assert_204(
        &request(&runtime, "OnPastaDumpVariables", &[]),
        "second dump must be 204",
    );
    let doc: toml::Table = std::fs::read_to_string(&path).unwrap().parse().unwrap();
    assert_eq!(doc.get("好感度").and_then(|v| v.as_integer()), Some(4));

    // 保存変数・場面の続きは変わらず、save.toml は書かれない
    assert!(
        lua_bool(
            &runtime,
            r#"
            local save = require("pasta.save")
            local STORE = require("pasta.store")
            return save.好感度 == 4 and save.名前 == "まさお"
                and STORE.co_scene == _G.__co
                and coroutine.status(_G.__co) == "suspended"
            "#,
        ),
        "save variables and continuation must be unchanged"
    );
    assert_no_save_file(temp.path());
}

/// 4.2・4.3: 無効なら・external なら 204 で、警告を書き、ファイルを書かない。
#[test]
fn on_pasta_dump_variables_refused_when_disabled_or_external() {
    pasta_lua::init_tracing();

    let disabled = tempfile::TempDir::new().unwrap();
    let external = tempfile::TempDir::new().unwrap();
    for (temp, pasta_toml, level, warning) in [
        (
            &disabled,
            "[loader]
debug_mode = true
",
            "local",
            "OnPastaDumpVariables: debug is disabled; variables not dumped",
        ),
        (
            &external,
            "[loader]
debug_mode = true

[debug]
enabled = true
",
            "external",
            "OnPastaDumpVariables: SecurityLevel external is refused; variables not dumped",
        ),
    ] {
        let runtime = load_ghost(temp.path(), pasta_toml);
        {
            let _log = pasta_lua::LoadDirGuard::new(temp.path().to_path_buf());
            assert_204(
                &request_at_level(&runtime, "OnPastaDumpVariables", &[], level),
                level,
            );
        }
        assert!(
            !temp.path().join(VARIABLES_TOML).exists(),
            "{level}: variables.toml must not be written"
        );
        // ランタイムと登録簿の両方が手放すとロガーが破棄され、ログがフラッシュされる。
        drop(runtime);
        pasta_lua::GlobalLoggerRegistry::instance().unregister(temp.path());
        let log =
            std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log")).unwrap();
        assert_eq!(
            log.lines().filter(|l| l.contains(warning)).count(),
            1,
            "{level}: one warning line {warning:?}: {log}"
        );
    }
}

/// scene-play-locals 1.1・1.3: Reference2 のインライン表の値を、再生するシーンのローカル変数として読める。
#[test]
fn on_pasta_scene_reads_reference2_locals() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["天気を言う", "", "{ '天気' = '雨' }"],
    ));
    assert!(value.contains("雨"), "Reference2 local is read: {value}");
}

/// 型を確かめる Reference2（文字列・整数・真偽値・日付）
const TYPED_LOCALS: &str = "{ '天気' = '雨', '回数' = 3, '晴れ' = true, '日' = 1979-05-27 }";

/// `needle` を含むフィクスチャの行の番号（1 始まり）を文字列で返す。
fn line_of(needle: &str) -> String {
    let at = FIXTURE
        .lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("fixture line {needle:?}"));
    (at + 1).to_string()
}

/// scene-play-locals 1.1・1.2・1.5・2.1〜2.4: 文字列・整数・真偽値・日付が、名前でも位置でも台詞と
/// Lua のブロックから読め、整数は算術に使える。
#[test]
fn reference2_locals_typed_values_by_name_and_position() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    let line = line_of("天気は＄天気");
    for (id, refs) in [
        ("OnPastaScene", ["型を言う", "", TYPED_LOCALS]),
        ("OnPastaSceneAt", [REL_PATH, line.as_str(), TYPED_LOCALS]),
    ] {
        let value = value_of_200(&request(&runtime, id, &refs));
        for expected in [
            "天気は雨、",
            "回数は3、",
            "晴れはtrue、",
            "日は1979-05-27」",
            "」4、",
            "boolean",
        ] {
            assert!(value.contains(expected), "{id}: {expected:?} in {value}");
        }
    }
}

/// scene-play-locals 1.3: `＞` の呼び先・ローカルシーン・チェイントークの続きでも同じ値が読める。
#[test]
fn reference2_locals_reach_call_local_scene_and_continuation() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["型を言う", "型のローカル", TYPED_LOCALS],
    ));
    assert!(
        value.contains("ローカルでは雨、") && value.contains("回数は3」"),
        "local scene: {value}"
    );

    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["呼ぶ元", "", TYPED_LOCALS],
    ));
    assert!(value.contains("呼ばれ先で雨"), "call target: {value}");

    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["続きで言う", "", TYPED_LOCALS],
    ));
    assert!(value.contains("一言目"), "beat 1: {value}");
    assert!(!value.contains("続きで"), "beat 2 not yet: {value}");
    runtime
        .exec(
            r#"
            local REG = require("pasta.shiori.event.register")
            REG.OnTestContinue = function() return require("pasta.store").co_scene end
            "#,
        )
        .unwrap();
    let value = value_of_200(&request(&runtime, "OnTestContinue", &[]));
    assert!(value.contains("続きで雨"), "continuation: {value}");
}

/// scene-play-locals 1.4・1.7: 本文の代入が勝ち、Reference2 は転記された `＄r0` に勝ち、
/// 次の Reference2 無しの再生には残らない。
#[test]
fn reference2_locals_precedence_and_no_carry_over() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());

    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["代入する", "", TYPED_LOCALS],
    ));
    assert!(
        value.contains("代入の後は10"),
        "body assignment wins: {value}"
    );

    // 比べる元: Reference2 が無ければ ＄r0 は Reference0
    let value = value_of_200(&request(&runtime, "OnPastaScene", &["参照を言う"]));
    assert!(
        value.contains("参照は参照を言う"),
        "transcribed r0: {value}"
    );
    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["参照を言う", "", "{ 'r0' = '上書き' }"],
    ));
    assert!(
        value.contains("参照は上書き"),
        "Reference2 wins over r0: {value}"
    );

    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["天気を言う", "", TYPED_LOCALS],
    ));
    assert!(value.contains("今日は雨"), "with Reference2: {value}");
    let value = value_of_200(&request(&runtime, "OnPastaScene", &["天気を言う"]));
    assert!(
        value.contains("【値の無い変数：＄天気】"),
        "next playback without Reference2: {value}"
    );
}

/// `@pasta_log` を、`warn` を `_G.__warns` に集める表に差し替える（ほかの口は元のものへ渡す）。
fn capture_warnings(runtime: &PastaLuaRuntime) {
    runtime
        .exec(
            r#"
            local orig = require("@pasta_log")
            _G.__warns = {}
            package.loaded["@pasta_log"] = setmetatable(
                { warn = function(m) table.insert(_G.__warns, m) end },
                { __index = orig })
            "#,
        )
        .unwrap();
}

/// 集めた警告を取り出して空にする。
fn take_warnings(runtime: &PastaLuaRuntime) -> Vec<String> {
    runtime
        .lua()
        .load("local w = _G.__warns; _G.__warns = {}; return w")
        .eval()
        .unwrap()
}

/// 失敗の表記だけの Value（末尾の `\e` を含む）であることを確かめる。
fn assert_failure_value(response: &str, notation: &str) {
    assert_eq!(
        value_of_200(response),
        format!("{notation}\\e"),
        "{response:?}"
    );
}

/// scene-play-locals 3.1〜3.4・1.6: Reference2 が無い・`""`・`{}` では今と同じ台詞で警告が無く、
/// Reference1 を `""` にすればグローバルシーンが鳴り、Reference1 の `{ … }` はローカルシーンの名前になる。
#[test]
fn reference2_absent_or_empty_keeps_current_behavior() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());
    capture_warnings(&runtime);

    let line = LINE_GLOBAL_BODY.to_string();
    for (id, r0, r1) in [
        ("OnPastaScene", "再生大域", ""),
        ("OnPastaSceneAt", REL_PATH, line.as_str()),
    ] {
        let base = value_of_200(&request(&runtime, id, &[r0, r1]));
        assert!(base.contains("大域の声"), "{id}: {base}");
        // フィクスチャのアクターは未登録なので、再生ごとの警告は Reference2 が無いときと比べる
        let base_warnings = take_warnings(&runtime);
        for r2 in ["", "{}"] {
            let value = value_of_200(&request(&runtime, id, &[r0, r1, r2]));
            assert_eq!(
                value, base,
                "{id} Reference2 {r2:?} must not change the talk"
            );
            assert_eq!(take_warnings(&runtime), base_warnings, "{id} {r2:?}");
        }
    }

    // 3.3: Reference1 を空にして Reference2 を付けるとグローバルシーン
    let value = value_of_200(&request(
        &runtime,
        "OnPastaScene",
        &["再生大域", "", "{ 'a' = 1 }"],
    ));
    assert!(value.contains("大域の声"), "global scene played: {value}");
    take_warnings(&runtime); // 未登録アクターの警告を捨てる

    // 3.4: Reference1 に表を書くと、その名前のローカルシーンを探す
    let response = request(&runtime, "OnPastaScene", &["再生親", "{ '天気' = '雨' }"]);
    assert_failure_value(
        &response,
        "【シーンが見つからない：OnPastaScene ＊再生親 ・{ '天気' = '雨' }】",
    );
    assert_eq!(
        take_warnings(&runtime),
        ["OnPastaScene: scene not found: global='再生親', local='{ '天気' = '雨' }'"]
    );
}

/// scene-play-locals 4.1〜4.5・4.8: 誤りの 3 種が、両方のイベントで表記と警告 1 行になり、
/// 何も再生せず場面の続きを変えない。複数の誤りでは並べ替えた最初のキーの 1 件だけ。
#[test]
fn reference2_errors_report_notation_and_one_warning() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());
    capture_warnings(&runtime);
    runtime
        .exec(
            r#"
            _G.__old_co = coroutine.create(function() coroutine.yield("古い続き") end)
            require("pasta.store").co_scene = _G.__old_co
            "#,
        )
        .unwrap();

    // (Reference2, 表記の <ラベル>：<E> の後ろ, 警告の <E>: の後ろ)
    let cases: &[(&str, &str, &str)] = &[
        (
            r#"{ 天気 = "雨" }"#,
            r#"表にできない値：{E} Reference2（"{ 天気 = "雨" }"）"#,
            r#"bad argument: Reference2='{ 天気 = "雨" }' (invalid unquoted key, expected letters, numbers, `-`, `_`)"#,
        ),
        (
            "3",
            r#"表にできない値：{E} Reference2（"3"）"#,
            "bad argument: Reference2='3' (not an inline table)",
        ),
        (
            r#"{ "a" = 1 } x"#,
            r#"表にできない値：{E} Reference2（"{ "a" = 1 } x"）"#,
            r#"bad argument: Reference2='{ "a" = 1 } x' (unexpected key or value, expected newline, `#`)"#,
        ),
        (
            "[]",
            r#"表にできない値：{E} Reference2（"[]"）"#,
            "bad argument: Reference2='[]' (not an inline table)",
        ),
        (
            "[1]",
            r#"表にできない値：{E} Reference2（"[1]"）"#,
            "bad argument: Reference2='[1]' (not an inline table)",
        ),
        (
            r#"{ "1" = 1 }"#,
            r#"名前にならない値：{E} Reference2（"1"）"#,
            "bad argument: Reference2 key='1' (not a local variable name)",
        ),
        (
            r#"{ "__x__" = 1 }"#,
            r#"名前にならない値：{E} Reference2（"__x__"）"#,
            "bad argument: Reference2 key='__x__' (not a local variable name)",
        ),
        (
            r#"{ "a b" = 1 }"#,
            r#"名前にならない値：{E} Reference2（"a b"）"#,
            "bad argument: Reference2 key='a b' (not a local variable name)",
        ),
        (
            r#"{ "a" = [1] }"#,
            "代入できない値：{E} Reference2 ＄a（表）",
            "bad argument: Reference2 key='a' (table is not assignable)",
        ),
        (
            r#"{ "a".b = 1 }"#,
            "代入できない値：{E} Reference2 ＄a（表）",
            "bad argument: Reference2 key='a' (table is not assignable)",
        ),
        // 4.5: "1" < "b" なので、"b" の型の誤りより先に "1" の名前の誤りだけが出る
        (
            r#"{ "b" = [1], "1" = 2 }"#,
            r#"名前にならない値：{E} Reference2（"1"）"#,
            "bad argument: Reference2 key='1' (not a local variable name)",
        ),
        // 4.8: 表記では `\` が `\\` になり、警告は届いたまま
        (
            r"{ 'C:\dic' = 1 }",
            r#"名前にならない値：{E} Reference2（"C:\\dic"）"#,
            r"bad argument: Reference2 key='C:\dic' (not a local variable name)",
        ),
    ];

    let line = LINE_GLOBAL_BODY.to_string();
    for (id, r0, r1) in [
        ("OnPastaScene", "再生大域", ""),
        ("OnPastaSceneAt", REL_PATH, line.as_str()),
    ] {
        for (r2, notation, warning) in cases {
            let response = request(&runtime, id, &[r0, r1, r2]);
            assert_failure_value(&response, &format!("【{}】", notation.replace("{E}", id)));
            assert_eq!(
                take_warnings(&runtime),
                [format!("{id}: {warning}")],
                "{id} {r2:?}: exactly one warning"
            );
        }
    }

    assert!(
        lua_bool(
            &runtime,
            r#"
            return require("pasta.store").co_scene == _G.__old_co
                and coroutine.status(_G.__old_co) == "suspended"
            "#,
        ),
        "continuation must remain after failures"
    );
}

/// scene-play-locals 4.6: Reference0 → Reference1 → Reference2 → シーンの検索 の順に確かめ、最初の誤りだけを出す。
#[test]
fn reference2_checked_after_reference0_and_1_before_lookup() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_debug_ghost(temp.path());
    capture_warnings(&runtime);

    // Reference2 の誤りはシーンの検索より先
    assert_failure_value(
        &request(&runtime, "OnPastaScene", &["無い名前", "", "3"]),
        r#"【表にできない値：OnPastaScene Reference2（"3"）】"#,
    );
    assert_eq!(
        take_warnings(&runtime),
        ["OnPastaScene: bad argument: Reference2='3' (not an inline table)"]
    );
    assert_failure_value(
        &request(
            &runtime,
            "OnPastaSceneAt",
            &["dic/test/無い.pasta", "1", "3"],
        ),
        r#"【表にできない値：OnPastaSceneAt Reference2（"3"）】"#,
    );
    assert_eq!(take_warnings(&runtime).len(), 1);

    // Reference0 の誤りは Reference2 より先
    let response = request(&runtime, "OnPastaScene", &["", "", "3"]);
    assert!(
        !value_of_200(&response).contains("Reference2"),
        "Reference0 error first: {response:?}"
    );
    let warnings = take_warnings(&runtime);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("Reference0"), "{warnings:?}");

    // Reference1（行）の誤りは Reference2 より先
    assert_failure_value(
        &request(&runtime, "OnPastaSceneAt", &[REL_PATH, "x", "3"]),
        r#"【数にできない値：OnPastaSceneAt Reference1（"x"）】"#,
    );
    assert_eq!(
        take_warnings(&runtime),
        ["OnPastaSceneAt: bad argument: Reference1='x'"]
    );
}

/// scene-play-locals 5.1・5.2: `external`・デバッグ無効では、Reference2 が誤っていても 204 と門の警告 1 行だけ。
#[test]
fn reference2_not_read_when_gate_refuses() {
    let enabled = tempfile::TempDir::new().unwrap();
    let disabled = tempfile::TempDir::new().unwrap();
    let line = LINE_GLOBAL_BODY.to_string();
    for (temp, pasta_toml, level, reason) in [
        (
            &enabled,
            "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\n",
            "external",
            "SecurityLevel external is refused",
        ),
        (
            &disabled,
            "[loader]\ndebug_mode = true\n",
            "local",
            "debug is disabled",
        ),
    ] {
        let runtime = load_ghost(temp.path(), pasta_toml);
        capture_warnings(&runtime);
        for (id, r0, r1) in [
            ("OnPastaScene", "再生大域", ""),
            ("OnPastaSceneAt", REL_PATH, line.as_str()),
        ] {
            assert_204(
                &request_at_level(&runtime, id, &[r0, r1, "[1]"], level),
                &format!("{id} {level}"),
            );
            assert_eq!(
                take_warnings(&runtime),
                [format!("{id}: {reason}; scene not played")],
                "{id} {level}: only the gate warning"
            );
        }
    }
}

/// scene-play-locals 4.7: `[logging] balloon_warning = false` では 204 で、警告は出る。
#[test]
fn reference2_error_with_balloon_warning_off_is_204_with_warning() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(
        temp.path(),
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\n\n[logging]\nballoon_warning = false\n",
    );
    capture_warnings(&runtime);
    let line = LINE_GLOBAL_BODY.to_string();
    for (id, r0, r1) in [
        ("OnPastaScene", "再生大域", ""),
        ("OnPastaSceneAt", REL_PATH, line.as_str()),
    ] {
        assert_204(&request(&runtime, id, &[r0, r1, "3"]), id);
        assert_eq!(
            take_warnings(&runtime),
            [format!(
                "{id}: bad argument: Reference2='3' (not an inline table)"
            )]
        );
    }
}
