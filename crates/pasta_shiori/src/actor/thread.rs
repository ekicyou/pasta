//! 本番アクタースレッド。
//!
//! `!Send` な Lua VM（[`PastaShiori`] が内包する [`PastaLuaRuntime`]）を **専用 OS
//! スレッドへ pin** し、そのスレッド上で単一の同期 `recv()` ループを回して mailbox
//! を消費する（OS のウィンドウやメッセージの仕組みを使わない・全部の OS で同じ形）。
//!
//! # なぜ生リクエスト文字列を mailbox で運ぶのか
//! [`PastaLuaRuntime`]（mlua）は `!Send` であり、チャネルで越境できない。そこで mailbox
//! が運ぶのは `Send` な生の SHIORI リクエスト文字列（[`MailboxRequest::raw`]）のみとし、
//! **VM スレッド側でパースしてテーブル化**する。VM はこのスレッドで生成され、ここを一度も離れない。
//!
//! # コルーチン意味論の不変保存
//! `co_scene` はメッセージ間で VM 内に persist するため、`resume_until_valid` / `CALLBACK` は
//! tick から tick へ継続する。ループは VM を駆動するだけでコルーチン状態には干渉しない。
//!
//! # 単一 consumer 不変条件
//! 消費は単一の同期 `recv()` ループのみで行い、`select!` を張らない
//! （mailbox.rs のドキュメント参照）。

use std::io;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use flume::Receiver;
use pasta_lua::{ClaimToken, LoadDirGuard};

use crate::actor::mailbox::ActorMsg;
use crate::actor::module_ref::ModuleRef;
use crate::shiori::PastaShiori;

/// アクタースレッドのハンドル。
///
/// VM はこのスレッドに pin され、決して越境しない。SHIORI スレッドは mailbox
/// （[`ActorMsg`]）越しにのみ VM へアクセスする。
pub struct ActorThread {
    handle: Option<JoinHandle<()>>,
    /// ロード成否（`SHIORI.load` 成功＝true）。失敗時もスレッドは drain を続ける
    /// （request は NotInitialized/Load エラーを返す）。
    loaded: bool,
    /// join の約束の送り手（容量 1）。送ればスレッドは普通に戻り、落とすと「見捨てた」になる
    /// （スレッドが参照を返して終わる）。
    keep: flume::Sender<()>,
    /// スレッドが走っている間 pasta.dll を外さない参照。返すのは join した親か、見捨てられたスレッドの一方だけ。
    module: ModuleRef,
}

impl ActorThread {
    /// `SHIORI.load` がアクタースレッド上で成功したか。
    pub fn loaded(&self) -> bool {
        self.loaded
    }

    /// アクタースレッドを join する（ack の後の後始末）。
    ///
    /// 呼び出し側は事前に [`ActorMsg::Stop`] を mailbox へ送り、ループを抜けさせて
    /// おくこと（さもなくば `recv()` が永久に待ち続けて join できない）。
    ///
    /// 約束を送る → join → 参照を返す、の順（約束の前に join すると止まる）。スレッドが panic で
    /// 終わっていても参照は返す（閉包は約束に届いていないので、返すのはここだけ）。
    pub fn join(mut self) -> thread::Result<()> {
        let _ = self.keep.send(());
        let result = match self.handle.take() {
            Some(h) => h.join(),
            None => Ok(()),
        };
        self.module.release();
        result
    }

    /// アクタースレッドを **detach** する（ack が来ずに手放すとき）。
    ///
    /// 約束の送り手を落とすので、スレッドは片付けの後に自分で参照を返して終わる
    /// （`FreeLibraryAndExitThread`）。
    pub fn detach(self) {
        drop(self);
    }
}

