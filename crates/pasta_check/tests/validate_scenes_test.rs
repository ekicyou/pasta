//! シーンの登録と名前の重なり・組み込みの名前のテスト（2.4・5.1〜5.4）

use std::collections::BTreeMap;

use pasta_check::validate::{
    ACT_METHODS, FileText, FindingKind, GLOBAL_NAMES, GhostInput, SHIORI_ACT_METHODS, Severity,
    TomlSource, validate_ghost,
};

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";

/// (パス, 中身) の辞書でゴーストを検査し、シーンの問題だけを (パス, 行, 種類, 理由) で返す
fn scene_findings(toml: &str, dics: &[(&str, &str)]) -> Vec<(String, u32, FindingKind, String)> {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: toml.into() },
        entries: dics.iter().map(|(p, _)| p.to_string()).collect(),
    };
    let files: BTreeMap<_, _> = dics
        .iter()
        .map(|(p, t)| {
            (
                p.to_string(),
                FileText::Text {
                    text: t.to_string(),
                },
            )
        })
        .collect();
    validate_ghost(&input, &files)
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::GlobalSceneCollision
                    | FindingKind::LocalSceneCollision
                    | FindingKind::SceneShadowedByBuiltin
            )
        })
        .map(|f| (f.path, f.line.unwrap(), f.code, f.message))
        .collect()
}

#[test]
fn global_match_name_collision_is_warning_on_each_declaration() {
    let found = scene_findings(
        TOML,
        &[(
            "dic/talk.pasta",
            "＊会話·A\n  さくら：あ。\n＊会話_A\n  さくら：い。\n",
        )],
    );
    assert_eq!(
        found,
        vec![
            (
                "dic/talk.pasta".into(),
                1,
                FindingKind::GlobalSceneCollision,
                "シーン「会話·A」は照合用の名前「会話_A」が「会話_A」（ghost/master/dic/talk.pasta:3）と同じになり、同じ名前のシーンとして選ばれます".into()
            ),
            (
                "dic/talk.pasta".into(),
                3,
                FindingKind::GlobalSceneCollision,
                "シーン「会話_A」は照合用の名前「会話_A」が「会話·A」（ghost/master/dic/talk.pasta:1）と同じになり、同じ名前のシーンとして選ばれます".into()
            ),
        ]
    );
    assert_eq!(
        FindingKind::GlobalSceneCollision.severity(),
        Severity::Warning
    );
}

#[test]
fn global_collision_across_files_shows_other_file() {
    let found = scene_findings(
        TOML,
        &[
            ("dic/a.pasta", "＊会話·A\n  さくら：あ。\n"),
            ("dic/b.pasta", "\n＊会話_A\n  さくら：い。\n"),
        ],
    );
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!((found[0].0.as_str(), found[0].1), ("dic/a.pasta", 1));
    assert!(
        found[0].3.contains("（ghost/master/dic/b.pasta:2）"),
        "{found:?}"
    );
    assert_eq!((found[1].0.as_str(), found[1].1), ("dic/b.pasta", 2));
    assert!(
        found[1].3.contains("（ghost/master/dic/a.pasta:1）"),
        "{found:?}"
    );
}

#[test]
fn same_name_alias_and_prefix_are_not_reported() {
    // ＊会話 は既定の別名で OnTalk に置き換わる。同名・前方一致も報告しない（5.3）
    let found = scene_findings(
        TOML,
        &[
            (
                "dic/a.pasta",
                "＊会話\n  さくら：あ。\n＊OnTalk\n  さくら：い。\n＊挨拶\n  さくら：う。\n",
            ),
            (
                "dic/b.pasta",
                "＊挨拶\n  さくら：え。\n＊挨拶朝\n  さくら：お。\n",
            ),
        ],
    );
    assert_eq!(found, vec![]);
}

