//! 深いパス・ANSI に無い名前・依存 DLL・探索のフォルダ・名前の解き方の確かめ（要件 2.x・3.x・4.x）。

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use pasta_lua::PastaLoader;

use crate::dll_fixture::{
    DEEP_MIN_LEN, FFI_TOML, VIA_DEP_VALUE, Which, assert_long_paths_disabled_if_required, deep_dir,
    loaded_module, lock, make_ghost, non_ansi_dir, put_dlls, put_foo, short_path,
};

/// 長いパスの接頭辞。文言・結果に出てはいけない（2.7）。
const VERBATIM_PREFIX: &str = r"\\?\";

/// 拡張長の接頭辞 `\\?\` を付けたパス（`path` は `/` を含まない絶対パス）。
fn verbatim(path: &Path) -> PathBuf {
    let mut s = std::ffi::OsString::from(r"\\?\");
    s.push(path.as_os_str());
    PathBuf::from(s)
}

/// `file` を DLL のフォルダ＋既定のフォルダの探し方で読み、隣の `dep.dll` を通る `via_dep()` を呼んで解放する。
/// 読めなければ `GetLastError()` を返す。
fn load_and_call_via_dep(file: &Path) -> Result<i32, u32> {
    use windows_sys::Win32::Foundation::{FreeLibrary, GetLastError};
    use windows_sys::Win32::System::LibraryLoader::{
        GetProcAddress, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        LoadLibraryExW,
    };
    let w: Vec<u16> = OsStr::new(file).encode_wide().chain(Some(0)).collect();
    // SAFETY: w は NUL 終端。テスト用に作った DLL を読む。
    let h = unsafe {
        LoadLibraryExW(
            w.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        )
    };
    if h.is_null() {
        // SAFETY: 直前の失敗の理由を読むだけ。
        return Err(unsafe { GetLastError() });
    }
    // SAFETY: 名前は NUL 終端。型は foo.c の宣言どおり（cdecl）。
    let v = unsafe {
        let via_dep: extern "C" fn() -> i32 =
            std::mem::transmute(GetProcAddress(h, c"via_dep".as_ptr().cast()).expect("via_dep"));
        via_dep()
    };
    // SAFETY: 上で得た参照を 1 つ返す。
    unsafe { FreeLibrary(h) };
    Ok(v)
}

/// 決定事項 3 の試し打ち（要件 2.5・9.10）。pasta の置き換えを通さず、300 文字超えのフォルダの `foo.dll` を
/// `\\?\` の形で読み、隣の `dep.dll` が解けることを確かめる。`\\?\` で解けなければ 8.3 名のパスを同じ形で試す。
/// CI では `LongPathsEnabled=0` にしたうえで `PASTA_TEST_LONG_PATHS_DISABLED=1` で走らせる。
#[test]
fn deep_path_dll_resolves_neighbour_dep_without_long_path_setting() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let dir = deep_dir(temp.path());
    put_dlls(&dir, Which::V1);
    let file = dir.join("foo.dll");
    assert!(file.as_os_str().len() > DEEP_MIN_LEN);
    assert_long_paths_disabled_if_required(&file);
    // ローダー（LoadLibrary*）は長いパスの設定の外にある。MS の文書の対象 API の一覧に無く、開発機の実測でも
    // CI（run 39139127106・LongPathsEnabled=0）でも接頭辞なしの長いパスの DLL が読めた。
    // なので接頭辞なしの結果は確かめに使わず、記録だけ残す（読めても読めなくても解放まで済ませる）。
    eprintln!(
        "long-path probe: unprefixed = {:?}",
        load_and_call_via_dep(&file)
    );

    let route = match load_and_call_via_dep(&verbatim(&file)) {
        Ok(v) => {
            assert_eq!(v, VIA_DEP_VALUE);
            r"\\?\"
        }
        Err(verbatim_err) => {
            // 8.3 名を得る呼び出しも長いパスなので `\\?\` の形で渡す（結果も同じ形になる）。
            let short = short_path(&verbatim(&file));
            match load_and_call_via_dep(&short) {
                Ok(v) => {
                    assert_eq!(v, VIA_DEP_VALUE);
                    "8.3"
                }
                Err(short_err) => panic!(
                    "深いパスの foo.dll を読めない: \\\\?\\ の形 = エラー {verbatim_err}・8.3 名 {} = エラー {short_err}",
                    short.display()
                ),
            }
        }
    };
    eprintln!("long-path probe: route = {route}");

    assert_eq!(loaded_module("foo.dll"), None);
    assert_eq!(loaded_module("dep.dll"), None);
}

/// `lua` で `ffi.load(NAME)` を呼ぶ。読めれば `(add(1, 2), via_dep(), tostring(結果))`、
/// 失敗すれば `pcall` が受け取った文言を返す。名前は文字列のつなぎでなくグローバルで渡す。
fn ffi_load_and_call(lua: &mlua::Lua, name: &str) -> Result<(i32, i32, String), String> {
    lua.globals().set("NAME", name).unwrap();
    let (ok, a, b, c): (bool, mlua::Value, Option<i32>, Option<String>) = lua
        .load(
            r#"
            if not PASTA_TEST_CDEF then
              ffi.cdef [[ int add(int a, int b); int via_dep(void); ]]
              PASTA_TEST_CDEF = true
            end
            local ok, foo = pcall(ffi.load, NAME)
            if not ok then return false, tostring(foo) end
            return true, foo.add(1, 2), foo.via_dep(), tostring(foo)
            "#,
        )
        .eval()
        .unwrap();
    if ok {
        let a = a.as_i32().expect("add の結果が数でない");
        Ok((a, b.unwrap(), c.unwrap()))
    } else {
        // 元の ffi.load の理由は ANSI のことがあるので、崩れても読める形で返す。
        Err(a.as_string().unwrap().to_string_lossy())
    }
}

