//! `＆重み`・`＆順番` の値と両方書きの読み込みエラー（scene-pick-modes 要件 3.2・4.2・4.4・4.5）
//!
//! - 1 以上の整数でない重み・整数でない順番は、位置と書いた値つきのトランスパイルエラーで止まる。
//! - 同じシーンに重みと順番の両方（ファイル属性から受け継いだ重み＋シーンの順番も）は誤り。
//! - 正しい値・同じ名前の 2 回書き（後勝ち）は通る。
//!
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use pasta_lua::loader::{LoaderError, PastaLoader};

/// 辞書を読み込み、トランスパイルの失敗の文を返す（通れば None）
fn load_failure_of(content: &str) -> Option<String> {
    let temp = common::create_temp_with_pasta(content);
    match PastaLoader::load(temp.path()) {
        Ok(_) => None,
        Err(LoaderError::PartialTranspileError { failures, .. }) => {
            assert_eq!(failures.len(), 1, "{failures:?}");
            Some(failures[0].error.clone())
        }
        Err(other) => panic!("expected PartialTranspileError, got {other:?}"),
    }
}

/// `＊挨拶` の 2 行目に `attr_line` を置いた辞書
fn load_failure(attr_line: &str) -> Option<String> {
    load_failure_of(&format!("＊挨拶\n  {attr_line}\n  さくら：こんにちは\n"))
}

#[test]
fn invalid_weight_stops_loading_with_position() {
    for (line, value) in [
        ("＆重み：0", "`0`"),
        ("＆重み：-1", "`-1`"),
        ("＆重み：1.5", "`1.5`"),
        ("＆重み：多め", "`多め`"),
        ("＆重み：「3」", "`「3」`"),
        ("＆weight：0", "`0`"),
    ] {
        let error =
            load_failure(line).unwrap_or_else(|| panic!("{line} は読み込みエラーになるべし"));
        assert!(
            error.contains(&format!("`＆重み` の値は 1 以上の整数で書く: {value}")),
            "{line}: {error}"
        );
        assert!(error.contains("[L2:"), "{line}: {error}");
    }
}

#[test]
fn invalid_order_stops_loading_with_position() {
    for (line, value) in [
        ("＆順番：1.5", "`1.5`"),
        ("＆順番：最初", "`最初`"),
        ("＆順番：「1」", "`「1」`"),
        ("＆index：first", "`first`"),
    ] {
        let error =
            load_failure(line).unwrap_or_else(|| panic!("{line} は読み込みエラーになるべし"));
        assert!(
            error.contains(&format!("`＆順番` の値は整数で書く: {value}")),
            "{line}: {error}"
        );
        assert!(error.contains("[L2:"), "{line}: {error}");
    }
}

#[test]
fn valid_weight_and_order_load() {
    for line in [
        "＆重み：1",
        "＆重み：５",
        "＆重み：5.0",
        "＆weight：3",
        "＆順番：0",
        "＆順番：-3",
        "＆順番：１",
        "＆順番：2.0",
        "＆index：7",
        // 同じ名前の 2 回書きは後勝ちで 1 つになり、誤りにしない（4.5）
        "＆重み：2＆重み：3",
        "＆順番：1＆index：2",
    ] {
        assert_eq!(load_failure(line), None, "{line} は通るべし");
    }
}

#[test]
fn weight_with_order_stops_loading_at_order_position() {
    for line in ["＆重み：2＆順番：1", "＆順番：1＆weight：2"] {
        let error =
            load_failure(line).unwrap_or_else(|| panic!("{line} は読み込みエラーになるべし"));
        assert!(
            error.contains("`＆重み` と `＆順番` は同じシーンに書けない"),
            "{line}: {error}"
        );
        assert!(error.contains("[L2:"), "{line}: {error}");
    }
}

#[test]
fn inherited_weight_with_scene_order_stops_loading() {
    // ファイル属性の重みを受け継いだシーンに順番を書くと、畳んだ後に両方あるので誤り（3.2）
    let content = "＆重み：2\n＊挨拶\n  ＆順番：1\n  さくら：こんにちは\n";
    let error = load_failure_of(content).expect("受け継いだ重み＋順番は読み込みエラーになるべし");
    assert!(
        error.contains("`＆重み` と `＆順番` は同じシーンに書けない"),
        "{error}"
    );
    // 位置は順番の属性（3 行目）
    assert!(error.contains("[L3:"), "{error}");
}

#[test]
fn invalid_weight_in_local_scene_stops_loading() {
    let content = "＊挨拶\n  さくら：こんにちは\n  ・小話＆重み：0\n  さくら：ねえ\n";
    let error =
        load_failure_of(content).expect("ローカルシーンの重み 0 は読み込みエラーになるべし");
    assert!(
        error.contains("`＆重み` の値は 1 以上の整数で書く: `0`"),
        "{error}"
    );
}
