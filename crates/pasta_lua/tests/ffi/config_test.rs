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

/// `require "ffi"` の失敗の文言と、グローバル・`package.loaded` の `ffi` が `nil` か。
fn ffi_closed_state(ghost: &std::path::Path) -> (String, bool) {
    let runtime = PastaLoader::load(ghost).expect("ゴーストを読めない");
    runtime
        .lua()
        .load(
            r#"
            local ok, e = pcall(require, "ffi")
            assert(not ok, "require \"ffi\" が通った")
            return tostring(e), ffi == nil and package.loaded.ffi == nil
            "#,
        )
        .eval()
        .unwrap()
}

/// `ffi = false` と未記入（キーなし・`[loader]` の節なし）は同じで、FFI を開かない（1.1・1.3）。
/// 同じフォルダで pasta.toml だけを書き換えて比べる（失敗の文言は探したパスを含むため）。
#[test]
fn ffi_false_and_unset_keep_ffi_closed() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), "[loader]\ndebug_mode = true\n");
    let unset = ffi_closed_state(&ghost);

    for toml in ["[loader]\ndebug_mode = true\nffi = false\n", ""] {
        std::fs::write(ghost.join("pasta.toml"), toml).unwrap();
        assert_eq!(ffi_closed_state(&ghost), unset, "pasta.toml: {toml:?}");
    }
    let (err, nil) = unset;
    assert!(nil, "未記入なのに ffi が開いている");
    assert!(err.contains("module 'ffi' not found"), "{err}");
}

/// `ffi = true` では `ffi == require "ffi"` で、`ffi.load` は pasta のラッパ（Lua の関数）に
/// 置き換わる。元の `ffi.load` は C の関数（1.2）。
#[test]
fn ffi_true_opens_same_table_and_replaces_load() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let runtime = PastaLoader::load(&ghost).expect("ffi = true のゴーストを読めない");
    let lua = runtime.lua();
    let same: bool = lua
        .load(r#"return ffi == require "ffi" and ffi == package.loaded.ffi"#)
        .eval()
        .unwrap();
    assert!(same, "グローバルの ffi と require \"ffi\" が別の表");
    let load_info = |lua: &mlua::Lua| {
        let ffi: mlua::Table = lua.globals().get("ffi").unwrap();
        ffi.get::<mlua::Function>("load").unwrap().info()
    };
    let info = load_info(lua);
    assert_eq!(
        (info.what, info.source.as_deref()),
        ("Lua", Some("=pasta_ffi_load"))
    );

    let orig_what = load_info(&crate::dll_fixture::bare_ffi_lua()).what;
    assert_eq!(orig_what, "C");
}

/// `ffi = true` では `ffi.load` 以外の FFI（`cdef`・`new`・`C`）が LuaJIT のまま使える（1.6）。
#[test]
fn ffi_true_keeps_other_ffi_features() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let runtime = PastaLoader::load(&ghost).expect("ffi = true のゴーストを読めない");
    let (x, y, pid): (i32, f64, u32) = runtime
        .lua()
        .load(
            r#"
            ffi.cdef [[
              typedef struct { int x; double y; } pasta_pt;
              unsigned long GetCurrentProcessId(void);
            ]]
            local p = ffi.new("pasta_pt", 7, 1.5)
            return p.x, p.y, tonumber(ffi.C.GetCurrentProcessId())
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!((x, y, pid), (7, 1.5, std::process::id()));
}

/// 真偽値でない `[loader] ffi` は pasta.toml の読み込みの失敗（1.4）。
#[test]
fn ffi_non_bool_fails_to_load() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    for (i, value) in ["\"yes\"", "1"].iter().enumerate() {
        let ghost = make_ghost(
            &temp.path().join(format!("g{i}")),
            &format!("[loader]\ndebug_mode = true\nffi = {value}\n"),
        );
        let err = PastaLoader::load(&ghost)
            .err()
            .unwrap_or_else(|| panic!("ffi = {value} なのに読めた"))
            .to_string();
        assert!(err.contains("ffi"), "{err}");
    }
}
