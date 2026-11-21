//! 本番 teardown（`Stop{done}` ack）。
//!
//! teardown は **`ActorMsg::Stop { done }` 制御メッセージ**を
//! mailbox（単一 FIFO）へ送ることで成立する。Stop は同一 FIFO を通るため、先行
//! メッセージは Stop の前に drain される（clean drain）。アクター（[`thread`] の
//! `spawn_actor_thread_with` が回す単一の同期 `recv()` ループ）は Stop で:
//!
//! 1. 残メッセージを drain 後（FIFO 順序が保証）、
//! 2. VM（`PastaShiori` が内包する `!Send` Lua VM）を drop して debug backend teardown を
//!    完了させ、mailbox の receiver を閉じ、
//! 3. `done` ack を送る、
//!
//! という順序で teardown する。その後スレッドは join の約束を待ってから終わる。SHIORI 側は `done_rx.recv_timeout` で **有界に** ack を
//! 待ち、ack 受信＝全資源解放済みを確認する。
//!
//! # ack の後の join／detach
//! ack の後にスレッドをどう扱うかは芯の型 `ActorInstance` の後始末の決まりが選ぶ:
//! unload は ack の後に join し、ack が来なければ手放す（abandon）。本モジュールは
//! Stop と ack だけを受け持ち、スレッドには触れない。
//!
//! # 冪等性
//! done ack を受けた後の二重 teardown は、mailbox の receiver が既に drop されている
//! ため `Sender::send` が `Disconnected` を返す。これを **already-done** として扱い、
//! 安全に no-op を返す（ハングも panic もしない）。
//!
//! # 異常記録
//! ack が timeout した／途中で異常が生じた場合は [`tracing`] で記録し、
//! [`TeardownReport::anomaly`] に載せる。**panic も `unwrap`/`expect` も使わず**、
//! 異常を呼び出し側へ値で返してホスト（SSP）プロセスを巻き込まない。

use std::time::Duration;

use flume::Sender;

use crate::actor::mailbox::ActorMsg;

/// teardown の結果レポート。成否・冪等 no-op・異常を **値で** 返す（ホストプロセスを巻き込まないため）。
#[derive(Debug, Clone)]
pub struct TeardownReport {
    /// `done` ack を期限内に受信したか。
    pub acked: bool,
    /// すでに teardown 済みで、今回は安全な no-op だったか。
    ///
    /// mailbox receiver が drop 済み（=アクター終了済み）で Stop 送信が `Disconnected`
    /// になった場合に `true`。この場合 `acked` は `false`（新たな ack は来ない）。
    pub already_done: bool,
    /// teardown 途中の異常（ack timeout・送信失敗等）。clean teardown／冪等 no-op の双方とも `None`。
    pub anomaly: Option<String>,
    /// アクタースレッドの join を終えたか。
    ///
    /// ack が届き、後始末の決まりが「ack の後に join」のときだけ真
    /// （`ActorInstance::shutdown`）。Stop と ack だけの道（[`teardown_via_sender`]）では常に偽。
    pub joined: bool,
}

/// mailbox 送信端のみで teardown（Stop{done}→ack 待ち）を試行する冪等経路。
///
/// - mailbox receiver が drop 済み（アクター既終了）→ **already-done** として no-op。
/// - 送信成功 → `done_rx.recv_timeout(timeout)`:
///   - `Ok(())` → `acked=true`（clean）。
///   - `Err(Timeout)` / `Err(Disconnected)` → 異常を記録（[`tracing::warn`]）し `anomaly=Some(..)`。
pub fn teardown_via_sender(tx: &Sender<ActorMsg>, timeout: Duration) -> TeardownReport {
    let (stop, done_rx) = ActorMsg::stop();

    if tx.send(stop).is_err() {
        tracing::debug!(
            seam = "actor.stop",
            ok = false,
            "teardown: mailbox receiver already gone; treating as already-done (idempotent no-op)"
        );
        return TeardownReport {
            acked: false,
            already_done: true,
            anomaly: None,
            joined: false,
        };
    }
    tracing::debug!(
        seam = "actor.stop",
        ok = true,
        "teardown: Stop enqueued; awaiting done ack"
    );

    match done_rx.recv_timeout(timeout) {
        Ok(()) => {
            tracing::debug!(
                seam = "actor.done",
                acked = true,
                "teardown: done ack received (resources released)"
            );
            TeardownReport {
                acked: true,
                already_done: false,
                anomaly: None,
                joined: false,
            }
        }
        Err(flume::RecvTimeoutError::Timeout) => {
            let msg = format!("teardown: done ack timed out after {timeout:?}");
            tracing::warn!("{msg}");
            TeardownReport {
                acked: false,
                already_done: false,
                anomaly: Some(msg),
                joined: false,
            }
        }
        Err(flume::RecvTimeoutError::Disconnected) => {
            // Stop は送れたが done sender が ack 前に drop された（アクター異常終了等）。
            let msg =
                "teardown: done channel disconnected before ack (actor dropped sender)".to_string();
            tracing::warn!("{msg}");
            TeardownReport {
                acked: false,
                already_done: false,
                anomaly: Some(msg),
                joined: false,
            }
        }
    }
}
