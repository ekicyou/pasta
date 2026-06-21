//! アクタースレッド（R1 本丸 / task 2.1）。
//!
//! `std::thread::spawn` 内で `wintf_winmsg_executor::block_on(actor future)` を回し、
//! その future が **実 `PastaLuaRuntime`（`!Send` mlua VM）** を生成・所有して Lua を
//! 実行する。VM はこのアクタースレッドを越えない（`mlua::Lua` の `!Send` 制約を
//! コンパイル時に強制）。`JoinHandle` を保持し、`Arc<AtomicBool>` shutdown フラグ →
//! `JoinHandle::join` の teardown idiom を debug backend（`debug/handle.rs` の
//! `DebugHandle::Drop`）から写経する。
//!
//! # `wintf_winmsg_executor` 0.0.3 の確認済み API（registry ソース実読）
//!
//! - `block_on<'a, T: 'a>(future: impl Future<Output = T> + 'a) -> T`
//!   — 呼び出しスレッドの Windows メッセージループを回し、`!Send` かつ非 `'static` な
//!     future を完了まで駆動して戻り値を返す（内部は `async_task::spawn_unchecked`）。
//!     `examples/threads.rs` が `std::thread::spawn` 内での `block_on` を実証済み。
//! - 再ポーリングは executor 内部の `MSG_ID_WAKE` メッセージ（`Waker::wake`）で駆動される。
//!   producer 側はコマンドを送ったあとに保持済みの `Waker` を起こすことで、CPU スピンを
//!   伴わずにアクタースレッドのメッセージループを再ポーリングへ進められる。
//!
//! # VM ホスティングの選択（R1 証拠強度）
//!
//! design.md「Allowed Dependencies / Upstream」の方針どおり **実 `PastaLuaRuntime`**
//! を future 内で生成・所有する。`PastaLuaRuntime::new(TranspileContext::new())` は
//! 重いフィクスチャ/スクリプトを要さずに完全な VM を構築できるため（`context.rs` の
//! `TranspileContext: Default`、`runtime/mod.rs` の `new`）、スコープ縮小（`mlua::Lua`
//! 直ホストへの代替）は不要。これにより R1.1 の証拠は最強（実 VM ホスト）になる。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::{self, JoinHandle, ThreadId};

use super::mailbox::{mailbox, ActorMsg, MailboxReceiver, MailboxSender};
use super::responder::PocResponse;
use crate::context::TranspileContext;
use crate::runtime::PastaLuaRuntime;

/// アクタースレッド上で 1 回の Lua 実行を行った結果。
///
/// VM 本体は越境せず、ここに含まれる **値だけ** が reply チャネルで SSP/test
/// スレッドへ戻る（R2.3 スレッド分離の証跡）。
#[derive(Debug, Clone, Copy)]
pub struct LuaProbeOutcome {
    /// Lua が評価した数値結果（PoC では `return <int>` を実行）。
    pub lua_result: i64,
    /// Lua を実際に実行したスレッドの id（＝アクタースレッド id のはず）。
    pub exec_thread_id: ThreadId,
}

/// アクター future に投入するコマンド（mailbox の最小形）。
///
/// 本タスク（2.1）は VM ホスト＋Lua 実行の実証に必要な最小コマンドのみを持つ。
/// 完全な GET/NOTIFY marshaling は task 3.2、reload teardown は task 2.2 の責務。
enum ActorCommand {
    /// Lua スニペットをアクタースレッド上の VM で実行し、(結果, 実行スレッド id) を
    /// reply で返す。
    RunLua {
        /// 実行する Lua スニペット（`return <expr>` を想定）。
        script: String,
        /// 結果（または実行エラー文字列）を 1 回だけ返す reply チャネル。
        reply: Sender<Result<LuaProbeOutcome, String>>,
    },
}

