//! R5 実証テスト: talk FIFO・二層 gate・即時 preempt（task 5.2）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ、既定テストスイートを一切汚染しない（R7.1/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 5.2）:
//! 「FIFO 投入→GET tick drain で再生、`talking` 中は非即時抑止、即時は GET tick で
//!   上書き配信される統合テストが緑。」
//!
//! 正直さ（テストが本当に失敗しうること）の設計:
//! - 配信「再生」は **アクタースレッド上の実 VM に talk が現に乗った**（実
//!   `store.lua` の `STORE.co_scene` が当該シーンのコルーチンを保持し suspended で
//!   ある）ことを `run_lua_string` で観測する。芝居の bool ではなく実 VM 状態。
//! - 二層 gate は (1) 礼儀 gate（`set_talking(true)` 中は非即時 drain 抑止）と
//!   (2) 配信可否 gate（GET tick=Ref3=1 のみ配信／NOTIFY tick=Ref3=0 は held）を
//!   それぞれ独立に落としうる形で assert する（gate が誤動作すれば落ちる）。
//! - preempt は先行 talk を **実 store.lua の close 経路**（`STORE.reset()` と同一の
//!   `coroutine.close`＋参照破棄）で閉じてから上書きする。本 VM は LuaJIT（luajit52）で
//!   `coroutine.close` を持たないため、close の終端は `dead` ではなく **`closed`
//!   （参照破棄＝abandon・以後 resume されない）** である（kick_harness.rs の LuaJIT 所見
//!   参照）。閉じた先行 talk が live なアクティブ talk でなくなったことを実 VM 上で観測
//!   する（close しなければ先行は live のままで assert は落ちる）。
//! - 駆動主体は executor: 各 drain/配信は `SimDriver::tick` → `KickHarness` の
//!   `on_second_change` 経由で起こり、VM 操作は `run_lua_string` でアクタースレッドに
//!   閉じる。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::kick_harness::KickHarness;
use pasta_lua::actor_poc::sim_driver::SimDriver;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和。
/// 写経元: `crates/pasta_lua/tests/common/mod.rs`（PASTA_DEBUG=1 がテストを壊す既知問題）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// R5.1: FIFO 投入 → GET tick（Ref3=1）の OnSecondChange drain で FIFO 順に配信
/// （実 VM に talk が乗る）。NOTIFY tick（Ref3=0）では配信されず held。
#[test]
fn fifo_drains_and_delivers_on_get_tick_in_order() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();

    // FIFO に 2 シーンを投入（消費は OnSecondChange 排出点のみ）。
    kick.enqueue(101);
    kick.enqueue(102);

    // まず NOTIFY tick（Ref3=0・再生不能）: 配信可否 gate により配信されない。
    let notify_tick = sim.tick(false);
    let delivered_on_notify = kick.on_second_change(&sim, notify_tick);
    assert!(
        delivered_on_notify.is_empty(),
        "NOTIFY tick (Ref3=0) must NOT deliver — script would be ignored by SSP (held); got {delivered_on_notify:?}"
    );
    // held なので FIFO はまだ 2 件残っているはず。
    assert_eq!(
        kick.pending_len(),
        2,
        "kicks must be held in FIFO while only NOTIFY ticks arrive"
    );

    // GET tick（Ref3=1・再生可）: FIFO 排出点で配信される。
    let get_tick = sim.tick(true);
    let delivered = kick.on_second_change(&sim, get_tick);
    assert_eq!(
        delivered,
        vec![101, 102],
        "on a GET tick the FIFO must drain and deliver in enqueue order"
    );

    // 実 VM 上に最後の talk が現に乗っている（実 store.lua の co_scene 観測）。
    assert!(
        kick.active_talk_is_live(),
        "the delivered talk must be a live (suspended) coroutine on the actor VM (real co_scene)"
    );
    assert_eq!(
        kick.active_scene(),
        Some(102),
        "the active talk on the VM must be the last delivered scene"
    );

    kick.shutdown();
}

