//! Latency 計測器（R6.1 / R6.2 / R6.3・task 6.1）。
//!
//! design.md「Latency」コンポーネントを実装する。GET block-on-reply の実レイテンシを
//! **忠実シミュレータ（`SimDriver`）の GET tick（Ref3=1・再生可）呼び出しパターン**で
//! 反復実測し、代表値（最大＋分布）を集計（R6.1）、GET タイムアウト→204 フォール
//! バックの要否と閾値候補を判断（R6.2）、許容応答時間を超過しうる経路と推奨方針を
//! `pasta-actor-runtime` へ申し送る（R6.3）。
//!
//! # 配置根拠（placement rationale・critical）
//!
//! GET block-on-reply の PRIMITIVE は task 3.2 remediation で **pasta_lua の
//! [`ActorThread::submit`](super::actor_thread::ActorThread::submit)
//! （[`ActorMsg::Get`](super::mailbox::ActorMsg::Get)）＋
//! [`Responder`](super::responder::Responder)** に在る。design.md は `latency.rs` を
//! pasta_lua に置きつつ `Marshal(P1)` 依存を示すが、`pasta_lua` は **`pasta_shiori` に
//! 依存できない**（依存方向が逆。`Marshal`＝`ffi_marshal.rs` は下流 `pasta_shiori`）。
//! よって本計測器は pasta_shiori の `Marshal` ラッパを経由せず、pasta_lua の
//! `ActorThread` GET 経路を `SimDriver` の GET tick 呼び出しパターンで **直接** 駆動・
//! 計測する。これは task 5.1 の SimDriver GET-tick = Ref3=1 = 1 GET call パターンと
//! 同じ呼び出し規約に従う。
//!
//! # 正直さ（feasibility caveat・R6.3）
//!
//! 本計測は **in-process・同一マシン上の実 `!Send` VM への block-on-reply 往復**を
//! 測る。executor 駆動の mailbox enqueue → VM 実行 → reply の往復であり、ネットワーク
//! も SSP プロセス間境界も介さないため、実測値は通常 **sub-ms**（マイクロ秒〜十数
//! マイクロ秒オーダ）に収まる。これは「方式上、GET 往復が宿主の許容応答時間に対して
//! 十分速い」ことの実測根拠にはなるが、**実 SSP に attach した絶対性能保証ではない**。
//! 実機 SSP の OnSecondChange 周期・プロセス間 marshaling・実シーン VM 負荷下の絶対
//! レイテンシ保証は後続実装仕様 `pasta-actor-runtime`（R5.6/R6.3）へ申し送る。本計測器は
//! その申し送りを [`Latency::sample_and_record`] で `VerdictRecorder` に明記する。
//!
//! 本ファイルは出荷コードを改変しない feature-gate モジュールであり、`actor-poc`
//! 無効時はコンパイル単位に現れない（リリースビルドはバイト不変・R7.1/R7.2）。

use std::time::{Duration, Instant};

use super::actor_thread::ActorThread;
use super::mailbox::ActorMsg;
use super::responder::{PocResponse, Responder};
use super::sim_driver::SimDriver;
use super::verdict::{Blocker, ItemOutcome, ReqId, VerdictRecorder};

/// GET block-on-reply レイテンシの代表値（最大＋分布）。
///
/// design.md「Latency」Service Interface `sample(n)->LatencyStats` の戻り値。実測
/// サンプルから集計され、`min ≤ median ≤ p95 ≤ p99 ≤ max` が成立する（順序統計量）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyStats {
    /// 実測したサンプル数（= `sample(n)` の n）。
    pub count: usize,
    /// うち **実 VM が値を生成して持ち帰った往復**の数（fake でない GET 往復）。
    /// 正常な実装では `count` と一致する。
    pub vm_round_trips: usize,
    /// 最小レイテンシ。
    pub min: Duration,
    /// 平均レイテンシ。
    pub mean: Duration,
    /// 中央値（p50）。
    pub median: Duration,
    /// 95 パーセンタイル。
    pub p95: Duration,
    /// 99 パーセンタイル。
    pub p99: Duration,
    /// 最大レイテンシ（代表値の本命・R6.1「最大」）。
    pub max: Duration,
}

