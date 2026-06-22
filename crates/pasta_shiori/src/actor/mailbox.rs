//! 単一直列 mailbox（本番化・task 3.2／R6.1-R6.4）。
//!
//! SHIORI スレッド（producer）↔アクタースレッド（consumer）間の単一直列キュー。
//! 本番アクターランタイム（design.md「Mailbox / Reply / Teardown」コンポーネント）の
//! メッセージ契約の中核であり、`actor_poc/mailbox.rs`（std mpsc・PoC）を **flume**
//! へ差し替えて本番昇格したもの。
//!
//! # チャンネル決定（design.md Technology Stack / RN7）
//! - mailbox 本体 = **flume unbounded**。consumer は単一（アクタースレッド）で、
//!   `recv_async().await`（task 3.3 で配線）による直列処理。flume `Sender` は
//!   `Send + Sync + Clone` ゆえ将来 `static MAILBOX` へ lock-free 共有でき、
//!   送信パスに Mutex を置かない（R8 構造的達成の素地）。
//! - reply / done = **flume bounded(1)**。GET 応答は `recv_timeout`（同期・task 3.4）、
//!   teardown done ack は `recv()` で待つ（task 7.x）。
//!
//! # 単一 consumer 不変条件（design.md Revalidation Trigger）
//! mailbox は **単一の非 cancel `recv_async` ループ**で消費し、`select!` を張らない。
//! 将来の新メッセージ源（Kick 等）は別チャンネル＋`select!` ではなく [`ActorMsg`] の
//! variant として同一 mailbox へ merge する。この不変条件により flume の
//! cancel 欠陥（#104/#135）に構造的に到達しない。
//!
//! # FIFO 順序保証（R6.1-R6.4）
//! flume は送信順＝受信順の FIFO を保証する。単一 mailbox を単一 consumer が逐次
//! drain することで、近接した複数 `try_send` も投入順＝処理順で逐次処理され、
//! 同時並行 VM アクセスは構造的に発生しない。複数の `Sender` クローンから合流した
//! 場合も、各送信は flume 内部で全順序化され、drain 順は一意に確定する。
//!
//! ## task 3.2 のスコープ
//! 本タスクは mailbox の FIFO / 単一直列 / 順序契約を確立・検証する。`ActorMsg` の
//! `Get`/`Stop` が運ぶ実 payload（`!Send` な Lua リクエストテーブル・VM 応答）の
//! marshaling は task 3.3（VM pin）/ 3.4（marshaling 本番化）の責務であり、ここでは
//! mailbox 層が運ぶ最小の placeholder 型（[`MailboxRequest`] / [`Reply`]）で
//! FIFO・単一直列・順序一意性を実証する。

use flume::{Receiver, Sender, bounded, unbounded};

/// mailbox が運ぶ SHIORI リクエストの最小表現（task 3.2 の placeholder）。
///
/// 本番では GET/NOTIFY が VM へ渡す Lua リクエストテーブル相当を運ぶが、それは
/// `!Send` であり marshaling（task 3.4）の責務。task 3.2 では FIFO / 単一直列 /
/// 順序一意性を実証するための最小・`Send` な表現に留める。`seq` は投入順＝処理順を
/// 検証するための順序識別子。
///
/// > 申し送り（task 3.3/3.4）: 本番では `req: LuaRequestTable`（VM スレッド内で構築・
/// > 消費する `!Send` 値）へ置換する。mailbox は `Send` 境界を越えるため、SHIORI
/// > スレッド側で受け取った生リクエスト文字列を運び、VM 側でパース／テーブル化する
/// > 設計（design.md の `LuaRequestTable` は VM 同居の表現）が自然。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxRequest {
    /// 投入順＝処理順の検証に用いる順序識別子。
    pub seq: u64,
    /// SHIORI リクエスト本文（本番では VM 側でパースされる生文字列を想定）。
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

