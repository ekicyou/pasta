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

/// Neutralize ambient DAP debug env vars before any test thread starts.
///
/// The developer session may export `PASTA_DEBUG=1` / `PASTA_DEBUG_PORT=9276`.
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
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
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

/// 5: 通常モード（debug 無効）非破壊。索引は構築されず（`scene_at` は常に None）、
/// 行マッピング双方向 resolve は debug 有効時と同一（挙動不変）。
#[test]
fn normal_mode_builds_no_index_and_line_mapping_is_unchanged() {
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

    // debug 有効側で同一フィクスチャの行マッピングを取得し、無効でも不変であることを
    // 「無効では map が無い ＝ 旧来の .lua 実行に一切干渉しない」ことで担保する。
    // さらに有効側の双方向 resolve が正しく機能する（索引追加が既存マッピングを壊さない）。
    let temp_on = tempfile::TempDir::new().expect("temp dir");
    let base_on = temp_on.path();
    let pasta_on = make_base_dir(base_on, true);
    let pasta_on_key = pasta_on.to_string_lossy().to_string();
    let runtime_on = PastaLoader::load_with_config(base_on, RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let map_on = runtime_on
        .debug_source_map()
        .expect("enabled runtime holds map");

    // グローバル宣言行（行 7）は生成 .lua の create_scene 行へ対応する（既存行マッピング）。
    let global_decl_lua = map_on.resolve_pasta_to_lua(&pasta_on_key, 7);
    assert!(
        !global_decl_lua.is_empty(),
        "索引追加後も既存の `.pasta`→`.lua` 逆引きは機能する（行マッピング不変・7.1）: {global_decl_lua:?}"
    );
    // 逆方向も整合: その .lua 行を .pasta へ戻すと元の .pasta ファイルを指す。
    let (chunk, lua_line) = global_decl_lua[0].clone();
    let back = map_on.resolve_lua_to_pasta(&chunk, lua_line);
    assert!(
        back.is_some(),
        "索引追加後も `.lua`→`.pasta` 前方解決は機能する（双方向不変）"
    );
}

// ===========================================================================
// task 6.1 統合/回帰テスト（pasta-scene-kick-from-cursor・requirements 3.1/3.3/3.4/7.1）。
//
// task 2.2（`scene_identity_index_test::finalize_join_resolves_*`）は「位置 → 索引
// identity」かつ「索引 identity == `collect_scenes`」を固定する。task 3.1 の
// `playscene_tests` は **合成** SourceMap で `resolve_and_kick` の composite-string 組立を、
// `kick_local_composite_test.lua` は Lua 単体で try_dispatch の
// 分岐を固定する。いずれも「索引 identity が、実ランタイム上の kick の引き当ての
// 解決対象に一致する」ことは END-TO-END では固定していない。
//
// 本 6.1 は単一の **実ロード済みランタイム** 上で次のループを閉じる:
//   位置 → `SourceMap::scene_at`（索引 identity）
//        → kick 取次が組むキック名（global: `OnTalk_1` / local: `:OnTalk_1:挨拶_1`）
//        → シーン表（`STORE.scenes`）からの完全一致の引き当て（= kick.lua try_dispatch と同手順）
//        → 着地した runtime (global_name, local_name) が `collect_scenes`（SSOT）に一致。
// これにより「索引 identity == kick の解決対象 == runtime 実シーン」を実機で
// 突合する（3.1/3.3）。加えて debug 有効・索引充填済みの同一ランタイムで行マッピング
// 双方向 resolve のラウンドトリップ不変を固定する（3.4 後方非破壊）。
// ===========================================================================

/// kick 取次（`debug::playscene::build_kick_scene`）と **同一契約**で確定 identity から
/// キック名を組む（global → `scene_id`、local → `:{parent}:{scene_id}`）。
///
/// 本関数は production の private ヘルパと同じ規則をテスト側で再現し、`scene_at` が返す
/// identity が kick 経路に渡されるキック名へどう写像されるかを固定する（playscene.rs
/// `build_kick_scene` のドキュメント契約）。組んだキック名を [`kick_lookup_runtime`]
/// がシーン表から完全一致で引き、ループを閉じる。
fn kick_scene_key(id: &SceneIdentity) -> String {
    match &id.parent {
        Some(p) => format!(":{p}:{}", id.scene_id),
        None => id.scene_id.clone(),
    }
}

/// キック名を **実ロード済みランタイム** のシーン表から完全一致で引き当て、着地した
/// runtime シーン `(global_name, local_name)` を返す（kick.lua try_dispatch と同手順）。
///
/// - `:parent:local` 形式（local-composite）: kick.lua と同じく `^:([^:]+):(.+)$` で
///   parent/local_name に分解し、`SCENE.get(parent, local_name)` で引く。
/// - 先頭 `:` 無し（global）: `SCENE.get_start(name)` で開始関数を引く。
///
/// 前方一致の検索（`SCENE.search`）は通らない。解決不能なら `None`
/// （= kick が unresolved drop する位置）。
fn kick_lookup_runtime(lua: &Lua, kick_name: &str) -> Option<(String, String)> {
    let (global, local) = match kick_name
        .strip_prefix(':')
        .and_then(|rest| rest.split_once(':'))
    {
        Some((p, l)) => (p.to_string(), l.to_string()),
        None => (kick_name.to_string(), "__start__".to_string()),
    };
    let found: bool = lua
        .load(
            r#"
            local g, l = ...
            local SCENE = require("pasta.scene")
            local fn
            if l == "__start__" then fn = SCENE.get_start(g) else fn = SCENE.get(g, l) end
            return type(fn) == "function"
        "#,
        )
        .call((global.as_str(), local.as_str()))
        .expect("scene table lookup must not error");
    found.then_some((global, local))
}

/// 6.1-(1) kick の引き当ての解決対象一致（requirements 3.1/3.3）。
///
/// 実ロード済みランタイム上で、既知の global / local 宣言領域の行から `scene_at` で
/// identity を確定し、その identity をキック名へ写像し（global → `OnTalk_1`、
/// local → `:OnTalk_1:挨拶_1`）、シーン表から完全一致で引いた着地シーンが `collect_scenes`
/// （runtime SSOT）に存在することを END-TO-END で固定する。
///
/// 真正性（RED の論拠）: もし索引 identity と kick の引き当てが乖離すれば
/// （例: local が global の `__start__` へ潰れる／parent 取り違え／scene_id 不一致）、
/// 完全一致の引き当ては失敗するか `collect_scenes` の対応エントリと食い違い、本テストの
/// `assert` は落ちる。
#[test]
fn index_identity_matches_runtime_kick_lookup_target() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path();
    let pasta_file = make_base_dir(base, true);
    let pasta_key = pasta_file.to_string_lossy().to_string();

    let runtime = PastaLoader::load_with_config(base, RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let source_map = runtime
        .debug_source_map()
        .expect("enabled debug runtime holds the aggregated source map");
    let lua = runtime.lua();

    // runtime SSOT。
    let scenes =
        pasta_lua::runtime::finalize::collect_scenes(lua).expect("collect_scenes must succeed");

    // --- global: 本体領域行 → identity (OnTalk_1, None) → キック名 "OnTalk_1" ---
    let global_id = source_map
        .scene_at(&pasta_key, LINE_GLOBAL1_BODY)
        .expect("global 本体行は identity へ解決する");
    assert_eq!(
        global_id,
        ident("OnTalk_1", None),
        "scene_at は global identity を返す"
    );
    let global_key = kick_scene_key(&global_id);
    assert_eq!(global_key, "OnTalk_1", "global の キック名は素の scene_id");

    let (g_global, g_local) =
        kick_lookup_runtime(lua, &global_key).expect("global キック名は runtime シーンへ着地する");
    // global kick は __start__（無名 start シーン）へ着地するのが kick 決定ノートの契約。
    assert_eq!(
        g_global, "OnTalk_1",
        "global kick の引き当ては索引 identity と同じ global へ着地する"
    );
    assert!(
        scenes.iter().any(|(g, l)| *g == g_global && *l == g_local),
        "global kick の着地 ({g_global}, {g_local}) は collect_scenes(SSOT) に存在する: {scenes:?}"
    );
    // 着地 global は索引 identity の scene_id と一致（kick 解決対象 == 索引 identity）。
    assert_eq!(
        g_global, global_id.scene_id,
        "kick の引き当ての解決対象 global == 索引 identity の scene_id"
    );

    // --- local: 名前付き local 本体行 → identity (挨拶_1, Some(OnTalk_1)) → ":OnTalk_1:挨拶_1" ---
    let local_id = source_map
        .scene_at(&pasta_key, LINE_LOCAL_BODY)
        .expect("local 本体行は identity へ解決する");
    assert_eq!(
        local_id,
        ident("挨拶_1", Some("OnTalk_1")),
        "scene_at は local identity を返す"
    );
    let local_key = kick_scene_key(&local_id);
    assert_eq!(
        local_key, ":OnTalk_1:挨拶_1",
        "local の キック名は composite ':parent:local'"
    );

    let (l_global, l_local) = kick_lookup_runtime(lua, &local_key)
        .expect("local-composite キック名は runtime シーンへ着地する");
    // local 分岐は __start__ へ潰さず local 同一性を保持する（kick 決定ノート）。
    assert_eq!(
        (l_global.as_str(), l_local.as_str()),
        ("OnTalk_1", "挨拶_1"),
        "local kick の引き当ては索引 identity と同じ (OnTalk_1, 挨拶_1) へ着地する"
    );
    assert!(
        scenes.iter().any(|(g, l)| *g == l_global && *l == l_local),
        "local kick の着地 ({l_global}, {l_local}) は collect_scenes(SSOT) に存在する: {scenes:?}"
    );
    // 着地 (global, local) は索引 identity の (parent, scene_id) と一致。
    assert_eq!(
        l_global,
        local_id.parent.clone().expect("local は parent を持つ"),
        "kick の引き当ての解決対象 parent == 索引 identity の parent"
    );
    assert_eq!(
        l_local, local_id.scene_id,
        "kick の引き当ての解決対象 local == 索引 identity の scene_id"
    );

    // 念のため: global と local は別シーンへ解決している（潰れていない）。
    assert_ne!(
        (g_global.as_str(), g_local.as_str()),
        (l_global.as_str(), l_local.as_str()),
        "global kick(__start__) と local kick(挨拶_1) は別シーンへ解決する"
    );
}

/// 6.1-(2) 行マッピング後方非破壊の回帰（requirements 3.4）。
///
/// **debug 有効・索引充填済み** の同一ランタイムで、既知の `.pasta` 宣言行について
/// `resolve_pasta_to_lua`（逆引き）→ `resolve_lua_to_pasta`（前方）のラウンドトリップが
/// 元の `.pasta` 位置へ戻ることを固定する。索引（`scene_at` が `Some` を返す）が
/// **同居** しても行マッピング双方向 resolve が一切変質しないことを表明する。
///
/// 2.2 の `normal_mode_*` は debug **無効**側（map 自体が無い）で非干渉を示すのに対し、
/// 本テストは索引が **存在する** 状態での行マッピング不変を直接固定する点で補完関係。
///
/// 真正性（RED の論拠）: もし索引追加が `forward`/`reverse` を汚染（行ズレ・キー喪失）
/// すれば、ラウンドトリップは元行へ戻らず `assert_eq!` が落ちる。
#[test]
fn line_mapping_roundtrip_unchanged_with_index_present() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path();
    let pasta_file = make_base_dir(base, true);
    let pasta_key = pasta_file.to_string_lossy().to_string();

    let runtime = PastaLoader::load_with_config(base, RuntimeConfig::new())
        .expect("debug-enabled runtime must load");
    let map = runtime
        .debug_source_map()
        .expect("enabled runtime holds map");

    // 前提: 索引は **充填済み**（同居している）。
    assert!(
        map.scene_at(&pasta_key, LINE_GLOBAL1_BODY).is_some(),
        "本テストは索引が存在する状態での行マッピング不変を固定する（前提確認）"
    );

    // 既知の `.pasta` 宣言行群（global #1 宣言・local 宣言・global #2 宣言）。
    // それぞれ生成 `.lua` 行へ対応を持つはず（create_scene 行）。
    for pasta_line in [7u32, 11u32, 14u32] {
        let lua_targets = map.resolve_pasta_to_lua(&pasta_key, pasta_line);
        assert!(
            !lua_targets.is_empty(),
            "`.pasta` 行 {pasta_line} は索引同居下でも `.lua` 行へ逆引きできる（3.4）"
        );

        // 逆引きで得た各 (chunk, lua_line) を前方解決すると、元の `.pasta` 行へ戻る。
        for (chunk, lua_line) in &lua_targets {
            let pos = map
                .resolve_lua_to_pasta(chunk, *lua_line)
                .expect("逆引きで得た `.lua` 行は前方解決で `.pasta` 位置へ戻る（双方向整合）");
            assert_eq!(
                pos.line, pasta_line,
                "ラウンドトリップは元の `.pasta` 行 {pasta_line} へ戻る（行マッピング不変・3.4）"
            );
        }
    }

    // 索引クエリと行マッピングクエリが互いに干渉しないこと（同一行で双方が機能する）。
    // local 宣言行は scene_at で identity を返しつつ、行マッピングも逆引き可能。
    let local_id = map.scene_at(&pasta_key, LINE_LOCAL_BODY);
    assert!(local_id.is_some(), "索引クエリは同居下でも機能する");
    let local_decl_lua = map.resolve_pasta_to_lua(&pasta_key, 11);
    assert!(
        !local_decl_lua.is_empty(),
        "同一マップ上で行マッピングクエリも同居下で機能する（相互非干渉・3.4）"
    );
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
// 結果、2 本目の cursor 領域が 1 本目（会話_A_1）へ MIS-RESOLVE する（wrong-scene kick）。
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
        "sanitize 衝突 2 本は別シーンへ解決する（wrong-scene kick の回帰防止）"
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
