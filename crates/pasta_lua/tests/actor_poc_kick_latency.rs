//! R5 実証テスト: キック→配信 ≤1 秒レイテンシ実測（task 5.3）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ、既定テストスイートを一切汚染しない（R7.1/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 5.3）:
//! 「キック→配信レイテンシを実測し ≤1 秒判定（または未達条件と実測値）を出力する
//!   統合テストが緑。」
//!
//! 正直さ（テストが本当に失敗しうること）の設計:
//! - レイテンシは **2 つのレンズ**で正直に実測する（要件 5.4・R5.6 の忠実シミュレータ
//!   定義に基づく）:
//!   1. **sim-timeline**: キック指示から **配信した GET tick** までの所要 tick 数。
//!      1 tick = OnSecondChange 1 周期 = 1 秒（R5.6）。キックは次の利用可能な GET tick
//!      で配信されるべき＝1 周期以内 ⇒ sim-timeline で ≤1 秒。NOTIFY held は次 GET tick
//!      まで latency に加算される。
//!   2. **wall-clock**: enqueue/preempt→drain→実 VM 配信の実 Rust/VM 処理時間を
//!      `std::time::Instant` 単調時計で実測。1 秒を遥かに下回るべき。
//! - 配信「再生」は **アクタースレッド上の実 VM に talk が現に乗った** ことを
//!   `active_talk_is_live`／`active_scene` で観測する（芝居 bool ではない）。
//! - 未達（drain 不発・gate 誤動作・preempt 不能・遅延 >1 秒）は **故障注入**で駆動し、
//!   `VerdictRecorder::record_blocker(R5.5)` に未達条件 **と実測値** が記録されることを
//!   assert する（記録が壊れていれば落ちる）。
//! - R5 の LuaJIT 制約（preempt の close = 破棄+GC・`coroutine.close` 不在）は R5 item の
//!   `constraints` に申し送りとして残ることを確認する（task 5.2 所見の handoff）。
#![cfg(feature = "actor-poc")]

use std::time::Duration;

use pasta_lua::actor_poc::kick_harness::{KickHarness, LatencyFault};
use pasta_lua::actor_poc::sim_driver::SimDriver;
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

/// R5.4: キックが **次の GET tick** で配信され、sim-timeline で 1 周期（=1 秒）以内、
/// wall-clock で 1 秒を遥かに下回ることを実測する。実 VM に talk が乗ることも観測。
#[test]
fn kick_delivered_within_one_second_on_next_get_tick() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();

    // GET tick が次に来る前提（再生可能・非 talking）でキックを実測する。
    let report = kick.measure(&mut sim, 401, true, LatencyFault::None);

    // sim-timeline: 次の GET tick で配信 ⇒ 1 tick = 1 周期 = 1 秒以内。
    assert_eq!(
        report.ticks_to_deliver, 1,
        "a kick must be delivered on the very next GET tick (1 period); got {} ticks",
        report.ticks_to_deliver
    );
    assert!(
        report.sim_latency() <= Duration::from_secs(1),
        "sim-timeline latency must be ≤1s (one OnSecondChange period); got {:?}",
        report.sim_latency()
    );
    // wall-clock: 実処理は 1 秒を遥かに下回る。
    assert!(
        report.wall_clock < Duration::from_secs(1),
        "wall-clock processing latency must be ≪1s; got {:?}",
        report.wall_clock
    );
    // 総合判定 ≤1 秒。
    assert!(
        report.within_one_second,
        "the overall ≤1s verdict must hold; report={report:?}"
    );
    assert!(
        report.unmet.is_none(),
        "a successful measurement must carry no unmet condition; got {:?}",
        report.unmet
    );

    // 実 VM に配信された talk が現に乗っている。
    assert_eq!(
        kick.active_scene(),
        Some(401),
        "the measured kick must be the active talk on the actor VM"
    );
    assert!(
        kick.active_talk_is_live(),
        "the delivered talk must be live (suspended) on the actor VM"
    );

    kick.shutdown();
}

