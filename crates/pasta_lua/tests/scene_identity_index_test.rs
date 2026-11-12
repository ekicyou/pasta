//! task 2.2 統合テスト（pasta-scene-kick-from-cursor）。
//!
//! `.pasta` を transpile→ロード（debug 有効）後、finalize join が確定した
//! [`SceneIdentityIndex`]（`Arc<SourceMap>` の write-once スロット）が、既知のシーン
//! 宣言行から **ランタイム実 identity** な (scene_id, parent) を返すことを観測する。
//!
//! 観測対象（requirements 3.1/3.2/3.3/7.1）:
//! 1. グローバル本体領域の行 → `(OnTalk_1, None)`（global・parent なし）。
//! 2. 名前付き local 領域の行 → `(挨拶_1, Some(OnTalk_1))`（local・parent あり）。
//! 3. 同 base 2 本目のグローバル領域 → `(OnTalk_2, None)`（per-base 出現順突合）。
//! 4. 索引が返す identity が `collect_scenes`（runtime SSOT）の値に一致する。
//! 5. 通常モード（debug 無効）非破壊: 索引は構築されず、行マッピング双方向 resolve は不変。

use std::path::{Path, PathBuf};

use pasta_lua::debug::source_map::SceneIdentity;
use pasta_lua::mlua::Lua;
use pasta_lua::{PastaLoader, RuntimeConfig};

/// Neutralize ambient debug env vars before any test thread starts.
///
/// The developer session may export `PASTA_DEBUG=1`.
/// Without this guard, the debug-disabled `PastaLoader::load_with_config` here
/// would still enable the debug backend (making `debug_enabled()` true and
/// building a `SourceMap`), so the "normal mode" assertions fail. Tests must
/// behave identically regardless of the session environment, so we clear these
/// here. Each integration-test binary compiles separately, so this standalone
/// binary needs its own copy of the guard held in `tests/common/mod.rs`.
///
/// Running inside a `#[ctor]` (executed before `main`, while the process is still
/// single-threaded) makes the `remove_var` calls race-free under the Rust 2024
/// edition where `std::env::remove_var` is `unsafe`.
#[ctor::ctor(unsafe)]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
    }
}

/// 統合フィクスチャ（global「会話」×2 ＋ 名前付き local「挨拶」）。行番号は本ファイルの
/// アサーションが依存するため、フィクスチャ編集時は行も追従すること。
/// 既定の別名表（`OnTalk = ["会話"]`、scene-name-alias）により `＊会話` の登録名は `OnTalk_N`。
///  7: ＊会話        → global OnTalk_1（本体 __start__ 領域）
///  8: さくら：「おはよう」
///  9: ＞挨拶
/// 10:
/// 11: ・挨拶          → local 挨拶_1（parent OnTalk_1）
/// 12: さくら：「やあ」
/// 13:
/// 14: ＊会話         → global OnTalk_2（同 base 2 本目）
/// 15: さくら：「また会話だよ」
const FIXTURE: &str = include_str!("fixtures/scene_identity_index.pasta");

/// `.pasta` 行（本ファイル中で意味を固定）。
const LINE_GLOBAL1_BODY: u32 = 8; // OnTalk_1 本体（__start__ 領域）
const LINE_LOCAL_BODY: u32 = 12; // 挨拶_1 本体
const LINE_GLOBAL2_BODY: u32 = 15; // OnTalk_2 本体