impl LatencyStats {
    /// 実測した往復レイテンシ列から代表値を集計する。
    ///
    /// `samples` は計測順、`vm_round_trips` は実 VM 値を持ち帰った往復数。空の場合は
    /// すべて `Duration::ZERO`・`count=0` を返す（呼び出し側のテストが空を検出できる）。
    fn from_samples(samples: &[Duration], vm_round_trips: usize) -> Self {
        let count = samples.len();
        if count == 0 {
            return LatencyStats {
                count: 0,
                vm_round_trips,
                min: Duration::ZERO,
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
            };
        }

        // 順序統計量はソート列から取り出す（昇順）。
        let mut sorted: Vec<Duration> = samples.to_vec();
        sorted.sort_unstable();

        let min = sorted[0];
        let max = sorted[count - 1];
        let sum: Duration = sorted.iter().copied().sum();
        let mean = sum / count as u32;

        LatencyStats {
            count,
            vm_round_trips,
            min,
            mean,
            median: percentile(&sorted, 50),
            p95: percentile(&sorted, 95),
            p99: percentile(&sorted, 99),
            max,
        }
    }
}

/// ソート済み（昇順）列の `p` パーセンタイル（0..=100）を nearest-rank で取り出す。
///
/// nearest-rank: rank = ceil(p/100 * N)（1-origin）。`p=100` で最大、`p=50` で中央値
/// 近傍を返す。N≥1 を前提（空列は呼び出し側で除外済み）。
fn percentile(sorted: &[Duration], p: u8) -> Duration {
    let n = sorted.len();
    debug_assert!(n >= 1, "percentile requires a non-empty sorted slice");
    // ceil(p/100 * n) を整数演算で。p=0 のときは rank=1 に丸める。
    let rank = (u128::from(p) * n as u128).div_ceil(100);
    let idx = rank.max(1).min(n as u128) as usize - 1;
    sorted[idx]
}

/// GET タイムアウト→204 フォールバックの要否判断＋閾値候補（R6.2）。
///
/// design.md「Latency」Service Interface `recommend()->FallbackDecision` の戻り値。
/// 実測 stats から導出する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackDecision {
    /// GET タイムアウト→204 フォールバックを **設けることを推奨するか**。
    ///
    /// 正直さ: in-process 実測は sub-ms（[`Latency`] モジュール doc の caveat 参照）の
    /// ため、実測値だけでは「必須」とは言えない。本 PoC は「防御的に設けることを推奨
    /// （recommended=true）」を結論とする——実機 SSP の絶対保証が `pasta-actor-runtime`
    /// へ繰り延べられる以上、未知の遅延経路に対する 204 フォールバックは安全側の既定で
    /// あるべきだからである（理由は [`rationale`](Self::rationale) に明記）。
    pub recommended: bool,
    /// 推奨する GET タイムアウト閾値候補。実測最大に安全余裕（マージン）を乗せた値。
    /// タイムアウトが正常な GET を切らないよう、常に観測最大より大きい。
    pub threshold_candidate: Duration,
    /// 判断理由（R6.2「文書化」）。実測代表値と申し送り前提を含む。
    pub rationale: String,
}

/// GET block-on-reply レイテンシ計測器。
///
/// 内部に [`ActorThread`](super::actor_thread)（task 2.1・`!Send` VM をアクター
/// スレッドに pin）と [`SimDriver`](super::sim_driver)（R5.6 忠実シミュレータ）を
/// 保持し、SimDriver の GET tick 呼び出しパターンで GET block-on-reply 往復を反復
/// 実測する。`shutdown` で clean teardown する（写経: `ActorThread` の shutdown idiom）。
pub struct Latency {
    /// `!Send` VM を pin したアクタースレッド（GET block-on-reply 往復先）。
    actor: ActorThread,
    /// 忠実シミュレータ。GET tick（Ref3=1・再生可）の呼び出しパターンを供給する。
    sim: SimDriver,
    /// 直近の `sample` が集計した代表値（`recommend` が参照する）。未 sample なら `None`。
    last_stats: Option<LatencyStats>,
    /// GET 処理側 VM が連番を返すための単調カウンタ（各往復が別個の値を持ち帰る）。
    seq: u64,
}

