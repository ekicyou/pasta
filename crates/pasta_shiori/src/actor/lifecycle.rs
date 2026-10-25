//! 旧来の入口の置き場（LegacySlot・仕様 `shiori4-v3-portable`・design.md「LegacySlot」）。
//!
//! 旧来の SHIORI の入口（`windows.rs` の `load`／`request`／`unload`／`DllMain`）は、呼び出し側に
//! インスタンスのハンドルを返せない ABI である。そのためこの入口だけは、プロセスに 1 つの
//! 置き場 [`LEGACY`]（芯の型 [`ActorInstance`] を 1 つ持つ）を残す（5.7）。芯・アクター・版 3 の
//! 口はグローバルな状態を持たず、ここを参照しない。
//!
//! # 窓口の自由関数（名前と形は今のまま・7.1・7.4）
//! - [`spawn_actor`]: 先に [`teardown_actor`] で前のインスタンスを片付け（7.2）、新しい
//!   [`ActorInstance`] を起こして置き場へ入れる。
//! - [`marshal_request`]: 置き場の芯の型へ要求を渡す。空なら 204（SHIORI スレッドを待たせない）。
//! - [`teardown_actor`]: 置き場から取り出して後始末する（ack の後に join・7.6）。空なら冪等の no-op。
//! - [`teardown_actor_detached`]: 同じだが join しない（`DllMain` の FreeLibrary の detach だけ）。
//! - [`kick_sink`]: 置き場の芯の型へ kick を積む送り先。空なら `kick.drop` の診断ログで捨てる。
//!
//! # 送信の道にロックを置かない
//! 置き場は [`ArcSwapOption`] で、送信の道（[`marshal_request`]・[`kick_sink`]）は `load_full()`
//! で `Arc` を写すだけである。差し替え（[`spawn_actor`]・[`teardown_actor`]）だけが `store`／`swap`
//! する。VM はアクタースレッドから出ず、越境するのは `Send + Sync` な [`ActorInstance`] だけ。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwapOption;
use pasta_lua::debug::{KickRequest, KickSink};

use crate::actor::instance::{ActorInstance, ClaimMode, JoinPolicy, TEARDOWN_TIMEOUT};
use crate::actor::marshaling::default_204;
use crate::actor::teardown::TeardownReport;

/// 旧来の入口の置き場。プロセスに 1 つの状態はこれだけ（5.7）。
static LEGACY: ArcSwapOption<ActorInstance> = ArcSwapOption::const_empty();

/// アクターを起こして置き場へ入れる（`load` 起点・loader lock の外）。
///
/// 前のインスタンスがあれば先に [`teardown_actor`] で片付ける（reload・7.2）。
/// 戻り値はアクタースレッド上での `PastaShiori::load` 成否。ロードに失敗してもインスタンスは
/// 置き場に入り、以後の GET の応答は失敗の種類で異なる（`PastaShiori::request` を参照）。
/// スレッドを起こせなかったときと、ディレクトリをほかのインスタンスが占有しているとき
/// （`SpawnError`）は偽を返し、置き場は空のまま。
///
/// 前の片付けが打ち切りに終わり（ack が来ずにスレッドを手放した）同じディレクトリを読み直すときは、
/// 手放したスレッドの VM がまだ持つ占有を引き継ぐ（今の SSP の再読み込みを保つ・7.1・7.2）。
pub fn spawn_actor(hinst: isize, load_dir: PathBuf) -> bool {
    spawn_actor_with_teardown_timeout(hinst, load_dir, TEARDOWN_TIMEOUT)
}

