use super::*;

/// 既定の探索パターンは慣例 dic 配置（直下・一階層・多階層）を網羅する
/// 再帰形 glob を返す（Requirement 2.5）。
#[test]
fn default_pasta_patterns_is_recursive() {
    assert_eq!(default_pasta_patterns(), vec!["dic/**/*.pasta".to_string()]);
}

/// `LoaderConfig::default()` の `pasta_patterns` も同じ再帰形既定を採用する。
#[test]
fn loader_config_default_uses_recursive_pasta_patterns() {
    assert_eq!(
        LoaderConfig::default().pasta_patterns,
        vec!["dic/**/*.pasta".to_string()]
    );
}

/// テスト補助: `custom_fields["ghost"]` をテーブルとして取り出す。
fn ghost_table(config: &PastaConfig) -> &toml::Table {
    config
        .custom_fields
        .get("ghost")
        .and_then(|v| v.as_table())
        .expect("ghost section should exist after apply_shiori_defaults")
}

/// `[ghost]` セクションを完全に省略した場合、補完後に全4キーが
/// SSOT 既定値（180/300/30/1.5）で埋まる（Requirement 3.1/3.2/3.3）。
#[test]
fn apply_shiori_defaults_fills_all_ghost_keys_when_section_omitted() {
    // Construct a ghost-absent config WITHOUT routing through `from_str`,
    // because `parse`/`from_str` now applies `apply_shiori_defaults` once and
    // would materialize `[ghost]` before this test could observe its absence.
    // Building `custom_fields` directly lets us exercise the function in
    // isolation on a genuinely ghost-absent input.
    let mut config = PastaConfig::default();
    config.custom_fields.insert("actor".to_string(), {
        let mut actor = toml::Table::new();
        actor.insert(
            "name".to_string(),
            toml::Value::String("sakura".to_string()),
        );
        toml::Value::Table(actor)
    });
    assert!(
        config.custom_fields.get("ghost").is_none(),
        "precondition: ghost section absent before apply"
    );

    config.apply_shiori_defaults();

    let ghost = ghost_table(&config);
    assert_eq!(
        ghost.get("talk_interval_min").unwrap().as_integer(),
        Some(180)
    );
    assert_eq!(
        ghost.get("talk_interval_max").unwrap().as_integer(),
        Some(300)
    );
    assert_eq!(ghost.get("hour_margin").unwrap().as_integer(), Some(30));
    assert_eq!(ghost.get("spot_newlines").unwrap().as_float(), Some(1.5));
}

/// 部分的に書かれた `[ghost]`（明示値 `talk_interval_min=120`）は不変で、
/// 欠落キーのみ既定で補完される（Requirement 3.4: 明示値を上書きしない）。
#[test]
fn apply_shiori_defaults_preserves_explicit_ghost_values() {
    let mut config = PastaConfig::from_str("[ghost]\ntalk_interval_min = 120\n").unwrap();

    config.apply_shiori_defaults();

    let ghost = ghost_table(&config);
    // 明示値は不変
    assert_eq!(
        ghost.get("talk_interval_min").unwrap().as_integer(),
        Some(120)
    );
    // 欠落キーは既定で補完
    assert_eq!(
        ghost.get("talk_interval_max").unwrap().as_integer(),
        Some(300)
    );
    assert_eq!(ghost.get("hour_margin").unwrap().as_integer(), Some(30));
    assert_eq!(ghost.get("spot_newlines").unwrap().as_float(), Some(1.5));
}

/// 補完は冪等: 二重適用しても結果が変わらない（Service Interface invariant）。
#[test]
fn apply_shiori_defaults_is_idempotent() {
    let mut once = PastaConfig::from_str("[ghost]\ntalk_interval_min = 120\n").unwrap();
    once.apply_shiori_defaults();

    let mut twice = PastaConfig::from_str("[ghost]\ntalk_interval_min = 120\n").unwrap();
    twice.apply_shiori_defaults();
    twice.apply_shiori_defaults();

    assert_eq!(ghost_table(&once), ghost_table(&twice));
}

/// `[package]`（エンジンプロファイル専用）は補完対象外であり、
/// 補完後も追加・削除・変更されない（Design: [package] は補完しない）。
#[test]
fn apply_shiori_defaults_does_not_touch_package_section() {
    let mut config =
        PastaConfig::from_str("[package]\nname = \"demo\"\nversion = \"1.0\"\n").unwrap();
    let before = config.custom_fields.get("package").cloned();

    config.apply_shiori_defaults();

    let after = config.custom_fields.get("package").cloned();
    assert_eq!(
        before, after,
        "[package] section must be untouched by apply_shiori_defaults"
    );
}

