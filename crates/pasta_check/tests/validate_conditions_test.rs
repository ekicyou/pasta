//! 条件の保存変数の未代入と、優先で塞がれた場面の注意のテスト（8.1〜8.5）

use std::collections::BTreeMap;

use pasta_check::validate::{
    FileText, FindingKind, GhostInput, Report, Severity, TomlSource, validate_ghost,
};

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";

fn validate(dics: &[(&str, FileText)]) -> Report {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: dics.iter().map(|(p, _)| p.to_string()).collect(),
        dirs: vec![],
    };
    let files: BTreeMap<_, _> = dics
        .iter()
        .map(|(p, t)| (p.to_string(), t.clone()))
        .collect();
    validate_ghost(&input, &files)
}

fn text(t: &str) -> FileText {
    FileText::Text { text: t.into() }
}

/// 2 つの注意だけを (パス, 行, 種類, 文) で返す
fn condition_findings(report: &Report) -> Vec<(String, u32, FindingKind, String)> {
    report
        .findings
        .iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::CondVarUnassigned | FindingKind::SceneShadowedByPriority
            )
        })
        .map(|f| (f.path.clone(), f.line.unwrap(), f.code, f.message.clone()))
        .collect()
}

fn unassigned(path: &str, line: u32, name: &str) -> (String, u32, FindingKind, String) {
    (
        path.into(),
        line,
        FindingKind::CondVarUnassigned,
        format!(
            "条件の変数 ＄＊{name} に辞書のどこでも代入していません（Lua で代入していれば問題ありません）"
        ),
    )
}

#[test]
fn unassigned_save_var_in_condition_is_warned_where_written() {
    let report = validate(&[
        (
            "dic/a.pasta",
            text(concat!(
                "＆＄＊初回\n",
                "＊雑談＆＄＊経験値≧500＆＄気分＝「高」\n",
                "　Alice：やあ\n",
                "　・選択肢＆＄＊未定＆＄＊書き間違い＞1\n",
                "　Alice：ほい\n",
                "＊雑談\n",
                "　Alice：よう\n",
            )),
        ),
        (
            "dic/b.pasta",
            text(concat!(
                "％Alice\n",
                "　＄＊未定＝1\n",
                "＊別\n",
                "　＄＊経験値＝10\n",
                "　＄気分＝「高」\n",
                "　Alice：あ\n",
            )),
        ),
    ]);
    // ファイル属性の ＄＊初回 は 2 つのシーンが受け継いでも、ファイル属性の行に 1 回。
    // 代入のある ＄＊経験値（シーンの項目）・＄＊未定（アクター定義）と、ローカル変数の ＄気分 は出ない
    assert_eq!(
        condition_findings(&report),
        vec![
            unassigned("dic/a.pasta", 1, "初回"),
            unassigned("dic/a.pasta", 4, "書き間違い"),
        ]
    );
    assert_eq!(FindingKind::CondVarUnassigned.severity(), Severity::Warning);
    // 注意では終了の状態を変えない（8.2）
    assert!(!report.has_errors(), "{:?}", report.findings);
}

#[test]
fn unassigned_warning_column_points_at_the_condition() {
    let report = validate(&[("dic/a.pasta", text("＊雑談＆＄＊無い\n　Alice：やあ\n"))]);
    let f = report
        .findings
        .iter()
        .find(|f| f.code == FindingKind::CondVarUnassigned)
        .expect("注意が出る");
    assert_eq!((f.line, f.column), (Some(1), Some(4)));
}

fn shadowed(
    path: &str,
    line: u32,
    name: &str,
    priority: i64,
    blocker_line: u32,
) -> (String, u32, FindingKind, String) {
    (
        path.into(),
        line,
        FindingKind::SceneShadowedByPriority,
        format!(
            "シーン「{name}」は、条件の無い ＆優先：{priority} の同じ名前のシーン（ghost/master/dic/a.pasta:{blocker_line}）があるため選ばれません"
        ),
    )
}

