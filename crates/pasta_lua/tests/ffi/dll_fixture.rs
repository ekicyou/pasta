//! FFI のテストの共通の部品（lua-cmodule-robustness 1.1）。
//!
//! - テスト用の `dep.dll` と、`dep.dll` に頼る `foo.dll`（2 版）を、プロセスで 1 回だけ、
//!   対象のアーキテクチャの `cl.exe` で作る（静的 CRT `/MT`）。人の手での用意は要らない。
//! - 全テストを 1 つの鍵で直列にする（DLL のモジュールはプロセス全体で共有されるため）。
//! - ゴーストの雛形のフォルダを作り、版を選んで DLL を写す。
//! - 読み込み済みのモジュールを名前で引く・8.3 名を得る・FFI だけを開いた素の VM を作る。
//!
//! 5.x のタスクはこのファイルを書き換えず、足りない補助は自分のテストのファイルに置く。

#![allow(dead_code)]

use std::ffi::OsStr;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::common::{NON_ANSI_DIR_NAME, copy_fixture_into, make_deep_dir};

/// FFI を開くゴーストの pasta.toml（`[loader] ffi = true`）。
pub const FFI_TOML: &str = "[loader]\ndebug_mode = true\nffi = true\n";

/// 深いフォルダの長さの下限（文字数）。`MAX_PATH`（260）を確実に超える。
pub const DEEP_MIN_LEN: usize = 300;

const DEP_C: &str = r#"
__declspec(dllexport) int dep_value(void) { return 40; }
"#;

const FOO_C: &str = r#"
__declspec(dllimport) int dep_value(void);
__declspec(dllexport) int add(int a, int b) { return a + b; }
__declspec(dllexport) int via_dep(void) { return dep_value() + 2; }
__declspec(dllexport) int which(void) { return WHICH; }
__declspec(dllexport) void release(void *p) { (void)p; }
"#;

/// `via_dep()` が返す値（`dep_value()` の 40 ＋ 2）。
pub const VIA_DEP_VALUE: i32 = 42;

/// `foo.dll` の版。`which()` が返す値で見分ける。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    V1 = 1,
    V2 = 2,
}

/// 作った DLL の置き場所。
pub struct Dlls {
    pub dep: PathBuf,
    pub foo_v1: PathBuf,
    pub foo_v2: PathBuf,
    // 置き場所はプロセスの終わりまで残す（static なので drop されない）。
    _dir: tempfile::TempDir,
}

impl Dlls {
    pub fn foo(&self, which: Which) -> &Path {
        match which {
            Which::V1 => &self.foo_v1,
            Which::V2 => &self.foo_v2,
        }
    }
}

/// FFI のテストを直列にする鍵。テストは最初にこれを取る。
pub fn lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    // 前のテストの失敗（poison）で後のテストを巻き添えにしない。
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// テスト用の DLL（プロセスで 1 回だけ作る）。
pub fn dlls() -> &'static Dlls {
    static DLLS: OnceLock<Dlls> = OnceLock::new();
    DLLS.get_or_init(build_dlls)
}

fn target_triple() -> &'static str {
    if cfg!(target_arch = "x86") {
        "i686-pc-windows-msvc"
    } else {
        "x86_64-pc-windows-msvc"
    }
}

fn cl(cwd: &Path, args: &[&str]) {
    let mut cmd: Command = cc::windows_registry::find(target_triple(), "cl.exe")
        .unwrap_or_else(|| panic!("{} の cl.exe が見つからない", target_triple()));
    let out = cmd
        .current_dir(cwd)
        .args(["/nologo", "/MT", "/LD"])
        .args(args)
        .output()
        .expect("cl.exe を起動できない");
    assert!(
        out.status.success(),
        "cl.exe {args:?} が失敗:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn build_dlls() -> Dlls {
    let dir = tempfile::Builder::new()
        .prefix("pasta_ffi_dlls")
        .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
        .expect("DLL の置き場所を作れない");
    let dep_dir = dir.path().join("dep");
    std::fs::create_dir_all(&dep_dir).unwrap();
    std::fs::write(dep_dir.join("dep.c"), DEP_C).unwrap();
    cl(&dep_dir, &["dep.c"]);

    let dep_lib = dep_dir.join("dep.lib");
    let build_foo = |which: Which| {
        let d = dir.path().join(format!("v{}", which as i32));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("foo.c"), FOO_C).unwrap();
        let define = format!("/DWHICH={}", which as i32);
        cl(&d, &[&define, "foo.c", dep_lib.to_str().unwrap()]);
        d.join("foo.dll")
    };
    let foo_v1 = build_foo(Which::V1);
    let foo_v2 = build_foo(Which::V2);
    Dlls {
        dep: dep_dir.join("dep.dll"),
        foo_v1,
        foo_v2,
        _dir: dir,
    }
}

