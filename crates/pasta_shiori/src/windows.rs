//! Windows SHIORI DLL interface
//!
//! Provides SHIORI protocol entry points for Windows DLL.

use crate::error::*;
use crate::shiori::*;
use crate::util::hglobal::*;
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
    if hdir.is_null() || len == 0 {
        warn!("load called with null HGLOBAL or zero length");
        return false;
    }
    // SHIORI is already initialized in DllMain
    match SHIORI.get() {
        Some(raw) => raw.load(hdir, len),
        None => false,
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
            *len = 0;
            ptr::null_mut()
        }
    }
}

struct RawShiori<T: Shiori + Default + Sized>(isize, Arc<Mutex<Option<T>>>);

impl<T: Shiori + Default + Sized> RawShiori<T> {
    fn new(hinst: isize) -> Self {
        // Note: tracing subscriber is NOT initialized here.
        // It is deferred to PastaShiori::load() after pasta.toml is read,
        // so that [logging] configuration can be applied (Requirement 6).
        RawShiori(hinst, Arc::new(Mutex::new(None)))
    }

    fn unload(&self) -> bool {
        match self.unload_impl() {
            Ok(_) => (),
            Err(e) => {
                error!("[pasta_shiori::unload] {e}");
            }
        };
        true
    }

    fn load(&self, hdir: HGLOBAL, len: usize) -> bool {
        match self.load_impl(hdir, len) {
            Ok(ret) => ret,
            Err(e) => {
                error!("[pasta_shiori::load] {e}");
                false
            }
        }
    }

    fn request(&self, req: HGLOBAL, len: &mut usize) -> HGLOBAL {
        match self.request_impl(req, *len) {
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
        let mut guard = self.1.lock()?;
        *guard = None;
        let mut shiori = T::default();
        let hdir = ShioriString::capture(hdir, len);
        let dir = hdir.to_ansi_str()?;
        let rc = shiori.load(hinst, dir)?;
        *guard = Some(shiori);
        Ok(rc)
    }

    fn request_impl(&self, hreq: HGLOBAL, len: usize) -> MyResult<(HGLOBAL, usize)> {
        let mut guard = self.1.lock()?;
        match *guard {
            None => Err(MyError::NotInitialized),
            Some(ref mut shiori) => {
                let hreq = ShioriString::capture(hreq, len);
                let req = hreq.to_utf8_str()?;
                let res = shiori.request(req)?;
                let hres = ShioriString::clone_from_str_nofree(res);
                Ok(hres.value())
            }
        }
    }

    fn error_response(e: MyError) -> (HGLOBAL, usize) {
        let res = e.to_shiori_response();
        let hres = ShioriString::clone_from_str_nofree(res);
        hres.value()
    }
}
