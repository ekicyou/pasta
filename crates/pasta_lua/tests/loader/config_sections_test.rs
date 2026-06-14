//! Phase A: loader/config.rs のインラインテストを外部化（クラスタ分割 C2）
//!
//! 本ファイルは Lua 検索パス / G1 未カバー公開挙動 / GhostConfig SSOT のクラスタを
//! 担う。core 設定セクション（loader/logging/persistence/lua/[debug]）は
//! config_test.rs 側に残置（C2 凝集境界・600行上限）。

use pasta_lua::loader::{
    GhostConfig, LoaderConfig, LoaderError, PastaConfig, PersistenceConfig, TalkConfig,
    default_lua_search_paths,
};

// ========================================
// Lua Search Path tests (lua-module-path-resolution spec)
// ========================================

// ============================================================================
// G1 (review-improvement-loop cell 3.14): untested public behavior
// ============================================================================

// ----- PastaConfig::load (ファイルベース読み込み) -----
//
// NOTE: load の成功経路は tests/loader/startup_test.rs::test_config_load_with_file、
// ファイル欠落→ConfigNotFound は同 test_config_load_not_found（バリアント＋パス検証）と
// src/loader/error.rs::test_config_not_found_display（Display メッセージ検証）で既カバー。
// 本セルで真に新規なのは「不正 TOML → LoaderError::Config」経路のみ。

/// 壊れた TOML は Config（パースエラー）になること。
/// （load の3経路中、唯一の未カバー経路。成功/NotFound は startup_test.rs で既カバー）
#[test]
fn test_load_invalid_toml_returns_config_error() {
    let temp = tempfile::TempDir::new().unwrap();
    std::fs::write(temp.path().join("pasta.toml"), "= broken toml =").unwrap();

    let err = PastaConfig::load(temp.path()).unwrap_err();
    assert!(
        matches!(err, LoaderError::Config(_, _)),
        "invalid TOML must yield Config error, got: {err:?}"
    );
}

// ----- パースエラー・型不一致の許容挙動 -----

/// [loader] セクションのフィールド型が不正な場合、from_str はエラーを返すこと
/// （loader セクションは厳格パース）。
#[test]
fn test_invalid_loader_field_type_fails_parse() {
    let toml_str = r#"
[loader]
pasta_patterns = "not-an-array"
"#;
    assert!(PastaConfig::from_str(toml_str).is_err());
}

/// [loader] 内の未知キーは無視されること（前方互換）。
#[test]
fn test_unknown_loader_keys_are_ignored() {
    let toml_str = r#"
[loader]
debug_mode = false
future_option = "tolerated"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(!config.loader.debug_mode);
}

/// カスタムセクションの型不一致は panic せず None になること
/// （get_custom_config の .ok() フォールバック）。
#[test]
fn test_custom_section_type_mismatch_returns_none() {
    let toml_str = r#"
[logging]
rotation_days = "fourteen"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(
        config.logging().is_none(),
        "wrongly-typed [logging] must deserialize to None, not panic"
    );
    // セクション自体は custom_fields には残っている
    assert!(config.custom_fields.contains_key("logging"));
}

// ----- TalkConfig -----
//
// NOTE: TalkConfig の既定値（禁則文字2フィールドを除く）・直接デシリアライズ・部分上書きは
// tests/sakura_script/output_test.rs の test_talk_config_default /
// test_talk_config_from_toml_full / test_talk_config_partial_override で既カバー。
// 本セルの増分は (1) 禁則文字2フィールドの既定値、(2) PastaConfig::talk() アクセサ経路。

/// 禁則文字2フィールドの既定値固定。
/// （output_test.rs の test_talk_config_default は chars_line_start_prohibited /
/// chars_line_end_prohibited を検証していないため、その差分のみをここで固定する）
#[test]
fn test_talk_config_default_kinsoku_fields() {
    let config = TalkConfig::default();
    assert_eq!(config.chars_line_start_prohibited, "゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ");
    assert_eq!(config.chars_line_end_prohibited, "（［｛「『([{｢");
}

/// [talk] 部分指定: 指定フィールドのみ上書きされ、残りは既定値になること。
/// （増分は PastaConfig::talk() アクセサ経由のセクション抽出経路。
/// TalkConfig 単体の部分上書きは output_test.rs::test_talk_config_partial_override で既カバー）
#[test]
fn test_talk_config_from_toml_partial() {
    let toml_str = r#"
[talk]
script_wait_period = 1500
chars_period = "."
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let talk = config.talk().expect("talk section should exist");
    assert_eq!(talk.script_wait_period, 1500);
    assert_eq!(talk.chars_period, ".");
    // 未指定フィールドは既定値
    assert_eq!(talk.script_wait_normal, 50);
    assert_eq!(talk.chars_comma, "、，,");
}

