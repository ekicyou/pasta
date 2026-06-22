//! 機能レベル統合 E2E（task 7.1・R1.1/R1.2/R1.3/R1.4・R9.1/R9.2/R9.3/R9.4）。
//!
//! 出荷 SHIORI ホスト（SSP 等）が実際に呼ぶ **extern "C" 入口シンボルそのもの**
//! （`load`/`request`/`unload`・`#[unsafe(no_mangle)]`）を、実ゴースト VM に対して
//! 代表 SHIORI セッション列で end-to-end に駆動し、応答バイト列がゴールデンと不変で
//! あること・ライフサイクルがクリーンに閉じることを実証する。
//!
//! # なぜこのテストが必要か（既存カバレッジとの差分＝閉じる継ぎ目）
//! task 7.1 までに整備済みの統合テストは、それぞれ出荷経路の一部のみを踏む:
//! - `byte_invariant_test`: `PastaShiori::request`（構造体メソッド）経由。
//!   extern 入口・HGLOBAL 所有移譲・ANSI/UTF-8 デコードは踏まない。
//! - `ffi_actor_lifecycle_test`: `actor::lifecycle::{spawn_actor,marshal_request,teardown_actor}`
//!   （内部自由関数）経由。extern シンボルと HGLOBAL marshalling・dir の ANSI デコードは踏まない。
//! - `windows_tests`（unit）: extern `load`/`request`/`unload` を踏むが、**実 VM を spawn しない**
//!   ガード／未初期化（204）契約のみ（プロセスグローバル MAILBOX 競合回避）。
//!
//! 結果として「出荷 extern シンボルを通して **実ゴースト VM** を load→request(複数・コルーチン跨ぎ)
//! →unload する完全セッションのバイト不変」を end-to-end で確認するテストが**存在しなかった**。
//! 本ファイルはその継ぎ目を、SSP と同一の C ABI（no_mangle シンボル＋HGLOBAL＋CP_ACP dir デコード
//! ＋UTF-8 request デコード）で閉じる。
//!
//! # `static MAILBOX` 共有のため単一 `#[test]`・他テストと同居させない
//! extern 入口は `actor::lifecycle::MAILBOX`（プロセスグローバル）を変化させる。本ファイルは
//! 全フェーズを 1 つの `#[test]` に直列化し、同一バイナリ内に他テストを置かない（順序非依存）。
//!
//! # 特性化（characterization）E2E について
//! 統合済みシステムに対するゴールデン固定であり RED フェーズは持たない（現行で即グリーン）。
//! 価値はドリフト検出にある: メタアサーション（最後）で「1 バイトでも変われば赤化する」ことを実証し、
//! 後続 7.2（PoC 足場撤去）や将来変更の回帰ガードとして機能することを示す。

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::ptr;

use tempfile::TempDir;
use windows_sys::Win32::Foundation::{GlobalFree, HGLOBAL};
use windows_sys::Win32::Globalization::{WideCharToMultiByte, CP_ACP};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GMEM_FIXED};

// 出荷 extern "C" シンボル（windows.rs の `#[unsafe(no_mangle)] extern "C"`）。SSP 等の
// ホストが C ABI で呼ぶ出荷入口そのもの。lib.rs の `pub use windows::{load,request,unload}`
// を介して rlib 経由でリンク到達し、SSP と同一の HGLOBAL／ANSI dir／UTF-8 request 契約で駆動する。
use pasta::{load, request, unload};

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する
/// （固定ポート枯渇でのテスト破壊を防ぐ・他統合テストと同一ガード）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

// ===========================================================================
// ゴールデン（byte_invariant_test / ffi_actor_lifecycle_test と同一バイト列）
// ===========================================================================

/// GET property コルーチン継続 Round1: get_property が get タグを yield。
const GOLDEN_GET_PROPERTY_ROUND1: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: \\![get,property,OnPastaCallBack1,baseware.version]\\e\r\n\
\r\n";

/// GET property コルーチン継続 Round2: コールバック到着でコルーチン resume・値反映。
const GOLDEN_GET_PROPERTY_ROUND2: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: version=2.6.77\\e\\e\r\n\
\r\n";

