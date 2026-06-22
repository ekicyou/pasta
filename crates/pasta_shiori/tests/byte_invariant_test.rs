//! FFI 入口応答バイト列のゴールデン特性化テスト（ByteInvariantSuite）
//!
//! 関連仕様: `.kiro/specs/pasta-actor-runtime/`
//! - 充足要件: R1.1（外部 SHIORI 挙動バイト不変）／R1.2（全既存テスト回帰不変）／
//!   R1.4（差分検出で revert 可能な回帰ガード）
//! - 設計: design.md の **ByteInvariantSuite** コンポーネント
//!   （Testing Strategy → Integration Tests「ByteInvariant ゴールデン」）
//!
//! ## 目的
//! 本ファイルは pasta-actor-runtime の **アクター化リファクタ着手前**に敷設する、
//! FFI 入口（`PastaShiori::request` = research.md の外部観測点）応答バイト列の
//! **特性化（characterization）ゴールデン**である。現行実装の応答バイト列を
//! ゴールデンとして固定し、以後の各リファクタ段（event stream / actor+marshaling /
//! teardown / unsafe 撤去 / 足場撤去）で 1 バイトでも応答が変化したら検出する
//! 回帰ガードとして機能する。
//!
//! ## 非挙動タスク（特性化）について
//! これは新規挙動を導入しないテスト基盤タスクである。RED フェーズは存在しない
//! （現行実装で即グリーン）。ゴールデンは現行挙動そのものを「正」として符号化する。
//! 将来のリファクタで応答バイトが変化した場合、本テストが赤化し revert を促す。
//!
//! ## 決定論・ハーメティック性
//! - 乱数依存（OnTalk 等のランダム選択）を避け、**決定論的に同一バイトを返す**
//!   代表イベント列のみを採用する:
//!   1. OnBoot（hello-pasta 単一シーン・固定さくらスクリプト出力）
//!   2. GET property + コルーチン継続（async_callback フィクスチャの
//!      get_property → コールバック resume の 2 ラウンド）
//!   3. OnSecondChange（OnTalk ハンドラ未登録フィクスチャで決定論的に 204）
//!   4. NOTIFY（無関係イベント → 即 204）
//! - `PASTA_DEBUG` 系 env は `common` の `#[ctor]` で中和済み（固定ポート枯渇回避）。

mod common;

#[path = "common/async_callback_support.rs"]
mod async_callback_support;

use pasta::{PastaShiori, Shiori};
use std::path::Path;
use tempfile::TempDir;

/// SHIORI/3.0 リクエストを正規化（改行 → CRLF、終端 `\r\n\r\n` 付与）して
/// FFI 入口 `PastaShiori::request` を直接叩き、**生の応答文字列（=バイト列）** を返す。
///
/// 既存統合テスト（`test_env` / `async_callback_support`）と同一の正規化規則を用いる。
/// ここでは構造化パースを通さず、外部観測点である生バイトをそのまま捕捉する。
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

/// バイト不変アサーション。
///
/// 期待文字列と実応答を **バイト列として厳密比較**する。差分時は十六進ダンプを
/// 含めて報告し、どのリファクタ段でバイトが動いたか即座に判別できるようにする。
fn assert_golden_bytes(label: &str, actual: &str, expected: &str) {
    assert_eq!(
        actual.as_bytes(),
        expected.as_bytes(),
        "\nByteInvariant golden mismatch for [{label}]\n\
         FFI 入口応答バイト列が現行ゴールデンと不一致です（外部 SHIORI 挙動の変化＝回帰）。\n\
         意図的な挙動変更でない限り revert してください（R1.1/R1.4）。\n\
         expected (len={}): {:?}\n\
         actual   (len={}): {:?}\n",
        expected.len(),
        expected,
        actual.len(),
        actual,
    );
}

// ===========================================================================
// ゴールデン定数（現行実装から特性化採取・2026-06 時点）
// ---------------------------------------------------------------------------
// これらは「現状の正しさ」ではなく「現状そのもの」を符号化する。リファクタで
// 変化したら本テストが検出する。文字列は CRLF・終端 \r\n\r\n を含む完全応答。
// ===========================================================================

/// OnBoot（hello-pasta 単一 OnBoot シーン）の完全応答。
const GOLDEN_ONBOOT: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: \\p[0]\\s[1]起動したよ～。\\_w[950]\\n[150]\\p[1]\\s[11]さあ、\\_w[450]始めようか。\\_w[950]\\e\r\n\
\r\n";

/// GET property コルーチン継続 Round1: get_property が get タグを yield。
const GOLDEN_GET_PROPERTY_ROUND1: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: \\![get,property,OnPastaCallBack1,baseware.version]\\e\r\n\
\r\n";

/// GET property コルーチン継続 Round2: コールバック到着でコルーチン resume・値反映。
const GOLDEN_GET_PROPERTY_ROUND2: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: version=2.6.77\\e\\e\r\n\
\r\n";

/// OnSecondChange（OnTalk ハンドラ未登録フィクスチャ → 決定論的 204）。
const GOLDEN_ON_SECOND_CHANGE_204: &str = "SHIORI/3.0 204 No Content\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
\r\n";