/// アクタースレッドのハンドル。
///
/// producer（SSP/test）側が保持し、コマンド送信・アクタースレッド id 参照・
/// teardown を行う。VM 本体はこのハンドルには載らない（アクタースレッドに pin）。
pub struct ActorThread {
    /// アクター future へコマンドを送る mailbox 送信端。
    cmd_tx: Sender<ActorCommand>,
    /// task 1.4 の単一直列 [`Mailbox`](super::mailbox) 送信端（GET/NOTIFY marshaling 用）。
    ///
    /// `RunLua` コマンド経路（task 2.1 の R1 プローブ）とは別に、SHIORI marshaling
    /// （[`ActorMsg::Get`]／[`ActorMsg::Notify`]）を同一のアクタースレッド＝同一の
    /// `!Send` VM へ流すための加算的な経路。`ffi_marshal`（task 3.2）がここへ enqueue
    /// する。actor future は両経路を同じ poll で drain するため、VM はアクター
    /// スレッドに pin されたまま GET/NOTIFY を処理する（R2.3 スレッド分離）。
    mailbox_tx: MailboxSender,
    /// アクター future の最初のポーリングで捕捉した `Waker`。コマンド送信後に
    /// `wake()` してメッセージループを再ポーリングへ進める（スピン回避）。
    /// `Waker` は `Send + Sync`（`async_task` 由来）なので producer 側から起こせる。
    waker: Arc<Mutex<Option<Waker>>>,
    /// debug backend の shutdown idiom 写経（`debug/handle.rs`）。立てると future が
    /// 完了し block_on が戻る。
    shutdown: Arc<AtomicBool>,
    /// アクタースレッドの `JoinHandle`（teardown 時に join する）。
    join_handle: Option<JoinHandle<()>>,
    /// アクタースレッドの id（spawn 時にスレッド自身が報告した値）。
    actor_thread_id: ThreadId,
}

/// teardown（shutdown→join）の結果レポート。
#[derive(Debug, Clone, Copy)]
pub struct TeardownReport {
    /// `JoinHandle::join` が panic なく成功したか（clean teardown）。
    pub joined_cleanly: bool,
    /// join したアクタースレッドの id（VM をホストしていたスレッド）。
    pub actor_thread_id: ThreadId,
}

