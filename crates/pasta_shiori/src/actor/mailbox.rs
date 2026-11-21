//! 単一直列 mailbox。
//!
//! SHIORI スレッド（producer）↔アクタースレッド（consumer）間の単一直列キュー。
//!
//! # チャンネル
//! - mailbox 本体 = **flume unbounded**。consumer は単一（アクタースレッド）で、
//!   同期 `recv()` ループ（`thread.rs`）による直列処理。flume `Sender` は
//!   `Send + Sync + Clone` ゆえ芯の型 `ActorInstance` 越しに lock-free に共有され、
//!   送信パスに Mutex を置かない。
//! - reply / done = **flume bounded(1)**。GET 応答・teardown done ack とも
//!   `recv_timeout` で有界に待つ。
//!
//! # 単一 consumer 不変条件
//! mailbox は **単一の同期 `recv()` ループ**で消費し、`select!` を張らない。
//! 将来の新メッセージ源は別チャンネル＋`select!` ではなく [`ActorMsg`] の
//! variant として同一 mailbox へ merge する。この不変条件により flume の
//! cancel 欠陥（#104/#135）に構造的に到達しない。
//!
//! # FIFO 順序保証
//! flume は送信順＝受信順の FIFO を保証する。単一 mailbox を単一 consumer が逐次
//! drain することで、近接した複数 `try_send` も投入順＝処理順で逐次処理され、
//! 同時並行 VM アクセスは構造的に発生しない。複数の `Sender` クローンから合流した
//! 場合も、各送信は flume 内部で全順序化され、drain 順は一意に確定する。
//!
//! ## payload の形
//! mailbox が運ぶのは `Send` な値だけである。リクエストは生の SHIORI リクエスト文字列
//! （[`MailboxRequest`]）のまま運び、`!Send` な Lua リクエストテーブルへの変換は
//! アクタースレッド上の `PastaShiori::request` が行う。応答は SHIORI 応答文字列
//! （`String`）で返る。

use flume::{Receiver, Sender, bounded, unbounded};

/// mailbox が運ぶ SHIORI リクエスト（生のリクエスト文字列）。
///
/// Lua リクエストテーブルは `!Send` で mailbox を越えられないため、SHIORI スレッドが
/// 受け取った生文字列をそのまま運び、アクタースレッド上の `PastaShiori::request` が
/// パースしてテーブル化する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxRequest {
    /// 投入順＝処理順を観測するための順序識別子（ログに出る）。
    pub seq: u64,
    /// SHIORI リクエスト本文（アクタースレッド側でパースされる生文字列）。
    pub raw: String,
}

impl MailboxRequest {
    /// 最小コンストラクタ。`raw` を空にして順序だけを運ぶ用途にも使える。
    pub fn new(seq: u64, raw: impl Into<String>) -> Self {
        MailboxRequest {
            seq,
            raw: raw.into(),
        }
    }
}

/// mailbox を流れるメッセージ。
///
/// SHIORI marshaling の契約 3 種を表す:
/// - [`ActorMsg::Get`]: GET 同期契約。`reply`（flume `bounded(1)`）を同梱し、アクターが
///   応答値を送るか drop（→受信側 `recv_timeout` が 204）する exactly-once 経路。
/// - [`ActorMsg::Notify`]: NOTIFY fire-and-forget。応答経路を持たず即 204 で終結する。
/// - [`ActorMsg::Stop`]: teardown 制御メッセージ。`done`（flume `bounded(1)`）で完了 ack を
///   返す。先行メッセージを drain 後に同一 FIFO 上で処理される（clean drain）。
#[derive(Debug)]
pub enum ActorMsg {
    /// GET 同期メッセージ。
    Get {
        /// VM へ渡す SHIORI リクエスト（生文字列。テーブル化はアクタースレッド側）。
        req: MailboxRequest,
        /// GET 応答経路。SHIORI 応答文字列の全体（`SHIORI.request` の戻り値、または
        /// リクエスト処理エラー時の 500 応答）を運ぶ。アクターが `send` すれば値、drop すれば
        /// 受信側 `recv_timeout`→204（exactly-once は flume の move/drop 意味論が与える）。
        reply: Sender<String>,
    },
    /// NOTIFY fire-and-forget メッセージ。応答経路を持たない。
    Notify {
        /// VM へ渡す SHIORI リクエスト（生文字列。テーブル化はアクタースレッド側）。
        req: MailboxRequest,
    },
    /// teardown 制御メッセージ。drain 完了・資源解放後に `done` で ack を返す。
    Stop {
        /// teardown 完了 ack 経路（flume `bounded(1)`）。SHIORI 側は `recv_timeout` で有界に待つ。
        done: Sender<()>,
    },
}

impl ActorMsg {
    /// teardown 制御メッセージを構築する。
    ///
    /// 返り値の done [`Receiver`]（`bounded(1)`）で完了 ack を待つ。
    pub fn stop() -> (ActorMsg, Receiver<()>) {
        let (done, done_rx) = bounded(1);
        (ActorMsg::Stop { done }, done_rx)
    }
}

