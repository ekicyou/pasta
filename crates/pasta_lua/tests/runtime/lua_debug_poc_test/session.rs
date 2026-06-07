//! DebugSession 駆動: VM をホスト役スレッドで起動し pause フックを設置、
//! cmd/event チャネルを公開する（R2/R4 共有・トランスポート非依存・
//! スレッドモデル①〜③の実体）。
//!
//! 本モジュールは task 3.1 を実装する。兄弟モジュール
//! `harness_types`（`build_jit_off_vm` / `DebugCommand` / `DebugEvent` /
//! `Breakpoint`）、`pause_gate`（`PauseGate` / `should_pause` /
//! `block_until_command`）、`hook_probe`（`set_global_hook` + `EVERY_LINE` の
//! グローバルフック方式 = R1 実証済み）を再利用する。
//!
//! ## 設計準拠（design.md "スレッドモデル" / "session.rs" / requirements.md R2.1, R2.2）
//! - **① VM スレッドはホスト所有**: 本番では SSP のリクエストスレッド、PoC では
//!   **テストが「ホスト役」で spawn する**。`DebugSession`/フック/`PauseGate` は
//!   自スレッドを spawn・所有しない（外部所有）。
//! - **② 長命トランスポート**: 本タスクはトランスポート非依存。コントローラ
//!   （テスト）が **チャネルを直接** 読み書きする（socket は task 3.2）。
//! - **③ チャネルが唯一の seam**: フック ↔ コントローラは
//!   `Sender<DebugEvent>` / `Receiver<DebugCommand>` のみで接続。`mlua::Lua` は
//!   `!Send` のため**スレッドへ move しない**。closure に move してよいのは
//!   チャネル端点と `Arc<Atomic*>`（進行観測）のみ。
//! - **④ 無期限ブレークが正・timeout はテスト専用**: 停止コア
//!   （`block_until_command` / `await_resume`）は無期限 `recv()`。デッドロック
//!   検出 timeout は**コントローラ（テスト）側 watchdog のみ**に置く。
//!
//! ## 停止の証明（進行観測）
//! VM/ホスト役スレッド上で、ラインフックが**実行行ごと**に共有
//! `Arc<AtomicUsize>` を 1 だけ進める。標的行（ブレークポイント）に達すると
//! `block_until_command` で無期限ブロックするため、**ブロック中はカウンタが
//! 凍結**する。`Continue` 受信後はブロックが解け、標的行より後の行で**カウンタが
//! 再び進む**。コントローラは「停止中の非進行」と「再開後の進行」を観測して
//! R2.1（停止継続）・R2.2（停止地点からの再開）を実証する。

#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use mlua::{HookTriggers, VmState};

use super::harness_types::{Breakpoint, DebugCommand, DebugEvent};
use super::pause_gate::{block_until_command, should_pause, PauseGate};

/// コントローラ（テストドライバ）が握る seam ハンドル。
///
/// `cmd_tx` でフックへコマンドを送り（`Continue`）、`event_rx` でフックからの
/// イベント（`Stopped`）を受ける。`progress` はホスト役スレッド上のフックが
/// 実行行ごとに進める共有カウンタで、停止中は凍結し再開後に進む。
/// `handle` はホスト役スレッドの join ハンドル（テスト終了時に bounded join）。
///
/// `mlua::Lua` は **この構造体に一切含まれない**（VM はホスト役スレッドに閉じる・
/// `!Send` 遵守。スレッドモデル③）。
pub(crate) struct DebugSession {
    /// コントローラ → フック: コマンド送出端（`Continue` / `Inspect`）。
    pub cmd_tx: Sender<DebugCommand>,
    /// フック → コントローラ: イベント受信端（`Stopped` 等）。
    pub event_rx: Receiver<DebugEvent>,
    /// 実行行ごとにフックが進める共有カウンタ（停止中は凍結・再開で進行）。
    pub progress: Arc<AtomicUsize>,
    /// ホスト役（VM）スレッドの join ハンドル。完了時に `Result<(), String>` を返す。
    ///
    /// **`mlua::Result<()>` を直接スレッド境界で返せない**点が `!Send` 制約の実証:
    /// `mlua::Error` は内部に `Arc<dyn Error>`（`!Send`）を持つため
    /// `JoinHandle<mlua::Result<()>>` は `T: Send` を満たせない。よって VM スレッドは
    /// 内部で mlua エラーを `String` へ変換してから返す（スレッドモデル③: チャネル
    /// 端点と Send 値のみが seam を越える）。
    handle: Option<JoinHandle<Result<(), String>>>,
}

