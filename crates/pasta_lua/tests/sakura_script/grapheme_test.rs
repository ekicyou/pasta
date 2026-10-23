//! talk-grapheme-clusters: 台詞の「1 文字」を書記素クラスタで扱うことのテスト。
//!
//! 特性化テスト（要件 3.4・3.5・5.1・6.5）: 本体をクラスタ化する前の出力を固定する。
//! 期待値は以後書き換えない。台詞には 1 コードポイントのクラスタだけを使い、
//! `ﾞ`・`ﾟ` は前の字につながらない位置（単位の直後）にだけ置く。

use crate::common::create_sakura_test_runtime_with_config;
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script::tokenizer::{CharSets, TokenKind, Tokenizer};

// ============================================
// 特性化テスト 1: 既定の chars_* の各文字の種類
// ============================================

/// 既定の `chars_*` の全文字と、今の種類。`･` は読点より前に判定するリーダーに入る。
const DEFAULT_CHAR_KINDS: &[(&str, TokenKind)] = &[
    ("｡。．.", TokenKind::Period),
    ("、，,", TokenKind::Comma),
    ("？！!?", TokenKind::Strong),
    ("･・‥…", TokenKind::Leader),
    (
        "゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､ｰﾞﾟ",
        TokenKind::LineStartProhibited,
    ),
    ("（［｛「『([{｢", TokenKind::LineEndProhibited),
];

#[test]
fn test_default_chars_each_char_kind() {
    let config = TalkConfig::default();
    let tokenizer = Tokenizer::new(&config).unwrap();

    // 表が既定の chars_* の全文字を覆っていること
    let all_defaults = [
        &config.chars_period,
        &config.chars_comma,
        &config.chars_strong,
        &config.chars_leader,
        &config.chars_line_start_prohibited,
        &config.chars_line_end_prohibited,
    ];
    for c in all_defaults.iter().flat_map(|s| s.chars()) {
        assert!(
            DEFAULT_CHAR_KINDS.iter().any(|(s, _)| s.contains(c)),
            "表に無い既定の文字: {c:?}"
        );
    }

    for (chars, kind) in DEFAULT_CHAR_KINDS {
        for c in chars.chars() {
            let text = c.to_string();
            let tokens = tokenizer.tokenize(&text);
            assert_eq!(tokens.len(), 1, "{c:?}");
            assert_eq!(tokens[0].text, text, "{c:?}");
            assert_eq!(tokens[0].kind, *kind, "{c:?}");
        }
    }
}

// ============================================
// 特性化テスト 2: 混ぜた台詞の talk_to_script の出力
// ============================================

/// 既定の chars_* の全文字・ひらがな・漢字・ASCII・半角カナ（濁点なし）・単独の `\`・タグを混ぜた台詞。
/// `ﾞ`・`ﾟ` は単位（`\s[…]`）の直後に置き、前の字とつながらないようにする。
const MIXED_TALK: &str = r"こんにちは、世界。Hello, pasta 123!\h｡．.，,？！!?･・‥…\s[0]ﾞ゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､ｰ\s[10]ﾟ（［｛「『([{｢ｱｲｳｴｵ\あ漢字です";

/// 一般の字にもウェイトが見えるように、script_wait_normal だけ既定から変える。
fn mixed_talk_config() -> TalkConfig {
    TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    }
}

#[test]
fn test_mixed_talk_output_is_fixed() {
    let out: String = create_sakura_test_runtime_with_config(&mixed_talk_config())
        .load(format!(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, [==[{MIXED_TALK}]==])
            "#
        ))
        .eval()
        .unwrap();
    assert_eq!(
        out,
        r"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]、\_w[450]世\_w[50]界\_w[50]。\_w[950]H\_w[50]e\_w[50]l\_w[50]l\_w[50]o\_w[50],\_w[450] \_w[50]p\_w[50]a\_w[50]s\_w[50]t\_w[50]a\_w[50] \_w[50]1\_w[50]2\_w[50]3\_w[50]!\_w[450]\h｡．.，,？！!?\_w[950]･\_w[150]・\_w[150]‥\_w[150]…\_w[150]\s[0]ﾞ゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､ｰ\s[10]ﾟ（［｛「『([{｢ｱ\_w[50]ｲ\_w[50]ｳ\_w[50]ｴ\_w[50]ｵ\_w[50]\あ\_w[50]漢\_w[50]字\_w[50]で\_w[50]す\_w[50]"
    );
}

