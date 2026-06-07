//! `DebugSession`: the protocol-INDEPENDENT stop state machine (design
//! "DebugSession（停止状態機械・protocol 非依存）", requirements 1.2 / 1.6 / 3.4).
//!
//! # Role in the backend
//!
//! [`DebugSession`] is the stop core that sits on the VM thread behind the line
//! hook. Per executed line the installed hook (see [`crate::debug::hook`]) calls
//! [`DebugSession::on_line`] (via the [`LineHook`] trait). When the current
//! `(source, line)` matches a registered breakpoint the session enters its STOP
//! LOOP: it emits a [`SessionEvent::Stopped`] and blocks on the command channel
//! until the controller resumes it.
//!
//! It is DAP-agnostic: it speaks only [`SessionCommand`] / [`SessionEvent`] over
//! `std::sync::mpsc`, never DAP types and never `mlua::Lua`/`mlua::Error` across
//! the channel (design "Invariants"; `mlua::Error` is `!Send`, so any internal
//! error is stringified into [`SessionEvent::Error`] before it could cross).
//!
//! # Channel seam (design "スレッドモデル ③")
//!
//! The session owns the VM-thread ends of two channels:
//! - `cmd_rx: Receiver<SessionCommand>` — controller → session (e.g. `Continue`).
//! - `event_tx: Sender<SessionEvent>` — session → controller (e.g. `Stopped`).
//!
//! Plus a shared [`BreakpointSet`] (an `Arc<Mutex<…>>` clone) so breakpoints set
//! on the controller side are observed live by the hook side.
//!
//! # Unbounded stop core (design "無期限ブロックが正" / スレッドモデル ④)
//!
//! The STOP LOOP blocks on `cmd_rx.recv()` **indefinitely** — there is NO
//! watchdog / timeout in the core; an indefinite break is the correct behaviour.
//! Any timeout lives ONLY in test controllers. The loop and `on_line` ALWAYS
//! return `Ok(VmState::Continue)` because LuaJIT cannot Yield from a hook.
//!
//! # Session lifecycle (design "セッションライフサイクル")
//!
//! `DebugSession` / `BreakpointSet` are owned at the **runtime scope** so they
//! survive across many short Lua executions (SHIORI requests); the hook is
//! installed once at VM init. This type therefore bakes in NO per-execution
//! lifetime assumptions — `on_line` is a `&self` method that can be called any
//! number of times across many chunk executions.
//!
//! # Implemented scope (tasks 2.2 + 2.5)
//!
//! Both the `Running` / stop-at-breakpoint / continue path AND `Stepping`
//! (over / into / out) are implemented. The [`RunMode::Stepping`] variant and
//! [`StepKind`] carry the coroutine identity and captured stack depth so the
//! per-line completion decision (see [`DebugSession::step_should_stop`])
//! resolves over/into/out by comparing the current thread + depth against the
//! values captured at the stop point. The step key survives a coroutine
//! `yield`→`resume` because the `(thread, base_depth)` pair stays valid across
//! the suspension. The stop core itself remains UNBOUNDED — it blocks on
//! `cmd_rx.recv()` with no watchdog / timeout. Inspect commands
//! (`StackTrace`/`Variables`/`Scopes`/`Threads`) are handled IN the stop loop
//! ON THIS VM THREAD (task 4.1): they capture the stack / variables of the
//! running coroutine via `current_thread()` and emit the result event, then
//! keep blocking (they do not resume execution). `mlua::Lua` never crosses a
//! thread (it is `!Send`).

#![allow(dead_code)]

use std::sync::mpsc::{Receiver, Sender};

use mlua::{Debug, Lua, VmState};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::hook::LineHook;
use crate::debug::inspect::{capture_stack, capture_variables};
use crate::debug::types::{
    Scope, SessionCommand, SessionEvent, StopReason, ThreadId, ThreadInfo,
};

/// The DAP thread id reported for the (single) main execution thread.
///
/// task 2.2 derives a fixed id; richer per-coroutine thread ids are owned by the
/// inspect / threads work (2.3+ / 4.1).
const MAIN_THREAD_ID: u32 = 1;

/// Which kind of step is in progress while [`RunMode::Stepping`].
///
/// Defined here (session.rs owns the state machine) so task 2.5's StepController
/// can implement the over/into/out depth comparison without restructuring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepKind {
    /// Step over the current line's calls (R1.3).
    Over,
    /// Step into the next call (R1.4).
    In,
    /// Step out to the caller (R1.5).
    Out,
}

/// The session's run mode (design "DebugSession 状態機械").
///
/// `Stepping` keeps the coroutine identity (`thread`) and the captured stack
/// depth (`base_depth`) so the StepController can decide over/into/out by
/// comparing the current thread+depth against these (task 2.5). It also records
/// the `start_line` so step-over can detect "line changed in the same frame"
/// (a same-frame statement that spans the call's own line must not re-trigger).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunMode {
    /// Run until a breakpoint is hit (or termination).
    Running,
    /// A step (over/into/out) is in progress, keyed by coroutine identity and
    /// the captured base stack depth (design "DebugSession 状態機械"):
    /// the per-line decision (see [`DebugSession::step_should_stop`]) compares
    /// the CURRENT thread+depth+line against these to decide over/into/out.
    Stepping {
        /// Which step kind is in progress.
        kind: StepKind,
        /// Identity of the coroutine the step is tracking
        /// (`current_thread().state()` address as a `usize`). The step only
        /// considers lines on THIS thread; lines on the host loop or another
        /// coroutine are skipped (design "thread 不一致 ... はスキップして継続").
        /// Because a coroutine's own stack is preserved across `yield`, this
        /// key remains valid across `yield`→`resume` (採択B survival).
        thread: crate::debug::types::ThreadId,
        /// Lua call depth captured when the step began (`capture_stack().len()`).
        base_depth: u32,
        /// Source line the step began on, for step-over "line changed"
        /// detection within the same frame (`depth == base_depth`).
        start_line: u32,
    },
}

