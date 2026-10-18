//! 深いパス・ANSI に無い名前・依存 DLL・探索のフォルダ・名前の解き方の確かめ（要件 2.x・3.x・4.x）。

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use pasta_lua::PastaLoader;

use crate::dll_fixture::{
    DEEP_MIN_LEN, FFI_TOML, VIA_DEP_VALUE, Which, assert_long_paths_disabled_if_required, deep_dir,
    loaded_module, lock, make_ghost, non_ansi_dir, put_dep, put_dlls, put_foo, short_path,
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
            // 2.7（文言に接頭辞が出ない）は失敗の文言で確かめる（下のテスト）。
            let (sum, dep, _) = ffi_load_and_call(runtime.lua(), &name)
                .unwrap_or_else(|e| panic!("{label}: ffi.load({name:?}) が失敗: {e}"));
            assert_eq!((sum, dep), (3, VIA_DEP_VALUE), "{label}: {name}");
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

// ---- 名前の解決と依存 DLL（5.3・要件 3.x・4.x・9.3・9.4） ----

/// `ERROR_MOD_NOT_FOUND` の Windows の理由（`FormatMessageW` の UTF-16 を UTF-8 にしたもの）。
fn mod_not_found_message() -> String {
    use windows_sys::Win32::Foundation::ERROR_MOD_NOT_FOUND;
    use windows_sys::Win32::System::Diagnostics::Debug::{
        FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageW,
    };
    let mut buf = [0u16; 512];
    // SAFETY: buf は書き込める buf.len() 個の u16。IGNORE_INSERTS なので引数の並びは読まれない。
    let len = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            std::ptr::null(),
            ERROR_MOD_NOT_FOUND,
            0,
            buf.as_mut_ptr(),
            buf.len() as u32,
            std::ptr::null(),
        )
    };
    assert!(len > 0, "FormatMessageW が失敗");
    String::from_utf16_lossy(&buf[..len as usize])
}

/// ラッパの上位値 `orig`（元の `ffi.load`）を、呼ばれた引数を `PASTA_ORIG_CALLS` に記録する関数に差し替える。
/// 上位値に触るため、テストの VM にだけ `debug` のライブラリを足す。
fn spy_original_load(lua: &mlua::Lua) {
    lua.load_std_libs(mlua::StdLib::DEBUG).unwrap();
    lua.load(
        r##"
        PASTA_ORIG_CALLS = {}
        for i = 1, math.huge do
          local n, orig = debug.getupvalue(ffi.load, i)
          assert(n, "ラッパの上位値に orig が無い")
          if n == "orig" then
            debug.setupvalue(ffi.load, i, function(...)
              PASTA_ORIG_CALLS[#PASTA_ORIG_CALLS + 1] =
                { n = select("#", ...), name = tostring((...)), second = tostring((select(2, ...))) }
              return orig(...)
            end)
            return
          end
        end
        "##,
    )
    .exec()
    .unwrap();
}

/// 元の `ffi.load` が受け取った引数（数・1 番目・2 番目の tostring）。
fn original_calls(lua: &mlua::Lua) -> Vec<(usize, String, String)> {
    let calls: mlua::Table = lua.globals().get("PASTA_ORIG_CALLS").unwrap();
    calls
        .sequence_values::<mlua::Table>()
        .map(|c| {
            let c = c.unwrap();
            (
                c.get("n").unwrap(),
                c.get("name").unwrap(),
                c.get("second").unwrap(),
            )
        })
        .collect()
}

/// `ffi.load(NAME[, SECOND])` で読んだ `foo` の `which()`（版）。`add`・`via_dep` も確かめる。
/// 失敗すれば `pcall` が受け取った文言を返す。`second` は Lua の式（`None` なら 1 引数で呼ぶ）。
fn load_which(lua: &mlua::Lua, name: &str, second: Option<&str>) -> Result<i32, String> {
    lua.globals().set("NAME", name).unwrap();
    let call = match second {
        Some(expr) => format!("pcall(ffi.load, NAME, {expr})"),
        None => "pcall(ffi.load, NAME)".to_string(),
    };
    let (ok, v): (bool, mlua::Value) = lua
        .load(format!(
            r#"
            if not PASTA_TEST_CDEF_WHICH then
              ffi.cdef [[ int add(int a, int b); int via_dep(void); int which(void); ]]
              PASTA_TEST_CDEF_WHICH = true
            end
            local ok, foo = {call}
            if not ok then return false, tostring(foo) end
            assert(foo.add(1, 2) == 3 and foo.via_dep() == {VIA_DEP_VALUE})
            return true, foo.which()
            "#
        ))
        .eval()
        .unwrap();
    if ok {
        Ok(v.as_i32().expect("which の結果が数でない"))
    } else {
        Err(v.as_string().unwrap().to_string_lossy())
    }
}

/// カレントのフォルダと `PATH` を一時的に変え、落ちるときに戻す（FFI の鍵を持った中で使う）。
struct CwdAndPath {
    cwd: PathBuf,
    path: Option<std::ffi::OsString>,
}

impl CwdAndPath {
    /// カレントのフォルダを `dir` にし、`PATH` の先頭に `dir` を足す。
    fn point_to(dir: &Path) -> Self {
        let saved = Self {
            cwd: std::env::current_dir().unwrap(),
            path: std::env::var_os("PATH"),
        };
        let mut paths = vec![dir.to_path_buf()];
        if let Some(p) = &saved.path {
            paths.extend(std::env::split_paths(p));
        }
        std::env::set_current_dir(dir).unwrap();
        // SAFETY: FFI のテストは鍵で直列になっており、この間に環境変数を読み書きするほかのスレッドは無い。
        unsafe { std::env::set_var("PATH", std::env::join_paths(paths).unwrap()) };
        saved
    }
}

impl Drop for CwdAndPath {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.cwd);
        // SAFETY: point_to と同じ。
        unsafe {
            match &self.path {
                Some(p) => std::env::set_var("PATH", p),
                None => std::env::remove_var("PATH"),
            }
        }
    }
}

