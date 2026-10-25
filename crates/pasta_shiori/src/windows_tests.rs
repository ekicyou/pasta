//! FFI extern 入口（`load`/`loadu`/`request`/`unload`/`DllMain`）の in-process テスト。
//!
//! task 5.1 で FFI が **アクターモデル**（`actor::lifecycle` の `static LEGACY` 所有）へ
//! 再配線されたため、旧 `RawShiori<MockShiori>` ベースの dispatch テストは廃止し、本番経路の
//! **FFI 境界契約**（HGLOBAL 所有移譲＝解放／null・zero-len ガード／未初期化時 204／panic 非
//! unwind）を実 extern 関数に対して検証する。
//!
//! # プロセス全域状態の取り扱い
//! `actor::lifecycle::LEGACY` と `loadu` 初期化フラグはどちらもプロセスグローバルである。
//! これらに触れるテストは `lock_global_state()` で直列化し、末尾で `unload()` して未初期化へ
//! 戻すことで順序非依存を保つ。`loadu` のフラグ契約を検証するテストは、実在しない
//! ディレクトリを渡して `spawn_actor` へ到達させる（VM は起こさず LEGACY のみ設定される）。
//! 実 VM を起こす load→request→unload→reload サイクルの E2E はアクタースレッド・実ゴーストを
//! 要するため統合テスト（`tests/`・既定ビルド）側に置く。

use super::*;
use crate::actor::marshaling::default_204;
use std::ffi::c_long;
use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalFlags};

/// Win32 SDK `GMEM_INVALID_HANDLE` (minwinbase.h) — returned by `GlobalFlags`
/// for freed/invalid handles. Not exported by windows-sys 0.61.
const GMEM_INVALID_HANDLE: u32 = 0x8000;

const LEAK_PROBE_LEN: usize = 64;

/// `windows.rs` のプロセス全域状態（`lifecycle::LEGACY` と `loadu` 初期化フラグ）を
/// 触るテストを直列化する。並列実行下でも互いの前提（LEGACY 未初期化／フラグの状態）を
/// 壊さない。
static FFI_GLOBAL_STATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 直列化ロックを取得する（poison は無視して内部値を使う）。
fn lock_global_state() -> std::sync::MutexGuard<'static, ()> {
    FFI_GLOBAL_STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// 文字列を HGLOBAL へ載せて `(handle, len)` を返す。`nofree` なので解放は
/// 受け取った FFI 入口（所有権移譲）が行う。
fn alloc_str_handle(text: &str) -> (HGLOBAL, c_long) {
    let (h, len) = ShioriString::clone_from_str_nofree(text)
        .expect("probe allocation must succeed")
        .value();
    (h, len.try_into().expect("probe length must fit c_long"))
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
fn read_and_free(h: HGLOBAL, len: c_long) -> String {
    let s = ShioriString::capture(
        h,
        len.try_into()
            .expect("response length must be non-negative"),
    );
    s.to_utf8_str().unwrap().to_string()
}

// ------------------------------------------------------------------
// load ガード経路（null / zero-len）と HGLOBAL 所有移譲（解放）。
// ------------------------------------------------------------------

#[test]
fn extern_load_null_is_rejected() {
    assert_eq!(load(ptr::null_mut(), 5), 0, "null HGLOBAL must be rejected");
}

#[test]
fn extern_load_zero_len_returns_false_and_frees_input() {
    let _guard = lock_global_state();
    let h = alloc_leak_probe();
    assert_eq!(load(h, 0), 0, "zero length must be rejected");
    assert!(
        hglobal_is_freed(h),
        "load len==0 guard must free the incoming HGLOBAL (ownership transfer)"
    );
}

// NOTE: 実 `load`→spawn_actor（アクタースレッド起動・`static LEGACY` 設定）を踏む E2E は
// プロセスグローバル LEGACY を変化させ、並列実行中の他ユニットテスト（None 経路）と競合する。
// そのため load→request→unload→reload サイクルは統合テスト（`tests/`）へ置き、本ユニット
// 群は LEGACY を spawn しない（順序非依存）ガード／未初期化契約のみを検証する。

// ------------------------------------------------------------------
// loadu（UTF-8 設置パス）と「loadu 済みなら load を無視」契約（要件 2.4/2.5/2.6）。
// ------------------------------------------------------------------

/// `loadu` → `load` 無視 → `unload` でフラグ解除、までを 1 つのテストへ直列に詰める
/// （プロセス全域フラグと LEGACY を共有するため）。
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
    assert_eq!(
        loadu(h, len),
        0,
        "loadu with a missing install dir must fail to load"
    );

    // loadu 済みフラグが立つため、後続の load は再ロードせず TRUE を返す（要件 2.5）。
    // 入力バイト列は無視されるので leak probe を渡し、解放されることも確認する。
    let probe = alloc_leak_probe();
    assert_eq!(
        load(probe, LEAK_PROBE_LEN as c_long),
        1,
        "load after loadu must be ignored and return TRUE"
    );
    assert!(
        hglobal_is_freed(probe),
        "ignored load must still free the incoming HGLOBAL (ownership transfer)"
    );

    // unload でフラグが下りる。
    assert_eq!(unload(), 1, "unload must always return true");

    // フラグ解除後は従来どおり ANSI デコードでロードする（要件 2.6・dir 不在なので false）。
    let (h, len) = alloc_str_handle(&missing);
    assert_eq!(
        load(h, len),
        0,
        "after unload the legacy load must decode ANSI and attempt a real load"
    );

    // 後始末: LEGACY を未初期化へ戻す（他テストの前提）。
    assert_eq!(unload(), 1);
}

