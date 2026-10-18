//! `[loader] ffi` の読み取りと FFI を開く・開かないの確かめ（要件 1.x・9.7）。

use pasta_lua::PastaLoader;

use crate::dll_fixture::{FFI_TOML, Which, lock, make_ghost, put_dlls};

/// 煙のテスト（4.1）: `ffi = true` のゴーストでは `ffi.load("foo")` が探索のフォルダの DLL を返し、
/// `ffi = false` では `ffi` が `nil` のまま。
#[test]
fn ffi_load_smoke_reads_dll_from_search_folder() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();

    let on = make_ghost(&temp.path().join("on"), FFI_TOML);
    put_dlls(&on.join("scripts"), Which::V1);
    let runtime = PastaLoader::load(&on).expect("ffi = true のゴーストを読めない");
    let sum: i32 = runtime
        .lua()
        .load(
            r#"
            local ffi = require "ffi"
            ffi.cdef [[ int add(int a, int b); ]]
            local foo = ffi.load("foo")
            return foo.add(1, 2)
            "#,
        )
        .eval()
        .expect("ffi.load(\"foo\") が失敗した");
    assert_eq!(sum, 3);
    // 引数なしの呼び出しは、元の ffi.load と同じ `got no value` の文言のまま（6.4）。
    let no_arg: String = runtime
        .lua()
        .load("local ok, e = pcall(ffi.load); return e")
        .eval()
        .unwrap();
    assert!(no_arg.ends_with("got no value)"), "{no_arg}");
    drop(runtime);

    let off = make_ghost(
        &temp.path().join("off"),
        "[loader]\ndebug_mode = true\nffi = false\n",
    );
    let runtime = PastaLoader::load(&off).expect("ffi = false のゴーストを読めない");
    let is_nil: bool = runtime.lua().load("return ffi == nil").eval().unwrap();
    assert!(is_nil, "ffi = false なのに ffi が開いている");
}
