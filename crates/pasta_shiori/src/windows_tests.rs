//! FFI extern 入口（`load`/`request`/`unload`/`DllMain`）の in-process テスト。
//!
//! task 5.1 で FFI が **アクターモデル**（`actor::lifecycle` の `static MAILBOX` 所有）へ
//! 再配線されたため、旧 `RawShiori<MockShiori>` ベースの dispatch テストは廃止し、本番経路の
//! **FFI 境界契約**（HGLOBAL 所有移譲＝解放／null・zero-len ガード／未初期化時 204／panic 非
//! unwind）を実 extern 関数に対して検証する。
//!
//! # `static MAILBOX` の取り扱い（順序非依存）
//! `actor::lifecycle::MAILBOX` はプロセスグローバルだが、本テスト群は **アクターを spawn しない**
//! （`load` を実ゴーストなしで呼ばない）。`request`/`unload` は MAILBOX 未初期化（None）経路の
//! 契約のみを検証するため、他テストとの実行順序に依存しない（None なら 204／冪等 no-op）。
//! 実 VM を起こす load→request→unload→reload サイクルの E2E はアクタースレッド・実ゴーストを
//! 要するため統合テスト（`tests/`・`actor-poc` 不要の既定ビルド）側に置く。

use super::*;
use crate::actor::marshaling::default_204;
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalFlags, GMEM_MOVEABLE};

/// Win32 SDK `GMEM_INVALID_HANDLE` (minwinbase.h) — returned by `GlobalFlags`
/// for freed/invalid handles. Not exported by windows-sys 0.61.
const GMEM_INVALID_HANDLE: u32 = 0x8000;

const LEAK_PROBE_LEN: usize = 64;

/// GMEM_MOVEABLE 確保（freed 検出を `GlobalFlags` で安全に行える handle-table エントリ）。
fn alloc_leak_probe() -> HGLOBAL {
    // SAFETY: 単純確保。ガード経路が消費（解放）する。
    let h = unsafe { GlobalAlloc(GMEM_MOVEABLE, LEAK_PROBE_LEN) };
    assert!(!h.is_null(), "probe allocation must succeed");
    h
}

/// `GlobalFlags` は解放済み handle に `GMEM_INVALID_HANDLE` を返す。
fn hglobal_is_freed(h: HGLOBAL) -> bool {
    // SAFETY: GlobalFlags は handle を検証し、解放済みなら GMEM_INVALID_HANDLE を返す。
    unsafe { GlobalFlags(h) == GMEM_INVALID_HANDLE }
}

/// 応答 HGLOBAL を UTF-8 として読み、解放する。
fn read_and_free(h: HGLOBAL, len: usize) -> String {
    let s = ShioriString::capture(h, len);
    s.to_utf8_str().unwrap().to_string()
}

// ------------------------------------------------------------------
// load ガード経路（null / zero-len）と HGLOBAL 所有移譲（解放）。
// ------------------------------------------------------------------

#[test]
fn extern_load_null_is_rejected() {
    assert!(!load(ptr::null_mut(), 5), "null HGLOBAL must be rejected");
}

#[test]
fn extern_load_zero_len_returns_false_and_frees_input() {
    let h = alloc_leak_probe();
    assert!(!load(h, 0), "zero length must be rejected");
    assert!(
        hglobal_is_freed(h),
        "load len==0 guard must free the incoming HGLOBAL (ownership transfer)"
    );
}

// NOTE: 実 `load`→spawn_actor（アクタースレッド起動・`static MAILBOX` 設定）を踏む E2E は
// プロセスグローバル MAILBOX を変化させ、並列実行中の他ユニットテスト（None 経路）と競合する。
// そのため load→request→unload→reload サイクルは統合テスト（`tests/`）へ置き、本ユニット
// 群は MAILBOX を spawn しない（順序非依存）ガード／未初期化契約のみを検証する。

// ------------------------------------------------------------------
// request ガード経路（null / 未初期化 → 204）と HGLOBAL 解放。
// ------------------------------------------------------------------

#[test]
fn extern_request_null_returns_null_and_zero_len() {
    let mut len = 1234usize;
    let res = request(ptr::null_mut(), &mut len);
    assert!(res.is_null());
    assert_eq!(len, 0, "out-len must be zeroed on the null path");
}

#[test]
fn extern_request_without_actor_returns_204_and_frees_input() {
    // MAILBOX 未初期化（アクター未 spawn）。本番アクター経路は SHIORI スレッドを無限
    // 待機させない契約のため、旧 500 ではなく安全網 204 を返す（R5.6）。入力は解放される。
    let h = alloc_leak_probe();
    let mut len = LEAK_PROBE_LEN;
    let res = request(h, &mut len);
    assert!(!res.is_null(), "204 fallback must produce a response HGLOBAL");
    let body = read_and_free(res, len);
    assert_eq!(
        body.as_bytes(),
        default_204().as_bytes(),
        "request without an actor must return the 204 safety net (not 500)"
    );
    assert!(
        hglobal_is_freed(h),
        "request must free the incoming HGLOBAL (ownership transfer)"
    );
}

// ------------------------------------------------------------------
// unload は未初期化でも常に true（冪等 no-op）。
// ------------------------------------------------------------------

#[test]
fn extern_unload_without_actor_returns_true() {
    assert!(unload(), "unload with no actor must be a safe no-op returning true");
    // 二重 unload も安全（冪等）。
    assert!(unload(), "double unload must remain a safe no-op");
}

// ------------------------------------------------------------------
// DllMain の非 attach 経路。
// ------------------------------------------------------------------

#[test]
fn dll_main_non_attach_paths() {
    const DLL_PROCESS_DETACH: u32 = 0;
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_THREAD_ATTACH: u32 = 2;

    // ATTACH は no-op で true（spawn は load 起点・loader lock 回避）。
    assert!(DllMain(0, DLL_PROCESS_ATTACH, ptr::null_mut()));

    // DETACH は teardown_actor へ委譲。アクター不在でも unload()→true。
    assert!(DllMain(0, DLL_PROCESS_DETACH, ptr::null_mut()));

    // その他の通知コードは no-op で true。
    assert!(DllMain(0, DLL_THREAD_ATTACH, ptr::null_mut()));
    assert!(DllMain(0, 99, ptr::null_mut()));
}

// ------------------------------------------------------------------
// R8: unsafe impl Send/Sync 撤去の静的確認。
// ------------------------------------------------------------------

/// `shiori.rs` から `unsafe impl Send/Sync for PastaShiori` が撤去されたことを、ソースを
/// 読み取って静的に保証する（R8.1/R8.3）。VM はアクタースレッドへ pin され構造的に
/// スレッド安全性を担保するため、手動 unsafe impl は存在してはならない。
#[test]
fn pasta_shiori_has_no_unsafe_send_sync_impl() {
    let src = include_str!("shiori.rs");
    // コメント行（`//` 始まり）を除外して非コメントの実コードのみを走査する。
    // （撤去を説明するコメント自体は撤去された impl 名を文中で参照するため。）
    for line in src.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        assert!(
            !trimmed.contains("unsafe impl Send for PastaShiori"),
            "unsafe impl Send for PastaShiori must be removed (R8.1); offending line: {line}"
        );
        assert!(
            !trimmed.contains("unsafe impl Sync for PastaShiori"),
            "unsafe impl Sync for PastaShiori must be removed (R8.1); offending line: {line}"
        );
    }
}
