//! R8 段階判定テスト: `stage()` / `render()` / `run_all`（task 7.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ、既定テストスイートを一切汚染しない（R7.1/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 7.1・3 つすべて）:
//! (a) R1〜R6 の成否組合せに対し正しい段階（NO-GO／条件付き GO／GO／GO+）が決まる。
//! (b) 最低ライン（R1+R2+R3）未達時に NO-GO 文書（ブロッカー＋回避候補）が出力される。
//! (c) 条件付き GO 以上で後続前提結論（採用 executor 統合方式・VM pin/teardown 方針・
//!     marshaling 契約・drop→204 ガード方針・coroutine 生存条件・GET レイテンシと
//!     フォールバック要否）が明記される。
//! また (7.3): `run_all` が全 probe を実行し判定可能な VerdictDocument を生む。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::run_all;
use pasta_lua::actor_poc::verdict::{
    Blocker, GoNoGoStage, ItemOutcome, ReqId, VerdictRecorder,
};

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和。
/// 写経元: `crates/pasta_lua/tests/common/mod.rs`（PASTA_DEBUG=1 がテストを壊す既知問題）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// 指定した R1〜R6 の成否を持つレコーダを組み立てるヘルパ。
/// `successes[i]` が `true` なら R(i+1) を success item として記録する。
/// false の項目は failure item として記録する（R8.2「全項目を成否にかかわらず記録」）。
fn recorder_with(successes: [bool; 6]) -> VerdictRecorder {
    let mut rec = VerdictRecorder::new();
    for (i, &ok) in successes.iter().enumerate() {
        let challenge = (i + 1) as u8;
        let outcome = if ok {
            ItemOutcome::success(format!("R{challenge} 成立"))
        } else {
            ItemOutcome::failure(format!("R{challenge} 不成立"))
        };
        rec.record_item(ReqId::item(challenge), outcome);
    }
    rec
}

// ---- (a) stage logic（R8.1）: 多数の成否組合せ ----

/// R1 不成立 → 必ず NO-GO（R2〜R6 が何であっても）。
#[test]
fn r1_failure_is_always_no_go() {
    // R1=false で残りを全パターン総当たり。
    for bits in 0u8..32 {
        let r2 = bits & 1 != 0;
        let r3 = bits & 2 != 0;
        let r4 = bits & 4 != 0;
        let r5 = bits & 8 != 0;
        let r6 = bits & 16 != 0;
        let rec = recorder_with([false, r2, r3, r4, r5, r6]);
        assert_eq!(
            rec.stage(),
            GoNoGoStage::NoGo,
            "R1 不成立は段階を必ず NO-GO にする（R2..R6={r2}{r3}{r4}{r5}{r6}）"
        );
    }
}

/// 最低ライン: R1∧R2∧R3 が揃って初めて 条件付き GO。R3 欠落は NO-GO。
#[test]
fn minimum_line_requires_r1_r2_r3() {
    // R1+R2 だが R3 欠落 → NO-GO（最低ライン未達）。R4 が成立していても昇格しない。
    assert_eq!(
        recorder_with([true, true, false, true, false, false]).stage(),
        GoNoGoStage::NoGo,
        "R3 欠落は最低ライン未達＝NO-GO（R4 成立でも昇格しない）"
    );
    // R1+R3 だが R2 欠落 → NO-GO。
    assert_eq!(
        recorder_with([true, false, true, false, false, false]).stage(),
        GoNoGoStage::NoGo,
        "R2 欠落は最低ライン未達＝NO-GO"
    );
    // R1+R2+R3 ちょうど → 条件付き GO。
    assert_eq!(
        recorder_with([true, true, true, false, false, false]).stage(),
        GoNoGoStage::ConditionalGo,
        "R1+R2+R3 ちょうどで 条件付き GO（最低ライン）"
    );
}

