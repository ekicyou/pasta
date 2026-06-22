//! 本番 teardown（`Stop{done}` ack）と reload リーク検査（task 4.1・
//! R7.1/R7.2/R7.3/R7.4/R7.5）。
//!
//! design.md「Mailbox / Reply / Teardown」コンポーネントおよび「System Flows → reload
//! teardown」の本番化。teardown は **`ActorMsg::Stop { done }` 制御メッセージ**を
//! mailbox（単一 FIFO）へ送ることで成立する。Stop は同一 FIFO を通るため、先行
//! メッセージは Stop の前に drain される（clean drain）。アクター（[`thread`] の
//! `spawn_actor_thread` が回す単一 `recv_async` ループ）は Stop で:
//!
//! 1. 残メッセージを drain 後（FIFO 順序が保証）、
//! 2. VM（`PastaShiori` が内包する `!Send` Lua VM）を drop し、debug backend teardown・
//!    メッセージ専用ウィンドウ破棄を完了させ（block_on 完了時にこのスレッド上で実行）、
//! 3. `done` ack を送ってループを抜ける、
//!
//! という順序で teardown する。SHIORI 側は `done_rx.recv_timeout` で **有界に** ack を
//! 待ち、ack 受信＝全資源解放済みを確認してからスレッドを **detach** する
//! （[`thread::ActorThread::detach`]）。
//!
//! # `JoinHandle`／二重 join を採らない理由（設計補足・R7.4）
//! 完了確認は done ack で行うため、`JoinHandle::join`・`Arc<AtomicBool>` shutdown フラグ・
//! `take()` 二重 join 回避の小細工は **不要**（join 自体を廃止）。これにより AC4 の
//! 「二重 join 回避」は構造的に成立する。スレッドは detach し、block_on 完了後に自然
//! 終了する。
//!
//! # 冪等性（R7.4）
//! done ack を受けた後の二重 teardown は、mailbox の receiver が既に drop されている
//! ため `Sender::send` が `Disconnected` を返す。これを **already-done** として扱い、
//! 安全に no-op を返す（ハングも panic もしない）。本番 FFI 経路（task 5.1）で unload と
//! `DllMain detach` が重なっても二重解放を起こさない構造的根拠。
//!
//! # 異常記録（R7.5）
//! ack が timeout した／途中で異常が生じた場合は [`tracing`] で記録し、
//! [`TeardownReport::anomaly`] に載せる。**panic も `unwrap`/`expect` も使わず**、
//! 異常を呼び出し側へ値で返してホスト（SSP）プロセスを巻き込まない。
//!
//! # gating
//! 本モジュールはアクタースレッド（[`thread`]・wintf `block_on` 依存）と実 OS リソース
//! 計測（`windows-sys/Win32_System_Threading`）を要するため、`actor-poc` feature 有効時
//! のみコンパイルされる（`actor/mod.rs` で `#[cfg(feature = "actor-poc")]`）。FFI 入口
//! （`windows.rs` の `load`/`unload`）への配線は task 5.1。既定出荷ビルドはバイト不変。

use std::path::Path;
use std::time::Duration;

use flume::Sender;

use crate::actor::mailbox::ActorMsg;
use crate::actor::thread::{spawn_actor_thread, ActorThread};

/// teardown の結果レポート（R7.1/R7.4/R7.5）。
///
/// `panic`/`expect` を使わず、teardown の成否・冪等 no-op・異常を **値で** 返す
/// （ホストプロセスを巻き込まないため）。
#[derive(Debug, Clone)]
pub struct TeardownReport {
    /// `done` ack を期限内に受信したか（clean teardown の主証跡）。
    pub acked: bool,
    /// すでに teardown 済みで、今回は安全な no-op だったか（冪等・R7.4）。
    ///
    /// mailbox receiver が drop 済み（=アクター終了済み）で Stop 送信が `Disconnected`
    /// になった場合に `true`。この場合 `acked` は `false`（新たな ack は来ない）。
    pub already_done: bool,
    /// teardown 途中の異常（ack timeout・送信失敗等）。`Some` ならホストを巻き込まずに
    /// 記録した異常内容（R7.5）。clean teardown／冪等 no-op の双方とも `None`。
    pub anomaly: Option<String>,
}

