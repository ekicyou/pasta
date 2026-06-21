//! R4 実証テスト: executor 駆動下での実コルーチン／callback 生存（task 4.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ、既定テストスイートを一切汚染しない（R7.1/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 4.1）:
//! 「executor 駆動下で `co_scene` が中断地点から継続し `CALLBACK` が解決する
//!   統合テストが緑。」
//!
//! 正直さ（テストが本当に失敗しうること）の設計:
//! - コルーチン／callback の意味論は **無改変の実 `*.lua`**（`store.lua` の
//!   `STORE.co_scene`、`event/init.lua` の `resume_until_valid`／`set_co_scene`、
//!   `callback.lua` の `CALLBACK.pending`／`stage_pending`／`consume_staged`／
//!   `try_route`）から来る。Rust/Lua の再実装ではない（design「境界線」R4）。
//! - **駆動主体は executor**。各 resume／callback 解決は、アクタースレッドへ投入した
//!   別個のコマンド＝別個の executor ポーリングで起こる（ホスト SHIORI tick ではない）。
//! - co_scene 継続テストは「中断地点の次から再開した」ことを観測する（カウンタが
//!   2→3→… と進む。1 にリセットされない）。リセット/喪失すれば assert は落ちる。
//! - callback テストは pending 登録→後続契機 resume→完了を観測する。解決しなければ
//!   `dead` に到達せず assert は落ちる。
//! - 喪失注入（4.4）は `VerdictRecorder` の R4.4 ブロッカーが記録されることを確認する。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::coroutine_probe::{CoroutineProbe, LossKind};
use pasta_lua::actor_poc::verdict::{ReqId, VerdictRecorder};

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和。
/// 写経元: `crates/pasta_lua/tests/common/mod.rs`（PASTA_DEBUG=1 がテストを壊す既知問題）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// R4.1: executor 駆動でシーンコルーチン（`STORE.co_scene` モデル）を中断地点から
/// resume し、継続実行（中断の次から再開）することを確認する。
#[test]
fn co_scene_continues_from_suspension_under_executor_drive() {
    let probe = CoroutineProbe::start();

    let result = probe.resume_scene();

    // 中断地点から継続した観測値（カウンタが進んだ列）。リセットされていない。
    assert!(
        result.continued,
        "co_scene must continue from its suspension point under executor drive (not restart)"
    );
    // 実シーン継続モデル: yield ごとに 1,2,3 と進み、最後に完了（dead）する。
    assert_eq!(
        result.observed, vec![1, 2, 3],
        "the coroutine must progress past each yield (1→2→3), proving real continuation"
    );
    assert!(
        result.completed,
        "the coroutine must reach completion (dead) after the final resume"
    );
    // 駆動主体が executor であった証跡: 複数回の executor ポーリング（drive）を要した。
    assert!(
        result.executor_drives >= 3,
        "each resume must be a distinct executor drive (>= 3), not a single host tick; got {}",
        result.executor_drives
    );

    probe.shutdown();
}

/// R4.2: executor 駆動下で非同期 callback（`CALLBACK.pending` モデル）を発行し、
/// 後続の駆動契機で resume して callback が解決しコルーチンが継続することを確認する。
#[test]
fn callback_resolves_on_later_executor_drive() {
    let probe = CoroutineProbe::start();

    let result = probe.resolve_callback();

    assert!(
        result.staged,
        "the coroutine must stage a pending callback (CALLBACK.stage_pending) and suspend"
    );
    assert!(
        result.was_pending_before_resolution,
        "CALLBACK.pending must hold the waiting coroutine before the later drive resolves it"
    );
    assert!(
        result.resolved,
        "the pending callback must resolve on a later executor drive (CALLBACK.try_route)"
    );
    assert!(
        result.completed,
        "the coroutine must continue to completion after the callback resolves"
    );
    // 解決後 pending から除去されている（実 try_route が pending を消費した）。
    assert!(
        result.pending_cleared_after,
        "CALLBACK.pending must no longer hold the entry after resolution"
    );

    probe.shutdown();
}

/// R4.3: コルーチン／callback ロジックが **実 `*.lua`** であることを実証する。
/// 実モジュールから読み込んだ関数／テーブルが VM 上に実在し、実際に駆動された
/// ことを確認する（Rust 再実装ではない証跡）。
#[test]
fn drives_real_lua_modules_not_a_reimplementation() {
    let probe = CoroutineProbe::start();

    let fidelity = probe.assert_real_lua_loaded();

    // store.lua: STORE は実モジュールで、co_scene フィールドを持つ。
    assert!(
        fidelity.store_has_co_scene_field,
        "real store.lua must expose STORE.co_scene (loaded from disk, not reimplemented)"
    );
    // callback.lua: CALLBACK.pending テーブル＋実関数群が存在する。
    assert!(
        fidelity.callback_has_pending_table,
        "real callback.lua must expose CALLBACK.pending"
    );
    assert!(
        fidelity.callback_has_real_functions,
        "real callback.lua must expose stage_pending/consume_staged/try_route/next_event_id"
    );
    // event/init.lua: resume_until_valid が公開関数として実在する。
    assert!(
        fidelity.event_has_resume_until_valid,
        "real event/init.lua must expose _resume_until_valid"
    );
    // second_change.lua: REG.OnSecondChange（drain/sweep 契機）が登録されている。
    assert!(
        fidelity.second_change_registered,
        "real second_change.lua must register REG.OnSecondChange"
    );

    probe.shutdown();
}

/// R4.4: 継続喪失（resume 不能／状態喪失）を注入したとき、`VerdictRecorder` の
/// R4.4 にブロッカーが NO-GO 根拠として記録されることを確認する。
#[test]
fn injected_continuation_loss_records_r4_4_blocker() {
    let mut rec = VerdictRecorder::new();
    let probe = CoroutineProbe::start();

    // 喪失注入: co_scene を resume 前に喪失させる（state lost）と継続不能になる。
    let blocker = probe.record_loss(&mut rec, LossKind::CoSceneLost);

    assert!(
        blocker.is_some(),
        "an injected continuation loss must be classified and recorded as a blocker"
    );

    let blockers = rec.blockers(ReqId::sub(4, 4));
    assert_eq!(
        blockers.len(),
        1,
        "the loss must record exactly one R4.4 blocker (NO-GO ground)"
    );
    assert!(
        blockers[0].reason.contains("co_scene"),
        "the blocker reason must identify the lost-continuation condition, got: {}",
        blockers[0].reason
    );

    probe.shutdown();
}

/// 喪失注入の分類が互いに区別される（generic でない）ことを確認する。
#[test]
fn loss_kinds_are_distinguished_under_r4_4() {
    let mut rec = VerdictRecorder::new();
    let probe = CoroutineProbe::start();

    probe.record_loss(&mut rec, LossKind::CoSceneLost);
    probe.record_loss(&mut rec, LossKind::CallbackPendingLost);

    let blockers = rec.blockers(ReqId::sub(4, 4));
    assert_eq!(blockers.len(), 2, "both injected losses must be recorded under R4.4");
    assert!(
        blockers.iter().any(|b| b.reason.contains("co_scene")),
        "co_scene loss must be distinguished"
    );
    assert!(
        blockers.iter().any(|b| b.reason.contains("CALLBACK")),
        "callback pending loss must be distinguished"
    );

    probe.shutdown();
}
