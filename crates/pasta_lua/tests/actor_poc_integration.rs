//! R8.2 / R7.3 統合走行テスト＋ VerdictDocument 成果物生成（task 8.2）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ、既定テストスイートを一切汚染しない（R7.1/R7.4）。
//!
//! 本タスク（8.2）の責務は task 7.1（段階判定ロジック＋ `run_all` 結線）と区別される:
//! - 7.1: `stage()` / `render()` / `run_all` の段階判定ロジックの正しさ（`actor_poc_verdict_stage.rs`）。
//! - 8.2: 全 probe（R1〜R6）を **end-to-end 実走** した実 `VerdictDocument` が、
//!   (a) 全項目試行結果・(b) 段階判定・(c) 後続申し送り（R5 の LuaJIT `coroutine.close`
//!   不在制約／R6 のフォールバック申し送りを含む）を成果物として含むことの確認と、
//!   その実走文書を **コミット可能な成果物ファイル** として出力する生成器（R8.2/R7.3）。
//!
//! 観測可能な完了条件（tasks.md 8.2）:
//! 「end-to-end 走行が全項目試行を含む段階判定文書を成果物として出力する。」
//!
//! 成果物の生成方法（再生成手順）:
//!   `cargo test -p pasta_lua --features actor-poc --test actor_poc_integration \
//!        generate_verdict_document_artifact -- --ignored`
//! を 1 回実行すると、実 `run_all()` の出力（実アクタースレッド／実 `!Send` VM を起動した
//! 実走結果）を `.kiro/specs/pasta-actor-feasibility/verdict-document.md` へ書き出す。
//! 通常の `cargo test` 実行ではこの生成器は `#[ignore]` のため起動せず、コミット済み
//! ファイルを非決定に書き換えない（R7.4 テスト非汚染）。アサーションは別テストが担う。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::run_all;
use pasta_lua::actor_poc::verdict::{GoNoGoStage, ReqId};

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和。
/// 写経元: `crates/pasta_lua/tests/common/mod.rs`（PASTA_DEBUG=1 がテストを壊す既知問題）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// コミット済み成果物ファイルのパス（ワークスペース相対）。
/// `CARGO_MANIFEST_DIR` は `crates/pasta_lua` を指すため、2 つ上がワークスペース root。
fn verdict_document_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(".kiro")
        .join("specs")
        .join("pasta-actor-feasibility")
        .join("verdict-document.md")
}

