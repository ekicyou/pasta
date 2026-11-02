//! モジュールの参照（design.md「ModuleRef」）。アクタースレッドが走っている間、
//! pasta.dll が外れないように参照を 1 つ持つ。`unsafe` はこのファイルの 3 か所だけ。
//!
//! `Drop` を持たない（返し忘れた参照はプロセスの終わりまで DLL を外さない、安全な側）。
//! 返すのは同じ `acquire` の結果について `release` か `release_and_exit_thread` のどちらか 1 回だけで、
//! 誰が返すかは型ではなくアクタースレッドの約束で決まる。

// 6.2 でアクタースレッドへ配線するまでは単体テストだけが使う。
#![allow(dead_code)]

/// この関数を含むモジュール（出荷では pasta.dll）の参照 1 つ。
/// `HMODULE` は `Send` でないので `usize` で持つ。
#[derive(Clone, Copy)]
pub(crate) struct ModuleRef {
    #[cfg(windows)]
    handle: usize,
}

#[cfg(windows)]
impl ModuleRef {
    /// 参照を 1 つ足す。失敗は OS の誤り（GetLastError）。
    pub(crate) fn acquire() -> std::io::Result<ModuleRef> {
        use windows_sys::Win32::System::LibraryLoader::{
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GetModuleHandleExW,
        };
        let mut h = std::ptr::null_mut();
        // SAFETY: 渡す番地はこのモジュールの中の関数 `acquire` そのもの（FROM_ADDRESS では
        // 文字列でなく番地として読まれる）。`h` は有効な書き込み先。参照の数を 1 つ足すだけ。
        let ok = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                Self::acquire as *const u16,
                &mut h,
            )
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ModuleRef { handle: h as usize })
    }

    /// 参照を返す（join を終えた親だけ。スレッドはもう DLL のコードを走らせていない）。
    pub(crate) fn release(self) {
        use windows_sys::Win32::Foundation::FreeLibrary;
        // SAFETY: `handle` は `acquire` が足した参照で、前提により返すのはこの 1 回だけ。
        // 呼び手のコードはまだ外れない（呼び手自身が別の参照で読まれている）。
        unsafe { FreeLibrary(self.handle as _) };
    }

    /// 参照を返してスレッドを終える（見捨てられたアクタースレッドの閉包の最後だけ）。戻らない。
    pub(crate) fn release_and_exit_thread(self) -> ! {
        use windows_sys::Win32::System::LibraryLoader::FreeLibraryAndExitThread;
        // SAFETY: `handle` は `acquire` が足した参照で、返すのはこの 1 回だけ。OS が参照を返して
        // そのままスレッドを終えるので、外れた DLL のコードへ戻ることはない。
        unsafe { FreeLibraryAndExitThread(self.handle as _, 0) }
    }
}

/// 非 Windows では参照を持たず、3 つとも何もしない。
#[cfg(not(windows))]
impl ModuleRef {
    pub(crate) fn acquire() -> std::io::Result<ModuleRef> {
        Ok(ModuleRef {})
    }

    pub(crate) fn release(self) {}

    pub(crate) fn release_and_exit_thread(self) {}
}

#[cfg(all(test, windows))]
mod tests {
    use super::ModuleRef;
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleHandleExW,
    };

    /// この関数を含むモジュールが読まれたままか（参照の数を変えずに引く）。
    fn module_loaded() -> bool {
        let mut h = std::ptr::null_mut();
        // SAFETY: テストの確かめ。参照の数を変えずにこの関数の番地のモジュールを引くだけ。
        let ok = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                module_loaded as *const u16,
                &mut h,
            )
        };
        ok != 0 && !h.is_null()
    }

    #[test]
    fn acquire_then_release_keeps_module_loaded() {
        let r = ModuleRef::acquire().expect("参照が取れる");
        assert!(module_loaded(), "足した後もモジュールが読まれたまま");
        r.release();
        assert!(
            module_loaded(),
            "返した後も元の参照は残る（二重に返さない）"
        );
    }

    #[test]
    fn two_refs_released_once_each_stay_balanced() {
        let a = ModuleRef::acquire().expect("1 つ目");
        let b = ModuleRef::acquire().expect("2 つ目");
        a.release();
        assert!(module_loaded());
        b.release();
        assert!(module_loaded(), "足した数だけ返しても元の参照は残る");
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::ModuleRef;

    #[test]
    fn non_windows_is_noop() {
        let r = ModuleRef::acquire().expect("非 Windows では必ず取れる");
        r.release();
        ModuleRef::acquire().unwrap().release_and_exit_thread();
    }
}
