//! `ffi.load` の置き換えの `unsafe` の層（DLL の先読み・同名の確かめ・解放・理由の文言）。
//!
//! `unsafe` はこのファイルだけに置く。パスや名前は呼ぶ側（`resolve`）が作った
//! NUL 終端の UTF-16 で受け取る。

use std::ffi::c_void;
use std::ptr::{NonNull, null, null_mut};
use windows_sys::Win32::Foundation::{FreeLibrary, GetLastError};
use windows_sys::Win32::System::Diagnostics::Debug::{
    FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageW,
};
use windows_sys::Win32::System::LibraryLoader::{
    GetModuleHandleW, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
    LoadLibraryExW,
};

/// 読み込んだモジュール（NULL でない HMODULE）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Module(NonNull<c_void>);

/// NUL 終端でなければ呼び出しに渡さない（API が終端を探して読み過ぎるのを防ぐ）。
fn assert_nul_terminated(s: &[u16]) {
    assert_eq!(s.last(), Some(&0), "UTF-16 の名前は NUL 終端であること");
}

fn load(name: &[u16], flags: u32) -> Result<Module, u32> {
    assert_nul_terminated(name);
    // SAFETY: `name` は NUL 終端の UTF-16 で、呼び出しの間生きている。
    // hFile は予約で NULL を渡す決まり。
    let h = unsafe { LoadLibraryExW(name.as_ptr(), null_mut(), flags) };
    // SAFETY: 引数を取らず、このスレッドの最後のエラーを読むだけ。
    NonNull::new(h)
        .map(Module)
        .ok_or_else(|| unsafe { GetLastError() })
}

/// ゴーストのファイルを読む（DLL のフォルダ＋既定のフォルダ。カレントと `PATH` は探さない）。
/// 失敗は `GetLastError` の番号。
pub(super) fn load_ghost_dll(path: &[u16]) -> Result<Module, u32> {
    load(
        path,
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
    )
}

/// OS の既定の探索で読む（flags 0。LuaJIT の `LoadLibraryExA(name, NULL, 0)` と同じ探索）。
pub(super) fn load_system_dll(file_name: &[u16]) -> Result<Module, u32> {
    load(file_name, 0)
}

/// `GetModuleHandleW(file_name)`。参照の数は増えない。
pub(super) fn module_named(file_name: &[u16]) -> Option<Module> {
    assert_nul_terminated(file_name);
    // SAFETY: `file_name` は NUL 終端の UTF-16 で、呼び出しの間生きている。
    NonNull::new(unsafe { GetModuleHandleW(file_name.as_ptr()) }).map(Module)
}

/// `FreeLibrary`。読み込みで得た参照 1 つを返す。
pub(super) fn free(module: Module) {
    // SAFETY: `module` は `LoadLibraryExW` が返した NULL でないハンドルで、
    // 呼ぶ側はその参照 1 つにつき 1 回だけここへ渡す。
    unsafe { FreeLibrary(module.0.as_ptr()) };
}

/// `FormatMessageW` の UTF-8。言語の指定（0）と末尾の `\r\n` は LuaJIT の
/// `FormatMessageA` と同じ。文言が得られなければ空文字列。
pub(super) fn system_message(code: u32) -> String {
    let mut buf = [0u16; 512];
    // SAFETY: `buf` は書き込める `buf.len()` 個の u16。ALLOCATE_BUFFER を使わないので
    // 解放するものは無く、IGNORE_INSERTS なので引数の並びは読まれない（NULL でよい）。
    let len = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            null(),
            code,
            0,
            buf.as_mut_ptr(),
            buf.len() as u32,
            null(),
        )
    };
    String::from_utf16_lossy(&buf[..len as usize])
}

/// VM が持つ参照（1 モジュールにつき 1 つ）。mlua の app data に置き、
/// `lua_close` の後に落ちたときに全部を `FreeLibrary` する。
#[derive(Default)]
pub(super) struct DllPins(Vec<Module>);

impl DllPins {
    /// 既にあれば今回の参照を解放して false、無ければ保って true。
    pub(super) fn pin(&mut self, module: Module) -> bool {
        if self.0.contains(&module) {
            free(module);
            false
        } else {
            self.0.push(module);
            true
        }
    }
}

impl Drop for DllPins {
    fn drop(&mut self) {
        for m in self.0.drain(..) {
            free(m);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NUL 終端の UTF-16。
    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    #[test]
    fn kernel32_via_os_search_is_pinned_once() {
        let a = load_system_dll(&w("kernel32")).expect("kernel32");
        let b = load_system_dll(&w("kernel32.dll")).expect("kernel32.dll");
        assert_eq!(a, b);
        assert_eq!(module_named(&w("kernel32.dll")), Some(a));
        let mut pins = DllPins::default();
        assert!(pins.pin(a));
        assert!(!pins.pin(b), "同じモジュールは 1 つだけ持つ");
        assert_eq!(pins.0.len(), 1);
    }

    /// kernel32 は解放されないので参照の数は見えない。普段は読まれない
    /// System32 の DLL で、集まりを捨てると読み込みの前の状態へ戻ることを確かめる。
    #[test]
    fn dropping_pins_returns_ref_count() {
        let name = w("msimg32.dll");
        assert_eq!(
            module_named(&name),
            None,
            "前提: テストの前は読み込まれていない"
        );
        let a = load_system_dll(&name).expect("msimg32");
        let b = load_system_dll(&name).expect("msimg32 again");
        let mut pins = DllPins::default();
        assert!(pins.pin(a));
        assert!(!pins.pin(b));
        assert_eq!(module_named(&name), Some(a));
        drop(pins);
        assert_eq!(module_named(&name), None, "参照の数が戻っていない");
    }

    #[test]
    fn ghost_path_loads_same_module_as_name() {
        let root = std::env::var("SystemRoot").expect("SystemRoot");
        let path = w(&format!(r"{root}\System32\kernel32.dll"));
        let m = load_ghost_dll(&path).expect("kernel32 by path");
        assert_eq!(module_named(&w("kernel32.dll")), Some(m));
        free(m);
    }

    #[test]
    fn missing_name_gives_utf8_reason() {
        let code = load_system_dll(&w("pasta_no_such_module_9f3c.dll")).unwrap_err();
        assert_eq!(code, 126); // ERROR_MOD_NOT_FOUND
        let msg = system_message(code);
        assert!(!msg.trim_end().is_empty());
        assert!(msg.ends_with("\r\n"), "末尾の改行を残す: {msg:?}");
        let root = std::env::var("SystemRoot").expect("SystemRoot");
        let path = w(&format!(r"{root}\pasta_no_such_dir_9f3c\x.dll"));
        assert!(load_ghost_dll(&path).is_err());
        assert_eq!(module_named(&w("pasta_no_such_module_9f3c.dll")), None);
    }

    #[test]
    fn unknown_code_gives_empty_reason() {
        assert_eq!(system_message(0xDEAD_BEEF), "");
    }
}