/// The protocol-independent stop state machine (design "DebugSession").
///
/// Holds the shared [`BreakpointSet`], the VM-thread channel ends, and the
/// current [`RunMode`]. The installed line hook calls [`on_line`](Self::on_line)
/// per executed line; on a breakpoint hit the session emits
/// [`SessionEvent::Stopped`] and blocks until the controller resumes it.
///
/// `RunMode` is interior-mutable via a `Cell` (rather than `&mut self`) because
/// the [`LineHook`] trait hands the hook a `&self`; `DebugSession` is owned on a
/// single (VM) thread, so a `Cell` is sufficient and keeps the value mutable
/// across the many `&self` line-hook calls without a lock.
pub(crate) struct DebugSession {
    /// Shared breakpoint store (an `Arc<Mutex<…>>` clone; live-updated by the
    /// controller side, read here on the hook side).
    breakpoints: BreakpointSet,
    /// Controller → session command end (e.g. `Continue`). Blocked on in the
    /// stop loop with an UNBOUNDED `recv()` (no watchdog in the core).
    cmd_rx: Receiver<SessionCommand>,
    /// Session → controller event end (e.g. `Stopped` / `Terminated` / `Error`).
    event_tx: Sender<SessionEvent>,
    /// Current run mode. `Cell` because the line hook calls `&self`; the session
    /// is single-threaded (VM thread), so interior mutability is race-free here.
    mode: std::cell::Cell<RunMode>,
}

impl DebugSession {
    /// Construct a session over a shared breakpoint store and the VM-thread
    /// channel ends. Starts in [`RunMode::Running`].
    ///
    /// The session is meant to be owned at runtime scope (design
    /// "セッションライフサイクル") and survive across many chunk executions; it
    /// holds no per-execution state.
    pub(crate) fn new(
        breakpoints: BreakpointSet,
        cmd_rx: Receiver<SessionCommand>,
        event_tx: Sender<SessionEvent>,
    ) -> Self {
        Self {
            breakpoints,
            cmd_rx,
            event_tx,
            mode: std::cell::Cell::new(RunMode::Running),
        }
    }

    /// The current run mode (controller-side / test observation helper).
    pub(crate) fn mode(&self) -> RunMode {
        self.mode.get()
    }

    /// Extract `(source, line)` from a hook [`mlua::Debug`] frame.
    ///
    /// Prefers the chunk `source()` name; falls back to `short_src`; uses
    /// `current_line()` for the line (0 when unavailable). This matches the PoC
    /// extraction convention (`pause_gate::source_and_line` / `hook` tests).
    fn source_and_line(debug: &Debug) -> (String, u32) {
        let src = debug.source();
        let source = src
            .source
            .as_ref()
            .map(|c| c.as_ref().to_string())
            .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
            .unwrap_or_default();
        let line = debug.current_line().unwrap_or(0) as u32;
        (source, line)
    }

    /// The running coroutine's identity and current Lua call depth, observed
    /// from inside the hook (design "DebugSession 状態機械").
    ///
    /// `lua.current_thread()` inside the hook resolves to the RUNNING coroutine
    /// (proven by task 2.4); its `.state()` pointer is a STABLE [`ThreadId`]
    /// across `yield`/`resume`. The depth is the number of Lua frames on that
    /// thread, reused from [`capture_stack`] (no extra FFI helper — keeps the
    /// StepController boundary inside `session.rs`).
    fn current_thread_and_depth(lua: &Lua) -> (ThreadId, u32) {
        let thread = lua.current_thread();
        let tid = ThreadId::from_state(thread.state());
        let depth = capture_stack(lua, &thread).len() as u32;
        (tid, depth)
    }

    /// The step-stop decision for [`RunMode::Stepping`] (design
    /// "DebugSession 状態機械（StepController）").
    ///
    /// Returns `true` IFF the current line is the step's completion point:
    ///
    /// - **thread mismatch** (the current thread is NOT `step.thread` — i.e. the
    ///   host loop or ANOTHER coroutine): SKIP (`false`). The step only fires on
    ///   its own thread; because a coroutine's stack is preserved across `yield`,
    ///   the `(thread, base_depth)` key stays valid across `yield`→`resume`
    ///   (採択B), so the step completes at the right line after the resume.
    /// - **Over**: stop when `depth < base_depth` (returned to the caller) OR
    ///   (`depth == base_depth` AND the line changed from `start_line`) — the
    ///   next line in the same frame or shallower; lines in DEEPER frames (the
    ///   called function's body) are skipped.
    /// - **In**: stop at the next stoppable line on this thread — a deeper frame
    ///   (entered a callee) OR a changed line in the same/shallower frame.
    /// - **Out**: stop when `depth < base_depth` (returned to the caller frame).
    ///
    /// `cur_thread` / `depth` are the values from
    /// [`current_thread_and_depth`](Self::current_thread_and_depth); `line` is
    /// the current source line.
    fn step_should_stop(
        kind: StepKind,
        thread: ThreadId,
        base_depth: u32,
        start_line: u32,
        cur_thread: ThreadId,
        depth: u32,
        line: u32,
    ) -> bool {
        // thread 不一致（ホストループ・別コルーチン）の行はスキップして継続。
        if cur_thread != thread {
            return false;
        }
        match kind {
            StepKind::Over => {
                depth < base_depth || (depth == base_depth && line != start_line)
            }
            StepKind::In => depth > base_depth || line != start_line,
            StepKind::Out => depth < base_depth,
        }
    }

