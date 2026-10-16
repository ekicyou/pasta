//! 検査の核の入力と、pasta.toml・辞書の集め方のテスト（1.1・1.7・2.x・11.1）

use std::collections::BTreeMap;

use pasta_check::validate::{
    FileText, GhostInput, TomlSource, select_dictionary_files, validate_ghost,
};

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";
const TALK: &str = "＊挨拶\n  さくら：こんにちは。\n";

fn ghost() -> (GhostInput, BTreeMap<String, FileText>) {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: vec!["pasta.toml".into(), "dic/talk.pasta".into()],
    };
    let files = select_dictionary_files(&input)
        .into_iter()
        .map(|p| (p, FileText::Text { text: TALK.into() }))
        .collect();
    (input, files)
}

#[test]
fn correct_ghost_returns_report_without_errors_and_same_report_twice() {
    let (input, files) = ghost();
    let first = validate_ghost(&input, &files);
    let second = validate_ghost(&input, &files);
    assert!(!first.has_errors(), "{:?}", first.lines());
    assert_eq!(first.lines(), second.lines());
    assert_eq!(
        select_dictionary_files(&input),
        select_dictionary_files(&input)
    );
}

/// JS から渡す形: camelCase のキーとタグ `kind`
#[test]
fn input_reads_camel_case_with_kind_tag() {
    #[derive(serde::Deserialize)]
    struct Doc {
        input: GhostInput,
        files: BTreeMap<String, FileText>,
    }
    let doc: Doc = toml::from_str(
        r#"
        [input]
        entries = ["dic/a.pasta"]
        pastaToml = { kind = "unreadable", reason = "denied" }
        [files]
        "dic/a.pasta" = { kind = "text", text = "x" }
        "dic/b.pasta" = { kind = "notUtf8" }
        "dic/c.pasta" = { kind = "unreadable", reason = "busy" }
        "#,
    )
    .unwrap();
    assert!(
        matches!(doc.input.pasta_toml, TomlSource::Unreadable { ref reason } if reason == "denied")
    );
    assert_eq!(doc.input.entries, vec!["dic/a.pasta"]);
    assert!(matches!(&doc.files["dic/a.pasta"], FileText::Text { text } if text == "x"));
    assert!(matches!(doc.files["dic/b.pasta"], FileText::NotUtf8));
    assert!(
        matches!(&doc.files["dic/c.pasta"], FileText::Unreadable { reason } if reason == "busy")
    );

    let missing: TomlSource = toml::from_str(r#"kind = "missing""#).unwrap();
    assert!(matches!(missing, TomlSource::Missing));
}

// ---- pasta.toml の [loader]・[scene]（2.2・2.4・2.5・2.10） ----

use pasta_check::validate::{Finding, FindingKind};

/// pasta.toml についての報告だけを取り出す
fn config_findings(src: TomlSource) -> Vec<Finding> {
    let input = GhostInput {
        pasta_toml: src,
        // 辞書が無いことの注意（no-dictionary-files）を混ぜない
        entries: vec!["dic/a.pasta".into()],
    };
    validate_ghost(&input, &BTreeMap::new())
        .findings
        .into_iter()
        .filter(|f| f.path == "pasta.toml")
        .collect()
}

fn toml_findings(text: &str) -> Vec<Finding> {
    config_findings(TomlSource::Text { text: text.into() })
}

fn codes_lines(found: &[Finding]) -> Vec<(FindingKind, Option<u32>)> {
    found.iter().map(|f| (f.code, f.line)).collect()
}

#[test]
fn toml_syntax_error_has_line_and_column_of_toml() {
    let text = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\nx = = 1\n";
    let found = toml_findings(text);
    assert_eq!(found.len(), 1, "{found:?}");
    let f = &found[0];
    assert_eq!(f.code, FindingKind::ConfigSyntax);
    // toml 自身の説明の行・列と同じ
    let desc = toml::from_str::<toml::Table>(text).unwrap_err().to_string();
    let expected = format!("line {}, column {}", f.line.unwrap(), f.column.unwrap());
    assert!(desc.contains(&expected), "{desc} / {expected}");
    assert_eq!(f.line, Some(3));
    assert!(
        f.message
            .starts_with("pasta.toml を TOML として読めません: ")
    );
}

#[test]
fn loader_key_type_errors_name_the_key_and_line() {
    let text = "[loader]\npasta_patterns = \"x\"\nlua_search_paths = [1]\ntranspiled_output_dir = 3\ndebug_mode = \"yes\"\nunknown = 1\n";
    let found = toml_findings(text);
    assert_eq!(
        codes_lines(&found),
        vec![
            (FindingKind::ConfigLoaderType, Some(2)),
            (FindingKind::ConfigLoaderType, Some(3)),
            (FindingKind::ConfigLoaderType, Some(4)),
            (FindingKind::ConfigLoaderType, Some(5)),
        ]
    );
    for (f, key) in found.iter().zip([
        "pasta_patterns",
        "lua_search_paths",
        "transpiled_output_dir",
        "debug_mode",
    ]) {
        assert!(
            f.message
                .starts_with(&format!("[loader] の {key} の型が違います: ")),
            "{}",
            f.message
        );
    }
}

#[test]
fn loader_not_a_table_is_loader_type_error() {
    let found = toml_findings("loader = 1\n");
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::ConfigLoaderType, Some(1))]
    );
}

