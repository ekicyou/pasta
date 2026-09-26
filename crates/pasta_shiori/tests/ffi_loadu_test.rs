//! UTF-8 初期化入口（`loadu`）の FFI 検証（task 2.7・要件 2.4 / 2.5 / 2.6 / 6.4 / 6.9）。
//!
//! 出荷 SHIORI ホスト（SSP 2.6.92 以降）が呼ぶ **extern "C" 入口シンボルそのもの**
//! （`loadu`/`load`/`request`/`unload`）を C ABI で駆動し、設計
//! 「Testing Strategy > E2E Tests（pasta_shiori・本番リクエスト経路）」項 3 の系列を検証する。
//!
//!  1. 非 ANSI 名（[`common::NON_ANSI_DIR_NAME`]）のディレクトリへ設置したゴーストの
//!     パスを UTF-8 で `loadu` へ渡し、ロードが成功する（要件 2.4 / 6.9）。
//!  2. `request` の GET が 500 でも既定 204 でもない正常応答を返す。
//!  3. `unload` で終了処理。
//!  4. 再度 `loadu` で初期化したあと従来入口 `load` を呼ぶと、ANSI では表現できず欠落した
//!     パスを渡しても TRUE が返り、直後の GET が同一の正常応答を返す＝ロード済み状態が
//!     壊れない（要件 2.5）。
//!  5. `unload` 後に `load` を単独で呼ぶと、従来どおり ANSI の設置パスでロードする（要件 2.6）。
//!
//! # プロセス全域 static を共有するため 1 本のテスト関数で直列化する
//! `actor::lifecycle::MAILBOX` と `windows.rs` の `loadu` 初期化フラグはどちらもプロセス
//! グローバルである。本ファイルは全段を 1 つの `#[test]` に直列に詰め、同一バイナリ内に
//! 他のテストを置かない（順序非依存）。
//!
//! `mod common;` は `PASTA_DEBUG` / `PASTA_DEBUG_PORT` を main 前に中和する `#[ctor]` を
//! 取り込むためにも必須である（要件 6.4）。`#[ignore]` は付けない（要件 6.6）。

#![cfg(windows)]

mod common;

use std::path::{Path, PathBuf};
use std::ptr;

use common::{NON_ANSI_DIR_NAME, copy_fixture_into};
use tempfile::TempDir;
use windows_sys::Win32::Foundation::{GlobalFree, HGLOBAL};
use windows_sys::Win32::Globalization::{CP_ACP, WideCharToMultiByte};
use windows_sys::Win32::System::Memory::{GMEM_FIXED, GlobalAlloc};

// 出荷 extern "C" シンボル。`load`/`request`/`unload` は lib.rs の再エクスポート経由、
// `loadu` は `#[unsafe(no_mangle)]` されたシンボルを C ABI でそのまま宣言して呼ぶ
// （ホストから見える名前と呼び出し規約で到達することを同時に確かめる）。
use pasta::{load, request, unload};

unsafe extern "C" {
    /// SHIORI DLL 共通仕様の `loadu`（設置パスを UTF-8 で受け取る初期化入口・要件 2.4）。
    fn loadu(hdir: HGLOBAL, len: usize) -> bool;
}

/// 正常応答のゴールデン。`shiori_lifecycle` フィクスチャの `entry.lua` が
/// シーン `テスト挨拶`（`dic/test/lifecycle.pasta`）を解決して返す 200 応答。
///
/// シーンモジュールの `require` が失敗すると `entry.lua` は 500 を返すため、この
/// バイト列の一致はモジュール解決まで含めた正常動作の証明になる。
const GOLDEN_OK: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Value: ライフサイクルテスト成功！\r\n\
\r\n";

const GET_REQUEST: &str = "GET SHIORI/3.0\nCharset: UTF-8\nID: テスト挨拶\nSender: SSP\n";

// ===========================================================================
// HGLOBAL ヘルパ（ホスト側の所有契約を再現）
// ===========================================================================

/// バイト列を HGLOBAL へ載せて extern 入口へ所有権を渡す（入口側が解放する）。
fn alloc_hglobal(bytes: &[u8]) -> HGLOBAL {
    // SAFETY: GMEM_FIXED 確保。失敗は null を返すので検査する。
    let h = unsafe { GlobalAlloc(GMEM_FIXED, bytes.len()) };
    assert!(
        !h.is_null(),
        "GlobalAlloc must succeed for {} bytes",
        bytes.len()
    );
    // SAFETY: h は bytes.len() バイトの有効ブロックを指す。
    unsafe {
        let dst = std::slice::from_raw_parts_mut(h as *mut u8, bytes.len());
        dst.copy_from_slice(bytes);
    }
    h
}