    /// The STOP LOOP: emit `Stopped(reason, thread_id)` then block until the
    /// controller resumes / steps / disconnects.
    ///
    /// Promotes the PoC `pause_gate::await_resume`. Emits
    /// [`SessionEvent::Stopped`] with the given `reason` / `thread_id`, then
    /// blocks on an **UNBOUNDED** `cmd_rx.recv()` — NO watchdog/timeout in the
    /// core (design "無期限ブロックが正"). Command handling:
    ///
    /// - [`SessionCommand::Continue`] → resume in `Running`: clear any stepping
    ///   mode and return `Ok(VmState::Continue)` (R1.6).
    /// - [`SessionCommand::Next`] / [`StepIn`](SessionCommand::StepIn) /
    ///   [`StepOut`](SessionCommand::StepOut) → ENTER stepping: capture the
    ///   current `(thread, base_depth, start_line)` from `lua`/`debug` at THIS
    ///   stop point, set [`RunMode::Stepping`], and resume (return `Continue`) so
    ///   the VM runs until the step target line is reached (R1.3 / R1.4 / R1.5).
    /// - [`SessionCommand::Disconnect`] → emit [`SessionEvent::Terminated`] and
    ///   return `Ok(VmState::Continue)` so the VM is released (never hang the
    ///   host when the client disconnects; design "Error Handling").
    /// - [`SessionCommand::SetBreakpoints`] → applied live to the shared set and
    ///   keep blocking.
    /// - Inspect commands (`StackTrace` / `Scopes` / `Variables` / `Threads`) →
    ///   run their FFI ON THIS VM THREAD (capture stack / variables on
    ///   `current_thread()`), emit the result event, and KEEP BLOCKING (they do
    ///   not resume execution). The `mlua::Lua` never crosses a thread (!Send).
    /// - Channel disconnect (`recv()` Err) → return `Ok(VmState::Continue)`: a
    ///   safe fallback so the VM is never hung when the controller is gone.
    ///
    /// Always returns `Ok(VmState::Continue)` (LuaJIT cannot Yield from a hook).
    fn stop_loop(
        &self,
        lua: &Lua,
        debug: &Debug,
        reason: StopReason,
        thread_id: u32,
    ) -> mlua::Result<VmState> {
        // Notify the controller of the stop. A missing receiver must not abort
        // the stop, so a send failure is ignored.
        let _ = self.event_tx.send(SessionEvent::Stopped { reason, thread_id });

        loop {
            match self.cmd_rx.recv() {
                // Resume (R1.6): leave stepping mode entirely.
                Ok(SessionCommand::Continue) => {
                    self.mode.set(RunMode::Running);
                    return Ok(VmState::Continue);
                }

                // Enter a step (over/into/out): capture the stepping context at
                // THIS stop point and resume so the VM runs to the step target.
                Ok(cmd @ (SessionCommand::Next | SessionCommand::StepIn | SessionCommand::StepOut)) => {
                    let kind = match cmd {
                        SessionCommand::Next => StepKind::Over,
                        SessionCommand::StepIn => StepKind::In,
                        _ => StepKind::Out,
                    };
                    let (thread, base_depth) = Self::current_thread_and_depth(lua);
                    let (_source, start_line) = Self::source_and_line(debug);
                    self.mode.set(RunMode::Stepping {
                        kind,
                        thread,
                        base_depth,
                        start_line,
                    });
                    return Ok(VmState::Continue);
                }

                // Tear down: end the session but release the VM so the host is
                // not hung (design "Error Handling").
                Ok(SessionCommand::Disconnect) => {
                    let _ = self.event_tx.send(SessionEvent::Terminated);
                    return Ok(VmState::Continue);
                }

                // `setBreakpoints` is the one command valid during execution
                // (design "System Flows": `Arc<Mutex>` 共有). Apply it live to
                // the shared set and keep blocking. Full reply routing is task
                // 4.1; here we only mutate the shared store so a just-set
                // breakpoint is observed when the session resumes.
                Ok(SessionCommand::SetBreakpoints { source, lines }) => {
                    let _ = self.breakpoints.set_breakpoints(&source, &lines);
                    continue;
                }

                // Inspect commands (task 4.1 full wiring) — run ON THE VM THREAD
                // here in the stop loop, where the FFI is safe (`mlua::Lua` is
                // !Send and never crosses a thread). Each emits its result event
                // and KEEPS BLOCKING: inspect/stack/scopes/threads do not resume
                // execution, so the session stays stopped until continue / step /
                // disconnect.

                // StackTrace: capture the running coroutine's call stack from
                // `current_thread()` and emit it (R2.1 / R3.3).
                Ok(SessionCommand::StackTrace) => {
                    let frames = capture_stack(lua, &lua.current_thread());
                    let _ = self.event_tx.send(SessionEvent::Stack(frames));
                    continue;
                }

                // Variables: decode the variablesReference to a frame level
                // (DapAdapter numbering `var_ref = frame_level + 1`), capture
                // that frame's locals + upvalues, and emit them (R2.2 / R3.3).
                Ok(SessionCommand::Variables { var_ref }) => {
                    let frame_level = var_ref.saturating_sub(1);
                    let vars = capture_variables(lua, &lua.current_thread(), frame_level);
                    let _ = self.event_tx.send(SessionEvent::Variables(vars));
                    continue;
                }

                // Scopes: a minimal single `Locals` scope for the requested
                // frame (variablesReference = frame_id + 1, matching the
                // DapAdapter numbering). The DapAdapter self-answers `scopes` at
                // decode time and treats this event as a wire no-op, so emitting
                // it keeps the session contract consistent WITHOUT producing a
                // second response to the client (no double-response).
                Ok(SessionCommand::Scopes { frame_id }) => {
                    let scopes = vec![Scope {
                        name: "Locals".to_string(),
                        variables_reference: frame_id + 1,
                    }];
                    let _ = self.event_tx.send(SessionEvent::Scopes(scopes));
                    continue;
                }

                // Threads: a single fixed main thread descriptor. Per-coroutine
                // thread enumeration is out of scope here; the DAP client only
                // needs one thread to drive stackTrace / variables (R3.3).
                Ok(SessionCommand::Threads) => {
                    let threads = vec![ThreadInfo {
                        id: MAIN_THREAD_ID,
                        name: "main".to_string(),
                    }];
                    let _ = self.event_tx.send(SessionEvent::Threads(threads));
                    continue;
                }

                // Controller gone: never hang the VM — resume.
                Err(_) => return Ok(VmState::Continue),
            }
        }
    }