// ------------------------------------------------------------------
// request ガード経路（null / 未初期化 → 204）と HGLOBAL 解放。
// ------------------------------------------------------------------

#[test]
fn extern_request_null_returns_null_and_zero_len() {
    let mut len: c_long = 1234;
    let res = request(ptr::null_mut(), &mut len);
    assert!(res.is_null());
    assert_eq!(len, 0, "out-len must be zeroed on the null path");
}

#[test]
fn extern_request_without_actor_returns_204_and_frees_input() {
    let _guard = lock_global_state();
    // LEGACY 未初期化（アクター未 spawn）。本番アクター経路は SHIORI スレッドを無限
    // 待機させない契約のため、旧 500 ではなく安全網 204 を返す（R5.6）。入力は解放される。
    let h = alloc_leak_probe();
    let mut len = LEAK_PROBE_LEN as c_long;
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
    assert_eq!(
        unload(),
        1,
        "unload with no actor must be a safe no-op returning true"
    );
    // 二重 unload も安全（冪等）。
    assert_eq!(unload(), 1, "double unload must remain a safe no-op");
}

// ------------------------------------------------------------------
// C の `long` の長さ（7.3）: 負は 0 と同じ・応答が収まらなければ null と 0・長さの指す先が null なら null。
// ------------------------------------------------------------------

#[test]
fn len_from_c_treats_negative_as_zero() {
    assert_eq!(len_from_c(-1), 0);
    assert_eq!(len_from_c(c_long::MIN), 0);
    assert_eq!(len_from_c(0), 0);
    assert_eq!(len_from_c(42), 42);
    assert_eq!(len_from_c(c_long::MAX), c_long::MAX as usize);
}

#[test]
fn len_to_c_overflow_frees_response_and_returns_null_and_zero() {
    let h = alloc_leak_probe();
    assert_eq!(len_to_c(h, LEAK_PROBE_LEN), (h, LEAK_PROBE_LEN as c_long));
    // 収まる側は手放さない（呼び出し側が解放する）。
    assert!(!hglobal_is_freed(h));
    // c_long に収まらない長さ（x86 でも usize は u32 なので作れる）。
    let (res, len) = len_to_c(h, c_long::MAX as usize + 1);
    assert!(res.is_null(), "overflowing response length must yield null");
    assert_eq!(len, 0, "overflowing response length must yield length 0");
    assert!(hglobal_is_freed(h), "overflowing response must be freed");
}

#[test]
fn extern_load_negative_len_is_rejected_like_zero_and_frees_input() {
    let _guard = lock_global_state();
    let h = alloc_leak_probe();
    assert_eq!(load(h, -1), 0, "negative length must be rejected like zero");
    assert!(
        hglobal_is_freed(h),
        "negative length must still free the input"
    );
}

#[test]
fn extern_request_negative_len_is_an_empty_request() {
    let _guard = lock_global_state();
    let h = alloc_leak_probe();
    let mut len: c_long = -5;
    let res = request(h, &mut len);
    assert!(
        !res.is_null(),
        "negative length must take the empty-request path"
    );
    let body = read_and_free(res, len);
    assert_eq!(body.as_bytes(), default_204().as_bytes());
    assert!(
        hglobal_is_freed(h),
        "request must free the incoming HGLOBAL"
    );
}

#[test]
fn extern_request_null_len_pointer_returns_null() {
    let h = alloc_leak_probe();
    let res = request(h, ptr::null_mut());
    assert!(res.is_null(), "null length pointer must yield null");
    assert!(
        hglobal_is_freed(h),
        "request must still free the incoming HGLOBAL"
    );
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
    assert_eq!(DllMain(0, DLL_PROCESS_ATTACH, ptr::null_mut()), 1);

    // DETACH は join しない後始末（teardown_actor_detached）へ委譲。アクター不在でも 1。
    assert_eq!(DllMain(0, DLL_PROCESS_DETACH, ptr::null_mut()), 1);

    // その他の通知コードは no-op で true。
    assert_eq!(DllMain(0, DLL_THREAD_ATTACH, ptr::null_mut()), 1);
    assert_eq!(DllMain(0, 99, ptr::null_mut()), 1);
}

/// プロセス終了による DETACH（reserved 非 null）は teardown（unload）を呼ばない。
#[test]
fn dll_main_process_termination_detach_skips_teardown() {
    let _guard = lock_global_state();
    const DLL_PROCESS_DETACH: u32 = 0;
    let mut marker = 0u8;

    LOADU_INITIALIZED.store(true, Ordering::SeqCst);
    assert_eq!(
        DllMain(0, DLL_PROCESS_DETACH, (&mut marker as *mut u8).cast()),
        1
    );
    // 後始末を通っていればフラグは下りている。
    assert!(LOADU_INITIALIZED.load(Ordering::SeqCst));

    assert_eq!(unload(), 1);
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
