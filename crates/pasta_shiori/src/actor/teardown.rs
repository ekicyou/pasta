//! 本番 teardown（`Stop{done}` ack）（task 4.1・R7.1/R7.4/R7.5）。
//!
//! design.md「Mailbox / Reply / Teardown」コンポーネントおよび「System Flows → reload
//! teardown」の本番化。teardown は **`ActorMsg::Stop { done }` 制御メッセージ**を
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
//! # ack の後の join／detach（仕様 `shiori4-v3-portable` 4.2・4.4・7.6）
//! ack の後にスレッドをどう扱うかは芯の型 `ActorInstance` の後始末の決まりが選ぶ:
//! unload は ack の後に join し、ack が来なければ手放す（abandon）。本モジュールは
//! Stop と ack だけを受け持ち、スレッドには触れない。完了確認は done ack で行うので、
//! `Arc<AtomicBool>` shutdown フラグ・`take()` 二重 join 回避の小細工は持たない（R7.4）。
//! 再読み込みのリーク検査は `tests/actor_reload_leak_test.rs` が持つ（ponytail-shiori 6.1）。
//!
//! # 冪等性（R7.4）
//! done ack を受けた後の二重 teardown は、mailbox の receiver が既に drop されている
//! ため `Sender::send` が `Disconnected` を返す。これを **already-done** として扱い、
//! 安全に no-op を返す（ハングも panic もしない）。本番 FFI 経路（task 5.1）で unload が
//! 重なっても二重解放を起こさない構造的根拠。
//!
//! # 異常記録（R7.5）
//! ack が timeout した／途中で異常が生じた場合は [`tracing`] で記録し、
//! [`TeardownReport::anomaly`] に載せる。**panic も `unwrap`/`expect` も使わず**、
//! 異常を呼び出し側へ値で返してホスト（SSP）プロセスを巻き込まない。

use std::time::Duration;

use flume::Sender;

use crate::actor::mailbox::ActorMsg;

/// teardown の結果レポート（R7.1/R7.4/R7.5）。
///
/// `panic`/`expect` を使わず、teardown の成否・冪等 no-op・異常を **値で** 返す
/// （ホストプロセスを巻き込まないため）。
#[derive(Debug, Clone)]
pub struct TeardownReport {
    /// `done` ack を期限内に受信したか（clean teardown の主証跡）。
    pub acked: bool,
    /// すでに teardown 済みで、今回は安全な no-op だったか（冪等・R7.4）。
    ///
    /// mailbox receiver が drop 済み（=アクター終了済み）で Stop 送信が `Disconnected`
    /// になった場合に `true`。この場合 `acked` は `false`（新たな ack は来ない）。
    pub already_done: bool,
    /// teardown 途中の異常（ack timeout・送信失敗等）。`Some` ならホストを巻き込まずに
    /// 記録した異常内容（R7.5）。clean teardown／冪等 no-op の双方とも `None`。
    pub anomaly: Option<String>,
    /// アクタースレッドの join を終えたか（仕様 `shiori4-v3-portable` 4.4・7.6 の観測点）。
    ///
    /// ack が届き、後始末の決まりが「ack の後に join」のときだけ真
    /// （`ActorInstance::shutdown`）。Stop と ack だけの道（[`teardown_via_sender`]）では常に偽。
    pub joined: bool,
}

/// mailbox 送信端のみで teardown（Stop{done}→ack 待ち）を試行する冪等経路。
///
/// 本番 FFI 経路では `ActorThread` ハンドルを持たずに `static LEGACY`（task 5.1）の
/// `Sender` 越しに teardown を撃つ場面があり、また二重 teardown（unload の重複）も
/// 起こりうる。本関数はそのどちらでも安全に振る舞う:
///
/// - mailbox receiver が drop 済み（アクター既終了）→ `Sender::send` が `Disconnected`
///   → **already-done** として no-op（`already_done=true`・hang/panic なし・R7.4）。
/// - 送信成功 → `done_rx.recv_timeout(timeout)`:
///   - `Ok(())` → `acked=true`（clean）。
///   - `Err(Timeout)` / `Err(Disconnected)` → 異常を記録（[`tracing::warn`]）し
///     `anomaly=Some(..)`（R7.5・ホストを巻き込まない）。
pub fn teardown_via_sender(tx: &Sender<ActorMsg>, timeout: Duration) -> TeardownReport {
    let (stop, done_rx) = ActorMsg::stop();

    // Stop を単一 FIFO へ投入。先行メッセージはこの Stop の前に drain される（clean
    // drain）。receiver drop 済みなら Disconnected = already-done（冪等 no-op）。
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
    // 観測ログ点（R10.4・無効時ゼロコスト）: Stop を mailbox へ投入し done ack を待つ。
    tracing::debug!(
        seam = "actor.stop",
        ok = true,
        "teardown: Stop enqueued; awaiting done ack"
    );

    // done ack を有界に待つ。teardown が壊れて ack を返さない／drain せず break した
    // 場合に無限待機せず、異常として終結させる（テストはハングせず失敗する）。
    match done_rx.recv_timeout(timeout) {
        Ok(()) => {
            // 観測ログ点（R10.4）: done ack 受信＝drain/VM 破棄/cleanup 完了（clean teardown）。
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
            // ack が来ない = drain/cleanup/ack のいずれかが完了していない疑い。
            // 記録するがホスト（SSP）プロセスは巻き込まない（R7.5）。
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
            // teardown の完了確認は取れていないため異常として記録（R7.5）。
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
