//! 補完・最小起動・SSOT ガードの統合テスト（pasta-config-restructure task 5.2）。
//!
//! 検証対象:
//! 1. 最小起動＋辞書読込 (R2.2 / R2.5 / R6.4)
//! 2. 最小/フル等価 — `@pasta_config.ghost.*` の Rust/Lua 一貫性 (R3.1 / R3.3)
//! 3. `[actor]` 不在警告で起動/補完が停止しない (R2.3)
//! 4. SSOT ガード: ドキュメント・Lua フォールバックリテラルと
//!    `GhostConfig::default()` の一致によるドリフト検出 (R5.4 / R5.5)

use crate::common;

use common::copy_fixture_to_temp;
use pasta_lua::loader::{
    GhostConfig, LoaderError, PastaConfig, PastaLoader, default_hour_margin, default_spot_newlines,
    default_talk_interval_max, default_talk_interval_min,
};
use std::path::PathBuf;

/// テスト補助: 実ランタイムを `@pasta_config` の `ghost` から数値キーを読み出す。
fn read_pasta_config_ghost_number(runtime: &pasta_lua::PastaLuaRuntime, key: &str) -> f64 {
    let script = format!(
        r#"
        local config = require("@pasta_config")
        return config.ghost.{key}
    "#
    );
    let result = runtime.exec(&script).unwrap();
    result
        .as_f64()
        .or_else(|| result.as_i64().map(|v| v as f64))
        .unwrap_or_else(|| panic!("@pasta_config.ghost.{key} should be a number"))
}

// ============================================================================
// 1. 最小起動＋辞書読込 (R2.2 / R2.5 / R6.4)
// ============================================================================

/// 最小構成（`[actor]` のみ ＋ `dic/talk.pasta` 直下）で起動でき、
/// `dic/` 直下の辞書が `pasta_patterns` の SHIORI デフォルト（`dic/**/*.pasta`）で
/// 読み込まれること。
#[test]
fn minimal_actor_only_starts_and_loads_dic_directly() {
    let temp = copy_fixture_to_temp("minimal_actor_only");
    let runtime = PastaLoader::load(temp.path()).unwrap();

    // 起動後ランタイムが実行可能であること（R2.2）。
    let alive = runtime.exec("return 1 + 1").unwrap();
    assert_eq!(alive.as_i64(), Some(2));

    // `dic/` 直下の `talk.pasta` が既定 glob で発見・登録されたこと（R2.5 / R6.4）。
    // ローダはトランスパイル結果を scene_dic.lua に列挙する。
    let scene_dic_path = temp
        .path()
        .join("profile/pasta/cache/lua/pasta/scene_dic.lua");
    let scene_dic = std::fs::read_to_string(&scene_dic_path)
        .expect("scene_dic.lua should be generated on load");
    assert!(
        scene_dic.contains("pasta.scene.talk"),
        "dic/ 直下の talk.pasta が既定 glob で登録されていない: {scene_dic}"
    );
}

// ============================================================================
// 2. 最小/フル等価 — @pasta_config.ghost.* の Rust/Lua 一貫性 (R3.1 / R3.3)
// ============================================================================

/// 最小構成（`[actor]` のみ）と、SSOT 値を明示した等価なフル記述で、
/// 補完後の `custom_fields.ghost` が Rust 側で同値になること（R3.1）。
#[test]
fn minimal_and_full_ghost_are_equivalent_in_rust() {
    // 最小: [ghost] 省略 → 補完で実体化。
    let minimal = PastaConfig::from_str("[actor.\"女の子\"]\nspot = 0\n").unwrap();

    // フル: SSOT 値を明示記述。補完は明示値を上書きしないので同値になるはず。
    let full = PastaConfig::from_str(
        "[actor.\"女の子\"]\nspot = 0\n\
         [ghost]\n\
         talk_interval_min = 180\n\
         talk_interval_max = 300\n\
         hour_margin = 30\n\
         spot_newlines = 1.5\n",
    )
    .unwrap();

    let minimal_ghost = minimal
        .custom_fields
        .get("ghost")
        .and_then(toml::Value::as_table)
        .expect("ghost should be materialized for minimal config");
    let full_ghost = full
        .custom_fields
        .get("ghost")
        .and_then(toml::Value::as_table)
        .expect("ghost should be present for full config");

    assert_eq!(
        minimal_ghost, full_ghost,
        "最小構成の補完済み ghost とフル記述の ghost が一致しない"
    );
}

