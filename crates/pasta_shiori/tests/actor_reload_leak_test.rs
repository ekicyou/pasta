//! reload リーク検査の統合テスト（task 4.1・R7.2/R7.3）。
//!
//! reload（spawn→teardown ×N）でカーネルハンドル／USER オブジェクトがリーク・枯渇
//! しないことを、実 OS カウンタ（`GetProcessHandleCount`/`GetGuiResources`）で検査する。
//!
//! # なぜ専用のテストバイナリに置くか（flake の根本原因）
//! 計測するカウンタは **プロセス全体**の値である。libtest は同一バイナリ内のテストを
//! 並列スレッドで走らせるため、以前 `actor_teardown_test.rs` に同居していた頃は、
//! 兄弟テスト（各々アクタースレッド＋VM を spawn→teardown する）のハンドル解放が
//! 計測窓の途中に重なり、baseline だけが兄弟分（~14）高く採られて growth が負に振れ、
//! slope 判定が偽陽性になった（遅い CI ランナーでのみ顕在化）。
//! cargo はテストバイナリを直列に実行するので、本ファイルに計測するテストはこの 1 本だけを置けば
//! 計測中に同一プロセスで他のテストが動くことはない。**このファイルにアクターを起こすテストを
//! 追加しないこと。**（アクターも OS の資源も触らない `user_objects_from` の単体テストだけは同居させる。）
//!
//! 検査の一式（`ReloadProbe`・OS の資源の数え上げ）は、このテストだけが使うので出荷のコードでなく
//! ここに置く（ponytail-shiori 6.1）。

use std::path::{Path, PathBuf};
use std::time::Duration;

use pasta::actor::mailbox::mailbox;
use pasta::actor::teardown::teardown_via_sender;
use pasta::actor::thread::spawn_actor_thread;
use tempfile::TempDir;

/// reload 反復の結果レポート（R7.2 の re-spawn ＋ R7.3 のリーク不在）。
#[derive(Debug, Clone, Copy)]
struct ReloadReport {
    /// 実際に回した reload サイクル数（途中で hang/panic していなければ要求数と一致）。
    cycles_run: usize,
    /// うち clean に teardown（done ack 受信・異常なし）したサイクル数。
    clean_teardowns: usize,
    /// 実 OS リソースのリーク指標（Windows のみ `Some`。非 Windows は `None`）。
    leak_metric: Option<LeakMetric>,
}

/// reload 反復前後の実 OS リソースカウンタとその増分（R7.3）。
///
/// ベースラインは **ウォームアップ後** に採取する（一回性初期化を除外するため）。
/// 計測は `GetProcessHandleCount` / `GetGuiResources(GR_USEROBJECTS)` による。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(windows), allow(dead_code))]
struct LeakMetric {
    /// ウォームアップ後・計測サイクル開始時のカーネルハンドル数。
    kernel_handles_baseline: u32,
    /// N サイクル完了後のカーネルハンドル数。
    kernel_handles_final: u32,
    /// カーネルハンドル数の増分（final - baseline）。
    kernel_handle_growth: i64,
    /// ウォームアップ後・計測サイクル開始時の USER オブジェクト数。
    user_objects_baseline: u32,
    /// N サイクル完了後の USER オブジェクト数。
    user_objects_final: u32,
    /// USER オブジェクト数の増分（final - baseline）。
    user_object_growth: i64,
}

/// reload teardown の反復リーク検査ハーネス（design「Reload / Teardown Tests」）。
///
/// spawn→teardown（`Stop{done}` ack）→ 再 spawn のサイクルを反復し、各サイクルで
/// アクタースレッド・VM・チャネルが漏れなく解放される
/// （clean teardown）こと、サイクル間でリソースが累積しない（リーク／枯渇不在）ことを
/// **done ack 後に** 実測する。
struct ReloadProbe;

impl ReloadProbe {
    /// ウォームアップに使うサイクル数。最初の数回は実 VM 構築の一回性コストを
    /// 吸収させ、ベースライン採取から除外する。
    const WARMUP_CYCLES: usize = 2;

    /// reload（spawn→teardown）サイクルを `n` 回反復し、各サイクルの teardown 結果と
    /// 累積リーク指標を返す。
    ///
    /// 各サイクルは新規 `spawn_actor_thread`（新スレッド＋新 VM＋新チャネル）→
    /// 送信端からの `teardown_via_sender`（`Stop{done}` ack）→ 手放し（detach）。ack を受けてから
    /// 次サイクルを spawn するため、計測は **done ack 後** に行われる（解放完了後の実測）。
    ///
    /// - Postconditions: 全サイクルで done ack（clean teardown）・再 spawn 正常稼働。
    /// - Invariants: スレッドハンドル／USER オブジェクト／port がサイクル間で枯渇しない。
    fn run_cycles(load_dir: &Path, n: usize, timeout: Duration) -> ReloadReport {
        // (1) ウォームアップ: 実 VM 構築の一回性コストを baseline から除外する。
        for _ in 0..Self::WARMUP_CYCLES {
            let _ = run_one_reload_cycle(load_dir, timeout);
        }

        // (2) ウォームアップ後のベースライン採取（計測サイクルの開始点）。
        let baseline = sample_resources();

        // (3) 計測対象の N サイクルを回す。
        let mut clean_teardowns = 0usize;
        for _ in 0..n {
            if run_one_reload_cycle(load_dir, timeout) {
                clean_teardowns += 1;
            }
        }

        // (4) N サイクル後のリソースカウンタ採取と増分算出（done ack 後の実測）。
        let final_sample = sample_resources();
        let leak_metric = LeakMetric::from_samples(baseline, final_sample);

        ReloadReport {
            cycles_run: n,
            clean_teardowns,
            leak_metric,
        }
    }
}