/// 条件付き GO に R4 が加わると GO。ただし R5/R6 のいずれか欠落なら GO+ には上がらない。
#[test]
fn r4_promotes_to_go_but_not_go_plus_without_r5_r6() {
    // R1+R2+R3+R4 → GO（R5/R6 欠落）。
    assert_eq!(
        recorder_with([true, true, true, true, false, false]).stage(),
        GoNoGoStage::Go,
        "R1+R2+R3+R4 で GO"
    );
    // R5 だけ追加で R6 欠落 → まだ GO（GO+ には R5∧R6 が必要）。
    assert_eq!(
        recorder_with([true, true, true, true, true, false]).stage(),
        GoNoGoStage::Go,
        "R5 のみ（R6 欠落）では GO+ に上がらない"
    );
    // R6 だけ追加で R5 欠落 → まだ GO。
    assert_eq!(
        recorder_with([true, true, true, true, false, true]).stage(),
        GoNoGoStage::Go,
        "R6 のみ（R5 欠落）では GO+ に上がらない"
    );
}

/// 全項目成立 → GO+（高信頼）。
#[test]
fn all_success_is_go_plus() {
    assert_eq!(
        recorder_with([true, true, true, true, true, true]).stage(),
        GoNoGoStage::GoPlus,
        "R1〜R6 全成立で GO+"
    );
}

/// R4 欠落のまま R5+R6 が成立しても GO には上がれない（R4 は GO の前提）。
#[test]
fn r5_r6_without_r4_stays_conditional() {
    assert_eq!(
        recorder_with([true, true, true, false, true, true]).stage(),
        GoNoGoStage::ConditionalGo,
        "R4 欠落では R5+R6 成立でも 条件付き GO 止まり（R4 は GO の前提・R5/R6 は GO+ 前提）"
    );
}

/// 未記録（item が無い）項目は不成立として扱う（保守的）。
#[test]
fn missing_item_counts_as_failure() {
    let rec = VerdictRecorder::new(); // 何も記録していない
    assert_eq!(
        rec.stage(),
        GoNoGoStage::NoGo,
        "何も記録されていなければ R1 不成立扱い＝NO-GO"
    );
}

// ---- (b) NO-GO 文書（R8.4）: ブロッカー＋回避候補 ----

/// 最低ライン未達時、`render()` は NO-GO 文書を生み、記録済みブロッカーと回避候補を含む。
#[test]
fn no_go_document_contains_blockers_and_workarounds() {
    let mut rec = recorder_with([false, false, false, false, false, false]);
    // R1.4 に NO-GO 根拠ブロッカー（回避候補つき）を記録。
    rec.record_blocker(
        ReqId::sub(1, 4),
        Blocker::with_workaround(
            "executor 上で VM ホストが不成立（reload teardown ハング）",
            "メッセージ専用ウィンドウの Drop 解放を見直し再計測する",
        ),
    );

    let doc = rec.render();
    let text = doc.to_text();

    assert_eq!(doc.stage, GoNoGoStage::NoGo, "最低ライン未達は NO-GO 文書");
    assert!(doc.is_no_go(), "NO-GO 文書として判定可能であること");
    // ブロッカーの reason が文書に含まれる。
    assert!(
        text.contains("reload teardown ハング"),
        "NO-GO 文書はブロッカー条件を含むこと、got:\n{text}"
    );
    // 回避候補が文書に含まれる。
    assert!(
        text.contains("Drop 解放を見直し"),
        "NO-GO 文書は回避候補（workaround）を含むこと、got:\n{text}"
    );
    // 結論セクション（R8.5）は最低ライン未達では出さない。
    assert!(
        doc.conclusions.is_none(),
        "最低ライン未達では後続前提結論を出さない（条件付き GO 以上のみ）"
    );
}

// ---- (c) 結論（R8.5）: 6 つの後続前提トピックが明記される ----

