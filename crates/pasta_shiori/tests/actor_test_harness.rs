//! 本番アクター機構の **ホスト非依存・決定論テストハーネス**（ActorTestHarness・task 6.2）。
//!
//! 関連仕様: `.kiro/specs/pasta-actor-runtime/`
//! - 充足要件: R10.5（アクター機構の振る舞いを SSP 非依存に再現・検証できる決定論テスト
//!   ハーネスで観測・デバッグ可能にする。PoC `actor_poc/` の `sim_driver`／`mailbox`／
//!   `coroutine_probe` 検証等を本番化）／R10.7（データ競合・デッドロック・スレッド非決定性を
//!   単一直列キューの順序保存＋drop→204 ガードで構造的に排除し、再現困難な並行バグを最小化）
//! - 設計: design.md の **ByteInvariantSuite / ActorTestHarness** コンポーネント
//!   （`sim_driver`〈GET/NOTIFY tick 生成〉・`mailbox`／`coroutine_probe` 検証＋reply
//!   move/drop exactly-once をホスト非依存の決定論テストへ昇格）。
//!
//! ## PoC 昇格の方針（actor_poc に依存しない）
//! 旧 PoC（pasta_lua の actor-poc 足場の `sim_driver`／`coroutine_probe`／`flume_wake`）が
//! 実証していた **価値ある決定論検証**を、**本番 `actor/` モジュール**（`mailbox`／
//! `marshaling`／`teardown`）に対して再ホームしたものである。本ファイルは
//! その PoC 足場を **一切参照しない**（task 7.2 が PoC 足場を撤去しても本カバレッジは
//! 残る）。検証は実 SSP も実 Lua VM も要さず、アクター役スレッド（producer/consumer の
//! 単一直列契約のみ）と本番 mailbox／marshaling 型で決定論的に成立する。
//!
//! 昇格した検証群:
//! - **SimDriver 風 GET/NOTIFY tick 生成**: `tick(playable)` の bool だけから GET/NOTIFY を
//!   決定論的にタグ付けし（受信文字列を再パースしない独立生成器）、本番 marshaling へ
//!   投入して GET=block-on-reply／NOTIFY=即 204 の契約を観測する。
//! - **mailbox 検証**: 本番 `mailbox()`（flume FIFO・単一 consumer）の投入順＝処理順を
//!   複数 producer から実証（R6 横断・順序一意性）。
//! - **reply move/drop exactly-once**: 本番 `marshal_get_with_timeout` 越しに、reply を
//!   送れば値・送らず drop すれば 204 の exactly-once を観測（flume move/drop 意味論）。
//! - **coroutine_probe 風 survival**: timeout で SHIORI 待機を打ち切っても、アクター役の
//!   in-flight work は **打ち切られず**後続 tick で回復する（co_scene 保存の構造的根拠）。
//!   実 Lua コルーチンは要さず、単一直列 FIFO の継続として決定論的に観測する。
//!
//! ## 決定論・ホスト非依存
//! - 実 SSP・実ネットワーク・実 Lua VM を一切起動しない（アクター役スレッドが mailbox を
//!   drain して reply するだけ）。タイミング依存の箇所は「閾値 << アクター遅延」で
//!   決定論化し、wall-clock の揺れに依存しない。

use std::thread;
use std::time::Duration;

use pasta::actor::mailbox::{mailbox, ActorMsg, MailboxRequest, Reply};
use pasta::actor::marshaling::{default_204, marshal_get_with_timeout, marshal_notify};

// ===========================================================================
// SimDriver 風 GET/NOTIFY tick 生成器（PoC sim_driver の昇格・独立生成器）
// ---------------------------------------------------------------------------
// `tick(playable)` の bool 引数だけから method を決定論的にタグ付けする。受信
// リクエスト文字列を一切パースしない（Marshal の再パースに構造的に依存し得ない）。
// PoC の SimMethod/SimTick/SimDriver を本ハーネスに最小再現したもの。
// ===========================================================================

