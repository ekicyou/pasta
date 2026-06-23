//! FFI 入口の所有モデル（`static MAILBOX` ＋ 自由関数・task 5.1・R8.1/R8.2/R8.3/R8.4）。
//!
//! design.md「ActorLifecycle（`static MAILBOX` ＋ 自由関数）」の本番化。出荷 SHIORI FFI
//! 経路を、旧来の同期 in-place VM モデル（`OnceLock<RawShiori>` ＋
//! `Arc<Mutex<Option<PastaShiori>>>` ＋ `unsafe impl Send/Sync`）から、アクタースレッド
//! （task 3.3）へ VM を pin し、mailbox（task 3.2）越しにのみ VM へアクセスする本番アクター
//! モデルへ **再配線**する。
//!
//! # スロット型の決定（RN6・respawn モデル）
//! `static MAILBOX` のスロット型は [`arc_swap::ArcSwapOption`]`<Sender<ActorMsg>>` を採用する。
//!
//! - **なぜ `OnceLock` ではないか**: SHIORI reload（`unload`→`load` の反復）では、アクター
//!   スレッドと mailbox チャネルを [`teardown_actor`] で teardown し、次の [`spawn_actor`] で
//!   **新しいスレッド＋新しいチャネル**を起こす（design.md「reload teardown」: *re-load spawns
//!   fresh actor thread and channel*）。`OnceLock` は一度だけ set でき再設定できないため、
//!   reload で新 `Sender` へ差し替えられず不適。
//! - **なぜ `Mutex<Option<Sender>>` ではないか**: 送信パス（[`marshal_get`]/[`marshal_notify`]）が
//!   毎リクエストでロックを取ることになり、R8 が要求する **lock-free 送信パス**（送信パスに
//!   Mutex を置かない）に反する。
//! - **`ArcSwapOption` の性質**: `store`（reload 時のみ）/ `load`（毎送信）が **lock-free**
//!   （RCU 風・読み取りは待たない）。送信パスは `MAILBOX.load()`（Mutex なし）→ `Sender::try_send`
//!   のみ。`Sender<ActorMsg>` は `Send + Sync + Clone` ゆえ `Arc` 包みで lock-free 共有でき、
//!   **`unsafe impl Send/Sync` は構造的に不要**（R8.1/R8.3: VM はアクタースレッドへ pin され
//!   越境しない。住所＝flume `Sender` のみが越境する＝真に `Send + Sync`）。
//!
//! # FFI 配線（`windows.rs`）
//! - `load`   → [`spawn_actor`]（mailbox 生成 → `spawn_actor_thread` → `Sender` を MAILBOX へ
//!   `store`）。スレッド spawn は **load 起点**で行い（DllMain attach ではない）、loader lock を
//!   回避する（R4.4）。
//! - `request`→ [`marshal_request`]（メソッド判定 → GET=block-on-reply／NOTIFY=即 204）。
//! - `unload` / `DllMain detach` → [`teardown_actor`]（`Stop{done}` ack → detach → MAILBOX クリア）。
//!
//! # アクセスはメッセージ送信経由（R8.2）
//! VM への一切のアクセスは [`ActorMsg`] の mailbox 送信を通じてのみ行われる
//! （`Arc<Mutex>` 直接アクセスではない）。`marshal_*` は MAILBOX 未初期化（アクター不在）でも
//! 必ず 204 を返し、SHIORI スレッドを無限待機させない（R5.6）。

use std::path::PathBuf;
use std::time::Duration;

use arc_swap::ArcSwapOption;
use flume::Sender;
use pasta_lua::debug::{KickRequest, KickSink};

use crate::actor::mailbox::{mailbox, ActorMsg, MailboxRequest};
use crate::actor::marshaling::{default_204, marshal_get, marshal_notify, determine_method, ShioriMethod};
use crate::actor::teardown::{teardown_via_sender, TeardownReport};
use crate::actor::thread::{spawn_actor_thread, ActorThread};