/// [`spawn_actor`] の、前のインスタンスの片付けで ack を待つ上限を差し替える形
/// （試験から短い値を注入する内部の口。本番の上限は [`TEARDOWN_TIMEOUT`] のまま）。
#[doc(hidden)]
pub fn spawn_actor_with_teardown_timeout(
    hinst: isize,
    load_dir: PathBuf,
    teardown_timeout: Duration,
) -> bool {
    // 前のディレクトリは片付けの前に手元へ取る（新しいグローバルな状態は持たない）。
    let prev_dir = LEGACY.load().as_ref().map(|i| i.load_dir().to_path_buf());
    let report = teardown_slot(&LEGACY, JoinPolicy::JoinAfterAck, teardown_timeout);
    let timed_out = !report.acked && !report.already_done;
    let mode = if timed_out && prev_dir.as_deref() == Some(load_dir.as_path()) {
        ClaimMode::TakeOver
    } else {
        ClaimMode::Exclusive
    };

    let Ok(instance) = ActorInstance::spawn_with(hinst, load_dir, mode) else {
        // 理由は ActorInstance::spawn_with がログに出している。
        return false;
    };
    let loaded = instance.loaded();
    tracing::debug!(
        seam = "actor.spawn",
        loaded,
        "lifecycle: actor spawned from FFI load entry"
    );
    LEGACY.store(Some(Arc::new(instance)));
    loaded
}

/// SHIORI request を置き場の芯の型へ渡す（`request` FFI 入口）。置き場が空なら 204。
pub fn marshal_request(request: &str) -> String {
    request_into(&LEGACY, request)
}

fn request_into(slot: &ArcSwapOption<ActorInstance>, request: &str) -> String {
    match slot.load_full() {
        Some(instance) => instance.request(request),
        // アクター未起動（load 前 / teardown 後）: 安全側として 204。
        None => default_204(),
    }
}

/// 置き場のアクターを後始末する（`unload` 入口・7.1・7.6）。
///
/// 戻り値は後始末の結果（clean／冪等 no-op／異常を値で返す。ホストを巻き込まない）。
/// ack が届けばアクタースレッドを join してから返る（ack が来なければ手放す）。
pub fn teardown_actor() -> TeardownReport {
    teardown_slot(&LEGACY, JoinPolicy::JoinAfterAck, TEARDOWN_TIMEOUT)
}

/// 置き場のアクターを、join せずに後始末する（`DllMain` の FreeLibrary の detach だけ・7.6）。
///
/// loader lock の中ではアクタースレッドの終わりを待てないので、ack を受けたらスレッドを手放す。
pub fn teardown_actor_detached() -> TeardownReport {
    teardown_slot(&LEGACY, JoinPolicy::Detach, TEARDOWN_TIMEOUT)
}

fn teardown_slot(
    slot: &ArcSwapOption<ActorInstance>,
    policy: JoinPolicy,
    timeout: Duration,
) -> TeardownReport {
    // 取り出し＝以後の送信を遮断（次の spawn まで置き場は空）。
    let Some(instance) = slot.swap(None) else {
        // 置き場が空（load 前 / 片付け済み）: 冪等 no-op。
        return TeardownReport {
            acked: false,
            already_done: true,
            anomaly: None,
            joined: false,
        };
    };
    match Arc::try_unwrap(instance) {
        Ok(instance) => instance.shutdown_with_timeout(policy, timeout),
        // 片付けと同時に別のスレッドで要求が走っている（単一スレッドの規約に反する呼び方）。
        // 停止の要求だけをして、最後の参照の Drop でスレッドを手放す（join は保証しない）。
        // `request_stop` が ack を受けていれば、Drop の Stop は閉じた mailbox に届かず
        // already-done になるので、Drop は待たずに手放す（`DllMain` の道の loader lock の中でも
        // join で待たない）。ack が timeout していたときだけ、最後の Drop が ack を最大 5 秒
        // 待ち、届けば join する（規約に反する同時の呼び出しでだけ起きる残りの端）。
        Err(shared) => shared.request_stop(),
    }
}

/// 置き場の芯の型へ kick を積む。送れなければ診断ログ（`seam = "kick.drop"`）を残して偽。
fn kick_into(slot: &ArcSwapOption<ActorInstance>, req: KickRequest) -> bool {
    let sent = slot
        .load_full()
        .is_some_and(|instance| instance.kick(req.scene.clone()));
    if !sent {
        tracing::debug!(
            seam = "kick.drop",
            scene = %req.scene,
            "kick dropped: legacy slot empty or mailbox full/disconnected -> discarded"
        );
    }
    sent
}

