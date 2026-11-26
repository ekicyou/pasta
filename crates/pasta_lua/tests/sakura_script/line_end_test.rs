//! `talk_to_script` の 4 つ目の引数（行末）と、行末の値の決まる順の結合テスト。
//!
//! 値は ghost_dev と同じ `normal=90, period=1200, comma=600, strong=750, leader=300`。

use crate::common;

use common::create_sakura_test_runtime_with_config;
use pasta_lua::loader::TalkConfig;

fn ghost_dev() -> TalkConfig {
    TalkConfig {
        script_wait_normal: 90,
        script_wait_period: 1200,
        script_wait_comma: 600,
        script_wait_strong: 750,
        script_wait_leader: 300,
        ..Default::default()
    }
}

fn strong_eol(v: i64) -> TalkConfig {
    TalkConfig {
        script_wait_strong_eol: Some(v),
        ..ghost_dev()
    }
}

fn normal_eol(v: i64) -> TalkConfig {
    TalkConfig {
        script_wait_normal_eol: Some(v),
        ..ghost_dev()
    }
}

/// `SAKURA` を読み込んだ後に `body` を走らせ、返した文字列を受け取る。
fn run(config: &TalkConfig, body: &str) -> String {
    let lua = create_sakura_test_runtime_with_config(config);
    lua.load(format!(
        "local SAKURA = require \"@pasta_sakura_script\"\n{body}"
    ))
    .eval()
    .unwrap()
}

fn run_err(config: &TalkConfig, body: &str) -> String {
    let lua = create_sakura_test_runtime_with_config(config);
    lua.load(format!(
        "local SAKURA = require \"@pasta_sakura_script\"\n{body}"
    ))
    .eval::<String>()
    .unwrap_err()
    .to_string()
}

const SHAPE_ERROR: &str =
    "talk_to_script の line_ends は { pos = 整数, author_wait = 真偽 } の並びで渡してください";

// ============================================
// nil の既定（最後の見える字が行末）
// ============================================

#[test]
fn nil_の既定は最後の見える字が行末() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script(nil, "雨？\\s[5]")"#
        ),
        r"雨\_w[40]？\_w[1150]\s[5]"
    );
}

#[test]
fn 最後の見える字がエスケープなら待ちなし() {
    assert_eq!(
        run(
            &normal_eol(1250),
            r#"return SAKURA.talk_to_script(nil, "あ\\\\")"#
        ),
        r"あ\_w[40]\\"
    );
}

#[test]
fn 直後の作者の待ちは自動の待ちを置き換える() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script(nil, "雨？\\_w[500]")"#
        ),
        r"雨\_w[40]？\_w[500]"
    );
}

#[test]
fn 直後が作者の待ちでないタグなら自動の待ち() {
    for tag in [r"\\w5", r"\\__w[500]"] {
        assert_eq!(
            run(
                &strong_eol(1200),
                &format!(r#"return SAKURA.talk_to_script(nil, "雨？{tag}")"#)
            ),
            format!(r"雨\_w[40]？\_w[1150]{}", tag.replace(r"\\", r"\"))
        );
    }
}

#[test]
fn 見える字が無ければ行末なし() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script(nil, "\\s[5]")"#
        ),
        r"\s[5]"
    );
}

// ============================================
// 表で渡す行末
// ============================================

#[test]
fn 空の表は行末なし() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script(nil, "雨？", nil, {})"#
        ),
        r"雨\_w[40]？\_w[700]"
    );
}

#[test]
fn 書かれた行末だけを使う() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script(nil, "雨？雨？", nil, { { pos = 6 } })"#
        ),
        r"雨\_w[40]？\_w[1150]雨\_w[40]？\_w[700]"
    );
}

#[test]
fn 作者の待ちの印を受け取る() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script(nil, "雨？", nil, { { pos = 6, author_wait = true } })"#
        ),
        r"雨\_w[40]？"
    );
}

