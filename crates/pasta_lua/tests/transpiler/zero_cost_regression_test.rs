//! Task 1.2 — ゼロコスト回帰テスト基盤（バイト不変スナップショット）。
//!
//! 仕様参照:
//! - requirements.md **7.1**:
//!   「While ソースマップ生成が無効である, the ソースマップ生成器 shall 生成 `.lua` の
//!    出力内容を従来と完全に一致させる（バイト不変）」
//! - design.md "Testing Strategy" → "Integration Tests" → **ゼロコスト回帰**（633 行）:
//!   「debug 無効（sink=None）で生成 `.lua` がバイト不変（既存スナップショットと一致）」
//! - design.md "Performance / Backward Compatibility" → 二相のゼロコスト（643-649 行）:
//!   OFF 経路はローダが sink を装着せず `record_*` が no-op のため出力バイト不変
//! - design.md "RecordWiring" Invariants（353 行）: sink=None で出力バイト不変
//!
//! # このモジュールの役割（RED 検出器）
//!
//! これは後続の producer タスク（2.x: 全 `generate_*` への `record_span` 配線）が
//! **デバッグ無効時の生成 `.lua` を 1 バイトも変えていない**ことを保証する安全網である。
//! 現時点の本番 transpile 経路（`LuaTranspiler::transpile`）は **source-map sink を
//! 一切装着しない** ＝ そのままデバッグ OFF 経路であるため、ここでその出力を
//! 受理済みベースラインとして固定する。後続実装が OFF 経路の出力を意図せずずらすと
//! このスナップショットが落ちて検出される。
//!
//! 既存の個別スナップショット（`snapshot_test.rs`）に加え、本モジュールは
//! **代表的な複数構文を 1 ファイルへ集約**したフィクスチャ（トーク・スコープ定義3型・
//! コードブロック・単語定義・変数代入・シーン呼び出し）で OFF 経路出力を 1 つの
//! durable な基準へ固定し、要件 7.1 を明示マッピングする。

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

/// **7.1 — デバッグ無効（sink 未装着）でのバイト不変スナップショット**。
///
/// 本番 transpile 経路（`LuaTranspiler::default()` = source-map sink 非装着 ＝ OFF 経路）
/// で代表的な複数構文を含むフィクスチャを変換し、生成 `.lua` を **コミット済みの
/// スナップショット基準と完全一致**させる。後続の producer 配線（2.x）が OFF 経路の
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
    // 後続実装がこのバイト列を変えると本アサートが落ちる（7.1 の RED 検出器）。
    assert_snapshot!("zero_cost_off_path_all_syntax", lua_code);
}

/// **7.1 補強 — OFF 経路の決定性**。
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
