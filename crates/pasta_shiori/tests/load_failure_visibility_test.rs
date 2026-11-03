//! ロード失敗・リクエスト処理エラーの可視化 E2E テスト（task 2.6）。
//!
//! 本番と同じリクエスト経路——`lifecycle::spawn_actor`（FFI `load` 相当）で実 VM を
//! アクタースレッドへ pin し、`lifecycle::marshal_request`（FFI `request` 相当）で
//! アクター境界を跨いで応答を得る経路——を通して、以下のケースを検証する（要件 6.3）。
//!
//!  (a) SHIORI 応答モジュール（`pasta.shiori.entry`）のロードに失敗するゴースト
//!  (b) 利用者初期化スクリプト（`main`）のロードに失敗するゴースト
//!  (c) ロードは成功するが `SHIORI.request` が複数行メッセージでエラーになるゴースト
//!  (d) 旧来の入口 `load` に、存在しないフォルダを渡す（Windows のみ・ponytail-shiori 要件 2）
//!  (e) 旧来の入口 `load` に、ANSI で表せない文字を含むパスを渡す（同上・ACP が 65001 なら飛ばす）
//!
//! 各ケースで確認するのは以下（要件 4.2 / 4.3 / 4.4 / 4.5 / 4.6 / 4.8 / 4.10 / 5.5）。
//!
//!  1. 初期化の成否: (a)(b)(d)(e) は `load` が false、(c) は true。
//!  2. GET が 500 を返し、既定 204 へ読み替えられない。
//!  3. `X-ERROR-REASON` の値に CR / LF を含まない（単一行）。
//!  4. 理由に起動モジュール名と根本原因（(c) は複数行メッセージの各行）が含まれる。
//!  5. 応答全体が `SHIORI/3.0 500 …` 行＋ `Name: value` ヘッダ列＋ `\r\n\r\n` 終端。
//!  6. NOTIFY は 204 のまま（安全網の維持）。
//!
//! # 直列実行（プロセス全域 static を共有するため）
//! `lifecycle::LEGACY` はプロセスグローバルである。本ファイルは **1 つの `#[test]`** に
//! 全ケースを直列に詰め、各ケースの終わりに `teardown_actor`（unload 相当）してから
//! 次のケースを spawn する（既存 `ffi_actor_lifecycle_test.rs` と同方式）。
//!
//! `mod common;` は `PASTA_DEBUG` を main 前に中和する `#[ctor]` を
//! 取り込むためにも必須である（要件 6.4）。`#[ignore]` は付けない（要件 6.6）。

mod common;

use common::copy_fixture_to_temp;
use pasta::actor::lifecycle::{marshal_request, spawn_actor, teardown_actor};
use pasta::actor::marshaling::default_204;
use tempfile::TempDir;

/// ベースとなる正常ゴースト（`shiori_lifecycle`）を temp へ展開し、`scripts/` 配下へ
/// 上書きスクリプトを置く。
///
/// `scripts/` は自己展開された内蔵スクリプトより優先される検索パスなので、ここへ置いた
/// ファイルが起動モジュールの実体を差し替える。
fn ghost_with_script(relative: &str, source: &str) -> TempDir {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let path = temp.path().join("scripts").join(relative);
    std::fs::create_dir_all(path.parent().expect("親ディレクトリ")).expect("ディレクトリ作成");
    std::fs::write(&path, source).expect("上書きスクリプトの書き込み");
    temp
}

/// SHIORI/3.0 リクエストを正規化（改行→CRLF・終端付与）する。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

const GET_REQUEST: &str = "GET SHIORI/3.0\nCharset: UTF-8\nID: version\nSender: SSP\n";
const NOTIFY_REQUEST: &str =
    "NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\nSender: SSP\n";