/// 最小構成（`[actor]` のみ）とフル記述の双方を **実ランタイムで起動** し、
/// Lua から読み出した `@pasta_config.ghost.*` が SSOT 既定値と一致すること。
/// Rust 補完と Lua 露出の一貫性を実際の経路で証明する（R3.3）。
#[test]
fn minimal_and_full_expose_same_ghost_to_lua() {
    // 最小構成フィクスチャ（[actor] のみ）を実起動。
    let minimal_temp = copy_fixture_to_temp("minimal_actor_only");
    let minimal_runtime = PastaLoader::load(minimal_temp.path()).unwrap();

    // SSOT 既定値が Lua の @pasta_config.ghost に露出していること（R3.3）。
    assert_eq!(
        read_pasta_config_ghost_number(&minimal_runtime, "spot_newlines"),
        default_spot_newlines(),
    );
    assert_eq!(
        read_pasta_config_ghost_number(&minimal_runtime, "talk_interval_min"),
        default_talk_interval_min() as f64,
    );
    assert_eq!(
        read_pasta_config_ghost_number(&minimal_runtime, "talk_interval_max"),
        default_talk_interval_max() as f64,
    );
    assert_eq!(
        read_pasta_config_ghost_number(&minimal_runtime, "hour_margin"),
        default_hour_margin() as f64,
    );

    // フル記述（SSOT 値を明示）を別の一時ツリーで実起動し、同値を確認。
    let full_temp = copy_fixture_to_temp("minimal_actor_only");
    std::fs::write(
        full_temp.path().join("pasta.toml"),
        "[actor.\"女の子\"]\nspot = 0\n\
         [actor.\"男の子\"]\nspot = 1\n\
         [ghost]\n\
         talk_interval_min = 180\n\
         talk_interval_max = 300\n\
         hour_margin = 30\n\
         spot_newlines = 1.5\n",
    )
    .unwrap();
    let full_runtime = PastaLoader::load(full_temp.path()).unwrap();

    // 最小/フルで Lua 露出値が一致すること（最小=補完値 == フル=明示値）。
    for key in ["spot_newlines", "talk_interval_min", "talk_interval_max", "hour_margin"] {
        assert_eq!(
            read_pasta_config_ghost_number(&minimal_runtime, key),
            read_pasta_config_ghost_number(&full_runtime, key),
            "最小/フルで @pasta_config.ghost.{key} が一致しない",
        );
    }
}

// ============================================================================
// 3. [actor] 不在警告で起動/補完が停止しない (R2.3)
// ============================================================================

/// `[actor]` を含まない設定で起動し、不在警告が1回発火し、かつ起動・補完が
/// 停止しないこと（R2.3）。警告メッセージは production の `apply_shiori_defaults`
/// が発する実フレーズで照合する（span/test 名と誤マッチしない識別句）。
#[tracing_test::traced_test]
#[test]
fn actor_absence_warns_without_halting_startup() {
    // 実起動経路を通すため、最小フィクスチャをコピーしてから pasta.toml を
    // [actor] 不在（[ghost] のみ）に差し替える。dic/ はそのまま流用。
    let temp = copy_fixture_to_temp("minimal_actor_only");
    std::fs::write(
        temp.path().join("pasta.toml"),
        "[ghost]\ntalk_interval_min = 120\n",
    )
    .unwrap();

    // 起動が停止しない（Ok を返す）こと。
    let runtime = PastaLoader::load(temp.path()).unwrap();
    let alive = runtime.exec("return 7").unwrap();
    assert_eq!(alive.as_i64(), Some(7));

    // actor 不在警告が発火していること（production の実フレーズで照合）。
    assert!(
        logs_contain("No [actor] section is defined"),
        "actor 不在時には起動を妨げない警告が発火するべき (R2.3)"
    );
}