/// 置き場の `ActorInstance` へ kick を積む窓口（試験と互換のために残す）。
///
/// 本番の kick は各 `ActorInstance` が自分の送り先（`kick_sink_for`）で受けるので、ここは通らない。
///
/// クロージャは [`LEGACY`]（`'static`）だけに頼るので `'static` かつ `Send + Sync`。
pub fn kick_sink() -> KickSink {
    Arc::new(|req| {
        kick_into(&LEGACY, req);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GET_VERSION: &str = "GET SHIORI/3.0\r\nID: version\r\n\r\n";

    /// 置き場の型がロックを含まない（`ArcSwapOption`）こと（送信の道にロックを置かない）。
    #[test]
    fn slot_is_lock_free_and_empty_by_default() {
        let slot: ArcSwapOption<ActorInstance> = ArcSwapOption::const_empty();
        assert!(
            slot.load().is_none(),
            "empty slot loads None without locking"
        );
    }

    /// 置き場が空のときの要求は 204 に倒し、無限待機しない。
    #[test]
    fn request_on_empty_slot_returns_204() {
        let slot: ArcSwapOption<ActorInstance> = ArcSwapOption::const_empty();
        assert_eq!(
            request_into(&slot, GET_VERSION).as_bytes(),
            default_204().as_bytes()
        );
    }

    /// 置き場が空のときの kick は panic せず送らない（片付けとの競合）。
    #[test]
    fn kick_on_empty_slot_is_noop() {
        let slot: ArcSwapOption<ActorInstance> = ArcSwapOption::const_empty();
        assert!(!kick_into(
            &slot,
            KickRequest {
                scene: "OnTestScene".into(),
            }
        ));
        assert!(slot.load().is_none(), "kick must not fill the slot");
    }

    /// アクター無しの後始末は冪等の no-op（ハングなし）。
    #[test]
    fn teardown_without_actor_is_idempotent_noop() {
        let slot: ArcSwapOption<ActorInstance> = ArcSwapOption::const_empty();
        for _ in 0..2 {
            let report = teardown_slot(&slot, JoinPolicy::Detach, TEARDOWN_TIMEOUT);
            assert!(
                report.already_done && !report.acked && report.anomaly.is_none(),
                "teardown with no actor must be a safe idempotent no-op: {report:?}"
            );
        }
    }

    /// 実際に起こした芯の型を置き場に入れると kick が届き、後始末で置き場が空になると
    /// 届かなくなる。二度目の後始末は冪等の no-op。
    #[test]
    fn kick_reaches_real_instance_until_teardown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let instance = ActorInstance::spawn(0, dir.path().to_path_buf()).expect("spawn");
        let slot = ArcSwapOption::from_pointee(instance);

        assert!(kick_into(
            &slot,
            KickRequest {
                scene: "OnTestScene".into(),
            }
        ));

        let report = teardown_slot(&slot, JoinPolicy::Detach, TEARDOWN_TIMEOUT);
        assert!(report.acked && !report.joined, "{report:?}");
        assert!(slot.load().is_none(), "teardown must empty the slot");
        assert!(!kick_into(
            &slot,
            KickRequest {
                scene: "OnLate".into(),
            }
        ));

        let report = teardown_slot(&slot, JoinPolicy::Detach, TEARDOWN_TIMEOUT);
        assert!(report.already_done && !report.acked, "{report:?}");
    }

    /// 片付けの途中で別の参照が残っていても（規約に反する同時の呼び方）、停止の要求だけをして返る。
    #[test]
    fn teardown_with_shared_reference_only_requests_stop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let instance = ActorInstance::spawn(0, dir.path().to_path_buf()).expect("spawn");
        let slot = ArcSwapOption::from_pointee(instance);
        let in_flight = slot.load_full().expect("instance");

        let report = teardown_slot(&slot, JoinPolicy::Detach, TEARDOWN_TIMEOUT);
        assert!(report.acked && !report.joined, "{report:?}");
        assert!(!in_flight.kick("OnLate".into()), "actor must be stopped");
        // 最後の参照の Drop で手放す（Stop 済みなので待たずに返る）。
        drop(in_flight);
    }
}
