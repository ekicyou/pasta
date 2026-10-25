//! 芯の型 [`ActorInstance`]（仕様 `shiori4-v3-portable`・design.md「ActorInstance」）。
//!
//! インスタンス 1 つ＝アクター 1 つ。mailbox の送信端・アクタースレッドのハンドル・
//! ディレクトリ・要求の連番だけを持ち、グローバルな状態を参照しない（ログの登録を除く・5.1）。
//! 要求はそのインスタンスの mailbox だけを通り（5.2）、後始末はほかのインスタンスに触れない（4.3）。
//!
//! 後始末の決まり（design.md「後始末の決まり」）: Stop を送って ack を待ち、ack が届いて
//! [`JoinPolicy::JoinAfterAck`] ならアクタースレッドを join する。それ以外は手放す（detach）。
//! ack が来なければ手放したことを警告のログ（`seam = "instance.abandon"`）に出す（4.2）。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use flume::Sender;
use pasta_lua::LoadDirGuard;
use pasta_lua::debug::{KickRequest, KickSink};

use crate::actor::mailbox::{ActorMsg, MailboxRequest, mailbox};
use crate::actor::marshaling::{
    ShioriMethod, default_204, determine_method, marshal_get, marshal_notify,
};
use crate::actor::teardown::{TeardownReport, teardown_via_sender};
use crate::actor::thread::{ActorThread, spawn_actor_thread_with};

/// 後始末で ack を待つ上限（4.2）。通常運転では後始末は速やかに終わる。
/// 万一アクターが ack を返さない場合でも呼び出し側（＝ホスト）を無限待機させない安全網。
pub const TEARDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// ack が届いた後にアクタースレッドを join するか（design.md「後始末の決まり」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinPolicy {
    /// ack が届けば join してから返る（版 3 の unload・旧来の unload・4.4）。
    JoinAfterAck,
    /// join しない（`DllMain` の FreeLibrary の detach。loader lock の中では待てない）。
    Detach,
}

/// [`ActorInstance::spawn`] の失敗。
#[derive(Debug)]
pub enum SpawnError {
    /// OS がスレッドを起こせない。
    Thread(std::io::Error),
    /// アクタースレッドが ready を返す前に終わった（unwind のビルドでのパニック）。
    /// 終わったスレッドは `spawn_actor_thread_with` の中で join 済み（資源を残さない）。
    NotReady,
}

/// インスタンス 1 つ＝アクター 1 つ（5.1）。`Send + Sync`（VM はアクタースレッドから出ない・6.3）。
pub struct ActorInstance {
    /// mailbox の送信端（強い参照はこれ 1 つ。kick の送り先は弱い参照だけを持つ）。
    tx: Sender<ActorMsg>,
    /// アクタースレッドのハンドル。後始末を終えると `None`（Drop の二重の後始末を防ぐ）。
    thread: Option<ActorThread>,
    /// ゴーストのディレクトリ。
    load_dir: PathBuf,
    /// 要求の連番（FIFO 観測用の `seq`）。
    seq: AtomicU64,
}

// 芯の型が `Send + Sync` であることのコンパイルの検査（6.3）。
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ActorInstance>();
};

/// 送信端の弱い参照を閉じ込めた kick の送り先を作る（5.5）。
///
/// 弱い参照なので、kick の送り先（VM が持つ）が mailbox の受け口を生かし続けない。
/// インスタンスが送信端を手放した後は送れず、診断ログ（`seam = "kick.drop"`）を残して捨てる。
fn kick_sink_for(tx: &Sender<ActorMsg>) -> KickSink {
    let weak = tx.downgrade();
    Arc::new(move |req: KickRequest| {
        let sent = weak.upgrade().is_some_and(|tx| {
            tx.try_send(ActorMsg::Kick {
                scene: req.scene.clone(),
            })
            .is_ok()
        });
        if !sent {
            tracing::debug!(
                seam = "kick.drop",
                scene = %req.scene,
                "kick dropped: instance mailbox gone or full -> discarded"
            );
        }
    })
}

impl ActorInstance {
    /// アクタースレッドを起こし、VM を構築して `SHIORI.load` を呼ぶ。
    ///
    /// 返った時点で VM の構築と `SHIORI.load` の呼び出しは終わっている（2.3）。辞書・Lua・
    /// `SHIORI.load` の失敗はインスタンスを返す（[`loaded`](Self::loaded) が偽）。
    pub fn spawn(hinst: isize, load_dir: PathBuf) -> Result<Self, SpawnError> {
        let (tx, rx) = mailbox();
        // VM を作る前に kick の送り先を渡す（5.5）。
        let sink = kick_sink_for(&tx);
        match spawn_actor_thread_with(hinst, load_dir.clone(), rx, Some(sink)) {
            Ok(thread) => Ok(Self {
                tx,
                thread: Some(thread),
                load_dir,
                seq: AtomicU64::new(0),
            }),
            Err(e) => {
                let _log_context = LoadDirGuard::new(load_dir);
                tracing::error!(seam = "instance.spawn", error = %e, "actor thread could not start");
                // ready の前に終わったスレッドは spawn_actor_thread_with が join 済み。
                Err(if e.kind() == std::io::ErrorKind::Other {
                    SpawnError::NotReady
                } else {
                    SpawnError::Thread(e)
                })
            }
        }
    }

