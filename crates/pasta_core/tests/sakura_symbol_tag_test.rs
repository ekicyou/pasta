//! さくらスクリプト記号タグ（-+*?&）パーステスト
//!
//! ukadoc公式タグリストに記載された記号文字タグが
//! パーサーで正しくAction::SakuraScriptとして認識されることを検証。
//! Requirements: 2.1, 2.2, 2.3, 2.4, 4.2

use pasta_core::parser::{FileItem, parse_str};

/// テスト用ヘルパー: シーン内の最初のアクション行のアクションリストを取得
fn parse_actions(source: &str) -> Vec<pasta_core::parser::Action> {
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
            if let pasta_core::parser::LocalSceneItem::ActionLine(al) = item {
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
            if let pasta_core::parser::Action::SakuraScript { script, .. } = a {
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
            if let pasta_core::parser::Action::SakuraScript { script, .. } = a {
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
            if let pasta_core::parser::Action::SakuraScript { script, .. } = a {
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
            if let pasta_core::parser::Action::SakuraScript { script, .. } = a {
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
            if let pasta_core::parser::Action::SakuraScript { script, .. } = a {
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
        pasta_core::parser::Action::Talk { text, .. } => {
            assert_eq!(text, "こんにちは");
        }
        other => panic!("最初のアクションはTalkであるべし: {:?}", other),
    }

    // 2. SakuraScript: "\-"
    match &actions[1] {
        pasta_core::parser::Action::SakuraScript { script, .. } => {
            assert_eq!(script, r"\-");
        }
        other => panic!("2番目のアクションはSakuraScriptであるべし: {:?}", other),
    }

    // 3. Talk: "。"
    match &actions[2] {
        pasta_core::parser::Action::Talk { text, .. } => {
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
            if let pasta_core::parser::Action::SakuraScript { script, .. } = a {
                Some(script.as_str())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(scripts, vec![r"\h", r"\s[0]", r"\_w[500]"]);
}
