//! R6 実証テスト: GET block-on-reply レイテンシ実測＋フォールバック要否判断（task 6.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切汚染
//! しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 6.1）:
//! 「n 回 GET の代表値を集計し、フォールバック要否判断＋閾値候補を出力する統合
//!   テストが緑。」
//!
//! 正直さ（テストが本当に失敗しうること）の設計:
//! - レイテンシは **アクタースレッド上の実 `!Send` VM への実 GET block-on-reply
//!   往復**（`ActorThread::submit(ActorMsg::Get{responder})` → `Responder` 受信）を
//!   忠実シミュレータ（`SimDriver`）の GET tick（Ref3=1・再生可）呼び出しパターンで
//!   n 回反復し、各往復を `std::time::Instant` 単調時計で実測する（R6.1・design.md
//!   「Latency」Service Interface `sample(n)->LatencyStats`）。
//! - 各 GET が **実 VM が生成した値**を本当に持ち帰ったこと（fake でない往復）を
//!   `LatencyStats::vm_round_trips` で観測する（n 個すべてが実 VM 由来の値）。
//! - 代表値（最大＋分布）は実測サンプルから導出され、`max ≥ p99 ≥ median ≥ min` と
//!   `count == n` が成立する（捏造・空なら落ちる）。
//! - `recommend()` は実測 stats から GET タイムアウト→204 フォールバックの要否と
//!   閾値候補を導出し、閾値候補が観測最大より大きい（合理的な余裕を持つ）こと、
//!   判断理由が記録されていることを assert する（R6.2）。
//! - R6.3 申し送り（in-process 実測は sub-ms・実機 SSP 絶対保証は `pasta-actor-runtime`
//!   へ繰り延べ）が `VerdictRecorder` の R6 item／R6.3 ブロッカーに残ることを assert
//!   する（記録が壊れていれば落ちる）。
//!
//! 配置根拠（placement rationale）:
//! GET block-on-reply の PRIMITIVE は task 3.2 remediation で **pasta_lua の
//! `ActorThread::submit(ActorMsg::Get)` ＋ `Responder`** に在る。design.md は
//! `latency.rs` を pasta_lua に置き `Marshal(P1)` 依存を示すが、`pasta_lua` は
//! `pasta_shiori` に依存できない（依存方向が逆）。よって GET 往復は pasta_shiori の
//! Marshal ラッパを経由せず、pasta_lua の `ActorThread` GET 経路を `SimDriver` の
//! GET tick 呼び出しパターンで **直接** 計測する。
#![cfg(feature = "actor-poc")]

use std::time::Duration;

use pasta_lua::actor_poc::latency::{FallbackDecision, Latency, LatencyStats};
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

/// 代表値の単調性を一括検証するヘルパ（`min ≤ median ≤ p95 ≤ p99 ≤ max`）。
fn assert_distribution_monotonic(stats: &LatencyStats) {
    assert!(
        stats.min <= stats.median,
        "min must be ≤ median; min={:?} median={:?}",
        stats.min,
        stats.median
    );
    assert!(
        stats.median <= stats.p95,
        "median must be ≤ p95; median={:?} p95={:?}",
        stats.median,
        stats.p95
    );
    assert!(
        stats.p95 <= stats.p99,
        "p95 must be ≤ p99; p95={:?} p99={:?}",
        stats.p95,
        stats.p99
    );
    assert!(
        stats.p99 <= stats.max,
        "p99 must be ≤ max; p99={:?} max={:?}",
        stats.p99,
        stats.max
    );
}

/// R6.1: `sample(n)` が n 回の **実 GET block-on-reply 往復**を実測し、代表値
/// （最大＋分布）を実測サンプルから集計する。
#[test]
fn sample_collects_n_real_get_round_trips_and_aggregates_representative_stats() {
    const N: usize = 64;

    let mut latency = Latency::start();
    let stats = latency.sample(N);

    // 正確に n サンプル収集されている（空・捏造なら落ちる）。
    assert_eq!(
        stats.count, N,
        "sample(n) must collect exactly n latency samples; got {}",
        stats.count
    );

    // n 個すべてが **実 VM が生成した値**を持ち帰った往復である（fake round trip でない）。
    assert_eq!(
        stats.vm_round_trips, N,
        "every GET must really round-trip a VM-produced value; got {} of {}",
        stats.vm_round_trips, N
    );

    // 代表値の単調性（捏造・破損なら崩れる）。
    assert_distribution_monotonic(&stats);

    // 実測なので有限・非ゼロ上限（in-process でも 0ns ちょうどには張り付かない）。
    assert!(
        stats.max >= stats.min,
        "max must be ≥ min; max={:?} min={:?}",
        stats.max,
        stats.min
    );
    assert!(
        stats.mean > Duration::ZERO,
        "mean of real measurements must be > 0; got {:?}",
        stats.mean
    );

    latency.shutdown();
}

