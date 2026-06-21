//! R1 ブロッカー記録経路（R1.4 / task 2.3）。
//!
//! design.md「ActorThread」Implementation Notes の方針——
//! 「不成立時は `Verdict::record_blocker` で NO-GO 根拠化」——および
//! Requirements Traceability の行 1.4（`actor_thread.rs`・`verdict.rs` →
//! `Verdict::record_blocker`）を実装する。
//!
//! task 2.1（[`ActorThread`]）／2.2（[`ReloadProbe`]）は R1 が **GO**（成立）で
//! あることを既に示している。本タスク（2.3）は逆側——「VM ホスト／teardown が
//! **不成立** だった場合に、どの条件で落ちたかを切り分けて `record_blocker` で
//! 残し NO-GO 根拠化する経路」——を実装し、テストでその経路を実際に駆動する。
//!
//! # `!Send` 違反は実行時に起こらない（正直なモデル化）
//!
//! R1.4 が挙げる不成立条件のうち「VM の `!Send` 制約違反」は **Rust の型システムが
//! コンパイル時に構造的に禁止** する（`mlua::Lua` が `!Send` なので、VM をアクター
//! スレッド外へ move しようとするコードはそもそもコンパイルできない。
//! `actor_thread.rs` のモジュール doc 参照）。したがって `!Send` 違反は **実行時に
//! 観測されうるブロッカーではない**。本モジュールはこれを偽の実行時チェックで
//! 演じる（芝居する）のではなく、分類列挙の専用バリアント
//! [`R1Blocker::SendViolationStructurallyPrevented`] として「構造的に予防済みである」
//! 事実そのものを正直に記録する。
//!
//! 実行時に検出されうる不成立条件は次のとおりで、これらが本タスクの主眼である:
//! - reload リーク（[`R1Blocker::ReloadLeak`]）— `ReloadProbe` のリソース増分が
//!   許容を超える。
//! - reload 後クラッシュ（[`R1Blocker::PostReloadCrash`]）— サイクル途中の
//!   panic/異常で clean teardown 数が要求サイクル数に満たない。
//! - ハング／teardown 不成立（[`R1Blocker::TeardownHang`]）— join が成立せず
//!   アクタースレッドが終わらない。
//! - spawn 失敗（[`R1Blocker::SpawnFailure`]）— アクタースレッドや executor の
//!   起動自体が成立しない。
//!
//! # フォールトインジェクションの継ぎ目（dead code にしない）
//!
//! 実 R1 は GO なので、失敗→ブロッカー経路を **決定論的に** 駆動するには注入が要る。
//! 本モジュールは「R1 試行の結果」を表す [`R1Outcome`] を継ぎ目に置き、
//! [`record_r1`] がそれを分類して `VerdictRecorder` へ記録する。実運用の
//! [`run_r1_probe`] は [`ActorThread`]＋[`ReloadProbe`] を実際に回して
//! [`R1Outcome`] を組み立てる（成功時は `Healthy`）。テストは [`R1Outcome`] の
//! 各失敗バリアントを直接注入し、対応するブロッカーが R1.4 の下に記録されること
//! （＝NO-GO 根拠化）を assert する。注入の継ぎ目は production の [`run_r1_probe`]
//! と test の双方から使われるため dead code ではない。

use super::actor_thread::ActorThread;
use super::teardown::ReloadProbe;
use super::verdict::{Blocker, ItemOutcome, ReqId, VerdictRecorder};

/// R1（executor 上 VM ホスト＋reload teardown）1 回の試行結果。
///
/// フォールトインジェクションの継ぎ目。実運用では [`run_r1_probe`] が
/// [`ActorThread`]＋[`ReloadProbe`] の実測からこれを組み立て、テストでは各
/// 失敗バリアントを直接注入して [`record_r1`] のブロッカー分類を駆動する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R1Outcome {
    /// VM ホスト・reload teardown ともに成立（GO）。`cycles` は確認した reload
    /// サイクル数。`record_r1` は成功 item を記録する。
    Healthy {
        /// clean teardown を確認した reload サイクル数。
        cycles: usize,
    },
    /// reload リーク: `ReloadProbe` のリソース増分が許容上限を超えた（R1.3 不成立）。
    ReloadLeak {
        /// 計測したリソース種別（例: "USER objects" / "kernel handles"）。
        resource: String,
        /// 観測された増分。
        growth: i64,
        /// 許容上限。
        tolerance: i64,
    },
    /// reload 後クラッシュ: サイクル途中で panic/異常終了し、clean teardown 数が
    /// 要求サイクル数に満たない（R1.2 不成立）。
    PostReloadCrash {
        /// 要求した reload サイクル数。
        requested_cycles: usize,
        /// 実際に clean teardown できたサイクル数（< requested_cycles）。
        clean_teardowns: usize,
    },
    /// teardown ハング: shutdown 後に `JoinHandle::join` が成立せず、アクター
    /// スレッドが終わらない（R1.2 不成立・無限待機）。
    TeardownHang {
        /// ハングが観測された局面の説明（例: "join after shutdown"）。
        phase: String,
    },
    /// spawn 失敗: アクタースレッド／executor の起動自体が成立しない（R1.1 不成立）。
    SpawnFailure {
        /// 失敗理由（OS／executor からのエラー説明）。
        detail: String,
    },
}

