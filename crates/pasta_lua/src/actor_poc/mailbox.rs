//! 単一直列 mailbox（R2.3）。
//!
//! SSP（producer）スレッド → アクター（consumer）スレッド間の単一直列キュー。
//! `enqueue`（SSP 側）／`drain`（アクター側）でスレッドを分離し、VM 操作を
//! drain 側スレッドに閉じ込めることで `mlua` の `!Send` 制約を成立させる
//! 構造的素地を与える（design.md「Mailbox」コンポーネント・R2.3）。
//!
//! 実装は `std::sync::mpsc`（design.md Technology Stack で確定したメカニズム）の
//! 写経であり、新規依存を持ち込まない。mpsc は送信順＝受信順の FIFO を保証する
//! ため、単一 mailbox の直列処理により処理順序が構造的に確定する
//! （design.md「Mailbox / State: FIFO 順序保証」）。

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// mailbox を流れるメッセージの判別共用体。
///
/// SHIORI marshaling の 3 種を表す:
/// - `GetMsg`: GET 同期契約。応答値を SSP スレッドへ返すための reply チャネルを
///   内包する。本タスク（1.4）では FIFO ＋スレッド分離の実証に十分な最小の
///   `std::sync::mpsc::Sender` 1 回送信で reply を表現する。タスク 3.1 が
///   この reply 経路を Drop→204 ガード付きの `Responder` でラップし、応答未送信の
///   まま drop された場合に 204 フォールバックを撃つ保護へ発展させる。
/// - `NotifyMsg`: NOTIFY fire-and-forget。応答を持たず即 204 で終結する経路。
/// - `KickMsg`: talk FIFO へのキック投入。OnSecondChange drain 契機で消費される。
#[derive(Debug)]
pub enum ActorMsg {
    /// GET 同期メッセージ。`payload` は要求識別子相当、`reply` は応答経路。
    Get {
        /// 要求を識別する最小ペイロード（PoC では順序確認に用いる）。
        payload: u64,
        /// 応答値を SSP スレッドへ返す reply チャネル。
        /// 3.1 で Drop→204 ガード付き `Responder` に置き換わる予定の最小素地。
        reply: Sender<u64>,
    },
    /// NOTIFY fire-and-forget メッセージ。応答経路を持たない。
    Notify {
        /// 通知ペイロード（PoC では順序確認に用いる）。
        payload: u64,
    },
    /// talk FIFO へのキックメッセージ。
    Kick {
        /// キック対象シーン識別子相当（PoC では順序確認に用いる）。
        scene: u64,
    },
}

impl ActorMsg {
    /// メッセージ種別に依らず、順序検証用のペイロード値を取り出す。
    ///
    /// FIFO 順序を判定する単体テストが、3 種の variant を一様に並べて
    /// 投入順＝drain 順を確認できるようにするためのアクセサ。
    pub fn payload(&self) -> u64 {
        match self {
            ActorMsg::Get { payload, .. } => *payload,
            ActorMsg::Notify { payload } => *payload,
            ActorMsg::Kick { scene } => *scene,
        }
    }
}

/// SSP（producer）側ハンドル。`enqueue` でメッセージを mailbox に投入する。
///
/// クローン可能（`mpsc::Sender` と同じ意味論）であり、複数の SSP 経路から
/// 同一の単一直列キューへ合流できる。投入順は受信側の drain 順に保存される。
#[derive(Clone)]
pub struct MailboxSender {
    tx: Sender<ActorMsg>,
}

/// アクター（consumer）側ハンドル。drain 側スレッドが所有し、ここで取り出した
/// メッセージに対してのみ VM 操作を行う（VM 操作を drain スレッドに閉じる）。
///
/// `Receiver` は `!Sync` かつ単一所有であり、drain 側スレッドへ move された後は
/// producer 側から触れない。これがスレッド分離（R2.3）の型レベルの担保となる。
pub struct MailboxReceiver {
    rx: Receiver<ActorMsg>,
}