/// 隣の `dep.dll` を通る関数が呼べる（4.1・9.3）。`foo.dll` は静的 CRT で `KERNEL32.dll` に頼るので、
/// システムの DLL の依存も解けている（4.2）。`dep.dll` が隣に無ければ、カレントのフォルダと `PATH` に
/// あっても使わず（4.3）、`cannot load module 'foo': <理由>` で失敗し、プロセスは落ちない（4.5）。
#[test]
fn neighbour_dep_resolves_and_missing_dep_fails_without_crash() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let scripts = ghost.join("scripts");
    put_foo(&scripts, Which::V1);
    let elsewhere = temp.path().join("elsewhere");
    put_dep(&elsewhere);
    let runtime = PastaLoader::load(&ghost).unwrap();

    {
        let _env = CwdAndPath::point_to(&elsewhere);
        let err = ffi_load_and_call(runtime.lua(), "foo")
            .expect_err("隣に dep.dll が無いのに ffi.load(\"foo\") が通った");
        assert!(
            err.ends_with(&format!(
                "cannot load module 'foo': {}",
                mod_not_found_message()
            )),
            "{err}"
        );
        assert_eq!(loaded_module("foo.dll"), None);
        assert_eq!(loaded_module("dep.dll"), None);
    }

    // 失敗の後も同じ VM で、隣に dep.dll を置けば読める。
    put_dep(&scripts);
    let (sum, dep, _) = ffi_load_and_call(runtime.lua(), "foo").unwrap();
    assert_eq!((sum, dep), (3, VIA_DEP_VALUE));
    drop(runtime);
    assert_eq!(loaded_module("foo.dll"), None);
    assert_eq!(loaded_module("dep.dll"), None);
}