// ============================================================================
// 4. SSOT ガード: ドキュメント・Lua リテラルと SSOT の一致 (R5.4 / R5.5)
// ============================================================================

/// SSOT アンカー: `GhostConfig::default()` が確定既定値（180/300/30/1.5）で
/// あること。これがドリフト検出の基準点となる。
#[test]
fn ghost_config_default_is_ssot_anchor() {
    let d = GhostConfig::default();
    assert_eq!(d.talk_interval_min, 180);
    assert_eq!(d.talk_interval_max, 300);
    assert_eq!(d.hour_margin, 30);
    assert_eq!(d.spot_newlines, 1.5);

    // const default_*() 関数も同じ SSOT を返すこと（Lua 比較で利用するため）。
    assert_eq!(default_talk_interval_min(), d.talk_interval_min);
    assert_eq!(default_talk_interval_max(), d.talk_interval_max);
    assert_eq!(default_hour_margin(), d.hour_margin);
    assert_eq!(default_spot_newlines(), d.spot_newlines);
}

/// リポジトリルートを `CARGO_MANIFEST_DIR`（crates/pasta_lua）から2階層上に解決。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("repo root should resolve from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

/// フルリファレンステンプレート/ドキュメント（pasta-toml.md）に記載の ghost
/// 既定値が SSOT と一致すること（R5.4 / R5.5）。テンプレートが独立に乖離した
/// 場合に失敗する。
#[test]
fn config_reference_doc_matches_ssot() {
    let doc_path = repo_root()
        .join(".claude/skills/pasta-ghost-authoring/references/pasta-toml.md");
    let doc = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("config reference doc not readable at {doc_path:?}: {e}"));

    let d = GhostConfig::default();

    // 分類表の各行に SSOT 値が記載されていること（例: `talk_interval_min` | ... | `180`）。
    // SSOT 値の文字列表現でドリフトを検出する。
    let expect_row = |key: &str, value: String| {
        let has = doc.lines().any(|line| {
            line.contains(key) && line.contains(&format!("`{value}`"))
        });
        assert!(
            has,
            "config reference doc の `{key}` 行に SSOT 値 `{value}` が見つからない (R5.4/R5.5)"
        );
    };

    expect_row("talk_interval_min", d.talk_interval_min.to_string());
    expect_row("talk_interval_max", d.talk_interval_max.to_string());
    expect_row("hour_margin", d.hour_margin.to_string());
    // spot_newlines は浮動小数（"1.5"）。
    expect_row("spot_newlines", format!("{:?}", d.spot_newlines));
}

