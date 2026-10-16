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
        entries: vec![],
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