    /// Per-line entry point called by the installed hook (the [`LineHook`] seam).
    ///
    /// Decides whether to stop at the current line and, on a stop, enters the
    /// [`stop_loop`](Self::stop_loop). The decision is breakpoint-first so a
    /// breakpoint elsewhere ALWAYS stops (with reason [`StopReason::Breakpoint`])
    /// even while stepping — stepping never masks a breakpoint. Otherwise, in
    /// [`RunMode::Stepping`], the StepController decision
    /// ([`step_should_stop`](Self::step_should_stop)) decides whether this line
    /// is the step's completion point (reason [`StopReason::Step`]).
    ///
    /// Always returns `Ok(VmState::Continue)` (LuaJIT cannot Yield from a hook);
    /// non-target lines fall through immediately.
    ///
    /// `mlua::Error` is `!Send`; should any internal step produce one to report,
    /// it is stringified into [`SessionEvent::Error`] rather than crossing the
    /// boundary as a raw error (helper [`report_error`](Self::report_error)).
    fn on_line_impl(&self, lua: &Lua, debug: &Debug) -> mlua::Result<VmState> {
        let (source, line) = Self::source_and_line(debug);

        // Breakpoint-first: a breakpoint ALWAYS stops (reason Breakpoint), even
        // while Stepping — stepping must not mask breakpoints.
        if self.breakpoints.should_pause(&source, line) {
            return self.stop_loop(lua, debug, StopReason::Breakpoint, MAIN_THREAD_ID);
        }

        // Otherwise, while stepping, evaluate the StepController completion.
        if let RunMode::Stepping {
            kind,
            thread,
            base_depth,
            start_line,
        } = self.mode.get()
        {
            let (cur_thread, depth) = Self::current_thread_and_depth(lua);
            if Self::step_should_stop(
                kind, thread, base_depth, start_line, cur_thread, depth, line,
            ) {
                return self.stop_loop(lua, debug, StopReason::Step, MAIN_THREAD_ID);
            }
        }

        Ok(VmState::Continue)
    }

    /// Stringify an `mlua::Error` (`!Send`) into a [`SessionEvent::Error`] and
    /// send it to the controller, so a VM/FFI failure can cross the channel
    /// boundary (design "Error Handling"; never cross with a raw `mlua::Error`).
    ///
    /// Provided for inspect/step work (2.3+); the task 2.2 stop path is
    /// infallible, but routing exists so later tasks reuse the same seam.
    fn report_error(&self, err: &mlua::Error) {
        let _ = self.event_tx.send(SessionEvent::Error(err.to_string()));
    }
}

/// Plug the session into the hook seam: the installed line hook calls
/// `on_line(lua, &debug)` per executed line (design intent
/// `cb: move |lua, debug| { session.on_line(lua, &debug) }`).
impl LineHook for DebugSession {
    fn on_line(&self, lua: &Lua, debug: &Debug) -> mlua::Result<VmState> {
        self.on_line_impl(lua, debug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;
    use std::time::Duration;

    use mlua::{Lua, LuaOptions, StdLib};

    use crate::debug::breakpoints::BreakpointSet;
    use crate::debug::types::{SourceRef, StopReason};

    /// Controller-side watchdog (TEST-ONLY). The stop core stays UNBOUNDED;
    /// this only keeps CI from hanging on a regression.
    const WATCHDOG: Duration = Duration::from_secs(10);

    /// Source name and breakpoint line for the stop/continue scenario.
    const SCENARIO_SOURCE: &str = "@session_scenario";
    const BREAKPOINT_LINE: u32 = 3;

    /// Build an ALL_SAFE VM (so `jit` exists and `debug` is excluded). The hook
    /// `install` applies `jit.off()` itself. Promotes the PoC / hook-test VM.
    fn build_all_safe_vm() -> Lua {
        unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) }
    }

    /// A controller's view of a running host-thread session (test driver).
    ///
    /// `mlua::Lua` is `!Send` and is NEVER held here — it lives entirely inside
    /// the host thread. Only channel ends, the shared `Arc` progress counter, and
    /// the join handle (returning a `Send` `Result<(), String>`) cross.
    struct HostThread {
        cmd_tx: Sender<SessionCommand>,
        event_rx: Receiver<SessionEvent>,
        /// Incremented once per executed line by a recording hook wrapper, so the
        /// controller can observe progress freeze (while stopped) and resume.
        progress: Arc<AtomicUsize>,
        handle: Option<JoinHandle<Result<(), String>>>,
    }