    /// アクタースレッド上で `SHIORI.load` が成功したか。
    pub fn loaded(&self) -> bool {
        self.thread.as_ref().is_some_and(ActorThread::loaded)
    }

    /// ゴーストのディレクトリ。
    pub fn load_dir(&self) -> &Path {
        &self.load_dir
    }

    fn next_request(&self, req: &str) -> MailboxRequest {
        MailboxRequest::new(self.seq.fetch_add(1, Ordering::Relaxed), req.to_string())
    }

    /// 旧来の意味の要求: メソッドを判定して GET／NOTIFY へ渡す。読めなければ 204。
    pub fn request(&self, req: &str) -> String {
        match determine_method(req) {
            Some(ShioriMethod::Get) => self.get(req),
            Some(ShioriMethod::Notify) => self.notify(req),
            None => default_204(),
        }
    }

    /// GET: 応答を待つ（`GET_TIMEOUT` 5 秒の安全網・失敗は 204）。
    pub fn get(&self, req: &str) -> String {
        marshal_get(&self.tx, self.next_request(req))
    }

    /// NOTIFY: 積んで即 204 を返す（3.8）。
    pub fn notify(&self, req: &str) -> String {
        marshal_notify(&self.tx, self.next_request(req))
    }

    /// kick を積む。送れなければ（アクターが居ない・満杯）偽を返す。
    pub fn kick(&self, scene: String) -> bool {
        self.tx.try_send(ActorMsg::Kick { scene }).is_ok()
    }

    /// Stop を送って ack を待つだけ（スレッドには触れない）。二度目は `already_done`。
    pub fn request_stop(&self) -> TeardownReport {
        teardown_via_sender(&self.tx, TEARDOWN_TIMEOUT)
    }

    /// 後始末（4.1・4.2・4.4）。ack が届いて [`JoinPolicy::JoinAfterAck`] なら join してから返る。
    pub fn shutdown(self, policy: JoinPolicy) -> TeardownReport {
        self.shutdown_with_timeout(policy, TEARDOWN_TIMEOUT)
    }

    /// [`shutdown`](Self::shutdown) の、ack を待つ上限を差し替える形（試験から短い値を注入する口）。
    pub fn shutdown_with_timeout(
        mut self,
        policy: JoinPolicy,
        timeout: Duration,
    ) -> TeardownReport {
        self.finish(policy, timeout)
    }

    fn finish(&mut self, policy: JoinPolicy, timeout: Duration) -> TeardownReport {
        let mut report = teardown_via_sender(&self.tx, timeout);
        let Some(thread) = self.thread.take() else {
            return report;
        };
        if report.acked && policy == JoinPolicy::JoinAfterAck {
            // ack の後のアクタースレッドに待つ所は無いので、上限なしで join しても止まらない。
            let _ = thread.join();
            report.joined = true;
        } else {
            thread.detach();
            if !report.acked && !report.already_done {
                let _log_context = LoadDirGuard::new(self.load_dir.clone());
                tracing::warn!(
                    seam = "instance.abandon",
                    load_dir = %self.load_dir.display(),
                    "actor thread abandoned: no done ack within {timeout:?}"
                );
            }
        }
        report
    }
}

impl Drop for ActorInstance {
    /// まだ片付けていなければ `shutdown(JoinAfterAck)` と同じことをする。
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.finish(JoinPolicy::JoinAfterAck, TEARDOWN_TIMEOUT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// kick の送り先は、送信端が生きている間はアクター（受け口）に kick を届け、
    /// 送信端を手放した後は送れずに捨てる（弱い参照が受け口を生かし続けない・5.5）。
    #[test]
    fn kick_sink_delivers_while_sender_alive_and_drops_after() {
        let (tx, rx) = mailbox();
        let sink = kick_sink_for(&tx);

        sink(KickRequest {
            scene: "OnTestScene".into(),
        });
        match rx.try_recv() {
            Ok(ActorMsg::Kick { scene }) => assert_eq!(scene, "OnTestScene"),
            other => panic!("expected ActorMsg::Kick, got {other:?}"),
        }

        drop(tx);
        sink(KickRequest {
            scene: "OnLate".into(),
        });
        assert!(
            matches!(rx.try_recv(), Err(flume::TryRecvError::Disconnected)),
            "the sink must not keep the mailbox alive nor deliver after the sender is gone"
        );
    }

    /// 起こしたインスタンスの kick は、アクターが居る間は送れて真、片付けた後は送れずに偽。
    #[test]
    fn instance_kick_true_while_running_false_after_stop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let instance = ActorInstance::spawn(0, dir.path().to_path_buf()).expect("spawn");
        assert_eq!(instance.load_dir(), dir.path());

        assert!(
            instance.kick("OnTestScene".into()),
            "kick must be sent while the actor runs"
        );

        let report = instance.request_stop();
        assert!(report.acked, "request_stop must be acked: {report:?}");
        assert!(!report.joined, "request_stop does not touch the thread");
        assert!(
            !instance.kick("OnLate".into()),
            "kick must fail after the actor stopped"
        );

        // 二度目の後始末は冪等（already_done）。
        let report = instance.shutdown(JoinPolicy::JoinAfterAck);
        assert!(
            report.already_done && !report.acked && !report.joined,
            "{report:?}"
        );
    }
}
