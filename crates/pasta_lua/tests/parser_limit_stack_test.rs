//! 深さの上限とスタック（ponytail-dsl-parser 要件 3.4・3.6）
//!
//! - 上限ちょうどの 5 つの形（どれも演算子 256 個）を、1 MiB のスタックのスレッドで
//!   `LuaTranspiler::default().transpile` まで走らせ、あふれないことを確かめる。
//! - 65 段の括弧の行を含む辞書のゴーストを読み込むと、プロセスが落ちず、
//!   `parse_str` と同じ行・桁の構文の誤りで読み込みエラーになる。
//!
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use pasta_dsl::{ParseError, parse_str};
use pasta_lua::LuaTranspiler;
use pasta_lua::loader::{LoaderError, PastaLoader};

/// 1 MiB のスタックのスレッドで `f` を走らせる（あふれればテストのプロセスが落ちる）
fn on_1mib<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

/// `1` の後に `＋1` を 256 個つないだ式（演算子 256 個）
fn sum256() -> String {
    format!("1{}", "＋1".repeat(256))
}

/// `open` を n 段重ね、内側に演算子 256 個の式を置いて閉じる
fn nested(open: &str, n: usize) -> String {
    format!("{}{}{}", open.repeat(n), sum256(), "）".repeat(n))
}

/// 1 MiB のスレッドで解析・トランスパイル・片付けまで走らせる
fn transpile_on_1mib(src: String) {
    on_1mib(move || {
        let ast = parse_str(&src, "dic/a.pasta").expect("上限ちょうどは解析が通るべし");
        let mut output = Vec::new();
        LuaTranspiler::default()
            .transpile(&ast, &mut output)
            .expect("上限ちょうどはトランスパイルが通るべし");
        assert!(!output.is_empty());
    });
}

#[test]
fn at_limit_shapes_transpile_on_1mib() {
    let shapes = [
        // 関数呼び出し 64 段
        format!("＊挨拶\n  ＄x＝{}\n", nested("＠f（", 64)),
        // 括弧 64 段
        format!("＊挨拶\n  ＄x＝{}\n", nested("（", 64)),
        // 動的な関数呼び出し 64 段
        format!("＊挨拶\n  ＄x＝{}\n", nested("＠＄f（", 64)),
        // アクション行の関数呼び出し 64 段
        format!("＊挨拶\n  さくら：{}\n", nested("＠f（", 64)),
        // Call の引数の中の 63 段（Call の括弧と合わせて 64 段）
        format!("＊挨拶\n  ＞シーン（{}）\n", nested("（", 63)),
    ];
    for src in shapes {
        transpile_on_1mib(src);
    }
}

#[test]
fn loader_reports_65_level_paren_as_syntax_error() {
    // 2 行目の 65 段目の開き括弧は 5 + 65 = 70 桁目
    let content = format!("＊挨拶\n  ＄x＝{}１{}\n", "（".repeat(65), "）".repeat(65));
    let temp = common::create_temp_with_pasta(&content);
    let base = temp.path().to_path_buf();

    // 読み込みも 1 MiB のスレッドで走らせる（あふれればテストのプロセスが落ちる）
    on_1mib(move || {
        let err = PastaLoader::load(&base)
            .err()
            .expect("65 段の括弧の辞書の読み込みは失敗するべし");
        let message = err.to_string();
        let failures = match err {
            LoaderError::PartialTranspileError { failures, .. } => failures,
            other => panic!("expected PartialTranspileError, got {other:?}"),
        };
        assert_eq!(failures.len(), 1, "{message}");

        // ローダと同じファイル名で `parse_str` を呼び、同じ行・桁の構文の誤りであることを確かめる
        let filename = failures[0].source_path.to_string_lossy().to_string();
        let expected = parse_str(&content, &filename).unwrap_err();
        assert!(
            matches!(
                expected,
                ParseError::SyntaxError {
                    line: 2,
                    column: 70,
                    ..
                }
            ),
            "{expected:?}"
        );
        assert_eq!(failures[0].error, format!("Parse error: {expected}"));
    });
}
