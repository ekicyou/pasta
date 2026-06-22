//! RN1 薄い実証: flume `recv_async().await` × wintf `block_on` の wake 統合
//! （task 3.1・design.md Open Question RN1）。
//!
//! 本番アクタースレッド（task 3.3）の実装前に成立していなければならない核心を
//! 最小実証する焦点付きハーネス。**出荷既定コードは一切触らない**（`actor-poc`
//! feature 有効時のみコンパイル単位に現れる）。
//!
//! # 証明する命題
//!
//! 専用 OS スレッド上で `wintf_winmsg_executor::block_on(future)` を回し、future が
//! flume `Receiver::recv_async().await` をループで待つ。別スレッドが flume
//! `Sender::try_send` でメッセージを投入すると、**手動 wake / `PostMessage` を一切
//! 呼ばずに** その await が起床しメッセージを直列・FIFO で消費する。Stop メッセージで
//! ループを抜け `block_on` が戻り、スレッドを join できる。
//!
//! これは flume が `recv_async` の `poll` 時に渡された `Waker`（wintf executor 由来）を
//! 自分の待機リストへ登録し、`try_send` 時にその `Waker::wake()` を呼ぶ＝wintf の
//! メッセージ専用ウィンドウへ wake を post して message loop を再ポーリングさせる、
//! という native 統合が wintf 0.0.3 上で実際に効くことの裏取りである。
//!
//! # 既存 PoC（pasta_lua::actor_poc::actor_thread）との差分
//!
//! 既存 PoC は std `mpsc` ＋ future 側で捕捉した `Waker` を producer が
//! `wake_by_ref()` する **手動 wake** でループを進めていた。本番設計はチャンネルを
//! flume 1 本に統一し、その手動 wake を撤廃して `recv_async().await` の native Waker
//! 統合に依存する。本モジュールはその native 統合のみを切り出して実証する（手動
//! wake は一切行わない＝[`FlumeWakeProof::manual_wake_calls`] は常に 0）。

use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

/// 実証用メッセージ（mailbox の最小形）。
///
/// 本番 `ActorMsg`（Get/Notify/Stop）の縮約版で、wake 統合の検証に必要な最小限のみ。
#[derive(Debug, Clone, Copy)]
pub enum ProofMsg {
    /// アクター側で消費して `consumed` へ追記する値。
    Value(i64),
    /// ループを抜けて `block_on` を完了させる停止指示（本番 `ActorMsg::Stop` 相当）。
    Stop,
}

/// 実証結果。テストが観測する事実だけを値で持ち帰る（VM/チャンネル本体は越境しない）。
#[derive(Debug, Clone)]
pub struct FlumeWakeProof {
    /// 受信（アクター）スレッドが期限内に join できたか。
    ///
    /// `false` なら `recv_async().await` が起床せず `block_on` が戻らなかった＝wake
    /// 統合が効かなかったことを意味する（テストはここで失敗する）。
    pub joined_within_timeout: bool,
    /// アクター側が消費した値の列（投入順＝FIFO で並ぶはず）。
    pub consumed: Vec<i64>,
    /// `recv_async().await` を実際に駆動したスレッド（＝専用アクタースレッド）の id。
    pub actor_thread_id: ThreadId,
    /// `try_send` を行った送信スレッド（本関数の呼び出しスレッド）の id。
    pub sender_thread_id: ThreadId,
    /// 手動 wake を呼んだ回数。本実証は flume native Waker のみに依存するため常に 0。
    pub manual_wake_calls: u32,
}