impl ActorThread {
    /// アクタースレッドを起動する。
    ///
    /// `std::thread::spawn` 内で `block_on(actor future)` を回す。future は実
    /// `PastaLuaRuntime`（`!Send` VM）を生成・所有し、shutdown フラグが立つまで
    /// コマンドを処理し続ける。VM はこのスレッドを越えない。
    ///
    /// スレッド起動と同時にアクタースレッド id を受け取り、ハンドルに保持する。
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<ActorCommand>();
        let (mailbox_tx, mailbox_rx) = mailbox();
        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));

        // アクタースレッド id を spawn 後に確実に取得するための oneshot。
        let (id_tx, id_rx) = mpsc::channel::<ThreadId>();

        let thread_waker = Arc::clone(&waker);
        let thread_shutdown = Arc::clone(&shutdown);

        let join_handle = thread::Builder::new()
            .name("pasta-actor-poc".to_string())
            .spawn(move || {
                // アクタースレッド id を producer へ通知（block_on 突入前）。
                let _ = id_tx.send(thread::current().id());

                // ここで block_on が呼び出しスレッド（＝このアクタースレッド）の
                // メッセージループを回す。actor future は !Send VM を所有する。
                wintf_winmsg_executor::block_on(actor_future(
                    cmd_rx,
                    mailbox_rx,
                    thread_waker,
                    thread_shutdown,
                ));
            })
            .expect("actor thread must spawn");

        let actor_thread_id = id_rx
            .recv()
            .expect("actor thread must report its ThreadId before block_on");

        ActorThread {
            cmd_tx,
            mailbox_tx,
            waker,
            shutdown,
            join_handle: Some(join_handle),
            actor_thread_id,
        }
    }

    /// アクタースレッドの id（VM が pin されているスレッド）。
    pub fn actor_thread_id(&self) -> ThreadId {
        self.actor_thread_id
    }

    /// task 1.4 の [`Mailbox`](super::mailbox) 送信端を複製して返す。
    ///
    /// SHIORI marshaling 側（`ffi_marshal` / task 3.2）が GET/NOTIFY を同一の
    /// アクタースレッド＝同一の `!Send` VM へ enqueue するための加算的アクセサ。
    /// `MailboxSender` は `Clone`（mpsc 意味論）なので、複数の経路から単一直列
    /// キューへ合流できる。
    pub fn mailbox_sender(&self) -> MailboxSender {
        self.mailbox_tx.clone()
    }

    /// [`ActorMsg`] を mailbox に投入し、アクター future を起こす（加算的経路）。
    ///
    /// GET は [`ActorMsg::Get`] の `responder` を内包させて enqueue し、呼び出し側
    /// （SSP/test）は対応する `Receiver<PocResponse>` を block-on-reply する。NOTIFY は
    /// [`ActorMsg::Notify`] を enqueue し即制御を返す（fire-and-forget）。投入後に
    /// 保持済み `Waker` を起こしてメッセージループを再ポーリングへ進める（スピン回避）。
    ///
    /// アクタースレッドが既に teardown 済みでキューの受信端が drop されている場合は
    /// 投入したメッセージを `Err` として返す（GET の `responder` も `Err` に同梱されて
    /// 返り、呼び出し側が drop すれば Drop→204 ガードが撃たれる）。
    pub fn submit(&self, msg: ActorMsg) -> Result<(), ActorMsg> {
        self.mailbox_tx.enqueue(msg)?;
        self.wake_actor();
        Ok(())
    }

    /// Lua スニペットをアクタースレッド上の VM で実行し、(結果, 実行スレッド id) を
    /// block-on-reply で受け取る。
    ///
    /// 値だけが越境し、VM 本体は越境しない。コマンド送信後に保持済み `Waker` を
    /// 起こしてメッセージループを再ポーリングへ進める。
    pub fn run_lua_probe(&self, script: &str) -> Result<LuaProbeOutcome, String> {
        let (reply_tx, reply_rx) = mpsc::channel::<Result<LuaProbeOutcome, String>>();
        self.cmd_tx
            .send(ActorCommand::RunLua {
                script: script.to_string(),
                reply: reply_tx,
            })
            .map_err(|_| "actor thread is no longer accepting commands".to_string())?;

        // 送信したコマンドを処理させるため、アクター future を起こす。
        self.wake_actor();

        // block-on-reply: アクタースレッドが結果を返すまでブロックする。
        reply_rx
            .recv()
            .map_err(|_| "actor thread dropped the reply channel".to_string())?
    }

    /// アクター future のメッセージループを再ポーリングへ進める。
    ///
    /// 最初のポーリングで `Waker` が捕捉されている前提（spawn 直後に block_on が
    /// 一度ポーリングしている）。万一未捕捉でも、future 側は各ポーリングで
    /// `Waker` を更新するため、次の機会に拾われる。
    fn wake_actor(&self) {
        if let Some(w) = self.waker.lock().expect("waker mutex").as_ref() {
            w.wake_by_ref();
        }
    }

    /// shutdown フラグを立て、`JoinHandle` を join して teardown する（写経:
    /// `DebugHandle::Drop` の「フラグ→join」idiom）。
    ///
    /// 単一 teardown を保証するため `self` を消費する。
    pub fn shutdown(mut self) -> TeardownReport {
        let report = self.tear_down();
        report.expect("shutdown must perform exactly one teardown")
    }

    /// 内部 teardown 本体（`shutdown` と `Drop` の双方から呼ばれるが、`join_handle`
    /// を `take` するため二重 join しない）。
    fn tear_down(&mut self) -> Option<TeardownReport> {
        let join_handle = self.join_handle.take()?;

        // (1) shutdown フラグを立てる（future がこれを観測して完了→block_on が戻る）。
        self.shutdown.store(true, Ordering::SeqCst);
        // (2) フラグを観測させるため future を起こす（コマンドが無くても再ポーリング）。
        self.wake_actor();
        // (3) 同期 join: アクタースレッドの block_on 完了＝VM Drop までブロック。
        let joined_cleanly = join_handle.join().is_ok();

        Some(TeardownReport {
            joined_cleanly,
            actor_thread_id: self.actor_thread_id,
        })
    }
}

impl Drop for ActorThread {
    fn drop(&mut self) {
        // `shutdown` を呼ばずに drop された場合の保険 teardown（写経: Drop で
        // フラグ→join）。`shutdown` 済みなら `join_handle` は既に `take` 済みで no-op。
        let _ = self.tear_down();
    }
}