/// Lua フォールバックリテラルが SSOT と一致すること（R5.5）。
/// `act.lua`（spot_newlines = 1.5）および `virtual_dispatcher.lua`
/// （180/300/30）のハードコード既定を SSOT と突き合わせ、将来の乖離を捕捉する。
#[test]
fn lua_fallback_literals_match_ssot() {
    let scripts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pasta_scripts");
    let d = GhostConfig::default();

    // act.lua: spot_newlines の Lua 側フォールバックリテラル。
    let act_lua = std::fs::read_to_string(scripts.join("pasta/shiori/act.lua"))
        .expect("act.lua should be readable");
    let spot_default = format!("{:?}", d.spot_newlines); // "1.5"
    assert!(
        act_lua.contains(&format!("\"spot_newlines\", {spot_default}")),
        "act.lua の spot_newlines フォールバックリテラルが SSOT ({spot_default}) と不一致 (R5.5)"
    );

    // virtual_dispatcher.lua: talk_interval_min/max/hour_margin のフォールバックリテラル。
    let vd_lua = std::fs::read_to_string(scripts.join("pasta/shiori/event/virtual_dispatcher.lua"))
        .expect("virtual_dispatcher.lua should be readable");

    // resolve("...", "talk_interval_min", 180) の第3引数（既定値）。
    assert!(
        vd_lua.contains(&format!(
            "\"talk_interval_min\", {}",
            d.talk_interval_min
        )),
        "virtual_dispatcher.lua の talk_interval_min フォールバック ({}) が SSOT と不一致 (R5.5)",
        d.talk_interval_min
    );
    assert!(
        vd_lua.contains(&format!(
            "\"talk_interval_max\", {}",
            d.talk_interval_max
        )),
        "virtual_dispatcher.lua の talk_interval_max フォールバック ({}) が SSOT と不一致 (R5.5)",
        d.talk_interval_max
    );
    // hour_margin: `ghost.hour_margin or 30` の形でフォールバック。
    assert!(
        vd_lua.contains(&format!("ghost.hour_margin or {}", d.hour_margin)),
        "virtual_dispatcher.lua の hour_margin フォールバック ({}) が SSOT と不一致 (R5.5)",
        d.hour_margin
    );
}

// ============================================================================
// 5. 完全後方互換の回帰確認 (R3.5 / R4.5 / R6.1 / R6.2)
// ============================================================================

/// `[actor]` 不在警告の識別フレーズ。production の `apply_shiori_defaults`
/// （`config.rs`）が発する実フレーズと一致させること。フル記述（`[actor]` あり）
/// 起動でこのフレーズが**出ない**ことを確認するために使う。
const ACTOR_ABSENCE_PHRASE: &str = "No [actor] section is defined";

/// `[package]` を含む従来のフル記述（hello-pasta 形式）。`[package]`（エンジン
/// プロファイル専用）＋ `[actor]` ＋ 明示 `[ghost]`（SSOT とは**異なる**値）＋
/// その他セクション（`[loader]` / `[talk]`）を網羅する。明示 `[ghost]` 値が補完で
/// 上書きされないこと（no-clobber）を検証できるよう、既定とは違う値を入れる。
const LEGACY_FULL_CONFIG: &str = "\
[package]\n\
name = \"hello-pasta\"\n\
version = \"0.1.0\"\n\
edition = \"2021\"\n\
\n\
[actor.\"女の子\"]\n\
spot = 0\n\
\n\
[actor.\"男の子\"]\n\
spot = 1\n\
\n\
[ghost]\n\
talk_interval_min = 90\n\
talk_interval_max = 200\n\
hour_margin = 15\n\
spot_newlines = 2.5\n\
\n\
[loader]\n\
debug_mode = true\n\
\n\
[talk]\n\
script_wait_normal = 40\n";