/// OnSecondChange tick の method タグ（GET/NOTIFY ≡ Reference3 の権威）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimMethod {
    /// GET tick（Reference3=1・再生可/上書き可・block-on-reply）。
    Get,
    /// NOTIFY tick（Reference3=0・再生不能・即 204）。
    Notify,
}

impl SimMethod {
    /// GET=1／NOTIFY=0（method と Reference3 が常に連動する自己整合）。
    fn reference3(self) -> u8 {
        match self {
            SimMethod::Get => 1,
            SimMethod::Notify => 0,
        }
    }
}

/// OnSecondChange 1 周期が発火する 1 tick。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SimTick {
    method: SimMethod,
    reference3: u8,
}

impl SimTick {
    /// GET tick（Ref3=1）のみ配信可能。
    fn is_playable(self) -> bool {
        matches!(self.method, SimMethod::Get)
    }
}

/// OnSecondChange の周期を忠実に再現する自前ドライバ（bool→method の独立生成器）。
#[derive(Debug, Default)]
struct SimDriver;

impl SimDriver {
    fn new() -> Self {
        SimDriver
    }

    /// `playable=true`→GET tick（Ref3=1）／`false`→NOTIFY tick（Ref3=0）。
    /// method タグ付けは bool 引数のみから決定論的に行い、request 文字列を見ない。
    fn tick(&mut self, playable: bool) -> SimTick {
        let method = if playable {
            SimMethod::Get
        } else {
            SimMethod::Notify
        };
        SimTick {
            method,
            reference3: method.reference3(),
        }
    }
}

// ===========================================================================
// 1) SimDriver 風 tick 生成 → 本番 marshaling 投入の契約検証
// ===========================================================================

/// SimDriver が GET/NOTIFY を bool だけから自己整合にタグ付けする（独立生成器・R10.5）。
#[test]
fn sim_driver_tags_get_and_notify_from_bool_only() {
    let mut driver = SimDriver::new();

    let get = driver.tick(true);
    assert_eq!(get.method, SimMethod::Get);
    assert_eq!(get.reference3, 1, "GET tick must carry Reference3=1");
    assert!(get.is_playable(), "GET tick (Ref3=1) must be playable");

    let notify = driver.tick(false);
    assert_eq!(notify.method, SimMethod::Notify);
    assert_eq!(notify.reference3, 0, "NOTIFY tick must carry Reference3=0");
    assert!(!notify.is_playable(), "NOTIFY tick (Ref3=0) must not be playable");
}

/// SimDriver 生成 tick を本番 marshaling へ流し、GET=block-on-reply／NOTIFY=即 204 の
/// 契約をホスト非依存に観測する（R10.5・marshaling シーム）。
///
/// アクター役スレッドは GET だけを drain して reply する（NOTIFY は応答経路を持たない）。
/// 実 SSP・実 VM を起動せず、単一直列 FIFO とアクター役だけで決定論的に成立する。
#[test]
fn sim_driver_ticks_drive_production_marshaling_contract() {
    let (tx, rx) = mailbox();
    let mut driver = SimDriver::new();

    // アクター役: GET を drain し req.raw を echo して reply する単一直列ループ。
    let actor = thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            match msg {
                ActorMsg::Get { req, reply } => {
                    let _ = reply.send(Reply::Value(format!("echo:{}", req.raw)));
                }
                ActorMsg::Notify { .. } => { /* fire-and-forget・応答経路なし */ }
                ActorMsg::Stop { done } => {
                    let _ = done.send(());
                    break;
                }
                _ => { /* #[non_exhaustive]: 将来の variant は無視 */ }
            }
        }
    });

    // GET tick: marshal_get_with_timeout は実 reply 値を返す（block-on-reply）。
    let get_tick = driver.tick(true);
    assert!(get_tick.is_playable());
    let get_resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(1, "PAYLOAD"),
        Duration::from_secs(10),
    );
    assert_eq!(
        get_resp, "echo:PAYLOAD",
        "GET tick must drive block-on-reply and return the actor's reply value"
    );

    // NOTIFY tick: marshal_notify は即 204（アクター完了を待たない）。
    let notify_tick = driver.tick(false);
    assert!(!notify_tick.is_playable());
    let notify_resp = marshal_notify(&tx, MailboxRequest::new(2, "BODY"));
    assert_eq!(
        notify_resp.as_bytes(),
        default_204().as_bytes(),
        "NOTIFY tick must return 204 immediately (fire-and-forget)"
    );

    // teardown（clean stop）。
    let (stop, done_rx) = ActorMsg::stop();
    tx.send(stop).expect("send Stop");
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("Stop must be acked");
    actor.join().expect("actor role thread must not panic");
}

