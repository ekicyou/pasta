//! R1.4 実証テスト: VM ホスト/teardown 不成立条件の切り分けと `record_blocker` に
//! よる NO-GO 根拠化（task 2.3）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切汚染
//! しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 2.3）:
//! 「失敗注入で `record_blocker` がブロッカー条件を記録し NO-GO 根拠化される
//!   ことをテストで確認。」
//!
//! 設計（test に歯があること）:
//! - 実 R1 は GO（task 2.1/2.2）なので、失敗→ブロッカー経路を決定論的に駆動する
//!   ため `R1Outcome` の各失敗バリアントを **注入** する（フォールトインジェクション
//!   の継ぎ目）。
//! - 各注入で `record_r1` が R1.4（`ReqId::sub(1,4)`）の下に、落ちた条件を切り分けた
//!   reason のブロッカーを記録し、recorder から NO-GO 根拠として取り出せることを assert。
//! - ブロッカーが記録されない／分類を取り違える（generic な "failed" になる）と
//!   これらの assert は確実に失敗する。
//! - 加えて実 `run_r1_probe` を回し、GO 環境では成功 item が残り R1.4 にブロッカーが
//!   出ない（成功経路）ことを確認する。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::r1_probe::{record_r1, run_r1_probe, R1Blocker, R1Outcome};
use pasta_lua::actor_poc::verdict::{ReqId, VerdictRecorder};

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。`#[ctor]` 写経元:
/// `crates/pasta_lua/tests/common/mod.rs:26-32`（PASTA_DEBUG=1 がテストを壊す既知問題）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// R1.4: reload リーク注入で、切り分けたブロッカーが R1.4 の下に NO-GO 根拠として
/// 記録される。reason は分類ラベルと観測値を含み generic でない。
#[test]
fn injected_reload_leak_is_recorded_as_no_go_grounds() {
    let mut rec = VerdictRecorder::new();
    let classified = record_r1(
        &mut rec,
        &R1Outcome::ReloadLeak {
            resource: "USER objects".to_string(),
            growth: 24,
            tolerance: 8,
        },
    );

    assert_eq!(classified, Some(R1Blocker::ReloadLeak));

    let blockers = rec.blockers(ReqId::sub(1, 4));
    assert_eq!(
        blockers.len(),
        1,
        "an injected leak must record exactly one R1.4 blocker (NO-GO grounds)"
    );
    assert!(
        blockers[0].reason.contains("reload-leak"),
        "blocker reason must distinguish WHICH condition failed, got: {}",
        blockers[0].reason
    );
    assert!(
        blockers[0].reason.contains("24"),
        "blocker reason must reflect the observed growth, got: {}",
        blockers[0].reason
    );
}

/// R1.4: 全失敗条件（リーク・reload 後クラッシュ・ハング・spawn 失敗）が
/// **互いに区別された** reason で R1.4 に記録される。`!Send` 違反はコンパイル時に
/// 構造的予防されるため実行時注入には含めない（正直なモデル化）。
#[test]
fn every_runtime_failure_condition_is_distinguished_under_r1_4() {
    let injections = [
        (
            R1Outcome::ReloadLeak {
                resource: "kernel handles".to_string(),
                growth: 40,
                tolerance: 8,
            },
            R1Blocker::ReloadLeak,
        ),
        (
            R1Outcome::PostReloadCrash {
                requested_cycles: 24,
                clean_teardowns: 3,
            },
            R1Blocker::PostReloadCrash,
        ),
        (
            R1Outcome::TeardownHang {
                phase: "join after shutdown".to_string(),
            },
            R1Blocker::TeardownHang,
        ),
        (
            R1Outcome::SpawnFailure {
                detail: "executor message loop refused".to_string(),
            },
            R1Blocker::SpawnFailure,
        ),
    ];

    let mut rec = VerdictRecorder::new();
    for (outcome, expected) in &injections {
        let got = record_r1(&mut rec, outcome);
        assert_eq!(
            got.as_ref(),
            Some(expected),
            "each failure outcome must be classified into its specific blocker"
        );
    }

    let blockers = rec.blockers(ReqId::sub(1, 4));
    assert_eq!(
        blockers.len(),
        4,
        "all four runtime failure conditions must accumulate as R1.4 NO-GO grounds"
    );

    // 4 つの分類ラベルがすべて区別されて現れる（generic でない＝歯がある）。
    for label in [
        R1Blocker::ReloadLeak.label(),
        R1Blocker::PostReloadCrash.label(),
        R1Blocker::TeardownHang.label(),
        R1Blocker::SpawnFailure.label(),
    ] {
        assert!(
            blockers.iter().any(|b| b.reason.contains(label)),
            "blocker reasons must distinguish condition `{label}`, not collapse to a generic failure"
        );
    }
    assert_eq!(rec.blocker_count(), 4, "all conditions become NO-GO grounds");
}

/// 成功経路（実 probe）: GO 環境では `run_r1_probe` が `Healthy` を返し、
/// `record_r1` が R1 成功 item を残し R1.4 にブロッカーを出さない。
///
/// task 2.1/2.2 が GO を示しているので、ここでは少数サイクルで実 probe を回す。
#[test]
fn real_probe_records_success_item_and_no_blocker_when_go() {
    let outcome = run_r1_probe(6);

    // 実 R1 は GO（task 2.1/2.2）。万一実行時失敗が観測された場合でも、それは
    // 切り分けて記録される（その場合は下の assert が示す通り blocker が残る）。
    let mut rec = VerdictRecorder::new();
    let classified = record_r1(&mut rec, &outcome);

    match outcome {
        R1Outcome::Healthy { .. } => {
            assert!(classified.is_none(), "healthy probe must not classify a blocker");
            let item = rec.item(ReqId::item(1)).expect("R1 success item recorded");
            assert!(item.success, "GO probe must record a success item");
            assert!(
                rec.blockers(ReqId::sub(1, 4)).is_empty(),
                "GO probe must leave R1.4 free of NO-GO grounds"
            );
        }
        other => {
            // 実行時に不成立が観測されたなら、それが NO-GO 根拠化されていること。
            assert!(
                classified.is_some(),
                "a non-healthy real outcome must be recorded as a blocker: {other:?}"
            );
            assert!(
                !rec.blockers(ReqId::sub(1, 4)).is_empty(),
                "a real R1 failure must leave NO-GO grounds under R1.4: {other:?}"
            );
        }
    }
}