fn ident(scene_id: &str, parent: Option<&str>) -> SceneIdentity {
    SceneIdentity {
        scene_id: scene_id.to_string(),
        parent: parent.map(|p| p.to_string()),
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest)?;
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// `[debug]` の有無をパラメータ化して base_dir を構築し、フィクスチャ `.pasta` の絶対パスを返す。
fn make_base_dir(base: &Path, debug_enabled: bool) -> PathBuf {
    let pasta_file = base.join("dic/test/scene_identity_index.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, FIXTURE).unwrap();

    let debug_section = if debug_enabled {
        "\n[debug]\nenabled = true\nport = 0\n"
    } else {
        ""
    };
    std::fs::write(
        base.join("pasta.toml"),
        format!("[loader]\ndebug_mode = true\n{debug_section}"),
    )
    .unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(sub);
        let dst = base.join(sub);
        if src.exists() {
            std::fs::create_dir_all(&dst).unwrap();
            copy_dir(&src, &dst).unwrap();
        }
    }
    pasta_file
}

/// 1/2/3/4: debug 有効でロード後、索引が既知行からランタイム実 identity を返し、
/// その値が `collect_scenes` の SSOT と一致する。
#[test]
fn finalize_join_resolves_runtime_identities_for_global_and_local() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path();
    let pasta_file = make_base_dir(base, true);
    let pasta_key = pasta_file.to_string_lossy().to_string();

    let runtime = PastaLoader::load_with_config(base, RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    assert!(
        runtime.debug_enabled(),
        "enabled [debug] must install the backend"
    );
    let source_map = runtime
        .debug_source_map()
        .expect("enabled debug runtime must hold the aggregated source map");

    // (1) グローバル本体領域 → (OnTalk_1, None)。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_GLOBAL1_BODY),
        Some(ident("OnTalk_1", None)),
        "global 本体行 {LINE_GLOBAL1_BODY} は (OnTalk_1, None) へ解決する"
    );

    // (2) 名前付き local 本体領域 → (挨拶_1, Some(OnTalk_1))。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_LOCAL_BODY),
        Some(ident("挨拶_1", Some("OnTalk_1"))),
        "local 本体行 {LINE_LOCAL_BODY} は (挨拶_1, Some(OnTalk_1)) へ解決する"
    );

    // (3) 同 base 2 本目のグローバル → (OnTalk_2, None)（per-base 出現順突合）。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_GLOBAL2_BODY),
        Some(ident("OnTalk_2", None)),
        "2 本目 global 本体行 {LINE_GLOBAL2_BODY} は (OnTalk_2, None) へ解決する"
    );

    // (4) 索引の identity が collect_scenes（runtime SSOT）に一致する。
    let scenes = pasta_lua::runtime::finalize::collect_scenes(runtime.lua())
        .expect("collect_scenes must succeed");
    // global OnTalk_1 / OnTalk_2 が存在する。
    assert!(
        scenes.iter().any(|(g, _)| g == "OnTalk_1"),
        "runtime に OnTalk_1 が存在する: {scenes:?}"
    );
    assert!(
        scenes.iter().any(|(g, _)| g == "OnTalk_2"),
        "runtime に OnTalk_2 が存在する: {scenes:?}"
    );
    // local (OnTalk_1, 挨拶_1) が存在し、索引の (挨拶_1, OnTalk_1) と一致する。
    assert!(
        scenes.iter().any(|(g, l)| g == "OnTalk_1" && l == "挨拶_1"),
        "runtime に (OnTalk_1, 挨拶_1) が存在する: {scenes:?}"
    );
}

/// 5: 通常モード（debug 無効）では、シーンマップも索引も作らない。
#[test]
fn normal_mode_builds_no_index() {
    // debug 無効でロード: source map 自体が構築されない（None）。
    let temp_off = tempfile::TempDir::new().expect("temp dir");
    let base_off = temp_off.path();
    let _pasta_off = make_base_dir(base_off, false);

    let runtime_off = PastaLoader::load_with_config(base_off, RuntimeConfig::new())
        .expect("debug-disabled runtime must load");
    assert!(
        !runtime_off.debug_enabled(),
        "no [debug] section → backend disabled (zero-cost)"
    );
    assert!(
        runtime_off.debug_source_map().is_none(),
        "debug 無効では SourceMap を構築・保持しない（索引も無い・7.1）"
    );
}

