//! task 2.2 統合テスト（pasta-scene-kick-from-cursor）。
//!
//! `.pasta` を transpile→ロード（debug 有効）後、finalize join が確定した
//! [`SceneIdentityIndex`]（`Arc<SourceMap>` の write-once スロット）が、既知のシーン
//! 宣言行から **ランタイム実 identity** な (scene_id, parent) を返すことを観測する。
//!
//! 観測対象（requirements 3.1/3.2/3.3/7.1）:
//! 1. グローバル本体領域の行 → `(会話1, None)`（global・parent なし）。
//! 2. 名前付き local 領域の行 → `(挨拶_1, Some(会話1))`（local・parent あり）。
//! 3. 同 base 2 本目のグローバル領域 → `(会話2, None)`（per-base 出現順突合）。
//! 4. 索引が返す identity が `collect_scenes`（runtime SSOT）の値に一致する。
//! 5. 通常モード（debug 無効）非破壊: 索引は構築されず、行マッピング双方向 resolve は不変。

use std::path::{Path, PathBuf};

use pasta_lua::debug::source_map::SceneIdentity;
use pasta_lua::mlua::{Lua, Table, Value};
use pasta_lua::{PastaLoader, RuntimeConfig};

/// 統合フィクスチャ（global「会話」×2 ＋ 名前付き local「挨拶」）。行番号は本ファイルの
/// アサーションが依存するため、フィクスチャ編集時は行も追従すること。
///  7: ＊会話        → global 会話1（本体 __start__ 領域）
///  8: さくら：「おはよう」
///  9: ＞挨拶
/// 10:
/// 11: ・挨拶          → local 挨拶_1（parent 会話1）
/// 12: さくら：「やあ」
/// 13:
/// 14: ＊会話         → global 会話2（同 base 2 本目）
/// 15: さくら：「また会話だよ」
const FIXTURE: &str = include_str!("fixtures/scene_identity_index.pasta");