#[test]
fn test_mixed_talk_output_with_budoux_is_fixed() {
    let (out, line, width, jump): (String, i64, i64, bool) =
        create_sakura_test_runtime_with_config(&mixed_talk_config())
            .load(format!(
                r#"
                local SAKURA = require "@pasta_sakura_script"
                local col = {{}}
                local out = SAKURA.talk_to_script({{ budoux = {{8}} }}, [==[{MIXED_TALK}]==], col)
                return out, col.line, col.width, col.jump
                "#
            ))
            .eval()
            .unwrap();
    assert_eq!(
        out,
        r"こ\_w[50]ん\_w[50]に\_w[50]\nち\_w[50]は\_w[50]、\_w[450]\n世\_w[50]界\_w[50]。\_w[950]\nH\_w[50]e\_w[50]l\_w[50]l\_w[50]o\_w[50],\_w[450] \_w[50]p\_w[50]a\_w[50]s\_w[50]t\_w[50]a\_w[50] \_w[50]1\_w[50]2\_w[50]3\_w[50]!\_w[450]\h｡．\n.，,？！\n!?\_w[950]･\_w[150]・\_w[150]‥\_w[150]…\_w[150]\s[0]\nﾞ゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､ｰ\s[10]ﾟ\n（［｛\n「『([{｢ｱ\_w[50]ｲ\_w[50]ｳ\_w[50]ｴ\_w[50]ｵ\_w[50]\あ\_w[50]漢\_w[50]字\_w[50]で\_w[50]す\_w[50]"
    );
    assert_eq!((line, width, jump), (9, 24, false));
}

// ============================================
// 文字の種類の判定（CharSets::classify をクラスタの文字列で直接呼ぶ）
// ============================================

const WOMAN_GESTURING_OK: &str = "\u{1F646}\u{200D}\u{2640}\u{FE0F}"; // 🙆‍♀️

/// 要件 3.2・3.7: 複数コードポイントの項目は全体で当たり、その部品も単独で当たる。
#[test]
fn test_classify_multi_codepoint_item_and_its_parts() {
    let config = TalkConfig {
        chars_period: format!("。{WOMAN_GESTURING_OK}"),
        ..TalkConfig::default()
    };
    let sets = CharSets::from_config(&config);
    assert_eq!(sets.classify(WOMAN_GESTURING_OK), TokenKind::Period);
    assert_eq!(sets.classify("\u{2640}"), TokenKind::Period);
    assert_eq!(sets.classify("。"), TokenKind::Period);
}

/// 要件 3.3: 2 つ目以降のコードポイントでは種類を決めない。
#[test]
fn test_classify_ignores_non_first_codepoints() {
    let config = TalkConfig {
        chars_period: "\u{2640}\u{FE0F}".to_string(),
        ..TalkConfig::default()
    };
    let sets = CharSets::from_config(&config);
    assert_eq!(sets.classify(WOMAN_GESTURING_OK), TokenKind::General);
}

/// 要件 3.6: 全体で外れたら、先頭のコードポイントで判定する（`❗️` = `❗` + U+FE0F）。
#[test]
fn test_classify_falls_back_to_first_codepoint() {
    let config = TalkConfig {
        chars_strong: "\u{2757}".to_string(),
        ..TalkConfig::default()
    };
    let sets = CharSets::from_config(&config);
    assert_eq!(sets.classify("\u{2757}\u{FE0F}"), TokenKind::Strong);
}

// ============================================
// 単位でない所を書記素クラスタ 1 つずつ切り出す（要件 1.1・1.3・1.4・1.5・5.3）
// ============================================

fn tokenize(input: &str) -> Vec<(TokenKind, String)> {
    Tokenizer::new(&TalkConfig::default())
        .unwrap()
        .tokenize(input)
        .into_iter()
        .map(|t| (t.kind, t.text))
        .collect()
}