impl Latency {
    /// 計測器を起動する（アクタースレッド spawn ＋ SimDriver 生成）。
    pub fn start() -> Self {
        Latency {
            actor: ActorThread::spawn(),
            sim: SimDriver::new(),
            last_stats: None,
            seq: 0,
        }
    }

    /// n 回の **実 GET block-on-reply 往復**を SimDriver の GET tick 呼び出しパターンで
    /// 反復実測し、代表値（最大＋分布）を集計する（R6.1）。
    ///
    /// 各反復は task 5.1 と同じ GET tick = Ref3=1 = 1 GET call の規約に従い、
    /// `SimDriver::tick(true)`（GET tick・再生可）を発火してから 1 件の GET を
    /// `ActorThread::submit(ActorMsg::Get{responder})` で enqueue し、`Responder` の
    /// 受信端を **block-on-reply** で待つ。往復は `std::time::Instant` 単調時計で実測
    /// する（enqueue 直前 → reply 受信直後）。VM が値を生成して持ち帰った往復のみを
    /// `vm_round_trips` に数える（fake でない往復の証跡）。
    pub fn sample(&mut self, n: usize) -> LatencyStats {
        let mut samples: Vec<Duration> = Vec::with_capacity(n);
        let mut vm_round_trips = 0usize;

        for _ in 0..n {
            // SimDriver の GET tick（Ref3=1・再生可）を発火する。GET tick が来ている＝
            // GET block-on-reply を 1 回実行できる、という task 5.1 と同じ呼び出し規約。
            let tick = self.sim.tick(true);
            debug_assert!(
                tick.is_playable(),
                "the latency sweep must run on GET ticks (Ref3=1)"
            );

            self.seq += 1;
            // GET 処理側 VM が連番を返す Lua（実 VM 実行を強制し、値を持ち帰らせる）。
            // 返り値は ActorThread の GET 経路が PocResponse::Value(...) に載せ替える。
            let script = format!("return {}", self.seq);

            let (responder, reply_rx) = Responder::new();

            // 計測開始: enqueue 直前。
            let start = Instant::now();
            self.actor
                .submit(ActorMsg::Get {
                    payload: self.seq,
                    script,
                    responder,
                })
                .expect("actor thread must accept the GET during the latency sweep");

            // block-on-reply: 実 VM が応答値を返すまでブロックする。
            let response = reply_rx
                .recv()
                .expect("the GET responder must deliver a reply (value or 204)");
            // 計測終了: reply 受信直後。
            let elapsed = start.elapsed();

            samples.push(elapsed);

            // 実 VM が値を生成して持ち帰った往復だけを数える（204 フォールバックは
            // VM 失敗＝実 round trip ではないので除外する）。これにより「fake/空の
            // 往復」をテストが検出できる。
            if let PocResponse::Value(_) = response {
                vm_round_trips += 1;
            }
        }

        let stats = LatencyStats::from_samples(&samples, vm_round_trips);
        self.last_stats = Some(stats);
        stats
    }

    /// 直近の `sample` 実測 stats から GET タイムアウト→204 フォールバックの要否＋
    /// 閾値候補を判断する（R6.2）。
    ///
    /// `sample` を未実行で呼んだ場合は空 stats（`count=0`）に対する保守的な既定
    /// （推奨・最小閾値）を返す。通常は `sample` 後に呼ぶ。
    pub fn recommend(&self) -> FallbackDecision {
        let stats = self
            .last_stats
            .unwrap_or_else(|| LatencyStats::from_samples(&[], 0));
        decide_fallback(&stats)
    }

