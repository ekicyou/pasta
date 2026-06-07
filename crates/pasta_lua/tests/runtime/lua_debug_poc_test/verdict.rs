//! VerdictRecorder: `ItemOutcome` の収集と `compute_tier` による段階的 GO 判定の
//! 算出・出力（R5/R6 検証 / task 2.4）。
//!
//! 兄弟モジュール `harness_types` の共有型（`ItemOutcome`、`Tier`）を再利用する。
//!
//! ## 設計準拠（design.md "VerdictRecorder" / requirements.md R6.1〜R6.4）
//! - `compute_tier`: 段階的 GO 判定は R1→R2→R3→R4 の **単調な積み上げ**
//!   （design "Invariants: Tier は R1→R2→R3→R4 の単調な積み上げ"）。
//!   - **NoGo**: 最低ライン（R1 ＋ R2）が**両方とも**成立しないとき（R6.1 / R6.4）。
//!     R1 がいかなる方式でも不成立なら当然 NoGo に含まれる（R6.1）。
//!   - **ConditionalGo**: R1 ＋ R2 が成立（R6.1）。
//!   - **Go**: さらに R3 が成立（R6.1）。
//!   - **GoPlus**: さらに R4 が成立（R6.1）。R3 が不成立なら R4 が成立していても
//!     GoPlus に**昇格しない**（単調性: R4 は R3 を飛ばして数えない）。
//! - `report`: cargo test に判定可能な出力を行い（`println!` 用）、`research.md`
//!   への追記にも使える人間可読な文字列を返す。**全項目**（成否によらず）の
//!   id / passed / method / notes を列挙し（R6.2）、算出された `Tier` と、判定の
//!   妥当性前提＝R5 隔離条件（feature-gate ／ `cargo test` 実行 ／ サンドボックス
//!   維持）を注記する（R6.3）。
//!
//! ## id 規約
//! チャレンジ項目 R1〜R4 は `ItemOutcome.id` を文字列 `"R1"` 〜 `"R4"` で識別する。
//! `compute_tier` は id でルックアップし、該当 id の `ItemOutcome` が無い場合は
//! **未成立（not-passed）扱い**とする（欠落 = 安全側に倒す）。R5 系（`"R5"`）など
//! 他の id は判定段階の算出には影響せず、`report` には列挙される。

#![allow(dead_code)]

use super::harness_types::{ItemOutcome, Tier};

/// R5 隔離条件（判定の妥当性前提）の人間可読な注記。`report` が末尾に付与する。
const ISOLATION_PREMISE: &str = "判定の妥当性前提（R5 隔離条件 / R6.3）: \
feature-gate（lua-debug-poc・default 無効）／`cargo test` 実行／\
サンドボックス維持（std_debug 非露出）が成立していること。";

/// 指定 id の項目が **存在しかつ passed** であれば `true`。
/// 該当 id が無い場合は未成立（`false`）扱い（欠落 = not-passed）。
fn passed(outcomes: &[ItemOutcome], id: &str) -> bool {
    outcomes
        .iter()
        .find(|o| o.id == id)
        .map(|o| o.passed)
        .unwrap_or(false)
}

/// 項目結果から段階的 GO 判定を算出する（R6.1 / R6.4）。
///
/// R1→R2→R3→R4 の単調な積み上げ。ネストしたゲートで実装し、上位段階が
/// 下位段階の成立を必須とすること（単調性）を構造で担保する。
pub(crate) fn compute_tier(outcomes: &[ItemOutcome]) -> Tier {
    // 最低ライン: R1 ＋ R2 が両方 passed でなければ NoGo（R6.4）。
    // R1 がいかなる方式でも不成立なら、この時点で NoGo（R6.1）。
    if !(passed(outcomes, "R1") && passed(outcomes, "R2")) {
        return Tier::NoGo;
    }
    // ここから単調な積み上げ。R3 未成立なら ConditionalGo で打ち切り
    // （R4 が成立していても R3 を飛ばして GoPlus にしない＝単調性）。
    if !passed(outcomes, "R3") {
        return Tier::ConditionalGo;
    }
    // R3 まで成立。R4 未成立なら Go で打ち切り。
    if !passed(outcomes, "R4") {
        return Tier::Go;
    }
    // R1〜R4 すべて成立。
    Tier::GoPlus
}

