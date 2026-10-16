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
        // 解析器が前に付ける位置の文言を外し、残りを「構文エラー: 」の後ろに付ける
        let prefix = format!("Parse error in {path} at {line}:{column}: ");
        let rest = message.strip_prefix(&prefix).expect("前置きがある");
        assert_eq!(f.message, format!("構文エラー: {rest}"));
        assert!(!f.message.contains("Parse error in"), "{}", f.message);
    }
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
