//! 本番アクタースレッド（task 3.3・design.md「ActorThread」コンポーネント）。
//!
//! `!Send` な Lua VM（[`PastaShiori`] が内包する [`PastaLuaRuntime`]）を **専用 OS
//! スレッドへ pin** し、そのスレッド上の wintf メッセージループ（`block_on`）で
//! 単一の `recv_async().await` ループを回して mailbox（task 3.2）を消費する。
//! 各メッセージで実 `SHIORI.request` Function を VM 上で呼び、既存コルーチン
//! （`co_scene`／`resume_until_valid`／`CALLBACK`）を **無改変** に executor 駆動下で
//! resume・継続させる。
//!
//! # なぜ VM をスレッドへ pin し、生リクエスト文字列を mailbox で運ぶのか
//! [`PastaLuaRuntime`]（mlua）は `!Send` であり、SHIORI スレッドとアクタースレッドの
//! 間をチャネルで越境できない。そこで mailbox（[`ActorMsg`]）が運ぶのは `Send` な生の
//! SHIORI リクエスト文字列（[`MailboxRequest::raw`]）のみとし、**VM スレッド側でパース
//! してテーブル化**する（design.md `LuaRequestTable` は VM 同居表現）。VM はこのスレッド
//! で生成され、ここを一度も離れない。
//!
//! # コルーチン意味論の不変保存
//! 本スレッドは出荷経路と同一の [`PastaShiori::request`] をそのまま呼ぶ。`co_scene`
//! はメッセージ間で VM 内に persist するため、`resume_until_valid` / `CALLBACK` は
//! tick から tick へ継続する（GET property → callback resume の 2 ラウンド等）。
//! executor は VM を駆動するだけでコルーチン状態には一切干渉しない。
//!
//! # 単一 consumer 不変条件
//! 消費は単一の非 cancel `recv_async().await` ループのみで行い、`select!` を張らない
//! （mailbox.rs のドキュメント参照）。`Stop` は同一 FIFO 上で先行メッセージを drain
//! 後に処理され、ループを抜けて `block_on` が戻る。
//!
//! # 配線（task 5.1 で完了）
//! FFI 入口（`windows.rs` → [`crate::actor::lifecycle`] の `static MAILBOX`）が本スレッドへ
//! 配線済み。GET の完全な timeout/204/drop marshaling 契約は [`crate::actor::marshaling`]
//! （task 3.4）が担い、本スレッドの `Get` 応答は VM 応答文字列を [`Reply::Value`] で返す
//! （エラー時は reply を drop → marshaling 側で 204）。wintf（`block_on`）は本スレッドの
//! メッセージループに必須のため出荷依存である。

use std::net::SocketAddr;
use std::path::PathBuf;
use std::thread::{self, JoinHandle, ThreadId};

use flume::Receiver;

use crate::actor::mailbox::{ActorMsg, Reply};
use crate::shiori::{PastaShiori, Shiori};

/// アクタースレッドのハンドル。
///
/// VM はこのスレッドに pin され、決して越境しない。SHIORI スレッドは mailbox
/// （[`ActorMsg`]）越しにのみ VM へアクセスする。
pub struct ActorThread {
    /// VM をホストする専用 OS スレッドの join ハンドル。
    handle: Option<JoinHandle<()>>,
    /// VM を実行するスレッドの id（SHIORI スレッドと別であることの観測点・R4.2/R4.5）。
    actor_thread_id: ThreadId,
    /// ロード成否（`SHIORI.load` 成功＝true）。失敗時もスレッドは drain を続ける
    /// （request は NotInitialized/Load エラーを返す）。
    loaded: bool,
    /// アクタースレッド上で `debug::enable` が束縛した DAP リスナアドレス（R10.1/R10.2）。
    ///
    /// debug backend が **アクタースレッド上で**有効化された（＝`set_global_hook` が
    /// アクタースレッドに装着された）ことの観測点。`pasta.toml [debug] enabled=true`
    /// （port=0 でエフェメラル）のときのみ `Some`。debug 無効（既定・zero-cost）では
    /// `None`（ポートを開かない・R5.5）。VM 本体は越境しないが、束縛アドレス
    /// （`SocketAddr` は `Copy`・`Send`）はアクタースレッドから読み戻して観測できる。
    debug_local_addr: Option<SocketAddr>,
}

impl ActorThread {
    /// VM を実行するアクタースレッドの id を返す（R4.2/R4.5 の観測点）。
    pub fn actor_thread_id(&self) -> ThreadId {
        self.actor_thread_id
    }

    /// `SHIORI.load` がアクタースレッド上で成功したか。
    pub fn loaded(&self) -> bool {
        self.loaded
    }

    /// アクタースレッド上で debug backend が束縛した DAP リスナアドレス（R10.1/R10.2）。
    ///
    /// `Some(addr)` は `debug::enable` がアクタースレッド上で成立し、`set_global_hook`
    /// がアクタースレッドに装着された証跡（VSCode は `addr` へ attach する）。debug 無効
    /// （既定）では `None`（ポート不開放・zero-cost・R5.5）。
    pub fn debug_local_addr(&self) -> Option<SocketAddr> {
        self.debug_local_addr
    }