#[test]
fn other_sections_and_unknown_keys_are_not_checked() {
    let text = "[loader]\nother = 1\n[ghost]\ntalk_interval_min = \"x\"\n[logging]\nlevel = 3\n[scene]\nfoo = 1\n";
    assert!(toml_findings(text).is_empty());
}

#[test]
fn scene_alias_duplicate_reports_every_alias_line() {
    let text = "[scene.alias]\nOnTalk = [\"会話\"]\nOnBoot = [\"会話\", \"起動\"]\n";
    let found = toml_findings(text);
    assert_eq!(
        codes_lines(&found),
        vec![
            (FindingKind::ConfigSceneAlias, Some(2)),
            (FindingKind::ConfigSceneAlias, Some(3)),
        ]
    );
    // 置き換え先は pasta.dll の順（名前の昇順）
    assert_eq!(
        found[0].message,
        "[scene.alias] の別名「会話」が OnBoot・OnTalk に重複しています"
    );
}

#[test]
fn scene_alias_chain_reports_alias_lines() {
    let text = "[scene.alias]\nOnTalk = [\"会話\"]\n\"会話\" = [\"雑談\"]\n";
    let found = toml_findings(text);
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::ConfigSceneAlias, Some(2))]
    );
    assert!(
        found[0].message.contains("「会話」"),
        "{}",
        found[0].message
    );
}

#[test]
fn scene_alias_empty_name_reports_lines() {
    let found = toml_findings("[scene.alias]\nOnTalk = [\"会話\", \"\"]\n");
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::ConfigSceneAlias, Some(2))]
    );
    assert!(found[0].message.contains("空"), "{}", found[0].message);

    // 置き換え先が空で別名も無いときは行が取れない
    let found = toml_findings("[scene.alias]\n\"\" = []\n");
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::ConfigSceneAlias, None)]
    );
}

#[test]
fn scene_type_errors_are_scene_alias_errors() {
    for (text, line) in [
        ("scene = 1\n", 1),
        ("[scene]\nalias = 1\n", 2),
        ("[scene.alias]\nOnTalk = \"会話\"\n", 2),
        ("[scene.alias]\nOnTalk = [1]\n", 2),
    ] {
        let found = toml_findings(text);
        assert_eq!(
            codes_lines(&found),
            vec![(FindingKind::ConfigSceneAlias, Some(line))],
            "{text}"
        );
        assert!(
            found[0].message.contains("型が違います"),
            "{}",
            found[0].message
        );
    }
}

#[test]
fn missing_toml_is_config_missing_error() {
    let found = config_findings(TomlSource::Missing);
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::ConfigMissing, None)]
    );
    assert_eq!(
        found[0].message,
        pasta_check::validate::msg::config_missing()
    );
}

#[test]
fn unreadable_toml_is_file_unreadable_error() {
    let found = config_findings(TomlSource::Unreadable {
        reason: "denied".into(),
    });
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::FileUnreadable, None)]
    );
    assert_eq!(found[0].message, "ファイルを読めません: denied");
}

