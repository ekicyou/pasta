//! reload teardown と反復リーク検査（R1.2 / R1.3・task 2.2）。
//!
//! design.md「ReloadProbe」コンポーネントの実装。reload＝[`ActorThread::shutdown`]
//! → 再 [`ActorThread::spawn`] のサイクルを N 回反復し、各サイクルで
//! **メッセージ専用ウィンドウ（executor のメッセージループが持つ）・スレッド・
//! チャネル** が漏れなく解放（clean teardown）されることを確認する。
//!
//! # teardown idiom の出所
//!
//! 解放そのものは [`ActorThread`] が既に持つ `DebugHandle::Drop` 写経
//! （shutdown `AtomicBool` → `wake` → `JoinHandle::join`、`Drop` フォールバック）
//! に委譲する。`ReloadProbe` はその idiom を **反復** して回し、サイクル間で
//! リソースが累積しないことを実測する役割に徹する（design「Integration: debug
//! `DebugHandle::Drop` idiom を写経」）。
//!
//! # 「リーク検査を芝居にしない」ための実 OS 計測（R1.3 の crux）
//!
//! 単に spawn/shutdown を N 回回すだけでは「ハングしない・panic しない」しか
//! 言えない。本モジュールは Windows 上で **実リソースカウンタ** を採取する:
//!
//! - [`GetProcessHandleCount`] — プロセスのカーネルハンドル総数。スレッド／
//!   イベント／チャネル由来ハンドルがサイクルごとに漏れればここが増える。
//! - [`GetGuiResources`]`(GR_USEROBJECTS)` — プロセスの USER オブジェクト数。
//!   **メッセージ専用ウィンドウは USER オブジェクト** なので、executor の
//!   メッセージウィンドウがサイクルごとに破棄されなければここが ≒N 増える。
//!
//! 最初の数サイクルを **ウォームアップ** として捨ててからベースラインを採る
//! （初回 `PastaLuaRuntime` 構築は一回性のアロケーション／遅延初期化を伴いうる）。
//! その後 N サイクルを回し、増分が小さな許容値（≒N に比例しない）に収まることを
//! 呼び出し側テストが assert する。真の per-cycle リークがあれば増分は ≒N となり
//! 検出される。
//!
//! 非 Windows ではこれらの API が無いため `leak_metric` は `None` とし、clean
//! teardown のみを保証する（本 PoC は Windows ターゲット）。

use super::actor_thread::ActorThread;

/// reload を 1 回（spawn→shutdown）回し、teardown が clean だったかを返す。
///
/// VM ホスティングが成立していること自体は task 2.1 のテストが担保するため、
/// ここでは「spawn できる」「shutdown が join できる」ことだけを観測する。
fn run_one_reload_cycle() -> bool {
    let handle = ActorThread::spawn();
    // 単一 teardown（写経: shutdown フラグ→wake→JoinHandle::join）。
    let report = handle.shutdown();
    report.joined_cleanly
}

/// 反復 reload の結果レポート（R1.2 の clean teardown ＋ R1.3 のリーク指標）。
#[derive(Debug, Clone, Copy)]
pub struct ReloadReport {
    /// 実際に回した reload サイクル数（途中で hang/panic していなければ要求数と一致）。
    pub cycles_run: usize,
    /// うち clean に teardown（`JoinHandle::join` 成功）したサイクル数。
    pub clean_teardowns: usize,
    /// 実 OS リソースのリーク指標（Windows のみ `Some`。非 Windows は `None`）。
    pub leak_metric: Option<LeakMetric>,
}

impl ReloadReport {
    /// 全サイクルが clean teardown だったか（R1.2）。
    pub fn all_clean(&self) -> bool {
        self.cycles_run > 0 && self.clean_teardowns == self.cycles_run
    }
}

