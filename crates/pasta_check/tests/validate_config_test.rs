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
        dirs: vec![],
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
        dirs = ["dic"]
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
    assert_eq!(doc.input.dirs, vec!["dic"]);
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
        dirs: vec![],
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

// ---- [actor.*.font] の書き間違い（actor-display-style 7.5） ----

const UNKNOWN_COLOUR: &str = "[actor.\"おじさん\"] の font に知らない属性「colour」があります（書ける属性: color・name・height・bold・italic・underline・strike・sub・sup・outline・shadowcolor・shadowstyle）";
const COLOR_ARRAY: &str = "[actor.\"おじさん\"] の font の color は文字列・整数・真偽値のどれかで、空にできません: 配列が書かれています";
const NAME_BREAKS: &str = "[actor.\"おじさん\"] の font の name に「]」か「\\」が含まれています（さくらスクリプトのタグが壊れます）";

fn messages(found: &[Finding]) -> Vec<&str> {
    found.iter().map(|f| f.message.as_str()).collect()
}

/// 3 つの書き方のどれでも、1 つの誤りに 1 件・属性の値の行で報告する
#[test]
fn actor_font_errors_reported_per_attribute_line_in_three_forms() {
    let cases = [
        (
            "インライン表",
            "[actor.\"おじさん\"]\nspot = 0\nfont = { colour = \"1\", color = [1], name = \"a]\" }\n",
            [3, 3, 3],
        ),
        (
            "節",
            "[actor.\"おじさん\".font]\ncolour = \"1\"\ncolor = [1]\nname = \"a\\\\b\"\n",
            [2, 3, 4],
        ),
        (
            "ドット付きキー",
            "[actor.\"おじさん\"]\nfont.colour = \"1\"\nfont.color = [1]\nfont.name = \"a]\"\n",
            [2, 3, 4],
        ),
    ];
    for (name, text, lines) in cases {
        let found = toml_findings(text);
        assert_eq!(
            codes_lines(&found),
            lines
                .iter()
                .map(|l| (FindingKind::ConfigActorFont, Some(*l)))
                .collect::<Vec<_>>(),
            "{name}: {found:?}"
        );
        let mut got = messages(&found);
        got.sort();
        let mut want = vec![UNKNOWN_COLOUR, COLOR_ARRAY, NAME_BREAKS];
        want.sort();
        assert_eq!(got, want, "{name}");
        assert!(
            found
                .iter()
                .all(|f| f.severity == pasta_check::validate::Severity::Error
                    && f.code.code() == "config-actor-font"),
            "{name}"
        );
    }
}

