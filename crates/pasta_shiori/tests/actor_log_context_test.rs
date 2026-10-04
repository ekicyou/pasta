//! アクタースレッドの観測ログが、ロガーの数によらず自分のゴーストのログファイルへ届くことの検証
//! （仕様 `pasta-toml-logging-consistency` task 5.2・requirements 4.2・design.md `ActorLogContext`）。
//!
//! 2 つのゴーストのアクタースレッドを同時に起こし、ロガーを 2 つ登録した状態にする。文脈の無い
//! ログは「ロガーが 1 つだけのときだけ届く」規則（4.6）なので、アクタースレッドの入口で文脈を
//! 張らないと、メッセージループの観測ログ（`actor.spawn`・`actor.recv`・`actor.stop`）は捨てられる。
//!
//! 観測ログは debug・trace なので、tracing のフィルタ（プロセス全体で 1 つ）を `level = "trace"`
//! に固定できるよう、テスト関数を 1 本だけ置く専用のテストバイナリにしている。

mod common;

use std::path::Path;

use common::copy_fixture_to_temp;
use pasta::actor::mailbox::{ActorMsg, MailboxRequest, mailbox};
use pasta::actor::thread::{ActorThread, spawn_actor_thread};
use tempfile::TempDir;

fn trace_ghost() -> TempDir {
    let temp = copy_fixture_to_temp("shiori_lifecycle");
    let toml = temp.path().join("pasta.toml");
    let mut content = std::fs::read_to_string(&toml).unwrap();
    content.push_str("\n[logging]\nlevel = \"trace\"\n");
    std::fs::write(&toml, content).unwrap();
    temp
}

fn read_log(ghost: &Path) -> String {
    std::fs::read_to_string(ghost.join("profile/pasta/logs/pasta.log")).unwrap_or_default()
}

fn stop(tx: &flume::Sender<ActorMsg>, actor: ActorThread) {
    let (done, done_rx) = flume::bounded(1);
    tx.send(ActorMsg::Stop { done }).unwrap();
    done_rx.recv().unwrap();
    actor.join().unwrap();
}

#[test]
fn actor_thread_logs_reach_own_logger_with_two_loggers_registered() {
    let a = trace_ghost();
    let b = trace_ghost();

    let (tx_a, rx_a) = mailbox();
    let actor_a = spawn_actor_thread(0, a.path().to_path_buf(), rx_a);
    let (tx_b, rx_b) = mailbox();
    let actor_b = spawn_actor_thread(0, b.path().to_path_buf(), rx_b);
    assert!(actor_a.loaded() && actor_b.loaded());

    // ロガーが 2 つ登録された状態で、A のアクタースレッドに受信と停止のログを出させる。
    tx_a.send(ActorMsg::Notify {
        req: MailboxRequest::new(
            7,
            "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnTest\r\n\r\n",
        ),
    })
    .unwrap();
    stop(&tx_a, actor_a);
    stop(&tx_b, actor_b);

    let log_a = read_log(a.path());
    let log_b = read_log(b.path());
    // B の spawn は A のロガーが登録済みのときに出る（ロガー 2 つ）。
    assert!(
        log_b.contains("actor.spawn"),
        "B's spawn log must reach B's file:\n{log_b}"
    );
    assert!(
        log_a.contains("actor.recv"),
        "A's recv log must reach A's file:\n{log_a}"
    );
    assert!(
        log_a.contains("actor.stop"),
        "A's stop log must reach A's file:\n{log_a}"
    );
    // 他のゴーストのログファイルへは書かない（4.6）。
    assert!(
        !log_b.contains("seq=7"),
        "A's recv log must not reach B's file:\n{log_b}"
    );
    // A の done ack 後のログは、A のロガーの登録解除の後なので B のファイルへ流れない（4.7）。
    assert!(
        !log_b.contains("actor.done"),
        "post-ack log must be discarded:\n{log_b}"
    );
}