/// R6.1（正直さ）: sample 数を変えても `count` がその数に追従する（固定値を返す
/// 捏造実装なら落ちる）。
#[test]
fn sample_count_follows_requested_n() {
    let mut latency = Latency::start();

    let s8 = latency.sample(8);
    assert_eq!(s8.count, 8, "count must follow n=8");
    assert_eq!(s8.vm_round_trips, 8, "all 8 must be real VM round trips");

    let s32 = latency.sample(32);
    assert_eq!(s32.count, 32, "count must follow n=32");
    assert_eq!(s32.vm_round_trips, 32, "all 32 must be real VM round trips");

    latency.shutdown();
}

/// R6.2: `recommend()` が実測 stats から GET タイムアウト→204 フォールバックの
/// 要否＋閾値候補を導出する。閾値候補は観測最大より大きく（合理的な余裕を持つ）、
/// 判断理由が記録される。
#[test]
fn recommend_outputs_fallback_decision_consistent_with_measured_stats() {
    const N: usize = 64;

    let mut latency = Latency::start();
    let stats = latency.sample(N);
    let decision: FallbackDecision = latency.recommend();

    // 判断は実測 stats と整合する（recommend は sample 済みの実測に基づく）。
    // 閾値候補は観測最大より大きい（タイムアウトが正常 GET を切らない余裕）。
    assert!(
        decision.threshold_candidate > stats.max,
        "the fallback threshold candidate must exceed the observed max latency \
         (so a timeout never severs a healthy GET); threshold={:?} max={:?}",
        decision.threshold_candidate,
        stats.max
    );

    // 判断理由が空でない（R6.2「文書化」）。
    assert!(
        !decision.rationale.trim().is_empty(),
        "the fallback decision must document its rationale (R6.2)"
    );

    // 要否フラグと閾値候補は一貫: 推奨するなら閾値候補は有意（>0）。
    assert!(
        decision.threshold_candidate > Duration::ZERO,
        "the threshold candidate must be a positive duration"
    );

    latency.shutdown();
}

/// R6.3: 申し送りが `VerdictRecorder` に記録される。
/// - R6 item が成功で記録され、代表値（max/分布）が approach に反映される。
/// - in-process 実測は sub-ms・実機 SSP 絶対保証は `pasta-actor-runtime` へ繰り延べ、
///   という正直な caveat が R6.3 ブロッカー（申し送り）として残る。
#[test]
fn sample_and_record_hands_off_deferred_real_ssp_guarantee() {
    const N: usize = 64;

    let mut latency = Latency::start();
    let mut rec = VerdictRecorder::new();

    let (stats, decision) = latency.sample_and_record(N, &mut rec);

    // R6 item が success で記録される。
    let r6 = rec.item(ReqId::item(6)).expect("R6 item must be recorded");
    assert!(r6.success, "a completed latency sweep records R6 as success");

    // 代表値（最大 or 分布）が approach に反映されている（実測の持ち回り）。
    assert!(
        r6.approach.contains("max")
            || r6.approach.contains("最大")
            || r6.approach.contains("p99")
            || r6.approach.contains("代表値"),
        "the R6 approach must carry the measured representative latency; got: {}",
        r6.approach
    );

    // R6.3 申し送り（deferred real-SSP guarantee）が記録される。
    let handoffs = rec.blockers(ReqId::sub(6, 3));
    assert!(
        !handoffs.is_empty(),
        "the R6.3 deferred real-SSP guarantee handoff must be recorded"
    );
    let reason = &handoffs[0].reason;
    assert!(
        reason.contains("pasta-actor-runtime"),
        "the R6.3 handoff must defer the absolute real-SSP guarantee to pasta-actor-runtime; \
         got: {reason}"
    );

    // 申し送りは推奨フォールバック方針（閾値候補）を伴う（R6.3「推奨方針を記録」）。
    assert!(
        handoffs[0].workaround.is_some(),
        "the R6.3 handoff must carry a recommended fallback policy (threshold candidate)"
    );

    // 記録された stats/decision は sample/recommend の実測と一貫（捏造でない）。
    assert_eq!(stats.count, N, "recorded stats must reflect the real sweep");
    assert_eq!(stats.vm_round_trips, N, "all recorded round trips are real");
    assert!(
        decision.threshold_candidate > stats.max,
        "recorded decision threshold must exceed observed max"
    );

    latency.shutdown();
}
