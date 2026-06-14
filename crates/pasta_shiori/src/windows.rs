//! Windows SHIORI DLL interface
//!
//! Provides SHIORI protocol entry points for Windows DLL.

use crate::error::*;
use crate::shiori::*;
use crate::util::hglobal::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::*;
use tracing::{error, warn};
use windows_sys::Win32::Foundation::*;

static SHIORI: OnceLock<RawShiori<PastaShiori>> = OnceLock::new();

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
#[unsafe(no_mangle)]
extern "system" fn DllMain(
    hinst: isize,
    call_reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> bool {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => {
            // Initialize SHIORI instance when DLL is loaded
            // get_or_init ensures single initialization even if called multiple times
            SHIORI.get_or_init(|| RawShiori::new(hinst));
            true
        }
        DLL_PROCESS_DETACH => {
            // Cleanup is handled by Drop implementations
            unload()
        }
        _ => true,
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
    // 3.37 (G3): ownership of `hdir` transfers to the callee on entry, so
    // every early-return path below must free it (previously leaked).
    if len == 0 {
        warn!("load called with zero length");
        drop(ShioriString::capture(hdir, len));
        return false;
    }
    // SHIORI is already initialized in DllMain
    match SHIORI.get() {
        Some(raw) => raw.load(hdir, len),
        None => {
            drop(ShioriString::capture(hdir, len));
            false
        }
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
pub extern "C" fn unload() -> bool {
    match SHIORI.get() {
        Some(raw) => raw.unload(),
        None => false,
    }
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
    match SHIORI.get() {
        Some(raw) => raw.request(req, len),
        None => {
            // 3.37 (G3): the incoming HGLOBAL is owned by the callee even on
            // this degenerate path (previously leaked).
            drop(ShioriString::capture(req, *len));
            *len = 0;
            ptr::null_mut()
        }
    }
}

/// Convert a caught panic payload into a MyError so the FFI dispatch layer
/// can route it through the normal SHIORI error contract (3.37 / R3.7).
fn panic_to_error(payload: Box<dyn std::any::Any + Send>) -> MyError {
    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    };
    MyError::Script {
        message: format!("panic at SHIORI boundary: {msg}"),
    }
}

struct RawShiori<T: Shiori + Default + Sized>(isize, Arc<Mutex<Option<T>>>);

// 3.37 (G3): each dispatch method wraps its *_impl in catch_unwind.
// A panic unwinding out of the extern "C" entry points is undefined behavior
// historically and an immediate process abort since Rust 1.81 — either way it
// takes the host (SSP) down. catch_unwind converts it to the SHIORI error
// contract instead (load→false, request→500 response, unload→true).
// AssertUnwindSafe is sound here: the only state the closures touch is behind
// the Mutex, and a panic while holding the lock poisons it, after which every
// later call degrades to MyError::Poison rather than observing torn state.
// (In the release DLL panic=abort makes the catch unreachable; this protects
// dev/test builds and rlib consumers with the default unwind strategy.)
impl<T: Shiori + Default + Sized> RawShiori<T> {
    fn new(hinst: isize) -> Self {
        // Note: tracing subscriber is NOT initialized here.
        // It is deferred to PastaShiori::load() after pasta.toml is read,
        // so that [logging] configuration can be applied (Requirement 6).
        RawShiori(hinst, Arc::new(Mutex::new(None)))
    }

    fn unload(&self) -> bool {
        let result = catch_unwind(AssertUnwindSafe(|| self.unload_impl()))
            .unwrap_or_else(|p| Err(panic_to_error(p)));
        if let Err(e) = result {
            error!("[pasta_shiori::unload] {e}");
        }
        true
    }

    fn load(&self, hdir: HGLOBAL, len: usize) -> bool {
        let result = catch_unwind(AssertUnwindSafe(|| self.load_impl(hdir, len)))
            .unwrap_or_else(|p| Err(panic_to_error(p)));
        match result {
            Ok(ret) => ret,
            Err(e) => {
                error!("[pasta_shiori::load] {e}");
                false
            }
        }
    }

    fn request(&self, req: HGLOBAL, len: &mut usize) -> HGLOBAL {
        let result = catch_unwind(AssertUnwindSafe(|| self.request_impl(req, *len)))
            .unwrap_or_else(|p| Err(panic_to_error(p)));
        match result {
            Ok((res, res_len)) => {
                *len = res_len;
                res
            }
            Err(e) => {
                error!("[pasta_shiori::request] {e}");
                let (res, res_len) = Self::error_response(e);
                *len = res_len;
                res
            }
        }
    }
}

impl<T: Shiori + Default + Sized> RawShiori<T> {
    fn unload_impl(&self) -> MyResult<bool> {
        let mut guard = self.1.lock()?;
        *guard = None;
        Ok(true)
    }

    fn load_impl(&self, hdir: HGLOBAL, len: usize) -> MyResult<bool> {
        let hinst = self.0;
        // 3.37 (G3, round 2): capture FIRST — ownership of `hdir` transfers
        // to the callee on entry, so the poisoned-lock early return (`?`) and
        // a panic inside T::default() must both free it via Drop (previously
        // leaked). Capture only takes ownership of the handle; ordering vs
        // the lock has no protocol-visible effect.
        let hdir = ShioriString::capture(hdir, len);
        let mut guard = self.1.lock()?;
        *guard = None;
        let mut shiori = T::default();
        let dir = hdir.to_ansi_str()?;
        let rc = shiori.load(hinst, dir)?;
        *guard = Some(shiori);
        Ok(rc)
    }

    fn request_impl(&self, hreq: HGLOBAL, len: usize) -> MyResult<(HGLOBAL, usize)> {
        // 3.37 (G3, round 2): capture FIRST — the Err(Poison) and
        // Err(NotInitialized) early returns below previously returned before
        // taking ownership, leaking the incoming HGLOBAL on every request
        // sent before load (or after a panic poisoned the lock). Capture
        // only decodes/frees the request handle; moving it ahead of the lock
        // changes nothing observable for valid inputs.
        let hreq = ShioriString::capture(hreq, len);
        let mut guard = self.1.lock()?;
        match *guard {
            None => Err(MyError::NotInitialized),
            Some(ref mut shiori) => {
                let req = hreq.to_utf8_str()?;
                let res = shiori.request(req)?;
                let hres = ShioriString::clone_from_str_nofree(res)?;
                Ok(hres.value())
            }
        }
    }

    fn error_response(e: MyError) -> (HGLOBAL, usize) {
        let res = e.to_shiori_response();
        match ShioriString::clone_from_str_nofree(res) {
            Ok(hres) => hres.value(),
            // 3.37 (G3): if even the error response cannot be allocated,
            // degrade to "no response" (null + len 0) — the host treats a
            // null return as an absent response. Never hand out null+len>0.
            Err(_) => (ptr::null_mut(), 0),
        }
    }
}

// ============================================================================
// G1 (3.35): RawShiori dispatch layer / extern entry point tests.
//
// This module previously had ZERO tests. RawShiori<T> is generic over the
// Shiori trait, so the dispatch, error-response, and state-reset logic is
// testable in-process with a mock — no SSP host and no DLL loading required.
//
// IMPORTANT: none of these tests may call DllMain with DLL_PROCESS_ATTACH.
// The static SHIORI OnceLock is process-global; initializing it would make
// the "uninitialized" assertions below order-dependent across parallel tests.
// All extern-fn tests rely on the OnceLock staying uninitialized for the
// whole test process.
// ============================================================================
#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