/// [`R1Outcome`] の失敗を切り分けたブロッカー分類。
///
/// 各実行時バリアントは R1.4 が挙げる不成立条件のいずれに当たるかを表す。
/// [`R1Blocker::SendViolationStructurallyPrevented`] のみ「実行時には起こらず
/// コンパイル時に構造的予防される」事実を正直に表すバリアントである。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R1Blocker {
    /// reload リーク（R1.3 不成立・実行時検出）。
    ReloadLeak,
    /// reload 後クラッシュ（R1.2 不成立・実行時検出）。
    PostReloadCrash,
    /// teardown ハング／無限待機（R1.2 不成立・実行時検出）。
    TeardownHang,
    /// spawn 失敗（R1.1 不成立・実行時検出）。
    SpawnFailure,
    /// `!Send` 制約違反は Rust 型システムによりコンパイル時に構造的予防される
    /// ため、**実行時ブロッカーとしては発生しない**。本バリアントはその事実を
    /// 正直に文書化するためだけに存在し、実行時の `R1Outcome` からは生成されない
    /// （[`R1Blocker::classify`] が `R1Outcome` から作る分類には含まれない）。
    SendViolationStructurallyPrevented,
}

impl R1Blocker {
    /// 実行時 [`R1Outcome`] の失敗から該当ブロッカー分類を切り分ける。
    ///
    /// `Healthy`（成功）には対応するブロッカーが無いため `None` を返す。
    /// `!Send` 違反は実行時に発生しないため、ここから
    /// [`R1Blocker::SendViolationStructurallyPrevented`] が返ることはない。
    pub fn classify(outcome: &R1Outcome) -> Option<R1Blocker> {
        match outcome {
            R1Outcome::Healthy { .. } => None,
            R1Outcome::ReloadLeak { .. } => Some(R1Blocker::ReloadLeak),
            R1Outcome::PostReloadCrash { .. } => Some(R1Blocker::PostReloadCrash),
            R1Outcome::TeardownHang { .. } => Some(R1Blocker::TeardownHang),
            R1Outcome::SpawnFailure { .. } => Some(R1Blocker::SpawnFailure),
        }
    }

    /// このブロッカー分類を表す短いラベル（記録 reason の先頭に付与し、
    /// どの条件で落ちたかを一目で切り分けられるようにする）。
    pub fn label(&self) -> &'static str {
        match self {
            R1Blocker::ReloadLeak => "reload-leak",
            R1Blocker::PostReloadCrash => "post-reload-crash",
            R1Blocker::TeardownHang => "teardown-hang",
            R1Blocker::SpawnFailure => "spawn-failure",
            R1Blocker::SendViolationStructurallyPrevented => "!Send-violation(compile-time-prevented)",
        }
    }
}

/// R1.4 を指す `ReqId`（requirements.md の正本番号 R1.4）。
fn r1_blocker_req() -> ReqId {
    ReqId::sub(1, 4)
}

/// R1（項目全体）を指す `ReqId`（成功 item 用）。
fn r1_item_req() -> ReqId {
    ReqId::item(1)
}

