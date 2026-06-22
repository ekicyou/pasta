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
//! # task 3.3 のスコープ / 後続への申し送り
//! - 本タスクは **本番アクタースレッドを mailbox 駆動で隔離テスト**することが目的。
//!   FFI 入口（`shiori::request`）の配線は **task 5.1**（そこで wintf が出荷依存になる）。
//!   既存出荷経路（[`PastaShiori::request`] の同期ディスパッチ）は本タスクで一切改変
//!   しない（byte-invariant ゴールデン 4/4 を緑のまま保つ）。
//! - reply / timeout / 204 / drop の完全な marshaling 契約は **task 3.4**。本タスクの
//!   `Get` 応答は「VM が返した応答文字列を [`Reply::Value`] で送り返す」最小実装に留め、
//!   コルーチン継続を検証できる最小限とする（timeout/drop→204 はここでは扱わない）。
//!
//! # gating（CONCERNS 参照）
//! wintf（`block_on`）は現状 `actor-poc` feature 有効時のみリンクされる（task 1.2／
//! Cargo.toml）。本モジュールはアクタースレッドを **今テストできる**ようにしつつ、
//! 既定出荷ビルドへ wintf を早期リンクしないため、`actor-poc` feature でガードする。
//! task 5.1 で FFI が本スレッドへ切り替わる時に wintf が出荷依存へ昇格する。

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
    // VM 実行スレッド id とロード成否を呼び出し側へ返す小チャネル（値のみ越境）。
    let (ready_tx, ready_rx) = flume::bounded::<(ThreadId, bool)>(1);

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

                // 実行スレッド id とロード成否を返す（VM 本体は越境しない）。
                let _ = ready_tx.send((thread::current().id(), loaded));

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
                            // task 3.4 申し送り: 完全な timeout/drop→204 marshaling は 3.4。
                            // ここでは VM 応答文字列を Reply::Value で返す最小実装。
                            // エラー時は reply を送らず drop（受信側 Disconnected→3.4 で 204）。
                            if let Ok(resp) = shiori.request(&req.raw) {
                                let _ = reply.send(Reply::Value(resp));
                            }
                        }
                        // NOTIFY fire-and-forget: VM 上で処理し応答経路は持たない。
                        // 応答は捨てる（NOTIFY は即 204 で終結する・3.4 で契約確定）。
                        ActorMsg::Notify { req } => {
                            let _ = shiori.request(&req.raw);
                        }
                        // teardown: ack 経路を保持してループを抜ける。Stop は同一 FIFO を
                        // 通るため、ここに到達した時点で先行メッセージは drain 済み
                        // （clean drain）。ack はループ脱出後・cleanup 完了後に送る。
                        ActorMsg::Stop { done } => {
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
                }
                // block_on はこの async ブロック完了で戻り、メッセージ専用ウィンドウ等
                // executor 資源が解放される。スレッドは SHIORI 側が detach 済み。
            });
        })
        .expect("actor thread must spawn");

    // VM 構築完了（実行スレッド id・ロード成否）を待つ。
    let (actor_thread_id, loaded) = ready_rx
        .recv()
        .expect("actor thread must report readiness");

    ActorThread {
        handle: Some(handle),
        actor_thread_id,
        loaded,
    }
}
