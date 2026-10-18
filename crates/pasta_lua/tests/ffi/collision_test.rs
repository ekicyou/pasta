//! 同名の別 DLL の取り違えの確かめ（要件 5.1–5.4・9.5）。
//!
//! ゴーストごとに版の違う `foo.dll` を置き、`which()` の値でどのファイルを読んだかを見分ける。

use std::path::{Path, PathBuf};

use pasta_lua::{PastaLoader, PastaLuaRuntime};

use crate::dll_fixture::{FFI_TOML, Which, lock, make_ghost, put_dlls, short_path};

/// 取り違えの文言（`ffi_load/mod.rs` の `reason_collision` と 6.1 の形）。
const COLLISION_MSG: &str =
    "cannot load module 'foo': a different DLL named 'foo.dll' is already loaded in this process";

/// `base` のゴーストを読み、`which()` を宣言しておく（同じ VM で `cdef` を 2 回しないため）。
fn ghost_vm(base: &Path) -> PastaLuaRuntime {
    let runtime = PastaLoader::load(base)
        .unwrap_or_else(|e| panic!("ゴーストを読めない: {}: {e}", base.display()));
    runtime
        .lua()
        .load("ffi.cdef [[ int which(void); ]]")
        .exec()
        .unwrap();
    runtime
}

/// `ffi.load("foo").which()`。失敗なら `pcall` の文言。
fn load_which(runtime: &PastaLuaRuntime) -> Result<i32, String> {
    let (ok, v): (bool, mlua::Value) = runtime
        .lua()
        .load(r#"return pcall(function() return ffi.load("foo").which() end)"#)
        .eval()
        .unwrap();
    if ok {
        Ok(v.as_i32().expect("which() は整数"))
    } else {
        Err(v.to_string().unwrap())
    }
}

/// ゴースト A を保ったまま、別のフォルダのゴースト B は取り違えの文言で失敗し、A の DLL を返さない（5.1・5.2）。
/// A を落とした後は B が読める。
#[test]
fn other_ghost_same_name_fails_while_first_is_alive() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let a = make_ghost(&temp.path().join("a"), FFI_TOML);
    put_dlls(&a.join("scripts"), Which::V1);
    let b = make_ghost(&temp.path().join("b"), FFI_TOML);
    put_dlls(&b.join("scripts"), Which::V2);

    let vm_a = ghost_vm(&a);
    assert_eq!(load_which(&vm_a), Ok(Which::V1 as i32));

    let vm_b = ghost_vm(&b);
    let err = load_which(&vm_b).expect_err("B が A の DLL を返した");
    assert!(err.contains(COLLISION_MSG), "{err}");

    drop(vm_a);
    assert_eq!(load_which(&vm_b), Ok(Which::V2 as i32));
}

/// 同じ VM で 2 回読める（5.3）。同じフォルダの VM をもう 1 つ作っても読める（5.4）。
#[test]
fn same_file_loads_twice_and_from_second_vm() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let a = make_ghost(&temp.path().join("a"), FFI_TOML);
    put_dlls(&a.join("scripts"), Which::V1);

    let vm = ghost_vm(&a);
    assert_eq!(load_which(&vm), Ok(Which::V1 as i32));
    assert_eq!(load_which(&vm), Ok(Which::V1 as i32));

    let vm2 = ghost_vm(&a);
    assert_eq!(load_which(&vm2), Ok(Which::V1 as i32));
    assert_eq!(load_which(&vm), Ok(Which::V1 as i32));
}

/// `fs::canonicalize` の結果から `\\?\` を外した長い名前の綴り（8.3 名を含まない）。
fn long_form(path: &Path) -> PathBuf {
    let canon = std::fs::canonicalize(path).unwrap();
    let s = canon.to_str().expect("UTF-8 のパス");
    let s = s.strip_prefix(r"\\?\").expect("ドライブのパス");
    assert!(!s.starts_with("UNC"), "ドライブのパスで確かめる: {s}");
    PathBuf::from(s)
}

/// 基点を 8.3 名で綴った VM と長い名前で綴った VM（同じフォルダ）を並べても読める（5.4・決定事項 2）。
#[test]
fn short_and_long_spelling_of_same_folder_both_load() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    // 8 文字を超える名前のフォルダ（8.3 名が作られる）。
    let long = long_form(&make_ghost(
        &temp.path().join("pasta_long_ghost_folder"),
        FFI_TOML,
    ));
    put_dlls(&long.join("scripts"), Which::V1);
    let short = short_path(&long);
    // 空振りの緑を防ぐ: 8.3 名を作らないボリュームでは同じ文字列になり、このテストは意味を持たない。
    assert_ne!(
        short.as_os_str(),
        long.as_os_str(),
        "8.3 名が長い名前と同じ（このボリュームは 8.3 名を作らない）"
    );

    let vm_long = ghost_vm(&long);
    let vm_short = ghost_vm(&short);
    assert_eq!(load_which(&vm_long), Ok(Which::V1 as i32));
    assert_eq!(load_which(&vm_short), Ok(Which::V1 as i32));
    // 逆の順（8.3 名の側が先に持っている状態で、長い名前の側から読み直す）。
    drop(vm_long);
    let vm_long = ghost_vm(&long);
    assert_eq!(load_which(&vm_long), Ok(Which::V1 as i32));
}