/// `[actor]` セクションが存在しない補完時は、起動を妨げない軽量な警告
/// （warn レベルのログ）を1回発し、かつ補完処理は正常終了する
/// （Requirement 2.3: 起動継続・判別可能化）。
#[tracing_test::traced_test]
#[test]
fn apply_shiori_defaults_warns_when_actor_section_absent() {
    // `[actor]` を含まない最小構成（[ghost] のみ）。
    let mut config = PastaConfig::from_str("[ghost]\ntalk_interval_min = 120\n").unwrap();
    assert!(
        config.custom_fields.get("actor").is_none(),
        "precondition: actor section absent before apply"
    );

    // 補完はエラーや panic を起こさず正常終了する（起動継続）。
    config.apply_shiori_defaults();

    // actor 不在の警告が発火している（判別可能化）。
    // 注: `tracing-test` の `logs_contain` はスパン名（=テスト関数名）も
    // 走査するため、関数名に含まれない識別フレーズで照合する。
    assert!(
        logs_contain("No [actor] section is defined"),
        "actor-absence warning should be emitted when [actor] is missing"
    );
}

/// `[actor]` セクション（テーブル）が存在する場合は、actor 不在警告を
/// 発しない（誤検知しない）。補完処理は正常終了する。
#[tracing_test::traced_test]
#[test]
fn apply_shiori_defaults_does_not_warn_when_actor_section_present() {
    let mut config = PastaConfig::from_str("[actor]\nname = \"sakura\"\n").unwrap();

    config.apply_shiori_defaults();

    // actor が存在するので不在警告は出ない。
    assert!(
        !logs_contain("No [actor] section is defined"),
        "no actor-absence warning should be emitted when [actor] is present"
    );
}

/// `[ghost]` を一切書かない設定を `parse`（`from_str`）するだけで、補完
/// チョークポイント（`apply_shiori_defaults`）が経路上で1回適用され、結果の
/// `custom_fields` に `ghost` セクションが実体化し既定値（180/300/30/1.5）が
/// 入る（Requirement 3.1/3.3, Design: parse 戻り直前の単一補完）。
#[test]
fn parse_materializes_ghost_section_with_defaults() {
    // `[ghost]` を含まない最小構成（`[actor]` のみ）。
    let config = PastaConfig::from_str("[actor]\nname = \"sakura\"\n").unwrap();

    let ghost = ghost_table(&config);
    assert_eq!(
        ghost.get("talk_interval_min").unwrap().as_integer(),
        Some(180)
    );
    assert_eq!(
        ghost.get("talk_interval_max").unwrap().as_integer(),
        Some(300)
    );
    assert_eq!(ghost.get("hour_margin").unwrap().as_integer(), Some(30));
    assert_eq!(ghost.get("spot_newlines").unwrap().as_float(), Some(1.5));
}

// ---------------------------------------------------------------------------
// [scene.alias]（scene-name-alias task 2.1）
// ---------------------------------------------------------------------------

/// テスト補助: 読み込みエラーの文言を取り出す。
fn scene_alias_err(toml: &str) -> String {
    PastaConfig::from_str(toml)
        .expect_err("invalid [scene.alias] must be a config error")
        .to_string()
}

/// `[scene]` が無ければ内蔵の既定表（1.1・6.3）。`default()` と同じ表になる。
#[test]
fn scene_alias_absent_uses_builtin_default() {
    let config = PastaConfig::from_str("[actor]\nname = \"sakura\"\n").unwrap();
    assert_eq!(config.scene_aliases, SceneAliasTable::builtin_default());
    assert_eq!(config.scene_alias_source, SceneAliasSource::BuiltinDefault);

    let default = PastaConfig::default();
    assert_eq!(default.scene_aliases, config.scene_aliases);
    assert_eq!(default.scene_alias_source, config.scene_alias_source);
}

/// `[scene]` に `alias` が無いときも既定表（1.1）。
#[test]
fn scene_section_without_alias_uses_builtin_default() {
    let config = PastaConfig::from_str("[scene]\nother = 1\n").unwrap();
    assert_eq!(config.scene_aliases, SceneAliasTable::builtin_default());
    assert_eq!(config.scene_alias_source, SceneAliasSource::BuiltinDefault);
}

/// `[scene.alias]` の見出しだけなら空の表（2.3・5.6）。
#[test]
fn scene_alias_header_only_is_empty_table() {
    let config = PastaConfig::from_str("[scene.alias]\n").unwrap();
    assert!(config.scene_aliases.is_empty());
    assert_eq!(config.scene_alias_source, SceneAliasSource::PastaToml);
}

/// 書いた行だけが有効（既定とマージしない。2.1・2.2・2.4）。
#[test]
fn scene_alias_author_table_replaces_default() {
    let config = PastaConfig::from_str(
        "[scene.alias]\nOnTalk = [\"会話\", \"雑談\"]\nOnBoot = [\"起動\"]\n",
    )
    .unwrap();
    assert_eq!(config.scene_alias_source, SceneAliasSource::PastaToml);
    assert_eq!(config.scene_aliases.resolve("会話"), Some("OnTalk"));
    assert_eq!(config.scene_aliases.resolve("雑談"), Some("OnTalk"));
    assert_eq!(config.scene_aliases.resolve("起動"), Some("OnBoot"));

    // 既定とマージしない: OnBoot だけを書けば「会話」は置き換わらない。
    let only_boot = PastaConfig::from_str("[scene.alias]\nOnBoot = [\"起動\"]\n").unwrap();
    assert_eq!(only_boot.scene_aliases.resolve("会話"), None);
    assert_eq!(only_boot.scene_aliases.resolve("起動"), Some("OnBoot"));
}

