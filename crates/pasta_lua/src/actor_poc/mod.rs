//! pasta-actor-feasibility 使い捨て検証ハーネス（PoC）。
//!
//! `actor-poc` feature 有効時のみコンパイルされる隔離モジュール。
//! `wintf_winmsg_executor` 上に `!Send` な `mlua` VM をアクタースレッドとして
//! ホストし、reload teardown・block-on-reply marshaling・drop→204 ガード・
//! coroutine 生存・キック配信・GET レイテンシの go/no-go を実証する。
//!
//! 後続タスク（1.4 以降）が `actor_thread` / `mailbox` / `responder` 等の
//! サブモジュールをここに追加する。本モジュールは出荷コードを改変せず、
//! feature 無効時はコンパイル単位に現れない（リリースビルドはバイト不変）。

/// アクタースレッド（block_on ＋ `!Send` VM pin・JoinHandle/shutdown idiom・R1.1/R2.3）。
pub mod actor_thread;

/// reload teardown と反復リーク検査（shutdown→再 spawn を N 回・実ハンドル計測・R1.2/R1.3）。
pub mod teardown;

/// R1 ブロッカー記録経路（VM ホスト/teardown 不成立条件を切り分け record_blocker・R1.4）。
pub mod r1_probe;

/// テスト隔離土台（socket2 エフェメラル待受の写経・R7.4）。
pub mod test_isolation;

/// 単一直列 mailbox（enqueue／drain・FIFO 順序・スレッド分離・R2.3）。
pub mod mailbox;

/// GET oneshot responder ＋ Drop で未送信時 204 ガード（R3.1/R3.2/R3.3/R3.4）。
pub mod responder;

/// 段階判定レコーダ土台（項目別 outcome／blocker 累積・隔離前提・R8.2/R8.3/R7.3）。
pub mod verdict;

/// CoroutineProbe（executor 駆動下で実 `*.lua` の coroutine resume／callback 生存・R4.1〜R4.4）。
pub mod coroutine_probe;

/// SimDriver 忠実シミュレータ（OnSecondChange 周期＋GET/NOTIFY≡Reference3 自前タグ付け＋Status: talking 遷移・R5.6）。
pub mod sim_driver;

/// KickHarness（talk FIFO・二層 gate・即時 preempt・実 `coroutine.close()` 上書き・R5.1/R5.2/R5.3）。
pub mod kick_harness;

/// Latency（GET block-on-reply レイテンシ実測・代表値集計・フォールバック要否判断・R6.1/R6.2/R6.3）。
pub mod latency;

use mailbox::ActorMsg;
use responder::{PocResponse, Responder};
use verdict::{Blocker, ItemOutcome, ReqId, VerdictDocument, VerdictRecorder};