/// scene-play-event task 2.3（requirements 2.1/4.1/4.2）: `@pasta_debug` を Lua から呼ぶ。
/// 有効なら `enabled()` が真で、`scene_at` は相対・絶対パスとも登録名の組を返し、
/// 行が 1..=u32::MAX の整数でなければエラーにせず nil。無効なら偽で、マップが無いので常に nil。
#[test]
fn pasta_debug_module_reports_enabled_and_resolves_positions() {
    const PROBE: &str = r#"
        local file = ...
        local D = require("@pasta_debug")
        local function pair(f, l)
            local g, s = D.scene_at(f, l)
            return tostring(g) .. "/" .. tostring(s)
        end
        return table.concat({
            tostring(D.enabled()),
            pair("dic/test/scene_identity_index.pasta", 8),
            pair("dic\\test\\scene_identity_index.pasta", 12),
            pair(file, 15),
            pair(file, 0),
            pair(file, 1.5),
            pair(file, 4294967296),
            pair(file, "8"),
        }, "|")
    "#;
    let temp = tempfile::TempDir::new().expect("temp dir");
    let pasta_file = make_base_dir(temp.path(), true);
    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let got: String = runtime
        .lua()
        .load(PROBE)
        .call(pasta_file.to_string_lossy().to_string())
        .expect("@pasta_debug must be callable");
    assert_eq!(
        got,
        "true|OnTalk_1/__start__|OnTalk_1/挨拶_1|OnTalk_2/__start__|nil/nil|nil/nil|nil/nil|nil/nil"
    );

    let temp_off = tempfile::TempDir::new().expect("temp dir");
    let off_file = make_base_dir(temp_off.path(), false);
    let runtime_off = PastaLoader::load_with_config(temp_off.path(), RuntimeConfig::new())
        .expect("debug-disabled runtime must load");
    let got_off: String = runtime_off
        .lua()
        .load(PROBE)
        .call(off_file.to_string_lossy().to_string())
        .expect("@pasta_debug must be callable when disabled");
    assert_eq!(
        got_off,
        "false|nil/nil|nil/nil|nil/nil|nil/nil|nil/nil|nil/nil|nil/nil"
    );
}

/// scene-play-locals task 1.2（requirements 4.2）: `@pasta_debug.is_local_var_name` は
/// `＄名前` のローカル変数名の決まりに当たるかを返す。
#[test]
fn pasta_debug_is_local_var_name_checks_id_rule() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    make_base_dir(temp.path(), false);
    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("runtime must load");
    let got: String = runtime
        .lua()
        .load(
            r#"local D = require("@pasta_debug")
            return tostring(D.is_local_var_name("天気")) .. "|" .. tostring(D.is_local_var_name("1"))"#,
        )
        .eval()
        .expect("is_local_var_name must be callable");
    assert_eq!(got, "true|false");
}

// ===========================================================================
// task 6.1 統合/回帰テスト（pasta-scene-kick-from-cursor・requirements 3.1/3.3/3.4/7.1）。
//
// 単一の **実ロード済みランタイム** 上で次のループを閉じる:
//   位置 → `@pasta_debug.scene_at`（`resolve_position`。登録名の組 `(大域, ローカル)`）
//        → シーン表（`STORE.scenes`）からの完全一致の引き当て（`OnPastaSceneAt` と同手順）
//        → 着地した runtime (global_name, local_name) が `collect_scenes`（SSOT）に一致。
// これにより「位置の引きの組 == runtime 実シーン」を実機で突合する（3.1/3.3）。
// 加えて debug 有効・索引充填済みの同一ランタイムで行マッピング双方向 resolve の
// ラウンドトリップ不変を固定する（3.4 後方非破壊）。
// ===========================================================================