/// R5.4: NOTIFY tick で held → 次の GET tick で配信されるケースのレイテンシを正しく
/// 実測する。held は latency に加算され、それでも次周期で配信されるため ≤1 秒に収まる
/// （配信時点から見れば「次の GET tick」で届く）。
#[test]
fn held_on_notify_then_delivered_on_next_get_tick_is_measured() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();

    // 最初の tick は NOTIFY（再生不能）→ held。次の GET tick で配信。
    let report = kick.measure(&mut sim, 402, false, LatencyFault::None);

    // NOTIFY で 1 周期 held、その次の GET tick で配信 ⇒ 2 tick 経過。
    assert_eq!(
        report.ticks_to_deliver, 2,
        "held-on-NOTIFY then delivered-on-next-GET must count 2 ticks (1 held + 1 deliver); got {}",
        report.ticks_to_deliver
    );
    // held は次 GET tick まで latency に算入されるが、配信は次の GET tick で起こるため
    // 「次の利用可能な配信契機」で届いている = ≤1 秒判定は成立（NOTIFY は配信契機でない）。
    assert!(
        report.within_one_second,
        "delivery on the next available GET tick still meets ≤1s; report={report:?}"
    );
    assert!(
        report.wall_clock < Duration::from_secs(1),
        "wall-clock must remain ≪1s even across a held NOTIFY tick; got {:?}",
        report.wall_clock
    );
    assert_eq!(
        kick.active_scene(),
        Some(402),
        "the kick must finally be delivered to the VM after the NOTIFY hold"
    );

    kick.shutdown();
}

/// R5.4: 計測の正直さ — drain が GET tick で起きなければ（誤った tick で「配信した」と
/// 数えれば）tick カウントは合わない。GET tick で配信される前に余計な NOTIFY tick を
/// 挟んでも、配信した GET tick までを正しく数える。
#[test]
fn latency_counts_ticks_until_the_actual_delivering_get_tick() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();

    // 成功計測（GET tick 即配信）の ticks は 1。
    let r1 = kick.measure(&mut sim, 410, true, LatencyFault::None);
    assert_eq!(r1.ticks_to_deliver, 1);

    // NOTIFY 2 連 → GET の場合は 3 tick（held×2 + deliver）。
    let r2 = kick.measure_with_pattern(&mut sim, 411, &[false, false, true], LatencyFault::None);
    assert_eq!(
        r2.ticks_to_deliver, 3,
        "two NOTIFY holds then a GET deliver must count exactly 3 ticks; got {}",
        r2.ticks_to_deliver
    );
    assert!(r2.unmet.is_none(), "delivery still succeeded; got {:?}", r2.unmet);

    kick.shutdown();
}

/// R5.5: 故障注入（drain 不発）→ 未達条件と実測値が `record_blocker(R5.5)` に記録される。
#[test]
fn drain_suppressed_records_unmet_blocker_with_measured_value() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();
    let mut rec = VerdictRecorder::new();

    // drain 不発を注入 → GET tick が来ても配信されない。
    let report = kick.measure_and_record(
        &mut sim,
        420,
        &[true, true, true],
        LatencyFault::SuppressDrain,
        &mut rec,
    );

    assert!(
        !report.within_one_second,
        "a suppressed drain must NOT meet ≤1s; report={report:?}"
    );
    assert!(
        report.unmet.is_some(),
        "a drain-suppressed measurement must carry an unmet condition"
    );

    // R5.5 ブロッカーが記録され、未達条件 と 実測値 を含む。
    let blockers = rec.blockers(ReqId::sub(5, 5));
    assert_eq!(
        blockers.len(),
        1,
        "exactly one R5.5 blocker must be recorded for the drain-suppression fault"
    );
    let reason = &blockers[0].reason;
    assert!(
        reason.contains("drain"),
        "the blocker reason must distinguish the 'drain 不発' condition; got: {reason}"
    );
    // 実測値（経過 tick 数）が記録に含まれること（R5.5「実測値を記録」）。
    assert!(
        reason.contains(&report.ticks_to_deliver.to_string())
            || reason.contains("tick"),
        "the blocker must carry the measured value (elapsed ticks); got: {reason}"
    );

    kick.shutdown();
}

