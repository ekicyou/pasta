//! FFI 入口から通したログがゴーストのログファイルに残ることの E2E 検証
//! （仕様 `pasta-toml-logging-consistency`・design.md Testing Strategy > E2E Tests）。
//!
//! 出荷 extern "C" 入口（`loadu`・`load`・`request`・`unload`）を C ABI で駆動し、
//! `unload` の後（ロガーが破棄されフラッシュされた後）にログファイルの中身を読む。
//!
//! # 1 本の直列テストにする理由（design.md D3）
//! ログの経路はプロセス全域の状態（購読者・フィルタ・登録簿）を使い、FFI の
//! `MAILBOX`・`loadu` 初期化フラグもプロセスグローバルである。本ファイルは全段を
//! 1 つの `#[test]` に直列に詰め、ゴーストを 1 つずつ順に読む（登録簿は常に 0 個か 1 個）。
//! 段を足すときは同じテスト関数の末尾へ足し、他の `#[test]` を置かない。
//!
//! `mod common;` は `PASTA_DEBUG`・`PASTA_DEBUG_PORT`・`PASTA_LOG` を main 前に中和する
//! `#[ctor]` を取り込むためにも必須である（`PASTA_LOG` が残るとフィルタが変わる）。

#![cfg(windows)]

mod common;

use std::path::{Path, PathBuf};

use common::copy_fixture_to_temp;
use pasta::actor::marshaling::default_204;
use pasta::{load, request, unload};
use tempfile::TempDir;
use windows_sys::Win32::Foundation::{GlobalFree, HGLOBAL};
use windows_sys::Win32::System::Memory::{GMEM_FIXED, GlobalAlloc};

unsafe extern "C" {
    /// SHIORI DLL 共通仕様の `loadu`（設置パスを UTF-8 で受け取る初期化入口）。
    fn loadu(hdir: HGLOBAL, len: usize) -> bool;
}

const FIXTURE: &str = "shiori_lifecycle";

/// 既定のログファイル（設置ディレクトリからの相対パス）。
const DEFAULT_LOG: &str = "profile/pasta/logs/pasta.log";

/// `shiori_lifecycle` の `テスト挨拶` が返す正常応答。
const GOLDEN_OK: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Value: ライフサイクルテスト成功！\r\n\
\r\n";

const GET_REQUEST: &str =
    "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: テスト挨拶\r\nSender: SSP\r\n\r\n";

/// UTF-8 として不正なバイト列（`request` 入口の decode で落ちる）。
const INVALID_UTF8: &[u8] = b"GET SHIORI/3.0\r\nID: \xff\xfe\r\n\r\n";

// ===========================================================================
// HGLOBAL・FFI 入口の補助
// ===========================================================================

/// バイト列を HGLOBAL へ載せる（所有権は入口側へ渡り、入口側が解放する）。
fn alloc_hglobal(bytes: &[u8]) -> HGLOBAL {
    // SAFETY: GMEM_FIXED 確保。失敗は null を返すので検査する。
    let h = unsafe { GlobalAlloc(GMEM_FIXED, bytes.len()) };
    assert!(!h.is_null(), "GlobalAlloc must succeed");
    // SAFETY: h は bytes.len() バイトの有効ブロックを指す。
    unsafe { std::slice::from_raw_parts_mut(h as *mut u8, bytes.len()).copy_from_slice(bytes) };
    h
}

/// `request` を 1 回駆動し、応答 HGLOBAL（呼び出し側所有）を読んで解放する。
fn drive_request(bytes: &[u8]) -> String {
    let mut len = bytes.len();
    let h = request(alloc_hglobal(bytes), &mut len);
    assert!(!h.is_null(), "response HGLOBAL must not be null");
    // SAFETY: 応答は h が len バイトの有効ブロックを指す（emit_response_into の契約）。
    let s = unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(h as *const u8, len)) }
        .into_owned();
    // SAFETY: 応答 HGLOBAL は呼び出し側が解放する契約。
    unsafe { GlobalFree(h) };
    s
}

fn dir_bytes(dir: &Path) -> Vec<u8> {
    dir.to_str().expect("UTF-8 path").as_bytes().to_vec()
}