/// R1 試行結果を分類し、`VerdictRecorder` へ記録する（R1.4）。
///
/// - `Healthy` → 成功 item を `record_item(R1, ..)` で残す（成功経路）。
/// - 失敗各種 → 落ちた条件を切り分け、reason にその条件を反映した [`Blocker`] を
///   `record_blocker(R1.4, ..)` で残す（NO-GO 根拠化）。reason には
///   [`R1Blocker::label`] の分類ラベルを必ず含め、「どの条件で落ちたか」を
///   一般的な "failed" ではなく具体化する。
///
/// 戻り値: 記録したブロッカー分類（成功時は `None`）。呼び出し側／テストが
/// 「どの条件で NO-GO 根拠化したか」を確認できるようにする。
pub fn record_r1(rec: &mut VerdictRecorder, outcome: &R1Outcome) -> Option<R1Blocker> {
    match R1Blocker::classify(outcome) {
        None => {
            // 成功経路: VM ホスト＋reload teardown が成立。
            let cycles = match outcome {
                R1Outcome::Healthy { cycles } => *cycles,
                _ => unreachable!("classify returned None only for Healthy"),
            };
            rec.record_item(
                r1_item_req(),
                ItemOutcome::success(format!(
                    "executor 上 !Send VM ホスト＋reload teardown 成立（{cycles} サイクル clean）"
                )),
            );
            None
        }
        Some(blocker) => {
            // 失敗経路: 切り分けたブロッカー条件を reason に反映して記録（NO-GO 根拠）。
            // 失敗側でも item を残す（R8.2「成否にかかわらず全項目を試行し結果を残す」）。
            rec.record_item(
                r1_item_req(),
                ItemOutcome::failure(format!(
                    "executor 上 VM ホスト／teardown 不成立: {}",
                    blocker.label()
                )),
            );
            rec.record_blocker(r1_blocker_req(), blocker_for(&blocker, outcome));
            Some(blocker)
        }
    }
}

/// 切り分けたブロッカー分類と元 outcome から、reason／workaround を備えた
/// [`Blocker`] を組み立てる。reason は分類ラベル＋具体的な観測値を含む。
fn blocker_for(blocker: &R1Blocker, outcome: &R1Outcome) -> Blocker {
    match (blocker, outcome) {
        (
            R1Blocker::ReloadLeak,
            R1Outcome::ReloadLeak {
                resource,
                growth,
                tolerance,
            },
        ) => Blocker::with_workaround(
            format!(
                "[{}] reload 反復で {resource} が {growth} 増加（許容 {tolerance}）— \
                 teardown がリソースを解放しきれていない",
                blocker.label()
            ),
            "メッセージ専用ウィンドウ／ハンドルの Drop 解放を見直す（ReloadProbe で再計測）",
        ),
        (
            R1Blocker::PostReloadCrash,
            R1Outcome::PostReloadCrash {
                requested_cycles,
                clean_teardowns,
            },
        ) => Blocker::with_workaround(
            format!(
                "[{}] reload {requested_cycles} サイクル中 clean teardown は {clean_teardowns} 件のみ \
                 — reload 後にクラッシュ/異常終了する経路が残る",
                blocker.label()
            ),
            "クラッシュサイクルを最小再現し VM 再構築前のリソース状態を確認する",
        ),
        (R1Blocker::TeardownHang, R1Outcome::TeardownHang { phase }) => Blocker::new(format!(
            "[{}] {phase} でアクタースレッドが終了せずハング — JoinHandle::join が成立しない（無限待機）",
            blocker.label()
        )),
        (R1Blocker::SpawnFailure, R1Outcome::SpawnFailure { detail }) => Blocker::new(format!(
            "[{}] アクタースレッド/executor の起動に失敗: {detail}",
            blocker.label()
        )),
        // 分類と outcome が一致しない組合せは classify の不変条件上ありえないが、
        // 取りこぼし防止のため一般化した reason を残す（芝居ではなく防御）。
        (other, _) => Blocker::new(format!(
            "[{}] R1 不成立条件を切り分け（詳細は outcome を参照）",
            other.label()
        )),
    }
}

