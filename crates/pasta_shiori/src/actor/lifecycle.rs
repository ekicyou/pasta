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
//! - [`teardown_actor`]: 置き場から取り出して後始末する。空なら冪等の no-op。
//! - [`kick_sink`]: 置き場の芯の型へ kick を積む送り先。空なら `kick.drop` の診断ログで捨てる。
//!
//! # 送信の道にロックを置かない
//! 置き場は [`ArcSwapOption`] で、送信の道（[`marshal_request`]・[`kick_sink`]）は `load_full()`
//! で `Arc` を写すだけである。差し替え（[`spawn_actor`]・[`teardown_actor`]）だけが `store`／`swap`
//! する。VM はアクタースレッドから出ず、越境するのは `Send + Sync` な [`ActorInstance`] だけ。

use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwapOption;
use pasta_lua::debug::{KickRequest, KickSink};

use crate::actor::instance::{ActorInstance, JoinPolicy};
use crate::actor::marshaling::default_204;
use crate::actor::teardown::TeardownReport;

/// 旧来の入口の置き場。プロセスに 1 つの状態はこれだけ（5.7）。
static LEGACY: ArcSwapOption<ActorInstance> = ArcSwapOption::const_empty();

/// アクターを起こして置き場へ入れる（`load` 起点・loader lock の外）。
///
/// 前のインスタンスがあれば先に [`teardown_actor`] で片付ける（reload・7.2）。
/// 戻り値はアクタースレッド上での `PastaShiori::load` 成否。ロードに失敗してもインスタンスは
/// 置き場に入り、以後の GET の応答は失敗の種類で異なる（`PastaShiori::request` を参照）。
/// スレッドを起こせなかったとき（`SpawnError`）は偽を返し、置き場は空のまま。
pub fn spawn_actor(hinst: isize, load_dir: PathBuf) -> bool {
    teardown_actor();

    let Ok(instance) = ActorInstance::spawn(hinst, load_dir) else {
        // 理由は ActorInstance::spawn がログに出している。
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

/// 置き場のアクターを後始末する（`unload`／`DllMain detach` 入口・7.1）。
///
/// 戻り値は後始末の結果（clean／冪等 no-op／異常を値で返す。ホストを巻き込まない）。
/// この段では今どおり join しない（ack の後に join への切り替えは 5.1）。
pub fn teardown_actor() -> TeardownReport {
    teardown_slot(&LEGACY, JoinPolicy::Detach)
}

fn teardown_slot(slot: &ArcSwapOption<ActorInstance>, policy: JoinPolicy) -> TeardownReport {
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
        Ok(instance) => instance.shutdown(policy),
        // 片付けと同時に別のスレッドで要求が走っている（単一スレッドの規約に反する呼び方）。
        // 停止の要求だけをして、最後の参照の Drop でスレッドを手放す（join は保証しない）。
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

/// 旧来の入口の kick の送り先（置き場の芯の型へ委ねる）。
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
            let report = teardown_slot(&slot, JoinPolicy::Detach);
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

        let report = teardown_slot(&slot, JoinPolicy::Detach);
        assert!(report.acked && !report.joined, "{report:?}");
        assert!(slot.load().is_none(), "teardown must empty the slot");
        assert!(!kick_into(
            &slot,
            KickRequest {
                scene: "OnLate".into(),
            }
        ));

        let report = teardown_slot(&slot, JoinPolicy::Detach);
        assert!(report.already_done && !report.acked, "{report:?}");
    }

    /// 片付けの途中で別の参照が残っていても（規約に反する同時の呼び方）、停止の要求だけをして返る。
    #[test]
    fn teardown_with_shared_reference_only_requests_stop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let instance = ActorInstance::spawn(0, dir.path().to_path_buf()).expect("spawn");
        let slot = ArcSwapOption::from_pointee(instance);
        let in_flight = slot.load_full().expect("instance");

        let report = teardown_slot(&slot, JoinPolicy::Detach);
        assert!(report.acked && !report.joined, "{report:?}");
        assert!(!in_flight.kick("OnLate".into()), "actor must be stopped");
        // 最後の参照の Drop で手放す（Stop 済みなので待たずに返る）。
        drop(in_flight);
    }
}