impl DebugSession {
    /// 指定ブレークポイント集合でデバッグセッションを開始する。
    ///
    /// **テストが「ホスト役」としてスレッドを spawn**（スレッドモデル①）し、
    /// その**スレッド上で** jit-off VM を構築する（`mlua::Lua` は生成スレッドに
    /// 閉じ、決して move しない・`!Send` 遵守）。VM 上に `set_global_hook`
    /// （`EVERY_LINE`・R1 実証済みのグローバルフック方式）を設置し、コールバックは:
    /// 1. 進行カウンタを 1 進める（**実行行の証拠**）。
    /// 2. `should_pause(&gate, debug)` が真の標的行でのみ `block_until_command`
    ///    で `Stopped` を送出し `Continue` まで無期限ブロックする。
    /// 3. 常に `VmState::Continue` を返す（LuaJIT は `Yield` 不可）。
    ///
    /// 標的行でない行は即 fall-through する。返り値の `DebugSession` で
    /// コントローラがチャネル seam と進行カウンタを観測・制御する。
    pub(crate) fn start(breakpoints: HashSet<Breakpoint>) -> Self {
        // seam: コントローラ ↔ フック（チャネルのみ・スレッドモデル③）。
        let (cmd_tx, cmd_rx) = mpsc::channel::<DebugCommand>();
        let (event_tx, event_rx) = mpsc::channel::<DebugEvent>();

        // 進行観測（停止証明）。フックが実行行ごとに +1 する。
        let progress = Arc::new(AtomicUsize::new(0));
        let hook_progress = Arc::clone(&progress);

        // closure へ move するのは Sender/Receiver と Arc<Atomic*> のみ
        // （`mlua::Lua` は move しない・`!Send` 遵守）。
        // スレッド本体は `mlua::Error`（`!Send`）を境界へ漏らさないよう、内部で
        // `String` へ変換してから返す（クロージャでまとめて map_err）。
        let handle = std::thread::spawn(move || -> Result<(), String> {
            run_host_thread(breakpoints, cmd_rx, event_tx, hook_progress)
                .map_err(|e| e.to_string())
        });

        DebugSession {
            cmd_tx,
            event_rx,
            progress,
            handle: Some(handle),
        }
    }

    /// 進行カウンタの現在値を読む（コントローラ用）。
    pub(crate) fn progress(&self) -> usize {
        self.progress.load(Ordering::SeqCst)
    }

    /// ホスト役スレッドの完了を待つ（コントローラ用）。
    ///
    /// 無期限ブレークが正（スレッドモデル④）のため、**呼び出し側（テスト）が
    /// `Continue` を送ってから** join すること。スレッド本体の結果（mlua エラーは
    /// 既に `String` 化済み）を返す。
    pub(crate) fn join(&mut self) -> std::thread::Result<Result<(), String>> {
        self.handle
            .take()
            .expect("join must be called at most once")
            .join()
    }
}