/// 単一直列 mailbox を生成し、producer 側 (`MailboxSender`) と
/// consumer 側 (`MailboxReceiver`) のペアを返す。
pub fn mailbox() -> (MailboxSender, MailboxReceiver) {
    let (tx, rx) = mpsc::channel();
    (MailboxSender { tx }, MailboxReceiver { rx })
}

impl MailboxSender {
    /// SSP（producer）側からメッセージを mailbox に投入する。
    ///
    /// 投入順は mpsc の FIFO 保証により drain 側でそのまま保存される。
    /// consumer 側が既に drop されている場合は投入したメッセージを `Err` として
    /// 返す（送り先消失）。
    pub fn enqueue(&self, msg: ActorMsg) -> Result<(), ActorMsg> {
        self.tx.send(msg).map_err(|e| e.0)
    }
}

impl MailboxReceiver {
    /// 現在キューに溜まっているメッセージを投入順（FIFO）にすべて取り出す。
    ///
    /// アクタースレッド上で呼ばれることを意図する。ここで取り出した
    /// メッセージに対する処理（VM 操作）は呼び出しスレッド＝drain スレッドに
    /// 閉じる。producer 側がまだ生きていてもブロックせず、現時点で利用可能な
    /// 分だけを返す（OnSecondChange のような周期 drain 契機を模す）。
    pub fn drain(&self) -> Vec<ActorMsg> {
        let mut out = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(msg) => out.push(msg),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        out
    }

