//! おすすめメニューの検査のテスト: ＞おすすめ項目 の引数とメニューのシーンの台詞（10.3〜10.6・12.4）

use std::collections::BTreeMap;

use pasta_check::validate::{FileText, FindingKind, GhostInput, TomlSource, msg, validate_ghost};

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";

/// おすすめメニューの注意だけを (行, 列, 種類, 理由) で返す
fn recommend_findings(toml: &str, text: &str) -> Vec<(u32, u32, FindingKind, String)> {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: toml.into() },
        entries: vec!["dic/menu.pasta".into()],
        dirs: vec![],
    };
    let files = BTreeMap::from([(
        "dic/menu.pasta".to_string(),
        FileText::Text { text: text.into() },
    )]);
    let report = validate_ghost(&input, &files);
    assert!(
        report.stats.skip_reasons.is_empty(),
        "{:?}",
        report.findings
    );
    report
        .findings
        .into_iter()
        .filter(|f| {
            matches!(
                f.code,
                FindingKind::RecommendItemInvalid | FindingKind::RecommendMenuTalk
            )
        })
        .map(|f| (f.line.unwrap(), f.column.unwrap(), f.code, f.message))
        .collect()
}

#[test]
fn missing_item_scene_is_reported_by_exact_match() {
    // `＊おすすめ_カクヨム秋` は前方一致なら当たるが、項目のシーンは完全一致で探す
    let text = "＊メニュー\n  ＞おすすめ項目（「a」、「https://a/」、「おすすめ_カクヨム」）\n  ＞recommend_item（「b」、「https://b/」、「おすすめ_カクヨム秋」）\n＊おすすめ_カクヨム秋\n  さくら：あ。\n";
    assert_eq!(
        recommend_findings(TOML, text),
        vec![(
            2,
            3,
            FindingKind::RecommendItemInvalid,
            msg::recommend_item_scene_missing("おすすめ_カクヨム"),
        )]
    );
}

#[test]
fn keyword_third_arg_is_checked_like_positional() {
    // 名前つきの引数もキーを捨てて 3 つ目として渡るので、同じく照合する
    let text = "＊メニュー\n  ＞おすすめ項目（「a」、「https://a/」、名：「無い」）\n";
    assert_eq!(
        recommend_findings(TOML, text),
        vec![(
            2,
            3,
            FindingKind::RecommendItemInvalid,
            msg::recommend_item_scene_missing("無い"),
        )]
    );
}

#[test]
fn item_scene_is_found_after_alias() {
    let toml = format!("{TOML}[scene.alias]\n\"挨拶\" = [\"こんにちは\"]\n");
    let text = "＊メニュー\n  ＞おすすめ項目（「a」、「https://a/」、「こんにちは」）\n  ＞おすすめ項目（「b」、「https://b/」、（「挨拶」））\n＊挨拶\n  さくら：あ。\n";
    assert_eq!(recommend_findings(&toml, text), vec![]);
    let found = recommend_findings(TOML, text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, 2);
}

#[test]
fn arg_count_outside_two_to_four_is_reported() {
    let text = "＊メニュー\n  ＞おすすめ項目（「a」）\n  ＞おすすめ項目（「a」、「b」、「」、「d」、「e」）\n  ＞おすすめ項目\n  ＞おすすめ項目（「a」、「b」）\n  ＞おすすめ項目（「a」、「b」、「」、「d」）\n";
    assert_eq!(
        recommend_findings(TOML, text),
        vec![
            (
                2,
                3,
                FindingKind::RecommendItemInvalid,
                msg::recommend_item_arg_count(1)
            ),
            (
                3,
                3,
                FindingKind::RecommendItemInvalid,
                msg::recommend_item_arg_count(5)
            ),
            (
                4,
                3,
                FindingKind::RecommendItemInvalid,
                msg::recommend_item_arg_count(0)
            ),
        ]
    );
}

/// 変数・式・数の引数は実行時まで分からないので見ない（10.5）
#[test]
fn non_string_third_arg_is_not_checked() {
    let text = "＊メニュー\n  ＄名前＝「無い」\n  ＞おすすめ項目（「a」、「b」、＄名前）\n  ＞おすすめ項目（「a」、「b」、「無」＋「い」）\n  ＞おすすめ項目（「a」、「b」、１）\n";
    assert_eq!(recommend_findings(TOML, text), vec![]);
}

#[test]
fn menu_scene_talk_is_reported_but_called_scene_talk_is_not() {
    let toml = format!("{TOML}[scene.alias]\n\"sakura.recommendsites\" = [\"本棚メニュー\"]\n");
    let text = "＊本棚メニュー\n  さくら：ここは捨てられる。\n  ：続きも。\n  ＞共通\n  ・ローカル\n    さくら：見ない。\n＊共通\n  さくら：呼んだ先は見ない。\n  ＞おすすめ項目（「a」、「https://a/」）\n";
    assert_eq!(
        recommend_findings(&toml, text),
        vec![
            (
                2,
                3,
                FindingKind::RecommendMenuTalk,
                msg::recommend_menu_talk("本棚メニュー")
            ),
            (
                3,
                3,
                FindingKind::RecommendMenuTalk,
                msg::recommend_menu_talk("本棚メニュー")
            ),
        ]
    );
    // 名前を変えなければ ＊本棚メニュー はふつうのシーン
    assert_eq!(recommend_findings(TOML, text), vec![]);
}

/// 既定の別名（おすすめメニュー → sakura.recommendsites など）と char<数字> のメニュー
#[test]
fn default_alias_and_char_menu_scenes_are_menu_scenes() {
    let toml = format!(
        "{TOML}[scene.alias]\n\"char2.recommendsites\" = [\"二人目メニュー\"]\n\"char2x.recommendsites\" = [\"違う\"]\n"
    );
    let text = "＊おすすめメニュー\n  さくら：あ。\n＊二人目メニュー\n  さくら：い。\n＊違う\n  さくら：う。\n";
    let lines: Vec<u32> = recommend_findings(&toml, text)
        .into_iter()
        .map(|f| f.0)
        .collect();
    assert_eq!(lines, [2, 4]);
}
