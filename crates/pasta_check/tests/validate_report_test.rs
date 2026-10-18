//! 報告の形のテスト: 1 行の形・並び順・まとめの行・誤りと注意の固定（7.1〜7.8・2.9・6.7・6.8）

use pasta_check::validate::{Finding, FindingKind, Report, Severity, SkipReason, Stats, msg};

fn finding(kind: FindingKind, path: &str, line: Option<u32>, col: Option<u32>, m: &str) -> Finding {
    Finding::new(kind, path, line, col, m)
}

fn stats(n: usize, d: usize, reasons: &[SkipReason]) -> Stats {
    Stats {
        dictionary_files: n,
        unchecked_dynamic_calls: d,
        skip_reasons: reasons.to_vec(),
        ..Stats::default()
    }
}

#[test]
fn one_line_with_column_without_column_and_without_line() {
    let report = Report::new(
        vec![
            finding(
                FindingKind::Syntax,
                "dic/02-talk.pasta",
                Some(14),
                Some(5),
                "構文エラー: expected action_line",
            ),
            finding(
                FindingKind::CallTargetMissing,
                "dic/11-jump.pasta",
                Some(20),
                None,
                "x",
            ),
            finding(
                FindingKind::NoDictionaryFiles,
                "pasta.toml",
                None,
                None,
                "y",
            ),
        ],
        stats(13, 0, &[]),
    );
    let lines = report.lines();
    assert_eq!(
        lines[0],
        "ghost/master/dic/02-talk.pasta:14:5: 誤り: 構文エラー: expected action_line"
    );
    assert_eq!(lines[1], "ghost/master/dic/11-jump.pasta:20: 注意: x");
    assert_eq!(lines[2], "ghost/master/pasta.toml: 注意: y");
}

/// どの理由も 1 件 1 行。改行を含む説明（toml・pest など）は空白 1 つでつなぐ（7.1）
#[test]
fn message_with_newlines_is_one_line() {
    let f = finding(
        FindingKind::ConfigSyntax,
        "pasta.toml",
        Some(3),
        Some(5),
        "pasta.toml を TOML として読めません: a\r\n  b\n\nc",
    );
    assert_eq!(f.message, "pasta.toml を TOML として読めません: a b c");
    assert_eq!(
        f.line_text(),
        "ghost/master/pasta.toml:3:5: 誤り: pasta.toml を TOML として読めません: a b c"
    );
}

#[test]
fn findings_are_sorted_by_path_line_column_code_message() {
    let report = Report::new(
        vec![
            finding(FindingKind::Syntax, "dic/b.pasta", Some(1), Some(1), "b"),
            finding(
                FindingKind::CallTargetMissing,
                "dic/a.pasta",
                Some(3),
                Some(2),
                "z",
            ),
            finding(
                FindingKind::CallTargetMissing,
                "dic/a.pasta",
                Some(3),
                Some(2),
                "a",
            ),
            finding(FindingKind::Syntax, "dic/a.pasta", Some(3), Some(2), "s"),
            finding(FindingKind::Syntax, "dic/a.pasta", Some(3), Some(1), "c1"),
            finding(FindingKind::Syntax, "dic/a.pasta", Some(10), None, "l10"),
            finding(
                FindingKind::ModulePastaCollision,
                "dic/a.pasta",
                None,
                None,
                "noline",
            ),
        ],
        stats(2, 0, &[]),
    );
    let order: Vec<&str> = report.findings.iter().map(|f| f.message.as_str()).collect();
    assert_eq!(order, ["noline", "c1", "s", "a", "z", "l10", "b"]);
}

#[test]
fn stats_counts_follow_findings() {
    let report = Report::new(
        vec![
            finding(FindingKind::Syntax, "dic/a.pasta", Some(1), Some(1), "e"),
            finding(
                FindingKind::CallTargetMissing,
                "dic/a.pasta",
                Some(2),
                Some(1),
                "w",
            ),
            finding(
                FindingKind::ChoiceTargetMissing,
                "dic/a.pasta",
                Some(3),
                Some(1),
                "w",
            ),
        ],
        stats(1, 0, &[]),
    );
    assert_eq!(report.stats.errors, 1);
    assert_eq!(report.stats.warnings, 2);
    assert!(report.has_errors());

    let ok = Report::new(vec![], stats(1, 0, &[]));
    assert!(!ok.has_errors());
}

#[test]
fn summary_line_with_references_checked() {
    let report = Report::new(
        vec![finding(
            FindingKind::CallTargetMissing,
            "dic/a.pasta",
            Some(2),
            Some(1),
            "w",
        )],
        stats(13, 4, &[]),
    );
    assert_eq!(
        report.lines().last().unwrap(),
        "検査した辞書ファイル 13 個: 誤り 0 件、注意 1 件、飛び先を検査しなかった Call 4 件"
    );
    assert_eq!(report.lines().len(), 2);
}