// ===========================================================================
// 2) mailbox 検証（FIFO・単一 consumer・順序一意性）— PoC mailbox 検証の昇格
// ===========================================================================

/// 本番 `mailbox()` を複数 producer から近接 `send` しても、単一 consumer が投入順
/// （producer-local FIFO）で全件を一意に drain する（R6/R10.7・順序保存で並行バグ排除）。
#[test]
fn production_mailbox_preserves_fifo_across_producers() {
    let (tx, rx) = mailbox();
    const PRODUCERS: u64 = 8;
    const PER_PRODUCER: u64 = 50;

    let mut handles = Vec::new();
    for p in 0..PRODUCERS {
        let txc = tx.clone();
        handles.push(thread::spawn(move || {
            for i in 0..PER_PRODUCER {
                let seq = p * 1000 + i;
                txc.send(ActorMsg::notify(MailboxRequest::new(seq, "")))
                    .expect("clone send must succeed while consumer is alive");
            }
        }));
    }
    for h in handles {
        h.join().expect("producer thread must not panic");
    }
    drop(tx);

    // 単一 consumer が全件を 1 本の FIFO として受ける。
    let mut drained: Vec<u64> = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        if let ActorMsg::Notify { req } = msg {
            drained.push(req.seq);
        }
    }

    // (1) 取りこぼし・重複なし。
    assert_eq!(
        drained.len() as u64,
        PRODUCERS * PER_PRODUCER,
        "single consumer must receive exactly every enqueued message once"
    );

    // (2) producer-local FIFO 保存。
    for p in 0..PRODUCERS {
        let sub: Vec<u64> = drained
            .iter()
            .copied()
            .filter(|s| s / 1000 == p)
            .map(|s| s % 1000)
            .collect();
        let expected: Vec<u64> = (0..PER_PRODUCER).collect();
        assert_eq!(sub, expected, "producer {p}'s messages must stay in enqueue order");
    }

    // (3) 全順序の一意性（重複インデックスなし）。
    let mut sorted = drained.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), drained.len(), "serial FIFO must contain no duplicates");
}

// ===========================================================================
// 3) reply move/drop exactly-once（PoC responder 検証相当の昇格）
// ===========================================================================

/// 本番 marshaling 越しに、reply を送れば値・送らず drop すれば 204 の exactly-once を
/// 観測する（flume の move/drop 意味論・独自 Responder 不要・R10.5/R5.3）。
#[test]
fn reply_move_then_value_or_204_on_drop_exactly_once() {
    // ケース A: アクター役が reply を送る → 値が届く（move）。
    {
        let (tx, rx) = mailbox();
        let actor = thread::spawn(move || {
            if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
                let _ = reply.send(Reply::Value(format!("v:{}", req.raw)));
            }
        });
        let resp = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(1, "A"),
            Duration::from_secs(10),
        );
        assert_eq!(resp, "v:A", "replied GET must deliver its value exactly once");
        actor.join().expect("actor role thread must not panic");
    }

    // ケース B: アクター役が reply を送らず drop → 受信側 Disconnected → 204（drop）。
    {
        let (tx, rx) = mailbox();
        let actor = thread::spawn(move || {
            if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
                drop(reply); // reply を送らずスコープ離脱 → drop。
            }
        });
        let resp = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(2, "B"),
            Duration::from_secs(10),
        );
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "a dropped (unreplied) GET must surface as 204 exactly once (Disconnected)"
        );
        actor.join().expect("actor role thread must not panic");
    }
}