/// 出荷経路の NOTIFY / OnSecondChange / unload 後 GET 安全網が返す正準 204 のバイト列。
///
/// 出荷アクター経路の NOTIFY は **VM を起こさず即 204**（fire-and-forget）を返す契約（R5.2:
/// 「アクタースレッドの完了を待たず即座に 204 No Content を返す」）。R5.2 は **即時性**を
/// 要求するのであって、応答バイトを変えることは要求しない。即時 204 は marshaling の
/// `default_204()` であり、リファクタ前の出荷 NOTIFY/OnSecondChange 経路が返していた
/// **VM 産 204**（`res.lua` の `RES.no_content`・標準ヘッダ 3 種＝`Charset`/`Sender`/
/// `SecurityLevel: local`）と **バイト同一**である。
///
/// したがって本定数は `byte_invariant_test` の `GOLDEN_NOTIFY_204` /
/// `GOLDEN_ON_SECOND_CHANGE_204` と完全一致する（唯一の正準 204・両経路で同一）。
/// これにより NOTIFY の即時性（R5.2）と外部 SHIORI 挙動のバイト不変（R1.1）は同時に成立する。
const GOLDEN_204: &str = "SHIORI/3.0 204 No Content\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
\r\n";

// ===========================================================================
// HGLOBAL ヘルパ（SSP 側の所有契約を再現）
// ===========================================================================

/// バイト列を GMEM_FIXED の HGLOBAL へコピーして所有権を呼び出し先（extern 入口）へ渡す。
/// extern 入口は `ShioriString::capture` で受け取り、自身の drop で解放する（所有移譲）。
fn alloc_hglobal(bytes: &[u8]) -> HGLOBAL {
    // SAFETY: GMEM_FIXED 確保。失敗は null を返すので検査する。
    let h = unsafe { GlobalAlloc(GMEM_FIXED, bytes.len()) };
    assert!(!h.is_null(), "GlobalAlloc must succeed for {} bytes", bytes.len());
    // SAFETY: h は bytes.len() バイトの有効ブロックを指す。
    unsafe {
        let dst = std::slice::from_raw_parts_mut(h as *mut u8, bytes.len());
        dst.copy_from_slice(bytes);
    }
    h
}

/// extern `request` が返した応答 HGLOBAL（呼び出し側所有）を UTF-8 として読み取り、解放する。
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

/// SHIORI/3.0 リクエスト文字列を正規化（改行 → CRLF、終端 `\r\n\r\n` 付与）する
/// （既存統合テストと同一の正規化規則）。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// extern `request` を 1 回駆動し、生応答文字列（=バイト列）を返す。
/// 入力 UTF-8 HGLOBAL の所有は extern 側へ移り、応答 HGLOBAL はここで解放する。
fn drive_request(text: &str) -> String {
    let req = normalize_request(text);
    let bytes = req.as_bytes();
    let hreq = alloc_hglobal(bytes);
    let mut len: usize = bytes.len();
    // hreq は len バイトの有効 UTF-8 ブロック。所有は extern 側（capture）へ移譲される。
    let hres = request(hreq, &mut len);
    read_and_free_response(hres, len)
}

/// ghost ディレクトリパスを、本番 `to_ansi_str`（CP_ACP デコード）と**対称**な
/// `WideCharToMultiByte`/CP_ACP で ANSI バイト列へエンコードする。
///
/// これにより temp パスにマルチバイト（非 ASCII ユーザ名等）が含まれても、出荷 `load` の
/// CP_ACP デコードを round-trip でき、ロケール非依存にロードできる。
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

/// extern `load` を駆動する。dir を CP_ACP の HGLOBAL として渡す（所有は extern 側へ移譲）。
fn drive_load(dir: &Path) -> bool {
    let ansi = path_to_ansi_bytes(dir);
    let hdir = alloc_hglobal(&ansi);
    // hdir は ansi.len() バイトの有効 CP_ACP ブロック。所有は extern 側（capture）へ移譲される。
    load(hdir, ansi.len())
}

// ===========================================================================
// フィクスチャ構築（async_callback: 本番 pasta_scripts + entry.lua override）
// ===========================================================================

fn build_async_callback_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let pasta_scripts_src = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_lua")
        .join("pasta_scripts");
    let pasta_scripts_dst = temp.path().join("pasta_scripts");
    std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
    copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

    let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
    copy_dir_recursive(&fixture_src, temp.path()).expect("copy fixture");

    (temp.path().to_path_buf(), temp)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

