//! 本番アクタースレッド × mailbox 駆動 × 実 `!Send` VM コルーチン継続の統合テスト
//! （task 3.3・R4.1/R4.2/R4.3/R4.5/R9.1/R9.2/R9.3/R9.4）。
//!
//! ファイル全体を `actor-poc` feature でガードする（wintf `block_on` 依存）。feature
//! 無効時はこのテストバイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テスト
//! スイートを一切汚染しない（R7.1/R7.2）。既定出荷ビルドはバイト不変のまま。
//!
//! # 何を証明するか（task 3.3 の observable 完了条件）
//! 専用アクタースレッド上で `wintf block_on` が実 [`PastaShiori`] VM を生成・pin し、
//! 単一 `recv_async().await` ループで mailbox（task 3.2）を消費する。SHIORI（テスト）
//! スレッドから `ActorMsg::Get`/`Notify` を投入すると:
//!  (a) VM 実行スレッド ID ＝アクタースレッド ID（テスト/送信スレッドと別）— R4.2/R4.5。
//!  (b) コルーチン継続が tick をまたいで動く（GET property yield → callback resume の
//!      2 ラウンドで `co_scene` が VM 内に persist し継続）— R9.1/R9.2/R9.3/R9.4。
//!  (c) 生成された SHIORI 応答が期待バイト列に一致 — R4.1/R4.3。
//!
//! # テストが「自明に真」でないことの担保
//! Get 応答は `bounded(1)` の `recv_timeout` を有界で待つ。wake 統合が効かない／
//! コルーチンが継続しなければ、Round2 で値が反映されず（または応答が来ず）テストは
//! 失敗する。スレッド ID 比較は別スレッド実行を genuinely 観測する。

#![cfg(feature = "actor-poc")]

use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use pasta::actor::mailbox::{mailbox, ActorMsg, MailboxRequest, Reply};
use pasta::actor::thread::spawn_actor_thread;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元: `common/mod.rs`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// async_callback フィクスチャ（本番 pasta_scripts + entry.lua override）を temp へ
/// 展開し、`load_dir` を返す。`TempDir` は返り値で寿命保持する（スレッドより長命）。
/// 写経元: `byte_invariant_test.rs::load_async_callback`（ここでは VM をアクター
/// スレッドで構築するため path のみ返す）。
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

/// SHIORI/3.0 リクエストを正規化（改行→CRLF・終端付与）する。写経元:
/// `byte_invariant_test.rs::ffi_request_raw` の正規化規則。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// mailbox 経由で GET を投入し、アクタースレッドの VM 応答を有界待ちで取得する。
fn get_via_actor(tx: &flume::Sender<ActorMsg>, seq: u64, raw_request: &str) -> String {
    let (msg, reply_rx) = ActorMsg::get(MailboxRequest::new(seq, normalize_request(raw_request)));
    tx.send(msg).expect("send GET into mailbox");
    match reply_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("GET reply must arrive within timeout (actor VM must run and reply)")
    {
        Reply::Value(v) => v,
    }
}

// Round1/Round2 期待バイト列（byte_invariant_test の特性化ゴールデンと同一）。
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

/// 本番アクタースレッドが実 `!Send` VM を別スレッドで pin・駆動し、mailbox 越しの
/// GET でコルーチンが tick をまたいで継続することを実証する。
#[test]
fn actor_thread_pins_vm_and_continues_coroutine_across_messages() {
    let (load_dir, _temp) = build_async_callback_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);

    // (a) VM 実行スレッド ID がテスト/送信スレッドと別であること（R4.2/R4.5）。
    assert!(actor.loaded(), "actor thread must load the ghost VM");
    assert_ne!(
        actor.actor_thread_id(),
        thread::current().id(),
        "VM must run on a dedicated actor thread, not the SHIORI/test thread"
    );

    // (b)(c) コルーチン継続 2 ラウンド。Round1 で get_property が yield、Round2 の
    // callback でコルーチンが resume され値が反映される（co_scene が VM 内に persist）。
    let round1 = get_via_actor(
        &tx,
        1,
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n",
    );
    assert_eq!(
        round1.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "round1: get_property must yield the get-tag response (coroutine suspended)\nactual: {round1:?}"
    );

    let round2 = get_via_actor(
        &tx,
        2,
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnPastaCallBack1\nReference0: 2.6.77\n",
    );
    assert_eq!(
        round2.as_bytes(),
        EXPECT_ROUND2.as_bytes(),
        "round2: callback must resume the suspended coroutine and reflect the value\nactual: {round2:?}"
    );

    // clean stop: Stop を同一 FIFO で投入 → ループ脱出 → block_on 戻り → join 可能。
    let (stop, done_rx) = ActorMsg::stop();
    tx.send(stop).expect("send Stop into mailbox");
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("Stop must be acked (actor loop must reach Stop and exit)");
    actor.join().expect("actor thread must join cleanly after Stop");
}

/// NOTIFY fire-and-forget がアクタースレッド VM で処理され、後続 GET が同一 FIFO で
/// 返ることを実証する（NOTIFY 完了後に観測用 GET が返る・単一直列 FIFO）。
#[test]
fn actor_thread_processes_notify_then_get_in_fifo() {
    let (load_dir, _temp) = build_async_callback_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    // NOTIFY（応答経路なし・即投入）。アクターは VM 上で処理する。
    tx.send(ActorMsg::notify(MailboxRequest::new(
        1,
        normalize_request("NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\n"),
    )))
    .expect("send NOTIFY into mailbox");

    // 後続 GET は同一 FIFO で NOTIFY 処理完了後に返る。coroutine 開始イベントで
    // 200/yield 応答を取得し、VM がアクタースレッドで実際に駆動したことを確認する。
    let resp = get_via_actor(
        &tx,
        2,
        "GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n",
    );
    assert_eq!(
        resp.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "GET after NOTIFY must return the actor-VM response in FIFO order\nactual: {resp:?}"
    );

    let (stop, done_rx) = ActorMsg::stop();
    tx.send(stop).expect("send Stop into mailbox");
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("Stop must be acked");
    actor.join().expect("actor thread must join cleanly after Stop");
}