/// 段階判定オーケストレータ: 全 probe（R1〜R6）を **成否にかかわらず** 試行し、
/// 各項目／ブロッカーを `VerdictRecorder` へ記録し、隔離前提を確認したうえで段階を
/// 確定し判定文書を生成する（R8.1 / R8.2 / R8.3 / R8.4 / R8.5 / R7.3）。
///
/// design.md「Verdict … run_all」: `mod.rs`／`tests/actor_poc_*` から `run_all` で全項目を
/// 判定可能な形で実行する（R7.3）。本関数は実アクタースレッド／実 `!Send` VM を起動して
/// 各 probe を駆動し、判定可能な [`VerdictDocument`] を返す。
///
/// # 記録の経路（各 probe の公開 API を呼ぶ）
///
/// - **R1**: [`r1_probe::run_r1_probe`] → [`r1_probe::record_r1`]（VM ホスト＋reload
///   teardown を実駆動し item／ブロッカーを記録）。
/// - **R2/R3**: pasta_lua レベルのプリミティブ（[`actor_thread::ActorThread`] +
///   [`ActorMsg::Get`]/[`ActorMsg::Notify`] + [`Responder`]）で GET block-on-reply／
///   NOTIFY 即 204／drop→204 ガードを実駆動し item を記録する。SSP 側ラッパ `Marshal`
///   （`pasta_shiori::actor_poc::ffi_marshal`）は依存方向が逆（pasta_lua は pasta_shiori に
///   依存できない）ため、ここでは **同じプリミティブ** を直接駆動する（契約は同一）。
/// - **R4**: [`coroutine_probe::CoroutineProbe`]（resume_scene／resolve_callback を実駆動）。
/// - **R5**: [`kick_harness::KickHarness::measure_and_record`]（≤1 秒キック配信を実測記録）。
/// - **R6**: [`latency::Latency::sample_and_record`]（GET レイテンシ実測＋フォールバック判断）。
///
/// 全項目を成否にかかわらず試行する（R8.2）。いずれかの probe が不成立でも記録を残し、
/// 残りの probe も試行を続ける。
pub fn run_all() -> VerdictDocument {
    let mut rec = VerdictRecorder::new();

    // ---- R1: executor 上 VM ホスト＋reload teardown（本丸）----
    // 反復 reload を回して実測（過大にならない小さめのサイクル数で枯渇/リークを検査）。
    let r1_outcome = r1_probe::run_r1_probe(8);
    r1_probe::record_r1(&mut rec, &r1_outcome);

    // ---- R2 / R3: block-on-reply marshaling と drop→204 ガード ----
    // pasta_lua レベルのプリミティブで実駆動する（Marshal と同一契約・依存方向の制約）。
    record_r2_r3(&mut rec);

    // ---- R4: coroutine/callback 生存 ----
    record_r4(&mut rec);

    // ---- R5: ≤1 秒キック配信（忠実シミュレータ）----
    {
        use kick_harness::{KickHarness, LatencyFault};
        use sim_driver::SimDriver;
        let mut harness = KickHarness::start();
        let mut sim = SimDriver::new();
        // 既定パターン: 1 GET tick で配信（first_tick playable）。実測値を R5 item へ記録。
        let _ = harness.measure_and_record(&mut sim, 1, &[true], LatencyFault::None, &mut rec);
        harness.shutdown();
    }

    // ---- R6: GET レイテンシ実測＋フォールバック要否判断 ----
    {
        use latency::Latency;
        let mut lat = Latency::start();
        let _ = lat.sample_and_record(32, &mut rec);
        lat.shutdown();
    }

    // 隔離前提（R8.3）は render() 内で assert_isolation を取り込む。段階確定＋文書生成。
    rec.render()
}

