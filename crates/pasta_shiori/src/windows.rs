//! Windows SHIORI DLL interface
//!
//! # 所有モデル
//! VM はアクタースレッドへ pin し、`static LEGACY`（[`crate::actor::lifecycle`]）越しの
//! メッセージ送信でのみアクセスする。VM（`!Send`）はアクタースレッドを越えないので
//! `unsafe impl Send/Sync` は要らない。
//!
//! - `loadu` / `load` → [`lifecycle::spawn_actor`]。スレッド spawn は **load 起点**で行い loader lock を
//!   回避する。
//! - `request`→ [`lifecycle::marshal_request`]（GET=block-on-reply／NOTIFY=即 204）。
//! - `unload` → [`lifecycle::teardown_actor`]（`Stop{done}` ack の後に join）。
//! - `DllMain` → 何もしない（アクタースレッドがモジュールの参照を持つので、生きたアクターがいる間は
//!   FreeLibrary の detach は来ない）。
//!
//! # FFI の型
//! 戻り値は C の `BOOL`（32 ビットの int・1／0）、長さは C の `long`（Windows では x86・x64 とも
//! 32 ビット）。負の長さは 0 と
//! 同じに扱い、応答の長さが `long` に収まらなければ null と 0 を返す。
//!
//! # 初期化入口は 2 つ
//! DLL 共通仕様は設置パスを UTF-8 で渡す [`loadu`]（SSP 2.6.92 以降・`load` より優先して
//! 呼ばれる）と、システム ANSI で渡す従来の [`load`] を定める。両者の差は**パスのデコード
//! 方式のみ**で、所有権規約（受け取った HGLOBAL を全経路で解放）と panic 封じ込めは同一。
//! `loadu` で初期化済みのとき `load` は HGLOBAL を解放したうえで何もせず TRUE を返す
//! （仕様の推奨）。フラグは `unload` で下ろす。
//!
//! # panic 封じ込め
//! 各 extern 入口は [`catch_unwind`] で panic を SHIORI エラー契約へ封じる
//! （unwind プロファイル＝dev/test 向けの保険。release `panic=abort` では到達不能）。
//! アクタースレッド上の panic は（unwind プロファイルでは）アクタースレッドだけを終わらせ、
//! reply の drop により 204 へ倒れる（SHIORI スレッドへ unwind しない）。
//! リクエスト処理が**エラー**で終わった場合は 204 ではなく 500 応答を返す
//! （204 は panic／reply 喪失時の安全網のみ）。

use crate::actor::lifecycle;
use crate::util::hglobal::*;
use std::ffi::c_long;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, warn};
use windows_sys::Win32::Foundation::*;

/// Windows DLL entry point
///
/// # Safety
/// This function is called by the Windows loader. The caller must ensure:
/// - `hinst` is a valid module handle provided by the OS
/// - `call_reason` is a valid DLL notification code
/// - `reserved` may be null or a valid pointer depending on `call_reason`
///
/// `#[unsafe(no_mangle)]` is required for the Windows loader to find this symbol.
///
/// どの通知でも何もせず 1 を返す。スレッド spawn は SHIORI `load` 起点で行う（loader lock 回避）。
#[unsafe(no_mangle)]
extern "system" fn DllMain(
    _hinst: isize,
    _call_reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> i32 {
    // アクタースレッドがモジュールの参照を持つので生きたアクターがいる間 FreeLibrary の detach は来ず、
    // loader lock の中で Rust の後始末を走らせるのは危ないので、ここでは何もしない。
    1
}

/// `loadu` で初期化済みかを保持するプロセス全域フラグ。
///
/// `loadu` が `spawn_actor` へ到達した時点で立て（**ロード失敗時も立てる**）、`unload` で
/// 下ろす。失敗時も立てるのは、後続の `load` が ANSI パスで再ロードすると `loadu` の失敗
/// 原因（`last_load_error`）が「欠落したパスによる別の失敗」で上書きされ、可視化される
/// 原因が変わってしまうためである。
static LOADU_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// 設置パスのデコード方式（2 つの初期化入口の唯一の差）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum DirEncoding {
    /// `loadu`: UTF-8（非 ANSI 文字を欠落させない）。
    Utf8,
    /// `load`: システム ANSI コードページ。
    Ansi,
}

