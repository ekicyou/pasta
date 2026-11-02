//! 深さの守り（ponytail-dsl-parser 要件 3.1・3.3〜3.6）
//!
//! 1 行の閉じていない開き括弧の重なりは 64 段まで。65 段目の開き括弧の位置の
//! 構文の誤りにする。台詞・コメント・Lua ブロックの括弧も数える。
//! どのテストも 1 MiB のスタックのスレッドで、解析と AST の片付けまで走らせる。

use pasta_dsl::{ParseError, parse_str, parse_str_partial};

const PAREN_MSG: &str = "括弧の重なりが 64 段を超えています";

/// 1 MiB のスタックのスレッドで `f` を走らせる（あふれればテストのプロセスが落ちる）
fn on_1mib<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

/// 解析して AST を片付けたうえで、誤りだけを返す
fn parse_err(src: String) -> Option<ParseError> {
    on_1mib(move || parse_str(&src, "dic/a.pasta").err())
}

fn assert_paren_err(src: String, line: usize, column: usize) {
    match parse_err(src) {
        Some(ParseError::SyntaxError {
            file,
            line: l,
            column: c,
            message,
        }) => {
            assert_eq!(file, "dic/a.pasta");
            assert_eq!((l, c), (line, column), "{message}");
            assert_eq!(
                message,
                format!("Parse error in dic/a.pasta at {line}:{column}: {PAREN_MSG}")
            );
        }
        other => panic!("expected paren SyntaxError, got {other:?}"),
    }
}

/// `＊挨拶\n  ＄x＝` の後に `open` を n 段重ね、`１` と閉じ括弧で閉じた辞書
fn nested(open: &str, n: usize) -> String {
    format!("＊挨拶\n  ＄x＝{}１{}\n", open.repeat(n), "）".repeat(n))
}

/// `nested` で 65 段目の開き括弧がある桁（行の頭の `  ＄x＝` が 5 文字）
fn col_65th(open: &str) -> usize {
    5 + open.chars().count() * 64 + open.chars().count()
}

#[test]
fn paren_64_levels_pass() {
    for open in ["（", "＠f（", "＠＄f（"] {
        assert!(
            parse_err(nested(open, 64)).is_none(),
            "{open} 64 段は通るべし"
        );
    }
}

#[test]
fn paren_65_levels_error_at_65th() {
    assert_eq!(col_65th("（"), 70);
    for open in ["（", "＠f（", "＠＄f（"] {
        assert_paren_err(nested(open, 65), 2, col_65th(open));
    }
}

#[test]
fn paren_ascii_open_counts() {
    assert_paren_err(nested("(", 65), 2, col_65th("("));
}

#[test]
fn paren_10000_levels_error_without_crash() {
    for open in ["（", "＠f（", "＠＄f（"] {
        assert_paren_err(nested(open, 10_000), 2, col_65th(open));
    }
}

#[test]
fn paren_in_dialogue_comment_lua_block_counts() {
    let p65 = "（".repeat(65);
    // 台詞（`  さくら：` は 6 文字）
    assert_paren_err(format!("＊挨拶\n  さくら：{p65}\n"), 2, 6 + 65);
    // コメント
    assert_paren_err(format!("＊挨拶\n  さくら：やあ\n＃{p65}\n"), 3, 1 + 65);
    // Lua ブロック
    assert_paren_err(
        format!(
            "＊挨拶\n  さくら：やあ\n```lua\nlocal x = {}\n```\n",
            "(".repeat(65)
        ),
        4,
        10 + 65,
    );
}

#[test]
fn paren_depth_resets_per_line_and_crlf() {
    // 64 段の行を何行重ねても、行ごとに 0 へ戻る
    let line = format!("  さくら：{}\r\n", "（".repeat(64));
    assert!(parse_err(format!("＊挨拶\r\n{}", line.repeat(3))).is_none());
    // CRLF・BOM でも行と桁は pest と同じ数え方
    assert_paren_err(
        format!("\u{FEFF}＊挨拶\r\n  さくら：{}\r\n", "（".repeat(65)),
        2,
        6 + 65,
    );
}

#[test]
fn paren_extra_close_never_goes_negative() {
    // 閉じ括弧が先に 10 個あっても、その後の 65 段目で誤りになる
    let src = format!("＊挨拶\n  さくら：{}{}\n", "）".repeat(10), "（".repeat(65));
    assert_paren_err(src, 2, 6 + 10 + 65);
    // 閉じ括弧だけの行は誤りにならない
    assert!(parse_err(format!("＊挨拶\n  さくら：{}\n", ")".repeat(100))).is_none());
}

#[test]
fn partial_parse_with_deep_line_does_not_crash() {
    for n in [65, 10_000] {
        let src = format!(
            "＊挨拶\n  さくら：こんにちは\n{}＊別\n  さくら：またね\n",
            nested("（", n).trim_start_matches("＊挨拶\n")
        );
        let result = on_1mib(move || {
            let r = parse_str_partial(&src);
            r.errors.len()
        });
        assert!(result > 0, "{n} 段の行は誤りとして報告されるべし");
    }
}