fn general(s: &str) -> (TokenKind, String) {
    (TokenKind::General, s.to_string())
}

fn sakura(s: &str) -> (TokenKind, String) {
    (TokenKind::SakuraScript, s.to_string())
}

#[test]
fn test_zwj_emoji_is_one_token() {
    assert_eq!(
        tokenize(&format!("あ{WOMAN_GESTURING_OK}い")),
        vec![general("あ"), general(WOMAN_GESTURING_OK), general("い")]
    );
}

/// Prepend（U+0600）の後ろの `\` は単位の頭として残す。
#[test]
fn test_prepend_does_not_take_unit() {
    assert_eq!(
        tokenize("\u{600}\\h"),
        vec![general("\u{600}"), sakura(r"\h")]
    );
    assert_eq!(
        tokenize("\u{600}\\あ"),
        vec![general("\u{600}"), general("\\"), general("あ")]
    );
}

/// 単位の直後の結合文字は前の字につながらず単独のトークンになる。
#[test]
fn test_combining_after_unit_is_alone() {
    assert_eq!(
        tokenize("\\n\u{301}あ"),
        vec![sakura(r"\n"), general("\u{301}"), general("あ")]
    );
}

/// 単位にならない `\` には後ろの結合文字がつながる。
#[test]
fn test_lone_backslash_takes_combining_mark() {
    assert_eq!(tokenize("\\\u{301}"), vec![general("\\\u{301}")]);
}

#[test]
fn test_halfwidth_kana_with_voiced_mark_is_one_token() {
    assert_eq!(tokenize("ｶﾞ"), vec![general("ｶﾞ")]);
}

// ============================================
// budoux の改行と幅をクラスタで数える（要件 4.1〜4.8）
// ============================================

/// budoux を `widths` にしたアクターで `talk_to_script` を呼び、出力と終わりの列の字数を返す。
fn talk_budoux_width(widths: &str, input: &str) -> (String, i64) {
    crate::common::create_sakura_test_runtime()
        .load(format!(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local col = {{}}
            local out = SAKURA.talk_to_script({{ budoux = {{{widths}}} }}, [==[{input}]==], col)
            return out, col.width
            "#
        ))
        .eval()
        .unwrap()
}

/// 要件 4.3・4.4: 組み合わせの絵文字は 1 つで幅 2。
#[test]
fn test_budoux_counts_emoji_cluster_as_width_two() {
    for emoji in [
        WOMAN_GESTURING_OK,
        "\u{270C}\u{FE0F}",
        "\u{1F44D}\u{1F3FD}",
        "\u{1F1EF}\u{1F1F5}",
    ] {
        let (out, width) = talk_budoux_width("100", emoji);
        assert_eq!(out, emoji);
        assert_eq!(width, 2, "{emoji:?}");
    }
}

/// 要件 4.5: 1 コードポイントのクラスタと `\r\n` の幅は今と同じ（曖昧な字は 2・制御文字は 0）。
#[test]
fn test_budoux_single_codepoint_and_crlf_width_unchanged() {
    assert_eq!(talk_budoux_width("100", "a♀あ").1, 5);
    assert_eq!(talk_budoux_width("100", "あ\r\nい").1, 4);
}

/// 要件 4.1・4.2・4.8: 改行はクラスタの境界にだけ入り、クラスタの中身は続けて出る。
#[test]
fn test_budoux_breaks_only_at_cluster_boundaries() {
    let clusters = [
        "今日",
        WOMAN_GESTURING_OK,
        "は",
        "\u{270C}\u{FE0F}",
        "い",
        "\u{1F44D}\u{1F3FD}",
        "い",
        "\u{1F1EF}\u{1F1F5}",
        "天気",
        "か\u{3099}",
        "e\u{301}",
        "です",
        WOMAN_GESTURING_OK,
        WOMAN_GESTURING_OK,
        "ね",
    ];
    let input: String = clusters.concat();
    for widths in ["2", "3", "4", "6"] {
        let (out, _) = talk_budoux_width(widths, &input);
        assert_breaks_between(&clusters, &out);
    }
}

