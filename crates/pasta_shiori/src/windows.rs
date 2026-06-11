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
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalFlags};

    /// Win32 SDK `GMEM_INVALID_HANDLE` (minwinbase.h) — returned by
    /// `GlobalFlags` for freed/invalid handles. Not exported by
    /// windows-sys 0.61, so defined locally for the leak probes below.
    const GMEM_INVALID_HANDLE: u32 = 0x8000;

    /// Mock SHIORI driven purely by its inputs (RawShiori creates T::default()
    /// on each load, so behavior must be encoded in the load dir / request text).
    #[derive(Default)]
    struct MockShiori {
        loaded_dir: Option<String>,
    }

    impl Shiori for MockShiori {
        fn load<S: AsRef<OsStr>>(&mut self, _hinst: isize, dir: S) -> MyResult<bool> {
            let d = dir.as_ref().to_string_lossy().to_string();
            if d.contains("panicload") {
                panic!("mock load panic");
            }
            if d.contains("loaderr") {
                return Err(MyError::Load("mock load failure".to_string()));
            }
            let rc = !d.contains("loadfalse");
            self.loaded_dir = Some(d);
            Ok(rc)
        }

        fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
            let r = req.as_ref();
            if r == "PANIC" {
                panic!("mock request panic");
            }
            if r == "ERR" {
                return Err(MyError::Script {
                    message: "mock script failure".to_string(),
                });
            }
            Ok(format!(
                "RESP:{}:{}",
                self.loaded_dir.as_deref().unwrap_or("-"),
                r
            ))
        }
    }

    /// Panics in Drop when loaded from a "panicdrop" dir — exercises the
    /// unload path (unload_impl drops the stored instance under the lock).
    impl Drop for MockShiori {
        fn drop(&mut self) {
            if self
                .loaded_dir
                .as_deref()
                .is_some_and(|d| d.contains("panicdrop"))
            {
                panic!("mock drop panic");
            }
        }
    }

    /// Allocate an HGLOBAL holding `text` (ASCII only — load() decodes via the
    /// locale-dependent ANSI codepage, and ASCII is invariant across codepages).
    /// Ownership is transferred to the callee (which captures and frees it).
    fn alloc_hglobal(text: &str) -> (HGLOBAL, usize) {
        let s = ShioriString::clone_from_slice_nofree(text.as_bytes()).unwrap();
        (s.handle(), s.len())
    }

    /// Read a response HGLOBAL as UTF-8 and free it.
    fn read_and_free(h: HGLOBAL, len: usize) -> String {
        let s = ShioriString::capture(h, len);
        s.to_utf8_str().unwrap().to_string()
    }

    #[test]
    fn raw_load_and_request_roundtrip() {
        let raw = RawShiori::<MockShiori>::new(7);

        let (h, len) = alloc_hglobal("C:/mock/dir");
        assert!(raw.load(h, len), "load should report mock success");

        let (h, mut len) = alloc_hglobal("PING");
        let res = raw.request(h, &mut len);
        assert!(!res.is_null(), "response HGLOBAL should be non-null");
        let body = read_and_free(res, len);
        // Pins: ANSI dir decode passthrough, UTF-8 request decode, len update.
        assert_eq!(body, "RESP:C:/mock/dir:PING");
        assert_eq!(len, body.len(), "out-len must match response byte length");
    }

    #[test]
    fn raw_request_before_load_returns_500_not_initialized() {
        let raw = RawShiori::<MockShiori>::new(0);

        let (h, mut len) = alloc_hglobal("PING");
        let res = raw.request(h, &mut len);
        assert!(!res.is_null(), "error path must still return a response");
        let body = read_and_free(res, len);
        // Exact contract: error_response() == MyError::to_shiori_response().
        assert_eq!(body, MyError::NotInitialized.to_shiori_response());
        assert!(body.starts_with("SHIORI/3.0 500 Internal Server Error\r\n"));
    }

    #[test]
    fn raw_unload_resets_state_and_is_idempotent() {
        let raw = RawShiori::<MockShiori>::new(0);

        // unload before any load: still reports true (unload_impl always Ok).
        assert!(raw.unload(), "unload on never-loaded instance returns true");

        let (h, len) = alloc_hglobal("C:/mock/dir");
        assert!(raw.load(h, len));
        assert!(raw.unload(), "unload after load returns true");

        // After unload the instance slot is cleared → 500 Not initialized.
        let (h, mut len) = alloc_hglobal("PING");
        let body = read_and_free(raw.request(h, &mut len), len);
        assert_eq!(body, MyError::NotInitialized.to_shiori_response());
    }

    #[test]
    fn raw_request_propagates_shiori_error_as_500() {
        let raw = RawShiori::<MockShiori>::new(0);
        let (h, len) = alloc_hglobal("C:/mock/dir");
        assert!(raw.load(h, len));

        let (h, mut len) = alloc_hglobal("ERR");
        let body = read_and_free(raw.request(h, &mut len), len);
        assert!(body.starts_with("SHIORI/3.0 500 Internal Server Error\r\n"));
        assert!(
            body.contains("X-ERROR-REASON: Script error: mock script failure\r\n"),
            "X-ERROR-REASON should carry the Shiori error Display: {body}"
        );
    }

    #[test]
    fn raw_load_error_returns_false_and_leaves_uninitialized() {
        let raw = RawShiori::<MockShiori>::new(0);

        let (h, len) = alloc_hglobal("C:/loaderr");
        assert!(!raw.load(h, len), "load error must surface as false");

        // load_impl clears the slot before loading; on Err it stays None.
        let (h, mut len) = alloc_hglobal("PING");
        let body = read_and_free(raw.request(h, &mut len), len);
        assert_eq!(body, MyError::NotInitialized.to_shiori_response());
    }

    #[test]
    fn raw_load_false_still_stores_instance() {
        // Characterization: load_impl stores the instance even when the
        // Shiori impl reports load failure (Ok(false)). Requests afterwards
        // are dispatched to that instance rather than returning 500.
        // (PastaShiori handles its own degraded-request path internally.)
        let raw = RawShiori::<MockShiori>::new(0);

        let (h, len) = alloc_hglobal("C:/loadfalse");
        assert!(!raw.load(h, len), "mock reports load failure");

        let (h, mut len) = alloc_hglobal("PING");
        let body = read_and_free(raw.request(h, &mut len), len);
        assert_eq!(body, "RESP:C:/loadfalse:PING");
    }

    // ------------------------------------------------------------------
    // 3.37 (G3): panic containment at the FFI dispatch layer.
    //
    // RawShiori::{load,request,unload} sit directly under the extern "C"
    // entry points. A panic unwinding out of them would cross the FFI
    // boundary: historical UB, and since Rust 1.81 an immediate abort that
    // kills the host process (SSP). The dispatch layer must convert panics
    // into the SHIORI error contract (load→false, request→500, unload→true).
    // ------------------------------------------------------------------

    #[test]
    fn raw_request_catches_panic_as_500_then_poison_500() {
        let raw = RawShiori::<MockShiori>::new(0);
        let (h, len) = alloc_hglobal("C:/mock/dir");
        assert!(raw.load(h, len));

        // The mock panics inside Shiori::request while the mutex is held.
        let (h, mut len) = alloc_hglobal("PANIC");
        let res = raw.request(h, &mut len);
        assert!(!res.is_null(), "panic path must still produce a response");
        let body = read_and_free(res, len);
        assert!(
            body.starts_with("SHIORI/3.0 500 Internal Server Error\r\n"),
            "panic must surface as a 500 response, not unwind: {body}"
        );
        assert!(
            body.contains("mock request panic"),
            "panic payload should be carried in X-ERROR-REASON: {body}"
        );

        // The panic poisoned the instance mutex; subsequent requests must
        // degrade to the Poison 500 response instead of panicking again.
        // The probe also pins the ownership contract on the poison path:
        // request_impl captures the incoming HGLOBAL BEFORE taking the lock,
        // so even the Err(Poison) early return frees it. (A moveable probe is
        // safe here — the poison path never dereferences the payload.)
        let h = alloc_leak_probe();
        let mut len = LEAK_PROBE_LEN;
        let body = read_and_free(raw.request(h, &mut len), len);
        assert_eq!(body, MyError::Poison.to_shiori_response());
        assert!(
            hglobal_is_freed(h),
            "poisoned-lock request path must free the incoming HGLOBAL"
        );
    }

    #[test]
    fn raw_request_not_initialized_frees_input() {
        // 3.37 (G3, round 2): request before load returns Err(NotInitialized)
        // — but ownership of the incoming HGLOBAL still transferred to the
        // callee on entry, so this early return must free it (previously
        // leaked). Fixed by capturing before the lock in request_impl.
        let raw = RawShiori::<MockShiori>::new(0);

        let h = alloc_leak_probe();
        let mut len = LEAK_PROBE_LEN;
        let body = read_and_free(raw.request(h, &mut len), len);
        assert_eq!(body, MyError::NotInitialized.to_shiori_response());
        assert!(
            hglobal_is_freed(h),
            "NotInitialized request path must free the incoming HGLOBAL"
        );
    }

    #[test]
    fn raw_load_poisoned_lock_returns_false_and_frees_input() {
        // 3.37 (G3, round 2): load with a poisoned mutex returns
        // Err(Poison)→false — load_impl must capture hdir before lock() so
        // this early return frees the incoming HGLOBAL (previously leaked).
        let raw = RawShiori::<MockShiori>::new(0);
        let (h, len) = alloc_hglobal("C:/panicload");
        assert!(!raw.load(h, len), "panic during load poisons the mutex");

        let h = alloc_leak_probe();
        assert!(!raw.load(h, LEAK_PROBE_LEN), "poisoned lock → load false");
        assert!(
            hglobal_is_freed(h),
            "poisoned-lock load path must free the incoming HGLOBAL"
        );
    }

    #[test]
    fn raw_load_catches_panic_and_returns_false() {
        let raw = RawShiori::<MockShiori>::new(0);
        let (h, len) = alloc_hglobal("C:/panicload");
        assert!(!raw.load(h, len), "panic during load must surface as false");

        // Mutex got poisoned under the panic → request degrades to Poison 500.
        let (h, mut len) = alloc_hglobal("PING");
        let body = read_and_free(raw.request(h, &mut len), len);
        assert_eq!(body, MyError::Poison.to_shiori_response());
    }

    #[test]
    fn raw_unload_catches_panic_from_instance_drop() {
        let raw = RawShiori::<MockShiori>::new(0);
        let (h, len) = alloc_hglobal("C:/panicdrop");
        assert!(raw.load(h, len));

        // unload_impl drops the stored instance; the mock's Drop panics.
        // The dispatch layer must swallow it and keep the always-true
        // contract of unload.
        assert!(raw.unload());
    }

    // ------------------------------------------------------------------
    // extern "C" entry points with the process-global SHIORI uninitialized
    // (DllMain never runs in a test binary linking the rlib).
    // ------------------------------------------------------------------

    // ------------------------------------------------------------------
    // 3.37 (G3): SHIORI ownership-transfer contract on early-return paths.
    //
    // The host transfers ownership of the incoming HGLOBAL to the callee;
    // the guard paths (zero length / SHIORI uninitialized) previously
    // returned without freeing it, leaking host memory on every such call.
    //
    // The probe is GMEM_MOVEABLE: its HGLOBAL is a handle-table entry, not
    // a raw heap pointer, so GlobalFlags can safely report
    // GMEM_INVALID_HANDLE after GlobalFree. (Probing a freed GMEM_FIXED
    // pointer makes GlobalFlags walk freed heap memory and trips
    // STATUS_HEAP_CORRUPTION.) The guard paths under test never dereference
    // the payload, so a moveable handle exercises them faithfully —
    // GlobalFree accepts both kinds of handle.
    // ------------------------------------------------------------------

    const LEAK_PROBE_LEN: usize = 64;

    fn alloc_leak_probe() -> HGLOBAL {
        // SAFETY: plain allocation; the handle is consumed (freed) by the
        // guard path under test.
        let h = unsafe { GlobalAlloc(GMEM_MOVEABLE, LEAK_PROBE_LEN) };
        assert!(!h.is_null(), "probe allocation must succeed");
        h
    }

    /// GlobalFlags returns GMEM_INVALID_HANDLE once the handle is freed.
    fn hglobal_is_freed(h: HGLOBAL) -> bool {
        // SAFETY: GlobalFlags is documented to validate its handle and
        // report GMEM_INVALID_HANDLE for freed/invalid moveable handles.
        unsafe { GlobalFlags(h) == GMEM_INVALID_HANDLE }
    }

    #[test]
    fn extern_load_null_or_zero_len_returns_false_and_frees_input() {
        assert!(!load(ptr::null_mut(), 5), "null HGLOBAL must be rejected");

        // Non-null handle with len == 0 is rejected — and the input is owned
        // by the callee, so the guard path must free it.
        let h = alloc_leak_probe();
        assert!(!load(h, 0), "zero length must be rejected");
        assert!(
            hglobal_is_freed(h),
            "load len==0 guard must free the incoming HGLOBAL (ownership transfer)"
        );
    }

    #[test]
    fn extern_request_null_returns_null_and_zero_len() {
        let mut len = 1234usize;
        let res = request(ptr::null_mut(), &mut len);
        assert!(res.is_null());
        assert_eq!(len, 0, "out-len must be zeroed on the null path");
    }

    #[test]
    fn extern_entry_points_without_initialized_shiori_free_input() {
        // unload: OnceLock empty → false.
        assert!(!unload());

        // request with a valid handle: OnceLock empty → null + len 0, and the
        // incoming HGLOBAL must be freed (callee ownership).
        let h = alloc_leak_probe();
        let mut len = LEAK_PROBE_LEN;
        let res = request(h, &mut len);
        assert!(res.is_null());
        assert_eq!(len, 0);
        assert!(
            hglobal_is_freed(h),
            "request SHIORI-None guard must free the incoming HGLOBAL"
        );

        // load with a valid handle: OnceLock empty → false, input freed.
        let h = alloc_leak_probe();
        assert!(!load(h, LEAK_PROBE_LEN));
        assert!(
            hglobal_is_freed(h),
            "load SHIORI-None guard must free the incoming HGLOBAL"
        );
    }

    #[test]
    fn dll_main_non_attach_paths() {
        const DLL_PROCESS_DETACH: u32 = 0;
        const DLL_THREAD_ATTACH: u32 = 2;

        // DETACH delegates to unload(); with SHIORI uninitialized that is
        // false, and DllMain forwards it.
        assert!(!DllMain(0, DLL_PROCESS_DETACH, ptr::null_mut()));

        // Any other reason code is a no-op returning true.
        assert!(DllMain(0, DLL_THREAD_ATTACH, ptr::null_mut()));
        assert!(DllMain(0, 99, ptr::null_mut()));
    }
}