/// ホスト役（VM）スレッドの本体。**この関数の中だけで `mlua::Lua` を構築・所有**
/// し、スレッド境界を越えさせない（`!Send` 遵守・スレッドモデル①③）。
///
/// 手順:
/// 1. jit-off VM を**このスレッド上で**構築（`build_jit_off_vm`）。
/// 2. `PauseGate`（cmd_rx / event_tx を所有）を構築。
/// 3. `set_global_hook`（`EVERY_LINE`・R1 実証済みのグローバルフック方式）を設置。
///    コールバックは progress を +1 し、標的行で `Stopped` 送出 → `Continue` まで
///    無期限ブロック、常に `VmState::Continue` を返す。
/// 4. 既知シナリオを実行（標的行の後に観測可能な実行行を持つ）。
///
/// `mlua::Result<()>` を返すため `?` を素直に使える。境界を越える際の `String` 変換は
/// 呼び出し元（spawn クロージャ）が `map_err` で行う。
fn run_host_thread(
    breakpoints: HashSet<Breakpoint>,
    cmd_rx: Receiver<DebugCommand>,
    event_tx: Sender<DebugEvent>,
    hook_progress: Arc<AtomicUsize>,
) -> mlua::Result<()> {
    // (1) ホスト役スレッド上で VM を構築（このスレッドに閉じる・move しない）。
    let lua = super::harness_types::build_jit_off_vm()?;

    // (2) PauseGate を VM スレッド側へ構築（cmd_rx/event_tx を所有させる）。
    let gate = PauseGate::new(breakpoints, cmd_rx, event_tx);

    // (3) R1 で実証済みのグローバルフック方式（set_global_hook + EVERY_LINE）。
    lua.set_global_hook(HookTriggers::EVERY_LINE, move |_lua, debug| {
        // 実行行の証拠としてカウンタを進める。
        hook_progress.fetch_add(1, Ordering::SeqCst);
        // 標的行でのみ停止（Stopped 送出 → Continue まで無期限ブロック）。
        if should_pause(&gate, debug) {
            return block_until_command(&gate, debug);
        }
        // 標的外は即継続。
        Ok(VmState::Continue)
    })?;

    // (4) シナリオ実行: 標的行の後に観測可能な追加実行行を持つ既知チャンク。
    // ブロック中は後続行が実行されないため progress が凍結し、Continue 後に
    // 後続行が実行されて progress が進む（R2.1/R2.2 の進行観測の土台）。
    run_breakpoint_scenario(&lua)?;

    lua.remove_global_hook();
    Ok(())
}

/// R2 検証シナリオ用チャンク名（標的ソース）。
pub(crate) const SCENARIO_SOURCE: &str = "@r2_scenario";

/// 標的（ブレークポイント）行番号。`SCENARIO_SOURCE` 内の 1-origin 行。
pub(crate) const BREAKPOINT_LINE: u32 = 3;