/// reload を 1 回（spawn→teardown）回し、teardown が clean（ack が届き異常が無い）だったかを返す。
fn run_one_reload_cycle(load_dir: &Path, timeout: Duration) -> bool {
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir.to_path_buf(), rx);
    let report = teardown_via_sender(&tx, timeout);
    // ack の有無に関わらず手放す（join でハングさせない）。ack 済みならループは抜け済み。
    actor.detach();
    report.acked && report.anomaly.is_none()
}

impl LeakMetric {
    /// 2 つのスナップショットから増分指標を組み立てる。両方が実値を持つ
    /// （＝Windows で計測成功）ときのみ `Some`。
    fn from_samples(baseline: ResourceSample, final_sample: ResourceSample) -> Option<LeakMetric> {
        let kernel_handles_baseline = baseline.kernel_handles?;
        let kernel_handles_final = final_sample.kernel_handles?;
        let user_objects_baseline = baseline.user_objects?;
        let user_objects_final = final_sample.user_objects?;

        Some(LeakMetric {
            kernel_handles_baseline,
            kernel_handles_final,
            kernel_handle_growth: i64::from(kernel_handles_final)
                - i64::from(kernel_handles_baseline),
            user_objects_baseline,
            user_objects_final,
            user_object_growth: i64::from(user_objects_final) - i64::from(user_objects_baseline),
        })
    }
}

/// プラットフォーム非依存のリソーススナップショット（採取できなければ `None`）。
#[derive(Debug, Clone, Copy)]
struct ResourceSample {
    /// カーネルハンドル数（Windows のみ）。
    kernel_handles: Option<u32>,
    /// USER オブジェクト数（Windows のみ）。
    user_objects: Option<u32>,
}

/// 現在プロセスの実 OS リソースカウンタを採取する（PoC 計測アプローチの流用）。
///
/// Windows では `GetProcessHandleCount`（カーネルハンドル・スレッド／イベント／
/// チャネル由来ハンドルが漏れれば増える）と `GetGuiResources(GR_USEROBJECTS)`
/// （USER オブジェクト。アクターはウィンドウを作らないので増えないはず）を読む。非 Windows では
/// 計測手段がないため `None`。
#[cfg(windows)]
fn sample_resources() -> ResourceSample {
    use windows_sys::Win32::Foundation::{GetLastError, SetLastError};
    use windows_sys::Win32::System::Threading::{
        GR_USEROBJECTS, GetCurrentProcess, GetGuiResources, GetProcessHandleCount,
    };

    // SAFETY: `GetCurrentProcess` は擬似ハンドル（クローズ不要）を返す。
    // `GetProcessHandleCount` には有効な out ポインタを渡す。いずれも純粋な
    // 読み取り API で、プロセス状態を変更しない。
    let process = unsafe { GetCurrentProcess() };

    let kernel_handles = unsafe {
        let mut count: u32 = 0;
        if GetProcessHandleCount(process, &mut count) != 0 {
            Some(count)
        } else {
            None
        }
    };

    // SAFETY: `GetGuiResources` は読み取り専用。`SetLastError`/`GetLastError` は
    // 呼び出しスレッドの誤りの番号を読み書きするだけ。アクターはウィンドウを作らないので
    // 試験のプロセスでは本当に 0 個になりうる。0 は誤りの番号で読み分ける。
    let user_objects = unsafe {
        SetLastError(0);
        let n = GetGuiResources(process, GR_USEROBJECTS);
        user_objects_from(n, GetLastError())
    };

    ResourceSample {
        kernel_handles,
        user_objects,
    }
}

/// `GetGuiResources` の戻り値と、その直後の `GetLastError` から USER オブジェクト数を読む。
/// 0 は「USER オブジェクト 0 個」と「呼び出し失敗」の両方を表すので、誤りの番号で読み分ける
/// （誤りの番号 0 なら本当に 0 個、それ以外は測れない）。
#[cfg(windows)]
fn user_objects_from(n: u32, last_error: u32) -> Option<u32> {
    if n != 0 || last_error == 0 {
        Some(n)
    } else {
        None
    }
}

/// 非 Windows: 実ハンドル計測 API がないため常に計測不能（`None`）。
#[cfg(not(windows))]
fn sample_resources() -> ResourceSample {
    ResourceSample {
        kernel_handles: None,
        user_objects: None,
    }
}