/// `[package]` を含むフル記述が**エラーなく**ロードでき、`[package]` が
/// `custom_fields` にそのまま保持され（害なく・警告なし）、明示 `[ghost]` 値が
/// 補完で**上書きされない**こと（R4.5 / R6.1 / R6.2）。
///
/// あわせて、`#[traced_test]` で `[actor]` 不在警告フレーズが**出ない**ことを
/// 確認する（フル記述は `[actor]` を含むため）。
#[tracing_test::traced_test]
#[test]
fn legacy_full_config_with_package_loads_cleanly() {
    let config = PastaConfig::from_str(LEGACY_FULL_CONFIG)
        .expect("legacy full config with [package] must parse without error (R6.1/R6.2)");

    // `[package]` は custom_fields にそのまま保持される（エラーや警告にならない）。
    let package = config
        .custom_fields
        .get("package")
        .and_then(toml::Value::as_table)
        .expect("[package] should be preserved in custom_fields (R4.5/R6.2)");
    assert_eq!(
        package.get("name").and_then(toml::Value::as_str),
        Some("hello-pasta"),
        "[package].name は無改変で保持されるべき"
    );
    assert_eq!(
        package.get("version").and_then(toml::Value::as_str),
        Some("0.1.0"),
    );
    assert_eq!(
        package.get("edition").and_then(toml::Value::as_str),
        Some("2021"),
    );

    // 明示 `[ghost]` 値は補完で上書きされない（no-clobber, R6.1）。SSOT 既定
    // （180/300/30/1.5）ではなく、作者の書いた値（90/200/15/2.5）が残ること。
    let ghost = config
        .custom_fields
        .get("ghost")
        .and_then(toml::Value::as_table)
        .expect("[ghost] should be present");
    assert_eq!(
        ghost.get("talk_interval_min").and_then(toml::Value::as_integer),
        Some(90),
        "明示 ghost 値が補完で上書きされてはならない (R6.1)"
    );
    assert_eq!(
        ghost.get("talk_interval_max").and_then(toml::Value::as_integer),
        Some(200),
    );
    assert_eq!(
        ghost.get("hour_margin").and_then(toml::Value::as_integer),
        Some(15),
    );
    assert_eq!(
        ghost.get("spot_newlines").and_then(toml::Value::as_float),
        Some(2.5),
    );

    // `[actor]` を含むので不在警告は発火しない（無警告起動の確認, R6.2）。
    assert!(
        !logs_contain(ACTOR_ABSENCE_PHRASE),
        "[actor] を含むフル記述で actor 不在警告が出てはならない (R6.2)"
    );
}

/// `[package]` を含むフル記述で**実ランタイムが起動**でき（無警告・無挙動変化）、
/// 明示 `[ghost]` 値が Lua の `@pasta_config.ghost.*` にそのまま露出すること
/// （補完が明示値をクロバーしない＝R6.1、実起動経路での後方互換＝R6.3/R6.4）。
#[tracing_test::traced_test]
#[test]
fn legacy_full_config_starts_runtime_without_warning() {
    // 最小フィクスチャ（dic/ を流用）に従来フル記述の pasta.toml を被せる。
    let temp = copy_fixture_to_temp("minimal_actor_only");
    std::fs::write(temp.path().join("pasta.toml"), LEGACY_FULL_CONFIG).unwrap();

    // フル記述で実起動が成功する（Ok）。
    let runtime = PastaLoader::load(temp.path())
        .expect("legacy full config must start a real runtime (R6.1/R6.3)");
    let alive = runtime.exec("return 21 + 21").unwrap();
    assert_eq!(alive.as_i64(), Some(42));

    // 明示 ghost 値が Lua 側にそのまま露出する（補完で上書きされていない, R6.1）。
    assert_eq!(
        read_pasta_config_ghost_number(&runtime, "talk_interval_min"),
        90.0,
        "明示 ghost 値が Lua 露出時に上書きされてはならない (R6.1)"
    );
    assert_eq!(
        read_pasta_config_ghost_number(&runtime, "spot_newlines"),
        2.5,
    );

    // フル記述（`[actor]` あり）の実起動で actor 不在警告が出ないこと（無警告起動）。
    assert!(
        !logs_contain(ACTOR_ABSENCE_PHRASE),
        "[actor] を含むフル記述の実起動で actor 不在警告が出てはならない (R6.2)"
    );
}

/// `pasta.toml` 不在は引き続き既存の `LoaderError::ConfigNotFound` で弾かれる
/// （ファイル不在を許容しない＝R3.5）。最小化対象は記述量であって、ファイルの
/// 存在自体は必須のまま。
#[test]
fn missing_pasta_toml_is_rejected_with_config_not_found() {
    // pasta.toml を含まない空ディレクトリ。
    let temp = tempfile::TempDir::new().unwrap();

    let err = PastaConfig::load(temp.path())
        .expect_err("file absence must be rejected, not tolerated (R3.5)");

    // 既存の specific なエラー variant（ConfigNotFound）で弾かれること。
    assert!(
        matches!(err, LoaderError::ConfigNotFound(_)),
        "ファイル不在は既存の ConfigNotFound で弾かれるべき (R3.5)。実際: {err:?}"
    );
}
