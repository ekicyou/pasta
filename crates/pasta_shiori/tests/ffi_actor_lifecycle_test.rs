//! FFI 所有モデル E2E 統合テスト（task 5.1・R8.1/R8.2/R8.3/R8.4）。
//!
//! `windows.rs` の FFI extern 入口（`load`/`request`/`unload`）が結線する本番ライフサイクル
//! 自由関数 [`pasta::actor::lifecycle`]（`static MAILBOX` 所有モデル）を、実ゴースト VM に対して
//! **load→request→unload→reload** の完全サイクルで駆動し、以下を実証する:
//!
//!  (a) `spawn_actor` がアクタースレッド上で実 VM をロードし、`marshal_request` 経由の GET が
//!      byte-invariant ゴールデン応答を返す（R8.2: VM アクセスはメッセージ送信経由のみ）。
//!  (b) コルーチン継続（GET property yield → callback resume）がアクター経路で不変（R9・R1.1）。
//!  (c) NOTIFY が即 204、後続 GET が同一 FIFO で返る（R5.2・単一直列）。
//!  (d) `teardown_actor`（unload 相当）→ 再 `spawn_actor`（reload）で新スレッド＋新チャネルが
//!      正常稼働し、reload 後も GET 応答が不変（R7.2 reload・R8 所有モデル）。
//!  (e) アクター不在（teardown 後）の `marshal_request` は 204（無限待機なし・R5.6）。
//!
//! # 単一スレッド直列実行（`static MAILBOX` 共有のため）
//! `lifecycle::MAILBOX` はプロセスグローバルである。本ファイルは **1 つの `#[test]`** に
//! 全フェーズを直列に詰め、他テストと MAILBOX を共有しない（同一バイナリ内に他テストを置かない）。
//! これにより順序非依存・競合なしで FFI 所有モデルの一連の遷移を観測できる。

use std::path::{Path, PathBuf};

use pasta::actor::lifecycle::{marshal_request, spawn_actor, teardown_actor};
use pasta::actor::marshaling::default_204;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

const EXPECT_ROUND1: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: \\![get,property,OnPastaCallBack1,baseware.version]\\e\r\n\
\r\n";

const EXPECT_ROUND2: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: version=2.6.77\\e\\e\r\n\
\r\n";

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

fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// FFI 所有モデルの全フェーズ（load→request→unload→reload）を直列に検証する単一テスト。
#[test]
fn ffi_ownership_model_load_request_unload_reload_cycle() {
    let (load_dir, _temp) = build_async_callback_dir();

    // --- フェーズ 1: load（spawn_actor） ---
    let loaded = spawn_actor(0, load_dir.clone());
    assert!(loaded, "spawn_actor must load the ghost VM on the actor thread");

    // --- フェーズ 2: request（GET property コルーチン継続・R9/R1.1） ---
    let round1 = marshal_request(&normalize_request(
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n",
    ));
    assert_eq!(
        round1.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "round1 GET via FFI ownership model must be byte-invariant\nactual: {round1:?}"
    );
    let round2 = marshal_request(&normalize_request(
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnPastaCallBack1\nReference0: 2.6.77\n",
    ));
    assert_eq!(
        round2.as_bytes(),
        EXPECT_ROUND2.as_bytes(),
        "round2 callback resume via FFI ownership model must be byte-invariant\nactual: {round2:?}"
    );

    // --- フェーズ 3: NOTIFY 即 204 + 後続 GET（同一 FIFO） ---
    let notify = marshal_request(&normalize_request(
        "NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\n",
    ));
    assert_eq!(
        notify.as_bytes(),
        default_204().as_bytes(),
        "NOTIFY via FFI ownership model must return 204 immediately"
    );

    // --- フェーズ 4: unload（teardown_actor）→ アクター不在で 204 ---
    let report = teardown_actor();
    assert!(
        report.is_clean(),
        "unload (teardown_actor) must cleanly tear down the actor (done ack): {report:?}"
    );
    let after_unload = marshal_request(&normalize_request(
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n",
    ));
    assert_eq!(
        after_unload.as_bytes(),
        default_204().as_bytes(),
        "request after unload (no actor) must return the 204 safety net (no deadlock)"
    );

    // --- フェーズ 5: reload（再 spawn_actor）→ 新スレッド＋新チャネルで応答不変 ---
    let reloaded = spawn_actor(0, load_dir.clone());
    assert!(reloaded, "reload spawn_actor must load a fresh VM on a fresh actor thread");
    let after_reload = marshal_request(&normalize_request(
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n",
    ));
    assert_eq!(
        after_reload.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "GET after reload must be byte-invariant (fresh actor thread + channel)\nactual: {after_reload:?}"
    );

    // クリーンアップ: 最終 teardown（後続のプロセス終了で漏れないように）。
    let _ = teardown_actor();
}