#[test]
fn test_talk_config_none_when_missing() {
    let config = PastaConfig::from_str("").unwrap();
    assert!(config.talk().is_none());
}

// ----- DebugFileConfig セクション供給 -----

/// [debug] セクションが無い場合 debug() は None（既定 OFF パス）。
#[test]
fn test_debug_config_none_when_missing() {
    let toml_str = r#"
[loader]
debug_mode = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(config.debug().is_none());
}

/// [debug] の port 指定が反映されること（既定 9276 の上書き）。
#[test]
fn test_debug_config_custom_port_from_toml() {
    let toml_str = r#"
[debug]
enabled = true
port = 12345
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let debug = config.debug().expect("[debug] section should parse");
    assert!(debug.enabled);
    assert_eq!(debug.port, 12345);
}

// ----- PersistenceConfig::effective_file_path 第3分岐 -----

/// obfuscate=true かつ .json でも .dat でもない拡張子は ".dat" が付加されること。
#[test]
fn test_persistence_effective_file_path_appends_dat_for_other_ext() {
    let config = PersistenceConfig {
        obfuscate: true,
        file_path: "profile/pasta/save/data.bin".to_string(),
        debug_mode: false,
    };
    assert_eq!(
        config.effective_file_path(),
        "profile/pasta/save/data.bin.dat"
    );
}

#[test]
fn test_default_lua_search_paths_contains_user_scripts() {
    // Requirement 1.2: scripts should be at priority 2 (second position)
    let paths = default_lua_search_paths();
    assert_eq!(paths.len(), 5, "Should have 5 search paths");
    assert_eq!(
        paths,
        vec![
            "profile/pasta/save/lua",
            "scripts",
            "profile/pasta/pasta_scripts",
            "profile/pasta/cache/lua",
            "scriptlibs",
        ],
        "Search paths should be in correct priority order"
    );
}

#[test]
fn test_default_lua_search_paths_user_scripts_priority() {
    // Requirement 1.3: scripts (index 1) should come before profile/pasta/pasta_scripts (index 2)
    let paths = default_lua_search_paths();
    let scripts_pos = paths.iter().position(|p| p == "scripts");
    let pasta_scripts_pos = paths
        .iter()
        .position(|p| p == "profile/pasta/pasta_scripts");
    assert!(scripts_pos.is_some(), "scripts should be in search paths");
    assert!(
        pasta_scripts_pos.is_some(),
        "profile/pasta/pasta_scripts should be in search paths"
    );
    assert!(
        scripts_pos.unwrap() < pasta_scripts_pos.unwrap(),
        "scripts should come before profile/pasta/pasta_scripts for override functionality"
    );
}

#[test]
fn test_loader_config_default_includes_user_scripts() {
    // Verify LoaderConfig::default() uses the new search paths
    let config = LoaderConfig::default();
    assert!(
        config.lua_search_paths.contains(&"scripts".to_string()),
        "Default LoaderConfig should include scripts"
    );
    assert!(
        config
            .lua_search_paths
            .contains(&"profile/pasta/pasta_scripts".to_string()),
        "Default LoaderConfig should include profile/pasta/pasta_scripts"
    );
}

// ----- GhostConfig SSOT -----

/// `[ghost]` の SHIORI デフォルト値の単一供給源（SSOT）。
/// `GhostConfig::default()` が現行 Lua リテラルと同値の
/// `180 / 300 / 30 / 1.5` をそのまま返すことを固定する（要件 1.2 / 1.3 / 3.3）。
#[test]
fn test_ghost_config_default_values() {
    let config = GhostConfig::default();
    assert_eq!(config.talk_interval_min, 180);
    assert_eq!(config.talk_interval_max, 300);
    assert_eq!(config.hour_margin, 30);
    assert_eq!(config.spot_newlines, 1.5);
}