/// 実運用の R1 プローブ: [`ActorThread`]＋[`ReloadProbe`] を実際に回して
/// [`R1Outcome`] を組み立てる（注入の継ぎ目を production からも使う）。
///
/// 実 R1 は GO のため通常は `Healthy` を返す。実行時に検出されうる不成立条件
/// （reload リーク・clean teardown 不足）が観測されれば対応する失敗バリアントを
/// 返す。これにより [`record_r1`] の失敗分類経路が production からも到達可能と
/// なり、テスト専用の dead code に陥らない。
///
/// `reload_cycles`: reload リーク／teardown を検査する反復回数。
pub fn run_r1_probe(reload_cycles: usize) -> R1Outcome {
    // (1) VM ホスト成立の最小確認: spawn → 1 回 Lua 実行 → shutdown が clean か。
    //     spawn/実行/teardown のいずれかが破綻すれば、それを実行時ブロッカーとして
    //     切り分ける。
    let handle = ActorThread::spawn();
    let probe = handle.run_lua_probe("return 1");
    if let Err(detail) = probe {
        // shutdown を試みてから（ベストエフォート）spawn/実行系の失敗として返す。
        let _ = handle.shutdown();
        return R1Outcome::SpawnFailure { detail };
    }
    let first_teardown = handle.shutdown();
    if !first_teardown.joined_cleanly {
        return R1Outcome::TeardownHang {
            phase: "initial shutdown join".to_string(),
        };
    }

    // (2) reload 反復 ＋ リーク検査（R1.2/R1.3 を ReloadProbe で実測）。
    let report = ReloadProbe::run_cycles(reload_cycles);
    if !report.all_clean() {
        return R1Outcome::PostReloadCrash {
            requested_cycles: report.cycles_run,
            clean_teardowns: report.clean_teardowns,
        };
    }

    // (3) Windows でリソース増分が許容を超えていればリーク（実行時検出）。
    //     計測不能（非 Windows / 取得失敗）の場合はリーク判定をスキップする。
    #[cfg(windows)]
    if let Some(leak) = report.leak_metric {
        // teardown.rs のテストと整合する小さな許容上限。per-cycle リークなら
        // ≒reload_cycles で確実に超える。
        const USER_OBJECT_TOLERANCE: i64 = 8;
        const HANDLE_TOLERANCE: i64 = 8;
        if leak.user_object_growth > USER_OBJECT_TOLERANCE {
            return R1Outcome::ReloadLeak {
                resource: "USER objects".to_string(),
                growth: leak.user_object_growth,
                tolerance: USER_OBJECT_TOLERANCE,
            };
        }
        if leak.kernel_handle_growth > HANDLE_TOLERANCE {
            return R1Outcome::ReloadLeak {
                resource: "kernel handles".to_string(),
                growth: leak.kernel_handle_growth,
                tolerance: HANDLE_TOLERANCE,
            };
        }
    }

    R1Outcome::Healthy {
        cycles: report.cycles_run,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 成功経路: `Healthy` を記録すると R1 item に success が残り、R1.4 にブロッカーが
    /// 残らない（NO-GO 根拠が生じない）。
    #[test]
    fn healthy_records_success_item_and_no_blocker() {
        let mut rec = VerdictRecorder::new();
        let recorded = record_r1(&mut rec, &R1Outcome::Healthy { cycles: 24 });

        assert!(recorded.is_none(), "healthy R1 must not record a blocker");
        let item = rec.item(ReqId::item(1)).expect("R1 item must be recorded");
        assert!(item.success, "healthy R1 must record a success item");
        assert!(
            rec.blockers(ReqId::sub(1, 4)).is_empty(),
            "healthy R1 must leave R1.4 free of NO-GO grounds"
        );
        assert_eq!(rec.blocker_count(), 0);
    }

    /// 失敗経路（リーク）: 切り分けたブロッカー条件が reason に反映され、R1.4 の
    /// 下に NO-GO 根拠として取り出せる。observed 値（増分・許容）も reason に載る。
    #[test]
    fn reload_leak_records_classified_blocker_under_r1_4() {
        let mut rec = VerdictRecorder::new();
        let recorded = record_r1(
            &mut rec,
            &R1Outcome::ReloadLeak {
                resource: "USER objects".to_string(),
                growth: 24,
                tolerance: 8,
            },
        );

        assert_eq!(recorded, Some(R1Blocker::ReloadLeak));

        let blockers = rec.blockers(ReqId::sub(1, 4));
        assert_eq!(blockers.len(), 1, "leak must record exactly one R1.4 blocker");
        let reason = &blockers[0].reason;
        assert!(
            reason.contains("reload-leak"),
            "reason must carry the distinguished classification label, got: {reason}"
        );
        assert!(
            reason.contains("USER objects") && reason.contains("24"),
            "reason must reflect the observed leak (resource + growth), got: {reason}"
        );
        // 失敗でも item は残る（R8.2）。失敗 item でブロッカー分類が見える。
        let item = rec.item(ReqId::item(1)).expect("R1 item recorded even on failure");
        assert!(!item.success, "failed R1 must record a failure item");
    }

    /// 失敗経路（reload 後クラッシュ）: clean teardown 不足が PostReloadCrash として
    /// 切り分けられ、要求/成立サイクル数が reason に反映される。
    #[test]
    fn post_reload_crash_records_classified_blocker() {
        let mut rec = VerdictRecorder::new();
        let recorded = record_r1(
            &mut rec,
            &R1Outcome::PostReloadCrash {
                requested_cycles: 24,
                clean_teardowns: 17,
            },
        );

        assert_eq!(recorded, Some(R1Blocker::PostReloadCrash));
        let blockers = rec.blockers(ReqId::sub(1, 4));
        assert_eq!(blockers.len(), 1);
        let reason = &blockers[0].reason;
        assert!(reason.contains("post-reload-crash"), "must carry crash label: {reason}");
        assert!(
            reason.contains("24") && reason.contains("17"),
            "reason must reflect requested/clean cycle counts, got: {reason}"
        );
    }

    /// 失敗経路（ハング）: TeardownHang が切り分けられ無限待機の根拠として残る。
    #[test]
    fn teardown_hang_records_classified_blocker() {
        let mut rec = VerdictRecorder::new();
        let recorded = record_r1(
            &mut rec,
            &R1Outcome::TeardownHang {
                phase: "join after shutdown".to_string(),
            },
        );

        assert_eq!(recorded, Some(R1Blocker::TeardownHang));
        let reason = &rec.blockers(ReqId::sub(1, 4))[0].reason;
        assert!(reason.contains("teardown-hang"), "must carry hang label: {reason}");
        assert!(
            reason.contains("join after shutdown"),
            "reason must reflect the hang phase, got: {reason}"
        );
    }

    /// 失敗経路（spawn 失敗）: SpawnFailure が切り分けられ起動不成立の根拠として残る。
    #[test]
    fn spawn_failure_records_classified_blocker() {
        let mut rec = VerdictRecorder::new();
        let recorded = record_r1(
            &mut rec,
            &R1Outcome::SpawnFailure {
                detail: "executor message loop failed to start".to_string(),
            },
        );

        assert_eq!(recorded, Some(R1Blocker::SpawnFailure));
        let reason = &rec.blockers(ReqId::sub(1, 4))[0].reason;
        assert!(reason.contains("spawn-failure"), "must carry spawn label: {reason}");
        assert!(
            reason.contains("executor message loop"),
            "reason must reflect the spawn failure detail, got: {reason}"
        );
    }

    /// 各失敗分類が **互いに区別される** reason を生む（misclassification を検出）。
    /// 全分類を 1 つの recorder に流し、それぞれ別の R1.4 ブロッカーとして
    /// 区別可能なラベルで蓄積されることを確認する。
    #[test]
    fn each_failure_condition_is_distinguished_not_generic() {
        let outcomes = [
            R1Outcome::ReloadLeak {
                resource: "kernel handles".to_string(),
                growth: 30,
                tolerance: 8,
            },
            R1Outcome::PostReloadCrash {
                requested_cycles: 10,
                clean_teardowns: 0,
            },
            R1Outcome::TeardownHang {
                phase: "drain".to_string(),
            },
            R1Outcome::SpawnFailure {
                detail: "OS thread spawn refused".to_string(),
            },
        ];

        let mut rec = VerdictRecorder::new();
        for o in &outcomes {
            record_r1(&mut rec, o);
        }

        let blockers = rec.blockers(ReqId::sub(1, 4));
        assert_eq!(blockers.len(), 4, "every failure condition must be recorded under R1.4");

        // ラベルがすべて異なる（generic な "failed" ではない）ことを確認。
        let labels = [
            R1Blocker::ReloadLeak.label(),
            R1Blocker::PostReloadCrash.label(),
            R1Blocker::TeardownHang.label(),
            R1Blocker::SpawnFailure.label(),
        ];
        for label in labels {
            assert!(
                blockers.iter().any(|b| b.reason.contains(label)),
                "a blocker reason must distinguish condition `{label}`"
            );
        }
        // 全 4 件が NO-GO 根拠として R1.4 に揃う。
        assert_eq!(rec.blocker_count(), 4);
    }

    /// `!Send` 違反は実行時には発生しない（コンパイル時に構造的予防）ことを正直に
    /// モデル化: `classify` は `!Send` バリアントを生成しない。
    #[test]
    fn send_violation_is_compile_time_prevented_not_runtime_classified() {
        // 実行時 outcome のどれを分類しても SendViolation は出ない。
        let runtime_outcomes = [
            R1Outcome::Healthy { cycles: 1 },
            R1Outcome::ReloadLeak {
                resource: "x".to_string(),
                growth: 1,
                tolerance: 0,
            },
            R1Outcome::PostReloadCrash {
                requested_cycles: 1,
                clean_teardowns: 0,
            },
            R1Outcome::TeardownHang {
                phase: "x".to_string(),
            },
            R1Outcome::SpawnFailure {
                detail: "x".to_string(),
            },
        ];
        for o in &runtime_outcomes {
            assert_ne!(
                R1Blocker::classify(o),
                Some(R1Blocker::SendViolationStructurallyPrevented),
                "!Send violation must never be produced from a runtime outcome"
            );
        }
        // バリアント自体はその事実を文書化するために存在し、ラベルでそう述べる。
        assert!(
            R1Blocker::SendViolationStructurallyPrevented
                .label()
                .contains("compile-time-prevented"),
            "the !Send variant must honestly state it is compile-time-prevented"
        );
    }
}