/// `request` が返した応答 HGLOBAL（呼び出し側所有）を UTF-8 として読み、解放する。
fn read_and_free_response(h: HGLOBAL, len: usize) -> String {
    assert!(!h.is_null(), "response HGLOBAL must not be null");
    // SAFETY: 応答は h が len バイトの有効ブロックを指す（emit_response_into の契約）。
    let s = unsafe {
        let bytes = std::slice::from_raw_parts(h as *const u8, len);
        String::from_utf8(bytes.to_vec()).expect("response must be valid UTF-8")
    };
    // SAFETY: 応答 HGLOBAL は呼び出し側（このテスト）が解放する契約（nofree で渡される）。
    unsafe {
        GlobalFree(h);
    }
    s
}

/// SHIORI/3.0 リクエストを正規化（改行 → CRLF・終端付与）する。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// extern `request` を 1 回駆動して生応答（＝バイト列）を返す。
fn drive_request() -> String {
    let req = normalize_request(GET_REQUEST);
    let bytes = req.as_bytes();
    let hreq = alloc_hglobal(bytes);
    let mut len: usize = bytes.len();
    let hres = request(hreq, &mut len);
    read_and_free_response(hres, len)
}

/// 設置パスを UTF-8 バイト列として `loadu` へ渡す（要件 2.4）。
fn drive_loadu(dir: &Path) -> bool {
    let utf8 = dir
        .to_str()
        .expect("設置パスは UTF-8 で表現できるべき")
        .as_bytes()
        .to_vec();
    let hdir = alloc_hglobal(&utf8);
    // SAFETY: hdir は utf8.len() バイトの有効 UTF-8 ブロック。所有は入口側へ移譲される。
    unsafe { loadu(hdir, utf8.len()) }
}

/// 設置パスをシステム ANSI（CP_ACP）へ変換して従来入口 `load` へ渡す。
///
/// 非 ANSI 名のディレクトリでは表現できない文字が既定文字へ置換され、パスは欠落する。
/// それが「従来入口では非 ANSI の設置パスが届かない」という本仕様の前提そのものである。
fn drive_load(dir: &Path) -> bool {
    let ansi = path_to_ansi_bytes(dir);
    let hdir = alloc_hglobal(&ansi);
    load(hdir, ansi.len())
}

/// パスを本番 `to_ansi_str`（CP_ACP デコード）と対称な `WideCharToMultiByte`/CP_ACP で
/// ANSI バイト列へエンコードする。
fn path_to_ansi_bytes(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    assert!(!wide.is_empty(), "path must not be empty");
    let in_len = i32::try_from(wide.len()).expect("path length must fit i32");
    // SAFETY: 2 段呼び出し。1 回目は必要長取得（出力 null）、2 回目で確保済みバッファを満たす。
    unsafe {
        let needed = WideCharToMultiByte(
            CP_ACP,
            0,
            wide.as_ptr(),
            in_len,
            ptr::null_mut(),
            0,
            ptr::null(),
            ptr::null_mut(),
        );
        assert!(needed > 0, "WideCharToMultiByte length probe must succeed");
        let mut buf = vec![0u8; needed as usize];
        let written = WideCharToMultiByte(
            CP_ACP,
            0,
            wide.as_ptr(),
            in_len,
            buf.as_mut_ptr() as _,
            needed,
            ptr::null(),
            ptr::null_mut(),
        );
        assert_eq!(written, needed, "WideCharToMultiByte must fill the buffer");
        buf
    }
}

// ===========================================================================
// フィクスチャ構築
// ===========================================================================

/// `root` 直下の `name` ディレクトリへ `shiori_lifecycle` ゴーストを設置してパスを返す。
fn install_ghost(root: &TempDir, name: &str) -> PathBuf {
    let dir = root.path().join(name);
    copy_fixture_into("shiori_lifecycle", &dir);
    dir
}

// ===========================================================================
// 入口ログの検証（load / loadu の判別）
// ===========================================================================

/// 設置ディレクトリ配下の `*.log` を再帰的に集めて連結する。
///
/// ログの出力先は `[logging] file_path` で変わりうるため、固定パス決め打ちではなく
/// 設置ディレクトリ配下を走査する。
fn read_ghost_logs(dir: &Path) -> (String, Vec<PathBuf>) {
    fn walk(dir: &Path, out: &mut String, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out, found);
            } else if path.extension().is_some_and(|e| e == "log") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push_str(&text);
                }
                found.push(path);
            }
        }
    }
    let mut out = String::new();
    let mut found = Vec::new();
    walk(dir, &mut out, &mut found);
    (out, found)
}