/// 500 応答の構造（ステータス行・ヘッダ列・`\r\n\r\n` 終端）を検証し、
/// `X-ERROR-REASON` の値を返す（要件 4.3 / 4.6）。
fn assert_500_structure_and_take_reason(case: &str, response: &str) -> String {
    assert_ne!(
        response.as_bytes(),
        default_204().as_bytes(),
        "[{case}] GET が既定 204 へ読み替えられてはならない（要件 4.8）\nactual: {response:?}"
    );
    assert!(
        response.starts_with("SHIORI/3.0 500 Internal Server Error\r\n"),
        "[{case}] GET は 500 応答であるべき（要件 4.3 / 4.4）\nactual: {response:?}"
    );
    assert!(
        response.ends_with("\r\n\r\n"),
        "[{case}] 応答全体は空行（\r\n\r\n）で終端するべき（要件 4.6）\nactual: {response:?}"
    );

    // ステータス行＋ヘッダ列＋終端の構造を分解して検証する。
    let body = response
        .strip_suffix("\r\n\r\n")
        .expect("終端は直前で確認済み");
    let mut lines = body.split("\r\n");
    let status = lines.next().expect("ステータス行");
    assert_eq!(status, "SHIORI/3.0 500 Internal Server Error");

    let mut reason = None;
    let mut header_count = 0usize;
    for line in lines {
        header_count += 1;
        let (name, value) = line.split_once(": ").unwrap_or_else(|| {
            panic!("[{case}] ヘッダ行は `Name: value` 形式であるべき: {line:?}\nfull: {response:?}")
        });
        assert!(
            !name.is_empty() && !name.contains(' '),
            "[{case}] ヘッダ名が不正: {line:?}"
        );
        if name == "X-ERROR-REASON" {
            reason = Some(value.to_string());
        }
    }
    assert!(
        header_count >= 2,
        "[{case}] 500 応答は Charset と X-ERROR-REASON を持つべき\nactual: {response:?}"
    );
    assert!(
        response.contains("\r\nCharset: UTF-8\r\n"),
        "[{case}] 500 応答は Charset ヘッダを持つべき\nactual: {response:?}"
    );

    let reason = reason.unwrap_or_else(|| {
        panic!(
            "[{case}] 500 応答は X-ERROR-REASON ヘッダを持つべき（要件 4.3）\nactual: {response:?}"
        )
    });
    assert!(
        !reason.is_empty(),
        "[{case}] X-ERROR-REASON は根本原因を持つべき（要件 4.3）"
    );
    assert!(
        !reason.contains('\r') && !reason.contains('\n'),
        "[{case}] X-ERROR-REASON は単一行であるべき（要件 4.6）\nactual: {reason:?}"
    );
    reason
}

/// `X-ERROR-REASON` に期待するマーカーがすべて含まれることを検証する（要件 4.5 / 5.5）。
fn assert_reason_contains(case: &str, reason: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            reason.contains(marker),
            "[{case}] X-ERROR-REASON に `{marker}` が含まれるべき（要件 4.5 / 5.5）\nactual: {reason:?}"
        );
    }
}

/// 1 ケース分を本番経路で駆動する。
///
/// 呼び手が済ませた load の戻り値 `loaded` を受け、GET → NOTIFY → `teardown_actor`
/// （unload）まで完結させ、次ケースが汚れた `LEGACY` を引き継がないようにする。
fn run_case(case: &str, loaded: bool, expect_loaded: bool, markers: &[&str]) {
    assert_eq!(
        loaded, expect_loaded,
        "[{case}] load の戻り値が期待と異なる（要件 4.2）"
    );

    let get = marshal_request(&normalize_request(GET_REQUEST));
    let reason = assert_500_structure_and_take_reason(case, &get);
    assert_reason_contains(case, &reason, markers);

    // NOTIFY は本仕様の変更後も即 204 のまま（要件 4.10）。
    let notify = marshal_request(&normalize_request(NOTIFY_REQUEST));
    assert_eq!(
        notify.as_bytes(),
        default_204().as_bytes(),
        "[{case}] NOTIFY は 204 のままであるべき（要件 4.10）\nactual: {notify:?}"
    );

    let report = teardown_actor();
    assert!(
        (report.acked && report.anomaly.is_none()) || report.already_done,
        "[{case}] unload はクリーンに終わるべき: {report:?}"
    );
}

