// 行種別ごとのPestパーサーテスト
// comprehensive_control_flow.pastaから抽出したテストケース

use pasta::parser::parse_str;

#[test]
fn test_comment_line() {
    let input = "＃ 包括的なコントロールフロー参照実装\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "コメント行のパースに失敗: {:?}",
        result.err()
    );
}

#[test]
fn test_global_word_def() {
    let input = "＠挨拶：こんにちは　やあ　ハロー\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "グローバル単語定義のパースに失敗: {:?}",
        result.err()
    );
    let file = result.unwrap();
    assert_eq!(file.global_words.len(), 1);
    assert_eq!(file.global_words[0].name, "挨拶");
}

#[test]
fn test_global_label() {
    let input = "＊メイン\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "グローバルラベルのパースに失敗: {:?}",
        result.err()
    );
    let file = result.unwrap();
    assert_eq!(file.scenes.len(), 1);
    assert_eq!(file.scenes[0].name, "メイン");
}

#[test]
fn test_local_word_def() {
    let input = "＊メイン\n　＠場所：東京　大阪　京都\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "ローカル単語定義のパースに失敗: {:?}",
        result.err()
    );
    let file = result.unwrap();
    assert_eq!(file.scenes[0].local_words.len(), 1);
    assert_eq!(file.scenes[0].local_words[0].name, "場所");
}

#[test]
fn test_var_assign() {
    let input = "＊メイン\n　＄カウンタ＝１０\n";
    let result = parse_str(input, "test");
    assert!(result.is_ok(), "変数代入のパースに失敗: {:?}", result.err());
}

#[test]
fn test_speech_line_with_word() {
    let input = "＊メイン\n　さくら　：＠挨拶\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "発話行（単語展開）のパースに失敗: {:?}",
        result.err()
    );
}

#[test]
fn test_call_no_args() {
    let input = "＊メイン\n　＞自己紹介\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "Call文（引数なし）のパースに失敗: {:?}",
        result.err()
    );
}

#[test]
fn test_call_with_args() {
    let input = "＊メイン\n　＞カウント表示（＄カウンタ）\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "Call文（引数あり）のパースに失敗: {:?}",
        result.err()
    );
}

/// Phase 1 (REQ-BC-1): Jump statement (？) is deprecated
/// Use Call (＞) instead
#[test]
fn test_jump_statement_deprecated() {
    let input = "＊メイン\n　？会話分岐\n";
    let result = parse_str(input, "test");
    // Phase 1: Jump statement is rejected
    assert!(
        result.is_err(),
        "Phase 1: Jump statement (？) is deprecated. Use Call (＞) instead"
    );
}

#[test]
fn test_local_label_no_args() {
    let input = "＊メイン\n　-自己紹介\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "ローカルラベル（引数なし）のパースに失敗: {:?}",
        result.err()
    );
    let file = result.unwrap();
    assert_eq!(file.scenes[0].local_scenes.len(), 1);
    assert_eq!(file.scenes[0].local_scenes[0].name, "自己紹介");
    assert_eq!(file.scenes[0].local_scenes[0].params.len(), 0);
}

#[test]
fn test_local_label_with_args() {
    let input = "＊メイン\n　-カウント表示　＄値\n";
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "ローカルラベル（引数あり）のパースに失敗: {:?}",
        result.err()
    );
    let file = result.unwrap();
    assert_eq!(file.scenes[0].local_scenes.len(), 1);
    assert_eq!(file.scenes[0].local_scenes[0].name, "カウント表示");
    assert_eq!(file.scenes[0].local_scenes[0].params.len(), 1);
    assert_eq!(file.scenes[0].local_scenes[0].params[0], "値");
}

#[test]
fn test_rune_block() {
    let input = r#"＊メイン
　```rune
    pub fn ローカル関数(ctx, args) {
        let メッセージ = args[0];
    }
　```
"#;
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "Runeブロックのパースに失敗: {:?}",
        result.err()
    );
}

#[test]
fn test_comprehensive_control_flow_full() {
    let input = include_str!("fixtures/comprehensive_control_flow.pasta");
    let result = parse_str(input, "test");
    assert!(
        result.is_ok(),
        "comprehensive_control_flow.pastaの全体パースに失敗: {:?}",
        result.err()
    );
}