fn drive_loadu(dir: &Path) -> bool {
    let bytes = dir_bytes(dir);
    // SAFETY: 有効な UTF-8 ブロック。所有は入口側へ移譲される。
    unsafe { loadu(alloc_hglobal(&bytes), bytes.len()) }
}

/// 従来入口 `load`。`loadu` 済みなら中身を見ずに無視されるので、パスは UTF-8 のまま渡す。
fn drive_load(dir: &Path) -> bool {
    let bytes = dir_bytes(dir);
    load(alloc_hglobal(&bytes), bytes.len())
}

// ===========================================================================
// フィクスチャ・ログの補助（後続の段でも使う）
// ===========================================================================

/// `shiori_lifecycle` を一時ディレクトリへ複製したゴーストを作る。
fn install_ghost(extra_toml: &str) -> TempDir {
    let ghost = copy_fixture_to_temp(FIXTURE);
    set_extra_toml(ghost.path(), extra_toml);
    ghost
}

/// ゴーストの `pasta.toml` を「フィクスチャの原本 + `extra_toml`」に書き直す
/// （`[logging]`・`[persistence]` を書き足す。同じゴーストの設定を直すときにも使う）。
fn set_extra_toml(ghost: &Path, extra_toml: &str) {
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(FIXTURE)
        .join("pasta.toml");
    let mut content = std::fs::read_to_string(original).expect("read fixture pasta.toml");
    content.push('\n');
    content.push_str(extra_toml);
    std::fs::write(ghost.join("pasta.toml"), content).expect("write pasta.toml");
}

/// 設置ディレクトリからの相対パス `rel` のログファイルを読む（無ければ空）。
fn read_log(ghost: &Path, rel: &str) -> String {
    std::fs::read_to_string(ghost.join(rel)).unwrap_or_default()
}

fn assert_log_contains(log: &str, needle: &str, why: &str) {
    assert!(
        log.contains(needle),
        "{why}: missing {needle:?}\nlog:\n{log}"
    );
}

// ===========================================================================
// 1 本の直列テスト
// ===========================================================================

#[test]
fn ffi_logs_reach_the_ghost_log_file() {
    // --- 段 1: ロガー無し（最初の読み込みの前・4.7、4.8、7.5） ---
    // ログは捨てられ、panic せず、戻り値は従来どおり。
    let mut len = 123usize;
    let res = request(std::ptr::null_mut(), &mut len);
    assert!(res.is_null(), "null request returns null");
    assert_eq!(len, 0, "null request sets len to 0");
    assert_eq!(
        drive_request(INVALID_UTF8),
        default_204(),
        "invalid UTF-8 request returns the safety-net 204"
    );
    assert!(unload(), "unload before load returns true");

    // --- 段 2〜4: `request` 入口・アクタースレッド・`load` の無視（4.1、4.2、4.5、7.3） ---
    let ghost = install_ghost("[logging]\nlevel = \"trace\"\n");
    assert!(drive_loadu(ghost.path()), "loadu must load the ghost");
    assert_eq!(
        drive_request(INVALID_UTF8),
        default_204(),
        "invalid UTF-8 request still returns 204 (4.8)"
    );
    assert_eq!(
        drive_request(GET_REQUEST.as_bytes()),
        GOLDEN_OK,
        "GET response must be unchanged (4.8)"
    );
    assert!(drive_load(ghost.path()), "load after loadu returns true");
    assert!(unload(), "unload returns true");

    let log = read_log(ghost.path(), DEFAULT_LOG);
    // 文脈の無い FFI 入口スレッドのログ（「文脈なし→唯一のロガー」の規則で届く・4.1）。
    assert_log_contains(&log, "utf8 decode failed", "request entry warn (4.1)");
    assert_log_contains(&log, "seam=\"actor.try_send\"", "marshaling trace (4.1)");
    // アクタースレッドのメッセージループの観測ログ（4.2）。
    assert_log_contains(&log, "actor received Stop", "actor stop log (4.2)");
    assert_log_contains(
        &log,
        "ignored: already initialized via loadu",
        "ignored load warn (4.5)",
    );
}