/// 入口ログ 1 行が、期待した入口名でゴーストのログファイルへ届いていることを表明する。
///
/// この検証は 2 つの回帰を同時に押さえる。
///
/// 1. **出力時点**: 購読者登録前に出すと捨てられる（登録は `spawn_actor` の内側）。
/// 2. **スレッド文脈**: ログのファイル振り分けは load_dir のスレッドローカルで決まる。
///    FFI 入口スレッドで `LoadDirGuard` を張り直さないと、行は出ても振り分け先が無く
///    ゴーストのログファイルへは届かない（実機 SSP 確認で実際に踏んだ）。
///
/// `entry=load` は `entry=loadu` の部分文字列なので、後続フィールドまで含めて照合する
/// （区切りを含めない部分一致は `load` の表明が `loadu` でも通ってしまい空虚になる）。
fn assert_entry_logged(dir: &Path, expected_entry: &str) {
    let (logs, files) = read_ghost_logs(dir);
    // `entry` は `&str` のため tracing の既定フィールド整形ではクォート付きで出る
    // （`entry="loadu"`）。後続フィールドまで含めて照合するのは、区切りを含めない
    // 部分一致だと `load` の表明が `loadu` でも通ってしまい空虚になるため。
    let needle = format!("entry=\"{expected_entry}\" loaded=");
    assert!(
        logs.contains("SHIORI load entry completed"),
        "入口ログ行がゴーストのログファイルへ届くべき（{}）\n見つかった .log: {files:#?}\nlogs:\n{logs}",
        dir.display()
    );
    assert!(
        logs.contains(&needle),
        "入口ログは {needle:?} を含むべき（{}）\n見つかった .log: {files:#?}\nlogs:\n{logs}",
        dir.display()
    );
}

// ===========================================================================
// 設計 Testing Strategy > E2E Tests 項 3 の系列（1 本の直列テスト）
// ===========================================================================

#[test]
fn loadu_entry_drives_a_non_ansi_install_path_end_to_end() {
    let non_ansi_root = TempDir::new().expect("create temp dir");
    let non_ansi_dir = install_ghost(&non_ansi_root, NON_ANSI_DIR_NAME);

    // --- 段 1: 非 ANSI の設置パスを UTF-8 で渡す初期化（要件 2.4 / 6.9） ---
    assert!(
        drive_loadu(&non_ansi_dir),
        "loadu は非 ANSI の設置パスでもロードを完了するべき（要件 2.4 / 6.9）: {}",
        non_ansi_dir.display()
    );

    // --- 段 2: GET が正常応答（500 でも既定 204 でもない） ---
    let get = drive_request();
    assert_eq!(
        get.as_bytes(),
        GOLDEN_OK.as_bytes(),
        "loadu 後の GET は正常応答を返すべき（要件 2.4）\nactual: {get:?}"
    );

    // --- 段 3: 終了処理 ---
    assert!(unload(), "unload は常に true を返す");

    // 段 1 の初期化が `loadu` 経由だったことがログから判別できる。
    // （非同期 writer のため、ロガーが解放される unload の後に読む）
    assert_entry_logged(&non_ansi_dir, "loadu");

    // --- 段 4: loadu で初期化したあとの従来入口 load は状態を壊さない（要件 2.5） ---
    assert!(
        drive_loadu(&non_ansi_dir),
        "2 回目の loadu もロードを完了するべき（要件 2.4）"
    );
    assert!(
        drive_load(&non_ansi_dir),
        "loadu 済みの load は成功を返すべき（要件 2.5）"
    );
    let after_load = drive_request();
    assert_eq!(
        after_load.as_bytes(),
        GOLDEN_OK.as_bytes(),
        "loadu 済みの load はロード済み状態を壊してはならない（要件 2.5）\nactual: {after_load:?}"
    );

    // --- 段 5: unload 後の load 単独は従来どおりロードする（要件 2.6） ---
    assert!(unload(), "unload は常に true を返す");
    let ascii_root = TempDir::new().expect("create temp dir");
    let ascii_dir = install_ghost(&ascii_root, "ascii_install_path");
    assert!(
        drive_load(&ascii_dir),
        "unload でフラグが下りたあとの load は従来どおり ANSI の設置パスでロードするべき（要件 2.6）"
    );
    let after_legacy_load = drive_request();
    assert_eq!(
        after_legacy_load.as_bytes(),
        GOLDEN_OK.as_bytes(),
        "従来入口 load 単独でも GET は正常応答を返すべき（要件 2.6）\nactual: {after_legacy_load:?}"
    );

    // 後始末: プロセス終了でアクターが漏れないように最終 unload（冪等）。
    assert!(
        unload(),
        "final unload must remain a safe no-op returning true"
    );

    // 段 5 の初期化が従来入口 `load` 経由だったことがログから判別できる
    // （＝入口ログが 2 つの入口を取り違えていない）。
    assert_entry_logged(&ascii_dir, "load");
}
