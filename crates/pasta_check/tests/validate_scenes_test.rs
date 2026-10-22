//! シーンの登録と名前の重なり・組み込みの名前のテスト（2.4・5.1〜5.4）

use std::collections::BTreeMap;

use pasta_check::validate::{
    ACT_METHODS, FileText, FindingKind, GLOBAL_NAMES, GhostInput, Report, SHIORI_ACT_METHODS,
    Severity, SkipReason, TomlSource, validate_ghost,
};

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";

/// (パス, 中身) の辞書でゴーストを検査し、シーンの問題だけを (パス, 行, 種類, 理由) で返す
fn scene_findings(toml: &str, dics: &[(&str, &str)]) -> Vec<(String, u32, FindingKind, String)> {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: toml.into() },
        entries: dics.iter().map(|(p, _)| p.to_string()).collect(),
        dirs: vec![],
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
        dirs: vec![],
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
        [
            "yield",
            "チェイントーク",
            "close_ghost",
            "ゴースト終了",
            "pause_auto_talk",
            "自動トーク停止",
            "resume_auto_talk",
            "自動トーク再開"
        ]
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

// ---- 飛び先（6.1〜6.9） ----

fn report_of(toml: &str, dics: &[(&str, &str)]) -> pasta_check::validate::Report {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: toml.into() },
        entries: dics.iter().map(|(p, _)| p.to_string()).collect(),
        dirs: vec![],
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
}

/// 飛び先の注意だけを (行, 種類, 理由) で返す
fn reference_findings(toml: &str, text: &str) -> Vec<(u32, FindingKind, String)> {
    report_of(toml, &[("dic/talk.pasta", text)])
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::CallTargetMissing | FindingKind::ChoiceTargetMissing
            )
        })
        .map(|f| (f.line.unwrap(), f.code, f.message))
        .collect()
}

#[test]
fn call_finds_local_scene_of_own_global_scene_only() {
    let text = "＊会話\n  ＞選\n  ・選択\n    さくら：あ。\n＊別\n  ＞選\n";
    assert_eq!(
        reference_findings(TOML, text),
        vec![(
            6,
            FindingKind::CallTargetMissing,
            "Call の飛び先「選」が辞書のシーンにありません（Lua で定義していれば問題ありません）"
                .into()
        )]
    );
}

#[test]
fn call_finds_global_scene_by_prefix_after_alias() {
    let toml = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n[scene.alias]\n\"挨拶\" =[\"こんにちは\"]\n";
    let text = "＊挨拶\n  さくら：あ。\n＊会話\n  ＞こんにちは\n  ＞挨\n  ＞「挨拶」\n";
    assert_eq!(reference_findings(toml, text), vec![]);
    // 別名が無ければ「こんにちは」は見つからない
    let found = reference_findings(TOML, text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, 4);
}

#[test]
fn call_to_builtin_name_is_found() {
    let text = "＊会話\n  ＞transfer_req_to_var\n  ＞yield\n  ＞ゴースト終了\n  ＞自動トーク停止\n  ＞resume_auto_talk\n";
    assert_eq!(reference_findings(TOML, text), vec![]);
}

/// 自動トークの停止・再開も組み込みは完全一致だけ。途中まで・全角英字は見つからない（6.2）
#[test]
fn call_to_partial_or_fullwidth_builtin_name_is_missing() {
    let text = "＊会話\n  ＞自動トーク\n  ＞ｐａｕｓｅ＿ａｕｔｏ＿ｔａｌｋ\n";
    let missing = |line, name: &str| {
        (
            line,
            FindingKind::CallTargetMissing,
            format!(
                "Call の飛び先「{name}」が辞書のシーンにありません（Lua で定義していれば問題ありません）"
            ),
        )
    };
    assert_eq!(
        reference_findings(TOML, text),
        vec![
            missing(2, "自動トーク"),
            missing(3, "ｐａｕｓｅ＿ａｕｔｏ＿ｔａｌｋ")
        ]
    );
}

#[test]
fn string_literal_dynamic_call_is_checked_as_static() {
    let text = "＊会話\n  ＞「無いシーン」\n";
    assert_eq!(
        reference_findings(TOML, text),
        vec![(
            2,
            FindingKind::CallTargetMissing,
            "Call の飛び先「無いシーン」が辞書のシーンにありません（Lua で定義していれば問題ありません）"
                .into()
        )]
    );
}

/// 括弧で囲んだ文字列リテラルも実行時は `act:call_key("…")` になるので静的に探し、動的に数えない（6.2）
#[test]
fn parenthesized_string_literal_call_is_checked_as_static() {
    let text = "＊会話\n  ＞（「無い」）\n  ＞（（「会話」））\n";
    let report = report_of(TOML, &[("dic/talk.pasta", text)]);
    assert_eq!(
        reference_findings(TOML, text),
        vec![(
            2,
            FindingKind::CallTargetMissing,
            "Call の飛び先「無い」が辞書のシーンにありません（Lua で定義していれば問題ありません）"
                .into()
        )]
    );
    assert_eq!(report.stats.unchecked_dynamic_calls, 0);
}