/// 名前だけの `ffi.load` は `.dll` を足し（3.1）、探索のフォルダを順に探して最初のものを読む（3.2）。
/// `lua_search_paths` を変えると、`ffi.load` も `require` も同じ順で探す（3.3・9.4）。
#[test]
fn name_only_follows_lua_search_paths_order() {
    let _guard = lock();
    let swapped = "[loader]\ndebug_mode = true\nffi = true\nlua_search_paths = [\
        \"profile/pasta/save/lua\", \"scriptlibs\", \"scripts\", \
        \"profile/pasta/pasta_scripts\", \"profile/pasta/cache/lua\"]\n";
    let temp = tempfile::TempDir::new().unwrap();
    for (i, (toml, expected)) in [(FFI_TOML, 1), (swapped, 2)].into_iter().enumerate() {
        let ghost = make_ghost(&temp.path().join(format!("g{i}")), toml);
        for (dir, which) in [("scripts", Which::V1), ("scriptlibs", Which::V2)] {
            put_dlls(&ghost.join(dir), which);
            std::fs::write(
                ghost.join(dir).join("pasta_test_which.lua"),
                format!("return {}", which as i32),
            )
            .unwrap();
        }
        let runtime = PastaLoader::load(&ghost).unwrap();
        let lua = runtime.lua();
        let required: i32 = lua
            .load(r#"return require "pasta_test_which""#)
            .eval()
            .unwrap();
        assert_eq!(required, expected, "require: {toml}");
        assert_eq!(
            load_which(lua, "foo", None),
            Ok(expected),
            "ffi.load: {toml}"
        );
        assert_eq!(load_which(lua, "foo.dll", None), Ok(expected), "{toml}");
        drop(runtime);
        assert_eq!(loaded_module("foo.dll"), None);
    }
}

/// 探索のフォルダに無い名前だけの `ffi.load("kernel32")` は、`kernel32.dll` として元の `ffi.load` に
/// 渡り、システムの関数が呼べる（3.4・9.4）。2 番目の引数を渡しても同じ（3.8）。
#[test]
fn name_only_falls_back_to_system_dll() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let runtime = PastaLoader::load(&ghost).unwrap();
    let lua = runtime.lua();
    spy_original_load(lua);
    let pids: (u32, u32) = lua
        .load(
            r#"
            ffi.cdef [[ unsigned long GetCurrentProcessId(void); ]]
            return tonumber(ffi.load("kernel32").GetCurrentProcessId()),
                   tonumber(ffi.load("kernel32", true).GetCurrentProcessId())
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!(pids, (std::process::id(), std::process::id()));
    assert_eq!(
        original_calls(lua),
        vec![
            (1, "kernel32.dll".to_string(), "nil".to_string()),
            (2, "kernel32.dll".to_string(), "true".to_string()),
        ]
    );
}

/// 区切りのある相対の名前は、カレントのフォルダでなくゴーストの基点から解く（3.5）。絶対パスはそのまま（3.6）。
/// 2 番目の引数を渡しても結果は変わらず、そのまま元の `ffi.load` に渡る（3.8）。
#[test]
fn relative_names_resolve_from_ghost_base() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    put_dlls(&ghost.join("lib"), Which::V2);
    // 名前だけの探索・カレントのフォルダには別の版を置く（そちらを読んだら版で分かる）。
    put_dlls(&ghost.join("scripts"), Which::V1);
    let cwd = temp.path().join("cwd");
    put_dlls(&cwd.join("lib"), Which::V1);
    let runtime = PastaLoader::load(&ghost).unwrap();
    let lua = runtime.lua();
    spy_original_load(lua);
    let abs = ghost.join("lib").join("foo.dll");
    {
        let _env = CwdAndPath::point_to(&cwd);
        for name in ["lib/foo", r"lib\foo.dll", abs.to_str().unwrap()] {
            assert_eq!(load_which(lua, name, None), Ok(2), "{name}");
            for second in ["true", "false", "nil"] {
                assert_eq!(
                    load_which(lua, name, Some(second)),
                    Ok(2),
                    "{name}, {second}"
                );
            }
        }
    }
    let calls = original_calls(lua);
    assert_eq!(calls.len(), 12);
    for (i, (n, name, second)) in calls.into_iter().enumerate() {
        assert_eq!(name, "foo.dll", "元へはファイル名で渡る");
        let expected = ["nil", "true", "false", "nil"][i % 4];
        assert_eq!(
            (n, second.as_str()),
            (1 + usize::from(i % 4 != 0), expected)
        );
    }
    drop(runtime);
    assert_eq!(loaded_module("foo.dll"), None);
}

/// 区切りのある名前（相対・絶対）のファイルが無ければ、元の `ffi.load` へ渡さず、
/// 「モジュールが見つからない」の Windows の理由で失敗する（3.7）。名前だけの探索に同名があっても読まない。
#[test]
fn missing_path_name_fails_without_original() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    put_dlls(&ghost.join("scripts"), Which::V1);
    let runtime = PastaLoader::load(&ghost).unwrap();
    let lua = runtime.lua();
    spy_original_load(lua);
    let abs = ghost.join("lib").join("foo.dll");
    let reason = mod_not_found_message();
    for name in [
        "lib/foo".to_string(),
        r"lib\foo.dll".to_string(),
        "lib/".to_string(),
        abs.to_str().unwrap().to_string(),
        slashed(&abs),
    ] {
        let err = load_which(lua, &name, Some("true"))
            .err()
            .unwrap_or_else(|| panic!("ffi.load({name:?}) が通った"));
        assert!(
            err.ends_with(&format!("cannot load module '{name}': {reason}")),
            "{err}"
        );
    }
    assert_eq!(original_calls(lua), vec![]);
    assert_eq!(loaded_module("foo.dll"), None);
}