/// `base_dir` に最小のゴーストの雛形を作り、`pasta.toml` を `pasta_toml` で書く。`base_dir` を返す。
pub fn make_ghost(base_dir: &Path, pasta_toml: &str) -> PathBuf {
    copy_fixture_into("minimal", base_dir);
    std::fs::write(base_dir.join("pasta.toml"), pasta_toml).unwrap();
    base_dir.to_path_buf()
}

/// `root` の下の、300 文字を超える深いゴーストのフォルダ（まだ雛形は作らない）。
pub fn deep_dir(root: &Path) -> PathBuf {
    make_deep_dir(root, DEEP_MIN_LEN)
}

/// `root` の下の、どの ANSI コードページでも表せない名前のフォルダ（まだ雛形は作らない）。
pub fn non_ansi_dir(root: &Path) -> PathBuf {
    root.join(NON_ANSI_DIR_NAME)
}

/// `dir` に版 `which` の `foo.dll` を写す。
pub fn put_foo(dir: &Path, which: Which) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::copy(dlls().foo(which), dir.join("foo.dll")).unwrap();
}

/// `dir` に `dep.dll` を写す。
pub fn put_dep(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::copy(&dlls().dep, dir.join("dep.dll")).unwrap();
}

/// `dir` に版 `which` の `foo.dll` と、その隣に `dep.dll` を写す。
pub fn put_dlls(dir: &Path, which: Which) {
    put_foo(dir, which);
    put_dep(dir);
}

fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}

/// `GetModuleHandleW(file_name)`。読み込み済みならそのハンドル（数値）を返す。
pub fn loaded_module(file_name: &str) -> Option<usize> {
    let name = wide(OsStr::new(file_name));
    // SAFETY: name は NUL 終端の UTF-16。戻り値の参照の数は増えない。
    let h = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(name.as_ptr()) };
    (!h.is_null()).then_some(h as usize)
}

/// `GetShortPathNameW` で得た 8.3 名のパス（`path` は存在すること）。
pub fn short_path(path: &Path) -> PathBuf {
    use windows_sys::Win32::Storage::FileSystem::GetShortPathNameW;
    let long = wide(path.as_os_str());
    // SAFETY: long は NUL 終端。バッファ無し（0）で必要な長さ（NUL 込み）を得る。
    let need = unsafe { GetShortPathNameW(long.as_ptr(), std::ptr::null_mut(), 0) };
    assert!(need > 0, "GetShortPathNameW が失敗: {}", path.display());
    let mut buf = vec![0u16; need as usize];
    // SAFETY: buf は need 文字ぶん確保してある。
    let len = unsafe { GetShortPathNameW(long.as_ptr(), buf.as_mut_ptr(), need) };
    assert!(
        len > 0 && len < need,
        "GetShortPathNameW が失敗: {}",
        path.display()
    );
    buf.truncate(len as usize);
    PathBuf::from(std::ffi::OsString::from_wide(&buf))
}

/// 長いパスの設定が無効な前提を示す環境変数（CI の LongPathCiStep が `1` にする）。
pub const LONG_PATHS_DISABLED_ENV: &str = "PASTA_TEST_LONG_PATHS_DISABLED";

/// 前提の確かめ（要件 2.5）。環境変数が `1` のとき、次の 2 つを確かめ、どちらかが崩れていれば失敗にする。
/// - プロセスの側: 接頭辞なしの 260 文字超えの `existing` へ `GetFileAttributesW` が失敗する。
///   マニフェストの無いテストの EXE ではレジストリによらず失敗するので、これだけでは CI の手順が効いた証にならない。
/// - OS の側: レジストリの `LongPathsEnabled` が無いか 0（CI の手順が効いている）。
pub fn assert_long_paths_disabled_if_required(existing: &Path) {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::Storage::FileSystem::{GetFileAttributesW, INVALID_FILE_ATTRIBUTES};
    use windows_sys::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RegGetValueW,
    };
    if std::env::var(LONG_PATHS_DISABLED_ENV).as_deref() != Ok("1") {
        return;
    }
    assert!(
        existing.exists(),
        "確かめる対象が無い: {}",
        existing.display()
    );
    let s = existing.as_os_str();
    assert!(s.len() > 260, "260 文字以下のパスでは確かめられない");
    assert!(
        !s.to_string_lossy().starts_with(r"\\"),
        "接頭辞・UNC の無いパスで確かめる"
    );
    let w = wide(s);
    // SAFETY: w は NUL 終端の UTF-16。
    let attrs = unsafe { GetFileAttributesW(w.as_ptr()) };
    assert_eq!(
        attrs, INVALID_FILE_ATTRIBUTES,
        "{LONG_PATHS_DISABLED_ENV}=1 なのに接頭辞なしの長いパスが通った（長いパスの設定が切れていない）"
    );

    let key = wide(OsStr::new(r"SYSTEM\CurrentControlSet\Control\FileSystem"));
    let name = wide(OsStr::new("LongPathsEnabled"));
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: key・name は NUL 終端の UTF-16。RRF_RT_REG_DWORD なので書き込みは value の 4 バイトに収まり、
    // size にその大きさを渡している。種類の受け取りは要らないので null。
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut value).cast(),
            &mut size,
        )
    };
    assert!(
        status == ERROR_FILE_NOT_FOUND || (status == ERROR_SUCCESS && value == 0),
        "CI の手順が効いていない（LongPathsEnabled = {value}・RegGetValueW = {status}）"
    );
}

