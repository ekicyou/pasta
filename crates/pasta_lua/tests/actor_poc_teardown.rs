//! R1.2 / R1.3 実証テスト: reload teardown と反復リーク検査（task 2.2）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切汚染
//! しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 2.2）:
//! 「N 回 reload 後もハンドル/ポートが枯渇せず clean teardown する統合テストが緑。」
//!
//! 「リーク検査が芝居でない」ための設計（design.md ReloadProbe / R1.3 の crux）:
//! - 単に N 回 spawn/shutdown を回すだけでは「ハングしない・panic しない」しか
//!   証明できず、「ハンドル漏れがない」は証明できない。そこで `ReloadProbe` は
//!   **実 OS リソース計測**（Windows: `GetProcessHandleCount` のカーネルハンドル数 ＋
//!   `GetGuiResources(GR_USEROBJECTS)` の USER オブジェクト数。メッセージ専用
//!   ウィンドウは USER オブジェクトなので、サイクルごとのウィンドウ漏れはここに
//!   現れる）を **ウォームアップ後のベースライン** と **N サイクル後** で採取し、
//!   増分が小さな許容値（≒ N に比例しない）に収まることを assert する。
//! - もし各サイクルでメッセージ専用ウィンドウ／ハンドルが解放されていなければ、
//!   USER オブジェクト数 or カーネルハンドル数が ≒N で増え、この assert は確実に
//!   失敗する（テストに歯がある）。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::teardown::ReloadProbe;

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

/// reload サイクル数。実 `PastaLuaRuntime`（mlua VM）構築は非自明なコストなので、
/// 全体のテスト時間を抑えつつ「真の per-cycle リークなら N≒増分で確実に検出できる」
/// 規模として 24 回を選択する（per-cycle で USER ウィンドウ 1 個でも漏れれば +24 と
/// なり、許容値を大きく超える）。
const RELOAD_CYCLES: usize = 24;

/// R1.2: shutdown→再 spawn の reload サイクルを N 回反復し、各サイクルが
/// clean teardown（JoinHandle join 成功・panic/hang なし）することを確認する。
#[test]
fn every_reload_cycle_tears_down_cleanly() {
    let report = ReloadProbe::run_cycles(RELOAD_CYCLES);

    assert_eq!(
        report.cycles_run, RELOAD_CYCLES,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        report.clean_teardowns, RELOAD_CYCLES,
        "every reload cycle must tear down cleanly (shutdown flag -> JoinHandle::join, no panic)"
    );
    assert!(
        report.all_clean(),
        "all_clean() must hold when every cycle joined cleanly"
    );
}

/// R1.3: N 回 reload 後も実 OS リソース（カーネルハンドル数・USER オブジェクト数）が
/// 枯渇／単調リークしないことを、ウォームアップ後ベースラインとの **増分上限** で確認する。
///
/// この assert は「サイクルごとにメッセージ専用ウィンドウ・スレッド・チャネルが
/// 解放される」ことを実測で裏取りする。解放されなければ増分が ≒N に達して失敗する。
#[test]
fn repeated_reload_does_not_leak_os_resources() {
    let report = ReloadProbe::run_cycles(RELOAD_CYCLES);

    // 計測が成立し、サイクルがすべて clean であることが前提（リーク判定の妥当性前提）。
    assert!(report.all_clean(), "leak check is only meaningful when teardown is clean");

    #[cfg(windows)]
    {
        let leak = report
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");

        // 増分上限: per-cycle リーク（≒N=24）なら確実に超える小さな許容値。
        // teardown が真に解放していれば増分は 0 付近に収まる。OS/アロケータの
        // 一回性ゆらぎを吸収するための小さな余裕のみを許す。
        const HANDLE_TOLERANCE: i64 = 8;
        const USER_OBJECT_TOLERANCE: i64 = 8;

        assert!(
            leak.kernel_handle_growth <= HANDLE_TOLERANCE,
            "kernel handle count grew by {} across {} reload cycles (baseline={}, final={}); \
             a per-cycle handle leak would show ~{} growth (tolerance {})",
            leak.kernel_handle_growth,
            RELOAD_CYCLES,
            leak.kernel_handles_baseline,
            leak.kernel_handles_final,
            RELOAD_CYCLES,
            HANDLE_TOLERANCE
        );

        assert!(
            leak.user_object_growth <= USER_OBJECT_TOLERANCE,
            "USER object count grew by {} across {} reload cycles (baseline={}, final={}); \
             a leaked message-only window per cycle would show ~{} growth (tolerance {})",
            leak.user_object_growth,
            RELOAD_CYCLES,
            leak.user_objects_baseline,
            leak.user_objects_final,
            RELOAD_CYCLES,
            USER_OBJECT_TOLERANCE
        );
    }

    #[cfg(not(windows))]
    {
        // 本 PoC は Windows ターゲット。非 Windows では実ハンドル計測 API がないため、
        // clean teardown のみを保証する（leak_metric は None）。
        assert!(
            report.leak_metric.is_none(),
            "non-windows builds do not sample a handle metric"
        );
    }
}
