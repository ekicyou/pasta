//! talk-grapheme-clusters: 台詞の「1 文字」を書記素クラスタで扱うことのテスト。
//!
//! 特性化テスト（要件 3.4・3.5・5.1・6.5）: 本体をクラスタ化する前の出力を固定する。
//! 期待値は以後書き換えない。台詞には 1 コードポイントのクラスタだけを使い、
//! `ﾞ`・`ﾟ` は前の字につながらない位置（単位の直後）にだけ置く。

use crate::common::create_sakura_test_runtime_with_config;
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script::tokenizer::{TokenKind, Tokenizer};

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