/// SHIORI loadu entry point（DLL 共通仕様・SSP 2.6.92 以降）
/// Called after DLL initialization (DllMain has already run).
///
/// # Safety
/// This function is called from external C code (SHIORI host such as SSP).
/// The caller must ensure:
/// - `hdir` is a valid HGLOBAL containing the ghost directory path encoded in UTF-8
/// - `len` is the exact byte length of the data in `hdir`
/// - The HGLOBAL will be freed by the callee (ownership transfer)
///
/// `#[unsafe(no_mangle)]` is required for the SHIORI host to find this symbol.
#[unsafe(no_mangle)]
pub extern "C" fn loadu(hdir: HGLOBAL, len: c_long) -> i32 {
    load_entry("loadu", hdir, len_from_c(len), DirEncoding::Utf8).into()
}

/// SHIORI load entry point
/// Called after DLL initialization (DllMain has already run).
///
/// `loadu` で初期化済みのときは、受け取った HGLOBAL を解放したうえで何もせず TRUE を
/// 返す。そうでなければ従来どおり ANSI デコードでロードする。
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
pub extern "C" fn load(hdir: HGLOBAL, len: c_long) -> i32 {
    load_entry("load", hdir, len_from_c(len), DirEncoding::Ansi).into()
}

/// 2 つの初期化入口の共通本体（デコード方式のみが異なる）。
fn load_entry(entry: &str, hdir: HGLOBAL, len: usize, encoding: DirEncoding) -> bool {
    if hdir.is_null() {
        warn!("{entry} called with null HGLOBAL");
        return false;
    }
    // ownership of `hdir` transfers to the callee on entry: every early-return
    // path must free it (capture takes ownership; Drop frees).
    let hdir = ShioriString::capture(hdir, len);
    if encoding == DirEncoding::Ansi && LOADU_INITIALIZED.load(Ordering::SeqCst) {
        // loadu で初期化済み: 再ロードせず成功を返す（HGLOBAL は Drop で解放される）。
        warn!("[pasta_shiori::load] ignored: already initialized via loadu");
        return true;
    }
    if len == 0 {
        warn!("{entry} called with zero length");
        return false;
    }
    let result = catch_unwind(AssertUnwindSafe(|| load_impl(entry, &hdir, encoding)));
    match result {
        Ok(rc) => rc,
        Err(p) => {
            error!(
                "[pasta_shiori::{entry}] panic at SHIORI boundary: {}",
                panic_msg(&p)
            );
            false
        }
    }
}

fn load_impl(entry: &str, hdir: &ShioriString, encoding: DirEncoding) -> bool {
    let dir = match decode_dir(hdir, encoding) {
        Ok(d) => d,
        Err(e) => {
            error!("[pasta_shiori::{entry}] dir decode failed: {e}");
            return false;
        }
    };
    if encoding == DirEncoding::Utf8 {
        // アクター起動へ到達した時点で立てる（ロード失敗時も維持する）。
        LOADU_INITIALIZED.store(true, Ordering::SeqCst);
    }
    let dir_for_log = dir.clone();
    let loaded = lifecycle::spawn_actor(dir);
    // どちらの入口でロードされ成否はどうだったかを 1 行で記録する（ホスト差分の切り分け用）。
    // ファイル購読者は spawn_actor の中で登録されるので、出せるのはその復帰後。
    // ログの振り分けは load_dir のスレッドローカルで決まり、ここは FFI 入口スレッドなので、
    // 同じ load_dir でガードを張り直してから出す（文脈なしでは捨てられる）。
    let _guard = pasta_lua::LoadDirGuard::new(dir_for_log);
    info!(target: pasta_lua::logging::LOAD_ENTRY_TARGET, entry, loaded, "SHIORI load entry completed");
    loaded
}

/// 設置パスをデコードする（`loadu`=UTF-8／`load`=ANSI）。
fn decode_dir(
    hdir: &ShioriString,
    encoding: DirEncoding,
) -> crate::error::MyResult<std::path::PathBuf> {
    match encoding {
        DirEncoding::Utf8 => Ok(std::path::PathBuf::from(hdir.to_utf8_str()?)),
        DirEncoding::Ansi => Ok(std::path::PathBuf::from(hdir.to_ansi_str()?)),
    }
}

/// SHIORI unload entry point
///
/// # Safety
/// This function is called from external C code (SHIORI host).
/// No pointer parameters; safe to call at any time after DllMain.
///
/// `#[unsafe(no_mangle)]` is required for the SHIORI host to find this symbol.
#[unsafe(no_mangle)]
pub extern "C" fn unload() -> i32 {
    // 以後の load は従来どおり ANSI でロードする。
    LOADU_INITIALIZED.store(false, Ordering::SeqCst);
    let result = catch_unwind(|| {
        let report = lifecycle::teardown_actor();
        if let Some(anomaly) = report.anomaly {
            warn!("[pasta_shiori::unload] teardown anomaly: {anomaly}");
        }
    });
    if let Err(p) = result {
        error!(
            "[pasta_shiori::unload] panic at SHIORI boundary: {}",
            panic_msg(&p)
        );
    }
    // SHIORI 契約上 unload は常に TRUE。
    1
}

