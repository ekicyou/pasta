//! キック未使用時のバイト不変回帰（特性化・最重要 R6.3）。
//!
//! 関連仕様: `.kiro/specs/pasta-scene-kick/`
//! - 充足要件:
//!   - **R6.2**: 外部 SHIORI の通常会話挙動（キックを行わない通常イベント処理）を
//!     キック経路の追加によって変更しない。
//!   - **R6.3**: キック経路が一切使用されない（`STORE.kick_pending` が常に nil）間、
//!     キック導入前と同一の通常 SHIORI 応答挙動（バイト不変）を維持する。
//! - 設計: design.md の Testing Strategy
//!   「**バイト不変回帰**（特性化・最重要 R6.3）: キック未使用時（`kick_pending`=nil）、
//!   OnSecondChange を含む通常 SHIORI 応答がキック導入前とバイト一致」、および
//!   「**バイト不変**: `kick_pending` 不在時は dispatch フックが完全素通り → 既存
//!   dispatch のまま（R6.3）」。
//!
//! ## このファイルが固定するもの（`byte_invariant_test.rs` との差分）
//! `byte_invariant_test.rs` は pasta-actor-runtime のアクター化リファクタに対する
//! FFI 入口ゴールデンである。本ファイルはそれとは別に、**キック機構導入後も
//! 「キックを一度も呼ばなければ通常応答は 1 バイトも動かない」**ことを R6.3 を名指しで
//! 固定する専用の特性化テストである。具体的には:
//!   1. 通常イベント列（OnBoot ＋ OnSecondChange）の FFI 入口応答バイト列を
//!      ゴールデンとして固定し一致を assert（バイト不変）。
//!   2. その全期間で `STORE.kick_pending` が nil・`STORE.kick_force` が false の
//!      ままであること（＝キック dispatch フックが完全素通りであること）を
//!      Lua レベルで観測（4.4 の Lua 確認を SHIORI/actor レベルへ持ち上げる）。
//!
//! ## 決定論・ハーメティック性
//! - キックを一切呼ばない。乱数依存（OnTalk ランダム選択）を踏まないイベント列のみ採用:
//!   1. OnBoot（hello-pasta 単一 OnBoot シーン・固定さくらスクリプト出力）。
//!   2. OnSecondChange 初回 tick（仮想ディスパッチャの内部タイマ初期化のみで
//!      発行スキップ＝決定論的 204）。`scene_kick_e2e_test.rs` のフェーズ 2 と同型。
//! - `PASTA_DEBUG` 系 env は `common` の `#[ctor]`（`neutralize_debug_env`）で中和済み
//!   （固定ポート枯渇による偽陰性を回避）。本ファイルは `mod common;` 経由でこのガードを
//!   取り込む。

mod common;

use pasta::{PastaShiori, Shiori};

/// SHIORI/3.0 リクエストを正規化（改行 → CRLF、終端 `\r\n\r\n` 付与）して
/// FFI 入口 `PastaShiori::request` を直接叩き、**生の応答文字列（=バイト列）** を返す。
///
/// 既存特性化テスト（`byte_invariant_test.rs`）と同一の正規化規則を用いる。
fn ffi_request_raw(shiori: &mut PastaShiori, text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    shiori
        .request(&req)
        .expect("FFI request should not return an error")
}

/// バイト不変アサーション（差分時は十六進相当のデバッグ表現を含めて報告）。
fn assert_golden_bytes(label: &str, actual: &str, expected: &str) {
    assert_eq!(
        actual.as_bytes(),
        expected.as_bytes(),
        "\nキック未使用バイト不変ゴールデン不一致 [{label}]\n\
         キックを一度も呼んでいないのに通常 SHIORI 応答バイト列が変化しました（R6.3 違反）。\n\
         キック dispatch フックの素通り失敗（通常 dispatch の変質）でないか確認してください。\n\
         意図的な挙動変更でない限り revert してください（R6.2/R6.3）。\n\
         expected (len={}): {:?}\n\
         actual   (len={}): {:?}\n",
        expected.len(),
        expected,
        actual.len(),
        actual,
    );
}

/// `STORE.kick_pending` / `STORE.kick_force` を Lua レベルで読み戻す。
///
/// キック経路が一切使用されていないことの観測点。キックを呼ばない限り
/// `kick_pending` は nil・`kick_force` は false のままであるべき（dispatch フックの
/// 素通り前提条件）。
fn read_kick_state(shiori: &PastaShiori) -> String {
    let runtime = shiori.runtime().expect("runtime should exist after load");
    let lua = runtime.lua();
    lua.load(
        r#"
        local STORE = require("pasta.store")
        return string.format(
            "kick_pending=%s,kick_force=%s",
            tostring(STORE.kick_pending), tostring(STORE.kick_force)
        )
    "#,
    )
    .eval::<String>()
    .expect("eval STORE kick state")
}