#[test]
fn choice_finds_local_then_global_and_skips_on() {
    // `＠？script:…` は DSL の id に `:` が書けず構文エラーになるので、references.rs の単体テストで確かめる
    let text = "＊会話\n  ＠？選択\n  ＠？挨\n  ＠？OnBoot\n  ＠？無い\n  ・選択\n    さくら：あ。\n＊挨拶\n  さくら：い。\n";
    assert_eq!(
        reference_findings(TOML, text),
        vec![(
            5,
            FindingKind::ChoiceTargetMissing,
            "選択肢のジャンプ先「無い」が辞書のシーンにありません（Lua で定義・振り分けていれば問題ありません）"
                .into()
        )]
    );
}

/// 選択肢の列は字下げの後ろの `＠？` の位置。Call と同じく行の頭ではない（7.1）
#[test]
fn choice_column_is_at_choice_marker_like_call() {
    let text = "＊会話\n  ＞無いシーン\n  ＠？無い選択\n\u{3000}＠？無い選択２\n";
    let got: Vec<_> = report_of(TOML, &[("dic/talk.pasta", text)])
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::CallTargetMissing | FindingKind::ChoiceTargetMissing
            )
        })
        .map(|f| (f.line, f.column, f.code))
        .collect();
    assert_eq!(
        got,
        vec![
            (Some(2), Some(3), FindingKind::CallTargetMissing),
            (Some(3), Some(3), FindingKind::ChoiceTargetMissing),
            (Some(4), Some(2), FindingKind::ChoiceTargetMissing),
        ]
    );
}

#[test]
fn choice_does_not_use_builtin_names() {
    let text = "＊会話\n  ＠？yield\n";
    let found = reference_findings(TOML, text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].1, FindingKind::ChoiceTargetMissing);
}

#[test]
fn dynamic_calls_are_counted_not_reported() {
    let text = "＊会話\n  ＄変数＝「会話」\n  ＞＄変数\n  ＞＄変数\n  ＞会話\n";
    let report = report_of(TOML, &[("dic/talk.pasta", text)]);
    assert_eq!(reference_findings(TOML, text), vec![]);
    assert_eq!(report.stats.unchecked_dynamic_calls, 2);
    assert!(report.stats.skip_reasons.is_empty());
    assert_eq!(
        report.lines().last().unwrap(),
        "検査した辞書ファイル 1 個: 誤り 0 件、注意 0 件、飛び先を検査しなかった Call 2 件"
    );
}

#[test]
fn syntax_error_skips_references_with_notice() {
    let dics = [
        ("dic/a.pasta", "＊会話\n  ＞無いシーン\n  ＞＄変数\n"),
        ("dic/b.pasta", "＊別\n  さくら：い。\n＄＄＄\n"),
    ];
    let report = report_of(TOML, &dics);
    assert_eq!(report.stats.skip_reasons, [SkipReason::Syntax]);
    assert_eq!(report.stats.unchecked_dynamic_calls, 0);
    assert!(
        !report.findings.iter().any(|f| matches!(
            f.code,
            FindingKind::CallTargetMissing | FindingKind::ChoiceTargetMissing
        )),
        "{:?}",
        report.lines()
    );
    let lines = report.lines();
    assert!(
        lines.contains(&"構文エラーがあるため、飛び先の検査をしていません。".to_string()),
        "{lines:?}"
    );
    assert!(!lines.last().unwrap().contains("飛び先を検査しなかった"));
}

/// 無い飛び先への Call・選択肢を持つ辞書
const MISSING_TARGETS: &str = "＊会話\n  ＞無いシーン\n  ＠？無い選択\n";

/// (パス, 中身) の辞書
type Dics = Vec<(&'static str, FileText)>;

/// pasta.toml の中身と (パス, 中身) の辞書でゴーストを検査する
fn report_with(toml: TomlSource, dics: Dics) -> Report {
    let input = GhostInput {
        pasta_toml: toml,
        entries: dics.iter().map(|(p, _)| p.to_string()).collect(),
        dirs: vec![],
    };
    let files = dics.into_iter().map(|(p, t)| (p.to_string(), t)).collect();
    validate_ghost(&input, &files)
}

fn text(t: &str) -> FileText {
    FileText::Text { text: t.into() }
}

fn toml(t: &str) -> TomlSource {
    TomlSource::Text { text: t.into() }
}

fn missing_targets(report: &Report) -> usize {
    report
        .findings
        .iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::CallTargetMissing | FindingKind::ChoiceTargetMissing
            )
        })
        .count()
}