/// `out` を `\n` で割ったかけらをつなぐと `clusters` をつないだものに戻り、割れ目が全部クラスタの境界にある。
/// 割れ目の数を返す。
fn assert_breaks_between(clusters: &[&str], out: &str) -> usize {
    let mut starts = vec![0];
    for c in clusters {
        starts.push(starts.last().unwrap() + c.len());
    }
    let pieces: Vec<&str> = out.split(r"\n").collect();
    assert_eq!(pieces.concat(), clusters.concat(), "{out}");
    let mut pos = 0;
    for p in &pieces[..pieces.len() - 1] {
        pos += p.len();
        assert!(starts.contains(&pos), "クラスタの途中の改行: {out}");
    }
    pieces.len() - 1
}

/// `SAKURA.break_lines` を Lua から呼ぶ。
fn break_lines(input: &str, widths: &str) -> String {
    crate::common::create_sakura_test_runtime()
        .load(format!(
            r#"return require("@pasta_sakura_script").break_lines([==[{input}]==], {{{widths}}})"#
        ))
        .eval()
        .unwrap()
}

/// 要件 4.1・4.2・6.4: `break_lines` で組み合わせの絵文字・結合文字を並べた台詞を狭い幅で割る。
#[test]
fn test_break_lines_breaks_emoji_run_between_clusters() {
    // 絵文字だけが続くと budoux は区切らないので、語を挟む。
    // `か` ＋ U+3099 はコードポイントで割ると budoux が途中を区切る並び
    let (w, ga) = (WOMAN_GESTURING_OK, "か\u{3099}");
    let thumbs = "\u{1F44D}\u{1F3FD}";
    let jp = "\u{1F1EF}\u{1F1F5}";
    let clusters = [
        "今日", "は", w, w, w, "いい", "天気", w, w, "です", "ね", w, w, w, "明日", "は", ga,
        thumbs, thumbs, "雨", jp, jp, "の", "e\u{301}", "e\u{301}", "天気", ga, ga, ga, "今日",
    ];
    for widths in ["2", "3", "4", "6"] {
        let out = break_lines(&clusters.concat(), widths);
        assert!(
            assert_breaks_between(&clusters, &out) > 0,
            "割れていない widths={widths}: {out}"
        );
    }
}

/// 要件 4.3・4.8・6.4: 縦書きのアクター（`budoux` と `budoux_newlines`）でも `🙆‍♀️` は続けて出て、幅 2 で数える。
/// 語は「今日は」「🙆‍♀️いい」「天気」。`🙆‍♀️` を幅 4 と数えると「🙆‍♀️いい」の前で割れる。
#[test]
fn test_vertical_actor_keeps_emoji_cluster_together() {
    let out: String = crate::common::create_sakura_test_runtime()
        .load(format!(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = {{ budoux = {{12}}, budoux_newlines = 1.5 }}
            return SAKURA.talk_to_script(actor, [==[今日は{WOMAN_GESTURING_OK}いい天気]==])
            "#
        ))
        .eval()
        .unwrap();
    assert_eq!(out, format!(r"今日は{WOMAN_GESTURING_OK}いい\n[150]天気"));
}

/// 要件 4.7（決定 9）: 単独の `\` に続く語で上限を超えたら `\` の前で割る。
/// `\` ＋ U+0301 のクラスタは単独の `\` ではないので、その後ろで割る（`\\` にならない）。
#[test]
fn test_budoux_lone_backslash_guard_only_for_bare_backslash() {
    assert_eq!(break_lines(r"天気天気\今日は", "4"), r"天気天気\n\今日は");
    assert_eq!(
        break_lines("天気天気\\\u{301}今日は", "4"),
        "天気天気\\\u{301}\\n今日は"
    );
}

/// 要件 4.1・4.7: `\` ＋ 結合文字のクラスタは割れずに 1 つで出て、`\\` を作らない。
#[test]
fn test_budoux_backslash_with_combining_mark_stays_one_cluster() {
    let (out, _) = talk_budoux_width("4", "今日は\\\u{301}いい天気ですね");
    assert_eq!(out, "今日は\\n\\\u{301}いい\\n天気ですね");
}