/// pasta.dll（PastaConfig）が拒む pasta.toml と、検査が誤りにするものが同じ
#[test]
fn same_strictness_as_pasta_dll() {
    let cases = [
        "",
        "[loader]\npasta_patterns = [\"dic/*.pasta\"]\ndebug_mode = false\n",
        "[loader]\npasta_patterns = \"x\"\n",
        "[loader]\nlua_search_paths = \"x\"\n",
        "[loader]\ntranspiled_output_dir = []\n",
        "[loader]\ndebug_mode = 1\n",
        "loader = 1\n",
        "[loader]\nunknown = 1\n",
        "a = = 1\n",
        "[ghost]\ntalk_interval_min = \"x\"\n",
        "[scene]\nfoo = 1\n",
        "scene = 1\n",
        "[scene]\nalias = 1\n",
        "[scene.alias]\n",
        "[scene.alias]\nOnTalk = [\"会話\", \"雑談\"]\nOnBoot = [\"起動\"]\n",
        "[scene.alias]\nOnTalk = \"会話\"\n",
        "[scene.alias]\nOnTalk = [1]\n",
        "[scene.alias]\nOnTalk = [\"会話\"]\nOnBoot = [\"会話\"]\n",
        "[scene.alias]\nOnTalk = [\"会話\"]\n\"会話\" = [\"雑談\"]\n",
        "[scene.alias]\nOnTalk = [\"\"]\n",
        "[scene.alias]\n\"\" = [\"a\"]\n",
    ];
    for text in cases {
        let dll_rejects = pasta_lua::PastaConfig::from_str(text).is_err();
        let found = toml_findings(text);
        assert_eq!(dll_rejects, !found.is_empty(), "{text:?}: {found:?}");
        assert!(
            found
                .iter()
                .all(|f| f.severity == pasta_check::validate::Severity::Error)
        );
    }
}

// ---- 辞書ファイルと辞書モジュールの選び出し（2.1・2.3・2.6・2.7・2.8・11.8） ----

use pasta_check::validate::pasta_dll_paths;

fn input_with(patterns: &[&str], entries: &[&str]) -> GhostInput {
    let list = patterns
        .iter()
        .map(|p| format!("{p:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    GhostInput {
        pasta_toml: TomlSource::Text {
            text: format!("[loader]\npasta_patterns = [{list}]\n"),
        },
        entries: entries.iter().map(|e| e.to_string()).collect(),
    }
}

fn default_input(entries: &[&str]) -> GhostInput {
    input_with(&["dic/**/*.pasta"], entries)
}

/// 選び出しの段の報告だけ（code・パス）
fn discovery_findings(input: &GhostInput) -> Vec<(FindingKind, String)> {
    validate_ghost(input, &BTreeMap::new())
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::ConfigPatternInvalid
                    | FindingKind::PatternIgnored
                    | FindingKind::PatternNotPasta
                    | FindingKind::NoDictionaryFiles
                    | FindingKind::InitFileName
            )
        })
        .map(|f| (f.code, f.path))
        .collect()
}

#[test]
fn literal_component_ignores_case_and_dll_path_uses_pattern_spelling() {
    let input = default_input(&["Dic/talk.pasta", "DIC/sub/a.lua", "pasta.toml"]);
    // 報告のパスは実物の綴り
    assert_eq!(select_dictionary_files(&input), vec!["Dic/talk.pasta"]);
    // pasta.dll が見るパスは、ワイルドカードの無い成分がパターンの綴り
    let (pasta, lua) = pasta_dll_paths(&input);
    assert_eq!(pasta, vec!["dic/talk.pasta"]);
    assert_eq!(lua, vec!["dic/sub/a.lua"]);
}

#[test]
fn wildcard_component_is_case_sensitive() {
    let input = default_input(&["dic/X.PASTA", "dic/y.pasta"]);
    assert_eq!(select_dictionary_files(&input), vec!["dic/y.pasta"]);
    let input = input_with(&["d?c/*.pasta"], &["Dic/a.pasta", "dic/b.pasta"]);
    assert_eq!(select_dictionary_files(&input), vec!["dic/b.pasta"]);
}

#[test]
fn double_star_matches_zero_or_more_components() {
    let input = default_input(&[
        "dic/a.pasta",
        "dic/x/b.pasta",
        "dic/x/y/c.pasta",
        "other/d.pasta",
        "dic.pasta",
    ]);
    assert_eq!(
        select_dictionary_files(&input),
        vec!["dic/a.pasta", "dic/x/b.pasta", "dic/x/y/c.pasta"]
    );
    // 最後の ** はフォルダにしか一致しない
    let input = input_with(&["dic/**"], &["dic/a.pasta"]);
    assert!(select_dictionary_files(&input).is_empty());
}

#[test]
fn profile_is_excluded_and_duplicates_merged_in_ascending_order() {
    let input = input_with(
        &["**/*.pasta", "dic/*.pasta"],
        &["profile/p.pasta", "dic/b.pasta", "a.pasta", "dic/a.pasta"],
    );
    assert_eq!(
        select_dictionary_files(&input),
        vec!["a.pasta", "dic/a.pasta", "dic/b.pasta"]
    );
}

