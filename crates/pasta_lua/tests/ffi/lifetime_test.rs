//! DLL の参照の寿命の確かめ（要件 7.x・9.8）。
//!
//! DLL のモジュールはプロセス全体で共有されるので、`GetModuleHandleW` の有無で
//! 「まだ読み込まれているか」を見る。

use std::path::Path;

use pasta_lua::{PastaLoader, PastaLuaRuntime};

use crate::dll_fixture::{
    FFI_TOML, VIA_DEP_VALUE, Which, loaded_module, lock, make_ghost, put_dlls,
};

/// 同じ名前の別の DLL の取り違えの文言（collision_test と同じ）。
const COLLISION_MSG: &str =
    "cannot load module 'foo': a different DLL named 'foo.dll' is already loaded in this process";

fn ghost_vm(base: &Path) -> PastaLuaRuntime {
    let runtime = PastaLoader::load(base)
        .unwrap_or_else(|e| panic!("ゴーストを読めない: {}: {e}", base.display()));
    runtime
        .lua()
        .load("ffi.cdef [[ int add(int, int); int via_dep(void); int which(void); void release(void *); ]]")
        .exec()
        .unwrap();
    runtime
}

/// テストの前提: ほかのテストの DLL が残っていない。
fn assert_nothing_loaded(when: &str) {
    assert_eq!(
        loaded_module("foo.dll"),
        None,
        "{when}: foo.dll が残っている"
    );
    assert_eq!(
        loaded_module("dep.dll"),
        None,
        "{when}: dep.dll が残っている"
    );
}

/// `GetModuleFileNameW` のパス（`\\?\` が付くことがある）。
fn module_path(h: usize) -> String {
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
    let mut buf = vec![0u16; 32 * 1024];
    // SAFETY: h は読み込み済みのモジュール。buf は buf.len() 個の u16。
    let len = unsafe { GetModuleFileNameW(h as _, buf.as_mut_ptr(), buf.len() as u32) };
    assert!(len > 0, "GetModuleFileNameW が失敗");
    String::from_utf16_lossy(&buf[..len as usize])
}

/// `ffi.load` の戻り値を捨てて GC しても、先に取った関数が呼べる（7.1）。
#[test]
fn dropped_clib_keeps_functions_callable_after_gc() {
    let _guard = lock();
    assert_nothing_loaded("前");
    let temp = tempfile::TempDir::new().unwrap();
    let a = make_ghost(&temp.path().join("a"), FFI_TOML);
    put_dlls(&a.join("scripts"), Which::V1);

    let vm = ghost_vm(&a);
    vm.lua()
        .load(
            r#"
            do
                local foo = ffi.load("foo")
                ADD, VIA_DEP = foo.add, foo.via_dep
            end
            collectgarbage()
            collectgarbage()
            "#,
        )
        .exec()
        .unwrap();
    // 元の ffi.load だけなら、ここで clib の __gc が FreeLibrary して foo.dll は消える。
    assert!(
        loaded_module("foo.dll").is_some(),
        "GC で foo.dll が解放された"
    );
    let (sum, dep): (i32, i32) = vm.lua().load("return ADD(1, 2), VIA_DEP()").eval().unwrap();
    assert_eq!((sum, dep), (3, VIA_DEP_VALUE));

    drop(vm);
    assert_nothing_loaded("VM を落とした後");
}

/// `ffi.gc` の後始末を残したまま落としても落ちない。落とした後は何も残らず、新しい VM で読み直せる（7.2・7.3・9.8）。
#[test]
fn closing_with_pending_ffi_gc_frees_dlls_and_reload_works() {
    let _guard = lock();
    assert_nothing_loaded("前");
    let temp = tempfile::TempDir::new().unwrap();
    let a = make_ghost(&temp.path().join("a"), FFI_TOML);
    put_dlls(&a.join("scripts"), Which::V1);

    let vm = ghost_vm(&a);
    let dep: i32 = vm
        .lua()
        .load(
            r#"
            local foo = ffi.load("foo")
            -- グローバルに置いて、lua_close まで後始末（foo.release）を残す。
            PENDING = {}
            for i = 1, 16 do
                PENDING[i] = ffi.gc(ffi.new("int[1]"), foo.release)
            end
            return foo.via_dep()
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!(dep, VIA_DEP_VALUE);
    assert!(loaded_module("dep.dll").is_some());

    drop(vm);
    assert_nothing_loaded("VM を落とした後");

    // 再読み込みに当たる新しい VM で読み直せる。
    let vm = ghost_vm(&a);
    let (which, dep): (i32, i32) = vm
        .lua()
        .load(r#"local foo = ffi.load("foo"); return foo.which(), foo.via_dep()"#)
        .eval()
        .unwrap();
    assert_eq!((which, dep), (Which::V1 as i32, VIA_DEP_VALUE));
    drop(vm);
    assert_nothing_loaded("2 つ目の VM を落とした後");
}

/// 取り違えで失敗した B の DLL は残らない（7.4）。
#[test]
fn collision_failure_leaves_nothing_of_b() {
    let _guard = lock();
    assert_nothing_loaded("前");
    let temp = tempfile::TempDir::new().unwrap();
    let a = make_ghost(&temp.path().join("a"), FFI_TOML);
    put_dlls(&a.join("scripts"), Which::V1);
    let b = make_ghost(&temp.path().join("b"), FFI_TOML);
    put_dlls(&b.join("scripts"), Which::V2);

    let vm_a = ghost_vm(&a);
    let which: i32 = vm_a
        .lua()
        .load(r#"return ffi.load("foo").which()"#)
        .eval()
        .unwrap();
    assert_eq!(which, Which::V1 as i32);
    let h_a = loaded_module("foo.dll").expect("A の foo.dll");

    let vm_b = ghost_vm(&b);
    let (ok, err): (bool, mlua::Value) = vm_b
        .lua()
        .load(r#"return pcall(ffi.load, "foo")"#)
        .eval()
        .unwrap();
    assert!(!ok);
    let err = err.to_string().unwrap();
    assert!(err.contains(COLLISION_MSG), "{err}");

    // 名前で引けるのは A のまま。
    assert_eq!(loaded_module("foo.dll"), Some(h_a));
    // 一時フォルダは 8.3 名のことがある（CI）ので、ゴーストのフォルダからの末尾で見る。
    let path = module_path(h_a).to_lowercase();
    assert!(path.ends_with(r"\a\scripts\foo.dll"), "{path}");

    // A を落とすと、B を生かしたままでも foo.dll・dep.dll は残らない（B が何も持っていない）。
    drop(vm_a);
    assert_nothing_loaded("A を落とした後");
    drop(vm_b);
}