/// `@pasta_debug.scene_at(file, line)` で位置を登録名の組に引く（見つからなければ `None`）。
fn scene_at_pair(lua: &Lua, file: &str, line: u32) -> Option<(String, String)> {
    let (g, l): (Option<String>, Option<String>) = lua
        .load(r#"local f, l = ... return require("@pasta_debug").scene_at(f, l)"#)
        .call((file, line))
        .expect("@pasta_debug.scene_at must not error");
    g.zip(l)
}

/// シーン表に `(global, local)` の関数があるか（`OnPastaSceneAt` の引き当てと同手順）。
fn scene_fn_exists(lua: &Lua, global: &str, local: &str) -> bool {
    lua.load(
        r#"
        local g, l = ...
        local SCENE = require("pasta.scene")
        local fn
        if l == "__start__" then fn = SCENE.get_start(g) else fn = SCENE.get(g, l) end
        return type(fn) == "function"
    "#,
    )
    .call((global, local))
    .expect("scene table lookup must not error")
}

/// 6.1-(1) 位置の引きの組が runtime 実シーンに一致する（requirements 3.1/3.3）。
///
/// 既知の global / local 宣言領域の行を `scene_at`（`resolve_position`）で組に引き
/// （global → `(OnTalk_1, __start__)`、local → `(OnTalk_1, 挨拶_1)`）、シーン表から
/// 完全一致で引けて `collect_scenes`（SSOT）に在ることを END-TO-END で固定する。
/// 位置の引きと実シーンが乖離すれば（local が `__start__` へ潰れる・親の取り違え）落ちる。
#[test]
fn resolved_position_pair_matches_runtime_scene() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path();
    let pasta_file = make_base_dir(base, true);
    let pasta_key = pasta_file.to_string_lossy().to_string();

    let runtime = PastaLoader::load_with_config(base, RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let lua = runtime.lua();
    let scenes =
        pasta_lua::runtime::finalize::collect_scenes(lua).expect("collect_scenes must succeed");

    let global = scene_at_pair(lua, &pasta_key, LINE_GLOBAL1_BODY).expect("global 本体行は引ける");
    let local = scene_at_pair(lua, &pasta_key, LINE_LOCAL_BODY).expect("local 本体行は引ける");
    assert_eq!(global, ("OnTalk_1".to_string(), "__start__".to_string()));
    assert_eq!(local, ("OnTalk_1".to_string(), "挨拶_1".to_string()));

    for (g, l) in [&global, &local] {
        assert!(
            scene_fn_exists(lua, g, l),
            "({g}, {l}) はシーン表から引ける"
        );
        assert!(
            scenes.iter().any(|(sg, sl)| sg == g && sl == l),
            "({g}, {l}) は collect_scenes(SSOT) に存在する: {scenes:?}"
        );
    }
}

// ===========================================================================
// task 6.x sanitize 衝突 characterization テスト（pasta-scene-kick-from-cursor）。
//
// `SceneRegistry::increment_counter` は per-name カウンタを **生の** scene 名で採番する
// 一方、ランタイム（pasta/scene.lua `get_or_increment_counter`）は **sanitize 済み** base
// 名で採番する（create_scene が base_name = sanitize_name(scene.name) で呼ばれるため）。
//
// `sanitize_name` で同一 base へ写像されるが distinct な 2 つの生名（例: `会話·A` の
// U+00B7 MIDDLE DOT と `会話_A` のリテラル `_`、どちらも sanitize で `会話_A`）を 2 本
// 並べると:
//   - build 時: increment_counter は生名で別カウンタ → 双方 #1 → join_key は両方
//     `G:会話_A#1` に衝突する。
//   - runtime: sanitize base で採番 → `会話_A_1` / `会話_A_2`。
// 結果、2 本目の cursor 領域が 1 本目（会話_A_1）へ MIS-RESOLVE する（別のシーンを再生してしまう）。
//
// 本テストは「同 base へ衝突する 2 本の global が、それぞれ distinct な runtime identity
// （会話_A_1 / 会話_A_2）へ解決する」ことを固定する。fix 前は RED（2 本目が 会話_A_1 へ
// mis-resolve、または collect_scenes と不一致）になる。
// ===========================================================================

/// sanitize 衝突フィクスチャ（global「会話·A」「会話_A」＝ sanitize 後はどちらも「会話_A」）。
/// 行番号は本ファイルのアサーションが依存するため、フィクスチャ編集時は追従すること。
///  8: ＊会話·A        → global（sanitize base 採番なら 会話_A_1）
///  9: さくら：「おはよう A1」  ← 本体行
/// 10:
/// 11: ＊会話_A        → global（sanitize base 採番なら 会話_A_2）
/// 12: さくら：「おはよう A2」  ← 本体行（衝突 2 本目）
const COLLISION_FIXTURE: &str = include_str!("fixtures/scene_identity_collision.pasta");

const LINE_COLLISION_SCENE1_BODY: u32 = 9; // 会話_A_1 本体
const LINE_COLLISION_SCENE2_BODY: u32 = 12; // 会話_A_2 本体（衝突 2 本目）

/// 衝突フィクスチャ用の base_dir 構築（`make_base_dir` と同形だが別 `.pasta` を書く）。
fn make_collision_base_dir(base: &Path) -> PathBuf {
    let pasta_file = base.join("dic/test/scene_identity_collision.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, COLLISION_FIXTURE).unwrap();

    std::fs::write(
        base.join("pasta.toml"),
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\nport = 0\n",
    )
    .unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(sub);
        let dst = base.join(sub);
        if src.exists() {
            std::fs::create_dir_all(&dst).unwrap();
            copy_dir(&src, &dst).unwrap();
        }
    }
    pasta_file
}

/// 6.x: sanitize 衝突する 2 本の global が distinct な runtime identity へ解決する。
///
/// 生名 `会話·A`（U+00B7）と `会話_A`（`_`）は `sanitize_name` で両方 `会話_A`。
/// build 側カウンタが生名キー（衝突 fix 前）だと双方 #1 → 2 本目が 会話_A_1 へ
/// mis-resolve する。sanitize キー（fix 後）だと 会話_A_1 / 会話_A_2 と distinct。
#[test]
fn sanitize_colliding_globals_resolve_to_distinct_runtime_identities() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path();
    let pasta_file = make_collision_base_dir(base);
    let pasta_key = pasta_file.to_string_lossy().to_string();

    let runtime = PastaLoader::load_with_config(base, RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let source_map = runtime
        .debug_source_map()
        .expect("enabled debug runtime holds the aggregated source map");

    // runtime SSOT: 会話_A_1 / 会話_A_2 が distinct に存在する。
    let scenes = pasta_lua::runtime::finalize::collect_scenes(runtime.lua())
        .expect("collect_scenes must succeed");
    assert!(
        scenes.iter().any(|(g, _)| g == "会話_A_1"),
        "runtime に 会話_A_1 が存在する: {scenes:?}"
    );
    assert!(
        scenes.iter().any(|(g, _)| g == "会話_A_2"),
        "sanitize base 採番で 2 本目は 会話_A_2 になる（distinct）: {scenes:?}"
    );

    // 1 本目本体行 → 会話_A_1。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_COLLISION_SCENE1_BODY),
        Some(ident("会話_A_1", None)),
        "1 本目（会話·A）本体行 {LINE_COLLISION_SCENE1_BODY} は 会話_A_1 へ解決する"
    );

    // 2 本目本体行 → 会話_A_2（衝突 fix の core アサーション）。
    // fix 前: build join_key が生名キーで双方 G:会話_A#1 に衝突 → 2 本目が 会話_A_1 へ
    // mis-resolve（または index 未充填で None）になり、この assert が落ちる。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_COLLISION_SCENE2_BODY),
        Some(ident("会話_A_2", None)),
        "2 本目（会話_A）本体行 {LINE_COLLISION_SCENE2_BODY} は 会話_A_2 へ解決する \
         （fix 前は 会話_A_1 へ mis-resolve）"
    );

    // 2 本は別 runtime identity へ解決している（潰れていない）。
    assert_ne!(
        source_map.scene_at(&pasta_key, LINE_COLLISION_SCENE1_BODY),
        source_map.scene_at(&pasta_key, LINE_COLLISION_SCENE2_BODY),
        "sanitize 衝突 2 本は別シーンへ解決する（別のシーンを再生する誤りの回帰防止）"
    );
}