#[test]
fn actor_font_value_problems_and_not_table() {
    let found = toml_findings(
        "[actor.\"おじさん\".font]\ncolor = \"\"\nheight = 1.5\nbold = { a = 1 }\nitalic = 1979-05-27\n",
    );
    assert_eq!(
        found
            .iter()
            .map(|f| (f.line, f.message.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (
                Some(2),
                "[actor.\"おじさん\"] の font の color は文字列・整数・真偽値のどれかで、空にできません: 空の文字列が書かれています"
            ),
            (
                Some(3),
                "[actor.\"おじさん\"] の font の height は文字列・整数・真偽値のどれかで、空にできません: 浮動小数点数が書かれています"
            ),
            (
                Some(4),
                "[actor.\"おじさん\"] の font の bold は文字列・整数・真偽値のどれかで、空にできません: 表が書かれています"
            ),
            (
                Some(5),
                "[actor.\"おじさん\"] の font の italic は文字列・整数・真偽値のどれかで、空にできません: 日時が書かれています"
            ),
        ]
    );

    // 表でないときは font の値の行
    let found = toml_findings("[actor.\"おじさん\"]\nspot = 0\nfont = \"red\"\n");
    assert_eq!(
        codes_lines(&found),
        vec![(FindingKind::ConfigActorFont, Some(3))]
    );
    assert_eq!(
        found[0].message,
        "[actor.\"おじさん\"] の font は表で書きます（例: font = { color = \"90,110,140\" }）"
    );
}

/// 正しい装い・装いの無いアクターは報告しない。誤りのあるアクターが複数でも全部報告する
#[test]
fn actor_font_correct_or_absent_is_silent_and_all_actors_are_checked() {
    let ok = "[actor.\"ぱすた\"]\nspot = 0\n[actor.\"おじさん\"]\nfont = { color = \"90,110,140\", bold = true, height = -2 }\n[actor.\"ギャル\".font]\n";
    assert!(toml_findings(ok).is_empty());
    assert!(toml_findings("actor = 1\n[ghost]\n").is_empty());

    let found = toml_findings(
        "[actor.\"ギャル\"]\nfont = 1\n[actor.\"おじさん\"]\nfont = { colour = \"1\" }\n",
    );
    assert_eq!(
        codes_lines(&found),
        vec![
            (FindingKind::ConfigActorFont, Some(2)),
            (FindingKind::ConfigActorFont, Some(4)),
        ]
    );
    assert!(
        found[0]
            .message
            .starts_with("[actor.\"ギャル\"] の font は表で書きます")
    );
}

/// 装いの誤りがあっても飛び先の検査は止めない
#[test]
fn actor_font_error_does_not_skip_references() {
    let (mut input, files) = ghost();
    input.pasta_toml = TomlSource::Text {
        text: format!("{TOML}[actor.\"さくら\"]\nfont = \"red\"\n"),
    };
    let report = validate_ghost(&input, &files);
    assert!(report.has_errors());
    assert!(report.stats.skip_reasons.is_empty(), "{:?}", report.lines());
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
        "[actor.\"おじさん\"]\nfont = { color = \"1\", bold = true }\n",
        "[actor.\"おじさん\"]\nfont = { colour = \"1\" }\n",
        "[actor.\"おじさん\".font]\ncolor = []\n",
        "[actor.\"おじさん\"]\nfont.name = \"a]\"\n",
        "[actor.\"おじさん\"]\nfont = \"red\"\n",
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
        dirs: vec![],
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

// ---- 辞書のパターンに一致するフォルダ（2.1〜2.7） ----

fn with_dirs(mut input: GhostInput, dirs: &[&str]) -> GhostInput {
    input.dirs = dirs.iter().map(|d| d.to_string()).collect();
    input
}

/// フォルダの誤りのパスだけ
fn folder_findings(input: &GhostInput) -> Vec<String> {
    validate_ghost(input, &BTreeMap::new())
        .findings
        .into_iter()
        .filter(|f| f.code == FindingKind::FolderMatchesPattern)
        .map(|f| f.path)
        .collect()
}

#[test]
fn folders_matching_patterns_are_errors_and_others_are_not() {
    let input = with_dirs(
        default_input(&["pasta.toml", "dic/x.pasta/inner.pasta", "dic/sub/a.pasta"]),
        &[
            "dic",
            "dic/x.pasta",
            "dic/x.lua",
            "dic/a.pasta",
            "dic/a.pasta/b.pasta",
            "dic/empty.pasta",
            "Dic/Up.pasta",
            "dic/sub",
            "dic/x.txt",
            "dic/X.PASTA",
            "profile",
            "profile/z.pasta",
        ],
    );
    assert_eq!(
        folder_findings(&input),
        vec![
            "Dic/Up.pasta",
            "dic/a.pasta",
            "dic/a.pasta/b.pasta",
            "dic/empty.pasta",
            "dic/x.lua",
            "dic/x.pasta",
        ]
    );
    // 一致したフォルダの中のファイルは今までどおり辞書ファイル（2.4）
    assert_eq!(
        select_dictionary_files(&input),
        vec!["dic/sub/a.pasta", "dic/x.pasta/inner.pasta"]
    );
    let report = validate_ghost(&input, &BTreeMap::new());
    // フォルダは辞書ファイルの数に入らない（2.5）
    assert_eq!(report.stats.dictionary_files, 2);
    // 1 行の形と並び順（2.7）
    let lines = report.lines();
    let at = |p: &str| lines.iter().position(|l| l.starts_with(p)).unwrap();
    assert_eq!(
        lines[at("ghost/master/dic/x.pasta: ")],
        "ghost/master/dic/x.pasta: 誤り: 辞書のパターンに一致するフォルダです。pasta.dll はこのフォルダを辞書として読めず、ゴーストを読み込めません"
    );
    assert!(at("ghost/master/dic/x.lua: ") < at("ghost/master/dic/x.pasta: "));
    assert!(at("ghost/master/dic/x.pasta: ") < at("ghost/master/dic/x.pasta/inner.pasta"));
}

#[test]
fn folder_rules_for_trailing_double_star_and_folder_only_patterns() {
    let dirs = ["dic", "dic/a", "dic/a/b", "other", "profile", "profile/x"];
    let cases: [(&str, &[&str]); 6] = [
        // 最後の ** は 1 段以上（dic そのものには一致しない）
        ("dic/**", &["dic/a", "dic/a/b"]),
        // / で終わる・最後が . のパターンはフォルダだけに一致する
        ("dic/", &["dic"]),
        ("dic/.", &["dic"]),
        ("dic/*", &["dic/a"]),
        // profile は除く
        ("**", &["dic", "dic/a", "dic/a/b", "other"]),
        // 成分 0 個は ghost/master そのもの（パスは空）
        ("", &[""]),
    ];
    for (pattern, expected) in cases {
        let input = with_dirs(input_with(&[pattern], &["pasta.toml"]), &dirs);
        assert_eq!(folder_findings(&input), expected, "{pattern}");
        // ファイルの選び出しはフォルダだけのパターンを使わない（今までどおり）
        assert!(select_dictionary_files(&input).is_empty(), "{pattern}");
    }
    // ghost/master そのものの 1 行
    let input = with_dirs(input_with(&["."], &["pasta.toml"]), &[]);
    let report = validate_ghost(&input, &BTreeMap::new());
    assert!(
        report.lines().contains(&format!(
            "ghost/master/: 誤り: {}",
            pasta_check::validate::msg::folder_matches_pattern()
        )),
        "{:?}",
        report.lines()
    );
}
