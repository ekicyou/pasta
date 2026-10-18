//! 検査の規則のテスト（4.x）

use std::collections::BTreeMap;

use pasta_check::validate::{
    FileText, FindingKind, GhostInput, Severity, TomlSource, module_name_of,
    select_dictionary_files, validate_ghost,
};

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\", \"*.pasta\"]\n";

/// モジュール名の重なりの問題だけを (パス, 種類, 理由) で返す
fn module_findings(entries: &[&str]) -> Vec<(String, FindingKind, String)> {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: entries.iter().map(|s| s.to_string()).collect(),
        dirs: vec![],
    };
    let files: BTreeMap<_, _> = select_dictionary_files(&input)
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let text = format!("＊場面{i}\n  さくら：こんにちは。\n");
            (p, FileText::Text { text })
        })
        .collect();
    validate_ghost(&input, &files)
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::ModulePastaCollision
                    | FindingKind::ModuleLuaCollision
                    | FindingKind::ModuleLuaShadowed
            )
        })
        .map(|f| (f.path, f.code, f.message))
        .collect()
}

#[test]
fn module_name_follows_pasta_dll_rule() {
    assert_eq!(module_name_of("dic/a-b.pasta"), "pasta.scene.a_b");
    assert_eq!(module_name_of("dic/a.b.pasta"), "pasta.scene.a_b");
    assert_eq!(module_name_of("dic/a_b.pasta"), "pasta.scene.a_b");
    assert_eq!(
        module_name_of("dic/dialog/npc/shop-keeper.lua"),
        "pasta.scene.dialog.npc.shop_keeper"
    );
    // 先頭の成分がちょうど dic のときだけ外す
    assert_eq!(module_name_of("talk.pasta"), "pasta.scene.talk");
    assert_eq!(module_name_of("dicx/a.pasta"), "pasta.scene.dicx.a");
    assert_eq!(module_name_of("Dic/a.pasta"), "pasta.scene.Dic.a");
    assert_eq!(module_name_of("x/dic/a.pasta"), "pasta.scene.x.dic.a");
}

#[test]
fn pasta_spellings_collide_three_errors() {
    let found = module_findings(&["dic/a-b.pasta", "dic/a.b.pasta", "dic/a_b.pasta"]);
    assert_eq!(found.len(), 3, "{found:?}");
    for (path, code, message) in &found {
        assert_eq!(*code, FindingKind::ModulePastaCollision);
        assert_eq!(code.severity(), Severity::Error);
        assert!(message.contains("pasta.scene.a_b"), "{message}");
        // 自分以外の全ファイルを示す
        for other in ["dic/a-b.pasta", "dic/a.b.pasta", "dic/a_b.pasta"] {
            assert_eq!(
                message.contains(&format!("ghost/master/{other}")),
                other != path,
                "{path}: {message}"
            );
        }
    }
}

#[test]
fn lua_same_as_pasta_is_warning_on_lua() {
    let found = module_findings(&["dic/talk.pasta", "dic/talk.lua"]);
    assert_eq!(found.len(), 1, "{found:?}");
    let (path, code, message) = &found[0];
    assert_eq!(path, "dic/talk.lua");
    assert_eq!(*code, FindingKind::ModuleLuaShadowed);
    assert_eq!(code.severity(), Severity::Warning);
    assert!(message.contains("ghost/master/dic/talk.pasta"), "{message}");
    assert!(message.contains("pasta.scene.talk"), "{message}");
}

#[test]
fn lua_with_lua_collide_errors() {
    let found = module_findings(&["dic/main.pasta", "dic/x-y.lua", "dic/x.y.lua"]);
    let codes: Vec<_> = found.iter().map(|(p, c, _)| (p.as_str(), *c)).collect();
    assert_eq!(
        codes,
        vec![
            ("dic/x-y.lua", FindingKind::ModuleLuaCollision),
            ("dic/x.y.lua", FindingKind::ModuleLuaCollision),
        ]
    );
    assert!(found[0].2.contains("ghost/master/dic/x.y.lua"));
    assert!(found[1].2.contains("ghost/master/dic/x-y.lua"));
}

/// dic/ の外の a.pasta と dic/a.pasta は同じ名前。階層が違えば別の名前
#[test]
fn outside_dic_and_hierarchy() {
    let found = module_findings(&["a.pasta", "dic/a.pasta"]);
    let paths: Vec<_> = found.iter().map(|(p, c, _)| (p.as_str(), *c)).collect();
    assert_eq!(
        paths,
        vec![
            ("a.pasta", FindingKind::ModulePastaCollision),
            ("dic/a.pasta", FindingKind::ModulePastaCollision),
        ]
    );

    // dic/x/a.pasta（x.a）と dic/x-a.pasta（x_a）は別、dic/x/a.pasta と dic/x.a.pasta（x_a）も別
    assert!(module_findings(&["dic/x/a.pasta", "dic/x-a.pasta", "dic/y.pasta"]).is_empty());
}