// ===========================================================================
// scene-identity-format 統合テスト（requirements 4.1/4.2/4.5/8.3）。
//
// 登録名が `{名前}_{通し番号}` になったことで、末尾が数字のシーン名（`＊章11`）が
// 同名シーンの 11 本目（`章_11`）と区別され、デバッガの索引は定義元の `.pasta`
// ファイルごとに突き合わせる。索引の identity はシーン表から完全一致で引ける。
// ===========================================================================

/// 登録名・名前の形のフィクスチャ（runtime の scene_identity_format_test と共用）。
/// 行番号は本ファイルのアサーションが依存するため、フィクスチャ編集時は追従すること。
const FORMAT_FIXTURE: &str = include_str!("fixtures/scene_identity_format.pasta");

/// `.pasta` ファイル群（`dic/test/` からの相対パス, 内容）を書き、debug 有効の base_dir を作る。
/// 書いた `.pasta` の絶対パスを同じ順で返す。
fn make_debug_base_dir(base: &Path, files: &[(&str, &str)]) -> Vec<PathBuf> {
    std::fs::write(
        base.join("pasta.toml"),
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\nport = 0\n",
    )
    .unwrap();
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(sub);
        let dst = base.join(sub);
        if src.exists() {
            std::fs::create_dir_all(&dst).unwrap();
            copy_dir(&src, &dst).unwrap();
        }
    }
    files
        .iter()
        .map(|(name, content)| {
            let path = base.join("dic/test").join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, content).unwrap();
            path
        })
        .collect()
}