#[test]
fn 範囲の外と境目に当たらない位置は無視する() {
    for pos in ["0", "-1", "7", "4", "9007199254740993"] {
        assert_eq!(
            run(
                &strong_eol(1200),
                &format!(
                    r#"return SAKURA.talk_to_script(nil, "雨？", nil, {{ {{ pos = {pos} }} }})"#
                )
            ),
            r"雨\_w[40]？\_w[700]",
            "pos = {pos}"
        );
    }
}

#[test]
fn 形が違えば決めた文言のエラー() {
    for arg in [
        "5",
        r#""x""#,
        "true",
        "{ 5 }",
        "{ {} }",
        r#"{ { pos = "abc" } }"#,
    ] {
        let err = run_err(
            &strong_eol(1200),
            &format!(r#"return SAKURA.talk_to_script(nil, "雨？", nil, {arg})"#),
        );
        assert!(err.contains(SHAPE_ERROR), "{arg}: {err}");
    }
}

// ============================================
// 値の決まる順
// ============================================

#[test]
fn アクターの行末だけ() {
    assert_eq!(
        run(
            &ghost_dev(),
            r#"return SAKURA.talk_to_script({ script_wait_strong_eol = 1000 }, "雨？")"#
        ),
        r"雨\_w[40]？\_w[950]"
    );
}

#[test]
fn talk_の行末だけ() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script({}, "雨？")"#
        ),
        r"雨\_w[40]？\_w[1150]"
    );
}

#[test]
fn アクターの行末は_talk_の行末より先() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script({ script_wait_strong_eol = 1000 }, "雨？")"#
        ),
        r"雨\_w[40]？\_w[950]"
    );
}

#[test]
fn 行の途中だけを変えたアクターは_talk_の行末を使う() {
    assert_eq!(
        run(
            &strong_eol(1200),
            r#"return SAKURA.talk_to_script({ script_wait_strong = 300 }, "雨？雨？")"#
        ),
        r"雨\_w[40]？\_w[250]雨\_w[40]？\_w[1150]"
    );
}

#[test]
fn lua_で書き換えた次の呼び出しから効く() {
    assert_eq!(
        run(
            &ghost_dev(),
            r#"
            local a = {}
            local before = SAKURA.talk_to_script(a, "雨？")
            a.script_wait_strong_eol = 1200
            return before .. "|" .. SAKURA.talk_to_script(a, "雨？")
            "#
        ),
        r"雨\_w[40]？\_w[700]|雨\_w[40]？\_w[1150]"
    );
}

#[test]
fn アクターの値の読み方() {
    for (value, expected) in [
        ("1200.0", r"雨\_w[40]？\_w[1150]"),
        (r#""1200""#, r"雨\_w[40]？\_w[1150]"),
        (r#""abc""#, r"雨\_w[40]？\_w[700]"),
        ("true", r"雨\_w[40]？\_w[700]"),
    ] {
        assert_eq!(
            run(
                &ghost_dev(),
                &format!(
                    r#"return SAKURA.talk_to_script({{ script_wait_strong_eol = {value} }}, "雨？")"#
                )
            ),
            expected,
            "{value}"
        );
    }
}

// ============================================
// 今の呼び方
// ============================================

#[test]
fn 行末のキーが無ければ_3_つの引数の呼び方は今と同じ() {
    assert_eq!(
        run(
            &ghost_dev(),
            r#"
            local a = { script_wait_strong = 750 }
            return SAKURA.talk_to_script(a, "雨？\\s[5]", {})
                .. "|" .. SAKURA.talk_to_script(nil, "雨？\\_w[500]")
                .. "|" .. SAKURA.talk_to_script(a, "雨？", nil, { { pos = 6 } })
            "#
        ),
        r"雨\_w[40]？\_w[700]\s[5]|雨\_w[40]？\_w[700]\_w[500]|雨\_w[40]？\_w[700]"
    );
}