// ----- [ghost] 実体化（補完後形状）の公開API検証 -----
//
// 要件 6.3 / 3.1 / 3.3: ロード後の単一補完ステップ（apply_shiori_defaults）が
// `from_str`（公開パースエントリ）経路上で1回適用され、`[ghost]` を書かなくても
// 補完後の `custom_fields` に `ghost` セクションが実体化することを、公開API境界で固定する。
// （クレート内 parse_materializes_ghost_section_with_defaults の公開API版ミラー。
// 同時に test_default_config の `custom_fields.is_empty()`（Default 経路・補完バイパス）
// が依然有効であることの対になる、from_str 経路の補完後形状を押さえる。）

/// `from_str` 補助: 補完後の `custom_fields["ghost"]` をテーブルとして取り出す。
fn materialized_ghost_table(config: &PastaConfig) -> &toml::Table {
    config
        .custom_fields
        .get("ghost")
        .expect("from_str must materialize a [ghost] section via apply_shiori_defaults")
        .as_table()
        .expect("materialized ghost entry must be a table")
}

/// `[ghost]` を一切書かない設定を **公開API** `PastaConfig::from_str` でパースすると、
/// 補完後の `custom_fields` に `ghost` テーブルが実体化し、4つの SSOT 既定値
/// （180 / 300 / 30 / 1.5）が揃うこと。
///
/// もし補完（apply_shiori_defaults）が parse 経路から外れれば、`ghost` キー自体が
/// 欠落して `materialized_ghost_table` の expect が失敗するため、このテストは
/// 実体化コントラクトを境界でピン留めする（タスク 2.3 の配線が存在するため通る）。
#[test]
fn test_from_str_materializes_ghost_defaults_when_section_omitted() {
    // `[ghost]` を含まない最小構成（`[actor]` のみ）。
    let config = PastaConfig::from_str("[actor]\nname = \"sakura\"\nspot = 0\n").unwrap();

    let ghost = materialized_ghost_table(&config);
    assert_eq!(
        ghost.get("talk_interval_min").and_then(toml::Value::as_integer),
        Some(180),
        "6.3/3.3: omitted [ghost] must materialize talk_interval_min=180 at the public API"
    );
    assert_eq!(
        ghost.get("talk_interval_max").and_then(toml::Value::as_integer),
        Some(300),
        "6.3/3.3: omitted [ghost] must materialize talk_interval_max=300 at the public API"
    );
    assert_eq!(
        ghost.get("hour_margin").and_then(toml::Value::as_integer),
        Some(30),
        "6.3/3.3: omitted [ghost] must materialize hour_margin=30 at the public API"
    );
    assert_eq!(
        ghost.get("spot_newlines").and_then(toml::Value::as_float),
        Some(1.5),
        "6.3/3.3: omitted [ghost] must materialize spot_newlines=1.5 at the public API"
    );
}

/// 明示カスタムフィールドを持ちつつ `[ghost]` を省略した設定を `from_str` でパースすると、
/// 作者が書いた明示フィールド（top-level `ghost_name` と `[user_data]` セクション）が
/// **保持**され、その隣に `ghost` が実体化すること（明示値を clobber しない）。
///
/// 補完は欠落キーの追加のみを行い、既存の custom_fields を破壊しないことを境界で固定する。
#[test]
fn test_from_str_preserves_explicit_custom_fields_alongside_materialized_ghost() {
    let toml_str = r#"
ghost_name = "TestGhost"

[actor]
name = "sakura"
spot = 0

[user_data]
key1 = "value1"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();

    // 明示カスタムフィールドは補完で上書き・除去されない。
    assert_eq!(
        config.custom_fields.get("ghost_name"),
        Some(&toml::Value::String("TestGhost".to_string())),
        "explicit top-level custom field must survive default materialization"
    );
    let user_data = config
        .custom_fields
        .get("user_data")
        .and_then(toml::Value::as_table)
        .expect("explicit [user_data] section must survive default materialization");
    assert_eq!(
        user_data.get("key1"),
        Some(&toml::Value::String("value1".to_string())),
        "explicit nested custom field must survive default materialization"
    );

    // 明示フィールドの隣に ghost が実体化し、SSOT 既定値が揃う。
    let ghost = materialized_ghost_table(&config);
    assert_eq!(
        ghost.get("talk_interval_min").and_then(toml::Value::as_integer),
        Some(180)
    );
    assert_eq!(
        ghost.get("talk_interval_max").and_then(toml::Value::as_integer),
        Some(300)
    );
    assert_eq!(
        ghost.get("hour_margin").and_then(toml::Value::as_integer),
        Some(30)
    );
    assert_eq!(
        ghost.get("spot_newlines").and_then(toml::Value::as_float),
        Some(1.5)
    );
}