/// 単一直列 mailbox を生成し、producer 側 [`Sender`] と consumer 側 [`Receiver`] の
/// ペアを返す（flume unbounded）。
///
/// `Receiver<ActorMsg>` は単一 consumer（アクタースレッド）が所有し、`select!` を張らない
/// （単一 consumer 不変条件）。
pub fn mailbox() -> (Sender<ActorMsg>, Receiver<ActorMsg>) {
    unbounded()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn notify(seq: u64) -> ActorMsg {
        ActorMsg::Notify {
            req: MailboxRequest::new(seq, ""),
        }
    }

    /// GET メッセージと、その応答の受け口（`bounded(1)`）。
    fn get(seq: u64) -> (ActorMsg, Receiver<String>) {
        let (reply, reply_rx) = bounded(1);
        let req = MailboxRequest::new(seq, "");
        (ActorMsg::Get { req, reply }, reply_rx)
    }

    /// `Stop` は順序識別子を持たないため `u64::MAX` で表す。
    fn seq_of(msg: &ActorMsg) -> u64 {
        match msg {
            ActorMsg::Get { req, .. } => req.seq,
            ActorMsg::Notify { req } => req.seq,
            ActorMsg::Stop { .. } => u64::MAX,
        }
    }

    /// 現在キューにある分だけをブロッキングせず FIFO で drain する。
    fn drain(rx: &Receiver<ActorMsg>) -> Vec<ActorMsg> {
        let mut out = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            out.push(msg);
        }
        out
    }

    /// 投入順 1..=N がそのままの順序で drain される。
    #[test]
    fn drains_in_enqueue_order() {
        let (tx, rx) = mailbox();
        const N: u64 = 100;

        for i in 1..=N {
            tx.send(notify(i))
                .expect("send should succeed while receiver is alive");
        }

        let order: Vec<u64> = drain(&rx).iter().map(seq_of).collect();
        let expected: Vec<u64> = (1..=N).collect();
        assert_eq!(
            order, expected,
            "messages must drain in the exact order they were enqueued (FIFO)"
        );
    }

    /// Get/Notify/Stop を混在投入しても、種別に依らず投入順がそのまま drain 順になる。
    #[test]
    fn all_variants_preserve_fifo_order() {
        let (tx, rx) = mailbox();

        // 応答経路は使わないが、drop すると閉じるため保持しておく。
        let (get_a, _rx_a) = get(11);
        let (get_b, _rx_b) = get(13);
        let (stop, _done_rx) = ActorMsg::stop();

        tx.send(notify(10)).unwrap();
        tx.send(get_a).unwrap();
        tx.send(notify(12)).unwrap();
        tx.send(get_b).unwrap();
        tx.send(stop).unwrap();

        let order: Vec<u64> = drain(&rx).iter().map(seq_of).collect();
        assert_eq!(
            order,
            vec![10, 11, 12, 13, u64::MAX],
            "all variants must drain in enqueue order; Stop trails after prior messages"
        );
    }

    /// 複数の `Sender` クローンから別スレッドで近接して送っても、単一 consumer が
    /// すべてを一意な全順序で逐次 drain し、各 producer 内の投入順も保存される。
    #[test]
    fn cloned_senders_feed_one_serial_fifo() {
        let (tx, rx) = mailbox();
        const PRODUCERS: u64 = 8;
        const PER_PRODUCER: u64 = 50;

        let mut handles = Vec::new();
        for p in 0..PRODUCERS {
            let txc = tx.clone();
            handles.push(thread::spawn(move || {
                for i in 0..PER_PRODUCER {
                    // producer ごとの投入順を consumer 側で単調増加として観測できるように符号化する。
                    let seq = p * 1000 + i;
                    txc.send(notify(seq))
                        .expect("clone send must succeed while consumer is alive");
                }
            }));
        }
        for h in handles {
            h.join().expect("producer thread must not panic");
        }
        drop(tx);

        let drained: Vec<u64> = drain(&rx).iter().map(seq_of).collect();

        assert_eq!(
            drained.len() as u64,
            PRODUCERS * PER_PRODUCER,
            "single consumer must receive exactly every enqueued message once"
        );

        for p in 0..PRODUCERS {
            let sub: Vec<u64> = drained
                .iter()
                .copied()
                .filter(|s| s / 1000 == p)
                .map(|s| s % 1000)
                .collect();
            let expected: Vec<u64> = (0..PER_PRODUCER).collect();
            assert_eq!(
                sub, expected,
                "producer {p}'s messages must stay in enqueue order within the serial FIFO"
            );
        }

        let mut sorted = drained.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            drained.len(),
            "the single serial FIFO must contain no duplicated messages"
        );
    }

    /// `Sender` のクローンからの送信は、1 つの `Receiver` が 1 本の FIFO として受ける。
    #[test]
    fn sender_is_clonable_receiver_is_single_consumer() {
        let (tx, rx) = mailbox();

        let tx2 = tx.clone();
        tx.send(notify(1)).unwrap();
        tx2.send(notify(2)).unwrap();

        let order: Vec<u64> = drain(&rx).iter().map(seq_of).collect();
        assert_eq!(
            order,
            vec![1, 2],
            "single consumer receives both clones' sends in order"
        );

        // 芯の型 ActorInstance 越しに SHIORI スレッドと共有する前提。
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Sender<ActorMsg>>();
    }

    /// GET の `reply` を `send` すれば値が届き、send せず drop すれば受信側が `Disconnected` を
    /// 観測する（marshaling の 204 の根拠）。
    #[test]
    fn get_reply_move_then_value_or_disconnected_on_drop() {
        let (tx, rx) = mailbox();

        let (get_a, reply_rx_a) = get(1);
        tx.send(get_a).unwrap();

        let (get_b, reply_rx_b) = get(2);
        tx.send(get_b).unwrap();
        drop(tx);

        let drained = drain(&rx);
        for msg in drained {
            // seq==2 は reply を送らずスコープを抜けて drop する。
            if let ActorMsg::Get { req, reply } = msg
                && req.seq == 1
            {
                reply.send("ok".to_string()).unwrap();
            }
        }

        assert_eq!(
            reply_rx_a.recv(),
            Ok("ok".to_string()),
            "replied GET must deliver its value exactly once"
        );
        assert!(
            reply_rx_b.recv().is_err(),
            "dropped reply sender must surface as Disconnected (the 204 fallback root cause)"
        );
    }
}