// ============================================
// 退行テスト: talk_to_script の自動ウェイト（要件 6.1・6.2・6.3・6.6・5.3）
// ============================================

/// `config` で `talk_to_script` を呼び、出力を返す。
fn talk(config: &TalkConfig, input: &str) -> String {
    create_sakura_test_runtime_with_config(config)
        .load(format!(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, [==[{input}]==])
            "#
        ))
        .eval()
        .unwrap()
}

/// 要件 2.3・2.4・6.1: 報告の 2 例。
#[test]
fn test_reported_examples_wait_once_per_cluster() {
    let config = TalkConfig {
        script_wait_normal: 90,
        ..Default::default()
    };
    assert_eq!(
        talk(&config, &format!("りょ{WOMAN_GESTURING_OK}✨")),
        format!(r"り\_w[40]ょ\_w[40]{WOMAN_GESTURING_OK}\_w[40]✨\_w[40]")
    );
    assert_eq!(
        talk(&mixed_talk_config(), "\u{270C}\u{FE0F}"),
        "\u{270C}\u{FE0F}\\_w[50]"
    );
}

/// 要件 3.2・6.2: chars_period に書いた組み合わせの絵文字の後ろに句点のウェイト。
#[test]
fn test_multi_codepoint_chars_period_gets_period_wait() {
    let config = TalkConfig {
        chars_period: format!("。{WOMAN_GESTURING_OK}"),
        ..mixed_talk_config()
    };
    assert_eq!(
        talk(&config, &format!("あ{WOMAN_GESTURING_OK}い")),
        format!(r"あ\_w[50]{WOMAN_GESTURING_OK}\_w[950]い\_w[50]")
    );
    // 要件 3.3: 部品（♀・U+FE0F）だけを書いても、クラスタは一般のまま 1 つのウェイト
    let config = TalkConfig {
        chars_period: "\u{2640}\u{FE0F}".to_string(),
        ..mixed_talk_config()
    };
    assert_eq!(
        talk(&config, WOMAN_GESTURING_OK),
        format!(r"{WOMAN_GESTURING_OK}\_w[50]")
    );
}

/// 要件 1.5・6.3・5.3: 肌色・国旗 2 つ・結合文字・半角カナ＋濁点の途中にウェイトが無い。
#[test]
fn test_no_wait_inside_clusters() {
    for cluster in [
        "\u{1F44D}\u{1F3FD}",
        "\u{1F1EF}\u{1F1F5}",
        "\u{1F1FA}\u{1F1F8}",
        "e\u{301}",
        "ｶﾞ",
    ] {
        assert_eq!(
            talk(&mixed_talk_config(), &format!("あ{cluster}")),
            format!(r"あ\_w[50]{cluster}\_w[50]"),
            "{cluster:?}"
        );
    }
    // 2 つの国旗が並んでも、国旗の間にだけウェイトが入る
    assert_eq!(
        talk(&mixed_talk_config(), "\u{1F1EF}\u{1F1F5}\u{1F1FA}\u{1F1F8}"),
        "\u{1F1EF}\u{1F1F5}\\_w[50]\u{1F1FA}\u{1F1F8}\\_w[50]"
    );
}

/// 要件 1.3・1.4・2.6・6.6: Prepend の後ろの単位、単位の直後の結合文字、単独の `\` の守り。
#[test]
fn test_unit_boundaries_and_lone_backslash_waits() {
    let config = mixed_talk_config();
    // `\h` の後ろにはウェイトが無い
    assert_eq!(talk(&config, "\u{600}\\h"), "\u{600}\\_w[50]\\h");
    // 単独の `\` の直後にはウェイトが無い
    assert_eq!(talk(&config, "\u{600}\\あ"), "\u{600}\\_w[50]\\あ\\_w[50]");
    // 単位の直後の U+0301 は単独のクラスタで、後ろに一般のウェイト
    assert_eq!(
        talk(&config, "\\n\u{301}あ"),
        "\\n\u{301}\\_w[50]あ\\_w[50]"
    );
    // `\` ＋ U+0301 のクラスタの後ろにはウェイトが入る
    assert_eq!(talk(&config, "\\\u{301}"), "\\\u{301}\\_w[50]");
}