/// 停止→再開検証用の既知チャンクを実行する。
///
/// チャンクは標的行（`BREAKPOINT_LINE`）の**後**に観測可能な実行行を複数持つ。
/// ラインフックが行ごとに progress を進めるため、標的行でブロックすると progress
/// が凍結し、`Continue` 後に後続行で progress が進む。これにより「停止地点からの
/// 再開」（R2.2）が進行カウンタで観測できる。
///
/// ソース名は `SCENARIO_SOURCE` を `set_name` で付与し、ブレークポイント
/// `(SCENARIO_SOURCE, BREAKPOINT_LINE)` が一意に当たるようにする。
fn run_breakpoint_scenario(lua: &mlua::Lua) -> mlua::Result<()> {
    // 行番号（1-origin）:
    //   1: local a = 1
    //   2: local b = a + 1
    //   3: local c = b + 1   <- BREAKPOINT_LINE（ここで停止）
    //   4: local d = c + 1   <- 停止中は未実行（progress 凍結）/ 再開後に実行
    //   5: local e = d + 1   <- 再開後に実行（progress 進行）
    //   6: return e
    let chunk = "\
local a = 1
local b = a + 1
local c = b + 1
local d = c + 1
local e = d + 1
return e
";
    lua.load(chunk).set_name(SCENARIO_SOURCE).exec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_debug_poc_test::harness_types::{build_jit_off_vm, ItemOutcome};
    use std::panic::AssertUnwindSafe;
    use std::time::Duration;

    /// コントローラ側 watchdog タイムアウト（**テスト専用**・停止コアには無し）。
    /// 停止コアは無期限 `recv()`（スレッドモデル④）だが、テストが CI を吊らせない
    /// ためにコントローラ側の全待機へ上限を設ける。
    const WATCHDOG: Duration = Duration::from_secs(10);

    /// R2.1（停止継続）・R2.2（停止地点からの再開）:
    /// 標的 (source,line) にブレークポイントを置き、ホスト役スレッドで VM を駆動。
    /// コントローラから:
    /// 1. `Stopped` イベントを watchdog 付き recv で受信（停止到達）。
    /// 2. 停止中は進行カウンタが**ある値で固定**され、短時間待っても**進まない**
    ///    ことを確認（R2.1: 停止継続＝進行フラグが変化しない）。
    /// 3. `Continue` を送出。
    /// 4. ホスト役スレッドを bounded に join（ハングしないことを保証）。
    /// 5. 進行カウンタが**停止時の値を超えて進んだ**ことを確認
    ///    （R2.2: 停止地点から再開し後続行が実行された）。
    ///
    /// `VmState` は PartialEq/Debug 非実装のため判定は `matches!` を用いる
    /// （ここではフック返り値を直接見ないが、停止コアは常に Continue を返す）。
    #[test]
    fn session_stops_at_breakpoint_and_resumes() {
        // 標的ブレークポイント = R2 シナリオの BREAKPOINT_LINE 行。
        let mut bps: HashSet<Breakpoint> = HashSet::new();
        bps.insert((SCENARIO_SOURCE.to_string(), BREAKPOINT_LINE));

        // ホスト役スレッドで VM/フックを起動（テストがホスト役・スレッドモデル①）。
        let mut session = DebugSession::start(bps);

        // (1) Stopped 到達を watchdog 付きで受信。
        let stopped = session
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("must receive a Stopped event before timeout (R2.1)");
        assert_eq!(
            stopped,
            DebugEvent::Stopped {
                source: SCENARIO_SOURCE.to_string(),
                line: BREAKPOINT_LINE,
            },
            "Stopped event must carry the breakpoint (source,line)"
        );

        // (2) 停止中は進行カウンタが凍結する（R2.1: 停止継続）。
        // 停止地点の値を確定し、短い猶予をはさんでも変化しないことを観測する。
        let at_stop = session.progress();
        std::thread::sleep(Duration::from_millis(150));
        let still = session.progress();
        assert_eq!(
            still, at_stop,
            "progress must NOT advance while blocked at the breakpoint (R2.1): \
             {at_stop} -> {still}"
        );

        // (3) 再開シグナル（Continue）を送る（R2.2）。
        session
            .cmd_tx
            .send(DebugCommand::Continue)
            .expect("sending Continue must succeed");

        // (4) ホスト役スレッドを bounded に join（ハング不可）。完了を待つ。
        //     join 自体は無期限だが、Continue 済みのため停止コアは復帰し終了する。
        //     念のため、別スレッドで join し watchdog で監視する。
        let join_result = join_with_watchdog(&mut session, WATCHDOG);
        let thread_result =
            join_result.expect("host-role thread must finish after Continue (no deadlock, R2.2)");
        thread_result
            .expect("host-role thread did not panic")
            .expect("scenario must execute to completion");

        // (5) 進行カウンタが停止時の値を超えて進んだ（R2.2: 停止地点から再開）。
        let after = session.progress();
        assert!(
            after > at_stop,
            "progress must advance past the stop value after Continue (R2.2): \
             at_stop={at_stop}, after={after}"
        );
    }

    /// `DebugSession::join` を **テスト専用 watchdog 付き**で実行する。
    ///
    /// 停止コアは無期限ブロックが正（スレッドモデル④）だが、テストが CI を
    /// 吊らせないよう join を別スレッドへ逃がし、`timeout` 内に終わらなければ
    /// `None`（＝デッドロック疑い）を返す。`Some(_)` はスレッド本体の結果。
    ///
    /// `JoinHandle` を所有権ごと別スレッドへ運ぶため、`DebugSession` から取り出して
    /// 渡す。スレッド結果（`Result<(), String>`・mlua エラーは String 化済みで Send 可）
    /// をチャネルで回収する。
    fn join_with_watchdog(
        session: &mut DebugSession,
        timeout: Duration,
    ) -> Option<std::thread::Result<Result<(), String>>> {
        // 内部ハンドルを奪い、join 専用スレッドへ move する。
        let handle = session
            .handle
            .take()
            .expect("join_with_watchdog must be called at most once");

        let (done_tx, done_rx) = mpsc::channel::<std::thread::Result<Result<(), String>>>();
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });

        done_rx.recv_timeout(timeout).ok()
    }

    /// progress カウンタ自体が「実行行で進む」観測子として機能することの
    /// 補助確認（停止が無い場合は完走して progress が複数行分進む）。
    /// これにより、停止テストの「凍結／再開後の進行」が**進行観測の有無**ではなく
    /// **停止挙動**に起因することを担保する（観測子の妥当性チェック・非空虚）。
    #[test]
    fn progress_advances_when_no_breakpoint() {
        // 当たらないブレークポイント（行 999）でセッションを起動 → 停止せず完走。
        let mut bps: HashSet<Breakpoint> = HashSet::new();
        bps.insert((SCENARIO_SOURCE.to_string(), 999));

        let mut session = DebugSession::start(bps);

        // 停止しないので Stopped は来ない。完走を join で待つ（Continue 不要）。
        let result = session.join().expect("thread must not panic");
        result.expect("scenario must run to completion without a breakpoint");

        // 実行行ぶん progress が進んでいること（観測子が機能している証拠）。
        assert!(
            session.progress() >= BREAKPOINT_LINE as usize,
            "progress must advance across executed lines when no breakpoint hit: got {}",
            session.progress()
        );
    }

    // -----------------------------------------------------------------------
    // (4.1) 異常検出: フック内パニック捕捉（Req 2.4）
    // -----------------------------------------------------------------------

    /// R2.4（フック内ブロッキング中の VM クラッシュ・パニック捕捉）:
    /// フックコールバック内で **Rust パニックを発生**させ、それを捕捉して
    /// `ItemOutcome { passed: false, notes: "panic: ..." }`（NO-GO 根拠）として
    /// **記録**し、ホスト役 VM スレッドが**アボート/ハングせず正常終了**することを
    /// 検証する。
    ///
    /// ## mlua 0.11.6 のフック内パニック挙動（実機ソース＋実機 cargo test 検証済み・本 PoC 知見）
    /// `set_global_hook` のコールバックは mlua の `callback_error_ext`
    /// （`state/util.rs`）が **`std::panic::catch_unwind`** で囲んで呼ぶ。フック内で
    /// パニックすると mlua はそれを `WrappedFailure::Panic` として捕捉し
    /// `lua_error`（LuaJIT 内部の longjmp/例外 unwind）で Lua エラーに変換して伝播する。
    /// エラーが Rust 呼び出し境界（`exec()`）まで戻ると mlua は `WrappedFailure::Panic` を
    /// 検出して **`resume_unwind` で呼び出しスレッド上に再 unwind** する
    /// （`state/raw.rs:845` のエラー pop 経路）。
    ///
    /// すなわち **フック内パニックは VM スレッドを即アボートさせず、`exec()` から
    /// 同一スレッド上に再 unwind する**。本検証は、in-hook をトリガする `exec()` を
    /// **`catch_unwind(AssertUnwindSafe)`** で囲んでこの再 unwind を**捕捉**する。
    /// これが「**実際に機能する機構で事象を捕捉・記録する**」（task 4.1）の採用経路。
    ///
    /// ## 重要な実機知見（payload 非保存・本 PoC 記録）
    /// 再 unwind されたパニックの **payload（`Box<dyn Any + Send>`）は元の
    /// `panic!("msg")` の `&str`/`String` には downcast できない**（実測: 未知の
    /// TypeId。MSVC + LuaJIT の `C-unwind` 例外境界を越える際に payload が保存され
    /// ないため）。したがって**元のパニックメッセージ本文は exec() 境界からは
    /// 復元不能**である。事象の確実な記録には、フック内でパニック直前に**Send 安全な
    /// サイドチャネル（`Arc<Mutex<Option<String>>>`）へ原因を残す**経路を採る。
    /// 捕捉成立（VM スレッドの正常終了＋unwind 検出）とサイドチャネルの原因記録の
    /// 双方で「捕捉＋記録」（R2.4）を満たす。この payload 非保存制約は後続実装仕様
    /// `pasta-vscode-lua-debug` への引き継ぎ事項として記録する。
    ///
    /// ## 設計準拠（design.md "Error Handling": VM 破壊回避 = catch_unwind）
    /// 停止コア（`block_until_command`）は無改変。パニックは**フックを撃つ**
    /// `exec()` の周囲（VM スレッド内）で `catch_unwind` する。`mlua::Lua` /
    /// 生 `lua_State` はスレッド境界を越えず、原因は**文字列のみ**がサイドチャネルと
    /// `ItemOutcome` へ運ばれる（`!Send` 遵守・スレッドモデル③）。
    #[test]
    fn pause_panic_is_captured() {
        const PANIC_SOURCE: &str = "@panic_scenario";
        const PANIC_LINE: u32 = 2;
        const PANIC_MSG: &str = "injected hook panic for R2.4";

        // Send 安全なサイドチャネル: フック内でパニック直前に原因を残す。
        // payload は unwind 境界で保存されない（実機知見）ため、原因の確実な記録は
        // この文字列で行う。
        let cause: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));
        let cause_hook = Arc::clone(&cause);

        // ホスト役スレッド: VM をスレッド内で構築し、標的行でパニックするフックを
        // 設置、in-hook をトリガする exec() を catch_unwind で囲んで結果を bool で
        // 返す（!Send 値はスレッド境界を越えさせない）。
        //
        // 返り値:
        //   Ok(true)   … パニックを捕捉できた（= 期待。記録対象）
        //   Ok(false)  … パニックが unwind しなかった（想定外・テスト失敗）
        //   Err(msg)   … VM 構築/フック設置の失敗（テスト失敗）
        let handle = std::thread::spawn(move || -> Result<bool, String> {
            let lua = build_jit_off_vm().map_err(|e| e.to_string())?;

            lua.set_global_hook(HookTriggers::EVERY_LINE, move |_lua, debug| {
                let line = debug.current_line().unwrap_or(0) as u32;
                let src = debug
                    .source()
                    .source
                    .as_ref()
                    .map(|c| c.as_ref().to_string())
                    .unwrap_or_default();
                if src == PANIC_SOURCE && line == PANIC_LINE {
                    // パニック直前に原因をサイドチャネルへ記録（payload 非保存対策）。
                    if let Ok(mut g) = cause_hook.lock() {
                        *g = Some(format!("{PANIC_MSG} at {src}:{line}"));
                    }
                    // フック内で Rust パニックを注入する（VM クラッシュ相当事象）。
                    panic!("{PANIC_MSG}");
                }
                Ok(VmState::Continue)
            })
            .map_err(|e| e.to_string())?;

            // in-hook（パニック）をトリガする exec() を catch_unwind で囲む。
            // AssertUnwindSafe: `&Lua` は UnwindSafe ではないが、パニック後に VM へ
            // 触れない（即 remove して捨てる）ため健全。
            let caught = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let chunk = "\
local a = 1
local b = a + 1
return b
";
                lua.load(chunk).set_name(PANIC_SOURCE).exec()
            }));

            // パニック後は VM 状態が不定でありうるため、以降 VM へは触れない
            // （フックだけは best-effort で外し、Lua はそのまま drop させる）。
            let _ = lua.remove_global_hook();

            // 捕捉できたか（Err = unwind 捕捉成立）。payload 本文はここでは扱わない
            // （境界で非保存）。原因はサイドチャネルに記録済み。
            Ok(caught.is_err())
        });

        // ホスト役スレッドを **bounded** join（テスト専用 watchdog・ハング不可）。
        // join 自体を別スレッドへ逃がして recv_timeout で監視する。
        let (done_tx, done_rx) = mpsc::channel::<std::thread::Result<Result<bool, String>>>();
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });
        let joined = done_rx
            .recv_timeout(WATCHDOG)
            .expect("host-role thread must finish (no hang) after a hook panic is caught (R2.4)");

        // スレッド本体自身は **パニックせず正常終了**していること
        // （パニックは内側 catch_unwind が吸収したので、スレッドは Ok で戻る）。
        let thread_outcome = joined.expect(
            "host-role thread must terminate gracefully, NOT abort/propagate the panic (R2.4)",
        );
        let panic_was_caught = thread_outcome
            .expect("VM construction / hook install must succeed before the panic injection");
        assert!(
            panic_was_caught,
            "the injected hook panic must be CAPTURED by catch_unwind around exec() (R2.4); \
             instead exec() returned normally"
        );

        // サイドチャネルに原因が記録されていること（payload 非保存のため、これが
        // 原因記録の確実な経路）。
        let recorded_cause = cause
            .lock()
            .unwrap()
            .clone()
            .expect("the panic cause must have been recorded via the side channel before panicking (R2.4)");

        // 捕捉したパニックを NO-GO 根拠の ItemOutcome として記録する（R2.4）。
        let outcome = ItemOutcome {
            id: "R2".to_string(),
            passed: false,
            method: "catch_unwind around in-hook exec() + Send-safe side channel for cause \
                     (mlua re-raises hook panic via resume_unwind; payload not preserved across \
                     LuaJIT C-unwind boundary)"
                .to_string(),
            notes: format!("panic: {recorded_cause}"),
        };

        // 記録内容の検証（非空虚: 注入原因が notes に乗っていること）。
        assert!(
            !outcome.passed,
            "a captured VM panic must be recorded as a failed ItemOutcome (NO-GO evidence, R2.4)"
        );
        assert!(
            outcome.notes.contains(PANIC_MSG),
            "the recorded outcome must carry the panic cause (R2.4): notes = {:?}",
            outcome.notes
        );
        assert!(
            outcome.notes.starts_with("panic:"),
            "panic outcomes must be tagged 'panic:' for NO-GO evidence (R2.4): {:?}",
            outcome.notes
        );

        println!(
            "[R2.4] hook panic CAPTURED and recorded (VM thread terminated gracefully, no hang): {}",
            outcome.notes
        );
    }

    // -----------------------------------------------------------------------
    // (4.1) 異常検出: デッドロック検出（テスト専用 watchdog・Req 2.4）
    // -----------------------------------------------------------------------

    /// R2.4（フック内ブロッキング中のデッドロックを捕捉し NO-GO 根拠として記録）:
    /// ブレークポイントで停止したまま **`Continue` を一切送らない**（ハング／
    /// デッドロックしたセッションのシミュレーション）。コントローラ側の
    /// **テスト専用 watchdog**（`recv_timeout` ＋ bounded join）で「完了しないこと」を
    /// 検出し、`ItemOutcome { passed: false, notes: "timeout/deadlock" }` として記録、
    /// テスト自体は**ハングせずに終了**する。
    ///
    /// ## 設計準拠（design.md "Error Handling" / スレッドモデル ④）
    /// 停止コア（`block_until_command` / `await_resume`）は**無期限 `recv()` のまま**で
    /// あり、timeout は**コントローラ（テスト）側にのみ**置く（停止コアには組み込ま
    /// ない）。検出後は、停止中の VM ホスト役スレッドを **detach（leak）** させて
    /// テストプロセスをクリーンに終わらせる（コアに timeout を焼き込んで強制終了
    /// させない）。`DebugSession::join` は呼ばず、内部ハンドルを drop して切り離す。
    #[test]
    fn deadlock_detected_by_watchdog_without_hang() {
        // 短い watchdog（CI を長く吊らせない）。停止コアは無期限なので、これは
        // テスト側の検出器であって停止コアの挙動ではない。
        const DEADLOCK_WATCHDOG: Duration = Duration::from_millis(500);

        // 標的ブレークポイントで停止 → Continue を**送らない** = 永久ブロック。
        let mut bps: HashSet<Breakpoint> = HashSet::new();
        bps.insert((SCENARIO_SOURCE.to_string(), BREAKPOINT_LINE));

        let mut session = DebugSession::start(bps);

        // (1) 停止到達は通常どおり受信できる（停止コアは生きている）。
        let stopped = session
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("must reach the breakpoint and emit Stopped");
        assert_eq!(
            stopped,
            DebugEvent::Stopped {
                source: SCENARIO_SOURCE.to_string(),
                line: BREAKPOINT_LINE,
            }
        );

        // (2) **Continue を送らない**。テスト専用 watchdog で「完了しないこと」を検出。
        //     内部ハンドルを奪い、join 専用スレッドへ逃がして recv_timeout で監視する。
        //     watchdog 期間内に完了しなければ「timeout/deadlock」と判定する。
        let handle = session
            .handle
            .take()
            .expect("session handle must be present");
        let (done_tx, done_rx) = mpsc::channel::<std::thread::Result<Result<(), String>>>();
        // この watcher スレッド自体は handle.join() で無期限ブロックしうるが、
        // テストはそれを待たず（detach）、recv_timeout で「完了しないこと」を確定する。
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });

        let completed = done_rx.recv_timeout(DEADLOCK_WATCHDOG);

        // (3) デッドロック判定: watchdog 期間内に完了しないこと（= Err(Timeout)）。
        let is_deadlock = matches!(completed, Err(mpsc::RecvTimeoutError::Timeout));
        assert!(
            is_deadlock,
            "session that never receives Continue must NOT complete within the watchdog \
             (simulated deadlock, R2.4); got: {completed:?}"
        );

        // (4) NO-GO 根拠として ItemOutcome を記録する（R2.4）。
        let outcome = ItemOutcome {
            id: "R2".to_string(),
            passed: false,
            method: "controller-side recv_timeout watchdog (stop-core stays unbounded recv(); \
                     timeout lives ONLY in the test controller, model ④)"
                .to_string(),
            notes: "timeout/deadlock".to_string(),
        };
        assert!(!outcome.passed, "a detected deadlock is NO-GO evidence (R2.4)");
        assert_eq!(
            outcome.notes, "timeout/deadlock",
            "deadlock outcomes must be tagged 'timeout/deadlock' (R2.4)"
        );

        // (5) 後始末: 停止中の VM ホスト役スレッドは **detach（leak）** させ、テスト
        //     プロセスをクリーンに終了させる。`session.handle` は既に take 済みなので
        //     Drop で join しない（コアへ timeout を焼き込まないための意図的 detach）。
        //     watcher スレッドも join しない（handle.join() で無期限ブロック中のため）。
        assert!(
            session.handle.is_none(),
            "VM host thread is intentionally detached (no forced timeout baked into the core, model ④)"
        );

        println!(
            "[R2.4] deadlock (no Continue) detected by TEST-ONLY watchdog without hanging; \
             recorded: {} (VM thread detached, stop-core left unbounded)",
            outcome.notes
        );
    }
}