/// (a)〜(e) を 1 本のテストで直列に検証する（プロセス全域 `LEGACY` 共有のため）。
#[test]
fn load_failures_and_request_errors_are_visible_through_the_actor_boundary() {
    // --- ケース (a): SHIORI 応答モジュールのロード失敗（要件 4.1 / 4.5 / 5.5） ---
    let entry_broken = ghost_with_script(
        "pasta/shiori/entry.lua",
        r#"error("PASTA_E2E_ENTRY_MARKER")"#,
    );
    run_case(
        "a: entry load failure",
        spawn_actor(entry_broken.path().to_path_buf()),
        false,
        &[
            // フォルダがあるときの失敗の理由の形は変えない（ponytail-shiori 要件 2.6）。
            "Load error: ",
            "failed to load startup module 'pasta.shiori.entry'",
            "PASTA_E2E_ENTRY_MARKER",
        ],
    );
    drop(entry_broken);

    // --- ケース (b): 利用者初期化スクリプトのロード失敗（要件 5.2 / 5.5） ---
    // `a b` の形は LuaJIT の構文エラーとなり、メッセージに後続トークンが出る。
    let main_broken = ghost_with_script("main.lua", "PASTA_E2E_MAIN_A PASTA_E2E_MAIN_B\n");
    run_case(
        "b: main load failure",
        spawn_actor(main_broken.path().to_path_buf()),
        false,
        &[
            "Load error: ",
            "failed to load startup module 'main'",
            "PASTA_E2E_MAIN_B",
        ],
    );
    drop(main_broken);

    // --- ケース (c): ロードは成功、リクエスト処理が複数行メッセージでエラー（要件 4.4 / 4.6） ---
    // 内蔵 `entry.lua` は `xpcall` で握って Lua 側で 500 化するため、Rust 側の `Err` 経路へ
    // 到達させるには保護なしの `SHIORI.request` で上書きする必要がある。
    let request_error = ghost_with_script(
        "pasta/shiori/entry.lua",
        // `string.char(10)` で実改行を作る（Rust/Lua 双方の escape 解釈を挟まないため）。
        r#"
SHIORI = SHIORI or {}
function SHIORI.load(hinst, load_dir) return true end
function SHIORI.request(req)
    error("PASTA_E2E_MULTILINE_FIRST" .. string.char(10) .. "PASTA_E2E_MULTILINE_SECOND")
end
function SHIORI.unload() end
return SHIORI
"#,
    );
    run_case(
        "c: multi-line request error",
        spawn_actor(request_error.path().to_path_buf()),
        true,
        &["PASTA_E2E_MULTILINE_FIRST", "PASTA_E2E_MULTILINE_SECOND"],
    );
    drop(request_error);

    // --- ケース (d)(e): 旧来の入口 `load` にフォルダの無いパスを渡す（ponytail-shiori 要件 2.1〜2.4） ---
    #[cfg(windows)]
    missing_dir::run_cases();

    // 後始末: プロセス終了でアクターが漏れないように最終 teardown。
    let _ = teardown_actor();
}

/// 旧来の入口 `load`（ANSI）にフォルダの無いパスを渡したときの 500（ponytail-shiori 要件 2.1〜2.4・9.4）。
///
/// GET の理由は `Load error: load directory not found: <入口がデコードしたパス>`。
/// ANSI で表せない文字は、入口のデコードと同じく置き換わった形で出る。
#[cfg(windows)]
mod missing_dir {
    use super::run_case;
    use crate::common::{NON_ANSI_DIR_NAME, copy_fixture_into};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;
    use windows_sys::Win32::Globalization::{
        CP_ACP, GetACP, MultiByteToWideChar, WideCharToMultiByte,
    };
    use windows_sys::Win32::System::Memory::{GMEM_FIXED, GlobalAlloc};

    /// 直した後の理由：フォルダが無いことと、入口がデコードした形のパス。
    fn expected_reason(dir: &Path) -> String {
        format!(
            "Load error: load directory not found: {}",
            ansi_round_trip(dir).display()
        )
    }