/// `AddDllDirectory` で既定の探索（`LOAD_LIBRARY_SEARCH_DEFAULT_DIRS`）にフォルダを足し、落ちるときに外す
/// （FFI の鍵を持った中で使う）。
struct DllDirectory(*mut std::ffi::c_void);

impl DllDirectory {
    fn add(dir: &Path) -> Self {
        use windows_sys::Win32::System::LibraryLoader::AddDllDirectory;
        let w: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: w は NUL 終端の UTF-16 の絶対パス。
        let cookie = unsafe { AddDllDirectory(w.as_ptr()) };
        assert!(
            !cookie.is_null(),
            "AddDllDirectory が失敗: {}",
            dir.display()
        );
        Self(cookie)
    }
}

impl Drop for DllDirectory {
    fn drop(&mut self) {
        // SAFETY: add で得た cookie を 1 回だけ返す。
        unsafe { windows_sys::Win32::System::LibraryLoader::RemoveDllDirectory(self.0) };
    }
}

/// 読み込み済みの `file_name` のモジュールのパス（`GetModuleFileNameW`）。
fn loaded_module_path(file_name: &str) -> String {
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
    let h = loaded_module(file_name).unwrap_or_else(|| panic!("{file_name} が読み込まれていない"));
    let mut buf = vec![0u16; 32768];
    // SAFETY: h は読み込み済みのモジュール。buf は buf.len() 個の u16。
    let len = unsafe { GetModuleFileNameW(h as _, buf.as_mut_ptr(), buf.len() as u32) };
    assert!(len > 0, "GetModuleFileNameW が失敗");
    String::from_utf16_lossy(&buf[..len as usize])
}

/// 依存 DLL がどのフォルダから読まれたか。`\\?\` の有無・8.3 名・大文字小文字に左右されないよう、
/// 末尾の `<フォルダ名>\dep.dll` だけで見分ける。
fn dep_loaded_from(folder: &str) -> bool {
    loaded_module_path("dep.dll")
        .to_lowercase()
        .ends_with(&format!(r"\{folder}\dep.dll"))
}

/// 同じ依存 DLL が DLL のフォルダにも既定の探索のフォルダ（`AddDllDirectory`）にもあれば、
/// DLL のフォルダのものを使う（4.4）。隣に無ければ既定の探索のフォルダのものが使われる（足したフォルダが効いている確かめ）。
#[test]
fn neighbour_dep_wins_over_default_search_dirs() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let scripts = ghost.join("scripts");
    put_foo(&scripts, Which::V1);
    let other = std::path::absolute(temp.path().join("other")).unwrap();
    put_dep(&other);
    let _dir = DllDirectory::add(&other);

    // 隣に無ければ、足したフォルダの dep.dll で読める。
    let runtime = PastaLoader::load(&ghost).unwrap();
    let (sum, dep, _) = ffi_load_and_call(runtime.lua(), "foo").unwrap();
    assert_eq!((sum, dep), (3, VIA_DEP_VALUE));
    assert!(
        dep_loaded_from("other"),
        "{}",
        loaded_module_path("dep.dll")
    );
    drop(runtime);
    assert_eq!(loaded_module("foo.dll"), None);
    assert_eq!(loaded_module("dep.dll"), None);

    // 隣にもあれば、隣のものを使う。
    put_dep(&scripts);
    let runtime = PastaLoader::load(&ghost).unwrap();
    let (sum, dep, _) = ffi_load_and_call(runtime.lua(), "foo").unwrap();
    assert_eq!((sum, dep), (3, VIA_DEP_VALUE));
    assert!(
        dep_loaded_from("scripts"),
        "{}",
        loaded_module_path("dep.dll")
    );
    drop(runtime);
    assert_eq!(loaded_module("foo.dll"), None);
    assert_eq!(loaded_module("dep.dll"), None);
}