/// 専用アクタースレッドを起動し、その上で `!Send` VM を生成・pin して mailbox を駆動する。
///
/// # 引数
/// - `load_dir`: VM を構築する [`PastaShiori::load`] へ渡す。
/// - `rx`: mailbox の consumer 側 [`Receiver`]。
/// - `claim`: VM の片付けで外すディレクトリの占有の札（[`PastaShiori::set_claim`]）。
///
/// # 戻り値
/// [`ActorThread`]（`loaded()` でロード成否を観測できる。`join()` 前に [`ActorMsg::Stop`] を
/// 送ること）。返った時点で VM の構築と `SHIORI.load` の呼び出しは終わっている。OS がスレッドを
/// 起こせないときは `thread::Builder::spawn` の誤り、スレッドが ready を返す前に終わった
/// ときは `io::ErrorKind::Other` の誤りを返す（パニックしない）。後者ではスレッドを
/// join してから返し、資源を残さない。
pub fn spawn_actor_thread_with(
    load_dir: PathBuf,
    rx: Receiver<ActorMsg>,
    claim: Option<ClaimToken>,
) -> io::Result<ActorThread> {
    // 参照はスレッドを起こす前に取る（参照の無いアクタースレッドを作らない）。
    let module = ModuleRef::acquire()?;
    let (keep, keep_rx) = flume::bounded::<()>(1);
    let (ready_tx, ready_rx) = flume::bounded::<bool>(1);

    let spawned = thread::Builder::new()
        .name("pasta-actor".to_string())
        .spawn(move || {
            // ログの振り分けの文脈。スレッドの終わりまで保ち、ループのログを自分のロガーへ届ける。
            // done ack の後に出るログは登録解除の後なので捨てられる。
            let log_context = LoadDirGuard::new(load_dir.clone());
            let mut shiori = PastaShiori::default();
            shiori.set_claim(claim);
            let loaded = shiori.load(load_dir.as_os_str()).unwrap_or(false);

            tracing::debug!(seam = "actor.spawn", loaded, "actor thread spawned; VM pinned on this thread");

            let _ = ready_tx.send(loaded);

            // done ack はループ脱出後、VM 破棄・cleanup を終えてから送る（ack 受信＝全資源解放済み）。
            let mut done_ack: Option<flume::Sender<()>> = None;
            while let Ok(msg) = rx.recv() {
                match msg {
                    ActorMsg::Get { req, reply } => {
                        let seq = req.seq;
                        tracing::trace!(seam = "actor.recv", method = "get", seq, "actor received GET");
                        // GET 応答を **必ず 1 回**返す。失敗は 204 に読み替えず
                        // 500 + X-ERROR-REASON で返す（ロード失敗・未ロード・Lua 実行エラーとも）。
                        match shiori.request(&req.raw) {
                            Ok(resp) => {
                                let _ = reply.send(resp);
                                tracing::trace!(seam = "actor.reply", method = "get", seq, "actor replied with VM response");
                            }
                            Err(e) => {
                                let _ = reply.send(e.to_shiori_response());
                                tracing::debug!(seam = "actor.reply", method = "get", seq, error = true, reason = %e, "actor replied with 500 error response");
                            }
                        }
                    }
                    // NOTIFY の応答は捨てる（呼び出し側は即 204 を返し済み）。
                    ActorMsg::Notify { req } => {
                        tracing::trace!(seam = "actor.recv", method = "notify", seq = req.seq, "actor received NOTIFY");
                        let _ = shiori.request(&req.raw);
                    }
                    // Stop は同一 FIFO を通るため、ここに到達した時点で先行メッセージは drain 済み。
                    ActorMsg::Stop { done } => {
                        tracing::debug!(seam = "actor.stop", "actor received Stop; draining done, breaking loop");
                        done_ack = Some(done);
                        break;
                    }
                }
            }

            // VM はこのスレッド上で drop する（debug backend も VM の一部としてここで teardown される）。
            drop(shiori);
            // mailbox receiver も ack 前に閉じる。スレッドの終わりまで rx を生かすと、
            // ack〜スレッド終了の隙間に Stop の再送が受理されて冪等判定が競合する。
            drop(rx);

            if let Some(done) = done_ack {
                let _ = done.send(());
                tracing::debug!(seam = "actor.done", "actor sent done ack after VM/resource cleanup");
            }
            // 持つ値を落としてから join の約束を待つ（約束を待つ間は DLL のコードを走らせない）。
            // 届けば普通に戻り、参照は join した親が返す。送り手が落ちていれば（見捨てられた）
            // 参照を返してスレッドを終える（Windows では戻らない）。
            drop(log_context);
            drop(load_dir);
            drop(ready_tx);
            if keep_rx.recv().is_err() {
                drop(keep_rx);
                module.release_and_exit_thread();
            }
        });
    let handle = match spawned {
        Ok(handle) => handle,
        Err(e) => {
            // 閉包は走らないので、参照はここで返す。
            module.release();
            return Err(e);
        }
    };

    // ready の送り手が落ちた＝スレッドが ready の前に終わった（パニック等）。join して資源を残さない。
    // 閉包は約束に届いていないので、参照は join の後にここで返す。
    let Ok(loaded) = ready_rx.recv() else {
        let _ = handle.join();
        module.release();
        return Err(io::Error::other(
            "actor thread ended before reporting readiness",
        ));
    };

    Ok(ActorThread {
        handle: Some(handle),
        loaded,
        keep,
        module,
    })
}