/// SHIORI request entry point
///
/// # Safety
/// This function is called from external C code (SHIORI host such as SSP).
/// The caller must ensure:
/// - `req` is a valid HGLOBAL containing a UTF-8 encoded SHIORI request,
///   or null (in which case this function returns null with `*len = 0`)
/// - `len` is null (this function then frees `req` and returns null) or a valid
///   pointer; on entry it holds the byte length of `req` (negative is treated
///   as 0), on return it is set to the byte length of the response
/// - The returned HGLOBAL is owned by the caller (must be freed by caller)
/// - The input HGLOBAL `req` will be freed by the callee (ownership transfer)
///
/// `#[unsafe(no_mangle)]` is required for the SHIORI host to find this symbol.
#[unsafe(no_mangle)]
#[expect(
    clippy::not_unsafe_ptr_arg_deref,
    reason = "C の ABI の書き出し。ポインタの正しさはホストとの契約（Safety）で、ほかの入口の HGLOBAL と同じく unsafe にしない"
)]
pub extern "C" fn request(req: HGLOBAL, len: *mut c_long) -> HGLOBAL {
    // SAFETY: 非 null の `len` はホストが有効な `long` を指す契約（上の Safety）。
    let Some(len) = (unsafe { len.as_mut() }) else {
        warn!("request called with null length pointer");
        if !req.is_null() {
            // 所有は受けた時点で移るので、長さが分からなくても解放する。
            drop(ShioriString::capture(req, 0));
        }
        return ptr::null_mut();
    };
    if req.is_null() {
        warn!("request called with null HGLOBAL");
        *len = 0;
        return ptr::null_mut();
    }
    // ownership transfer: capture frees the incoming HGLOBAL on every path.
    let hreq = ShioriString::capture(req, len_from_c(*len));
    let result = catch_unwind(AssertUnwindSafe(|| request_impl(&hreq)));
    let (res, res_len) = result.unwrap_or_else(|p| {
        error!(
            "[pasta_shiori::request] panic at SHIORI boundary: {}",
            panic_msg(&p)
        );
        // panic 時も 204 を返し（marshaling の安全網と同一バイト）、ホストへ unwind しない。
        emit_response(&crate::actor::marshaling::default_204())
    });
    let (res, res_len) = len_to_c(res, res_len);
    *len = res_len;
    res
}

fn request_impl(hreq: &ShioriString) -> (HGLOBAL, usize) {
    let req = match hreq.to_utf8_str() {
        Ok(r) => r,
        Err(e) => {
            // 不正バイト列も安全網の 204 を返す（必ず文字列を返す契約）。
            warn!("[pasta_shiori::request] utf8 decode failed: {e}");
            return emit_response(&crate::actor::marshaling::default_204());
        }
    };
    let response = lifecycle::marshal_request(req);
    emit_response(&response)
}

/// 応答文字列を HGLOBAL（nofree・呼び出し側所有）へ載せて `(handle, len)` を返す。
/// 確保失敗時は null+len0（ホストは null を「応答なし」と解釈する）。
fn emit_response(response: &str) -> (HGLOBAL, usize) {
    match ShioriString::clone_from_str_nofree(response) {
        Ok(hres) => hres.value(),
        Err(_) => (ptr::null_mut(), 0),
    }
}

/// C の `long` で受けた長さを `usize` にする。負は 0 と同じに扱う。
fn len_from_c(len: c_long) -> usize {
    usize::try_from(len).unwrap_or(0)
}

/// 応答の長さを C の `long` にする。収まらなければ応答を解放して null と 0 を返す
/// （確保の失敗と同じ道）。
fn len_to_c(h: HGLOBAL, len: usize) -> (HGLOBAL, c_long) {
    match c_long::try_from(len) {
        Ok(len) => (h, len),
        Err(_) => {
            error!("[pasta_shiori::request] response length {len} does not fit C long");
            if !h.is_null() {
                // SAFETY: h は emit_response が確保した呼び出し側所有の HGLOBAL で、まだ誰にも渡していない。
                unsafe { GlobalFree(h) };
            }
            (ptr::null_mut(), 0)
        }
    }
}

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
