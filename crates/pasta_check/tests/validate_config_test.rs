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