    /// アクタースレッドを join する（テスト／teardown 用）。
    ///
    /// 呼び出し側は事前に [`ActorMsg::Stop`] を mailbox へ送り、ループを抜けさせて
    /// おくこと（さもなくば `recv_async().await` が永久に Pending で join できない）。
    pub fn join(mut self) -> thread::Result<()> {
        match self.handle.take() {
            Some(h) => h.join(),
            None => Ok(()),
        }
    }

    /// アクタースレッドを **detach** する（本番 teardown 用・design.md「Teardown」）。
    ///
    /// 本番 teardown は `JoinHandle` で join せず、`ActorMsg::Stop { done }` の done ack
    /// で完了を確認する（ack 受信時に VM 破棄・cleanup 完了済み）。本メソッドは
    /// `JoinHandle` を join せずに drop してスレッドを detach する。`take()` 二重 join の
    /// 小細工も持たない（join 自体を廃止）。`done` ack を受け取った後に呼ぶこと。
    ///
    /// OS スレッドは block_on 完了後に自然終了し、ハンドルは drop で解放される
    /// （reload リーク検査がスレッドハンドルの非リークを実測で裏取りする）。
    pub fn detach(mut self) {
        // JoinHandle を drop すると Rust はスレッドを detach する（join しない）。
        // done ack 済みなので block_on は既に戻っており、スレッドは終了済みか終了間際。
        drop(self.handle.take());
    }
}