    /// パスを CP_ACP へエンコードする。表せない文字は既定文字へ置き換わり、
    /// 本番のホストが渡すバイト列と同じになる。
    fn to_ansi(dir: &Path) -> Vec<u8> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = dir.as_os_str().encode_wide().collect();
        let in_len = i32::try_from(wide.len()).expect("path length must fit i32");
        // SAFETY: 1 回目で必要長を測り、2 回目でその長さの Vec へ書く。
        unsafe {
            let call = |out: *mut u8, cap: i32| {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    wide.as_ptr(),
                    in_len,
                    out,
                    cap,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                )
            };
            let needed = call(std::ptr::null_mut(), 0);
            assert!(needed > 0, "WideCharToMultiByte length probe must succeed");
            let mut buf = vec![0u8; needed as usize];
            assert_eq!(call(buf.as_mut_ptr(), needed), needed);
            buf
        }
    }

    /// 入口（`load`）と同じく CP_ACP のバイト列をデコードし直した形のパス。
    fn ansi_round_trip(dir: &Path) -> PathBuf {
        use std::os::windows::ffi::OsStringExt;
        let ansi = to_ansi(dir);
        let in_len = i32::try_from(ansi.len()).expect("path length must fit i32");
        // SAFETY: 1 回目で必要長を測り、2 回目でその長さの Vec へ書く。
        let wide = unsafe {
            let needed =
                MultiByteToWideChar(CP_ACP, 0, ansi.as_ptr(), in_len, std::ptr::null_mut(), 0);
            assert!(needed > 0, "MultiByteToWideChar length probe must succeed");
            let mut buf = vec![0u16; needed as usize];
            assert_eq!(
                MultiByteToWideChar(CP_ACP, 0, ansi.as_ptr(), in_len, buf.as_mut_ptr(), needed),
                needed
            );
            buf
        };
        std::ffi::OsString::from_wide(&wide).into()
    }

    /// パスを CP_ACP へエンコードして出荷の `load` へ渡す（HGLOBAL は入口が解放する）。
    fn drive_ansi_load(dir: &Path) -> bool {
        let ansi = to_ansi(dir);
        // SAFETY: GMEM_FIXED のブロックへバイト列を写して渡す。
        unsafe {
            let h = GlobalAlloc(GMEM_FIXED, ansi.len());
            assert!(!h.is_null(), "GlobalAlloc must succeed");
            std::ptr::copy_nonoverlapping(ansi.as_ptr(), h as *mut u8, ansi.len());
            pasta::load(h, ansi.len() as i32) == 1
        }
    }

    fn run_missing_dir_case(case: &str, dir: &Path) {
        let reason = expected_reason(dir);
        run_case(case, drive_ansi_load(dir), false, &[reason.as_str()]);
    }

    pub(super) fn run_cases() {
        // (d) ASCII の、存在しないフォルダ。
        let root = TempDir::new().expect("create temp dir");
        run_missing_dir_case("d: missing dir", &root.path().join("no_such_ghost"));

        // (e) ゴーストは置いてあるが、パスに ANSI で表せない文字があって `load` からは届かない。
        // ACP が 65001（UTF-8）ならそのまま届いてしまうので飛ばす。
        // SAFETY: 引数なしの読み取りだけ。
        if unsafe { GetACP() } == 65001 {
            eprintln!("[e: non-ANSI missing dir] skipped: ACP is 65001");
            return;
        }
        let dir = root.path().join(NON_ANSI_DIR_NAME);
        copy_fixture_into("shiori_lifecycle", &dir);
        // 理由のパスが、表せない文字の置き換わった形であることを前提として確かめる（2.4）。
        assert!(
            ansi_round_trip(&dir).to_string_lossy().contains('?'),
            "非 ANSI の名前は CP_ACP で `?` へ置き換わるはず"
        );
        run_missing_dir_case("e: non-ANSI missing dir", &dir);
    }
}