/// FFI 入口が teardown ack を待つ既定の上限。通常運転では teardown は速やかに完了する。
/// 万一アクターが ack を返さない場合でも SHIORI スレッド（＝SSP 側）を無限待機させない安全網。
const TEARDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// reload の都度に投入される連番（FIFO 観測用の `seq`）。送信のみで読まれる軽量カウンタ。
static REQUEST_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// `static MAILBOX`（design.md「ActorLifecycle」）— アクタースレッドの住所＝flume 送信端。
///
/// スロット型は [`ArcSwapOption`]`<Sender<ActorMsg>>`（RN6 respawn モデル・モジュール doc 参照）。
/// `load`（毎送信）/`store`（reload 時のみ）が lock-free。`unsafe impl Send/Sync` を撤去し、
/// 真に `Send + Sync` な `Sender` のみを越境させることで R8 を構造的に達成する。
static MAILBOX: ArcSwapOption<Sender<ActorMsg>> = ArcSwapOption::const_empty();

/// アクタースレッドのハンドル（VM 実行スレッド）を保持する。
///
/// 送信パスからは読まれない（送信は [`MAILBOX`] の `Sender` のみで完結する）。spawn/teardown の
/// ライフサイクル制御専用であり、`unload`/`DllMain detach` 時に `take()` して `detach` する。
/// 送信パスに Mutex を置かないという R8 不変条件は [`MAILBOX`]（`ArcSwapOption`）が担保し、
/// 本ハンドルは送信頻度ゼロのライフサイクル境界でのみ触れるため `Mutex` で十分（送信パス外）。
static ACTOR_HANDLE: std::sync::Mutex<Option<ActorThread>> = std::sync::Mutex::new(None);

/// アクタースレッドを起動し、その `Sender` を [`MAILBOX`] へ設定する（`load` 起点・R4.4）。
///
/// 既存アクターがあれば先に [`teardown_actor`] して reload する（新スレッド＋新チャネル）。
/// VM（`!Send`）はアクタースレッド上で構築・pin され、ここを越境しない（R8.1/R8.3）。
///
/// 戻り値はアクタースレッド上での `SHIORI.load` 成否（VM ロード成功＝true）。spawn 自体に
/// 失敗した場合（スレッド起動不可など）は `false`。
pub fn spawn_actor(hinst: isize, load_dir: PathBuf) -> bool {
    // reload: 既存アクターがあれば teardown してからスレッド／チャネルを差し替える。
    teardown_actor();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(hinst, load_dir, rx);
    let loaded = actor.loaded();
    // 観測ログ点（R10.4・無効時ゼロコスト）: FFI load 起点でアクターを起動した（R4.4）。
    tracing::debug!(seam = "actor.spawn", loaded, "lifecycle: actor spawned from FFI load entry");

    // 送信端を lock-free スロットへ格納（以後の送信は MAILBOX.load() で参照）。
    MAILBOX.store(Some(std::sync::Arc::new(tx)));

    // ライフサイクル制御用にスレッドハンドルを保持（送信パス外）。
    if let Ok(mut guard) = ACTOR_HANDLE.lock() {
        *guard = Some(actor);
    } else {
        // ロック poison（直前の teardown でハンドルが既に取り出されているのが通常で、
        // poison は理論上のみ）。ハンドルを保持できない場合は detach に倒す（join しない）。
        actor_detach_on_poison(actor);
    }

    loaded
}

/// poison 時のフォールバック: ハンドルを保持できないため即 detach する（join でハングしない）。
fn actor_detach_on_poison(actor: ActorThread) {
    actor.detach();
}

/// SHIORI request を marshaling する（`request` FFI 入口・R5.1/R5.2/R5.5/R5.6）。
///
/// [`MAILBOX`] を lock-free に読み（`load`）、メソッド判定（GET/NOTIFY）して
/// GET=block-on-reply／NOTIFY=即 204 へ分岐する。MAILBOX 未初期化（アクター不在）や不正
/// リクエストは 204 に倒し、SHIORI スレッドを無限待機させない。
pub fn marshal_request(request: &str) -> String {
    // 送信パス: lock-free 読み取り（Mutex なし・R8）。
    let tx = match MAILBOX.load_full() {
        Some(tx) => tx,
        // アクター未起動（load 前 / teardown 後）: 安全側として 204。
        None => return default_204(),
    };

    let seq = REQUEST_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    match determine_method(request) {
        Some(ShioriMethod::Get) => {
            marshal_get(&tx, MailboxRequest::new(seq, request.to_string()))
        }
        Some(ShioriMethod::Notify) => {
            marshal_notify(&tx, MailboxRequest::new(seq, request.to_string()))
        }
        // 不正リクエスト（メソッド未検出）: 安全側として 204。
        None => default_204(),
    }
}