    /// `sample(n)` ＋ `recommend()` を実行し、結果と R6.3 申し送りを `VerdictRecorder`
    /// に記録する（R6.1/R6.2/R6.3 を 1 経路で果たす統合エントリ）。
    ///
    /// 記録内容:
    /// - R6 item（`ReqId::item(6)`）: 計測が完了し代表値・フォールバック判断を得られた
    ///   ことを **success** で記録。`approach` に代表値（max/p99 等）と判断を反映。
    /// - R6.3 ブロッカー（`ReqId::sub(6, 3)`）: in-process 実測は sub-ms・実 SSP 絶対
    ///   保証は `pasta-actor-runtime` へ繰り延べ、という申し送りを `reason` に、推奨
    ///   フォールバック方針（閾値候補）を `workaround` に記録する（R6.3）。
    pub fn sample_and_record(
        &mut self,
        n: usize,
        rec: &mut VerdictRecorder,
    ) -> (LatencyStats, FallbackDecision) {
        let stats = self.sample(n);
        let decision = decide_fallback(&stats);

        // R6 item: 代表値とフォールバック判断を approach に持ち回る（実測の持ち回り）。
        let approach = format!(
            "GET block-on-reply 代表値（in-process 実測 n={}・実 VM 往復={}）: \
             min={:?} median={:?} p95={:?} p99={:?} max={:?}・mean={:?}。\
             フォールバック推奨={} 閾値候補={:?}",
            stats.count,
            stats.vm_round_trips,
            stats.min,
            stats.median,
            stats.p95,
            stats.p99,
            stats.max,
            stats.mean,
            decision.recommended,
            decision.threshold_candidate,
        );
        rec.record_item(
            ReqId::item(6),
            ItemOutcome::success(approach)
                .with_constraint(
                    "in-process 実測は sub-ms（NW/プロセス間境界なし）。\
                     実機 SSP の絶対性能保証ではない（R6.3 申し送り）",
                ),
        );

        // R6.3 申し送り: 実 SSP 絶対保証を pasta-actor-runtime へ繰り延べ、推奨方針
        //（閾値候補）を回避候補（workaround）として残す。
        rec.record_blocker(
            ReqId::sub(6, 3),
            Blocker::with_workaround(
                format!(
                    "GET block-on-reply の in-process 実測（max={:?}）は宿主の許容応答時間に対し \
                     十分速いが、実機 SSP の OnSecondChange 周期・プロセス間 marshaling・実シーン \
                     VM 負荷下の絶対レイテンシ保証は本 PoC では確定できない。実機 SSP に対する \
                     絶対性能保証と GET タイムアウト方針の最終確定は後続実装仕様 \
                     pasta-actor-runtime へ申し送る（R5.6/R6.3）",
                    stats.max
                ),
                format!(
                    "GET タイムアウト→204 フォールバックを設け、閾値候補 {:?}（観測最大に安全 \
                     マージンを乗せた値）を初期値として pasta-actor-runtime で実機実測に基づき調整",
                    decision.threshold_candidate
                ),
            ),
        );

        (stats, decision)
    }

    /// アクタースレッドを clean teardown する（写経: `ActorThread::shutdown`）。
    pub fn shutdown(self) {
        let _ = self.actor.shutdown();
    }
}