#[test]
fn summary_line_and_notice_when_references_skipped() {
    let report = Report::new(
        vec![
            finding(FindingKind::Syntax, "dic/a.pasta", Some(1), Some(1), "e1"),
            finding(FindingKind::Syntax, "dic/b.pasta", Some(1), Some(1), "e2"),
            finding(FindingKind::Syntax, "dic/c.pasta", Some(1), Some(1), "e3"),
            finding(
                FindingKind::CallTargetMissing,
                "dic/d.pasta",
                Some(1),
                Some(1),
                "w",
            ),
        ],
        stats(13, 7, &[SkipReason::Syntax]),
    );
    let lines = report.lines();
    assert_eq!(lines.len(), 6);
    assert_eq!(
        lines[4],
        "構文エラーがあるため、飛び先の検査をしていません。"
    );
    assert_eq!(lines[5], "検査した辞書ファイル 13 個: 誤り 3 件、注意 1 件");
}

/// 案内の行は理由を表の順に「と」でつなぐ。省いたときはまとめの行に飛び先の項が無い（1.4・1.5）
#[test]
fn notice_lists_skip_reasons_in_table_order() {
    use SkipReason::*;
    for (reasons, notice) in [
        (
            &[Syntax][..],
            "構文エラーがあるため、飛び先の検査をしていません。",
        ),
        (
            &[UnreadableFile][..],
            "読めない辞書ファイルがあるため、飛び先の検査をしていません。",
        ),
        (
            &[Config][..],
            "pasta.toml の誤りがあるため、飛び先の検査をしていません。",
        ),
        (
            &[Syntax, Config][..],
            "構文エラーと pasta.toml の誤りがあるため、飛び先の検査をしていません。",
        ),
        (
            &[Syntax, UnreadableFile, Config][..],
            "構文エラーと読めない辞書ファイルと pasta.toml の誤りがあるため、飛び先の検査をしていません。",
        ),
    ] {
        let report = Report::new(vec![], stats(2, 3, reasons));
        assert!(report.stats.references_skipped());
        assert_eq!(
            report.lines(),
            vec![
                notice.to_string(),
                "検査した辞書ファイル 2 個: 誤り 0 件、注意 0 件".to_string()
            ]
        );
    }
    assert!(!stats(2, 3, &[]).references_skipped());
}

/// すべての種類の分類と code を固定する（7.7）。種類を足したらここにも足す
#[test]
fn every_kind_has_fixed_severity_and_code() {
    use FindingKind::*;
    use Severity::*;
    let table = [
        (ConfigMissing, Error, "config-missing"),
        (ConfigSyntax, Error, "config-syntax"),
        (ConfigLoaderType, Error, "config-loader-type"),
        (ConfigSceneAlias, Error, "config-scene-alias"),
        (ConfigActorFont, Error, "config-actor-font"),
        (ConfigPatternInvalid, Error, "config-pattern-invalid"),
        (PatternIgnored, Warning, "pattern-ignored"),
        (PatternNotPasta, Warning, "pattern-not-pasta"),
        (NoDictionaryFiles, Warning, "no-dictionary-files"),
        (FolderMatchesPattern, Error, "folder-matches-pattern"),
        (InitFileName, Error, "init-file-name"),
        (FileNotUtf8, Error, "file-not-utf8"),
        (FileUnreadable, Error, "file-unreadable"),
        (Syntax, Error, "syntax"),
        (TranspileArgAssign, Error, "transpile-arg-assign"),
        (TranspilePropertyInExpr, Error, "transpile-property-in-expr"),
        (
            TranspileContinuationWithoutActor,
            Error,
            "transpile-continuation-without-actor",
        ),
        (ModulePastaCollision, Error, "module-pasta-collision"),
        (ModuleLuaCollision, Error, "module-lua-collision"),
        (ModuleLuaShadowed, Warning, "module-lua-shadowed"),
        (GlobalSceneCollision, Warning, "global-scene-collision"),
        (LocalSceneCollision, Warning, "local-scene-collision"),
        (SceneShadowedByBuiltin, Warning, "scene-shadowed-by-builtin"),
        (CallTargetMissing, Warning, "call-target-missing"),
        (ChoiceTargetMissing, Warning, "choice-target-missing"),
    ];
    assert_eq!(table.len(), FindingKind::ALL.len());
    assert_eq!(FindingKind::ALL.len(), 25);
    for (kind, sev, code) in table {
        assert_eq!(kind.severity(), sev, "{code}");
        assert_eq!(kind.code(), code);
        let f = Finding::new(kind, "pasta.toml", None, None, "m");
        assert_eq!(f.severity, sev);
    }
    for kind in FindingKind::ALL {
        assert!(table.iter().any(|(k, _, _)| *k == kind));
    }
}

