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

// ---- 1 つの式の二項演算子は 256 個まで（要件 3.2・3.3・3.5・3.6）----

const OPS_MSG: &str = "1 つの式の演算子が 256 個を超えています";

/// `1` の後に `＋1` を n 個つないだ式（演算子 n 個）
fn sum(n: usize) -> String {
    format!("1{}", "＋1".repeat(n))
}

/// 演算子 256 個までを書いた `head` の直後に 257 個目の `＋` を置いた辞書の、その `＋` の桁で誤りになることを確かめる
fn assert_op_err(head: &str, tail: &str) {
    let line_head = head.rsplit('\n').next().unwrap();
    let column = line_head.chars().count() + 1;
    let line = head.matches('\n').count() + 1;
    match parse_err(format!("{head}＋{tail}")) {
        Some(ParseError::SyntaxError {
            line: l,
            column: c,
            message,
            ..
        }) => {
            assert_eq!((l, c), (line, column), "{message}");
            assert_eq!(
                message,
                format!("Parse error in dic/a.pasta at {line}:{column}: {OPS_MSG}")
            );
        }
        other => panic!("expected operator SyntaxError, got {other:?}"),
    }
}

#[test]
fn ops_256_pass() {
    assert!(parse_err(format!("＊挨拶\n  ＄x＝{}\n", sum(256))).is_none());
    // 括弧 64 段の内側に演算子 256 個を置いた、上限ちょうどの形
    let src = format!(
        "＊挨拶\n  ＄x＝{}{}{}\n",
        "（".repeat(64),
        sum(256),
        "）".repeat(64)
    );
    assert!(parse_err(src).is_none());
}

#[test]
fn ops_257_error_at_257th() {
    // 変数の代入の右辺
    assert_op_err(&format!("＊挨拶\n  ＄x＝{}", sum(256)), "1\n");
    // 関数呼び出しの位置の引数
    assert_op_err(&format!("＊挨拶\n  さくら：＠f（{}", sum(256)), "1）\n");
    // 関数呼び出しの名前つきの引数
    assert_op_err(&format!("＊挨拶\n  さくら：＠f（k：{}", sum(256)), "1）\n");
}

#[test]
fn ops_across_parens_count_together() {
    // （128 個）＋（128 個）で合わせて 257 個。257 個目は右の括弧の最後の `＋`
    let head = format!("＊挨拶\n  ＄x＝（{}）＋（{}", sum(128), sum(127));
    assert_op_err(&head, "1）\n");
}

#[test]
fn ops_in_two_fn_calls_are_separate() {
    let src = format!("＊挨拶\n  さくら：＠f（{}）＠g（{}）\n", sum(200), sum(200));
    assert!(parse_err(src).is_none());
    // 引数は式として数えている（片方を 257 個にすると誤り）
    assert_op_err(
        &format!("＊挨拶\n  さくら：＠f（{}）＠g（{}", sum(200), sum(256)),
        "1）\n",
    );
}

#[test]
fn ops_10000_error_without_crash() {
    let full = sum(10_000);
    // 257 個目の `＋` の前までが `sum(256)`
    let (head, tail) = full.split_at(sum(256).len());
    assert_op_err(
        &format!("＊挨拶\n  ＄x＝{head}"),
        &format!("{}\n", &tail["＋".len()..]),
    );
}

#[test]
fn partial_parse_with_long_expr_line_does_not_crash() {
    let src = format!(
        "＊挨拶\n  さくら：こんにちは\n  ＄x＝{}\n＊別\n  さくら：またね\n",
        sum(10_000)
    );
    let errors = on_1mib(move || parse_str_partial(&src).errors.len());
    assert!(errors > 0, "1 万個の演算子の行は誤りとして報告されるべし");
}