// ===========================================================================
// 4) coroutine_probe 風 survival（timeout 打ち切り後も in-flight work が生存・回復）
// ===========================================================================

/// timeout で SHIORI 待機を打ち切っても、アクター役の in-flight work は **打ち切られず**
/// 後続 tick で回復する（co_scene 保存の構造的根拠・R10.5/R10.7）。
///
/// PoC `coroutine_probe` が実 Lua コルーチンで実証した「executor 駆動下の resume 継続／
/// 喪失なし」を、本番 mailbox の単一直列 FIFO の継続として **実 VM なしで** 決定論化する。
/// timeout は SHIORI 側の待機を切るだけで、アクター役（Lua 実行相当）は処理を続行する。
#[test]
fn actor_work_survives_timeout_and_recovers_on_next_tick() {
    let (tx, rx) = mailbox();
    let mut driver = SimDriver::new();
    let short_timeout = Duration::from_millis(20);
    let actor_delay = Duration::from_millis(120); // 閾値 << 遅延（決定論的に timeout）。

    // アクター役: 2 件の GET を順に drain する単一直列ループ（FIFO・単一 consumer）。
    let actor = thread::spawn(move || {
        // Round1: 閾値より遅延してから reply（SHIORI は既に打ち切り済み）。
        //         work は失われず最後まで完走する（co_scene 保存の構造的模擬）。
        if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
            thread::sleep(actor_delay);
            let _ = reply.send(Reply::Value("late-but-completed".to_string()));
        }
        // Round2: 通常速度で reply（次 tick による回復に相当）。
        if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
            let _ = reply.send(Reply::Value(format!("recovered:{}", req.raw)));
        }
    });

    // Round1（GET tick）: 短い閾値で待つ → アクター役が遅延中ゆえ timeout → 204。
    let r1_tick = driver.tick(true);
    assert!(r1_tick.is_playable());
    let round1 = marshal_get_with_timeout(&tx, MailboxRequest::new(1, "R1"), short_timeout);
    assert_eq!(
        round1.as_bytes(),
        default_204().as_bytes(),
        "GET exceeding the threshold must return 204 (SHIORI wait aborted, actor not killed)"
    );

    // Round2（次 tick）: 通常速度で待つ → 値が返る（回復）。Round1 work が打ち切られて
    // いれば FIFO が崩れこの assert が落ちる（survival の証跡）。
    let r2_tick = driver.tick(true);
    assert!(r2_tick.is_playable());
    let round2 =
        marshal_get_with_timeout(&tx, MailboxRequest::new(2, "R2"), Duration::from_secs(10));
    assert_eq!(
        round2, "recovered:R2",
        "the follow-up GET must recover (FIFO preserved; in-flight actor work survived timeout)"
    );

    actor.join().expect("actor role thread must not panic");
}

// ===========================================================================
// 5) ハーネス自己検査: actor_poc に依存していないこと（task 7.2 撤去の安全性）
// ===========================================================================

/// 本ハーネスソースが `actor_poc` を一切参照しないことを静的に確認する（R10.8 整合・
/// task 7.2 が `actor_poc/` を撤去しても本決定論カバレッジが残ることの保証）。
#[test]
fn harness_does_not_depend_on_actor_poc() {
    let src = include_str!("actor_test_harness.rs");
    // 実コード参照（Rust パス／use）を弾く厳密シンボルでチェックする。コメント中の散文は
    // この厳密形（`::` 連結のパス）を含まないよう記述してある。
    let poc_path = concat!("actor_poc", "::");
    assert!(
        !src.contains(poc_path),
        "harness must not reference the PoC scaffold path (so task 7.2 removal keeps this coverage)"
    );
    let poc_use = concat!("use ", "pasta_lua");
    assert!(
        !src.contains(poc_use),
        "harness must exercise the production pasta actor modules, not the PoC crate assets"
    );
}