/// GET 応答（task 3.2 の placeholder）。
///
/// 本番では VM が構築したさくらスクリプト応答文字列を運ぶ（design.md「Reply」）。
/// exactly-once は flume の move/drop 意味論が自然に与える（独自 `Responder` 不要）:
/// アクターが `reply.send(Reply::Value(..))` すれば値、send せず drop すれば受信側の
/// 同期 `recv_timeout` が `Disconnected`→204。task 3.2 では型と FIFO 投入のみ扱い、
/// 実 `recv_timeout` 消費は task 3.4。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// VM が構築した応答値（本番ではさくらスクリプト文字列）。
    Value(String),
}

/// mailbox を流れるメッセージの判別共用体（design.md「Event Contract（Mailbox）」）。
///
/// SHIORI marshaling の契約 3 種を表す:
/// - [`ActorMsg::Get`]: GET 同期契約。`reply`（flume `bounded(1)`）を同梱し、アクターが
///   応答値を送るか drop（→受信側 `recv_timeout` が 204）する exactly-once 経路。
/// - [`ActorMsg::Notify`]: NOTIFY fire-and-forget。応答経路を持たず即 204 で終結する。
/// - [`ActorMsg::Stop`]: teardown 制御メッセージ。`done`（flume `bounded(1)`）で完了 ack を
///   返す。先行メッセージを drain 後に同一 FIFO 上で処理される（clean drain）。
///
/// `#[non_exhaustive]` により、将来 `Kick`（talk FIFO 投入・後続仕様 `pasta-scene-kick`）を
/// 破壊的変更なしに追加できる（別チャンネル＋`select!` ではなく **同一 mailbox への
/// variant merge** が単一 consumer 不変条件を保つ唯一の拡張手段）。
#[derive(Debug)]
#[non_exhaustive]
pub enum ActorMsg {
    /// GET 同期メッセージ。`reply` は flume `bounded(1)` の応答経路（exactly-once）。
    Get {
        /// VM へ渡す SHIORI リクエスト（task 3.2 placeholder・本番は `!Send` テーブル化）。
        req: MailboxRequest,
        /// GET 応答経路。アクターが `send` すれば値、drop すれば受信側 `recv_timeout`→204。
        reply: Sender<Reply>,
    },
    /// NOTIFY fire-and-forget メッセージ。応答経路を持たない。
    Notify {
        /// VM へ渡す SHIORI リクエスト（task 3.2 placeholder）。
        req: MailboxRequest,
    },
    /// teardown 制御メッセージ。drain 完了・資源解放後に `done` で ack を返す。
    Stop {
        /// teardown 完了 ack 経路（flume `bounded(1)`）。SHIORI 側は `recv()` で待つ。
        done: Sender<()>,
    },
}

impl ActorMsg {
    /// GET 応答経路 (`reply`) を持つメッセージを構築する。
    ///
    /// 呼び出し側は返り値の [`Receiver`]（`bounded(1)`）で応答を待つ（task 3.4 の
    /// `recv_timeout` 消費の素地）。task 3.2 では FIFO 投入と move/drop の確認に用いる。
    pub fn get(req: MailboxRequest) -> (ActorMsg, Receiver<Reply>) {
        let (reply, reply_rx) = bounded(1);
        (ActorMsg::Get { req, reply }, reply_rx)
    }

    /// NOTIFY メッセージを構築する（応答経路なし）。
    pub fn notify(req: MailboxRequest) -> ActorMsg {
        ActorMsg::Notify { req }
    }

    /// teardown 制御メッセージを構築する。
    ///
    /// 返り値の done [`Receiver`]（`bounded(1)`）で完了 ack を待つ（task 7.x の素地）。
    pub fn stop() -> (ActorMsg, Receiver<()>) {
        let (done, done_rx) = bounded(1);
        (ActorMsg::Stop { done }, done_rx)
    }
}

