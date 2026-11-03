//! ファイル属性の受け継ぎと宣言の重なりの決まり（`merge_attrs`・`PastaFile::global_scene_attrs`）
//!
//! Requirements 1.4, 3.1, 3.2, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 5.2

use pasta_dsl::parser::*;

/// 並びを `名前=値` / `＄変数 op 値` / `＄変数` の文字列にする
fn show(attrs: &[Attr]) -> Vec<String> {
    attrs
        .iter()
        .map(|a| match &a.body {
            AttrBody::Decl(v) => format!("{}={}", a.key, v),
            AttrBody::Cond { op, value } => format!("{}{}{}", a.key, op.as_str(), value),
            AttrBody::Flag => a.key.clone(),
        })
        .collect()
}

fn scenes(src: &str) -> Vec<Vec<String>> {
    let file = parse_str(src, "test.pasta").unwrap();
    file.global_scene_attrs().iter().map(|a| show(a)).collect()
}

#[test]
fn requirement_3_6_example() {
    let src = "＆ファイル：「属性１」＆あなたが好き：true\n＊シーン１＆ファイル：「属性２」\n　Alice：やあ\n";
    assert_eq!(
        scenes(src),
        vec![vec!["ファイル=属性２", "あなたが好き=true"]]
    );
}

#[test]
fn later_wins_in_written_order() {
    // ファイル属性 → 宣言行 → 初期部の順で後が勝つ・位置は先
    let src = "＆a：1＆b：1\n＊会話＆b：2＆c：2\n　＆a：3＆c：3\n　Alice：やあ\n";
    assert_eq!(scenes(src), vec![vec!["a=3", "b=2", "c=3"]]);
}

#[test]
fn later_file_attr_line_replaces_whole_set() {
    let src =
        "＆a：1＆b：1\n＊一\n　Alice：やあ\n＊二\n　Alice：やあ\n＆c：2\n＊三\n　Alice：やあ\n";
    assert_eq!(
        scenes(src),
        vec![vec!["a=1", "b=1"], vec!["a=1", "b=1"], vec!["c=2"]]
    );
}

#[test]
fn conditions_and_flags_are_not_folded() {
    let src = "＆＄＊経験値≧500＆＄＊初回\n＊会話＆＄＊経験値＜1500\n　＆＄＊初回\n　Alice：やあ\n";
    assert_eq!(
        scenes(src),
        vec![vec![
            "＄＊経験値>=500",
            "＄＊初回",
            "＄＊経験値<1500",
            "＄＊初回"
        ]]
    );
}

#[test]
fn unnamed_scene_inherits_file_attrs_not_previous_scene() {
    let src = "＆f：1\n＊会話＆a：1\n　Alice：やあ\n＊＆b：2\n　Alice：よう\n＊\n　Alice：ほい\n";
    assert_eq!(
        scenes(src),
        vec![vec!["f=1", "a=1"], vec!["f=1", "b=2"], vec!["f=1"]]
    );
}

#[test]
fn actor_and_words_do_not_reset_set() {
    let src = "＆f：1\n％さくら\n　＆x：9\n＠単語：あ\n＊会話\n　Alice：やあ\n";
    assert_eq!(scenes(src), vec![vec!["f=1"]]);
}

#[test]
fn no_carry_over_between_files() {
    let a = parse_str("＆f：1\n＊会話\n　Alice：やあ\n", "a.pasta").unwrap();
    let b = parse_str("＊会話\n　Alice：やあ\n", "b.pasta").unwrap();
    assert_eq!(show(&a.global_scene_attrs()[0]), vec!["f=1"]);
    assert!(b.global_scene_attrs()[0].is_empty());
}

#[test]
fn local_scene_folds_only_its_own_decl_line() {
    let src =
        "＆f：1\n＊会話＆g：1\n　Alice：やあ\n　・選択肢＆k：1＆k：2＆＄x＆＄x\n　Alice：ほい\n";
    let file = parse_str(src, "test.pasta").unwrap();
    let local = file
        .items
        .iter()
        .find_map(|i| match i {
            FileItem::GlobalSceneScope(s) => s.local_scenes.iter().find(|l| !l.attrs.is_empty()),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        show(&merge_attrs(&[], &local.attrs)),
        vec!["k=2", "＄x", "＄x"]
    );
}

#[test]
fn scene_without_attrs_is_empty() {
    assert_eq!(
        scenes("＊会話\n　Alice：やあ\n"),
        vec![Vec::<String>::new()]
    );
}