/// NOTIFY 無関係イベント → 即 204。
const GOLDEN_NOTIFY_204: &str = "SHIORI/3.0 204 No Content\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
\r\n";

// ===========================================================================
// フィクスチャ構築ヘルパ
// ===========================================================================

/// async_callback フィクスチャ（本番 pasta_scripts + entry.lua override）で
/// ロード済み `PastaShiori` を構築する。`TempDir` は返り値で寿命保持する。
fn load_async_callback() -> (PastaShiori, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 本番 pasta_scripts/（EVENT/CALLBACK/SHIORI_ACT 等）
    let pasta_scripts_src = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_lua")
        .join("pasta_scripts");
    let pasta_scripts_dst = temp.path().join("pasta_scripts");
    std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
    copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

    // フィクスチャ（entry.lua override + pasta.toml）
    let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
    copy_dir_recursive(&fixture_src, temp.path()).expect("copy fixture");

    let mut shiori = PastaShiori::default();
    let loaded = shiori
        .load(0, temp.path().as_os_str())
        .expect("SHIORI load should not error");
    assert!(loaded, "SHIORI load should return true");
    (shiori, temp)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

// ===========================================================================
// ゴールデン特性化テスト
// ===========================================================================

/// OnBoot の FFI 入口応答バイト列ゴールデン（hello-pasta 単一シーン）。
///
/// さくらスクリプト描画（`\p`/`\s`/`\_w`/`\n`/`\e`）と本文がバイト不変であること。
/// アクター化リファクタ後もこのバイト列が同一であることを保証する（R1.1）。
#[test]
fn golden_onboot_response_bytes() {
    let temp = common::copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();
    let loaded = shiori
        .load(1, temp.path().as_os_str())
        .expect("load should not error");
    assert!(loaded, "load should return true");

    let resp = ffi_request_raw(
        &mut shiori,
        "GET SHIORI/3.0\n\
Charset: UTF-8\n\
Sender: SSP\n\
SecurityLevel: local\n\
ID: OnBoot\n\
Reference0: マスターシェル\n",
    );
    assert_golden_bytes("OnBoot", &resp, GOLDEN_ONBOOT);
}

/// GET property + コルーチン継続の 2 ラウンドゴールデン。
///
/// Round1 で `get_property` がコルーチンを yield して get タグ応答を返し、
/// Round2 のコールバックでコルーチンが resume され値が反映される。
/// executor 駆動化（アクター化）後もこの 2 段のバイト列が同一であることを
/// 保証する（コルーチン意味論不変・R1.1/R9 横断観測）。
#[test]
fn golden_get_property_coroutine_continuation_bytes() {
    let (mut shiori, _temp) = load_async_callback();

    let round1 = ffi_request_raw(
        &mut shiori,
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n",
    );
    assert_golden_bytes("GET property round1 (yield)", &round1, GOLDEN_GET_PROPERTY_ROUND1);

    let round2 = ffi_request_raw(
        &mut shiori,
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnPastaCallBack1\nReference0: 2.6.77\n",
    );
    assert_golden_bytes("GET property round2 (resume)", &round2, GOLDEN_GET_PROPERTY_ROUND2);
}

/// OnSecondChange（OnTalk ハンドラ未登録フィクスチャ → 決定論的 204）と
/// NOTIFY 無関係イベント（即 204）の応答バイト列ゴールデン。
///
/// 乱数依存を避けるため、OnSecondChange は OnTalk を登録しない async_callback
/// フィクスチャ上で駆動し、決定論的に 204 No Content となることを固定する。
#[test]
fn golden_notify_and_onsecondchange_204_bytes() {
    let (mut shiori, _temp) = load_async_callback();

    let osc = ffi_request_raw(
        &mut shiori,
        "NOTIFY SHIORI/3.0\n\
Charset: UTF-8\n\
ID: OnSecondChange\n\
Reference0: 0\n\
Reference1: 0\n\
Reference2: 0\n\
Reference3: 0\n",
    );
    assert_golden_bytes("OnSecondChange (204)", &osc, GOLDEN_ON_SECOND_CHANGE_204);

    let notify = ffi_request_raw(
        &mut shiori,
        "NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\n",
    );
    assert_golden_bytes("NOTIFY unrelated (204)", &notify, GOLDEN_NOTIFY_204);
}

/// メタテスト: ゴールデンアサーションが「自明に真」ではないことの実証。
///
/// 特性化テストの価値は「バイトが 1 つでも変われば赤化する」点にある。
/// ここでは `assert_golden_bytes` が実際に差分を検出できること
/// （= 後続リファクタの回帰ガードとして機能すること）を、意図的な改竄に対して
/// パニックが起きることで確認する（R1.4 差分検出）。
#[test]
fn golden_assertion_actually_detects_drift() {
    // 1 文字だけ変えた偽応答はバイト不一致でパニックすべき。
    let tampered = GOLDEN_NOTIFY_204.replacen("204 No Content", "200 OK", 1);
    let result = std::panic::catch_unwind(|| {
        assert_golden_bytes("drift-detection self-check", &tampered, GOLDEN_NOTIFY_204);
    });
    assert!(
        result.is_err(),
        "ゴールデンアサーションはバイト差分でパニックすべき（回帰ガードが自明に真でない証明）"
    );
}
