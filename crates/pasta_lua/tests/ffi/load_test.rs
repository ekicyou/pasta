//! 深いパス・ANSI に無い名前・依存 DLL・探索のフォルダ・名前の解き方の確かめ（要件 2.x・3.x・4.x）。

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::dll_fixture::{
    DEEP_MIN_LEN, VIA_DEP_VALUE, Which, assert_long_paths_disabled_if_required, deep_dir,
    loaded_module, lock, put_dlls, short_path,
};

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
