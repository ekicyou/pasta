//! Windows SHIORI DLL interface
//!
//! Provides SHIORI protocol entry points for Windows DLL.
//!
//! # 所有モデル（task 5.1・R8.1/R8.2/R8.3/R8.4）
//! FFI 入口は旧来の同期 in-place VM モデル（`OnceLock<RawShiori<PastaShiori>>` ＋
//! `Arc<Mutex<Option<PastaShiori>>>` ＋ `unsafe impl Send/Sync`）から、**アクタースレッドへ
//! VM を pin し `static MAILBOX`（flume `Sender`）越しにメッセージ送信でのみアクセスする**
//! 本番アクターモデル（[`crate::actor::lifecycle`]）へ再配線済み。VM（`!Send`）はアクター
//! スレッドを越えないため `unsafe impl Send/Sync` は構造的に不要（撤去済み）。
//!
//! - `load`   → [`lifecycle::spawn_actor`]（mailbox 生成 → アクタースレッド spawn → `Sender`
//!   を MAILBOX へ格納）。スレッド spawn は **load 起点**で行い loader lock を回避する（R4.4）。
//! - `request`→ [`lifecycle::marshal_request`]（GET=block-on-reply／NOTIFY=即 204）。
//! - `unload` / `DllMain detach` → [`lifecycle::teardown_actor`]（`Stop{done}` ack → detach）。
//!
//! # panic 封じ込め（R3.7 維持）
//! 各 extern 入口は引き続き [`catch_unwind`] で panic を SHIORI エラー契約へ封じる
//! （unwind プロファイル＝dev/test 向けの保険。release `panic=abort` では到達不能）。
//! marshaling 自体は正常経路 panic-free（R5.10）で、アクタースレッド上の VM panic は
//! アクター側で捕捉され reply drop → 204 へ倒れる（SHIORI スレッドへ unwind しない）。

use crate::actor::lifecycle;
use crate::util::hglobal::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use tracing::{error, warn};
use windows_sys::Win32::Foundation::*;

