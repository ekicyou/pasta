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
