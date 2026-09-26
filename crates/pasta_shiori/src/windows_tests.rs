//! FFI extern 入口（`load`/`loadu`/`request`/`unload`/`DllMain`）の in-process テスト。
//!
//! task 5.1 で FFI が **アクターモデル**（`actor::lifecycle` の `static MAILBOX` 所有）へ
//! 再配線されたため、旧 `RawShiori<MockShiori>` ベースの dispatch テストは廃止し、本番経路の
//! **FFI 境界契約**（HGLOBAL 所有移譲＝解放／null・zero-len ガード／未初期化時 204／panic 非
//! unwind）を実 extern 関数に対して検証する。
//!
//! # プロセス全域状態の取り扱い
//! `actor::lifecycle::MAILBOX` と `loadu` 初期化フラグはどちらもプロセスグローバルである。
//! これらに触れるテストは `lock_global_state()` で直列化し、末尾で `unload()` して未初期化へ
//! 戻すことで順序非依存を保つ。`loadu` のフラグ契約を検証するテストは、実在しない
//! ディレクトリを渡して `spawn_actor` へ到達させる（VM は起こさず MAILBOX のみ設定される）。
//! 実 VM を起こす load→request→unload→reload サイクルの E2E はアクタースレッド・実ゴーストを
//! 要するため統合テスト（`tests/`・`actor-poc` 不要の既定ビルド）側に置く。

use super::*;
use crate::actor::marshaling::default_204;
use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalFlags};

/// Win32 SDK `GMEM_INVALID_HANDLE` (minwinbase.h) — returned by `GlobalFlags`
/// for freed/invalid handles. Not exported by windows-sys 0.61.
const GMEM_INVALID_HANDLE: u32 = 0x8000;

const LEAK_PROBE_LEN: usize = 64;

/// `windows.rs` のプロセス全域状態（`lifecycle::MAILBOX` と `loadu` 初期化フラグ）を
/// 触るテストを直列化する。並列実行下でも互いの前提（MAILBOX 未初期化／フラグの状態）を
/// 壊さない。
static FFI_GLOBAL_STATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 直列化ロックを取得する（poison は無視して内部値を使う）。
fn lock_global_state() -> std::sync::MutexGuard<'static, ()> {
    FFI_GLOBAL_STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// 文字列を HGLOBAL へ載せて `(handle, len)` を返す。`nofree` なので解放は
/// 受け取った FFI 入口（所有権移譲）が行う。
fn alloc_str_handle(text: &str) -> (HGLOBAL, usize) {
    ShioriString::clone_from_str_nofree(text)
        .expect("probe allocation must succeed")
        .value()
}

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
    let _guard = lock_global_state();
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
// loadu（UTF-8 設置パス）と「loadu 済みなら load を無視」契約（要件 2.4/2.5/2.6）。
// ------------------------------------------------------------------

/// `loadu` → `load` 無視 → `unload` でフラグ解除、までを 1 つのテストへ直列に詰める
/// （プロセス全域フラグと MAILBOX を共有するため）。
///
/// 設置ディレクトリには**実在しないパス**を使う。`spawn_actor` へは到達するが VM は
/// 起きないため軽量で、かつ「ロード失敗時もフラグを立てる」契約（設計
/// ShioriLoadEntry）をそのまま検証できる。
#[test]
fn loadu_initialized_flag_makes_load_a_noop_until_unload() {
    let _guard = lock_global_state();

    let missing = std::env::temp_dir().join("pasta_loadu_missing_dir_for_test");
    let missing = missing
        .to_str()
        .expect("temp path must be UTF-8")
        .to_string();

    // loadu: UTF-8 デコード → spawn_actor 到達（dir 不在によりロードは失敗）。
    let (h, len) = alloc_str_handle(&missing);
    assert!(
        !loadu(h, len),
        "loadu with a missing install dir must fail to load"
    );

    // loadu 済みフラグが立つため、後続の load は再ロードせず TRUE を返す（要件 2.5）。
    // 入力バイト列は無視されるので leak probe を渡し、解放されることも確認する。
    let probe = alloc_leak_probe();
    assert!(
        load(probe, LEAK_PROBE_LEN),
        "load after loadu must be ignored and return TRUE"
    );
    assert!(
        hglobal_is_freed(probe),
        "ignored load must still free the incoming HGLOBAL (ownership transfer)"
    );

    // unload でフラグが下りる。
    assert!(unload(), "unload must always return true");

    // フラグ解除後は従来どおり ANSI デコードでロードする（要件 2.6・dir 不在なので false）。
    let (h, len) = alloc_str_handle(&missing);
    assert!(
        !load(h, len),
        "after unload the legacy load must decode ANSI and attempt a real load"
    );

    // 後始末: MAILBOX を未初期化へ戻す（他テストの前提）。
    assert!(unload());
}

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
    let _guard = lock_global_state();
    // MAILBOX 未初期化（アクター未 spawn）。本番アクター経路は SHIORI スレッドを無限
    // 待機させない契約のため、旧 500 ではなく安全網 204 を返す（R5.6）。入力は解放される。
    let h = alloc_leak_probe();
    let mut len = LEAK_PROBE_LEN;
    let res = request(h, &mut len);
    assert!(
        !res.is_null(),
        "204 fallback must produce a response HGLOBAL"
    );
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
    let _guard = lock_global_state();
    assert!(
        unload(),
        "unload with no actor must be a safe no-op returning true"
    );
    // 二重 unload も安全（冪等）。
    assert!(unload(), "double unload must remain a safe no-op");
}

// ------------------------------------------------------------------
// DllMain の非 attach 経路。
// ------------------------------------------------------------------

#[test]
fn dll_main_non_attach_paths() {
    let _guard = lock_global_state();
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