/// `GetGuiResources` の 0 を、`GetLastError` で「本当に 0」と「測れない」に読み分ける（8.4）。
/// USER オブジェクトが 0 のプロセス（誤りの番号 0）では `Some(0)` を返す。
#[cfg(windows)]
#[test]
fn user_objects_zero_is_read_by_last_error() {
    assert_eq!(user_objects_from(0, 0), Some(0));
    assert_eq!(user_objects_from(0, 5), None);
    assert_eq!(user_objects_from(3, 0), Some(3));
}

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元: `common/mod.rs`。
#[ctor::ctor(unsafe)]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// async_callback フィクスチャを temp へ展開し `load_dir` を返す。`TempDir` は
/// 返り値で寿命保持する（アクタースレッドより長命）。写経元: `actor_teardown_test.rs`。
fn build_async_callback_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
    copy_dir_recursive(&fixture_src, temp.path()).expect("copy fixture");

    (temp.path().to_path_buf(), temp)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// R7.2/R7.3: reload（spawn→teardown ×N）でカーネルハンドル／USER オブジェクトが
/// リーク・枯渇しない。done ack 後に計測する（PoC の実 OS 計測アプローチを流用）。
///
/// # 判定（slope 法）
/// 真の per-cycle リークは増分が N に **線形比例**して増える（growth ≈ L·N）。一方
/// プロセス内の残留ノイズ（ランタイムの一回性確保等）は N に比例しない。そこで小さな N と
/// 3×N の 2 水準で計測し、**増分の傾き（per-cycle 増分）が大きい N で増えていない**ことを
/// assert する:
///   - L=0（リークなし）: growth(N)≈growth(3N)。差は N に依存せず小さい。
///   - L≥1（per-cycle リーク）: growth(3N)-growth(N) ≈ L·2N。N=6 なら ≥12 となり、
///     小さなノイズ許容を確実に超える → 検出される。
#[test]
fn repeated_reload_tears_down_and_does_not_leak() {
    let (load_dir, _temp) = build_async_callback_dir();

    // 小さい水準 N と大きい水準 3×N。per-cycle リーク 1 個でも slope ≈ L·2N = 12（N=6）。
    const N_SMALL: usize = 6;
    const N_LARGE: usize = N_SMALL * 3; // 18

    // 各 run_cycles は内部でウォームアップ→baseline→N サイクル→final を完結させるため、
    // 連続呼び出しは互いに独立・公平（それぞれ自前の baseline を採る）。
    let small = ReloadProbe::run_cycles(&load_dir, N_SMALL, Duration::from_secs(10));
    let large = ReloadProbe::run_cycles(&load_dir, N_LARGE, Duration::from_secs(10));

    assert_eq!(
        small.cycles_run, N_SMALL,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        large.cycles_run, N_LARGE,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        small.clean_teardowns, N_SMALL,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );
    assert_eq!(
        large.clean_teardowns, N_LARGE,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );

    #[cfg(windows)]
    {
        let small_leak = small
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");
        let large_leak = large
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");

        // slope（傾き）法の許容: per-cycle リークが 1 でもあれば growth(3N)-growth(N)
        // ≈ L·2N = 12（N=6）となるので、これを確実に下回る許容を置く。
        const SLOPE_TOLERANCE: i64 = 8;

        let handle_slope = large_leak.kernel_handle_growth - small_leak.kernel_handle_growth;
        let user_slope = large_leak.user_object_growth - small_leak.user_object_growth;

        assert!(
            handle_slope <= SLOPE_TOLERANCE,
            "kernel handle growth scales with cycle count: growth({}) = {} but growth({}) = {} \
             (delta = {} > tolerance {}); a per-cycle handle leak of L would make this delta \
             ~L*2N = ~{}. small(baseline={}, final={}), large(baseline={}, final={})",
            N_SMALL,
            small_leak.kernel_handle_growth,
            N_LARGE,
            large_leak.kernel_handle_growth,
            handle_slope,
            SLOPE_TOLERANCE,
            2 * N_SMALL,
            small_leak.kernel_handles_baseline,
            small_leak.kernel_handles_final,
            large_leak.kernel_handles_baseline,
            large_leak.kernel_handles_final,
        );

        assert!(
            user_slope <= SLOPE_TOLERANCE,
            "USER object growth scales with cycle count: growth({}) = {} but growth({}) = {} \
             (delta = {} > tolerance {}); a leaked USER object (window) per cycle would make this \
             delta ~2N = ~{}. small(baseline={}, final={}), large(baseline={}, final={})",
            N_SMALL,
            small_leak.user_object_growth,
            N_LARGE,
            large_leak.user_object_growth,
            user_slope,
            SLOPE_TOLERANCE,
            2 * N_SMALL,
            small_leak.user_objects_baseline,
            small_leak.user_objects_final,
            large_leak.user_objects_baseline,
            large_leak.user_objects_final,
        );
    }

    #[cfg(not(windows))]
    {
        assert!(
            small.leak_metric.is_none() && large.leak_metric.is_none(),
            "non-windows builds do not sample a handle metric"
        );
    }
}
