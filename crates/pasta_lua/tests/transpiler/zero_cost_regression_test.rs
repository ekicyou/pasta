//! ゼロコスト回帰テスト基盤（バイト不変スナップショット）。
//!
//! 全 `generate_*` への `record_span` 配線が **デバッグ無効時の生成 `.lua` を 1 バイトも変えていない** ことを
//! 保証する安全網。本番 transpile 経路（`LuaTranspiler::transpile`）は source-map sink を一切装着しない
//! ＝ デバッグ OFF 経路なので、その出力を受理済みベースラインとして固定し、後続実装が意図せずずらすと
//! スナップショットが落ちて検出される。
//!
//! 既存の個別スナップショット（`snapshot_test.rs`）に加え、代表的な複数構文（トーク・スコープ定義3型・
//! コードブロック・単語定義・変数代入・シーン呼び出し）を 1 ファイルへ集約したフィクスチャで OFF 経路出力を固定する。

use crate::common;

use common::e2e_helpers::transpile;
use insta::assert_snapshot;
use std::fs;
use std::path::PathBuf;

/// Get the path to test fixtures.
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// 代表的な複数構文を含むフィクスチャを読み込む。
///
/// 含む構文種別: 単語定義（グローバル/アクター/シーンローカル）・スコープ定義3型
/// （アクター/グローバルシーン/ローカルシーン）・トーク・変数代入（ローカル/
/// グローバル/プロパティ）・シーン呼び出し・コードブロック。
fn all_syntax_source() -> String {
    let fixture_path = fixtures_path().join("zero_cost_all_syntax.pasta");
    fs::read_to_string(&fixture_path).expect("Failed to read zero_cost_all_syntax.pasta fixture")
}

/// **デバッグ無効（sink 未装着）でのバイト不変スナップショット**。
///
/// 本番 transpile 経路（`LuaTranspiler::default()` = source-map sink 非装着 ＝ OFF 経路）
/// で代表的な複数構文を含むフィクスチャを変換し、生成 `.lua` を **コミット済みの
/// スナップショット基準と完全一致**させる。後続の producer 配線が OFF 経路の
/// 出力バイトを変えればこのテストが落ちる（回帰検出）。
#[test]
fn test_zero_cost_off_path_all_syntax_byte_invariant() {
    let lua_code = transpile(&all_syntax_source());

    // フィクスチャは複数構文を実際に展開する（vacuous でない）ことを保証。
    assert!(
        !lua_code.is_empty(),
        "fixture must transpile to real Lua output bytes"
    );

    // コミット済み .snap がデバッグ OFF 経路の受理済みベースライン。
    // 後続実装がこのバイト列を変えると本アサートが落ちる。
    assert_snapshot!("zero_cost_off_path_all_syntax", lua_code);
}

/// **OFF 経路の決定性**。
///
/// 本番 transpile（sink 非装着）を 2 回独立に実行すると生成バイトは完全一致する。
/// OFF 経路に非決定的な副作用が無いこと（＝バイト不変が安定）を直接表明する。
#[test]
fn test_zero_cost_off_path_is_deterministic() {
    let source = all_syntax_source();
    let run1 = transpile(&source);
    let run2 = transpile(&source);
    assert_eq!(
        run1, run2,
        "7.1: OFF-path transpile (no source-map sink) must be byte-for-byte deterministic"
    );
    assert!(!run1.is_empty(), "fixture must produce real output bytes");
}