/// Windows DLL entry point
/// Initializes SHIORI at DLL load/unload time.
///
/// # Safety
/// This function is called by the Windows loader. The caller must ensure:
/// - `hinst` is a valid module handle provided by the OS
/// - `call_reason` is a valid DLL notification code
/// - `_reserved` may be null or a valid pointer depending on `call_reason`
///
/// `#[unsafe(no_mangle)]` is required for the Windows loader to find this symbol.
///
/// # ロード起点 spawn / loader lock 回避（R4.4）
/// DllMain attach ではアクタースレッドを spawn しない（DllMain は loader lock 保持下で
/// 呼ばれ、スレッド生成は deadlock を招きうる）。スレッド spawn は SHIORI `load` 起点で
/// 行う。detach ではアクターを teardown する（VM・スレッド・チャネルの解放）。
#[unsafe(no_mangle)]
extern "system" fn DllMain(
    _hinst: isize,
    call_reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> bool {
    const DLL_PROCESS_DETACH: u32 = 0;

    if call_reason == DLL_PROCESS_DETACH {
        // detach: アクタースレッドを teardown（unload と重なっても冪等 no-op で安全）。
        unload()
    } else {
        // attach を含む他の通知では何もしない（spawn は load 起点・loader lock 回避）。
        true
    }
}

/// SHIORI load entry point
/// Called after DLL initialization (DllMain has already run).
///
/// # Safety
/// This function is called from external C code (SHIORI host such as SSP).
/// The caller must ensure:
/// - `hdir` is a valid HGLOBAL containing the ghost directory path encoded in
///   the system's ANSI codepage (e.g., Shift_JIS on Japanese Windows)
/// - `len` is the exact byte length of the data in `hdir`
/// - The HGLOBAL will be freed by the callee (ownership transfer)
///
/// `#[unsafe(no_mangle)]` is required for the SHIORI host to find this symbol.
#[unsafe(no_mangle)]
pub extern "C" fn load(hdir: HGLOBAL, len: usize) -> bool {
    if hdir.is_null() {
        warn!("load called with null HGLOBAL");
        return false;
    }
    // ownership of `hdir` transfers to the callee on entry: every early-return
    // path must free it (capture takes ownership; Drop frees).
    let hdir = ShioriString::capture(hdir, len);
    if len == 0 {
        warn!("load called with zero length");
        return false;
    }
    // panic 封じ込め: load 処理（dir デコード→spawn_actor）を catch_unwind で囲む。
    let result = catch_unwind(AssertUnwindSafe(|| load_impl(&hdir)));
    match result {
        Ok(rc) => rc,
        Err(p) => {
            error!("[pasta_shiori::load] panic at SHIORI boundary: {}", panic_msg(&p));
            false
        }
    }
}

/// load 本体: ANSI dir をデコードしてアクタースレッドを spawn（MAILBOX 設定）する。
fn load_impl(hdir: &ShioriString) -> bool {
    let dir = match hdir.to_ansi_str() {
        Ok(d) => d,
        Err(e) => {
            error!("[pasta_shiori::load] dir decode failed: {e}");
            return false;
        }
    };
    // hinst は本番アクター経路では VM 構築の付随情報。SHIORI 仕様上 load では渡されない
    // ため 0 を渡す（旧実装も DllMain で受けた hinst を保持していたが、本番では
    // load_dir のみが VM 構築に必須）。
    lifecycle::spawn_actor(0, std::path::PathBuf::from(dir))
}

/// SHIORI unload entry point
///
/// # Safety
/// This function is called from external C code (SHIORI host).
/// No pointer parameters; safe to call at any time after DllMain.
///
/// `#[unsafe(no_mangle)]` is required for the SHIORI host to find this symbol.
#[unsafe(no_mangle)]
pub extern "C" fn unload() -> bool {
    // teardown は冪等（二重 teardown／unload×detach 競合でも安全）。panic は封じる。
    let result = catch_unwind(AssertUnwindSafe(|| {
        let report = lifecycle::teardown_actor();
        if let Some(anomaly) = report.anomaly {
            warn!("[pasta_shiori::unload] teardown anomaly: {anomaly}");
        }
    }));
    if let Err(p) = result {
        error!("[pasta_shiori::unload] panic at SHIORI boundary: {}", panic_msg(&p));
    }
    // SHIORI 契約上 unload は常に true（既存の always-true 姿勢を維持）。
    true
}

/// SHIORI request entry point
/// Handles SHIORI requests using the initialized instance.
///
/// # Safety
/// This function is called from external C code (SHIORI host such as SSP).
/// The caller must ensure:
/// - `req` is a valid HGLOBAL containing a UTF-8 encoded SHIORI request,
///   or null (in which case this function returns null with `*len = 0`)
/// - `len` is a valid mutable reference; on entry it holds the byte length
///   of `req`, on return it is set to the byte length of the response
/// - The returned HGLOBAL is owned by the caller (must be freed by caller)
/// - The input HGLOBAL `req` will be freed by the callee (ownership transfer)
///
/// `#[unsafe(no_mangle)]` is required for the SHIORI host to find this symbol.
#[unsafe(no_mangle)]
pub extern "C" fn request(req: HGLOBAL, len: &mut usize) -> HGLOBAL {
    if req.is_null() {
        warn!("request called with null HGLOBAL");
        *len = 0;
        return ptr::null_mut();
    }
    // ownership transfer: capture frees the incoming HGLOBAL on every path.
    let hreq = ShioriString::capture(req, *len);
    let result = catch_unwind(AssertUnwindSafe(|| request_impl(&hreq)));
    match result {
        Ok((res, res_len)) => {
            *len = res_len;
            res
        }
        Err(p) => {
            error!("[pasta_shiori::request] panic at SHIORI boundary: {}", panic_msg(&p));
            // panic 時も 204 を返し（marshaling の安全網と同一バイト）、ホストへ unwind しない。
            emit_response(&crate::actor::marshaling::default_204(), len)
        }
    }
}

/// request 本体: UTF-8 デコード → marshaling → 応答 HGLOBAL 化。
fn request_impl(hreq: &ShioriString) -> (HGLOBAL, usize) {
    let req = match hreq.to_utf8_str() {
        Ok(r) => r,
        Err(e) => {
            // 不正バイト列: 安全網 204（旧実装は 500 を返したが、本番アクター経路は
            // SHIORI スレッドを無限待機させず必ず文字列を返す契約に統一する・R5.6）。
            warn!("[pasta_shiori::request] utf8 decode failed: {e}");
            let mut len = 0usize;
            return emit_response_into(&crate::actor::marshaling::default_204(), &mut len);
        }
    };
    let response = lifecycle::marshal_request(req);
    let mut len = 0usize;
    emit_response_into(&response, &mut len)
}

/// 応答文字列を HGLOBAL（nofree・呼び出し側所有）へ載せ、`*len` を更新して返す。
/// 確保失敗時は null+len0（ホストは null を「応答なし」と解釈する）。
fn emit_response(response: &str, len: &mut usize) -> HGLOBAL {
    let (h, l) = emit_response_into(response, len);
    *len = l;
    h
}

/// 応答文字列を HGLOBAL 化して `(handle, len)` を返す（`len` out も更新）。
fn emit_response_into(response: &str, len: &mut usize) -> (HGLOBAL, usize) {
    match ShioriString::clone_from_str_nofree(response) {
        Ok(hres) => {
            let (h, l) = hres.value();
            *len = l;
            (h, l)
        }
        Err(_) => {
            *len = 0;
            (ptr::null_mut(), 0)
        }
    }
}

/// 捕捉した panic ペイロードからメッセージ文字列を取り出す。
fn panic_msg(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