/// cargo test 出力 ＋ `research.md` 追記用の人間可読レポートを生成する
/// （R6.2 / R6.3 / R5.2）。
///
/// - 全項目（成否によらず）の id / passed / method / notes を列挙する（R6.2）。
/// - 算出された `Tier` を明記する。
/// - 判定の妥当性前提（R5 隔離条件）を注記する（R6.3）。
pub(crate) fn report(outcomes: &[ItemOutcome]) -> String {
    let tier = compute_tier(outcomes);
    let mut out = String::new();

    out.push_str("=== PoC 検証結果（段階的 GO 判定 / R6） ===\n");
    out.push_str("項目別結果（成否によらず全項目を記録 / R6.2）:\n");

    if outcomes.is_empty() {
        out.push_str("  (項目なし)\n");
    } else {
        for o in outcomes {
            let mark = if o.passed { "PASS" } else { "FAIL" };
            out.push_str(&format!(
                "  - {id} [{mark}] method={method} notes={notes}\n",
                id = o.id,
                mark = mark,
                method = o.method,
                notes = o.notes,
            ));
        }
    }

    out.push_str(&format!("算出 Tier: {tier:?}\n"));
    out.push_str(ISOLATION_PREMISE);
    out.push('\n');

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用 `ItemOutcome` の簡易コンストラクタ。
    fn outcome(id: &str, passed: bool) -> ItemOutcome {
        ItemOutcome {
            id: id.to_string(),
            passed,
            method: format!("{id}-method"),
            notes: format!("{id}-notes"),
        }
    }

    /// 最低ライン（R1 ＋ R2）未達は NoGo（R6.4）。
    /// R1 不成立、および R1 成立だが R2 不成立の双方を検証。
    #[test]
    fn tier_nogo_when_minimum_line_unmet() {
        // R1 fail（R2 以降が成立していても NoGo）。
        let r1_fail = vec![
            outcome("R1", false),
            outcome("R2", true),
            outcome("R3", true),
            outcome("R4", true),
        ];
        assert_eq!(compute_tier(&r1_fail), Tier::NoGo);

        // R1 pass だが R2 fail でも NoGo（最低ラインは R1 ＋ R2 の両方）。
        let r2_fail = vec![
            outcome("R1", true),
            outcome("R2", false),
            outcome("R3", true),
            outcome("R4", true),
        ];
        assert_eq!(compute_tier(&r2_fail), Tier::NoGo);
    }

    /// R1 ＋ R2 成立・R3 不成立は ConditionalGo（R6.1）。
    #[test]
    fn tier_conditional_go_on_r1_r2() {
        let outcomes = vec![
            outcome("R1", true),
            outcome("R2", true),
            outcome("R3", false),
            outcome("R4", false),
        ];
        assert_eq!(compute_tier(&outcomes), Tier::ConditionalGo);
    }

    /// R1 ＋ R2 ＋ R3 成立・R4 不成立は Go（R6.1）。
    #[test]
    fn tier_go_on_r3() {
        let outcomes = vec![
            outcome("R1", true),
            outcome("R2", true),
            outcome("R3", true),
            outcome("R4", false),
        ];
        assert_eq!(compute_tier(&outcomes), Tier::Go);
    }

    /// R1〜R4 すべて成立は GoPlus（R6.1）。
    #[test]
    fn tier_goplus_on_r4() {
        let outcomes = vec![
            outcome("R1", true),
            outcome("R2", true),
            outcome("R3", true),
            outcome("R4", true),
        ];
        assert_eq!(compute_tier(&outcomes), Tier::GoPlus);
    }

    /// 単調性: R3 不成立なら R4 成立でも GoPlus に昇格せず ConditionalGo に留まる
    /// （R4 は R3 を飛ばして数えない / design "単調な積み上げ"）。
    #[test]
    fn tier_monotonic_r4_requires_r3() {
        let outcomes = vec![
            outcome("R1", true),
            outcome("R2", true),
            outcome("R3", false),
            outcome("R4", true),
        ];
        assert_eq!(compute_tier(&outcomes), Tier::ConditionalGo);
    }

    /// PoC の実証済みチャレンジ項目（R1〜R4）を集約し、全項目成立 → GO+ 判定を
    /// 算出・出力する統合テスト（task 4.2 / Req 5.2・6.2・6.5）。
    ///
    /// 各 `ItemOutcome` の id は `compute_tier` のルックアップ規約（exact な
    /// `"R1"`〜`"R4"`）に従い、`method`/`notes` は実装フェーズで実証された結果を
    /// 要約する（hook_probe / session / frame_inspector / transport_loop の各テスト
    /// が裏付け）。`report` の出力は `--nocapture` で cargo test に現れ（Req 5.2）、
    /// 各 id・算出 Tier・R5 隔離前提を含むことを assert する（Req 6.2・6.3）。
    #[test]
    fn poc_verdict_aggregation_reports_goplus() {
        // PoC 実装フェーズの実証結果を集約（R1〜R4 すべて成立）。
        let outcomes = vec![
            ItemOutcome {
                id: "R1".to_string(),
                passed: true,
                method: "global jit.off() + set_global_hook (HookStrategy::GlobalHook)"
                    .to_string(),
                notes: "EVERY_LINE global hook fires across Lua-side coroutine.create \
                        dynamic coroutines on a global-jit.off VM; no D1 fallback needed"
                    .to_string(),
            },
            ItemOutcome {
                id: "R2".to_string(),
                passed: true,
                method: "in-hook blocking recv() (no yield), VmState::Continue".to_string(),
                notes: "stops at the target line (progress flag frozen) and resumes on \
                        the Continue command"
                    .to_string(),
            },
            ItemOutcome {
                id: "R3".to_string(),
                passed: true,
                method: "Lua::exec_raw + mlua::ffi (lua_getstack/lua_getlocal/\
                         lua_getupvalue/lua_getinfo)"
                    .to_string(),
                notes: "locals & upvalues by name+value (number/string/boolean/table) \
                        with std_debug NOT exposed; main-frame only (R3.4: running \
                        coroutine body frames unreachable from the main state)"
                    .to_string(),
            },
            ItemOutcome {
                id: "R4".to_string(),
                passed: true,
                method: "std::net loopback (127.0.0.1:0), minimal line protocol \
                         stopped/vars/continue, 3-thread split"
                    .to_string(),
                notes: "stop -> inspect -> resume round-trip carries real FFI-inspected \
                        vars; std-only, no extra crates"
                    .to_string(),
            },
        ];

        // 全 4 項目成立 → 単調な積み上げで GO+（GoPlus）。
        assert_eq!(
            compute_tier(&outcomes),
            Tier::GoPlus,
            "all of R1..R4 passed must compute Tier::GoPlus (Req 6.1)"
        );

        // 判定可能な出力を cargo test に表示する（Req 5.2 / `-- --nocapture`）。
        let text = report(&outcomes);
        println!("{text}");

        // 各チャレンジ項目 id が出力に含まれる（全項目記録 / Req 6.2）。
        assert!(text.contains("R1"), "report must list R1");
        assert!(text.contains("R2"), "report must list R2");
        assert!(text.contains("R3"), "report must list R3");
        assert!(text.contains("R4"), "report must list R4");

        // 算出 Tier が出力に含まれる。
        assert!(
            text.contains("GoPlus"),
            "report must include the computed Tier (GoPlus)"
        );

        // R5 隔離前提が注記されている（Req 6.3）。
        assert!(
            text.contains("R5"),
            "report must mention the R5 isolation premise"
        );
        assert!(
            text.contains("feature-gate"),
            "report must mention the feature-gate isolation premise"
        );
    }

    /// report は全項目（失敗項目を含む）の id を列挙し（R6.2）、R5 隔離前提
    /// （feature-gate / cargo test / サンドボックス）と算出 Tier を含む（R6.3）。
    #[test]
    fn report_includes_all_items_and_isolation_premise() {
        let outcomes = vec![
            ItemOutcome {
                id: "R1".to_string(),
                passed: true,
                method: "GlobalHook".to_string(),
                notes: "line hook fired".to_string(),
            },
            ItemOutcome {
                id: "R2".to_string(),
                passed: true,
                method: "blocking recv".to_string(),
                notes: "paused and resumed".to_string(),
            },
            ItemOutcome {
                id: "R3".to_string(),
                passed: false,
                method: "ffi exec_raw".to_string(),
                notes: "coroutine frame unreachable".to_string(),
            },
            ItemOutcome {
                id: "R4".to_string(),
                passed: false,
                method: "not attempted".to_string(),
                notes: "blocked on R3".to_string(),
            },
        ];

        let text = report(&outcomes);

        // 全項目の id が含まれる（成否によらず / R6.2）。失敗項目 R3/R4 も含む。
        assert!(text.contains("R1"), "report must list R1");
        assert!(text.contains("R2"), "report must list R2");
        assert!(text.contains("R3"), "report must list R3 (failing item)");
        assert!(text.contains("R4"), "report must list R4 (failing item)");

        // 失敗項目の notes が記録されている（個別記録 / R6.2）。
        assert!(
            text.contains("coroutine frame unreachable"),
            "report must record the failing item's notes"
        );

        // R5 隔離前提が注記されている（R6.3）。
        assert!(
            text.contains("feature-gate"),
            "report must mention the feature-gate isolation premise"
        );
        assert!(
            text.contains("cargo test"),
            "report must mention the cargo test isolation premise"
        );
        assert!(
            text.contains("サンドボックス"),
            "report must mention the sandbox isolation premise"
        );

        // 算出 Tier が含まれる（この入力は R3 fail なので ConditionalGo）。
        assert_eq!(compute_tier(&outcomes), Tier::ConditionalGo);
        assert!(
            text.contains("ConditionalGo"),
            "report must include the computed Tier"
        );
    }
}