/// 辞書の中身・集め方・別名表が分からないときは飛び先の検査を丸ごと省き、理由は当たるものだけ（1.1〜1.3・6.1）
#[test]
fn unknown_dictionary_contents_skip_references_with_matching_reasons() {
    use SkipReason::*;
    let a = || ("dic/a.pasta", text(MISSING_TARGETS));
    let bad_syntax = || ("dic/b.pasta", text("＊別\n＄＄＄\n"));
    let unreadable = FileText::Unreadable {
        reason: "denied".into(),
    };
    let cases: Vec<(&str, TomlSource, Dics, Vec<SkipReason>)> = vec![
        (
            "読めない辞書",
            toml(TOML),
            vec![a(), ("dic/b.pasta", unreadable.clone())],
            vec![UnreadableFile],
        ),
        (
            "UTF-8 で読めない辞書",
            toml(TOML),
            vec![a(), ("dic/b.pasta", FileText::NotUtf8)],
            vec![UnreadableFile],
        ),
        (
            "pasta.toml を読めない",
            TomlSource::Unreadable {
                reason: "denied".into(),
            },
            vec![a()],
            vec![Config],
        ),
        ("TOML の誤り", toml("[loader\n"), vec![a()], vec![Config]),
        (
            "[loader] の型",
            toml("[loader]\npasta_patterns = \"x\"\n"),
            vec![a()],
            vec![Config],
        ),
        (
            "読めないパターン",
            toml("[loader]\npasta_patterns = [\"dic/[*.pasta\", \"dic/*.pasta\"]\n"),
            vec![a()],
            vec![Config],
        ),
        (
            "[scene.alias] の誤り",
            toml("[scene.alias]\nOnTalk = [\"会話\", \"\"]\n"),
            vec![a()],
            vec![Config],
        ),
        (
            "構文エラー",
            toml(TOML),
            vec![a(), bad_syntax()],
            vec![Syntax],
        ),
        (
            "構文エラーと pasta.toml の誤り",
            toml("[scene.alias]\nOnTalk = [\"\"]\n"),
            vec![a(), bad_syntax()],
            vec![Syntax, Config],
        ),
        (
            "3 つとも",
            toml("[loader\n"),
            vec![a(), bad_syntax(), ("dic/c.pasta", FileText::NotUtf8)],
            vec![Syntax, UnreadableFile, Config],
        ),
    ];
    for (name, toml, dics, reasons) in cases {
        let report = report_with(toml, dics);
        let lines = report.lines();
        assert_eq!(report.stats.skip_reasons, reasons, "{name}: {lines:?}");
        assert_eq!(missing_targets(&report), 0, "{name}: {lines:?}");
        assert_eq!(report.stats.unchecked_dynamic_calls, 0, "{name}");
        // 原因そのものは今までどおり誤りとして報告する（1.7）
        assert!(report.has_errors(), "{name}");
        assert!(
            !lines.last().unwrap().contains("飛び先を検査しなかった"),
            "{name}: {lines:?}"
        );
    }
}

/// 省く条件に当たらない誤りだけなら飛び先を検査する（1.6・6.2）
#[test]
fn other_errors_still_check_references() {
    let a = || ("dic/a.pasta", text(MISSING_TARGETS));
    let cases: Vec<(&str, TomlSource, Dics, FindingKind)> = vec![
        (
            "pasta.toml が無い",
            TomlSource::Missing,
            vec![a()],
            FindingKind::ConfigMissing,
        ),
        (
            "init.pasta",
            toml(TOML),
            vec![a(), ("dic/init.pasta", text("＊別\n  さくら：あ。\n"))],
            FindingKind::InitFileName,
        ),
        (
            "モジュール名の重なり",
            toml(TOML),
            vec![
                ("dic/a-b.pasta", text(MISSING_TARGETS)),
                ("dic/a_b.pasta", text("＊別\n  さくら：あ。\n")),
            ],
            FindingKind::ModulePastaCollision,
        ),
        (
            "トランスパイルの誤り",
            toml(TOML),
            vec![
                a(),
                (
                    "dic/b.pasta",
                    text("＊別\n  さくら：あ。\n  ＄x＝＄％p＋1\n"),
                ),
            ],
            FindingKind::TranspilePropertyInExpr,
        ),
    ];
    for (name, toml, dics, kind) in cases {
        let report = report_with(toml, dics);
        let lines = report.lines();
        assert!(
            report.findings.iter().any(|f| f.code == kind),
            "{name}: {lines:?}"
        );
        assert!(report.stats.skip_reasons.is_empty(), "{name}: {lines:?}");
        assert_eq!(missing_targets(&report), 2, "{name}: {lines:?}");
    }
}

/// フォルダの誤りだけなら飛び先を検査する（1.6・6.2）
#[test]
fn folder_error_alone_still_checks_references() {
    let input = GhostInput {
        pasta_toml: toml(TOML),
        entries: vec!["dic/a.pasta".into()],
        dirs: vec!["dic".into(), "dic/x.pasta".into()],
    };
    let files = BTreeMap::from([("dic/a.pasta".to_string(), text(MISSING_TARGETS))]);
    let report = validate_ghost(&input, &files);
    let lines = report.lines();
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == FindingKind::FolderMatchesPattern && f.path == "dic/x.pasta"),
        "{lines:?}"
    );
    assert!(report.stats.skip_reasons.is_empty(), "{lines:?}");
    assert_eq!(missing_targets(&report), 2, "{lines:?}");
}
