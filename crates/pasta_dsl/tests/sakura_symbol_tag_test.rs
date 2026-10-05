//! さくらスクリプト記号タグ（-+*?&）パーステスト
//!
//! ukadoc公式タグリストに記載された記号文字タグが
//! パーサーで正しくAction::SakuraScriptとして認識されることを検証。
//! Requirements: 2.1, 2.2, 2.3, 2.4, 4.2

use pasta_dsl::parser::{FileItem, parse_str};

/// テスト用ヘルパー: シーン内の最初のアクション行のアクションリストを取得
fn parse_actions(source: &str) -> Vec<pasta_dsl::parser::Action> {
    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");
    let scene = ast
        .items
        .into_iter()
        .find_map(|item| {
            if let FileItem::GlobalSceneScope(scope) = item {
                Some(scope)
            } else {
                None
            }
        })
        .expect("GlobalSceneScopeが存在すべし");

    // local_scenes[0] が local_start_scene_scope（名前なし）
    let local = scene
        .local_scenes
        .into_iter()
        .next()
        .expect("ローカルシーンが存在すべし");

    // items の中から ActionLine を取得
    let action_line = local
        .items
        .into_iter()
        .find_map(|item| {
            if let pasta_dsl::parser::LocalSceneItem::ActionLine(al) = item {
                Some(al)
            } else {
                None
            }
        })
        .expect("アクション行が存在すべし");

    action_line.actions
}