impl TeardownReport {
    /// clean teardown（done ack 受信・異常なし）だったか。
    ///
    /// 冪等 no-op（`already_done`）は「clean」ではないが「安全」ではある。リーク検査の
    /// サイクル成功判定にはこの `is_clean()`（実際に ack を受けて teardown が走った）を
    /// 用いる。
    pub fn is_clean(&self) -> bool {
        self.acked && self.anomaly.is_none()
    }
}

/// アクタースレッドを teardown する（本番経路・design.md「reload teardown」）。
///
/// `tx`（mailbox 送信端）から `ActorMsg::Stop { done }` を送り、`timeout` 付きで done ack
/// を待つ。ack 受信＝アクターが残メッセージ drain・VM 破棄・debug teardown・ウィンドウ
/// 破棄を全て終えた合図なので、その後 `actor` を **detach** する（join しない）。
///
/// - ack 受信: `acked=true`・clean（[`TeardownReport::is_clean`]）。
/// - Stop 送信が `Disconnected`（アクター既終了）: `already_done=true`（冪等 no-op）。
/// - ack timeout / その他異常: `anomaly=Some(..)` を記録（[`tracing::warn`]）し、ホストを
///   巻き込まずに返す（R7.5）。スレッドは detach する（join でブロックしない）。
pub fn teardown_actor(
    tx: &Sender<ActorMsg>,
    actor: ActorThread,
    timeout: Duration,
) -> TeardownReport {
    let report = teardown_via_sender(tx, timeout);
    // done ack を受けたか否かに関わらずスレッドは detach する（join しない=R7.4）。
    // ack 済みなら block_on は戻り済み。timeout 異常時も join でハングさせない。
    actor.detach();
    report
}

/// mailbox 送信端のみで teardown（Stop{done}→ack 待ち）を試行する冪等経路。
///
/// 本番 FFI 経路では `ActorThread` ハンドルを持たずに `static MAILBOX`（task 5.1）の
/// `Sender` 越しに teardown を撃つ場面があり、また二重 teardown（unload と `DllMain
/// detach` の競合）も起こりうる。本関数はそのどちらでも安全に振る舞う:
///
/// - mailbox receiver が drop 済み（アクター既終了）→ `Sender::send` が `Disconnected`
///   → **already-done** として no-op（`already_done=true`・hang/panic なし・R7.4）。
/// - 送信成功 → `done_rx.recv_timeout(timeout)`:
///   - `Ok(())` → `acked=true`（clean）。
///   - `Err(Timeout)` / `Err(Disconnected)` → 異常を記録（[`tracing::warn`]）し
///     `anomaly=Some(..)`（R7.5・ホストを巻き込まない）。
pub fn teardown_via_sender(tx: &Sender<ActorMsg>, timeout: Duration) -> TeardownReport {
    let (stop, done_rx) = ActorMsg::stop();

    // Stop を単一 FIFO へ投入。先行メッセージはこの Stop の前に drain される（clean
    // drain）。receiver drop 済みなら Disconnected = already-done（冪等 no-op）。
    if tx.send(stop).is_err() {
        tracing::debug!(
            "teardown: mailbox receiver already gone; treating as already-done (idempotent no-op)"
        );
        return TeardownReport {
            acked: false,
            already_done: true,
            anomaly: None,
        };
    }

    // done ack を有界に待つ。teardown が壊れて ack を返さない／drain せず break した
    // 場合に無限待機せず、異常として終結させる（テストはハングせず失敗する）。
    match done_rx.recv_timeout(timeout) {
        Ok(()) => TeardownReport {
            acked: true,
            already_done: false,
            anomaly: None,
        },
        Err(flume::RecvTimeoutError::Timeout) => {
            // ack が来ない = drain/cleanup/ack のいずれかが完了していない疑い。
            // 記録するがホスト（SSP）プロセスは巻き込まない（R7.5）。
            let msg = format!("teardown: done ack timed out after {timeout:?}");
            tracing::warn!("{msg}");
            TeardownReport {
                acked: false,
                already_done: false,
                anomaly: Some(msg),
            }
        }
        Err(flume::RecvTimeoutError::Disconnected) => {
            // Stop は送れたが done sender が ack 前に drop された（アクター異常終了等）。
            // teardown の完了確認は取れていないため異常として記録（R7.5）。
            let msg =
                "teardown: done channel disconnected before ack (actor dropped sender)".to_string();
            tracing::warn!("{msg}");
            TeardownReport {
                acked: false,
                already_done: false,
                anomaly: Some(msg),
            }
        }
    }
}