/// 文言の表（design.md の Report）どおりの理由
#[test]
fn messages_follow_wording_table() {
    assert_eq!(
        msg::folder_matches_pattern(),
        "辞書のパターンに一致するフォルダです。pasta.dll はこのフォルダを辞書として読めず、ゴーストを読み込めません"
    );
    assert_eq!(
        msg::config_missing(),
        "pasta.toml がありません。pasta.dll はゴーストを読み込めません（辞書は既定の dic/**/*.pasta で集めました）"
    );
    assert_eq!(
        msg::config_syntax("x"),
        "pasta.toml を TOML として読めません: x"
    );
    assert_eq!(
        msg::config_loader_type("pasta_patterns", "x"),
        "[loader] の pasta_patterns の型が違います: x"
    );
    assert_eq!(
        msg::config_scene_alias_duplicate("会話", &["OnTalk".into(), "OnX".into()]),
        "[scene.alias] の別名「会話」が OnTalk・OnX に重複しています"
    );
    assert_eq!(
        msg::config_loader_not_table("x"),
        "[loader] が表ではありません: x"
    );
    assert_eq!(
        msg::config_scene_alias_empty("OnTalk"),
        "[scene.alias] の OnTalk に空の別名があります"
    );
    assert_eq!(
        msg::config_scene_alias_empty(""),
        "[scene.alias] に置き換え先の名前が空のものがあります"
    );
    assert_eq!(
        msg::config_scene_alias_chain("会話"),
        "[scene.alias] の「会話」は置き換え先と別名の両方に書かれています（別名を別名で置き換えることはできません）"
    );
    assert_eq!(
        msg::config_scene_alias_type("x"),
        "[scene.alias] の型が違います: x"
    );
    assert_eq!(
        msg::config_pattern_invalid("[", "x"),
        "pasta_patterns の「[」はパターンとして読めません: x"
    );
    assert_eq!(
        msg::pattern_ignored("../a"),
        "pasta_patterns の「../a」は .. などを含むため使われません"
    );
    assert_eq!(
        msg::pattern_not_pasta("dic/*"),
        "pasta_patterns の「dic/*」は .pasta で終わらないため、.lua の辞書モジュールを探すのに使われません"
    );
    assert_eq!(
        msg::no_dictionary_files(),
        "pasta_patterns に一致する辞書ファイルが 1 つもありません"
    );
    assert_eq!(
        msg::init_file_name("init.pasta"),
        "ファイル名 init.pasta は使えません。pasta.dll はゴーストを読み込めません"
    );
    assert_eq!(msg::file_not_utf8(), "UTF-8 として読めません");
    assert_eq!(msg::file_unreadable("x"), "ファイルを読めません: x");
    assert_eq!(msg::syntax("x"), "構文エラー: x");
    assert_eq!(
        msg::transpile_arg_assign(),
        "シーンの引数（＄０ など）には代入できません"
    );
    assert_eq!(
        msg::transpile_property_in_expr(),
        "プロパティ参照（＄％名前）は式の中で使えません。先に変数へ代入してください"
    );
    assert_eq!(
        msg::transpile_continuation_without_actor(),
        "アクターの無い継続行です。前の行にアクター行が必要です"
    );
    assert_eq!(
        msg::module_pasta_collision("pasta.scene.a_b", &["dic/a_b.pasta"]),
        "モジュール名 pasta.scene.a_b が ghost/master/dic/a_b.pasta と同じです。どちらかの辞書の内容が失われます"
    );
    assert_eq!(
        msg::module_pasta_collision("pasta.scene.a_b", &["dic/a-b.pasta", "dic/a.b.pasta"]),
        "モジュール名 pasta.scene.a_b が ghost/master/dic/a-b.pasta、ghost/master/dic/a.b.pasta と同じです。どちらかの辞書の内容が失われます"
    );
    assert_eq!(
        msg::module_lua_collision("pasta.scene.a", &["dic/a.lua"]),
        "モジュール名 pasta.scene.a が ghost/master/dic/a.lua と同じです。どちらかの .lua が読み込まれません"
    );
    assert_eq!(
        msg::module_lua_shadowed("pasta.scene.a", &["dic/a.pasta"]),
        "モジュール名 pasta.scene.a が ghost/master/dic/a.pasta と同じため、この .lua は読み込まれません"
    );
    assert_eq!(
        msg::scene_collision("会話·A", "会話_A", "会話_A", "dic/a.pasta", 3),
        "シーン「会話·A」は照合用の名前「会話_A」が「会話_A」（ghost/master/dic/a.pasta:3）と同じになり、同じ名前のシーンとして選ばれます"
    );
    assert_eq!(
        msg::scene_shadowed_by_builtin("yield"),
        "シーン「yield」は組み込みの飛び先と同じ名前のため、＞yield では呼ばれません"
    );
    assert_eq!(
        msg::call_target_missing("寄り道x"),
        "Call の飛び先「寄り道x」が辞書のシーンにありません（Lua で定義していれば問題ありません）"
    );
    assert_eq!(
        msg::choice_target_missing("先"),
        "選択肢のジャンプ先「先」が辞書のシーンにありません（Lua で定義・振り分けていれば問題ありません）"
    );
}