/// `\-` （ゴースト終了）がSakuraScriptとしてパースされること
#[test]
fn test_parse_sakura_hyphen_tag() {
    let source = "＊test\n　Alice：\\-\n";
    let actions = parse_actions(source);

    let sakura = actions
        .iter()
        .find_map(|a| {
            if let pasta_dsl::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .expect("SakuraScriptアクションが存在すべし");

    assert_eq!(sakura, r"\-");
}

/// `\+` （ランダム交代）がSakuraScriptとしてパースされること
#[test]
fn test_parse_sakura_plus_tag() {
    let source = "＊test\n　Alice：\\+\n";
    let actions = parse_actions(source);

    let sakura = actions
        .iter()
        .find_map(|a| {
            if let pasta_dsl::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .expect("SakuraScriptアクションが存在すべし");

    assert_eq!(sakura, r"\+");
}

/// `\*` （選択タイムアウト無効）がSakuraScriptとしてパースされること
#[test]
fn test_parse_sakura_asterisk_tag() {
    let source = "＊test\n　Alice：\\*\n";
    let actions = parse_actions(source);

    let sakura = actions
        .iter()
        .find_map(|a| {
            if let pasta_dsl::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .expect("SakuraScriptアクションが存在すべし");

    assert_eq!(sakura, r"\*");
}

/// `\_?` （タグ表示モード）がSakuraScriptとしてパースされること
#[test]
fn test_parse_sakura_underscore_question_tag() {
    let source = "＊test\n　Alice：\\_?\n";
    let actions = parse_actions(source);

    let sakura = actions
        .iter()
        .find_map(|a| {
            if let pasta_dsl::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .expect("SakuraScriptアクションが存在すべし");

    assert_eq!(sakura, r"\_?");
}

/// `\&[entity]` （エンティティ参照）がSakuraScriptとしてパースされること
#[test]
fn test_parse_sakura_ampersand_tag() {
    let source = "＊test\n　Alice：\\&[entity]\n";
    let actions = parse_actions(source);

    let sakura = actions
        .iter()
        .find_map(|a| {
            if let pasta_dsl::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .expect("SakuraScriptアクションが存在すべし");

    assert_eq!(sakura, r"\&[entity]");
}

/// `こんにちは\-。` が talk + sakura_script + talk の3要素に分割されること
#[test]
fn test_parse_sakura_symbol_in_mixed_text() {
    let source = "＊test\n　Alice：こんにちは\\-。\n";
    let actions = parse_actions(source);

    assert_eq!(actions.len(), 3, "3つのアクション要素に分割されるべし");

    // 1. Talk: "こんにちは"
    match &actions[0] {
        pasta_dsl::parser::Action::Talk { text, .. } => {
            assert_eq!(text, "こんにちは");
        }
        other => panic!("最初のアクションはTalkであるべし: {:?}", other),
    }

    // 2. SakuraScript: "\-"
    match &actions[1] {
        pasta_dsl::parser::Action::SakuraScript { script, .. } => {
            assert_eq!(script, r"\-");
        }
        other => panic!("2番目のアクションはSakuraScriptであるべし: {:?}", other),
    }

    // 3. Talk: "。"
    match &actions[2] {
        pasta_dsl::parser::Action::Talk { text, .. } => {
            assert_eq!(text, "。");
        }
        other => panic!("3番目のアクションはTalkであるべし: {:?}", other),
    }
}

/// 既存タグ（\h, \s[0], \_w[500]）がリグレッションなくパースされること
#[test]
fn test_parse_existing_tags_no_regression() {
    let source = "＊test\n　Alice：\\h\\s[0]\\_w[500]\n";
    let actions = parse_actions(source);

    assert_eq!(
        actions.len(),
        3,
        "3つのSakuraScriptアクションに分割されるべし"
    );

    let scripts: Vec<&str> = actions
        .iter()
        .filter_map(|a| {
            if let pasta_dsl::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(scripts, vec![r"\h", r"\s[0]", r"\_w[500]"]);
}

// ============================================================
// SSP と同じ読み方（paragraph-break-tag-only-talk）
// Requirements: 7.1〜7.9, 8.1, 8.4, 9.1, 9.4, 9.5, 10.3, 10.7, 10.8
// ============================================================

/// アクションを `S:`（さくらスクリプト）・`T:`（台詞）・`E:`（エスケープ）の並びに要約する
fn summarize(actions: &[pasta_dsl::parser::Action]) -> Vec<String> {
    use pasta_dsl::parser::Action;
    actions
        .iter()
        .map(|a| match a {
            Action::SakuraScript { script, .. } => format!("S:{script}"),
            Action::Talk { text, .. } => format!("T:{text}"),
            Action::Escape { sequence, .. } => format!("E:{sequence}"),
            other => format!("{other:?}"),
        })
        .collect()
}

/// アクション行 1 行（`Alice：{line}`）を要約した並び
fn actions_of(line: &str) -> Vec<String> {
    summarize(&parse_actions(&format!("＊test\n　Alice：{line}\n")))
}

/// シーン内のすべてのアクション行を要約した並び
fn all_action_lines(source: &str) -> Vec<Vec<String>> {
    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");
    ast.items
        .into_iter()
        .filter_map(|item| match item {
            FileItem::GlobalSceneScope(scope) => Some(scope),
            _ => None,
        })
        .flat_map(|scope| scope.local_scenes)
        .flat_map(|local| local.items)
        .filter_map(|item| match item {
            pasta_dsl::parser::LocalSceneItem::ActionLine(al) => Some(summarize(&al.actions)),
            _ => None,
        })
        .collect()
}

/// グローバル単語定義 1 行の値の並び
fn word_values(line: &str) -> Result<Vec<String>, String> {
    let ast = parse_str(&format!("{line}\n"), "test.pasta").map_err(|e| format!("{e:?}"))?;
    match ast.items.into_iter().next() {
        Some(FileItem::GlobalWord(w)) => Ok(w.words),
        other => panic!("GlobalWord を期待, got: {other:?}"),
    }
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// 名前の後ろの字はタグに含めない（7.1〜7.4）
#[test]
fn test_tag_name_does_not_swallow_following_text() {
    assert_eq!(actions_of(r"\nHello"), s(&[r"S:\n", "T:Hello"]));
    assert_eq!(actions_of(r"\w9OK"), s(&[r"S:\w9", "T:OK"]));
    assert_eq!(actions_of(r"\s12"), s(&[r"S:\s1", "T:2"]));
}

/// `\%` はエスケープとして 1 単位（8.1, 8.4）
#[test]
fn test_percent_escape() {
    assert_eq!(actions_of(r"100\%です"), s(&["T:100", r"E:\%", "T:です"]));
}

/// 同じ行で閉じる囲みは全体で 1 つのさくらスクリプト、中の `＠w` は展開しない（9.1, 9.5）
#[test]
fn test_closed_literal_is_single_sakura_script() {
    assert_eq!(
        actions_of(r"\_?＠w\s[1]\_?z"),
        s(&[r"S:\_?＠w\s[1]\_?", "T:z"])
    );
    assert_eq!(actions_of(r"\_!a\_?b\_!"), s(&[r"S:\_!a\_?b\_!"]));
}

/// 閉じない `\_?` は単独のタグ（9.4）
#[test]
fn test_unclosed_literal_is_single_tag() {
    assert_eq!(actions_of(r"\_?abc"), s(&[r"S:\_?", "T:abc"]));
}

/// 引数の中の `\` ＋ 1 文字と、引数の先頭の引用（7.5〜7.7）
#[test]
fn test_args_escape_pair_and_leading_quote() {
    assert_eq!(actions_of(r"\s[a\]b]"), s(&[r"S:\s[a\]b]"]));
    assert_eq!(
        actions_of(r#"\![raise,X,"a]b"]"#),
        s(&[r#"S:\![raise,X,"a]b"]"#])
    );
    assert_eq!(actions_of(r#"\q[5"x,OnX]"#), s(&[r#"S:\q[5"x,OnX]"#]));
}

/// 引数の途中の `"` は引用を開かない（7.7）
#[test]
fn test_mid_arg_quote_does_not_open_quote() {
    assert_eq!(
        actions_of(r#"\![raise,X,a"]b]"#),
        s(&[r#"S:\![raise,X,a"]"#, "T:b]"])
    );
}

/// 閉じない先頭の引用は引数にならず、名前までがタグ（7.8）
#[test]
fn test_unclosed_leading_quote() {
    assert_eq!(
        actions_of(r#"\q["abc,OnX]y"#),
        s(&[r"S:\q", r#"T:["abc,OnX]y"#])
    );
}

/// 引数は行をまたがない（7.8, 10.8）
#[test]
fn test_args_do_not_cross_lines() {
    let lines = all_action_lines("＊test\n　Alice：\\s[0\n　Alice：]\n");
    assert_eq!(lines, vec![s(&[r"S:\s", "T:[0"]), s(&["T:]"])]);
}

/// 単語の値: さくらスクリプトで始まる値はそのコマンド 1 個で終わる（10.7）
#[test]
fn test_word_value_sakura_script_is_single_command() {
    assert!(word_values(r"＠x：\nOK").is_err(), "\\nOK はパースエラー");
    assert!(word_values(r"＠x：\w10").is_err(), "\\w10 はパースエラー");
    assert_eq!(
        word_values(r"＠x：\_?\s[1]\_?").unwrap(),
        s(&[r"\_?\s[1]\_?"])
    );
    assert_eq!(word_values(r"＠x：\s[0]").unwrap(), s(&[r"\s[0]"]));
}

/// 名前の形にもエスケープにも囲みにも合わない `\` はパースエラー（10.3）
#[test]
fn test_invalid_backslash_is_parse_error() {
    for line in [r"\_", r"\あ"] {
        let source = format!("＊test\n　a：{line}\n");
        assert!(
            parse_str(&source, "test.pasta").is_err(),
            "{line} はパースエラーであるべし"
        );
    }
}