/// pasta.dll が見るパスから作る: 実物 Dic/a.pasta はパターンの綴りで dic/a.pasta → pasta.scene.a
#[test]
fn module_name_uses_pasta_dll_path() {
    let found = module_findings(&["Dic/a.pasta", "Dic/a.lua"]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "Dic/a.lua");
    assert!(found[0].2.contains("pasta.scene.a "), "{}", found[0].2);
    assert!(
        found[0].2.contains("ghost/master/Dic/a.pasta"),
        "{}",
        found[0].2
    );
}

/// 構文・読めないファイルの問題だけを返す
fn syntax_findings(files: Vec<(&str, FileText)>) -> Vec<pasta_check::validate::Finding> {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: files.iter().map(|(p, _)| p.to_string()).collect(),
        dirs: vec![],
    };
    let files: BTreeMap<_, _> = files.into_iter().map(|(p, t)| (p.to_string(), t)).collect();
    validate_ghost(&input, &files)
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::Syntax | FindingKind::FileNotUtf8 | FindingKind::FileUnreadable
            )
        })
        .collect()
}

#[test]
fn syntax_errors_in_two_files_both_reported_at_parser_position() {
    let bad_a = "＊挨拶\n  さくら：こんにちは。\n  {{{{壊れた}}}}\n";
    let bad_b = "＊会話\n  さくら：やあ。\n＊\n  ＠＠＠\n";
    let good = "＊場面\n  さくら：こんにちは。\n";
    let found = syntax_findings(vec![
        ("dic/a.pasta", FileText::Text { text: bad_a.into() }),
        ("dic/b.pasta", FileText::Text { text: bad_b.into() }),
        ("dic/c.pasta", FileText::Text { text: good.into() }),
    ]);
    assert_eq!(found.len(), 2, "{found:?}");
    for (f, (path, src)) in found
        .iter()
        .zip([("dic/a.pasta", bad_a), ("dic/b.pasta", bad_b)])
    {
        let Err(pasta_dsl::ParseError::SyntaxError {
            line,
            column,
            message,
            ..
        }) = pasta_dsl::parse_str(src, path)
        else {
            panic!("{path} は構文エラーになるはず");
        };
        assert_eq!(f.code, FindingKind::Syntax);
        assert_eq!(f.severity, Severity::Error);
        assert_eq!(f.path, path);
        assert_eq!(f.line, Some(line as u32));
        assert_eq!(f.column, Some(column as u32));
        // 解析器の文言の位置と枠を外し、末尾の `= …` の説明だけを「構文エラー: 」の後ろに付ける（7.1 の 1 行）
        let (_, short) = message
            .rsplit_once("\n  = ")
            .expect("pest の説明の行がある");
        assert_eq!(f.message, format!("構文エラー: {short}"));
        assert!(!f.message.contains('\n'), "{}", f.message);
        assert!(!f.message.contains("Parse error in"), "{}", f.message);
    }
}

/// 構文エラーは pest の短い説明だけの 1 行（7.1。design.md の例 `構文エラー: expected action_line` と同じ形）
#[test]
fn syntax_error_is_one_line_with_short_pest_description() {
    let found = syntax_findings(vec![(
        "dic/b-x.pasta",
        FileText::Text {
            text: "＊会話\n  ＠＠＠\n".into(),
        },
    )]);
    let lines: Vec<String> = found.iter().map(|f| f.line_text()).collect();
    assert_eq!(
        lines,
        vec!["ghost/master/dic/b-x.pasta:2:4: 誤り: 構文エラー: expected id".to_string()]
    );
}

#[test]
fn not_utf8_and_unreadable_are_errors_and_others_continue() {
    let found = syntax_findings(vec![
        ("dic/a.pasta", FileText::NotUtf8),
        (
            "dic/b.pasta",
            FileText::Unreadable {
                reason: "アクセスが拒否されました".into(),
            },
        ),
        (
            "dic/c.pasta",
            FileText::Text {
                text: "＊会話\n  {{{{壊れた}}}}\n".into(),
            },
        ),
    ]);
    let got: Vec<_> = found
        .iter()
        .map(|f| (f.path.as_str(), f.code, f.severity, f.message.as_str()))
        .collect();
    assert_eq!(got.len(), 3, "{got:?}");
    assert_eq!(
        got[0],
        (
            "dic/a.pasta",
            FindingKind::FileNotUtf8,
            Severity::Error,
            "UTF-8 として読めません"
        )
    );
    assert_eq!(
        got[1],
        (
            "dic/b.pasta",
            FindingKind::FileUnreadable,
            Severity::Error,
            "ファイルを読めません: アクセスが拒否されました"
        )
    );
    assert_eq!((got[2].0, got[2].1), ("dic/c.pasta", FindingKind::Syntax));
    assert!(found[0].line.is_none() && found[1].line.is_none());
}