    impl HostThread {
        fn progress(&self) -> usize {
            self.progress.load(Ordering::SeqCst)
        }
    }

    /// Start a host-thread session: on a separate thread build an ALL_SAFE
    /// jit-off VM, install the hook with a `DebugSession` as the `LineHook`, and
    /// run a known multi-line chunk with a breakpoint on a middle line.
    ///
    /// Mirrors the PoC `DebugSession::start`: the test is the "host role" that
    /// spawns the thread; the VM is constructed INSIDE it (`!Send`). The progress
    /// counter is bumped per line by wrapping the session in a recording closure
    /// that also delegates to `DebugSession::on_line` (so the stop behaviour is
    /// the session's, while progress observation is the test's).
    fn start_session(breakpoints: BreakpointSet) -> HostThread {
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

        let progress = Arc::new(AtomicUsize::new(0));
        let hook_progress = Arc::clone(&progress);

        // Only channel ends + Arc cross the boundary; mlua::Lua never does.
        let handle = std::thread::spawn(move || -> Result<(), String> {
            run_host_thread(breakpoints, cmd_rx, event_tx, hook_progress)
                .map_err(|e| e.to_string())
        });

        HostThread {
            cmd_tx,
            event_rx,
            progress,
            handle: Some(handle),
        }
    }

    /// Host-thread body: build the VM here (never crosses the boundary), install
    /// the hook wired to a `DebugSession`, and run the scenario chunk.
    fn run_host_thread(
        breakpoints: BreakpointSet,
        cmd_rx: Receiver<SessionCommand>,
        event_tx: Sender<SessionEvent>,
        hook_progress: Arc<AtomicUsize>,
    ) -> mlua::Result<()> {
        let lua = build_all_safe_vm();

        // The session is the real stop core under test.
        let session = DebugSession::new(breakpoints, cmd_rx, event_tx);

        // Wrap the session so each line also bumps progress (test observation),
        // then delegate the stop decision to the real `DebugSession::on_line`.
        let handler = move |lua: &Lua, debug: &Debug| {
            hook_progress.fetch_add(1, Ordering::SeqCst);
            session.on_line(lua, debug)
        };

        crate::debug::hook::install(&lua, handler)?;

        // Scenario (1-origin lines):
        //   1: local a = 1
        //   2: local b = a + 1
        //   3: local c = b + 1   <- BREAKPOINT_LINE (stop here)
        //   4: local d = c + 1   <- not executed while stopped (progress frozen)
        //   5: local e = d + 1   <- executed after Continue (progress advances)
        //   6: return e
        let chunk = "\
local a = 1
local b = a + 1
local c = b + 1
local d = c + 1
local e = d + 1
return e
";
        lua.load(chunk).set_name(SCENARIO_SOURCE).exec()?;
        lua.remove_global_hook();
        Ok(())
    }