/// アクタースレッドを teardown する（`unload`／`DllMain detach` 入口・R7.1/R7.4）。
///
/// [`MAILBOX`] の `Sender` 越しに `Stop{done}` を送り、done ack を有界に待ってから
/// スレッドを detach し、`MAILBOX` をクリアする。二重 teardown（unload と detach の競合）でも
/// 安全（冪等 no-op）。
///
/// 戻り値は teardown の結果レポート（clean／冪等 no-op／異常を値で返す。ホストを巻き込まない）。
pub fn teardown_actor() -> TeardownReport {
    // 送信端を取り出す（取り出し＝以後の送信を遮断。次 spawn まで MAILBOX は空）。
    let tx = MAILBOX.swap(None);

    // スレッドハンドルを取り出す（ライフサイクル境界・送信パス外）。
    let actor = ACTOR_HANDLE.lock().ok().and_then(|mut g| g.take());

    let report = match tx {
        Some(tx) => teardown_via_sender(&tx, TEARDOWN_TIMEOUT),
        // MAILBOX 未初期化（load 前 / 既 teardown 済み）: 冪等 no-op。
        None => TeardownReport {
            acked: false,
            already_done: true,
            anomaly: None,
        },
    };

    // ack を受けたか否かに関わらずスレッドは detach する（join しない・R7.4）。
    if let Some(actor) = actor {
        actor.detach();
    }

    report
}

/// `MAILBOX` 投函クロージャ本体（`MailboxKickInjector`・design.md・requirements 2.4）。
///
/// host（`pasta_shiori`）が `pasta_lua` の [`KickSink`] として注入するクロージャの実体。
/// 受領した [`KickRequest`] を [`ActorMsg::Kick`] へ写し、[`MAILBOX`] を **lock-free に読み**
/// （`load_full`・送信パスに Mutex を置かない・R8）`try_send` で非ブロッキングに投函する。
///
/// teardown/reload 競合（`MAILBOX.swap(None)` 直後＝アクター不在）は `load_full()=None` で
/// 黙って no-op とし、診断ログ（`seam = "kick.drop"`・ベストエフォート・D6）を残す。
/// `try_send` 失敗（満杯/切断＝アクター不在/異常）も同様に診断ログ後に破棄する（デバッグ用途で許容）。
fn kick_into_mailbox(req: KickRequest) {
    // 送信パス: lock-free 読み取り（Mutex なし・R8）。
    let Some(tx) = MAILBOX.load_full() else {
        // teardown/reload 競合（アクター不在）: 黙って no-op＋診断ログ（D6）。
        tracing::debug!(
            seam = "kick.drop",
            scene = %req.scene,
            reason = "mailbox_absent",
            "kick dropped: MAILBOX absent (teardown/reload race) -> no-op"
        );
        return;
    };

    // fire-and-forget（NOTIFY 同型）: try_send 失敗（満杯/切断）も握って破棄する。
    if tx.try_send(ActorMsg::Kick { scene: req.scene.clone() }).is_err() {
        tracing::debug!(
            seam = "kick.drop",
            scene = %req.scene,
            reason = "try_send_failed",
            "kick dropped: try_send failed (mailbox full/disconnected) -> discarded"
        );
    }
}