/// FFI だけを開いた素の VM（元の `ffi.load` のまま）。比べるテストの相手役。
pub fn bare_ffi_lua() -> mlua::Lua {
    // SAFETY: FFI は unsafe の標準ライブラリなので unsafe_new_with でしか開けない。テストの中だけで使う。
    unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::FFI, mlua::LuaOptions::default()) }
}

/// `dir\foo.dll` を DLL のフォルダ＋既定のフォルダの探し方で読み、(add(1,2), via_dep(), which()) を返して解放する。
fn probe_foo(dir: &Path) -> (i32, i32, i32) {
    use windows_sys::Win32::Foundation::FreeLibrary;
    use windows_sys::Win32::System::LibraryLoader::{
        GetProcAddress, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        LoadLibraryExW,
    };
    let path = wide(dir.join("foo.dll").as_os_str());
    // SAFETY: path は NUL 終端。テスト用に作った DLL を読む。
    let h = unsafe {
        LoadLibraryExW(
            path.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        )
    };
    assert!(!h.is_null(), "foo.dll を読めない: {}", dir.display());
    // SAFETY: 名前は NUL 終端。型は FOO_C の宣言どおり（cdecl）。
    let result = unsafe {
        let add: extern "C" fn(i32, i32) -> i32 =
            std::mem::transmute(GetProcAddress(h, c"add".as_ptr().cast()).expect("add"));
        let via_dep: extern "C" fn() -> i32 =
            std::mem::transmute(GetProcAddress(h, c"via_dep".as_ptr().cast()).expect("via_dep"));
        let which: extern "C" fn() -> i32 =
            std::mem::transmute(GetProcAddress(h, c"which".as_ptr().cast()).expect("which"));
        assert!(GetProcAddress(h, c"release".as_ptr().cast()).is_some());
        (add(1, 2), via_dep(), which())
    };
    // SAFETY: 上で得た参照を 1 つ返す。
    unsafe { FreeLibrary(h) };
    result
}

#[test]
fn dlls_are_built_and_copied_into_ghost_templates() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();

    // 2 つの探索のフォルダ（scripts・scriptlibs）に版の違う DLL を置いたゴースト。
    let a = make_ghost(&temp.path().join("a"), FFI_TOML);
    put_dlls(&a.join("scripts"), Which::V1);
    put_dlls(&a.join("scriptlibs"), Which::V2);
    // 別のゴーストのフォルダ。
    let b = make_ghost(&temp.path().join("b"), FFI_TOML);
    put_dlls(&b.join("scripts"), Which::V2);
    // 深いフォルダ・ANSI で表せない名前のフォルダ。
    let deep = make_ghost(&deep_dir(&temp.path().join("deep")), FFI_TOML);
    put_dlls(&deep.join("scripts"), Which::V1);
    let non_ansi = make_ghost(&non_ansi_dir(temp.path()), FFI_TOML);
    put_dlls(&non_ansi.join("scripts"), Which::V1);

    assert!(deep.display().to_string().chars().count() > DEEP_MIN_LEN);
    for ghost in [&a, &b, &deep, &non_ansi] {
        assert!(ghost.join("pasta.toml").is_file());
        assert!(ghost.join("scripts/foo.dll").is_file());
        assert!(ghost.join("scripts/dep.dll").is_file());
    }

    // 作った DLL が使えること（版・隣の dep.dll を通る関数）。解放後は残らない。
    assert_eq!(probe_foo(&a.join("scripts")), (3, VIA_DEP_VALUE, 1));
    assert_eq!(probe_foo(&a.join("scriptlibs")), (3, VIA_DEP_VALUE, 2));
    assert_eq!(probe_foo(&b.join("scripts")), (3, VIA_DEP_VALUE, 2));
    assert_eq!(loaded_module("foo.dll"), None);
    assert_eq!(loaded_module("dep.dll"), None);

    // 素の VM は FFI だけを開く。
    let lua = bare_ffi_lua();
    let has_ffi: bool = lua.load("return type(ffi) == 'table'").eval().unwrap();
    assert!(has_ffi);

    // 8.3 名は存在するパスに対して得られる（同じフォルダを指す）。
    let short = short_path(&a);
    assert!(short.join("scripts/foo.dll").is_file());
}