    /// Bounded join (TEST-ONLY watchdog): a regression that hangs the VM thread
    /// is a test failure, not a suite-killer. The stop core stays unbounded.
    fn join_with_watchdog(
        host: &mut HostThread,
        timeout: Duration,
    ) -> Option<std::thread::Result<Result<(), String>>> {
        let handle = host.handle.take().expect("join at most once");
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });
        done_rx.recv_timeout(timeout).ok()
    }

    /// R1.2 / R1.6 / R3.4 (the observable "done" for task 2.2):
    /// a breakpoint on a middle line stops execution (R1.2), the stop is genuine
    /// (progress freezes while blocked, R1.2 continuity), `Continue` resumes
    /// (R1.6), and a `Stopped` event reaches the controller (R3.4).
    #[test]
    fn session_stops_at_breakpoint_and_resumes() {
        // Breakpoint on the middle line of the scenario.
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);

        let mut host = start_session(breakpoints);

        // (1) Receive the Stopped event (R3.4). TEST-ONLY watchdog so CI can't
        // hang; the stop core itself blocks unbounded.
        let stopped = host
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("must receive a Stopped event before the watchdog (R3.4)");
        assert_eq!(
            stopped,
            SessionEvent::Stopped {
                reason: StopReason::Breakpoint,
                thread_id: MAIN_THREAD_ID,
            },
            "Stopped must report Breakpoint on the main thread id"
        );

        // (2) Execution is genuinely paused: progress freezes while stopped
        // (R1.2 — the stop holds). Capture the value, wait, confirm no advance.
        let at_stop = host.progress();
        std::thread::sleep(Duration::from_millis(150));
        let still = host.progress();
        assert_eq!(
            still, at_stop,
            "progress must NOT advance while blocked at the breakpoint (R1.2): \
             {at_stop} -> {still}"
        );

        // (3) Send Continue (R1.6).
        host.cmd_tx
            .send(SessionCommand::Continue)
            .expect("sending Continue must succeed");

        // (4) The host thread runs to completion after Continue (R1.6), bounded.
        let joined =
            join_with_watchdog(&mut host, WATCHDOG).expect("host thread must finish after Continue (R1.6)");
        joined
            .expect("host thread must not panic")
            .expect("scenario must run to completion after Continue");

        // (5) Progress advanced past the stop value — execution resumed past the
        // breakpoint (R1.6).
        let after = host.progress();
        assert!(
            after > at_stop,
            "progress must advance past the stop value after Continue (R1.6): \
             at_stop={at_stop}, after={after}"
        );
    }

    /// Validity of the progress observer (non-vacuous): with NO breakpoint the
    /// scenario runs to completion without any Continue and progress advances
    /// across executed lines. This proves the freeze in the stop test is due to
    /// the STOP behaviour, not a dead observer.
    #[test]
    fn progress_advances_when_no_breakpoint() {
        // A breakpoint that never matches (line 999).
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[999]);

        let mut host = start_session(breakpoints);

        // No stop → no Continue needed; join directly (bounded).
        let joined = join_with_watchdog(&mut host, WATCHDOG)
            .expect("host thread must finish without a breakpoint");
        joined
            .expect("host thread must not panic")
            .expect("scenario must run to completion with no breakpoint");

        assert!(
            host.progress() >= BREAKPOINT_LINE as usize,
            "progress must advance across executed lines when no breakpoint hits: got {}",
            host.progress()
        );
    }

    /// R3.4 + design "Error Handling": `Disconnect` while stopped tears the
    /// session down (emits `Terminated`) and releases the VM so the host thread
    /// completes — it must NOT hang.
    #[test]
    fn disconnect_while_stopped_terminates_and_releases_vm() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);

        let mut host = start_session(breakpoints);

        // Reach the breakpoint.
        let stopped = host
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("must reach the breakpoint and emit Stopped");
        assert_eq!(
            stopped,
            SessionEvent::Stopped {
                reason: StopReason::Breakpoint,
                thread_id: MAIN_THREAD_ID,
            }
        );

        // Disconnect (no Continue): the session must emit Terminated and resume
        // the VM so the host thread finishes (never hang the host).
        host.cmd_tx
            .send(SessionCommand::Disconnect)
            .expect("sending Disconnect must succeed");

        let terminated = host
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("Disconnect must emit a Terminated event");
        assert_eq!(
            terminated,
            SessionEvent::Terminated,
            "Disconnect while stopped must emit Terminated"
        );

        // The host thread must run to completion (VM released).
        let joined = join_with_watchdog(&mut host, WATCHDOG)
            .expect("host thread must finish after Disconnect (VM released, no hang)");
        joined
            .expect("host thread must not panic")
            .expect("scenario must run to completion after Disconnect");
    }

    /// Forward-compatibility: a non-resuming command (e.g. `StackTrace`, owned by
    /// task 2.3) must NOT release the stop loop; the session keeps blocking until
    /// `Continue`. Verifies the loop is future-proof without panicking/erroring.
    #[test]
    fn non_resuming_command_keeps_blocking_until_continue() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);

        let mut host = start_session(breakpoints);

        // Reach the breakpoint.
        host.event_rx
            .recv_timeout(WATCHDOG)
            .expect("must reach the breakpoint");

        let at_stop = host.progress();

        // Send a non-resuming command; the loop must keep blocking.
        host.cmd_tx
            .send(SessionCommand::StackTrace)
            .expect("sending StackTrace must succeed");

        // Give the session time to (incorrectly) resume if it were going to.
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(
            host.progress(),
            at_stop,
            "a non-resuming command (StackTrace) must NOT release the stop loop"
        );

        // Now Continue actually resumes.
        host.cmd_tx
            .send(SessionCommand::Continue)
            .expect("sending Continue must succeed");
        let joined = join_with_watchdog(&mut host, WATCHDOG)
            .expect("host thread must finish after Continue");
        joined
            .expect("host thread must not panic")
            .expect("scenario must complete after Continue");
        assert!(
            host.progress() > at_stop,
            "progress must advance after Continue following a non-resuming command"
        );
    }

    /// Unit-level proof of the no-watchdog stop core: when the controller never
    /// sends a resume, the host thread does NOT complete within a TEST-ONLY
    /// watchdog (the stop core blocks unbounded — design "無期限ブロックが正").
    /// The thread is then detached (no forced timeout baked into the core).
    #[test]
    fn stop_core_is_unbounded_without_continue() {
        const DEADLOCK_WATCHDOG: Duration = Duration::from_millis(500);

        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);

        let mut host = start_session(breakpoints);

        host.event_rx
            .recv_timeout(WATCHDOG)
            .expect("must reach the breakpoint and emit Stopped");

        // Never send Continue/Disconnect. The host thread must NOT complete.
        let handle = host.handle.take().expect("handle present");
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });
        let completed = done_rx.recv_timeout(DEADLOCK_WATCHDOG);
        assert!(
            matches!(completed, Err(RecvTimeoutError::Timeout)),
            "a session with no resume command must NOT complete within the watchdog \
             (stop core stays unbounded); got {completed:?}"
        );
        // Intentionally detach the blocked host thread (no forced core timeout).
        assert!(host.handle.is_none());
    }

    /// `report_error` stringifies an `mlua::Error` (`!Send`) into a
    /// `SessionEvent::Error` so it can cross the channel (design "Error
    /// Handling"). Proves the !Send-crossing seam without a VM thread.
    #[test]
    fn report_error_stringifies_mlua_error() {
        let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();
        let session = DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx);

        let err = mlua::Error::RuntimeError("boom".to_string());
        session.report_error(&err);

        match event_rx.recv().expect("an Error event must be sent") {
            SessionEvent::Error(msg) => assert!(
                msg.contains("boom"),
                "the stringified error must carry the message: {msg:?}"
            ),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    /// Compile-time guard: the channel-seam payloads are `Send` so they cross the
    /// VM/controller boundary (and `mlua::Lua` never does).
    #[test]
    fn channel_payloads_are_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SessionCommand>();
        assert_send::<SessionEvent>();
        // A type sanity check that `Mutex<Option<String>>` (used elsewhere as the
        // !Send-crossing seam) is Send, while documenting intent.
        assert_send::<Arc<Mutex<Option<String>>>>();
    }

    // =======================================================================
    // Task 2.5 — StepController (step over / into / out) integration tests.
    //
    // These drive a host-thread `DebugSession` over a chunk that contains a
    // function call AND a coroutine `yield`, set a breakpoint, then issue
    // `Next`/`StepIn`/`StepOut` and assert the EXACT `.lua` line the step
    // stops on (R1.3 / R1.4 / R1.5). They also prove:
    //   - a step started inside a coroutine survives `yield`/`resume` (採択B);
    //   - lines belonging to the host loop / a DIFFERENT coroutine are SKIPPED
    //     while stepping the target thread (thread-identity tracking);
    //   - a breakpoint elsewhere still stops while in Stepping mode.
    //
    // The stop core stays UNBOUNDED; only the controller side uses a watchdog.
    // =======================================================================

    /// A flexible step-test host: builds a VM, installs the hook wired to a
    /// `DebugSession`, and runs a caller-supplied chunk under a caller-supplied
    /// source name. The hook wrapper records the line seen just BEFORE each
    /// `session.on_line` call into a shared `AtomicU32`, so when the controller
    /// receives a `Stopped` event it can read the EXACT line the session
    /// stopped on (the wrapper's write happens-before the session's `Stopped`
    /// send, so the value is current by the time the controller observes it).
    struct StepHost {
        cmd_tx: Sender<SessionCommand>,
        event_rx: Receiver<SessionEvent>,
        /// Line the hook last entered `on_line` with (the stop line on a stop).
        last_line: Arc<std::sync::atomic::AtomicU32>,
        handle: Option<JoinHandle<Result<(), String>>>,
    }

    impl StepHost {
        /// Receive the next `Stopped` event (bounded by the test watchdog) and
        /// return `(reason, stop_line)` — `stop_line` read from `last_line`.
        fn recv_stop(&self) -> (StopReason, u32) {
            loop {
                match self
                    .event_rx
                    .recv_timeout(WATCHDOG)
                    .expect("must receive a session event before the watchdog")
                {
                    SessionEvent::Stopped { reason, .. } => {
                        return (reason, self.last_line.load(Ordering::SeqCst));
                    }
                    // Ignore non-stop events (none expected in these scenarios).
                    other => panic!("unexpected event while awaiting a stop: {other:?}"),
                }
            }
        }

        fn cont(&self, cmd: SessionCommand) {
            self.cmd_tx.send(cmd).expect("command send must succeed");
        }

        /// Bounded join: a stepping regression that hangs the VM thread is a
        /// test failure, not a suite-killer (the stop core stays unbounded).
        fn join(&mut self) {
            let handle = self.handle.take().expect("join at most once");
            let (done_tx, done_rx) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = done_tx.send(handle.join());
            });
            done_rx
                .recv_timeout(WATCHDOG)
                .expect("host thread must finish after Continue/Disconnect")
                .expect("host thread must not panic")
                .expect("scenario must run to completion");
        }
    }

    /// Start a `StepHost` running `chunk` (named `source`) with the given
    /// breakpoints. The chunk is `exec`'d directly (a `coroutine.create`/
    /// `resume` driver is part of the chunk text when a coroutine is needed).
    fn start_step_host(
        breakpoints: BreakpointSet,
        chunk: &'static str,
        source: &'static str,
    ) -> StepHost {
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

        let last_line = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let hook_last_line = Arc::clone(&last_line);

        let handle = std::thread::spawn(move || -> Result<(), String> {
            let lua = build_all_safe_vm();
            let session = DebugSession::new(breakpoints, cmd_rx, event_tx);

            let handler = move |lua: &Lua, debug: &Debug| {
                // Record the line BEFORE delegating (on_line may block here).
                let line = debug.current_line().unwrap_or(0) as u32;
                hook_last_line.store(line, Ordering::SeqCst);
                session.on_line(lua, debug)
            };

            crate::debug::hook::install(&lua, handler).map_err(|e| e.to_string())?;
            lua.load(chunk)
                .set_name(source)
                .exec()
                .map_err(|e| e.to_string())?;
            lua.remove_global_hook();
            Ok(())
        });

        StepHost {
            cmd_tx,
            event_rx,
            last_line,
            handle: Some(handle),
        }
    }

    // A chunk with a helper function call. Lines (1-origin):
    //   1: local function helper(x)
    //   2:     local y = x + 1      <- StepIn target (helper's first body line)
    //   3:     return y
    //   4: end
    //   5: local a = 1
    //   6: local b = helper(a)      <- BREAKPOINT (stop here, step from here)
    //   7: local c = b + 1          <- StepOver / StepOut target (back in caller)
    //   8: return c
    const CALL_SOURCE: &str = "@step_call_scenario";
    const CALL_CHUNK: &str = "\
local function helper(x)
    local y = x + 1
    return y