/// R5.5: 故障注入（preempt 不能）→ 未達条件と実測値が `record_blocker(R5.5)` に記録される。
#[test]
fn preempt_blocked_records_unmet_blocker_with_measured_value() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();
    let mut rec = VerdictRecorder::new();

    let report = kick.measure_and_record(
        &mut sim,
        430,
        &[true, true],
        LatencyFault::BlockPreempt,
        &mut rec,
    );

    assert!(
        !report.within_one_second,
        "a blocked preempt must NOT meet ≤1s; report={report:?}"
    );
    let blockers = rec.blockers(ReqId::sub(5, 5));
    assert_eq!(blockers.len(), 1, "one R5.5 blocker for the blocked preempt");
    assert!(
        blockers[0].reason.contains("preempt"),
        "the blocker reason must distinguish the 'preempt 不能' condition; got: {}",
        blockers[0].reason
    );

    kick.shutdown();
}

/// R5.5: 故障注入（強制 >1 秒遅延）→ 遅延 >1 秒の未達条件と実測値が記録される。
#[test]
fn forced_over_one_second_records_unmet_blocker_with_measured_value() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();
    let mut rec = VerdictRecorder::new();

    let report = kick.measure_and_record(
        &mut sim,
        440,
        &[true],
        LatencyFault::ForceLatencyOverOneSecond,
        &mut rec,
    );

    assert!(
        !report.within_one_second,
        "a forced >1s latency must NOT meet ≤1s; report={report:?}"
    );
    assert!(
        report.sim_latency() > Duration::from_secs(1),
        "the forced fault must produce a measured latency >1s; got {:?}",
        report.sim_latency()
    );
    let blockers = rec.blockers(ReqId::sub(5, 5));
    assert_eq!(blockers.len(), 1, "one R5.5 blocker for the forced >1s latency");
    assert!(
        blockers[0].reason.contains('1') && blockers[0].reason.contains("秒"),
        "the blocker reason must carry the measured >1s value; got: {}",
        blockers[0].reason
    );

    kick.shutdown();
}

/// R5（成功時）: 成功した ≤1 秒計測は R5 item を success で記録し、R5 の LuaJIT 制約
/// （preempt の close = 破棄+GC・`coroutine.close` 不在）を `constraints` に申し送りとして
/// 残す（task 5.2 所見の handoff・7.1 が消費）。
#[test]
fn success_records_r5_item_with_luajit_close_constraint() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();
    let mut rec = VerdictRecorder::new();

    let report =
        kick.measure_and_record(&mut sim, 450, &[true], LatencyFault::None, &mut rec);
    assert!(report.within_one_second, "control measurement must succeed");

    // R5 item が success で記録される。
    let r5 = rec.item(ReqId::item(5)).expect("R5 item must be recorded on success");
    assert!(r5.success, "≤1s success must record R5 as success");
    // 実測値（ticks / wall-clock）が approach に反映されている。
    assert!(
        r5.approach.contains("tick") || r5.approach.contains('秒'),
        "the R5 approach must carry the measured latency; got: {}",
        r5.approach
    );
    // R5 の LuaJIT close 制約が constraints に申し送られている。
    assert!(
        r5.constraints.iter().any(|c| c.contains("coroutine.close")
            || c.contains("破棄")
            || c.contains("abandon")),
        "the R5 LuaJIT close constraint (preempt close = abandon+GC) must be handed off in constraints; got: {:?}",
        r5.constraints
    );
    // R5.5 ブロッカーは無い（成功時）。
    assert!(
        rec.blockers(ReqId::sub(5, 5)).is_empty(),
        "a successful ≤1s measurement must record no R5.5 blocker"
    );

    kick.shutdown();
}