/// アクター future の本体。**このスレッド（アクタースレッド）上でのみ** 実行され、
/// `!Send` な `PastaLuaRuntime` を生成・所有する。
///
/// `async fn` だが I/O 待ちは行わず、各ポーリングで:
/// 1. 最初に `Waker` を捕捉/更新（producer が起こせるように）、
/// 2. shutdown フラグを確認（立っていれば完了して return → block_on が戻る）、
/// 3. mailbox に溜まったコマンドを drain して VM 上で処理、
/// 4. まだ続くなら `Pending` を返してメッセージループへ制御を返す。
///
/// VM はこの future のローカル変数として所有されるため、`mlua::Lua` の `!Send`
/// 制約により **構造的に** このスレッドを越えられない（越えようとすればコンパイル不可）。
async fn actor_future(
    cmd_rx: Receiver<ActorCommand>,
    mailbox_rx: MailboxReceiver,
    waker_slot: Arc<Mutex<Option<Waker>>>,
    shutdown: Arc<AtomicBool>,
) {
    // !Send VM をアクタースレッド上で生成・所有（R1.1）。重いフィクスチャ不要の
    // 実 PastaLuaRuntime（context.rs / runtime/mod.rs）。
    let runtime = PastaLuaRuntime::new(TranspileContext::new())
        .expect("PastaLuaRuntime must construct on the actor thread");

    // 各ポーリングでコマンド drain ＋ shutdown 監視を行う poll_fn ループ。
    std::future::poll_fn(move |cx: &mut Context<'_>| {
        // (1) producer がアクターを起こせるよう、最新の Waker を保存。
        {
            let mut slot = waker_slot.lock().expect("waker mutex");
            *slot = Some(cx.waker().clone());
        }

        // (2) shutdown 観測 → 完了（block_on が戻り、runtime=VM が Drop される）。
        if shutdown.load(Ordering::SeqCst) {
            return Poll::Ready(());
        }

        // (3a) R1 プローブ用 `RunLua` コマンド経路を drain（task 2.1 既存）。
        loop {
            match cmd_rx.try_recv() {
                Ok(ActorCommand::RunLua { script, reply }) => {
                    let outcome = run_lua_on_vm(&runtime, &script);
                    // 値（または文字列エラー）だけが越境する。VM は越えない。
                    let _ = reply.send(outcome);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    // producer が全 sender を drop した。これ以上コマンドは来ない。
                    // shutdown 同様に完了させ、block_on を戻す（teardown）。
                    return Poll::Ready(());
                }
            }
        }

        // (3b) SHIORI marshaling 用 task 1.4 mailbox を投入順（FIFO）に drain して
        //      VM 上で処理する（task 3.2 が enqueue する GET/NOTIFY/Kick）。VM 操作は
        //      この drain＝アクタースレッドに閉じる（R2.3）。GET は `Responder` で
        //      reply（VM 失敗時は reply せず drop→204 ガード）。NOTIFY は reply 無し
        //      fire-and-forget。Kick は本タスクの責務外（task 5 で消費）なので無視。
        for msg in mailbox_rx.drain() {
            match msg {
                ActorMsg::Get {
                    script, responder, ..
                } => match run_lua_on_vm(&runtime, &script) {
                    // 応答値には VM 実行スレッド id 由来の数値を載せる（R2.3 の証跡）。
                    Ok(outcome) => {
                        responder.reply(PocResponse::Value(thread_id_digits(outcome.exec_thread_id)))
                    }
                    // VM 失敗＝応答未送信扱い。responder を drop すると Drop→204 ガードが
                    // 撃たれ、SSP 側の block-on-reply は無限待機せず 204 で終結（R3.1/R2.4）。
                    Err(_) => drop(responder),
                },
                ActorMsg::Notify { script, .. } => {
                    // fire-and-forget: VM を実行するが応答経路を持たない。
                    let _ = run_lua_on_vm(&runtime, &script);
                }
                ActorMsg::Kick { .. } => {
                    // talk FIFO のキックは task 5（KickHarness/SimDriver）の責務。
                    // 本アクターループは marshaling（GET/NOTIFY）に限定する。
                }
            }
        }

        // (4) まだ続く。メッセージループへ制御を返す（次の wake で再ポーリング）。
        Poll::Pending
    })
    .await;
}

/// `ThreadId` の Debug 表現（"ThreadId(N)"）から N を抽出して u64 にする。
///
/// `ThreadId` は安定した数値化 API を公開していないため、`PocResponse::Value` に
/// 載せてチャネルを越境させる用途に限り Debug 表現を介す（R2.3 の証跡値）。
pub fn thread_id_digits(id: ThreadId) -> u64 {
    let s = format!("{id:?}");
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap_or(0)
}

/// アクタースレッド上の VM で Lua を実行し、結果と実行スレッド id を集約する。
///
/// この関数は `actor_future` 内＝アクタースレッド上でのみ呼ばれる。`runtime` は
/// `!Send` なので、この関数自体も別スレッドへは移せない（型レベル保証）。
fn run_lua_on_vm(runtime: &PastaLuaRuntime, script: &str) -> Result<LuaProbeOutcome, String> {
    let value = runtime
        .exec(script)
        .map_err(|e| format!("Lua exec failed on actor thread: {e}"))?;
    let lua_result = value
        .as_i64()
        .ok_or_else(|| format!("Lua probe must return an integer, got: {value:?}"))?;
    Ok(LuaProbeOutcome {
        lua_result,
        exec_thread_id: thread::current().id(),
    })
}
