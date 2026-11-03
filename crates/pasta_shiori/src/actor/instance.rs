//! 芯の型 [`ActorInstance`]（仕様 `shiori4-v3-portable`・design.md「ActorInstance」）。
//!
//! インスタンス 1 つ＝アクター 1 つ。mailbox の送信端・アクタースレッドのハンドル・
//! ディレクトリ・要求の連番だけを持ち、グローバルな状態を参照しない（ログの登録を除く・5.1）。
//! 要求はそのインスタンスの mailbox だけを通り（5.2）、後始末はほかのインスタンスに触れない（4.3）。
//!
//! 後始末の決まり（design.md「後始末の決まり」）: Stop を送って ack を待ち、ack が届けば
//! アクタースレッドを join する（受け口が既に閉じていたときも join）。それ以外は手放す（detach）。
//! ack が来なければ手放したことを警告のログ（`seam = "instance.abandon"`）に出す（4.2）。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use flume::Sender;
use pasta_lua::{GlobalLoggerRegistry, LoadDirGuard, PastaLogger};
use pest::error::LineColLocation;

use crate::actor::mailbox::{ActorMsg, MailboxRequest, mailbox};
use crate::actor::marshaling::{
    RequestKind, ShioriMethod, classify_request, default_204, marshal_get, marshal_notify,
};
use crate::actor::teardown::{TeardownReport, teardown_via_sender};
use crate::actor::thread::{ActorThread, spawn_actor_thread_with};
use crate::util::parsers::req::ParseError;

/// 後始末で ack を待つ上限（4.2）。通常運転では後始末は速やかに終わる。
/// 万一アクターが ack を返さない場合でも呼び出し側（＝ホスト）を無限待機させない安全網。
pub const TEARDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// 起動の前のディレクトリの占有のしかた（design.md「ActorInstance」・6.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimMode {
    /// 占有されていれば [`SpawnError::DirInUse`]（版 3 と、旧来の口の普通の読み込み）。
    Exclusive,
    /// 占有を新しい世代で奪う（旧来の口が、打ち切りに終わった自分の片付けの直後に同じディレクトリを読むときだけ・7.1・7.2）。
    TakeOver,
}

/// [`ActorInstance::spawn`] の失敗。
#[derive(Debug)]
pub enum SpawnError {
    /// 同じディレクトリを生きているインスタンスが占有している（6.2）。
    DirInUse,
    /// OS がスレッドを起こせない。
    Thread(std::io::Error),
    /// アクタースレッドが ready を返す前に終わった（unwind のビルドでのパニック）。
    /// 終わったスレッドは `spawn_actor_thread_with` の中で join 済み（資源を残さない）。
    NotReady,
}

/// インスタンス 1 つ＝アクター 1 つ（5.1）。`Send + Sync`（VM はアクタースレッドから出ない・6.3）。
pub struct ActorInstance {
    /// mailbox の送信端。
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

impl ActorInstance {
    /// アクタースレッドを起こし、VM を構築して `SHIORI.load` を呼ぶ（[`ClaimMode::Exclusive`]）。
    ///
    /// 返った時点で VM の構築と `SHIORI.load` の呼び出しは終わっている（2.3）。辞書・Lua・
    /// `SHIORI.load` の失敗はインスタンスを返す（[`loaded`](Self::loaded) が偽）。
    pub fn spawn(load_dir: PathBuf) -> Result<Self, SpawnError> {
        Self::spawn_with(load_dir, ClaimMode::Exclusive)
    }