/// `path` の区切りを `/` にした文字列。
fn slashed(path: &Path) -> String {
    path.to_str().unwrap().replace('\\', "/")
}

/// 置き場所の 3 種（300 文字超え・`NON_ANSI_DIR_NAME`・その両方）のゴーストのフォルダ。
fn placements(root: &Path) -> [(&'static str, PathBuf); 3] {
    [
        ("deep", deep_dir(&root.join("deep"))),
        ("non_ansi", non_ansi_dir(root)),
        ("both", deep_dir(&non_ansi_dir(&root.join("both")))),
    ]
}

/// 深いフォルダ・ANSI に無い文字のフォルダ・その両方のゴーストで、名前だけ・`\` の絶対パス・`/` の絶対パスの
/// どれでも `ffi.load` が読め、関数と隣の `dep.dll` を通る関数が呼べる（2.1–2.6・2.9・9.1・9.2）。
/// 結果の文字列に長いパスの接頭辞が出ない（2.7）。
#[test]
fn ffi_load_reads_dll_from_deep_and_non_ansi_folders() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    for (label, dir) in placements(temp.path()) {
        let ghost = make_ghost(&dir, FFI_TOML);
        let scripts = ghost.join("scripts");
        put_dlls(&scripts, Which::V1);
        let file = scripts.join("foo.dll");
        if label != "non_ansi" {
            assert!(file.as_os_str().len() > DEEP_MIN_LEN, "{label}");
            assert_long_paths_disabled_if_required(&file);
        } else {
            assert!(file.as_os_str().len() < 260, "{label}");
        }

        let runtime = PastaLoader::load(&ghost)
            .unwrap_or_else(|e| panic!("{label}: ゴーストを読めない: {e}"));
        let abs = file.to_str().unwrap().to_string();
        for name in ["foo".to_string(), abs.clone(), slashed(&file)] {
            let (sum, dep, shown) = ffi_load_and_call(runtime.lua(), &name)
                .unwrap_or_else(|e| panic!("{label}: ffi.load({name:?}) が失敗: {e}"));
            assert_eq!((sum, dep), (3, VIA_DEP_VALUE), "{label}: {name}");
            assert!(!shown.contains(VERBATIM_PREFIX), "{label}: {shown}");
        }
        // 次の置き場所の同名の foo.dll と取り違えないよう、VM を落として解放する。
        drop(runtime);
        assert_eq!(loaded_module("foo.dll"), None, "{label}");
        assert_eq!(loaded_module("dep.dll"), None, "{label}");
    }
}

/// 3 種の置き場所で失敗するときの文言に長いパスの接頭辞が出ない（2.7）。
/// 隣の `dep.dll` が無い（深いフォルダでは `\\?\` の形で先読みして失敗する）場合と、ファイルの無い絶対パスの場合。
#[test]
fn deep_path_failure_messages_have_no_verbatim_prefix() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    for (label, dir) in placements(temp.path()) {
        let ghost = make_ghost(&dir, FFI_TOML);
        let scripts = ghost.join("scripts");
        put_foo(&scripts, Which::V1); // dep.dll は置かない
        let runtime = PastaLoader::load(&ghost).unwrap();
        let missing = scripts.join("missing.dll");
        for name in [
            "foo".to_string(),
            scripts.join("foo.dll").to_str().unwrap().to_string(),
            slashed(&scripts.join("foo.dll")),
            missing.to_str().unwrap().to_string(),
            slashed(&missing),
        ] {
            let err = ffi_load_and_call(runtime.lua(), &name)
                .err()
                .unwrap_or_else(|| {
                    panic!("{label}: dep.dll が無いのに ffi.load({name:?}) が通った")
                });
            assert!(
                err.contains(&format!("cannot load module '{name}': ")),
                "{label}: {err}"
            );
            assert!(!err.contains(VERBATIM_PREFIX), "{label}: {err}");
        }
        drop(runtime);
        assert_eq!(loaded_module("foo.dll"), None, "{label}");
    }
}

/// ファイル名が ANSI で表せない DLL は、読まずに 2.8 の文言で失敗する（名前だけ・絶対パスの両方）。
#[test]
fn non_ansi_file_name_fails_with_file_name_message() {
    const FILE: &str = "foo_😀.dll";
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let scripts = ghost.join("scripts");
    put_dlls(&scripts, Which::V1);
    std::fs::rename(scripts.join("foo.dll"), scripts.join(FILE)).unwrap();
    let runtime = PastaLoader::load(&ghost).unwrap();
    let abs = scripts.join(FILE);
    for name in [
        "foo_😀".to_string(),
        abs.to_str().unwrap().to_string(),
        slashed(&abs),
    ] {
        let err = ffi_load_and_call(runtime.lua(), &name)
            .err()
            .unwrap_or_else(|| panic!("ffi.load({name:?}) が通った"));
        assert!(
            err.ends_with(&format!(
                "cannot load module '{name}': the file name '{FILE}' cannot be represented in the ANSI code page"
            )),
            "{err}"
        );
        assert_eq!(loaded_module(FILE), None);
    }
}