/// R2（GET block-on-reply／NOTIFY 即 204）と R3（drop→204 ガード）を pasta_lua レベルの
/// プリミティブで実駆動し、item を記録する（R8.2）。
///
/// `Marshal`（pasta_shiori）は依存方向が逆で pasta_lua から呼べないため、その契約と
/// 同一のプリミティブ（[`ActorThread`](actor_thread::ActorThread)＋[`ActorMsg`]＋
/// [`Responder`]）をここで直接駆動する。観測:
/// - GET: 実 VM が値を返し block-on-reply が応答値を持ち帰る（R2.1）。
/// - NOTIFY: 即 204 fire-and-forget（応答経路を持たない・R2.2）。
/// - drop→204: GET 処理側が VM 失敗で responder を reply せず drop すると 204 が撃たれ、
///   SSP 側が無限待機しない（R3.1）。
fn record_r2_r3(rec: &mut VerdictRecorder) {
    use actor_thread::ActorThread;

    let actor = ActorThread::spawn();

    // R2.1: GET block-on-reply。実 VM が値を返し、応答値を持ち帰る。
    let (responder, reply_rx) = Responder::new();
    let get_ok = actor
        .submit(ActorMsg::Get {
            payload: 1,
            script: "return 7".to_string(),
            responder,
        })
        .is_ok()
        && matches!(reply_rx.recv(), Ok(PocResponse::Value(_)));

    // R2.2: NOTIFY 即 204 fire-and-forget（応答経路なし・enqueue が成立すれば契約成立）。
    let notify_ok = actor
        .submit(ActorMsg::Notify {
            payload: 2,
            script: "return 0".to_string(),
        })
        .is_ok();

    // R3.1: drop→204 ガード。VM が失敗（Lua error）すると actor は responder を reply せず
    // drop し、Drop ガードが 204 を撃つ。SSP 側 block-on-reply は無限待機せず 204 で終結。
    let (responder, reply_rx) = Responder::new();
    let drop_guard_ok = actor
        .submit(ActorMsg::Get {
            payload: 3,
            script: "error('R3 fault injection: drop without reply')".to_string(),
            responder,
        })
        .is_ok()
        && matches!(reply_rx.recv(), Ok(PocResponse::NoContent204));

    actor.shutdown();

    // R2 item: GET block-on-reply ∧ NOTIFY 即 204 の双方が成立すれば成功。
    if get_ok && notify_ok {
        rec.record_item(
            ReqId::item(2),
            ItemOutcome::success(
                "GET block-on-reply が応答値を持ち帰り、NOTIFY が即 204 fire-and-forget で\
                 終結（ActorThread+Mailbox+Responder プリミティブで実駆動・VM はアクター\
                 スレッドに pin）",
            ),
        );
    } else {
        rec.record_item(
            ReqId::item(2),
            ItemOutcome::failure(format!(
                "block-on-reply marshaling が不成立（GET={get_ok}・NOTIFY={notify_ok}）"
            )),
        );
        rec.record_blocker(
            ReqId::sub(2, 4),
            Blocker::new(format!(
                "GET block-on-reply（{get_ok}）／NOTIFY 即 204（{notify_ok}）の往復が成立しない"
            )),
        );
    }

    // R3 item: drop→204 ガードが無限待機を消したか。
    if drop_guard_ok {
        rec.record_item(
            ReqId::item(3),
            ItemOutcome::success(
                "応答未送信のまま responder を drop（VM 失敗）しても Drop ガードが 204 を撃ち、\
                 block-on-reply は無限待機せず 204 で終結（デッドロック経路の原理的消滅）",
            ),
        );
    } else {
        rec.record_item(
            ReqId::item(3),
            ItemOutcome::failure("drop→204 ガードが機能せず block-on-reply が 204 で終結しない"),
        );
        rec.record_blocker(
            ReqId::sub(3, 4),
            Blocker::new(
                "応答未送信 drop で 204 フォールバックが届かず SSP 側がブロックし続ける経路が残る",
            ),
        );
    }
}

/// R4（coroutine/callback 生存）を [`CoroutineProbe`](coroutine_probe::CoroutineProbe) で
/// 実駆動し、item を記録する（R8.2）。
///
/// executor 駆動下で実 `*.lua` の `STORE.co_scene` が中断地点から resume し、
/// `CALLBACK.pending` が後続契機で解決することを観測する。双方が成立すれば R4 成功。
fn record_r4(rec: &mut VerdictRecorder) {
    use coroutine_probe::CoroutineProbe;

    let probe = CoroutineProbe::start();
    let scene = probe.resume_scene();
    let callback = probe.resolve_callback();
    probe.shutdown();

    let scene_ok = scene.continued && scene.completed;
    let callback_ok = callback.resolved && callback.completed;

    if scene_ok && callback_ok {
        rec.record_item(
            ReqId::item(4),
            ItemOutcome::success(format!(
                "executor 駆動下で実 *.lua の co_scene が中断地点から継続（observed={:?}・\
                 {} drives）し CALLBACK.pending が後続契機で解決（実 *.lua 無改変・\
                 コルーチン意味論は Lua のまま）",
                scene.observed, scene.executor_drives
            )),
        );
    } else {
        rec.record_item(
            ReqId::item(4),
            ItemOutcome::failure(format!(
                "executor 駆動下で coroutine/callback の継続が不成立\
                 （co_scene 継続={scene_ok}・callback 解決={callback_ok}）"
            )),
        );
        rec.record_blocker(
            ReqId::sub(4, 4),
            Blocker::with_workaround(
                format!(
                    "executor 移設後に継続喪失を観測（co_scene={scene_ok}・callback={callback_ok}）"
                ),
                "co_scene／pending レジストリを VM 移設後も保持し、resume 駆動を executor へ確実に橋渡しする",
            ),
        );
    }
}