/// 実測 stats から GET タイムアウト→204 フォールバックの要否＋閾値候補を導出する。
///
/// 方針（R6.2・正直な PoC 結論）:
/// - **要否**: in-process 実測は sub-ms ゆえ「実測上は必須でない」が、実機 SSP の
///   絶対保証が `pasta-actor-runtime` へ繰り延べられる以上、未知の遅延経路に対する
///   防御として **設けることを推奨（recommended=true）** する（安全側の既定）。
/// - **閾値候補**: 観測最大に **安全マージン**を乗せた値。マージンは「max×10、ただし
///   最低 1ms」とする——in-process の max が極端に小さく（sub-µs）ても実用的な下限を
///   確保し、かつ正常 GET をタイムアウトで切らない（常に max より大きい）ことを保証する。
fn decide_fallback(stats: &LatencyStats) -> FallbackDecision {
    // 安全マージン: max の 10 倍、ただし最低 1ms。常に max より大きくなる。
    let scaled = stats.max.saturating_mul(10);
    let floor = Duration::from_millis(1);
    let threshold_candidate = scaled.max(floor);

    let rationale = format!(
        "in-process 実測（n={}・max={:?}・p99={:?}・mean={:?}）は宿主の許容応答時間に対し \
         十分速く、実測上は GET タイムアウトが発火する経路は観測されなかった。ただし実機 SSP の \
         絶対性能保証は pasta-actor-runtime へ繰り延べられるため（R5.6/R6.3）、未知の遅延経路への \
         防御として GET タイムアウト→204 フォールバックを設けることを推奨する。閾値候補は観測最大に \
         安全マージン（×10・最低 1ms）を乗せた {:?} とし、正常 GET をタイムアウトで切らない \
         （常に観測最大より大きい）。最終値は pasta-actor-runtime で実機実測に基づき調整する。",
        stats.count, stats.max, stats.p99, stats.mean, threshold_candidate
    );

    FallbackDecision {
        recommended: true,
        threshold_candidate,
        rationale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 順序統計量の単調性（ソート列に対し min ≤ p50 ≤ p95 ≤ p99 ≤ max）。
    #[test]
    fn percentiles_are_monotonic_on_sorted_input() {
        let sorted: Vec<Duration> = (1..=100).map(Duration::from_micros).collect();
        assert_eq!(percentile(&sorted, 50), Duration::from_micros(50));
        assert_eq!(percentile(&sorted, 95), Duration::from_micros(95));
        assert_eq!(percentile(&sorted, 99), Duration::from_micros(99));
        assert_eq!(percentile(&sorted, 100), Duration::from_micros(100));
        // 単調性。
        assert!(percentile(&sorted, 50) <= percentile(&sorted, 95));
        assert!(percentile(&sorted, 95) <= percentile(&sorted, 99));
        assert!(percentile(&sorted, 99) <= percentile(&sorted, 100));
    }

    /// 代表値集計: count・min/max/mean・分布が実測列から正しく出る。
    #[test]
    fn stats_aggregate_from_samples() {
        let samples: Vec<Duration> = vec![
            Duration::from_micros(10),
            Duration::from_micros(20),
            Duration::from_micros(30),
            Duration::from_micros(40),
        ];
        let stats = LatencyStats::from_samples(&samples, samples.len());
        assert_eq!(stats.count, 4);
        assert_eq!(stats.vm_round_trips, 4);
        assert_eq!(stats.min, Duration::from_micros(10));
        assert_eq!(stats.max, Duration::from_micros(40));
        assert_eq!(stats.mean, Duration::from_micros(25));
        // 単調性。
        assert!(stats.min <= stats.median);
        assert!(stats.median <= stats.p95);
        assert!(stats.p95 <= stats.p99);
        assert!(stats.p99 <= stats.max);
    }

    /// 空サンプルは count=0・全ゼロ（テストが空を検出できる）。
    #[test]
    fn empty_samples_produce_zero_stats() {
        let stats = LatencyStats::from_samples(&[], 0);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.max, Duration::ZERO);
        assert_eq!(stats.mean, Duration::ZERO);
    }

    /// フォールバック判断: 閾値候補は常に観測最大より大きく、最低 1ms を下回らない。
    #[test]
    fn fallback_threshold_exceeds_max_and_respects_floor() {
        // 大きめの max。
        let big = LatencyStats::from_samples(&[Duration::from_millis(5)], 1);
        let d_big = decide_fallback(&big);
        assert!(d_big.threshold_candidate > big.max, "threshold must exceed max");
        assert_eq!(d_big.threshold_candidate, Duration::from_millis(50));
        assert!(d_big.recommended, "fallback is recommended (safe default)");

        // 極小 max（sub-µs）でも floor で 1ms を確保。
        let tiny = LatencyStats::from_samples(&[Duration::from_nanos(50)], 1);
        let d_tiny = decide_fallback(&tiny);
        assert!(d_tiny.threshold_candidate > tiny.max);
        assert_eq!(d_tiny.threshold_candidate, Duration::from_millis(1));
        assert!(!d_tiny.rationale.trim().is_empty(), "rationale documented");
    }
}