/// 1 つの辞書ファイルを検査し、トランスパイルの 3 つの問題だけを (種類, 行, 列, 理由) で返す。
/// あわせて pasta_lua のトランスパイルの成否と食い違わないことを確かめる
fn transpile_findings(text: &str) -> Vec<(FindingKind, Option<u32>, Option<u32>, String)> {
    let path = "dic/a.pasta";
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: vec![path.into()],
        dirs: vec![],
    };
    let files = BTreeMap::from([(path.to_string(), FileText::Text { text: text.into() })]);
    let found: Vec<_> = validate_ghost(&input, &files)
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::TranspileArgAssign
                    | FindingKind::TranspilePropertyInExpr
                    | FindingKind::TranspileContinuationWithoutActor
            )
        })
        .inspect(|f| {
            assert_eq!(f.severity, Severity::Error);
            assert_eq!(f.path, path);
        })
        .map(|f| (f.code, f.line, f.column, f.message))
        .collect();

    let ast = pasta_dsl::parse_str(text, path).expect("構文としては読める");
    let lua_failed = pasta_lua::LuaTranspiler::default()
        .transpile(&ast, &mut Vec::new())
        .is_err();
    assert_eq!(
        !found.is_empty(),
        lua_failed,
        "pasta_lua と食い違う: {text}"
    );
    found
}

#[test]
fn property_reference_inside_expressions_is_error() {
    let msg = "プロパティ参照（＄％名前）は式の中で使えません。先に変数へ代入してください";
    // (書き方, 誤りの列)。どれも 3 行目
    for (line, col) in [
        ("  ＄x＝＄％p＋1", 3),        // 式の一部
        ("  ＄x＝（＄％p）", 3),       // 括弧で包むとそのままではない
        ("  ＄＝＄％p", 3),            // 代入先の無い式
        ("  ＄％a＝＄％b", 3),         // プロパティへの代入の右辺
        ("  さくら：＠f（＄％p）", 7), // 台詞の中でも関数の引数は式（位置はアクション）
        ("  ＞場面（＄％p）", 3),      // Call の引数
        ("  ＞＄％p", 3),              // 動的な Call のキー
    ] {
        let text = format!("＊場面\n  さくら：はい。\n{line}\n");
        let found = transpile_findings(&text);
        assert_eq!(
            found,
            vec![(
                FindingKind::TranspilePropertyInExpr,
                Some(3),
                Some(col),
                msg.to_string()
            )],
            "{line}"
        );
    }
}

#[test]
fn property_reference_allowed_forms_are_not_errors() {
    for line in [
        "  ＄x＝＄％p",            // ローカル変数への代入の右辺がそのまま
        "  ＄＊g＝＄％p",          // グローバル変数への代入の右辺がそのまま
        "  さくら：値は＄％pです", // 台詞の中
        "  ＄％a＝1",              // プロパティへの代入の左辺
        "  ＄％a＝＄x＋1",         // 左辺がプロパティで右辺に参照が無い
    ] {
        let text = format!("＊場面\n  さくら：はい。\n{line}\n");
        assert!(transpile_findings(&text).is_empty(), "{line}");
    }
}

#[test]
fn continuation_without_actor_is_error_per_local_scene() {
    let msg = "アクターの無い継続行です。前の行にアクター行が必要です";
    // 開始のブロックの最初の行が継続行。列は字下げの後ろの `：` の位置（7.1。Call・選択肢と同じ）
    assert_eq!(
        transpile_findings("＊場面\n  ：続き\n"),
        vec![(
            FindingKind::TranspileContinuationWithoutActor,
            Some(2),
            Some(3),
            msg.to_string()
        )]
    );
    // 全角空白 1 つの字下げなら列 2
    assert_eq!(
        transpile_findings("＊場面\n\u{3000}：続き\n"),
        vec![(
            FindingKind::TranspileContinuationWithoutActor,
            Some(2),
            Some(2),
            msg.to_string()
        )]
    );
    // 前のローカルシーンのアクター行は引き継がない
    assert_eq!(
        transpile_findings("＊場面\n  さくら：はい。\n  ・次\n  ＄x＝1\n  ：続き\n"),
        vec![(
            FindingKind::TranspileContinuationWithoutActor,
            Some(5),
            Some(3),
            msg.to_string()
        )]
    );
    // 間に代入や Call があっても、同じローカルシーンの前のアクター行を引き継ぐ
    assert!(transpile_findings("＊場面\n  さくら：はい。\n  ＄x＝1\n  ：続き\n").is_empty());
}

#[test]
fn transpile_errors_found_in_every_file_and_occurrence() {
    let text = "＊一\n  ：続き\n  ＄x＝＄％p＋1\n＊二\n  ：また\n";
    let codes: Vec<_> = transpile_findings(text)
        .into_iter()
        .map(|(c, l, _, _)| (c, l))
        .collect();
    assert_eq!(
        codes,
        vec![
            (FindingKind::TranspileContinuationWithoutActor, Some(2)),
            (FindingKind::TranspilePropertyInExpr, Some(3)),
            (FindingKind::TranspileContinuationWithoutActor, Some(5)),
        ]
    );
}