/// 配列でない値は型不一致の読み込みエラー（toml の行・列つき。2.5）。
#[test]
fn scene_alias_non_array_value_is_error_with_line() {
    let msg = scene_alias_err("[actor]\nname = \"a\"\n\n[scene.alias]\nOnTalk = \"会話\"\n");
    assert!(msg.contains("line 5"), "{msg}");
}

/// 配列の中の文字列でない要素は読み込みエラー（2.5）。
#[test]
fn scene_alias_non_string_element_is_error_with_line() {
    let msg = scene_alias_err("[scene.alias]\nOnTalk = [\"会話\", 1]\n");
    assert!(msg.contains("line 2"), "{msg}");
}

/// 空文字列の別名・キーは読み込みエラー。キー名と行番号を含む（2.5）。
#[test]
fn scene_alias_empty_name_is_error() {
    let msg = scene_alias_err("[scene.alias]\nOnBoot = [\"起動\"]\nOnTalk = [\"\"]\n");
    assert!(msg.contains("empty name"), "{msg}");
    assert!(msg.contains("OnTalk"), "{msg}");
    assert!(msg.contains("line 3"), "{msg}");

    let msg = scene_alias_err("[scene.alias]\n\"\" = [\"x\"]\n");
    assert!(msg.contains("empty name"), "{msg}");
    assert!(msg.contains("line 2"), "{msg}");
}

/// 重複（配列間・配列内）は読み込みエラー。別名・両方の置き換え先・行番号を含む（2.6）。
#[test]
fn scene_alias_duplicate_is_error() {
    let msg = scene_alias_err("[scene.alias]\nOnTalk = [\"会話\"]\nOnBoot = [\"会話\"]\n");
    assert!(msg.contains("more than once"), "{msg}");
    assert!(msg.contains("会話"), "{msg}");
    assert!(msg.contains("OnTalk"), "{msg}");
    assert!(msg.contains("OnBoot"), "{msg}");
    assert!(msg.contains("line 2"), "{msg}");
    assert!(msg.contains("line 3"), "{msg}");

    let msg = scene_alias_err("[scene.alias]\nOnTalk = [\n  \"会話\",\n  \"会話\",\n]\n");
    assert!(msg.contains("more than once"), "{msg}");
    assert!(msg.contains("会話"), "{msg}");
    assert!(msg.contains("line 3"), "{msg}");
    assert!(msg.contains("line 4"), "{msg}");
}

/// 連鎖（自己参照を含む）は読み込みエラー。連鎖した名前と行番号を含む（3.5 / 10.3）。
#[test]
fn scene_alias_chain_is_error() {
    // 日本語のキーは toml の裸のキーにできないため引用符で書く。
    let msg = scene_alias_err("[scene.alias]\nOnTalk = [\"会話\"]\n\"会話\" = [\"雑談\"]\n");
    assert!(msg.contains("chained"), "{msg}");
    assert!(msg.contains("会話"), "{msg}");
    assert!(msg.contains("OnTalk"), "{msg}");
    assert!(msg.contains("line 2"), "{msg}");

    let msg = scene_alias_err("[scene.alias]\nOnTalk = [\"OnTalk\"]\n");
    assert!(msg.contains("chained"), "{msg}");
    assert!(msg.contains("OnTalk"), "{msg}");
    assert!(msg.contains("line 2"), "{msg}");
}

/// `[scene]` の `alias` 以外のキーは読まずに `custom_fields` に残り、
/// `[scene]` は書いたとおり出る（既定の表は補完しない）。
#[test]
fn scene_section_other_keys_are_preserved_in_custom_fields() {
    let config = PastaConfig::from_str("[scene]\nother = 1\n").unwrap();
    let scene = config
        .custom_fields
        .get("scene")
        .and_then(|v| v.as_table())
        .unwrap();
    assert_eq!(scene.get("other").and_then(|v| v.as_integer()), Some(1));
    assert!(
        scene.get("alias").is_none(),
        "builtin default must not be filled in"
    );

    let config = PastaConfig::from_str("[scene.alias]\nOnBoot = [\"起動\"]\n").unwrap();
    let scene = config
        .custom_fields
        .get("scene")
        .and_then(|v| v.as_table())
        .unwrap();
    let alias = scene.get("alias").and_then(|v| v.as_table()).unwrap();
    assert!(alias.contains_key("OnBoot"));
    assert!(!alias.contains_key("OnTalk"));
}

/// `alias` がテーブルでない・`scene` がテーブルでないと読み込みエラー（予約キー）。
#[test]
fn scene_alias_or_scene_not_table_is_error() {
    let msg = scene_alias_err("[scene]\nalias = \"x\"\n");
    assert!(msg.contains("line 2"), "{msg}");
    let msg = scene_alias_err("scene = 1\n");
    assert!(msg.contains("line 1"), "{msg}");
}
