//! `＆優先` の値の読み込みエラー（scene-condition-attributes 要件 5.5・5.6）
//!
//! - 整数でない優先（`＆優先：高い`・引用した `「1」`・小数の `1.5`、別名 `＆priority`）を書いた辞書は、
//!   位置つきのトランスパイルエラーで読み込みが止まる。
//! - 負の数・全角の数・整数の値になる小数（`-1`・`１`・`1.0`）は通る。
//!
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use pasta_lua::loader::{LoaderError, PastaLoader};

/// `＊挨拶` の 2 行目に `attr_line` を置いた辞書を読み込み、トランスパイルの失敗の文を返す（通れば None）
fn load_failure(attr_line: &str) -> Option<String> {
    let content = format!("＊挨拶\n  {attr_line}\n  さくら：こんにちは\n");
    let temp = common::create_temp_with_pasta(&content);
    match PastaLoader::load(temp.path()) {
        Ok(_) => None,
        Err(LoaderError::PartialTranspileError { failures, .. }) => {
            assert_eq!(failures.len(), 1, "{failures:?}");
            Some(failures[0].error.clone())
        }
        Err(other) => panic!("expected PartialTranspileError, got {other:?}"),
    }
}

#[test]
fn non_integer_priority_stops_loading_with_position() {
    for (line, value) in [
        ("＆優先：高い", "`高い`"),
        ("＆優先：「1」", "`「1」`"),
        ("＆優先：1.5", "`1.5`"),
        ("＆priority：high", "`high`"),
    ] {
        let error =
            load_failure(line).unwrap_or_else(|| panic!("{line} は読み込みエラーになるべし"));
        assert!(
            error.contains(&format!("`＆優先` の値は整数で書く: {value}")),
            "{line}: {error}"
        );
        // 位置は今のトランスパイルエラーと同じ `[L行:桁-L行:桁]`（2 行目）
        assert!(error.contains("[L2:"), "{line}: {error}");
    }
}

#[test]
fn integer_priority_loads() {
    for line in ["＆優先：-1", "＆優先：１", "＆優先：1.0", "＆priority：3"] {
        assert_eq!(load_failure(line), None, "{line} は通るべし");
    }
}