/// identity のシーンをシーン表から完全一致で引き（global → `SCENE.get_start`、
/// local → `SCENE.get(parent, scene_id)`）、最小の act で実行してトーク本文をつなげて返す。
fn run_identity(lua: &Lua, id: &SceneIdentity) -> String {
    let (global, local) = match &id.parent {
        Some(p) => (p.as_str(), id.scene_id.as_str()),
        None => (id.scene_id.as_str(), "__start__"),
    };
    lua.load(
        r#"
        local g, l = ...
        local SCENE = require("pasta.scene")
        local fn
        if l == "__start__" then fn = SCENE.get_start(g) else fn = SCENE.get(g, l) end
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
    .expect("identity must resolve to a runnable scene")
}

/// 4.1/4.2/8.3: 末尾が数字・`_` と数字で終わる・記号を含むシーン名で、カーソル行の identity が
/// 登録名どおりになり、その identity でシーン表から引いたシーンがその行のシーンである。
/// `＊章11` の identity は `章11_1` で、同名シーン `＊章` の 11 本目（`章_11`）にならない。
#[test]
fn identities_of_digit_and_symbol_names_resolve_in_scene_table() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let paths = make_debug_base_dir(
        temp.path(),
        &[("scene_identity_format.pasta", FORMAT_FIXTURE)],
    );
    let key = paths[0].to_string_lossy().to_string();
    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let map = runtime
        .debug_source_map()
        .expect("enabled runtime holds map");

    // (本体行, 期待 identity, その行のトーク本文)
    let cases = [
        (4, ident("A1_1", None), "A1本体"),
        (37, ident("A_11", None), "A本体11"),
        (70, ident("章_11", None), "章本体11"),
        (73, ident("章11_1", None), "章11本体"),
        (76, ident("章_2_1", None), "章_2本体"),
        (80, ident("返事2_1", Some("章_2_1")), "返事2本体"),
        (83, ident("会話_朝_1", None), "会話・朝本体"),
        (86, ident("挨拶_1", Some("会話_朝_1")), "挨拶本体1"),
    ];
    for (line, expected, text) in cases {
        let id = map.scene_at(&key, line);
        assert_eq!(id.as_ref(), Some(&expected), "行 {line} の identity");
        assert!(
            run_identity(runtime.lua(), &expected).contains(text),
            "identity {expected:?} はシーン表から行 {line} のシーンを引く"
        );
    }
}

/// 2 ファイル用: 同名のグローバル「会話」とその中のローカル「挨拶」。
/// 行 2: グローバル本体 / 行 6: ローカル本体。
const TWO_FILE_A: &str =
    "＊会話\n　さくら：「Aの会話」\n　＞挨拶\n\n　・挨拶\n　　さくら：「Aの挨拶」\n";
const TWO_FILE_B: &str =
    "＊会話\n　さくら：「Bの会話」\n　＞挨拶\n\n　・挨拶\n　　さくら：「Bの挨拶」\n";

/// 4.5: 2 つの `.pasta` に同名のグローバル（とその中のローカル）があるとき、2 つ目の
/// ファイルの行から得た identity は 2 つ目のファイルのシーンを指す（読み込み順に依らず、
/// 各ファイルの行の identity がそのファイルのシーンを引く）。
#[test]
fn same_named_scenes_in_two_files_resolve_to_their_own_file() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let paths = make_debug_base_dir(
        temp.path(),
        &[("a.pasta", TWO_FILE_A), ("b.pasta", TWO_FILE_B)],
    );
    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let map = runtime
        .debug_source_map()
        .expect("enabled runtime holds map");

    let mut seen = Vec::new();
    for (path, tag) in paths.iter().zip(["A", "B"]) {
        let key = path.to_string_lossy().to_string();
        let global = map
            .scene_at(&key, 2)
            .expect("global 本体行は identity を持つ");
        let local = map
            .scene_at(&key, 6)
            .expect("local 本体行は identity を持つ");
        assert_eq!(local.parent.as_deref(), Some(global.scene_id.as_str()));
        // 開始シーンは `＞挨拶` で自分のローカルも呼ぶので、本文の先頭で見分ける。
        assert!(run_identity(runtime.lua(), &global).starts_with(&format!("「{tag}の会話」")));
        assert_eq!(
            run_identity(runtime.lua(), &local),
            format!("「{tag}の挨拶」")
        );
        seen.push(global);
    }
    assert_ne!(
        seen[0], seen[1],
        "2 ファイルの同名シーンは別の identity になる"
    );
}