/// 専用アクタースレッドを起動し、その上で `!Send` VM を生成・pin して mailbox を
/// 駆動する。
///
/// # 引数
/// - `hinst` / `load_dir`: VM を構築する [`PastaShiori::load`] へ渡す（どちらも `Send`）。
///   VM 本体（`!Send`）はこのスレッド上で構築されるため越境しない。
/// - `rx`: mailbox の consumer 側 [`Receiver`]（単一 consumer 不変条件）。
///
/// # 動作
/// スレッド上で `wintf_winmsg_executor::block_on(async { ... })` を回す。future は
/// VM を構築後、`while let Ok(msg) = rx.recv_async().await` の単一ループで mailbox を
/// 消費する。flume の native Waker が別スレッドの `try_send` で wintf メッセージ
/// ループを起こす（task 3.1 で実証済み・手動 wake なし）。
///
/// # 戻り値
/// [`ActorThread`]。`actor_thread_id()` で VM 実行スレッド id、`loaded()` でロード
/// 成否を観測できる。`join()` 前に [`ActorMsg::Stop`] を送ること。
pub fn spawn_actor_thread(
    hinst: isize,
    load_dir: PathBuf,
    rx: Receiver<ActorMsg>,
) -> ActorThread {
    // VM 実行スレッド id・ロード成否・debug DAP 束縛アドレスを呼び出し側へ返す小チャネル
    // （いずれも `Send` な値のみ越境・VM 本体は越境しない）。`SocketAddr` は `Copy`。
    let (ready_tx, ready_rx) =
        flume::bounded::<(ThreadId, bool, Option<SocketAddr>)>(1);

    let handle = thread::Builder::new()
        .name("pasta-actor".to_string())
        .spawn(move || {
            // このスレッドの Windows メッセージループ上で future を完走させる。
            // future は `!Send` な PastaShiori を所有し、recv_async().await のみで待機。
            wintf_winmsg_executor::block_on(async move {
                // (1) VM をこのスレッドで構築・pin（!Send ゆえ越境不可）。
                let mut shiori = PastaShiori::default();
                let loaded = shiori
                    .load(hinst, load_dir.as_os_str())
                    .unwrap_or(false);

                // debug backend がアクタースレッド上で束縛した DAP アドレスを観測する
                // （R10.1/R10.2）。`debug::enable` は VM 構築（`load` 内）の一部として
                // **このアクタースレッド上で**呼ばれるため、`set_global_hook` はこの
                // スレッドに装着される。`Some` はその成立証跡（debug 無効なら `None`）。
                let debug_local_addr = shiori
                    .runtime()
                    .and_then(|r| r.debug_local_addr());

                // 観測ログ点（R10.4・無効時ゼロコスト）: アクタースレッド起動・VM pin 完了。
                tracing::debug!(seam = "actor.spawn", loaded, "actor thread spawned; VM pinned on this thread");

                // 実行スレッド id・ロード成否・debug 束縛アドレスを返す（VM 本体は越境しない）。
                let _ = ready_tx.send((thread::current().id(), loaded, debug_local_addr));

                // (2) 単一 recv ループ（select! なし・手動 wake なし）。
                //     Stop で break → cleanup → done ack → async ブロック完了 → block_on が戻る。
                //     done ack を持ち帰り、ループ脱出後（＝先行メッセージ drain 後）に
                //     VM 破棄・cleanup を終えてから ack を送る（task 4.1・R7.1/R7.4:
                //     ack 受信時に全資源解放済みを保証）。
                let mut done_ack: Option<flume::Sender<()>> = None;
                while let Ok(msg) = rx.recv_async().await {
                    match msg {
                        // GET 同期: 実 SHIORI.request を VM 上で呼び、応答値を返す。
                        // co_scene は VM 内に persist し、resume_until_valid/CALLBACK が
                        // メッセージ間で継続する（コルーチン意味論不変）。
                        ActorMsg::Get { req, reply } => {
                            let seq = req.seq;
                            // 観測ログ点（R10.4・無効時ゼロコスト）: GET を mailbox から受信。
                            tracing::trace!(seam = "actor.recv", method = "get", seq, "actor received GET");
                            // task 3.4 申し送り: 完全な timeout/drop→204 marshaling は 3.4。
                            // ここでは VM 応答文字列を Reply::Value で返す最小実装。
                            // エラー時は reply を送らず drop（受信側 Disconnected→3.4 で 204）。
                            match shiori.request(&req.raw) {
                                Ok(resp) => {
                                    let _ = reply.send(Reply::Value(resp));
                                    // 観測ログ点（R10.4）: VM 応答を reply で返した（exactly-once move）。
                                    tracing::trace!(seam = "actor.reply", method = "get", seq, "actor replied with VM response");
                                }
                                Err(_) => {
                                    // reply を送らず drop（受信側 Disconnected→marshaling で 204）。
                                    drop(reply);
                                    // 観測ログ点（R10.4）: VM 失敗で reply を drop（exactly-once drop）。
                                    tracing::debug!(seam = "actor.drop", method = "get", seq, "actor dropped reply (VM request failed) -> 204");
                                }
                            }
                        }
                        // NOTIFY fire-and-forget: VM 上で処理し応答経路は持たない。
                        // 応答は捨てる（NOTIFY は即 204 で終結する・3.4 で契約確定）。
                        ActorMsg::Notify { req } => {
                            // 観測ログ点（R10.4）: NOTIFY を mailbox から受信（応答経路なし）。
                            tracing::trace!(seam = "actor.recv", method = "notify", seq = req.seq, "actor received NOTIFY");
                            let _ = shiori.request(&req.raw);
                        }
                        // KICK fire-and-forget（仕様 `pasta-scene-kick` 3.1）: VM 上で
                        // SHIORI.kick(scene) を呼び、保留フラグ（kick_pending/kick_force）を
                        // 設置する。NOTIFY 同型で応答経路を持たず、戻り値も捨てる。複数 Kick
                        // は同一 FIFO 上で投入順に処理され、最後の scene が kick_pending を
                        // 上書きする。kick は protected call（shiori.rs 側で pcall 相当）ゆえ
                        // SHIORI.kick 不在・エラーでもアクタースレッドは落ちない。
                        ActorMsg::Kick { scene } => {
                            // 観測ログ点（R10.4・無効時ゼロコスト）: Kick を mailbox から受信。
                            tracing::trace!(seam = "actor.recv", method = "kick", scene = %scene, "actor received KICK");
                            shiori.kick(&scene);
                        }
                        // teardown: ack 経路を保持してループを抜ける。Stop は同一 FIFO を
                        // 通るため、ここに到達した時点で先行メッセージは drain 済み
                        // （clean drain）。ack はループ脱出後・cleanup 完了後に送る。
                        ActorMsg::Stop { done } => {
                            // 観測ログ点（R10.4）: Stop を受信しループ脱出（clean drain 完了）。
                            tracing::debug!(seam = "actor.stop", "actor received Stop; draining done, breaking loop");
                            done_ack = Some(done);
                            break;
                        }
                    }
                }

                // (3) teardown cleanup（task 4.1・design.md「reload teardown」順序）:
                //     VM（shiori）をこのスレッド上で明示 drop する。drop により
                //     PastaShiori が内包する !Send Lua VM・debug backend（socket bridge
                //     join・port 解放）・関連リソースが解放される（debug backend は VM の
                //     一部として VM drop 時にこのアクタースレッド上で teardown される）。
                //     メッセージ専用ウィンドウは block_on 完了時に executor が破棄する。
                drop(shiori);

                // (4) cleanup 完了後に done ack を送る（R7.1/R7.4: ack 受信＝全資源解放
                //     済み）。Stop 経由でなく rx Disconnected 等でループを抜けた場合は
                //     ack 経路を持たない（done_ack=None）ので送らない。
                if let Some(done) = done_ack {
                    let _ = done.send(());
                    // 観測ログ点（R10.4）: cleanup 完了後に done ack を送出（全資源解放済み）。
                    tracing::debug!(seam = "actor.done", "actor sent done ack after VM/resource cleanup");
                }
                // block_on はこの async ブロック完了で戻り、メッセージ専用ウィンドウ等
                // executor 資源が解放される。スレッドは SHIORI 側が detach 済み。
            });
        })
        .expect("actor thread must spawn");

    // VM 構築完了（実行スレッド id・ロード成否・debug DAP 束縛アドレス）を待つ。
    let (actor_thread_id, loaded, debug_local_addr) = ready_rx
        .recv()
        .expect("actor thread must report readiness");

    ActorThread {
        handle: Some(handle),
        actor_thread_id,
        loaded,
        debug_local_addr,
    }
}