// ===========================================================================
// ゴールデン定数（現行実装＝キック導入後から特性化採取・キック未使用経路）
// ---------------------------------------------------------------------------
// これらは「キックを一切呼ばない通常応答」を符号化する。キック dispatch フックの
// 挿入が通常 dispatch を変質させたら（＝素通りに失敗したら）本テストが検出する。
// 文字列は CRLF・終端 \r\n\r\n を含む完全応答。
// ===========================================================================

/// OnBoot（hello-pasta 単一 OnBoot シーン）の完全応答（キック未使用）。
///
/// 意図した変更（`.kiro/specs/sakura-script-newline` R3.1）: sakura(spot0)→kero(spot1) の
/// 単純切替は両スポットとも初テキストのため、完全遅延方式では段落区切り改行 `\n[150]` を
/// 出力しない（先出し版は離脱側スコープ末尾にゴミ改行を残していた。本仕様はこれを排除する）。
const GOLDEN_ONBOOT: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: \\p[0]\\s[1]起動したよ～。\\_w[950]\\p[1]\\s[11]さあ、\\_w[450]始めようか。\\_w[950]\\e\r\n\
\r\n";

/// OnSecondChange 初回 tick（仮想ディスパッチャの内部タイマ初期化のみ・発行スキップ
/// ＝決定論的 204）の完全応答（キック未使用）。
const GOLDEN_ON_SECOND_CHANGE_204: &str = "SHIORI/3.0 204 No Content\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
\r\n";

/// キック未使用を表すロード直後／各イベント後の STORE 状態。
const KICK_STATE_UNUSED: &str = "kick_pending=nil,kick_force=false";

// ===========================================================================
// 特性化テスト
// ===========================================================================

/// R6.3 中核: キックを一度も呼ばずに通常イベント列（OnBoot → OnSecondChange）を流し、
/// 応答がゴールデンとバイト一致し、かつ全期間で `STORE.kick_pending` が nil のまま
/// （＝キック dispatch フックが完全素通り）であることを固定する。
#[test]
fn kick_unused_normal_responses_are_byte_invariant() {
    let temp = common::copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();
    let loaded = shiori
        .load(1, temp.path().as_os_str())
        .expect("load should not error");
    assert!(loaded, "load should return true");

    // ロード直後: キック未使用（pending=nil・force=false）。
    assert_eq!(
        read_kick_state(&shiori),
        KICK_STATE_UNUSED,
        "ロード直後はキック未使用（pending=nil）であるべき"
    );

    // (1) OnBoot: 決定論的な単一シーン応答がバイト不変。
    let onboot = ffi_request_raw(
        &mut shiori,
        "GET SHIORI/3.0\n\
Charset: UTF-8\n\
Sender: SSP\n\
SecurityLevel: local\n\
ID: OnBoot\n\
Reference0: マスターシェル\n",
    );
    assert_golden_bytes("OnBoot (kick unused)", &onboot, GOLDEN_ONBOOT);

    // OnBoot 後もキック未使用のまま（dispatch フックは co_scene 起動を一切していない）。
    assert_eq!(
        read_kick_state(&shiori),
        KICK_STATE_UNUSED,
        "OnBoot 通常処理でキック状態が変化してはならない（フック素通り）"
    );

    // (2) OnSecondChange 初回 tick: 仮想ディスパッチャのタイマ初期化のみ＝決定論的 204。
    let osc = ffi_request_raw(
        &mut shiori,
        "NOTIFY SHIORI/3.0\n\
Charset: UTF-8\n\
Sender: SSP\n\
SecurityLevel: local\n\
Status: balloon(0=0)\n\
ID: OnSecondChange\n\
Reference0: 0\n\
Reference1: 0\n\
Reference2: 0\n\
Reference3: 0\n",
    );
    assert_golden_bytes(
        "OnSecondChange first tick (kick unused)",
        &osc,
        GOLDEN_ON_SECOND_CHANGE_204,
    );

    // OnSecondChange 通常処理でもキック未使用のまま（`kick_pending`=nil 素通り＝R6.3）。
    assert_eq!(
        read_kick_state(&shiori),
        KICK_STATE_UNUSED,
        "OnSecondChange 通常処理でキック状態が変化してはならない（kick_pending=nil 素通り・R6.3）"
    );
}

/// メタテスト: ゴールデンアサーションが「自明に真」ではないことの実証。
///
/// 特性化テストの価値は「バイトが 1 つでも変われば赤化する」点にある（load-bearing）。
/// `assert_golden_bytes` が実際に差分を検出してパニックすることを、意図的な改竄に対して
/// 確認する（このメタテストが緑＝回帰ガードが死んでいない証拠）。
#[test]
fn kick_unused_golden_assertion_actually_detects_drift() {
    let tampered = GOLDEN_ON_SECOND_CHANGE_204.replacen("204 No Content", "200 OK", 1);
    let result = std::panic::catch_unwind(|| {
        assert_golden_bytes(
            "drift-detection self-check",
            &tampered,
            GOLDEN_ON_SECOND_CHANGE_204,
        );
    });
    assert!(
        result.is_err(),
        "ゴールデンアサーションはバイト差分でパニックすべき（R6.3 回帰ガードが自明に真でない証明）"
    );
}
