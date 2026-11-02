//! ホストが `unload` を呼ばずに `FreeLibrary` で pasta.dll を外す場合の統合テスト（仕様 `ponytail-shiori`・
//! 要件 1.2・1.3・1.5・1.8）。
//!
//! 親の `#[test]` が、同じテストの実行ファイルを子のプロセスとして起こす（役は環境変数で指示）。子は
//! 実行ファイルの隣の `pasta.dll`（`cargo test` が cdylib として `target/<profile>/deps/` に作る）を
//! `LoadLibraryW` し、`GetProcAddress` で入口を引いて駆動し、`FreeLibrary` にかかった時間と、外した後に
//! モジュールが残っているかを標準出力に 1 行出す。親はその 1 行と子の終わり方を確かめる。
//! 子のプロセスに分けるのは、テストの実行ファイルに静的にリンクした pasta とは別の、本物の DLL の
//! 読み込みと解放をホストと同じ形で起こすため（落ちることがある場合 A を親から切り離すためでもある）。
//!
//! 今は直す前の期待（特性化テスト・要件 9.4）。直すタスク（6.2）で場合 A・B を直した後の期待に改める。

#![cfg(windows)]

mod common;

use std::ffi::{OsStr, c_long};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use common::copy_fixture_to_temp;
use shiori4_abi::{SHIORI_ABI_VERSION_3, ShioriApi};
use windows_sys::Win32::Foundation::{FreeLibrary, GlobalFree, HGLOBAL, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Memory::{GMEM_FIXED, GlobalAlloc};

/// 子の役（`A`・`B`・`C`）。無ければ子のテストは何もせずに通る。
const ROLE_ENV: &str = "PASTA_FREE_LIBRARY_ROLE";
/// 子が読むゴーストのフォルダ。
const DIR_ENV: &str = "PASTA_FREE_LIBRARY_DIR";
/// 子が標準出力に出す 1 行の頭。
const LINE_PREFIX: &str = "free_library:";

type LoadFn = unsafe extern "C" fn(HGLOBAL, c_long) -> i32;
type RequestFn = unsafe extern "C" fn(HGLOBAL, *mut c_long) -> HGLOBAL;
type UnloadFn = unsafe extern "C" fn() -> i32;
type GetApiFn = unsafe extern "C" fn(u32) -> *const ShioriApi;

fn dll_path() -> PathBuf {
    std::env::current_exe()
        .expect("current_exe")
        .with_file_name("pasta.dll")
}

fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain([0]).collect()
}

/// `name` の入口を引いて `F` の関数ポインタにする。
///
/// # Safety
/// `F` は `name` の入口の本当の型でなければならない。
unsafe fn proc<F: Copy>(module: HMODULE, name: &[u8]) -> F {
    // SAFETY: `name` は NUL 終端の ASCII。`module` は LoadLibraryW が返した生きたハンドル。
    let p = unsafe { GetProcAddress(module, name.as_ptr()) }
        .unwrap_or_else(|| panic!("GetProcAddress {:?}", String::from_utf8_lossy(name)));
    // SAFETY: 関数ポインタどうしの変換。型の正しさは呼ぶ側の契約。
    unsafe { std::mem::transmute_copy(&p) }
}

/// バイト列を HGLOBAL に載せる（所有は入口へ移る）。
fn alloc_hglobal(bytes: &[u8]) -> HGLOBAL {
    // SAFETY: GMEM_FIXED の確保。null は下で検査する。
    let h = unsafe { GlobalAlloc(GMEM_FIXED, bytes.len()) };
    assert!(!h.is_null(), "GlobalAlloc");
    // SAFETY: h は bytes.len() バイトの有効なブロック。
    unsafe { std::slice::from_raw_parts_mut(h as *mut u8, bytes.len()) }.copy_from_slice(bytes);
    h
}

/// 旧来の `loadu` に UTF-8 のフォルダを渡す。
fn drive_loadu(module: HMODULE, dir: &str) -> bool {
    // SAFETY: `loadu` の型は windows.rs の `extern "C" fn(HGLOBAL, c_long) -> i32`。
    let loadu: LoadFn = unsafe { proc(module, b"loadu\0") };
    let hdir = alloc_hglobal(dir.as_bytes());
    // SAFETY: hdir は dir.len() バイトの UTF-8。所有は入口へ移る。
    unsafe { loadu(hdir, dir.len().try_into().unwrap()) == 1 }
}

/// 旧来の `request` に GET を 1 回送り、応答を返す。
fn drive_get(module: HMODULE) -> String {
    // SAFETY: `request` の型は windows.rs の `extern "C" fn(HGLOBAL, *mut c_long) -> HGLOBAL`。
    let request: RequestFn = unsafe { proc(module, b"request\0") };
    let req = "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnBoot\r\nSender: SSP\r\n\r\n";
    let mut len: c_long = req.len().try_into().unwrap();
    // SAFETY: 要求の HGLOBAL の所有は入口へ移り、応答は len バイトで呼ぶ側が解放する。
    unsafe {
        let h = request(alloc_hglobal(req.as_bytes()), &mut len);
        assert!(!h.is_null(), "response HGLOBAL");
        let res = String::from_utf8_lossy(std::slice::from_raw_parts(
            h as *const u8,
            len.try_into().unwrap(),
        ))
        .into_owned();
        GlobalFree(h);
        res
    }
}