/// flume × wintf の wake 統合を実証する。
///
/// 手順:
/// 1. flume unbounded チャンネルを 1 本作る（本番 Mailbox と同じ単一直列 FIFO）。
/// 2. 専用 OS スレッドを spawn し、その上で `wintf_winmsg_executor::block_on(future)` を
///    回す。future は `while let Ok(msg) = rx.recv_async().await` でメッセージを消費し、
///    `Stop` でループを抜ける（本番 ActorThread の `while let` と同形）。
/// 3. **別スレッド（本関数の呼び出しスレッド）** から `tx.try_send` で `messages` を
///    投入し、最後に `Stop` を投入する。手動 wake は一切行わない。
/// 4. 受信スレッドを `timeout` 付きで join する。期限内に join できれば、別スレッドの
///    `try_send` が `recv_async().await` を起床させ全消費→Stop→`block_on` 完了まで
///    進んだことを意味する（wake 統合 OK）。
///
/// `timeout` は wake が効かない場合に無限ハングを避け、テストを **失敗** させるための
/// 有界待ち。eff/false 判定により「自明に真」なテストにならない。
pub fn run_flume_wake_proof(messages: &[ProofMsg], timeout: Duration) -> FlumeWakeProof {
    // (1) 本番 Mailbox と同じ flume unbounded（FIFO・単一 consumer・select! なし）。
    let (tx, rx) = flume::unbounded::<ProofMsg>();

    // 消費結果とアクタースレッド id を持ち帰るための小チャンネル（値のみ越境）。
    let (result_tx, result_rx) = flume::bounded::<(Vec<i64>, ThreadId)>(1);

    // (2) 専用アクタースレッドで block_on（!Send 制約下の future を完了まで駆動）。
    let actor = thread::Builder::new()
        .name("pasta-actor-poc-flume-wake".to_string())
        .spawn(move || {
            // このスレッド上の Windows メッセージループを回し、future を完走させる。
            // future は flume recv_async().await のみで待機し、手動 wake は使わない。
            let (consumed, tid) = wintf_winmsg_executor::block_on(async move {
                let mut consumed: Vec<i64> = Vec::new();
                // 本番 ActorThread と同形: 単一 recv・select! なし・手動 wake なし。
                // try_send 側が flume native Waker を起こすことだけが再ポーリングを駆動する。
                while let Ok(msg) = rx.recv_async().await {
                    match msg {
                        ProofMsg::Value(v) => consumed.push(v),
                        // Stop でループ脱出 → async ブロック完了 → block_on が戻る。
                        ProofMsg::Stop => break,
                    }
                }
                (consumed, thread::current().id())
            });
            // 値（消費列＋実行スレッド id）だけを越境させる。
            let _ = result_tx.send((consumed, tid));
        })
        .expect("actor thread must spawn");

    let sender_thread_id = thread::current().id();

    // (3) 別スレッド（=この呼び出しスレッド）から try_send で投入。手動 wake は一切なし。
    //     wake は flume の Waker（recv_async の poll 時に登録済み）が担う。
    for msg in messages {
        // recv_async がまだ初回 poll で Waker を登録していない可能性に依存しない:
        // flume はバッファにメッセージが残っていれば次の recv_async で即取得し、
        // 登録済み Waker があれば wake する。順序は unbounded の FIFO が保証する。
        tx.try_send(*msg)
            .expect("try_send into unbounded mailbox must not fail");
    }
    // 末尾に Stop を投入してループを終わらせる（clean stop）。
    tx.try_send(ProofMsg::Stop)
        .expect("try_send Stop must not fail");

    // (4) 受信結果を有界タイムアウトで待つ。期限内に来れば wake 統合 OK。
    //     来なければ recv_async が起床せず block_on が戻っていない＝統合 NG。
    let (consumed, actor_thread_id, joined_within_timeout) =
        match result_rx.recv_timeout(timeout) {
            Ok((consumed, tid)) => {
                // 結果を受け取れた = block_on は戻った。残るは OS スレッドの join。
                // ここからの join は即時のはずだが、念のため上限 `timeout` で見張る。
                let join_ok = join_with_timeout(actor, timeout);
                (consumed, tid, join_ok)
            }
            Err(_) => {
                // タイムアウト: future が起床せず block_on が戻らなかった。
                // スレッドは join せず leak させる（テストは失敗する想定なので可）。
                (Vec::new(), sender_thread_id, false)
            }
        };

    FlumeWakeProof {
        joined_within_timeout,
        consumed,
        actor_thread_id,
        sender_thread_id,
        // flume native Waker のみに依存し、本ハーネスは手動 wake を一度も呼ばない。
        manual_wake_calls: 0,
    }
}

/// OS スレッドを有界時間で join する。`JoinHandle` 自体には timeout 付き join が無いため、
/// 完了済みかをポーリングで確認する（result を既に受け取っている＝ほぼ即時に完了する）。
fn join_with_timeout(handle: thread::JoinHandle<()>, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    // 完了していれば join は即返る。is_finished で完了を待ってから join する。
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            return false;
        }
        thread::yield_now();
    }
    handle.join().is_ok()
}