#[test]
fn local_match_name_collision_is_warning_within_one_global_scene() {
    let found = scene_findings(
        TOML,
        &[(
            "dic/talk.pasta",
            "＊会話\n  ・選択·A\n  さくら：あ。\n  ・選択_A\n  さくら：い。\n  ・選択_A\n  さくら：う。\n＊雑談\n  ・選択·A\n  さくら：え。\n",
        )],
    );
    let lines: Vec<(u32, FindingKind)> = found.iter().map(|f| (f.1, f.2)).collect();
    assert_eq!(
        lines,
        vec![
            (2, FindingKind::LocalSceneCollision),
            (4, FindingKind::LocalSceneCollision),
            (6, FindingKind::LocalSceneCollision),
        ],
        "{found:?}"
    );
    assert!(
        found[0].3.starts_with(
            "シーン「選択·A」は照合用の名前「選択_A」が「選択_A」（ghost/master/dic/talk.pasta:4）"
        ),
        "{found:?}"
    );
    assert!(
        found[1]
            .3
            .contains("「選択·A」（ghost/master/dic/talk.pasta:2）"),
        "{found:?}"
    );
    assert_eq!(
        FindingKind::LocalSceneCollision.severity(),
        Severity::Warning
    );
}

#[test]
fn local_collision_within_continuation_scene_is_warning() {
    // 単独の `＊` も 1 つのグローバルシーンで、その中のローカルシーンは重なりを報告する（5.2）
    let found = scene_findings(
        TOML,
        &[(
            "dic/talk.pasta",
            "＊会話\n  さくら：あ。\n＊\n  ・選択·A\n  さくら：い。\n  ・選択_A\n  さくら：う。\n",
        )],
    );
    let lines: Vec<(u32, FindingKind)> = found.iter().map(|f| (f.1, f.2)).collect();
    assert_eq!(
        lines,
        vec![
            (4, FindingKind::LocalSceneCollision),
            (6, FindingKind::LocalSceneCollision),
        ],
        "{found:?}"
    );
}

#[test]
fn scene_named_as_builtin_is_warning() {
    let found = scene_findings(
        TOML,
        &[(
            "dic/talk.pasta",
            "＊yield\n  さくら：あ。\n＊transfer_req_to_var\n  さくら：い。\n＊yield2\n  さくら：う。\n",
        )],
    );
    assert_eq!(
        found,
        vec![
            (
                "dic/talk.pasta".into(),
                1,
                FindingKind::SceneShadowedByBuiltin,
                "シーン「yield」は組み込みの飛び先と同じ名前のため、＞yield では呼ばれません".into()
            ),
            (
                "dic/talk.pasta".into(),
                3,
                FindingKind::SceneShadowedByBuiltin,
                "シーン「transfer_req_to_var」は組み込みの飛び先と同じ名前のため、＞transfer_req_to_var では呼ばれません".into()
            ),
        ]
    );
    assert_eq!(
        FindingKind::SceneShadowedByBuiltin.severity(),
        Severity::Warning
    );
}

#[test]
fn builtin_check_uses_name_after_alias() {
    let toml =
        "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n[scene.alias]\nyield = [\"譲る\"]\n";
    let found = scene_findings(toml, &[("dic/talk.pasta", "＊譲る\n  さくら：あ。\n")]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].2, FindingKind::SceneShadowedByBuiltin);
}

#[test]
fn file_with_syntax_error_registers_no_scenes() {
    let dics = [
        ("dic/a.pasta", "＊会話·A\n  さくら：あ。\n"),
        ("dic/b.pasta", "＊会話_A\n  さくら：い。\n＄＄＄\n"),
    ];
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: dics.iter().map(|(p, _)| p.to_string()).collect(),
    };
    let files: BTreeMap<_, _> = dics
        .iter()
        .map(|(p, t)| {
            (
                p.to_string(),
                FileText::Text {
                    text: t.to_string(),
                },
            )
        })
        .collect();
    let report = validate_ghost(&input, &files);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == FindingKind::Syntax && f.path == "dic/b.pasta"),
        "{:?}",
        report.lines()
    );
    assert_eq!(scene_findings(TOML, &dics), vec![]);
}

#[test]
fn builtin_names_are_listed() {
    assert_eq!(
        GLOBAL_NAMES,
        ["yield", "チェイントーク", "close_ghost", "ゴースト終了"]
    );
    assert_eq!(ACT_METHODS.len(), 27);
    assert!(ACT_METHODS.contains(&"call_restore"));
    assert!(!ACT_METHODS.contains(&"__index"));
    assert_eq!(
        SHIORI_ACT_METHODS,
        [
            "build",
            "transfer_date_to_var",
            "transfer_req_to_var",
            "set_property",
            "get_property"
        ]
    );
}
