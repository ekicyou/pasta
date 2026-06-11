//! `AnalysisEngine::analyze` の no-panic 境界回帰テスト。
//!
//! LSP サーバーはエディタ由来の信頼できないドキュメントテキストを受け取る。
//! `server.rs` / `transport.rs` は `catch_unwind` で防御しているが、
//! `analyze` 自体が敵対的入力（不完全なマーカー・閉じない括弧・CRLF 混在・
//! BOM・NUL バイト・絵文字識別子・全角半角混在マーカー等）でパニックしない
//! ことをここで直接固定する（パニック＝トークン/診断の全損失のため）。

use pasta_lsp::analysis::AnalysisEngine;

#[test]
fn test_analyze_does_not_panic_on_adversarial_corpus() {
    let corpus: Vec<(&str, String)> = vec![
        ("bare global marker", "＊\n".to_string()),
        ("bare local marker", "・\n".to_string()),
        ("bare actor marker", "％\n".to_string()),
        ("bare word marker", "＠\n".to_string()),
        ("bare var marker", "＄\n".to_string()),
        ("bare call marker", "＞\n".to_string()),
        ("bare cue marker", "　！\n".to_string()),
        ("bare attr marker", "＆\n".to_string()),
        ("var_set_none", "＊ｓ\n　＄＝\n".to_string()),
        ("var_set trailing op", "＊ｓ\n　＄ｘ＝１＋\n".to_string()),
        (
            "var_set unclosed paren",
            "＊ｓ\n　＄ｘ＝（１＋２\n".to_string(),
        ),
        (
            "var_set unclosed string",
            "＊ｓ\n　＄ｘ＝「あ\n".to_string(),
        ),
        (
            "var_set fullwidth space indent",
            "＊ｓ\n　＄ｘ＝１\n".to_string(),
        ),
        ("var_set tab indent", "＊ｓ\n\t＄ｘ＝１\n".to_string()),
        ("var_set mixed marker $＊", "＊ｓ\n　$＊ｘ=1\n".to_string()),
        ("var_set mixed marker ＄*", "＊ｓ\n　＄*ｘ=1\n".to_string()),
        ("var_set emoji name", "＊ｓ\n　＄😀＝１\n".to_string()),
        ("var_set property", "％ａ\n　＄％ｘ＝１\n".to_string()),
        ("var_set wordref", "＊ｓ\n　＄ｘ＝＠\n".to_string()),
        ("var_set global", "＊ｓ\n　＄＊ｇ＝１＋２＊３\n".to_string()),
        (
            "cue unclosed args",
            "＊ｓ\n　！ｃｍｄ（１、２\n".to_string(),
        ),
        (
            "cue scope",
            "＊ｓ\n　！ｃｍｄ＠ｓｃｏｐｅ（１）\n".to_string(),
        ),
        ("cue string arg", "＊ｓ\n　！ｃｍｄ（「あ」）\n".to_string()),
        ("cue atref arg", "＊ｓ\n　！ｃｍｄ（＠ｗ）\n".to_string()),
        ("action emoji actor", "＊ｓ\n　😀：こんにちは\n".to_string()),
        ("action no colon talk", "＊ｓ\n　あいう\n".to_string()),
        ("crlf var_set", "＊ｓ\r\n　＄ｘ＝１\r\n".to_string()),
        ("crlf cue", "＊ｓ\r\n　！ｃｍｄ（１）\r\n".to_string()),
        ("bom scene", "\u{feff}＊挨拶\n".to_string()),
        ("no trailing newline", "＊ｓ\n　＄ｘ＝１".to_string()),
        ("lone cr", "＊ｓ\r　＄ｘ＝１\n".to_string()),
        ("null byte", "＊ｓ\n　＄ｘ＝１\u{0}\n".to_string()),
        ("comment only", "＃ comment\n".to_string()),
        (
            "deep paren 50",
            format!("＊ｓ\n　＄ｘ＝{}１{}\n", "（".repeat(50), "）".repeat(50)),
        ),
        (
            "binary fullwidth ops",
            "＊ｓ\n　＄ｘ＝１－２／３％４\n".to_string(),
        ),
        (
            "var_set rhs varref prop",
            "＊ｓ\n　＄ｘ＝＄％ｐ\n".to_string(),
        ),
        (
            "var_set rhs fncall global",
            "＊ｓ\n　＄ｘ＝＠＊ｆ（１、ｋ：２）\n".to_string(),
        ),
        ("code block", "＊ｓ\n```lua\nreturn 1\n```\n".to_string()),
        ("long line", format!("＊ｓ\n　Ａ：{}\n", "あ".repeat(10000))),
    ];

    let mut panics = vec![];
    for (name, src) in &corpus {
        let src = src.clone();
        let r = std::panic::catch_unwind(move || {
            let _ = AnalysisEngine::analyze(&src);
        });
        if r.is_err() {
            panics.push(*name);
        }
    }
    assert!(panics.is_empty(), "panicking inputs: {:?}", panics);
}
