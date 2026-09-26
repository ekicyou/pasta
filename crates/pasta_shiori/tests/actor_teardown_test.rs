//! 本番 teardown（`Stop{done}` ack）の統合テスト（task 4.1・R7.1/R7.4/R7.5）。
//! reload リーク検査（R7.2/R7.3）はプロセス全体カウンタを計測するため、兄弟テストと
//! 並列に走らないよう専用バイナリ `actor_reload_leak_test.rs` に分離している。
//!
//! task 5.1 で FFI 出荷経路がアクター経路へ昇格し、wintf と
//! `windows-sys/Win32_System_Threading`（実 `GetProcessHandleCount`/`GetGuiResources` 計測）が
//! 既定ビルドの依存になったため、本テストは **既定（no-feature）ビルドで実行される**
//! （旧 `actor-poc` ガードは撤去）。
//!
//! # 何を証明するか（task 4.1 の observable 完了条件）
//!  (a) `Stop{done}` を送ると、アクターが残メッセージ drain 後に VM 破棄・cleanup を
//!      終えて `done` ack を返し、SHIORI 側が ack を受けて完了する。Stop 前に投入した
//!      メッセージは Stop より前に処理される（drain-before-stop・clean drain）。
//!  (b) ack 後の二重 teardown は安全な no-op（hang/panic なし・冪等）。
//!
//! # テストが「自明に真」でないことの担保
//! done ack は `bounded(1)` の `recv_timeout` で有界に待つ。teardown が壊れて ack を
//! 返さなければ（または drain せず break すれば）テストはハングせず **失敗** する。

use std::path::{Path, PathBuf};
use std::time::Duration;

use pasta::actor::mailbox::{ActorMsg, MailboxRequest, Reply, mailbox};
use pasta::actor::teardown::teardown_actor;
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

/// async_callback フィクスチャを temp へ展開し `load_dir` を返す。`TempDir` は
/// 返り値で寿命保持する（アクタースレッドより長命）。写経元: `actor_thread_vm_test.rs`。
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
/// `actor_thread_vm_test.rs::normalize_request`。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// R7.1/R7.4: `Stop{done}` ack による clean teardown。Stop 前に投入した GET が
/// Stop より先に処理（drain-before-stop）され、その後 done ack が有界に返る。
#[test]
fn stop_done_ack_drains_then_tears_down_cleanly() {
    let (load_dir, _temp) = build_async_callback_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    // drain-before-stop: Stop の直前に GET を投入する。GET の応答は Stop より前に
    // 返らねばならない（同一 FIFO・先行メッセージ drain 後に Stop が処理される）。
    let (get, reply_rx) = ActorMsg::get(MailboxRequest::new(
        1,
        normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n"),
    ));
    tx.send(get).expect("send GET into mailbox");

    // teardown_actor: Stop{done} を送り、有界に done ack を待ち、スレッドを detach。
    let report = teardown_actor(&tx, actor, Duration::from_secs(10));

    // 先行 GET は Stop の前に drain されて応答済みのはず（clean drain の証跡）。
    let pre_stop = reply_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("GET enqueued before Stop must be drained and replied before teardown");
    let Reply::Value(_) = pre_stop;

    assert!(
        report.acked,
        "Stop must be acked within timeout (actor must drain, drop VM, clean up, then ack)"
    );
    assert!(
        report.is_clean(),
        "teardown must complete cleanly (done ack received, no anomaly)"
    );
}

/// R7.4: 冪等 teardown。done ack 後にもう一度 `teardown_actor` 相当の Stop 送信を
/// 行っても、閉じたチャネルを already-done として安全に no-op（hang/panic なし）。
#[test]
fn second_teardown_after_done_is_safe_noop() {
    let (load_dir, _temp) = build_async_callback_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let first = teardown_actor(&tx, actor, Duration::from_secs(10));
    assert!(first.acked, "first teardown must be acked");

    // 二重 teardown: アクタースレッドは既に終了し receiver は drop 済み。Stop の
    // 再送は Disconnected になり、teardown は already-done として安全に no-op を返す
    // （ハングしないことを有界 timeout で担保。冪等）。
    let second = teardown_idempotent_resend(&tx, Duration::from_secs(2));
    assert!(
        second.already_done,
        "second teardown after done must be a safe no-op (closed channel -> already done)"
    );
    assert!(
        !second.acked,
        "no fresh ack is expected on the idempotent second teardown"
    );
}

/// 二重 teardown を直接（ActorThread を持たずに）試行するヘルパ。本番 FFI 経路では
/// teardown は冪等であるべきなので、`teardown_actor` の「すでに done」分岐に相当する
/// 動作を mailbox tx のみで再現する（receiver drop 済み → send 失敗 → already_done）。
fn teardown_idempotent_resend(
    tx: &flume::Sender<ActorMsg>,
    timeout: Duration,
) -> pasta::actor::teardown::TeardownReport {
    pasta::actor::teardown::teardown_via_sender(tx, timeout)
}