/// 条件付き GO 以上では `render()` が後続前提結論（6 トピック）を明記する。
#[test]
fn conclusions_name_all_six_downstream_prerequisites() {
    let rec = recorder_with([true, true, true, true, true, true]); // GO+
    let doc = rec.render();
    let conclusions = doc
        .conclusions
        .as_ref()
        .expect("GO+ は後続前提結論を持つこと");
    let text = doc.to_text();

    // 6 つの後続前提トピック（R8.5 列挙）がいずれも文書に現れること。
    let required_topics = [
        "executor 統合",   // 採用 executor 統合方式
        "VM",              // VM pin/teardown 方針
        "teardown",        // VM pin/teardown 方針
        "marshaling",      // marshaling 契約
        "204",             // drop→204 ガード方針
        "coroutine",       // coroutine 生存条件
        "レイテンシ",       // GET レイテンシとフォールバック要否
        "フォールバック",   // フォールバック要否
    ];
    for topic in required_topics {
        assert!(
            text.contains(topic),
            "結論は後続前提トピック `{topic}` を明記すること、got:\n{text}"
        );
    }
    // 構造化された結論（個別フィールド）でも 6 トピックが取り出せること。
    assert_eq!(
        conclusions.len(),
        6,
        "6 つの後続前提結論が構造化されて列挙されること"
    );
}

/// 条件付き GO でも（最低ライン到達なら）結論が出る（GO+ 限定ではない）。
#[test]
fn conditional_go_also_emits_conclusions() {
    let rec = recorder_with([true, true, true, false, false, false]);
    let doc = rec.render();
    assert_eq!(doc.stage, GoNoGoStage::ConditionalGo);
    assert!(
        doc.conclusions.is_some(),
        "条件付き GO（最低ライン到達）でも後続前提結論を明記すること"
    );
}

// ---- (7.3): run_all が全 probe を実行し判定可能な文書を生む ----

/// `run_all` は R1〜R6 全項目を成否にかかわらず試行し、判定可能な VerdictDocument を返す。
/// 実アクタースレッド／実 VM を起動するため、安定性確認のためテストとしても 2 回回せる。
#[test]
fn run_all_executes_all_probes_and_yields_judgeable_document() {
    let doc = run_all();

    // 全 6 項目が記録されていること（R8.2「全項目を試行」）。
    for challenge in 1u8..=6 {
        assert!(
            doc.items.iter().any(|(id, _)| *id == ReqId::item(challenge)),
            "run_all は R{challenge} の item を記録すること（成否にかかわらず）"
        );
    }

    // 隔離前提が判定妥当性前提として確認されていること（R8.3）。
    assert!(
        doc.isolation.all_in_force(),
        "run_all は隔離前提（default off・バイト不変・非汚染）を判定前提として確認すること"
    );

    // 段階が確定し、文書が判定可能であること。実環境では R1〜R6 が成立する想定なので
    // 最低ライン以上（条件付き GO 以上）に達するはず＝結論が明記される。
    assert!(
        matches!(
            doc.stage,
            GoNoGoStage::ConditionalGo | GoNoGoStage::Go | GoNoGoStage::GoPlus
        ),
        "実環境の run_all は最低ライン以上に達する想定、got: {:?}\n{}",
        doc.stage,
        doc.to_text()
    );
    assert!(
        doc.conclusions.is_some(),
        "条件付き GO 以上では後続前提結論が明記されること（R8.5）"
    );

    // 文書テキストが空でなく、段階ラベルを含む。
    let text = doc.to_text();
    assert!(!text.trim().is_empty(), "VerdictDocument は非空のテキストを生む");
    assert!(
        text.contains(doc.stage.label()),
        "文書は確定した段階ラベルを含むこと"
    );
}

/// run_all 安定性: 2 回連続で実行しても同じ段階に収束する（実アクタースレッド spawn/teardown
/// を反復してもクラッシュ・退行しない）。
#[test]
fn run_all_is_stable_across_repeated_runs() {
    let first = run_all().stage;
    let second = run_all().stage;
    assert_eq!(
        first, second,
        "run_all は反復実行で同じ段階に収束すること（spawn/teardown 反復で退行しない）"
    );
}