/// 単一直列 mailbox を生成し、producer 側 [`Sender`] と consumer 側 [`Receiver`] の
/// ペアを返す（flume unbounded）。
///
/// `Sender<ActorMsg>` は `Send + Sync + Clone` ゆえ複数 SHIORI 経路（および将来の
/// `static MAILBOX`）から lock-free に合流でき、`Receiver<ActorMsg>` は単一 consumer
/// （アクタースレッド）が所有する。consumer は `recv_async().await`（task 3.3 配線）で
/// 単一 recv ループを回し、`select!` を張らない（単一 consumer 不変条件）。
pub fn mailbox() -> (Sender<ActorMsg>, Receiver<ActorMsg>) {
    unbounded()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    /// 種別に依らず順序識別子 (`seq`) を取り出すテストヘルパ。
    ///
    /// `Stop` は順序識別子を持たないため、テストでは予約値（`u64::MAX`）で表す
    /// （`Stop` を末尾に置く FIFO テストで終端マーカーとして機能する）。
    fn seq_of(msg: &ActorMsg) -> u64 {
        match msg {
            ActorMsg::Get { req, .. } => req.seq,
            ActorMsg::Notify { req } => req.seq,
            ActorMsg::Stop { .. } => u64::MAX,
        }
    }

    /// 単一 consumer が現在キューにあるメッセージを FIFO で drain する。
    ///
    /// 本番アクターは `recv_async().await` の単一ループで消費する（task 3.3）。
    /// task 3.2 のテストではブロッキングせず `try_recv` で「現時点で利用可能な分」を
    /// 投入順に取り出し、FIFO / 単一直列契約を確認する。
    fn drain(rx: &Receiver<ActorMsg>) -> Vec<ActorMsg> {
        let mut out = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            out.push(msg);
        }
        out
    }

    /// R6.1/R6.2（FIFO）: 投入順 1..=N がそのままの順序で drain される。
    #[test]
    fn drains_in_enqueue_order() {
        let (tx, rx) = mailbox();
        const N: u64 = 100;

        for i in 1..=N {
            tx.send(ActorMsg::notify(MailboxRequest::new(i, "")))
                .expect("send should succeed while receiver is alive");
        }

        let order: Vec<u64> = drain(&rx).iter().map(seq_of).collect();
        let expected: Vec<u64> = (1..=N).collect();
        assert_eq!(
            order, expected,
            "messages must drain in the exact order they were enqueued (FIFO)"
        );
    }

    /// R6.1/R6.4（全 variant FIFO）: Get/Notify/Stop を混在投入しても、種別に依らず
    /// 投入順がそのまま drain 順に保存され、順序が一意に確定する。
    #[test]
    fn all_variants_preserve_fifo_order() {
        let (tx, rx) = mailbox();

        // GET の応答経路（bounded(1)）は順序確認では使わないが、drop すると
        // exactly-once 経路が閉じるため保持しておく。
        let (get_a, _rx_a) = ActorMsg::get(MailboxRequest::new(11, ""));
        let (get_b, _rx_b) = ActorMsg::get(MailboxRequest::new(13, ""));
        let (stop, _done_rx) = ActorMsg::stop();

        // 種別を意図的に交互配置し、順序が種別ではなく投入順で決まることを示す。
        tx.send(ActorMsg::notify(MailboxRequest::new(10, ""))).unwrap();
        tx.send(get_a).unwrap();
        tx.send(ActorMsg::notify(MailboxRequest::new(12, ""))).unwrap();
        tx.send(get_b).unwrap();
        // Stop は同一 FIFO を通り、先行メッセージを drain 後に処理される（clean drain）。
        tx.send(stop).unwrap();

        let order: Vec<u64> = drain(&rx).iter().map(seq_of).collect();
        assert_eq!(
            order,
            vec![10, 11, 12, 13, u64::MAX],
            "all variants must drain in enqueue order; Stop trails after prior messages"
        );
    }

    /// R6.3/R6.4（単一直列・データ競合排除）: 複数の `Sender` クローンから別スレッドで
    /// 近接して `try_send`（flume の send は実質ノンブロッキング）しても、単一 consumer が
    /// すべてを一意な全順序で逐次 drain する。各 producer 内の投入順は consumer 側でも
    /// 保存され（producer-local FIFO）、同時並行 VM アクセスは構造的に発生しない。
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
                    // seq = producer*1000 + i により、各 producer の投入順が
                    // consumer 側で単調増加として観測できるよう符号化する。
                    let seq = p * 1000 + i;
                    txc.send(ActorMsg::notify(MailboxRequest::new(seq, "")))
                        .expect("clone send must succeed while consumer is alive");
                }
            }));
        }
        for h in handles {
            h.join().expect("producer thread must not panic");
        }
        // 全 producer が投入し終えてから単一 consumer が drain する。
        drop(tx);

        let drained: Vec<u64> = drain(&rx).iter().map(seq_of).collect();

        // (1) 単一直列の証跡: drain 総数 = 投入総数（取りこぼし・重複なし）。
        assert_eq!(
            drained.len() as u64,
            PRODUCERS * PER_PRODUCER,
            "single consumer must receive exactly every enqueued message once"
        );

        // (2) producer-local FIFO: 各 producer 由来のサブ列が投入順（単調増加）で
        //     保存される。これにより「同一 producer の近接投入が投入順＝処理順」を確認。
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

        // (3) 全順序の一意性: drain 列は決定的な 1 本の系列であり、重複インデックスは
        //     存在しない（単一 consumer ゆえ並行処理されない）。
        let mut sorted = drained.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            drained.len(),
            "the single serial FIFO must contain no duplicated messages"
        );
    }

    /// 単一 consumer 不変条件（design.md Revalidation Trigger）: consumer は 1 つの
    /// `Receiver` を所有し、recv は単一経路に閉じる。`Sender` のみ `Clone` でき、
    /// `Receiver` をクローンして複数 recv 経路（`select!` 相当）に分岐させない設計を
    /// 型レベルで示す（`mailbox()` は `Receiver` を 1 つしか返さない）。
    #[test]
    fn sender_is_clonable_receiver_is_single_consumer() {
        let (tx, rx) = mailbox();

        // Sender は Send + Sync + Clone（lock-free static 共有の素地）。
        let tx2 = tx.clone();
        tx.send(ActorMsg::notify(MailboxRequest::new(1, ""))).unwrap();
        tx2.send(ActorMsg::notify(MailboxRequest::new(2, ""))).unwrap();

        // 単一 consumer がすべてを 1 本の FIFO として受ける。
        let order: Vec<u64> = drain(&rx).iter().map(seq_of).collect();
        assert_eq!(order, vec![1, 2], "single consumer receives both clones' sends in order");

        // Sender が Send + Sync であることを静的に確認（static MAILBOX 共有の前提）。
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Sender<ActorMsg>>();
    }

    /// exactly-once（move/drop 意味論・R5.3 の素地）: GET の `reply` をアクター役が
    /// 受け取り `send` すれば値が届き、send せず drop すれば受信側が `Disconnected` を
    /// 観測する（task 3.4 の `recv_timeout`→204 の構造的根拠）。
    #[test]
    fn get_reply_move_then_value_or_disconnected_on_drop() {
        let (tx, rx) = mailbox();

        // ケース A: アクター役が reply を送る → 値が届く。
        let (get_a, reply_rx_a) = ActorMsg::get(MailboxRequest::new(1, ""));
        tx.send(get_a).unwrap();

        // ケース B: アクター役が reply を送らず drop → 受信側は Disconnected。
        let (get_b, reply_rx_b) = ActorMsg::get(MailboxRequest::new(2, ""));
        tx.send(get_b).unwrap();
        drop(tx);

        let drained = drain(&rx);
        for msg in drained {
            // seq==1 は reply を送る。seq==2 は reply を送らず drop（スコープ離脱）→
            // exactly-once の drop 側（受信端が Disconnected を観測する根拠）。
            if let ActorMsg::Get { req, reply } = msg
                && req.seq == 1
            {
                reply.send(Reply::Value("ok".to_string())).unwrap();
            }
        }

        assert_eq!(
            reply_rx_a.recv(),
            Ok(Reply::Value("ok".to_string())),
            "replied GET must deliver its value exactly once"
        );
        assert!(
            reply_rx_b.recv().is_err(),
            "dropped reply sender must surface as Disconnected (the 204 fallback root cause)"
        );
    }
}
