//! Windows SHIORI DLL interface
//!
//! Provides SHIORI protocol entry points for Windows DLL.

use crate::error::*;
use crate::shiori::*;
use crate::util::hglobal::*;
use std::ptr;
use std::sync::*;
use tracing::*;
use windows_sys::Win32::Foundation::*;

static SHIORI: OnceLock<RawShiori<PastaShiori>> = OnceLock::new();

/// Windows DLL entry point
/// Initializes SHIORI at DLL load/unload time.
///
/// # Safety
/// This is called by Windows loader.
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
/// This function is called from external C code.
#[unsafe(no_mangle)]
pub extern "C" fn load(hdir: HGLOBAL, len: usize) -> bool {
    // SHIORI is already initialized in DllMain
    match SHIORI.get() {
        Some(raw) => raw.load(hdir, len),
        None => false,
    }
}

/// SHIORI unload entry point
///
/// # Safety
/// This function is called from external C code.
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
/// This function is called from external C code.
#[unsafe(no_mangle)]
pub extern "C" fn request(req: HGLOBAL, len: &mut usize) -> HGLOBAL {
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
        // Initialize global tracing subscriber (only once)
        Self::init_tracing();

        RawShiori(hinst, Arc::new(Mutex::new(None)))
    }

    /// Initialize global tracing subscriber with GlobalLoggerRegistry.
    ///
    /// This is called once during DLL initialization. The subscriber uses
    /// GlobalLoggerRegistry to route logs to instance-specific files.
    fn init_tracing() {
        use pasta_lua::GlobalLoggerRegistry;
        use tracing_subscriber::fmt;
        use tracing_subscriber::prelude::*;

        // Try to set global subscriber. If it fails, another subscriber
        // is already set, which is fine.
        let _ = tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_writer(GlobalLoggerRegistry::instance().clone())
                    .with_ansi(false)
                    .with_target(true)
                    .with_level(true),
            )
            .try_init();
    }

    fn unload(&self) -> bool {
        match self.unload_impl() {
            Ok(_) => (),
            Err(e) => {
                error!("[pasta_shiori::unload] {e}");
                ()
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
            None => return Err(MyError::NotInitialized),
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