// ===========================================================================
// scene-name-alias task 6.1（requirements 7.1/7.2/7.3）。
//
// 同じファイルに別名 `＊会話` と本名 `＊OnTalk` が混在しても、既定の別名表
// （`OnTalk = ["会話"]`）のもとで各宣言は同じ base `OnTalk` の出現順で採番され
// （`OnTalk_1`〜`OnTalk_3`）、各行の identity・位置の引きの組・シーン表から引くシーンが
// その行の宣言のシーンになる。
// ===========================================================================

/// 別名と本名の混在フィクスチャ。行番号は下のケース表が依存する。
///  2: ＊会話   → OnTalk_1（3: 本体 / 6: ・挨拶 → 7: 本体）
///  9: ＊OnTalk → OnTalk_2（10: 本体）
/// 12: ＊会話   → OnTalk_3（13: 本体）
const ALIAS_MIXED_FIXTURE: &str = include_str!("fixtures/scene_alias_mixed.pasta");

/// 7.1/7.2/7.3: 各宣言行・本体行が宣言どおりの identity に対応づき、その位置の引きの組で
/// シーン表から引いたシーンが同名の他候補ではなくその行のシーンである。
#[test]
fn aliased_and_real_name_scenes_in_one_file_map_to_their_own_identities() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let paths = make_debug_base_dir(
        temp.path(),
        &[("scene_alias_mixed.pasta", ALIAS_MIXED_FIXTURE)],
    );
    let key = paths[0].to_string_lossy().to_string();
    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let map = runtime
        .debug_source_map()
        .expect("enabled runtime holds map");
    let lua = runtime.lua();

    // (行, 期待 identity, 期待の位置の引きの組, その行のシーンだけが持つ台詞)
    let cases = [
        (
            2,
            ident("OnTalk_1", None),
            ("OnTalk_1", "__start__"),
            "「会話その1」",
        ),
        (
            3,
            ident("OnTalk_1", None),
            ("OnTalk_1", "__start__"),
            "「会話その1」",
        ),
        (
            7,
            ident("挨拶_1", Some("OnTalk_1")),
            ("OnTalk_1", "挨拶_1"),
            "「会話1の挨拶」",
        ),
        (
            9,
            ident("OnTalk_2", None),
            ("OnTalk_2", "__start__"),
            "「本名のOnTalk」",
        ),
        (
            10,
            ident("OnTalk_2", None),
            ("OnTalk_2", "__start__"),
            "「本名のOnTalk」",
        ),
        (
            12,
            ident("OnTalk_3", None),
            ("OnTalk_3", "__start__"),
            "「会話その3」",
        ),
        (
            13,
            ident("OnTalk_3", None),
            ("OnTalk_3", "__start__"),
            "「会話その3」",
        ),
    ];
    for (line, expected, (g, l), text) in cases {
        let id = map.scene_at(&key, line);
        assert_eq!(id.as_ref(), Some(&expected), "行 {line} の identity（7.3）");
        assert_eq!(
            scene_at_pair(lua, &key, line),
            Some((g.to_string(), l.to_string())),
            "行 {line} の位置の引きの組"
        );
        assert!(
            scene_fn_exists(lua, g, l),
            "({g}, {l}) はシーン表から完全一致で引ける"
        );
        let played = run_identity(lua, &expected);
        assert!(
            played.starts_with(text),
            "行 {line} の再生は宣言のシーン（{text}）を指す（7.2）: {played}"
        );
    }
}