/// `.pasta` 行（本ファイル中で意味を固定）。
const LINE_GLOBAL1_BODY: u32 = 8; // 会話1 本体（__start__ 領域）
const LINE_LOCAL_BODY: u32 = 12; // 挨拶_1 本体
const LINE_GLOBAL2_BODY: u32 = 15; // 会話2 本体

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

    // (1) グローバル本体領域 → (会話1, None)。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_GLOBAL1_BODY),
        Some(ident("会話1", None)),
        "global 本体行 {LINE_GLOBAL1_BODY} は (会話1, None) へ解決する"
    );

    // (2) 名前付き local 本体領域 → (挨拶_1, Some(会話1))。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_LOCAL_BODY),
        Some(ident("挨拶_1", Some("会話1"))),
        "local 本体行 {LINE_LOCAL_BODY} は (挨拶_1, Some(会話1)) へ解決する"
    );

    // (3) 同 base 2 本目のグローバル → (会話2, None)（per-base 出現順突合）。
    assert_eq!(
        source_map.scene_at(&pasta_key, LINE_GLOBAL2_BODY),
        Some(ident("会話2", None)),
        "2 本目 global 本体行 {LINE_GLOBAL2_BODY} は (会話2, None) へ解決する"
    );

    // (4) 索引の identity が collect_scenes（runtime SSOT）に一致する。
    let scenes = pasta_lua::runtime::finalize::collect_scenes(runtime.lua())
        .expect("collect_scenes must succeed");
    // global 会話1 / 会話2 が存在する。
    assert!(
        scenes.iter().any(|(g, _)| g == "会話1"),
        "runtime に 会話1 が存在する: {scenes:?}"
    );
    assert!(
        scenes.iter().any(|(g, _)| g == "会話2"),
        "runtime に 会話2 が存在する: {scenes:?}"
    );
    // local (会話1, 挨拶_1) が存在し、索引の (挨拶_1, 会話1) と一致する。
    assert!(
        scenes
            .iter()
            .any(|(g, l)| g == "会話1" && l == "挨拶_1"),
        "runtime に (会話1, 挨拶_1) が存在する: {scenes:?}"
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
// `kick_local_composite_test.lua` は `SCENE.search` を **モック** して try_dispatch の
// 分岐を固定する。いずれも「索引 identity が、実ランタイム上の kick 名前検索の
// 解決対象に一致する」ことは END-TO-END では固定していない。
//
// 本 6.1 は単一の **実ロード済みランタイム** 上で次のループを閉じる:
//   位置 → `SourceMap::scene_at`（索引 identity）
//        → kick 取次が組む search-key（global: `会話1` / local: `:会話1:挨拶_1`）
//        → 実 `SCENE.search(name, parent)`（= kick.lua try_dispatch / co_exec の名前検索）
//        → 着地した runtime (global_name, local_name) が `collect_scenes`（SSOT）に一致。
// これにより「索引 identity == kick 名前検索の解決対象 == runtime 実シーン」を実機で
// 突合する（3.1/3.3）。加えて debug 有効・索引充填済みの同一ランタイムで行マッピング
// 双方向 resolve のラウンドトリップ不変を固定する（3.4 後方非破壊）。
// ===========================================================================

/// kick 取次（`debug::playscene::build_kick_scene`）と **同一契約**で確定 identity から
/// kick search-key を組む（global → `scene_id`、local → `:{parent}:{scene_id}`）。
///
/// 本関数は production の private ヘルパと同じ規則をテスト側で再現し、`scene_at` が返す
/// identity が kick 経路に渡される search-key へどう写像されるかを固定する（playscene.rs
/// `build_kick_scene` のドキュメント契約）。組んだ search-key を [`kick_search_runtime`]
/// が実 `SCENE.search` へ食わせ、ループを閉じる。
fn kick_scene_key(id: &SceneIdentity) -> String {
    match &id.parent {
        Some(p) => format!(":{p}:{}", id.scene_id),
        None => id.scene_id.clone(),
    }
}

/// kick search-key を **実ロード済みランタイム** の `SCENE.search` で解決し、着地した
/// runtime シーン `(global_name, local_name)` を返す（kick.lua try_dispatch と同手順）。
///
/// - `:parent:local` 形式（local-composite）: kick.lua の local 分岐と同じく
///   `^:([^:]+):(.+)$` で parent/local_name に分解し、`SCENE.search(local_name, parent)`
///   の **local 分岐** を引く。
/// - 先頭 `:` 無し（global）: kick.lua の global 分岐は `SCENE.co_exec(act, scene)` 経由で
///   `act:find_scene(scene, nil)` を引くが、その実体は `@pasta_search:search_scene(name, nil)`
///   と同じ global 検索（`__start__` を返す）であり、`SCENE.search(scene, nil)` で同一の
///   runtime シーンへ解決する。本ヘルパは search-key だけで着地シーンを観測するため
///   後者を用いる。
///
/// 返り値は `SCENE.search` 結果オブジェクトの `(global_name, local_name)`。解決不能なら
/// `None`（= kick が unresolved drop する位置）。
fn kick_search_runtime(lua: &Lua, search_key: &str) -> Option<(String, String)> {
    let scene_mod: Table = lua
        .load("return require('pasta.scene')")
        .eval()
        .expect("pasta.scene module loads on a finalized runtime");
    let search: pasta_lua::mlua::Function =
        scene_mod.get("search").expect("SCENE.search exists");

    // local-composite `:parent:local` を kick.lua と同じ規則で分解。
    let (name, parent): (String, Option<String>) =
        if let Some(rest) = search_key.strip_prefix(':') {
            match rest.split_once(':') {
                Some((p, local)) => (local.to_string(), Some(p.to_string())),
                None => (rest.to_string(), None),
            }
        } else {
            (search_key.to_string(), None)
        };

    // SCENE.search(name, parent_or_nil) → 結果オブジェクト or nil。
    let parent_arg = match &parent {
        Some(p) => Value::String(lua.create_string(p).unwrap()),
        None => Value::Nil,
    };
    let result: Value = search
        .call((name, parent_arg))
        .expect("SCENE.search call must not error");

    match result {
        Value::Table(t) => {
            let g: String = t.get("global_name").ok()?;
            let l: String = t.get("local_name").ok()?;
            Some((g, l))
        }
        _ => None,
    }
}

/// 6.1-(1) kick 名前検索の解決対象一致（requirements 3.1/3.3）。
///
/// 実ロード済みランタイム上で、既知の global / local 宣言領域の行から `scene_at` で
/// identity を確定し、その identity を kick search-key へ写像し（global → `会話1`、
/// local → `:会話1:挨拶_1`）、実 `SCENE.search` で解決した着地シーンが `collect_scenes`
/// （runtime SSOT）に存在することを END-TO-END で固定する。
///
/// 真正性（RED の論拠）: もし索引 identity と kick 名前検索が乖離すれば
/// （例: local が global の `__start__` へ潰れる／parent 取り違え／scene_id 不一致）、
/// `SCENE.search` の着地は `collect_scenes` の対応エントリと食い違い、本テストの
/// `assert` は落ちる。2.2 は `SCENE.search` を一切引かず、3.1 は合成マップ＋モック検索の
/// ため、この実機ループの突合は本テストが初めて閉じる。
#[test]
fn index_identity_matches_runtime_kick_search_target() {
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
    let scenes = pasta_lua::runtime::finalize::collect_scenes(lua)
        .expect("collect_scenes must succeed");

    // --- global: 本体領域行 → identity (会話1, None) → search-key "会話1" ---
    let global_id = source_map
        .scene_at(&pasta_key, LINE_GLOBAL1_BODY)
        .expect("global 本体行は identity へ解決する");
    assert_eq!(
        global_id,
        ident("会話1", None),
        "scene_at は global identity を返す"
    );
    let global_key = kick_scene_key(&global_id);
    assert_eq!(global_key, "会話1", "global の kick search-key は素の scene_id");

    let (g_global, g_local) = kick_search_runtime(lua, &global_key)
        .expect("global kick search-key は runtime シーンへ着地する");
    // global kick は __start__（無名 start シーン）へ着地するのが kick 決定ノートの契約。
    assert_eq!(
        g_global, "会話1",
        "global kick 名前検索は索引 identity と同じ global へ着地する"
    );
    assert!(
        scenes.iter().any(|(g, l)| *g == g_global && *l == g_local),
        "global kick の着地 ({g_global}, {g_local}) は collect_scenes(SSOT) に存在する: {scenes:?}"
    );
    // 着地 global は索引 identity の scene_id と一致（kick 解決対象 == 索引 identity）。
    assert_eq!(
        g_global, global_id.scene_id,
        "kick 名前検索の解決対象 global == 索引 identity の scene_id"
    );

    // --- local: 名前付き local 本体行 → identity (挨拶_1, Some(会話1)) → ":会話1:挨拶_1" ---
    let local_id = source_map
        .scene_at(&pasta_key, LINE_LOCAL_BODY)
        .expect("local 本体行は identity へ解決する");
    assert_eq!(
        local_id,
        ident("挨拶_1", Some("会話1")),
        "scene_at は local identity を返す"
    );
    let local_key = kick_scene_key(&local_id);
    assert_eq!(
        local_key, ":会話1:挨拶_1",
        "local の kick search-key は composite ':parent:local'"
    );

    let (l_global, l_local) = kick_search_runtime(lua, &local_key)
        .expect("local-composite kick search-key は runtime シーンへ着地する");
    // local 分岐は __start__ へ潰さず local 同一性を保持する（kick 決定ノート）。
    assert_eq!(
        (l_global.as_str(), l_local.as_str()),
        ("会話1", "挨拶_1"),
        "local kick 名前検索は索引 identity と同じ (会話1, 挨拶_1) へ着地する"
    );
    assert!(
        scenes
            .iter()
            .any(|(g, l)| *g == l_global && *l == l_local),
        "local kick の着地 ({l_global}, {l_local}) は collect_scenes(SSOT) に存在する: {scenes:?}"
    );
    // 着地 (global, local) は索引 identity の (parent, scene_id) と一致。
    assert_eq!(
        l_global,
        local_id.parent.clone().expect("local は parent を持つ"),
        "kick 名前検索の解決対象 parent == 索引 identity の parent"
    );
    assert_eq!(
        l_local, local_id.scene_id,
        "kick 名前検索の解決対象 local == 索引 identity の scene_id"
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