/// reload 反復前後の実 OS リソースカウンタとその増分（R1.3）。
///
/// ベースラインは **ウォームアップ後** に採取する（一回性初期化を除外するため）。
#[derive(Debug, Clone, Copy)]
pub struct LeakMetric {
    /// ウォームアップ後・計測サイクル開始時のカーネルハンドル数。
    pub kernel_handles_baseline: u32,
    /// N サイクル完了後のカーネルハンドル数。
    pub kernel_handles_final: u32,
    /// カーネルハンドル数の増分（final - baseline。負なら 0 方向）。
    pub kernel_handle_growth: i64,
    /// ウォームアップ後・計測サイクル開始時の USER オブジェクト数。
    pub user_objects_baseline: u32,
    /// N サイクル完了後の USER オブジェクト数。
    pub user_objects_final: u32,
    /// USER オブジェクト数の増分（final - baseline）。
    pub user_object_growth: i64,
}

/// reload teardown の反復リーク検査ハーネス（design「ReloadProbe」）。
pub struct ReloadProbe;

impl ReloadProbe {
    /// ウォームアップに使うサイクル数。最初の数回は実 VM 構築の一回性コストを
    /// 吸収させ、ベースライン採取から除外する（design「Warm up first」）。
    const WARMUP_CYCLES: usize = 3;

    /// reload（shutdown→再 spawn）サイクルを `n` 回反復し、各サイクルの teardown
    /// 結果と累積リーク指標を返す（design Service Interface
    /// `ReloadProbe::run_cycles(n) -> ReloadReport`）。
    ///
    /// - Postconditions: 全サイクルで clean teardown、リーク指標が（呼び出し側の
    ///   許容値内で）枯渇しない。
    /// - Invariants: ポート/ハンドルがサイクル間で枯渇しない。
    pub fn run_cycles(n: usize) -> ReloadReport {
        // (1) ウォームアップ: 実 VM 構築の一回性コストを baseline から除外する。
        //     ここでの teardown も clean であることは前提（壊れていれば後段で露見）。
        for _ in 0..Self::WARMUP_CYCLES {
            let _ = run_one_reload_cycle();
        }

        // (2) ウォームアップ後のベースライン採取（計測サイクルの開始点）。
        let baseline = sample_resources();

        // (3) 計測対象の N サイクルを回す。
        let mut clean_teardowns = 0usize;
        for _ in 0..n {
            if run_one_reload_cycle() {
                clean_teardowns += 1;
            }
        }

        // (4) N サイクル後のリソースカウンタ採取と増分算出。
        let final_sample = sample_resources();
        let leak_metric = LeakMetric::from_samples(baseline, final_sample);

        ReloadReport {
            cycles_run: n,
            clean_teardowns,
            leak_metric,
        }
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

/// 現在プロセスの実 OS リソースカウンタを採取する。
///
/// Windows では `GetProcessHandleCount`（カーネルハンドル）と
/// `GetGuiResources(GR_USEROBJECTS)`（USER オブジェクト＝メッセージ専用
/// ウィンドウを含む）を読む。非 Windows では計測手段がないため `None` を返す。
#[cfg(windows)]
fn sample_resources() -> ResourceSample {
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, GetGuiResources, GetProcessHandleCount, GR_USEROBJECTS,
    };

    // SAFETY: `GetCurrentProcess` は擬似ハンドル（クローズ不要）を返す。
    // `GetProcessHandleCount` には有効な out ポインタを渡す。いずれも純粋な
    // 読み取り API で、プロセス状態を変更しない。
    let process = unsafe { GetCurrentProcess() };

    let kernel_handles = unsafe {
        let mut count: u32 = 0;
        // 戻り値が 0（FALSE）なら失敗。その場合は計測不能として None。
        if GetProcessHandleCount(process, &mut count) != 0 {
            Some(count)
        } else {
            None
        }
    };

    // SAFETY: `GetGuiResources` は読み取り専用。戻り値 0 は「USER オブジェクトが
    // 0 個」と「呼び出し失敗」が区別できないが、長命なテストプロセスでは USER
    // オブジェクトが 0 になることは実質ない。安全側に倒し、0 のときは計測不能扱い
    // （None）として偽の増分を作らない。
    let user_objects = unsafe {
        let n = GetGuiResources(process, GR_USEROBJECTS);
        if n != 0 {
            Some(n)
        } else {
            None
        }
    };

    ResourceSample {
        kernel_handles,
        user_objects,
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