    /// [`spawn`](Self::spawn) の、ディレクトリの占有のしかたを選ぶ形。
    ///
    /// 起動の前にディレクトリを占有して札を得る。札はアクタースレッドの VM が持ち、VM の
    /// 片付けで札で外す。占有に失敗したら占有している側のログに理由を出し、
    /// [`SpawnError::DirInUse`] を返す（6.2・2.5）。
    pub fn spawn_with(load_dir: PathBuf, mode: ClaimMode) -> Result<Self, SpawnError> {
        let registry = GlobalLoggerRegistry::instance();
        let token = match mode {
            ClaimMode::Exclusive => match registry.try_claim(load_dir.clone()) {
                Some(token) => token,
                None => {
                    // 文脈を占有している側のディレクトリにして、その側のログのファイルへ出す。
                    let _log_context = LoadDirGuard::new(load_dir.clone());
                    tracing::warn!(
                        seam = "instance.claim",
                        load_dir = %load_dir.display(),
                        "load refused: the ghost directory is in use by another live instance"
                    );
                    return Err(SpawnError::DirInUse);
                }
            },
            ClaimMode::TakeOver => {
                let token = registry.take_over(load_dir.clone());
                let _log_context = LoadDirGuard::new(load_dir.clone());
                tracing::warn!(
                    seam = "instance.claim",
                    load_dir = %load_dir.display(),
                    "claim taken over from an abandoned actor thread of the same directory"
                );
                token
            }
        };

        let (tx, rx) = mailbox();
        match spawn_actor_thread_with(load_dir.clone(), rx, Some(token.clone())) {
            Ok(thread) => Ok(Self {
                tx,
                thread: Some(thread),
                load_dir,
                seq: AtomicU64::new(0),
            }),
            Err(e) => {
                // ready の前に終わったスレッドは、その VM の片付けで札を外している。項目が無ければ
                // 取り直し、理由を出した後に外す（項目を残さない）。
                // shortcut: 取り直す前に別のインスタンスが同じディレクトリを占有すると、そのロガーを既定のものに替えてしまう。unwind のビルドで同じディレクトリを同時に起こすときだけ起きるので、それが要る用途ができたら直す。
                let token = registry.try_claim(load_dir.clone()).unwrap_or(token);
                // 理由を出すため、そのディレクトリに既定のロガーを登録する（2.5）。
                if let Ok(logger) = PastaLogger::new(&load_dir, None) {
                    registry.register(load_dir.clone(), Arc::new(logger));
                }
                {
                    let _log_context = LoadDirGuard::new(load_dir);
                    tracing::error!(seam = "instance.spawn", error = %e, "actor thread could not start");
                }
                registry.release_claim(&token);
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

    /// 旧来の意味の要求: メソッドを判定して GET／NOTIFY へ渡す（2.x も Lua へ）。読めなければ 204。
    pub fn request(&self, req: &str) -> String {
        match classify_request(req) {
            Ok(RequestKind::Get | RequestKind::Shiori2(ShioriMethod::Get)) => self.get(req),
            Ok(RequestKind::Notify | RequestKind::Shiori2(ShioriMethod::Notify)) => {
                self.notify(req)
            }
            Err(e) => {
                self.warn_unreadable("legacy", req, &e);
                default_204()
            }
        }
    }

    /// 解けない要求を、このインスタンスのログに warn 1 行で残す（3.6〜3.8）。
    /// 要求は利用者の名前を含みうるので、位置と 1 行目だけを載せ、全文は載せない。
    pub(crate) fn warn_unreadable(&self, entry: &'static str, req: &str, err: &ParseError) {
        let (line, col) = match err.line_col {
            LineColLocation::Pos(p) | LineColLocation::Span(p, _) => p,
        };
        let first_line = req.lines().next().unwrap_or("");
        let _log_context = LoadDirGuard::new(self.load_dir.clone());
        tracing::warn!(
            seam = "request.unreadable",
            entry,
            line,
            col,
            first_line,
            "request could not be parsed"
        );
    }

    /// GET: 応答を待つ（`GET_TIMEOUT` 5 秒の安全網・失敗は 204）。
    pub fn get(&self, req: &str) -> String {
        marshal_get(&self.tx, self.next_request(req))
    }

    /// NOTIFY: 積んで即 204 を返す（3.8）。
    pub fn notify(&self, req: &str) -> String {
        marshal_notify(&self.tx, self.next_request(req))
    }

    /// Stop を送って ack を待つだけ（スレッドには触れない）。二度目は `already_done`。
    pub fn request_stop(&self) -> TeardownReport {
        teardown_via_sender(&self.tx, TEARDOWN_TIMEOUT)
    }

    /// 後始末（4.1・4.2・4.4）。ack が届けば join してから返る。
    pub fn shutdown(self) -> TeardownReport {
        self.shutdown_with_timeout(TEARDOWN_TIMEOUT)
    }

    /// [`shutdown`](Self::shutdown) の、ack を待つ上限を差し替える形（試験から短い値を注入する口）。
    pub fn shutdown_with_timeout(mut self, timeout: Duration) -> TeardownReport {
        self.finish(timeout)
    }

    fn finish(&mut self, timeout: Duration) -> TeardownReport {
        let mut report = teardown_via_sender(&self.tx, timeout);
        let Some(thread) = self.thread.take() else {
            return report;
        };
        // 受け口が既に閉じている（already_done）スレッドも、片付けを終えて約束を待つか panic で
        // 抜けたかなので join は止まらない。見捨てる（スレッドが参照を返す）のは ack が来ないときだけ。
        if report.acked || report.already_done {
            // ack の後のアクタースレッドは約束を待つだけなので、上限なしで join しても止まらない。
            let _ = thread.join();
            report.joined = true;
        } else {
            thread.detach();
            let _log_context = LoadDirGuard::new(self.load_dir.clone());
            tracing::warn!(
                seam = "instance.abandon",
                load_dir = %self.load_dir.display(),
                "actor thread abandoned: no done ack within {timeout:?}"
            );
        }
        report
    }
}

impl Drop for ActorInstance {
    /// まだ片付けていなければ `shutdown()` と同じことをする。
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.finish(TEARDOWN_TIMEOUT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 起こしたインスタンスは request_stop で ack を返し、その後の後始末は冪等に join する。
    #[test]
    fn instance_request_stop_then_shutdown_joins() {
        let dir = tempfile::tempdir().expect("tempdir");
        let instance = ActorInstance::spawn(dir.path().to_path_buf()).expect("spawn");
        assert_eq!(instance.load_dir(), dir.path());

        let report = instance.request_stop();
        assert!(report.acked, "request_stop must be acked: {report:?}");
        assert!(!report.joined, "request_stop does not touch the thread");

        // 二度目の後始末は冪等（already_done）で、受け口を閉じたスレッドを join する。
        let report = instance.shutdown();
        assert!(
            report.already_done && !report.acked && report.joined,
            "{report:?}"
        );
    }
}