    /// 次の 1 メッセージを投入順にブロッキング受信する。
    ///
    /// drain 側スレッドが「メッセージが来るまで待つ」ループを組む場合に用いる。
    /// producer 側がすべて drop されキューが空になると `None` を返す（終端）。
    pub fn recv(&self) -> Option<ActorMsg> {
        self.rx.recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc as std_mpsc;
    use std::thread;

    /// `!Send` を表すマーカー。VM 所有権の代理として drain 側スレッドが構築・保持し、
    /// チャネルを越えて producer 側へ戻らないことを型レベルで担保する。
    /// `mlua::Lua` と同様に `!Send`（raw pointer を保持）であり、もし drain クロージャを
    /// 誤って producer スレッドへ移そうとすればコンパイルが通らなくなる。
    struct VmMarker {
        thread_id: thread::ThreadId,
        _not_send: std::marker::PhantomData<*const ()>,
    }

    impl VmMarker {
        /// 構築したスレッドの id を捕捉する。これにより「どのスレッドで VM が
        /// 生成・操作されたか」を後から検証できる。
        fn new() -> Self {
            VmMarker {
                thread_id: thread::current().id(),
                _not_send: std::marker::PhantomData,
            }
        }
    }

    /// R2.3（FIFO 順序）: enqueue 順 1..=N が drain でそのままの順序で取り出される。
    #[test]
    fn drains_in_enqueue_order() {
        let (tx, rx) = mailbox();
        const N: u64 = 100;

        for i in 1..=N {
            tx.enqueue(ActorMsg::Notify { payload: i })
                .expect("enqueue should succeed while receiver is alive");
        }
        // producer 側を drop して drain がキュー終端を検出できるようにする。
        drop(tx);

        let drained: Vec<u64> = rx.drain().iter().map(ActorMsg::payload).collect();
        let expected: Vec<u64> = (1..=N).collect();
        assert_eq!(
            drained, expected,
            "messages must be drained in the exact order they were enqueued (FIFO)"
        );
    }

    /// R2.3（FIFO・全 variant）: Get/Notify/Kick の 3 種を混在投入しても、
    /// 種別に依らず投入順がそのまま drain 順に保存される。
    #[test]
    fn all_variants_preserve_fifo_order() {
        let (tx, rx) = mailbox();
        let (reply_tx, _reply_rx) = std_mpsc::channel::<u64>();

        // 種別を意図的に交互配置し、順序が種別ではなく投入順で決まることを示す。
        tx.enqueue(ActorMsg::Kick { scene: 10 }).unwrap();
        tx.enqueue(ActorMsg::Get { payload: 11, reply: reply_tx.clone() }).unwrap();
        tx.enqueue(ActorMsg::Notify { payload: 12 }).unwrap();
        tx.enqueue(ActorMsg::Get { payload: 13, reply: reply_tx }).unwrap();
        tx.enqueue(ActorMsg::Kick { scene: 14 }).unwrap();
        drop(tx);

        let order: Vec<u64> = rx.drain().iter().map(ActorMsg::payload).collect();
        assert_eq!(
            order,
            vec![10, 11, 12, 13, 14],
            "all three variants must drain in enqueue order regardless of variant type"
        );
    }

    /// R2.3（スレッド分離）: enqueue は producer スレッド、drain は別の spawn 済み
    /// スレッドで実行される。VM 所有権を表す `!Send` マーカーは drain スレッドで
    /// 構築・保持され、producer スレッドへは決して戻らない。
    ///
    /// 正直さの担保: drain クロージャ内で `thread::current().id()` を捕捉し、それが
    /// producer スレッドの id と**異なる**ことを assert する。もし drain が（誤って）
    /// producer スレッド上で実行されていれば、この assert は失敗する。
    #[test]
    fn drain_runs_on_separate_thread_and_vm_marker_stays_there() {
        let (tx, rx) = mailbox();
        let producer_id = thread::current().id();

        // GET の reply 経路で、drain スレッドが「VM 操作」を行ったスレッド id を
        // producer スレッドへ返してもらう（値だけが戻り、VM マーカー自体は越えない）。
        let (reply_tx, reply_rx) = std_mpsc::channel::<u64>();
        tx.enqueue(ActorMsg::Get { payload: 1, reply: reply_tx }).unwrap();
        for i in 2..=5u64 {
            tx.enqueue(ActorMsg::Notify { payload: i }).unwrap();
        }
        drop(tx);

        // drain は別スレッドで実行。`rx` と `VmMarker` はこのスレッドに閉じる。
        let consumer = thread::spawn(move || {
            // VM 所有権の代理（`!Send`）を drain スレッド内で構築。これは move
            // クロージャの外（producer スレッド）へは構造的に出せない。
            let vm = VmMarker::new();
            let drain_thread_id = thread::current().id();
            assert_eq!(
                vm.thread_id, drain_thread_id,
                "VM marker must be constructed on the drain thread"
            );

            let msgs = rx.drain();
            // 投入順（FIFO）が drain スレッドでも保存されること。
            let order: Vec<u64> = msgs.iter().map(ActorMsg::payload).collect();
            assert_eq!(order, vec![1, 2, 3, 4, 5], "FIFO order on drain thread");

            // GET の reply に、VM 操作を実行したスレッド id を返す（値のみ越境）。
            for m in &msgs {
                if let ActorMsg::Get { reply, .. } = m {
                    reply
                        .send(drain_thread_id_as_u64(drain_thread_id))
                        .expect("reply should reach the blocked producer");
                }
            }
            drain_thread_id
        });

        // producer 側: GET 応答（drain スレッド id 由来の値）を block-on-reply で受信。
        let replied = reply_rx.recv().expect("GET reply must arrive");
        let drain_thread_id = consumer.join().expect("consumer thread should not panic");

        // drain は producer とは別スレッドで実行された（スレッド分離）。
        assert_ne!(
            drain_thread_id, producer_id,
            "drain must run on a thread distinct from the producer/enqueue thread"
        );
        // reply で返ってきた id が drain スレッド由来であること（VM 操作が drain
        // 側に閉じていた証跡）。
        assert_eq!(
            replied,
            drain_thread_id_as_u64(drain_thread_id),
            "the GET reply value must originate from the drain thread's VM op"
        );
    }

    /// `ThreadId` は安定した数値化 API を公開していないため、Debug 表現を介して
    /// 比較可能な u64 を作る（テスト内の reply 値として越境させる用途に限る）。
    fn drain_thread_id_as_u64(id: thread::ThreadId) -> u64 {
        // ThreadId の Debug は "ThreadId(N)" 形式。N を抽出して数値化する。
        let s = format!("{id:?}");
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse().unwrap_or(0)
    }
}