end
local a = 1
local b = helper(a)
local c = b + 1
return c
";
    const CALL_BP_LINE: u32 = 6;

    /// R1.3 (step over): stopped at the call `local b = helper(a)` (line 6),
    /// `Next` must stop at line 7 in the SAME frame — NOT inside `helper`
    /// (lines 2/3). The called function's lines are skipped.
    #[test]
    fn step_over_skips_called_function() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(CALL_SOURCE), &[CALL_BP_LINE]);
        let mut host = start_step_host(breakpoints, CALL_CHUNK, CALL_SOURCE);

        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Breakpoint);
        assert_eq!(line, CALL_BP_LINE, "must stop at the breakpoint line first");

        // Step over the call.
        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "step over must stop with reason Step");
        assert_eq!(
            line, 7,
            "step over must stop at the next line in the SAME frame (7), NOT inside helper"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// R1.4 (step in): stopped at the call line 6, `StepIn` must stop at
    /// `helper`'s first executable body line (line 2).
    #[test]
    fn step_in_enters_called_function() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(CALL_SOURCE), &[CALL_BP_LINE]);
        let mut host = start_step_host(breakpoints, CALL_CHUNK, CALL_SOURCE);

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, CALL_BP_LINE);

        host.cont(SessionCommand::StepIn);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "step in must stop with reason Step");
        assert_eq!(
            line, 2,
            "step in must stop at the callee's first body line (2), entering helper"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// R1.5 (step out): step INTO `helper` (stop at line 2), then `StepOut`
    /// must stop back in the caller AFTER `helper` returns — line 7.
    #[test]
    fn step_out_returns_to_caller() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(CALL_SOURCE), &[CALL_BP_LINE]);
        let mut host = start_step_host(breakpoints, CALL_CHUNK, CALL_SOURCE);

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, CALL_BP_LINE);

        // Step into helper (now stopped at line 2, depth = base+1).
        host.cont(SessionCommand::StepIn);
        let (_reason, line) = host.recv_stop();
        assert_eq!(line, 2, "precondition: stepped into helper at line 2");

        // Step out: must return to the caller frame past the call (line 7).
        host.cont(SessionCommand::StepOut);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "step out must stop with reason Step");
        assert_eq!(
            line, 7,
            "step out must stop back in the caller after helper returns (line 7)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    // A coroutine scenario. The body runs inside a Lua-side coroutine and
    // `yield`s; a step started BEFORE the yield must complete AFTER the resume
    // (採択B survival). Chunk lines (1-origin):
    //   1: local body = function()
    //   2:     local a = 1            <- BREAKPOINT (stop inside the coroutine)
    //   3:     coroutine.yield()      <- step over from line 2 lands here...
    //   4:     local b = a + 1        <- ...a SECOND step over the yield lands here
    //                                     (only reached on the NEXT resume — 採択B)
    //   5:     return b
    //   6: end
    //   7: local co = coroutine.create(body)
    //   8: while coroutine.status(co) ~= 'dead' do   <- driver loop (OTHER thread)
    //   9:     coroutine.resume(co)                   <- driver loop (OTHER thread)
    //  10: end
    const CO_CHUNK: &str = "\
local body = function()
    local a = 1
    coroutine.yield()
    local b = a + 1
    return b
end
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
";
    const CO_SOURCE: &str = "@step_co_scenario";
    const CO_BODY_BP_LINE: u32 = 2; // `local a = 1` is chunk line 2.

    /// 採択B (yield/resume survival) + thread-mismatch skip: stopped inside the
    /// coroutine body at `local a = 1` (line 2), a first `Next` reaches the
    /// `coroutine.yield()` line (3); a SECOND `Next` steps OVER the yield. That
    /// second step suspends the coroutine — the driver loop (a DIFFERENT thread,
    /// lines 8/9) runs to re-resume and MUST be skipped — and only completes at
    /// the post-yield body line 4 on the NEXT resume of the SAME coroutine
    /// (proving the `(thread, base_depth)` key survives the yield boundary).
    #[test]
    fn step_over_survives_coroutine_yield_and_skips_other_threads() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(CO_SOURCE), &[CO_BODY_BP_LINE]);
        let mut host = start_step_host(breakpoints, CO_CHUNK, CO_SOURCE);

        // Stop inside the coroutine body at `local a = 1` (line 2).
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Breakpoint);
        assert_eq!(line, CO_BODY_BP_LINE, "must stop inside the coroutine body");

        // First step over: same frame, next line is `coroutine.yield()` (3).
        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step);
        assert_eq!(
            line, 3,
            "first step over must reach the yield line (3) in the same frame"
        );

        // Second step over: this steps OVER `coroutine.yield()`. The coroutine
        // suspends; the driver loop (lines 8/9, the MAIN thread) runs to
        // re-resume. Those driver lines must be SKIPPED (thread mismatch). The
        // step completes only on the SAME coroutine's post-yield body line 4 —
        // proving the step key survived the yield/resume boundary (採択B).
        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "stepping must stop with reason Step");
        assert_eq!(
            line, 4,
            "a step over `coroutine.yield()` must complete at the post-yield body \
             line (4) AFTER the resume — NOT on a driver-loop line (採択B survival)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// A breakpoint set elsewhere must STILL stop while in Stepping mode — a
    /// long-running step must not mask a breakpoint. Stop at the call (line 6),
    /// set a breakpoint at line 2 (inside helper), then `Next` (step over):
    /// even though step-over would skip helper, the line-2 breakpoint must fire.
    #[test]
    fn breakpoint_still_stops_while_stepping() {
        let breakpoints = BreakpointSet::new();
        // Breakpoint at the call line AND inside helper (line 2).
        breakpoints.set_breakpoints(&SourceRef::new(CALL_SOURCE), &[CALL_BP_LINE, 2]);
        let mut host = start_step_host(breakpoints, CALL_CHUNK, CALL_SOURCE);

        // First stop: the call line.
        let (_reason, line) = host.recv_stop();
        assert_eq!(line, CALL_BP_LINE);

        // Step over: helper's body would be skipped by step-over, but the line-2
        // breakpoint must still fire with reason Breakpoint.
        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(
            line, 2,
            "the breakpoint inside helper (line 2) must fire even while stepping over"
        );
        assert_eq!(
            reason,
            StopReason::Breakpoint,
            "a breakpoint hit while Stepping must report reason Breakpoint, not Step"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }
}