#[test]
fn lua_modules_are_collected_from_pasta_patterns() {
    let input = default_input(&["dic/a.pasta", "dic/m.lua", "profile/x.lua", "m.lua"]);
    let (pasta, lua) = pasta_dll_paths(&input);
    assert_eq!(pasta, vec!["dic/a.pasta"]);
    assert_eq!(lua, vec!["dic/m.lua"]);
    // .lua は辞書ファイルとしては数えない
    assert_eq!(select_dictionary_files(&input), vec!["dic/a.pasta"]);
}

#[test]
fn traversal_and_root_patterns_are_ignored_with_warning() {
    for pattern in [
        "../x/*.pasta",
        "dic/../*.pasta",
        "/abs/*.pasta",
        "C:/x/*.pasta",
        "dic\\..\\*.pasta",
    ] {
        let input = input_with(&[pattern, "dic/*.pasta"], &["dic/a.pasta", "x/b.pasta"]);
        assert_eq!(
            discovery_findings(&input),
            vec![(FindingKind::PatternIgnored, "pasta.toml".to_string())],
            "{pattern}"
        );
        assert_eq!(
            select_dictionary_files(&input),
            vec!["dic/a.pasta"],
            "{pattern}"
        );
    }
    let input = input_with(&["../x/*.pasta"], &[]);
    let found = validate_ghost(&input, &BTreeMap::new());
    let f = found
        .findings
        .iter()
        .find(|f| f.code == FindingKind::PatternIgnored)
        .unwrap();
    assert_eq!(
        f.message,
        pasta_check::validate::msg::pattern_ignored("../x/*.pasta")
    );
}

#[test]
fn non_pasta_pattern_searches_pasta_but_not_lua() {
    let input = input_with(&["dic/*.txt"], &["dic/a.txt", "dic/a.lua"]);
    assert_eq!(
        discovery_findings(&input),
        vec![(FindingKind::PatternNotPasta, "pasta.toml".to_string())]
    );
    assert_eq!(select_dictionary_files(&input), vec!["dic/a.txt"]);
    assert_eq!(pasta_dll_paths(&input).1, Vec::<String>::new());
}

#[test]
fn unreadable_pattern_is_config_pattern_invalid_error() {
    let input = input_with(&["dic/[*.pasta", "dic/*.pasta"], &["dic/a.pasta"]);
    let report = validate_ghost(&input, &BTreeMap::new());
    let f = report
        .findings
        .iter()
        .find(|f| f.code == FindingKind::ConfigPatternInvalid)
        .expect("config-pattern-invalid");
    assert_eq!(f.path, "pasta.toml");
    assert_eq!(f.severity, pasta_check::validate::Severity::Error);
    assert!(
        f.message
            .starts_with("pasta_patterns の「dic/[*.pasta」はパターンとして読めません: "),
        "{}",
        f.message
    );
    // 読めるパターンでは集め続ける
    assert_eq!(select_dictionary_files(&input), vec!["dic/a.pasta"]);
}

#[test]
fn init_file_names_are_errors() {
    let input = default_input(&[
        "dic/init.pasta",
        "dic/sub/init.lua",
        "dic/a.pasta",
        "dic/init.txt",
    ]);
    assert_eq!(
        discovery_findings(&input),
        vec![
            (FindingKind::InitFileName, "dic/init.pasta".to_string()),
            (FindingKind::InitFileName, "dic/sub/init.lua".to_string()),
        ]
    );
    // 名前はパターンの綴り（pasta.dll が見るパス）で判定し、報告は実物の綴り
    let input = input_with(&["dic/init.pasta"], &["dic/INIT.pasta"]);
    assert_eq!(
        discovery_findings(&input),
        vec![(FindingKind::InitFileName, "dic/INIT.pasta".to_string())]
    );
}

#[test]
fn no_dictionary_files_is_warning_on_pasta_toml() {
    let input = default_input(&["pasta.toml", "dic/X.PASTA", "profile/dic/a.pasta"]);
    assert_eq!(
        discovery_findings(&input),
        vec![(FindingKind::NoDictionaryFiles, "pasta.toml".to_string())]
    );
    let report = validate_ghost(&input, &BTreeMap::new());
    assert_eq!(report.stats.dictionary_files, 0);
    assert!(!report.has_errors(), "{:?}", report.lines());
}