/// R5.2: 礼儀 gate。`set_talking(true)` 中は **非即時** enqueue の drain を抑止し、
/// 再生中トークへ割り込まない。talking 解除＋GET tick で配信される。
#[test]
fn politeness_gate_suppresses_non_immediate_drain_while_talking() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();

    // 再生中（talking）に非即時キックを投入。
    sim.set_talking(true);
    kick.enqueue(201);

    // talking 中は GET tick が来ても非即時 drain は礼儀 gate で抑止される。
    let get_tick_while_talking = sim.tick(true);
    let delivered_mid_talk = kick.on_second_change(&sim, get_tick_while_talking);
    assert!(
        delivered_mid_talk.is_empty(),
        "while talking, a non-immediate kick must be suppressed (no mid-talk interruption); got {delivered_mid_talk:?}"
    );
    assert_eq!(
        kick.pending_len(),
        1,
        "the suppressed kick must remain in the FIFO until talking clears"
    );

    // talking 解除 → 次の GET tick で配信される。
    sim.set_talking(false);
    let get_tick = sim.tick(true);
    let delivered = kick.on_second_change(&sim, get_tick);
    assert_eq!(
        delivered,
        vec![201],
        "after talking clears, the held kick must deliver on the next GET tick"
    );

    kick.shutdown();
}

/// R5.3: 即時 preempt。礼儀 gate を無視し、先行 talk を実 `coroutine.close()` で閉じ、
/// GET tick で上書き配信。NOTIFY 状態では次の GET tick まで遅延。
#[test]
fn immediate_preempt_closes_prior_talk_and_overwrites_on_get_tick() {
    let mut sim = SimDriver::new();
    let mut kick = KickHarness::start();

    // 先行 talk を載せる（GET tick で通常配信）。
    kick.enqueue(301);
    let t0 = sim.tick(true);
    let _ = kick.on_second_change(&sim, t0);
    assert_eq!(kick.active_scene(), Some(301), "prior talk must be active");
    assert!(kick.active_talk_is_live(), "prior talk must be live before preempt");
    let prior = kick.active_talk_token().expect("there must be a prior talk token");

    // 再生中（talking）でも preempt は礼儀 gate を無視する。
    sim.set_talking(true);

    // preempt 指示。配信可否 gate には従う: NOTIFY 状態では遅延。
    kick.preempt(999);
    let notify_tick = sim.tick(false);
    let delivered_on_notify = kick.on_second_change(&sim, notify_tick);
    assert!(
        delivered_on_notify.is_empty(),
        "preempt must still obey the delivery gate — on a NOTIFY tick it is delayed; got {delivered_on_notify:?}"
    );
    // 先行 talk はまだ閉じられていない（配信契機が来ていないため）。
    assert!(
        kick.active_talk_is_live(),
        "prior talk must remain live until the preempt actually delivers on a GET tick"
    );

    // GET tick: preempt が礼儀 gate を無視して上書き配信される。
    let get_tick = sim.tick(true);
    let delivered = kick.on_second_change(&sim, get_tick);
    assert_eq!(
        delivered,
        vec![999],
        "on a GET tick the preempt must overwrite and deliver with priority (ignoring politeness)"
    );

    // 先行 talk が **実 store.lua の close 経路で閉じられた（破棄された）**ことを実 VM
    // 上で観測する（LuaJIT では terminal = closed/abandon。dead 完走も真）。
    assert!(
        kick.talk_token_is_dead(prior),
        "the prior talk must have been closed/abandoned via the real store.lua close path, proving preempt closed it"
    );
    // 上書きされた新 talk が live で active であること。
    assert_eq!(
        kick.active_scene(),
        Some(999),
        "the preempting scene must be the active talk after overwrite"
    );
    assert!(
        kick.active_talk_is_live(),
        "the preempting talk must be live (suspended) on the actor VM"
    );

    kick.shutdown();
}