/// 子の役。親が環境変数で役を渡したときだけ働く。
#[test]
fn child() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let dir = std::env::var(DIR_ENV).expect("ghost dir");
    let dll = wide(dll_path().as_os_str());
    // SAFETY: NUL 終端の UTF-16 のパス。
    let module = unsafe { LoadLibraryW(dll.as_ptr()) };
    assert!(!module.is_null(), "LoadLibraryW pasta.dll");

    match role.as_str() {
        // 旧来・C モジュールあり：loadu → GET → unload を呼ばずに外す。
        "A" => {
            assert!(drive_loadu(module, &dir), "loadu");
            let res = drive_get(module);
            // 前提（設計の Open Questions 8）：ffi.load でシステムの DLL を読めること。
            assert!(res.contains("Value: ffi=true"), "ffi.load failed: {res}");
        }
        // 版 3：インスタンスを手放したまま（unload を呼ばずに）外す。
        "B" => {
            // SAFETY: `shiori_get_api` の型は v3.rs の `extern "C" fn(u32) -> *const ShioriApi`。
            let get_api: GetApiFn = unsafe { proc(module, b"shiori_get_api\0") };
            // SAFETY: テーブルは pasta.dll の static で、FreeLibrary までしか使わない。
            let api =
                unsafe { ShioriApi::from_raw(get_api(SHIORI_ABI_VERSION_3), SHIORI_ABI_VERSION_3) }
                    .expect("v3 table");
            // ShioriInstance の Drop は unload を呼ぶので、forget で手放したままにする。
            std::mem::forget(api.load_v3(&dir).expect("load_v3"));
        }
        // 釣り合い：loadu → unload → 外す。
        "C" => {
            assert!(drive_loadu(module, &dir), "loadu");
            // SAFETY: `unload` の型は windows.rs の `extern "C" fn() -> i32`。
            let unload: UnloadFn = unsafe { proc(module, b"unload\0") };
            // SAFETY: 引数なしの入口。
            assert_eq!(unsafe { unload() }, 1, "unload");
        }
        other => panic!("unknown role {other}"),
    }

    let start = Instant::now();
    // SAFETY: LoadLibraryW で得た参照 1 つを返す。以後 pasta.dll の関数は呼ばない。
    unsafe { FreeLibrary(module) };
    let elapsed = start.elapsed();
    // SAFETY: NUL 終端の UTF-16 のパス。参照の数は増えない。
    let loaded = !unsafe { GetModuleHandleW(dll.as_ptr()) }.is_null();
    println!(
        "{LINE_PREFIX} case={role} elapsed_ms={} module_loaded={loaded}",
        elapsed.as_millis()
    );
}

/// 子の結果：`FreeLibrary` にかかった時間・外した後にモジュールが残っているか・子の終わりのコード。
/// 子が 1 行を出す前に落ちたときは `elapsed`・`loaded` が `None`、子の全体の時間が `wall`。
struct ChildResult {
    elapsed: Option<Duration>,
    loaded: Option<bool>,
    code: Option<i32>,
    wall: Duration,
}

fn run_child(role: &str) -> ChildResult {
    let dll = dll_path();
    // 前提（設計の Open Questions 8）：cargo test がテストの実行ファイルの隣に pasta.dll を作ること。
    assert!(
        dll.is_file(),
        "pasta.dll not next to test exe: {}",
        dll.display()
    );
    let ghost = copy_fixture_to_temp("free_library");
    let start = Instant::now();
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child", "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, role)
        .env(DIR_ENV, ghost.path())
        .env_remove("PASTA_DEBUG")
        .env_remove("PASTA_DEBUG_PORT")
        .env_remove("PASTA_LOG")
        .output()
        .expect("spawn child");
    let wall = start.elapsed();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // libtest の `test child ... ` の後ろに続けて出るので、行の途中から探す。
    let line = stdout
        .lines()
        .find_map(|l| l.find(LINE_PREFIX).map(|i| &l[i..]));
    eprintln!(
        "case {role}: code={:?} wall_ms={} line={line:?}",
        out.status.code(),
        wall.as_millis()
    );
    let field = |key: &str| {
        line.and_then(|l| l.split_whitespace().find_map(|w| w.strip_prefix(key)))
            .map(str::to_owned)
    };
    let result = ChildResult {
        elapsed: field("elapsed_ms=").map(|v| Duration::from_millis(v.parse().unwrap())),
        loaded: field("module_loaded=").map(|v| v == "true"),
        code: out.status.code(),
        wall,
    };
    // 子が落ちてよいのは場合 A だけ（直す前）。ほかは 1 行を出して 0 で終わる。
    if role != "A" {
        assert!(
            result.code == Some(0) && line.is_some(),
            "case {role} child failed\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    result
}

/// 場合 A：旧来の入口・C モジュールあり・unload を呼ばずに外す（1.3）。
/// 直す前：C モジュールの解放がローダーのロックを待ち、FreeLibrary が後始末の上限（5 秒）まで止まる。
/// その後に生きたスレッドの下で pasta.dll が外れるので、子が落ちることは許す。
#[test]
fn case_a_legacy_with_c_module_free_without_unload() {
    let r = run_child("A");
    // 子が 1 行を出す前に落ちたときは、子の全体の時間で確かめる。
    let elapsed = r.elapsed.unwrap_or(r.wall);
    assert!(
        elapsed >= Duration::from_secs(4),
        "case A: FreeLibrary returned in {elapsed:?} (code={:?})",
        r.code
    );
}

/// 場合 B：版 3 のインスタンスを手放したまま外す（1.2・1.5）。直す前：モジュールが外れる。
#[test]
fn case_b_v3_abandoned_instance_free() {
    let r = run_child("B");
    assert_eq!(r.loaded, Some(false), "case B: module must be unloaded");
}

/// 場合 C：loadu → unload → 外す（1.8）。参照を返し忘れて DLL を残さないこと（直す前も後も同じ）。
#[test]
fn case_c_balanced_unload_then_free() {
    let r = run_child("C");
    assert_eq!(r.loaded, Some(false), "case C: module must be unloaded");
}