#[test]
fn scenes_below_an_unconditional_priority_are_warned() {
    let report = validate(&[(
        "dic/a.pasta",
        text(concat!(
            "＊雑談＆優先：2\n",        // 1 塞ぐ
            "　Alice：あ\n",            // 2
            "＊雑談＆優先：5＆＄＊x\n", // 3 条件つきなので塞がない・自分は 2 より上
            "　＄＊x＝1\n",             // 4
            "＊雑談\n",                 // 5 優先 0 → 塞がれる
            "　Alice：う\n",            // 6
            "＊雑談＆優先：2\n",        // 7 同じ優先は塞がれない
            "　Alice：え\n",            // 8
            "＊雑談話\n",               // 9 照合用の名前が違う
            "　Alice：お\n",            // 10
            "＊会話\n",                 // 11
            "　Alice：か\n",            // 12
            "　・返事＆priority：1\n",  // 13 ローカルどうしで塞ぐ
            "　Alice：き\n",            // 14
            "　・返事\n",               // 15 塞がれる
            "　Alice：く\n",            // 16
            "＊会話\n",                 // 17 親が別なので塞がれない
            "　Alice：け\n",            // 18
            "　・返事\n",               // 19
            "　Alice：こ\n",            // 20
        )),
    )]);
    assert_eq!(
        condition_findings(&report),
        vec![
            shadowed("dic/a.pasta", 5, "雑談", 2, 1),
            shadowed("dic/a.pasta", 15, "返事", 1, 13),
        ]
    );
    assert_eq!(
        FindingKind::SceneShadowedByPriority.severity(),
        Severity::Warning
    );
    assert!(!report.has_errors(), "{:?}", report.findings);
}

#[test]
fn inherited_file_condition_means_not_a_blocker() {
    let report = validate(&[
        (
            "dic/a.pasta",
            text("＆＄＊初回\n＊雑談＆優先：3\n　Alice：あ\n"),
        ),
        ("dic/b.pasta", text("＊雑談\n　＄＊初回＝1\n")),
    ]);
    assert_eq!(condition_findings(&report), vec![]);
}

#[test]
fn shadowing_spans_files_and_aliases() {
    let input = GhostInput {
        pasta_toml: TomlSource::Text {
            text: format!("{TOML}[scene.alias]\n\"雑談\" =[\"おしゃべり\"]\n"),
        },
        entries: vec!["dic/a.pasta".into(), "dic/b.pasta".into()],
        dirs: vec![],
    };
    let files: BTreeMap<_, _> = [
        ("dic/a.pasta", "＊雑談＆優先：1\n　Alice：あ\n"),
        ("dic/b.pasta", "＊おしゃべり\n　Alice：い\n"),
    ]
    .into_iter()
    .map(|(p, t)| (p.to_string(), text(t)))
    .collect();
    let report = validate_ghost(&input, &files);
    assert_eq!(
        condition_findings(&report),
        vec![(
            "dic/b.pasta".into(),
            1,
            FindingKind::SceneShadowedByPriority,
            "シーン「おしゃべり」は、条件の無い ＆優先：1 の同じ名前のシーン（ghost/master/dic/a.pasta:1）があるため選ばれません".into()
        )]
    );
}

#[test]
fn skipped_when_a_file_cannot_be_read() {
    // 読めないファイルの中の代入を見落として偽の注意を出さない（8.4）
    let report = validate(&[
        (
            "dic/a.pasta",
            text(
                "＊雑談＆優先：1＆＄＊無い\n　Alice：あ\n＊雑談\n　Alice：い\n＊雑談＆優先：1\n　Alice：う\n",
            ),
        ),
        ("dic/b.pasta", FileText::NotUtf8),
    ]);
    assert!(!report.stats.skip_reasons.is_empty());
    assert_eq!(condition_findings(&report), vec![]);

    // 同じ辞書を読めれば両方の注意が出る
    let report = validate(&[(
        "dic/a.pasta",
        text(
            "＊雑談＆優先：1＆＄＊無い\n　Alice：あ\n＊雑談\n　Alice：い\n＊雑談＆優先：1\n　Alice：う\n",
        ),
    )]);
    let kinds: Vec<_> = condition_findings(&report)
        .into_iter()
        .map(|f| (f.1, f.2))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (1, FindingKind::CondVarUnassigned),
            (3, FindingKind::SceneShadowedByPriority),
        ]
    );
}