/// reload 反復の結果レポート（R7.2 の re-spawn ＋ R7.3 のリーク不在）。
#[derive(Debug, Clone, Copy)]
pub struct ReloadReport {
    /// 実際に回した reload サイクル数（途中で hang/panic していなければ要求数と一致）。
    pub cycles_run: usize,
    /// うち clean に teardown（done ack 受信）したサイクル数。
    pub clean_teardowns: usize,
    /// 実 OS リソースのリーク指標（Windows のみ `Some`。非 Windows は `None`）。
    pub leak_metric: Option<LeakMetric>,
}

/// reload 反復前後の実 OS リソースカウンタとその増分（R7.3）。
///
/// ベースラインは **ウォームアップ後** に採取する（一回性初期化を除外するため）。
/// 計測アプローチは PoC `pasta_lua::actor_poc::teardown::LeakMetric` の流用
/// （`GetProcessHandleCount` / `GetGuiResources(GR_USEROBJECTS)`）。
#[derive(Debug, Clone, Copy)]
pub struct LeakMetric {
    /// ウォームアップ後・計測サイクル開始時のカーネルハンドル数。
    pub kernel_handles_baseline: u32,
    /// N サイクル完了後のカーネルハンドル数。
    pub kernel_handles_final: u32,
    /// カーネルハンドル数の増分（final - baseline）。
    pub kernel_handle_growth: i64,
    /// ウォームアップ後・計測サイクル開始時の USER オブジェクト数。
    pub user_objects_baseline: u32,
    /// N サイクル完了後の USER オブジェクト数。
    pub user_objects_final: u32,
    /// USER オブジェクト数の増分（final - baseline）。
    pub user_object_growth: i64,
}

/// reload teardown の反復リーク検査ハーネス（design「Reload / Teardown Tests」）。
///
/// spawn→teardown（`Stop{done}` ack）→ 再 spawn のサイクルを反復し、各サイクルで
/// アクタースレッド・VM・メッセージ専用ウィンドウ・チャネルが漏れなく解放される
/// （clean teardown）こと、サイクル間でリソースが累積しない（リーク／枯渇不在）ことを
/// **done ack 後に** 実測する。
pub struct ReloadProbe;

impl ReloadProbe {
    /// ウォームアップに使うサイクル数。最初の数回は実 VM 構築の一回性コストを
    /// 吸収させ、ベースライン採取から除外する。
    const WARMUP_CYCLES: usize = 2;

    /// reload（spawn→teardown）サイクルを `n` 回反復し、各サイクルの teardown 結果と
    /// 累積リーク指標を返す。
    ///
    /// 各サイクルは新規 `spawn_actor_thread`（新スレッド＋新 VM＋新チャネル）→
    /// `teardown_actor`（`Stop{done}` ack→detach）。ack を受けてから次サイクルを spawn
    /// するため、計測は **done ack 後** に行われる（解放完了後の実測）。
    ///
    /// - Postconditions: 全サイクルで done ack（clean teardown）・再 spawn 正常稼働。
    /// - Invariants: スレッドハンドル／USER オブジェクト／port がサイクル間で枯渇しない。
    pub fn run_cycles(load_dir: &Path, n: usize, timeout: Duration) -> ReloadReport {
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

/// reload を 1 回（spawn→teardown）回し、teardown が clean（done ack 受信）だったかを返す。
fn run_one_reload_cycle(load_dir: &Path, timeout: Duration) -> bool {
    let (tx, rx) = crate::actor::mailbox::mailbox();
    let actor = spawn_actor_thread(0, load_dir.to_path_buf(), rx);
    let report = teardown_actor(&tx, actor, timeout);
    report.is_clean()
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
/// （USER オブジェクト＝メッセージ専用ウィンドウを含む）を読む。非 Windows では
/// 計測手段がないため `None`。
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
        if GetProcessHandleCount(process, &mut count) != 0 {
            Some(count)
        } else {
            None
        }
    };

    // SAFETY: `GetGuiResources` は読み取り専用。戻り値 0 は「USER オブジェクト 0 個」と
    // 「呼び出し失敗」が区別できないが、長命なテストプロセスでは 0 は実質ない。安全側に
    // 倒し、0 のときは計測不能扱い（None）として偽の増分を作らない。
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