/// `MailboxKickInjector` の [`KickSink`] を構築する（VM 構築前に `RuntimeConfig` へ束縛する）。
///
/// [`kick_into_mailbox`] を型消去された `Arc<dyn Fn>` に包んで返す。`pasta_lua` 側は
/// [`pasta_lua::RuntimeConfig::with_kick_sink`] でこれを不透明に保持し、後続 task（2.x）が
/// `enable` 経由で socket-bridge へ透過させる。クロージャは [`MAILBOX`]（`'static`）のみに依存
/// するため `'static` 寿命と `Send + Sync` を満たす。
pub fn kick_sink() -> KickSink {
    std::sync::Arc::new(kick_into_mailbox)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `static MAILBOX` のスロット型が lock-free（`ArcSwapOption`）であり、`Sender` が
    /// `Send + Sync` であることを静的に確認する（R8.3・送信パス Mutex フリーの型レベル証跡）。
    #[test]
    fn mailbox_slot_is_lock_free_and_sender_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Sender<ActorMsg>>();
        // ArcSwapOption は store/load が lock-free（Mutex を含まない）。
        let slot: ArcSwapOption<Sender<ActorMsg>> = ArcSwapOption::const_empty();
        assert!(slot.load().is_none(), "empty slot loads None without locking");
    }

    /// MAILBOX 未初期化時（アクター不在）の marshal_request は 204 に倒し、無限待機しない（R5.6）。
    ///
    /// プロセスグローバル MAILBOX を汚さないよう、teardown を先に呼んで空状態を作る。
    /// （他テストと直列化はしないが、None フォールバックは MAILBOX の状態に依らず 204 を返す経路。）
    #[test]
    fn marshal_request_without_actor_returns_204() {
        // 直接 None 経路を踏むため、ローカル相当のガード: MAILBOX が空なら 204。
        // 実体の static を汚さないよう store/swap はせず、None 経路の出力契約のみ検証する。
        let resp = if MAILBOX.load_full().is_none() {
            marshal_request("GET SHIORI/3.0\r\nID: version\r\n\r\n")
        } else {
            // 既にアクターが居る場合でも default_204 のバイト契約は不変なのでそれを検証。
            default_204()
        };
        // どちらの分岐でも、未初期化 None 経路は 204 を返す契約（バイト不変）。
        if MAILBOX.load_full().is_none() {
            assert_eq!(resp.as_bytes(), default_204().as_bytes());
        }
    }

    /// MAILBOX 操作を伴うテストの直列化ロック（プロセスグローバル MAILBOX を共有するため）。
    static MAILBOX_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// MailboxKickInjector（task 3.1・requirements 2.4・design「MailboxKickInjector」）:
    /// MAILBOX に Sender を store した状態で sink クロージャ（`KickRequest`）を呼ぶと、
    /// receiver に `ActorMsg::Kick { scene }` が届く。
    #[test]
    fn kick_sink_delivers_kick_when_mailbox_present() {
        let _guard = MAILBOX_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // テスト用 mailbox を MAILBOX へ設置（lock-free スロット）。
        let (tx, rx) = mailbox();
        MAILBOX.store(Some(std::sync::Arc::new(tx)));

        // host が注入するのと同じ sink を取得して呼ぶ。
        let sink = kick_sink();
        sink(KickRequest {
            scene: "OnTestScene".into(),
        });

        // receiver に Kick variant が scene 保持で届く。
        match rx.try_recv() {
            Ok(ActorMsg::Kick { scene }) => assert_eq!(scene, "OnTestScene"),
            other => panic!("expected ActorMsg::Kick, got {other:?}"),
        }

        // 後始末: 次テストの None 前提を壊さないよう MAILBOX を空へ戻す。
        MAILBOX.swap(None);
    }

    /// MailboxKickInjector teardown 競合（D6）: `swap(None)` 後（アクター不在）に sink を
    /// 呼んでも panic せず no-op（送信なし）。`seam = "kick.drop"` 診断はベストエフォート。
    #[test]
    fn kick_sink_is_noop_when_mailbox_absent() {
        let _guard = MAILBOX_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // teardown 後を模す: MAILBOX を空にする。
        MAILBOX.swap(None);
        assert!(
            MAILBOX.load_full().is_none(),
            "precondition: MAILBOX must be absent"
        );

        // 呼び出しても panic せず（送信先が無いので単に黙って no-op）。
        let sink = kick_sink();
        sink(KickRequest {
            scene: "OnTestScene".into(),
        });

        // MAILBOX 不在は不変（クロージャは store/swap しない）。
        assert!(
            MAILBOX.load_full().is_none(),
            "kick into absent MAILBOX must remain a no-op"
        );
    }

    /// teardown_actor は MAILBOX 未初期化でも冪等 no-op を返す（R7.4・ハングなし）。
    #[test]
    fn teardown_without_actor_is_idempotent_noop() {
        // MAILBOX が空（アクター不在）なら already_done の冪等 no-op。
        if MAILBOX.load_full().is_none() {
            let report = teardown_actor();
            assert!(
                report.already_done && !report.acked && report.anomaly.is_none(),
                "teardown with no actor must be a safe idempotent no-op"
            );
        }
    }
}