/// R8.2 / R7.3: 全 probe（R1〜R6）を end-to-end 実走した実 `VerdictDocument` が、
/// 全項目試行結果・段階判定・後続申し送りを **成果物として含む** ことを確認する。
///
/// 7.1 の `run_all` テストが「判定可能な文書を生む」までを見るのに対し、本テストは
/// 「成果物としての文書」が満たすべき内容契約（全項目・段階・申し送りの 3 点・特に
/// R5/R6 の具体的申し送り）を実走文書に対して assert する。いずれかが欠ければ失敗する。
#[test]
fn end_to_end_run_all_emits_full_staged_verdict_document() {
    let doc = run_all();
    let text = doc.to_text();

    // (a) 全項目試行結果: R1〜R6 すべての item が成否にかかわらず記録され、文書に現れる。
    for challenge in 1u8..=6 {
        let id = ReqId::item(challenge);
        assert!(
            doc.items.iter().any(|(rid, _)| *rid == id),
            "成果物は R{challenge} の item を含むこと（成否にかかわらず全項目試行・R8.2）"
        );
        assert!(
            text.contains(&id.to_string()),
            "成果物テキストは R{challenge} を項目別試行結果として明記すること、got:\n{text}"
        );
    }

    // (b) 段階判定: 文書は確定段階を持ち、そのラベルをテキストに含む。実走では R1〜R6 が
    //     成立する想定（GO+）。ただし段階はハードコードせず、最低ライン以上であることのみ確認。
    assert!(
        matches!(
            doc.stage,
            GoNoGoStage::ConditionalGo | GoNoGoStage::Go | GoNoGoStage::GoPlus
        ),
        "end-to-end 実走は最低ライン以上に達する想定、got: {:?}\n{text}",
        doc.stage
    );
    assert!(
        text.contains(doc.stage.label()),
        "成果物は確定した段階判定ラベルを含むこと、got:\n{text}"
    );

    // (c) 後続申し送り（R8.5）: 最低ライン以上なので結論が明記される。6 トピックが現れ、
    //     かつ R5 の LuaJIT 制約と R6 のフォールバック申し送りが具体的に載ること。
    let conclusions = doc
        .conclusions
        .as_ref()
        .expect("最低ライン以上の成果物は後続前提結論（申し送り）を持つこと（R8.5）");
    assert_eq!(
        conclusions.len(),
        6,
        "後続前提結論は 6 トピック（executor 統合／VM pin・teardown／marshaling 契約／\
         drop→204 ガード／coroutine 生存／GET レイテンシ・フォールバック）"
    );
    for topic in [
        "executor 統合",
        "teardown",
        "marshaling",
        "204",
        "coroutine",
        "レイテンシ",
        "フォールバック",
    ] {
        assert!(
            text.contains(topic),
            "成果物は後続申し送りトピック `{topic}` を明記すること、got:\n{text}"
        );
    }

    // R5 の LuaJIT coroutine.close 不在制約が R5 item の制約として実走文書に申し送られること。
    let r5 = doc
        .items
        .iter()
        .find(|(id, _)| *id == ReqId::item(5))
        .map(|(_, o)| o)
        .expect("R5 item が記録されていること");
    assert!(
        r5.constraints
            .iter()
            .any(|c| c.contains("coroutine.close") || c.contains("LuaJIT")),
        "R5 は LuaJIT coroutine.close 不在制約を申し送ること、got constraints: {:?}",
        r5.constraints
    );

    // R6 のフォールバック申し送りが結論テキストに具体的に現れること（GET タイムアウト→204）。
    assert!(
        text.contains("フォールバック") && text.contains("204"),
        "R6 の GET タイムアウト→204 フォールバック申し送りが成果物に現れること、got:\n{text}"
    );

    // 成果物テキストは非空。
    assert!(!text.trim().is_empty(), "VerdictDocument 成果物は非空テキストを生む");
}

/// 実走 `VerdictDocument` を **コミット可能な成果物ファイル** として出力する生成器。
///
/// 通常の `cargo test` を非決定に汚染しないよう `#[ignore]`。明示起動時のみ実 `run_all()`
/// を走らせ、その実走出力を `.kiro/specs/pasta-actor-feasibility/verdict-document.md` へ
/// 書き出す（手書きではなく実走反映の成果物）。再生成手順はファイル冒頭 doc を参照。
#[test]
#[ignore = "成果物生成器: --ignored で明示起動したときのみコミット済み md を書き換える"]
fn generate_verdict_document_artifact() {
    let doc = run_all();
    let body = doc.to_text();

    let header = format!(
        "<!--\n\
         この文書は自動生成された成果物です（手書き禁止）。\n\
         生成元: crates/pasta_lua/tests/actor_poc_integration.rs::generate_verdict_document_artifact\n\
         再生成: cargo test -p pasta_lua --features actor-poc --test actor_poc_integration \
         generate_verdict_document_artifact -- --ignored\n\
         実 run_all()（実アクタースレッド／実 !Send mlua VM）の end-to-end 実走を反映。\n\
         確定段階: {}\n\
         -->\n\n",
        doc.stage.label()
    );

    let path = verdict_document_path();
    let contents = format!("{header}{body}");
    std::fs::write(&path, contents)
        .unwrap_or_else(|e| panic!("成果物 {} の書き出しに失敗: {e}", path.display()));

    eprintln!(
        "verdict-document.md を生成しました: {} (段階: {})",
        path.display(),
        doc.stage.label()
    );
}