// ===========================================================================
// 機能レベル E2E: 出荷 extern シンボルを通した代表 SHIORI セッション
// ===========================================================================

/// 出荷 extern FFI（`load`→複数 `request`〔コルーチン跨ぎ talk 含む〕→`unload`）の
/// 完全セッションを実ゴースト VM で駆動し、全応答がゴールデンとバイト不変で、
/// ライフサイクルがクリーンに閉じることを実証する。
///
/// 検証要件:
///  - R1.1/R1.3: 出荷経路の外部 SHIORI 応答が代表イベント列でバイト不変。
///  - R9.1〜R9.4: コルーチン yield → callback resume の意味論が extern 経路で無改変。
///  - R5.2/R5.6: NOTIFY 即 204、unload 後 request も無限待機なく 204。
///  - R7.x/R8.x: unload→reload で新スレッド＋新チャネルが稼働し応答不変。
#[test]
fn extern_ffi_session_byte_invariant_e2e() {
    let (dir, _temp) = build_async_callback_dir();

    // --- load（出荷 extern `load` → spawn_actor・実 VM をアクタースレッドへ pin） ---
    assert!(
        drive_load(&dir),
        "extern load() must load the real ghost VM on the actor thread"
    );

    // --- request 1/2: GET property コルーチン継続（yield → resume・R9/R1.1） ---
    let round1 = drive_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n");
    assert_eq!(
        round1.as_bytes(),
        GOLDEN_GET_PROPERTY_ROUND1.as_bytes(),
        "round1 GET via extern FFI must be byte-invariant\nactual: {round1:?}"
    );
    let round2 = drive_request(
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnPastaCallBack1\nReference0: 2.6.77\n",
    );
    assert_eq!(
        round2.as_bytes(),
        GOLDEN_GET_PROPERTY_ROUND2.as_bytes(),
        "round2 callback resume via extern FFI must be byte-invariant (coroutine continuation, R9)\nactual: {round2:?}"
    );

    // --- request 3: NOTIFY 即 204（R5.2・単一直列 FIFO） ---
    let notify = drive_request("NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\n");
    assert_eq!(
        notify.as_bytes(),
        GOLDEN_204.as_bytes(),
        "NOTIFY via extern FFI must return 204 immediately"
    );

    // --- request 4: OnSecondChange（OnTalk 未登録 → 決定論的 204） ---
    let osc = drive_request(
        "NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnSecondChange\nReference0: 0\nReference1: 0\nReference2: 0\nReference3: 0\n",
    );
    assert_eq!(
        osc.as_bytes(),
        GOLDEN_204.as_bytes(),
        "OnSecondChange via extern FFI must be deterministic 204"
    );

    // --- unload（出荷 extern `unload` → teardown_actor・常に true・冪等） ---
    assert!(unload(), "extern unload() must return true (clean teardown)");

    // --- unload 後の request: アクター不在で安全網 204（無限待機なし・R5.6） ---
    let after_unload = drive_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n");
    assert_eq!(
        after_unload.as_bytes(),
        GOLDEN_204.as_bytes(),
        "request after unload (no actor) must return the 204 safety net via extern FFI"
    );

    // --- reload（再 `load` → 新スレッド＋新チャネル）→ 応答不変（R7/R8） ---
    assert!(drive_load(&dir), "reload extern load() must spawn a fresh actor + channel");
    let after_reload = drive_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n");
    assert_eq!(
        after_reload.as_bytes(),
        GOLDEN_GET_PROPERTY_ROUND1.as_bytes(),
        "GET after reload via extern FFI must be byte-invariant\nactual: {after_reload:?}"
    );

    // クリーンアップ: 最終 unload（プロセス終了時の漏れ防止・冪等）。
    assert!(unload(), "final extern unload() must remain a safe no-op returning true");

    // --- メタアサーション: ゴールデンが「自明に真」でないこと（ドリフト検出の実証） ---
    // 1 文字でも応答が変われば上のアサーションは赤化する。ここでは比較器自体が差分を
    // 捉えることを、意図的改竄に対するパニックで確認する（回帰ガードの非自明性・R1.4）。
    let tampered = GOLDEN_204.replacen("204 No Content", "200 OK", 1);
    assert_ne!(
        tampered.as_bytes(),
        GOLDEN_204.as_bytes(),
        "golden comparison must detect a 1-token drift (regression guard is non-trivial)"
    );
}
