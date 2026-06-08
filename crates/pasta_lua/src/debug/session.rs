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

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};

use mlua::{Debug, Lua, VmState};

use crate::debug::SourceMode;
use crate::debug::breakpoints::BreakpointSet;
use crate::debug::hook::LineHook;
use crate::debug::inspect::{capture_stack, capture_variables};
use crate::debug::source_map::{PastaPos, SourceMap};
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

/// The session's run mode (design "DebugSession 状態機械" + "PastaStepper").
///
/// `Stepping` keeps the coroutine identity (`thread`) and the captured stack
/// depth (`base_depth`) so the StepController can decide over/into/out by
/// comparing the current thread+depth against these (task 2.5). It also records
/// the `start_line` so step-over can detect "line changed in the same frame"
/// (a same-frame statement that spans the call's own line must not re-trigger).
///
/// For `.pasta`-granular stepping (task 5.4 / requirements 9.1–9.5) it ALSO
/// records `origin_pasta`: the resolved `.pasta` position the step began on
/// (`Some` only in [`SourceMode::Pasta`] with a map AND when the start `.lua`
/// line itself maps to a `.pasta` position; `None` otherwise). Together with
/// `(thread, base_depth)` this forms the frame identity `(thread, base_depth,
/// .pasta-file, .pasta-line)` used to consume all `.lua` lines mapping to the
/// SAME `.pasta` line and stop at the next DIFFERENT one (design 549–556).
///
/// Because `origin_pasta` holds a [`PastaPos`] (which owns a `String` file
/// path), `RunMode` is NOT `Copy`/`Eq`; it is stored behind a `RefCell` and
/// read via `clone()`.
#[derive(Debug, Clone, PartialEq)]
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
        /// The resolved `.pasta` position the step began on (design 544:
        /// `origin_pasta: Option<(ChunkName, u32)>`). Stored as the resolved
        /// [`PastaPos`] `{file, line}` — which IS the design's `(.pasta-source,
        /// line)` identity — so the "same `.pasta` line" test (stop decision
        /// step 3, design 552 「同 chunk・同行」) is a direct `PastaPos` equality
        /// of FILE + LINE, not a chunk-name comparison. `Some` only in
        /// [`SourceMode::Pasta`] with a map AND when the start `.lua` line maps;
        /// `None` for `.lua` mode / no map / unmapped start line (in which case
        /// the existing `.lua`-granularity decision is used unchanged — 9.5).
        origin_pasta: Option<PastaPos>,
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
    /// Current run mode. `RefCell` (not `Cell`) because [`RunMode::Stepping`]
    /// now carries a non-`Copy` `origin_pasta: Option<PastaPos>` (task 5.4); the
    /// line hook calls `&self` and the session is single-threaded (VM thread),
    /// so interior mutability is race-free here.
    mode: std::cell::RefCell<RunMode>,
    /// Immutable shared source map for `.pasta`↔`.lua` resolution, threaded in by
    /// [`enable`](crate::debug::enable) (task 4.2 plumbing). `Some` only when a
    /// map was supplied AND the present mode is [`SourceMode::Pasta`]; `None`
    /// otherwise (no map, or `SourceMode::Lua`) so the stepper keeps its existing
    /// `.lua` granularity (requirements 6.1 / 6.2). The `.pasta`-granular stepping
    /// CONSUMER of this field lands in task 5.4; this task only carries it here so
    /// it REACHES the stepper. `Arc` is the immutable shared form (design
    /// "Architecture": `Arc<SourceMap>` 不変共有).
    source_map: Option<Arc<SourceMap>>,
    /// The resolved present mode for this session (requirements 6.1 default
    /// `.pasta`, 6.2 `.lua`). Carried alongside `source_map` so the stepper (5.4)
    /// can branch its granularity; default [`SourceMode::Pasta`] keeps parity with
    /// [`DebugConfig`] when unset. This is the `enable`-time resolved fallback —
    /// the EFFECTIVE mode is `shared_mode` when present (task 5.5).
    source_mode: SourceMode,
    /// The SHARED, interior-mutable EFFECTIVE present mode (task 5.5 /
    /// requirement 6.3). `Some` when [`enable`](crate::debug::enable) threaded a
    /// shared cell whose value the socket bridge can flip on a DAP `attach`
    /// `sourcePresentation` (highest precedence, design 581). When present the
    /// stepper reads THIS (so an `attach` switches `.pasta`↔`.lua` step
    /// granularity for the current session); when `None` it falls back to the
    /// baked `source_mode`. Held as a cheap `Arc` clone shared with the
    /// socket-bridge thread, mirroring [`BreakpointSet`].
    shared_mode: Option<crate::debug::SharedSourceMode>,
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
            mode: std::cell::RefCell::new(RunMode::Running),
            // Default `.lua` granularity: no map, present mode `Pasta` but with no
            // map the stepper still behaves exactly as today (task 4.2 plumbing;
            // `.pasta` stepping is task 5.4). `enable` overrides these via
            // [`with_source_map`](Self::with_source_map) when a map is supplied in
            // `SourceMode::Pasta` (requirements 6.1 / 6.2).
            source_map: None,
            source_mode: SourceMode::default(),
            shared_mode: None,
        }
    }

    /// Thread the (optional) shared source map and present mode into the session
    /// (task 4.2 injection point: `enable → wiring → DebugSession`, design 548).
    ///
    /// This is the SKELETON plumbing only: it STORES the map+mode so the
    /// `.pasta`-granular stepper (task 5.4) can read them; no stepping behavior
    /// changes here. The gating decision lives in [`enable`](crate::debug::enable):
    /// it passes `Some(map)` only when a map exists AND the present mode is
    /// [`SourceMode::Pasta`] (requirements 6.1); for `None`/`SourceMode::Lua` the
    /// session keeps its existing `.lua` behavior (requirements 6.2, 7.2).
    pub(crate) fn with_source_map(
        mut self,
        source_map: Option<Arc<SourceMap>>,
        source_mode: SourceMode,
    ) -> Self {
        self.source_map = source_map;
        self.source_mode = source_mode;
        self
    }

    /// Thread the SHARED, interior-mutable EFFECTIVE present mode into the session
    /// (task 5.5 / requirement 6.3: `enable → wiring → DebugSession`).
    ///
    /// When set, the stepper reads this shared cell instead of the baked
    /// `source_mode`, so a DAP `attach` `sourcePresentation` (flipped on the
    /// socket-bridge thread, highest precedence per design 581) switches this
    /// session's `.pasta`↔`.lua` STEP granularity. The cell is initialised at
    /// `enable` to the resolved env > file > 既定 mode, so with no `attach`
    /// override the behavior is identical to the baked `source_mode` (task 5.4).
    pub(crate) fn with_shared_mode(
        mut self,
        shared_mode: Option<crate::debug::SharedSourceMode>,
    ) -> Self {
        self.shared_mode = shared_mode;
        self
    }

    /// The EFFECTIVE present mode for this session: the shared cell when threaded
    /// (so a DAP `attach` flip is observed, task 5.5), else the baked resolved
    /// `source_mode` (task 5.4). This is the single read the stepper consults.
    fn effective_mode(&self) -> SourceMode {
        match &self.shared_mode {
            Some(shared) => shared.get(),
            None => self.source_mode,
        }
    }

    /// The current run mode (controller-side / test observation helper).
    pub(crate) fn mode(&self) -> RunMode {
        self.mode.borrow().clone()
    }

    /// The threaded shared source map, if any (task 4.2 plumbing observation /
    /// task 5.4 stepper consumer). `Some` only when `enable` was given a map in
    /// [`SourceMode::Pasta`]; `None` for the default `.lua` behavior.
    pub(crate) fn source_map(&self) -> Option<&Arc<SourceMap>> {
        self.source_map.as_ref()
    }

    /// The EFFECTIVE present mode (requirements 6.1 / 6.2 / 6.3). Reads the shared
    /// cell when threaded (so a DAP `attach` flip is reflected, task 5.5), else
    /// the baked resolved mode. Default [`SourceMode::Pasta`] until `enable`
    /// threads the resolved mode.
    pub(crate) fn source_mode(&self) -> SourceMode {
        self.effective_mode()
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

    /// Resolve the current `(source, line)` to a `.pasta` position FOR STEPPING,
    /// gated on `.pasta` granularity being active (task 5.4 / requirements 9.5).
    ///
    /// Returns `Some(PastaPos)` ONLY when this session is in
    /// [`SourceMode::Pasta`] AND a [`SourceMap`] is present AND the current
    /// `.lua` line maps to a `.pasta` position. Otherwise `None`:
    /// - `SourceMode::Lua` or no map → `.pasta` stepping is disabled, the stepper
    ///   keeps its existing `.lua` granularity (9.5).
    /// - mapped chunk/line miss → the line is `.pasta`-unmapped (passed through;
    ///   9.4).
    ///
    /// The RAW hook `source` is passed straight through to
    /// [`SourceMap::resolve_lua_to_pasta`], which canonicalizes the chunk
    /// internally (task 3.4); the caller must NOT pre-canonicalize.
    fn resolve_current_pasta(&self, source: &str, line: u32) -> Option<PastaPos> {
        // Read the EFFECTIVE mode (shared cell when threaded) so a DAP `attach`
        // `sourcePresentation` flip switches `.pasta` stepping for this session
        // (task 5.5 / requirement 6.3).
        if self.effective_mode() != SourceMode::Pasta {
            return None;
        }
        let map = self.source_map.as_ref()?;
        map.resolve_lua_to_pasta(source, line).cloned()
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

    /// The `.pasta`-granular stop refinement layered on top of
    /// [`step_should_stop`](Self::step_should_stop) for [`SourceMode::Pasta`]
    /// (task 5.4 / requirements 9.1–9.5; design "PastaStepper" 549–556, Flow 4).
    ///
    /// This is ONLY consulted AFTER the existing `.lua`-granularity
    /// `step_should_stop` has already returned `true` for the current line — it
    /// can therefore only DEMOTE a `.lua` stop to "keep going", never invent a
    /// stop the `.lua` machinery would not have produced. The decision (design
    /// 549–556, reconciling branch 1 「異フレームは `.lua` 判定」 with 554/555
    /// 「step into/out は最初の `.pasta` 対応行で停止」 and 9.4 「未対応行は通過」):
    ///
    /// 1. **Current `.lua` line is `.pasta`-unmapped** (`cur_pasta == None`):
    ///    CONTINUE (`false`) — pass through unmapped lines, in ANY frame. This
    ///    realizes 9.4/E6 for the origin frame AND the "stop at the first
    ///    *mapped* `.pasta` line" of step into (9.2/E3 — skip unmapped callee
    ///    lines) and step out (9.3/E4 — skip unmapped caller lines). Sub-call /
    ///    recursion lines that reach here would only be in the origin frame,
    ///    because `step_should_stop` already excluded DEEPER frames for
    ///    `Over`/`Out` (E2/E5 are handled structurally by depth before this
    ///    function is consulted).
    /// 2. **Same frame as the step origin** (`cur_thread == thread` AND
    ///    `depth == base_depth`) **AND current `.pasta` == origin** (same FILE +
    ///    LINE): CONTINUE (`false`) — consume all `.lua` lines of the SAME
    ///    `.pasta` line in the origin frame (9.1/E1). The same-frame guard makes
    ///    step into "discard" the origin (design 554): a callee line is a
    ///    DIFFERENT frame, so even if it coincidentally maps to the origin's
    ///    `.pasta` line it is NOT consumed — it stops (step 3).
    /// 3. **Otherwise** — a mapped line that is either a DIFFERENT `.pasta` line
    ///    in the origin frame (9.1 next line) OR any mapped line in a different
    ///    frame (step into callee / step out caller — 9.2/9.3): STOP (`true`).
    ///
    /// `cur_thread`/`depth` come from
    /// [`current_thread_and_depth`](Self::current_thread_and_depth). `cur_pasta`
    /// is `resolve_lua_to_pasta(RAW source, line)` for the current line (the map
    /// canonicalizes the chunk internally — task 3.4 — so the RAW hook source is
    /// passed straight through). `origin_pasta` is the position captured when the
    /// step began (`None` if the start line was itself unmapped, in which case a
    /// mapped current line is a genuine `.pasta` transition and stops — step 3).
    fn pasta_step_should_stop(
        thread: ThreadId,
        base_depth: u32,
        origin_pasta: Option<&PastaPos>,
        cur_thread: ThreadId,
        depth: u32,
        cur_pasta: Option<&PastaPos>,
    ) -> bool {
        // (1) 現 `.lua` 行が `.pasta` 未対応 → 通過（継続・9.4/E6）。フレームに依らず
        //     未対応行は飛ばし、step into/out は最初の「対応行」で止める（9.2/9.3）。
        let Some(cur) = cur_pasta else {
            return false;
        };
        // (2) 同一起点フレーム かつ 現 `.pasta` 位置が起点と同一（同 file・同 line）→
        //     同一 `.pasta` 行を消化（継続・9.1/E1）。同フレーム条件により step into は
        //     起点 `.pasta` を破棄（呼び出し先は別フレームなので消化対象にしない）。
        let same_frame = cur_thread == thread && depth == base_depth;
        if same_frame && origin_pasta == Some(cur) {
            return false;
        }
        // (3) それ以外（同フレームで異なる `.pasta` 対応行、または別フレームの対応行）
        //     → 停止（9.1 次行 / 9.2 step into / 9.3 step out）。
        true
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
                    *self.mode.borrow_mut() = RunMode::Running;
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
                    let (source, start_line) = Self::source_and_line(debug);
                    // `.pasta`-granular origin (task 5.4 / 9.1): in `SourceMode::Pasta`
                    // with a map, resolve the START line's `.pasta` position so the
                    // stepper can consume all `.lua` lines of the SAME `.pasta` line
                    // and stop at the next DIFFERENT one. `None` for `.lua` mode / no
                    // map (9.5: unchanged `.lua` granularity) or an unmapped start
                    // line. The RAW hook source is passed straight through — the map
                    // canonicalizes the chunk internally (task 3.4); do NOT
                    // double-canonicalize.
                    let origin_pasta = self.resolve_current_pasta(&source, start_line);
                    *self.mode.borrow_mut() = RunMode::Stepping {
                        kind,
                        thread,
                        base_depth,
                        start_line,
                        origin_pasta,
                    };
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
    /// is the step's completion point at `.lua` granularity (reason
    /// [`StopReason::Step`]).
    ///
    /// In [`SourceMode::Pasta`] with a map, that `.lua`-granularity stop is then
    /// REFINED to `.pasta` granularity by
    /// [`pasta_step_should_stop`](Self::pasta_step_should_stop) (task 5.4 /
    /// requirements 9.1–9.5): same-`.pasta`-line and unmapped `.lua` lines in the
    /// origin frame are consumed (continue), and the step completes only at the
    /// next DIFFERENT `.pasta` line (same frame) or in a different frame (step
    /// into/out). In `.lua` mode / no map the stop is taken as-is (9.5).
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
        // Clone the mode out of the RefCell first so the borrow is released
        // before `stop_loop` (which re-borrows `mode`); `origin_pasta` is a
        // small owned `Option<PastaPos>`.
        let current_mode = self.mode.borrow().clone();
        if let RunMode::Stepping {
            kind,
            thread,
            base_depth,
            start_line,
            origin_pasta,
        } = current_mode
        {
            let (cur_thread, depth) = Self::current_thread_and_depth(lua);
            // (a) Existing `.lua`-granularity decision (9.5 unchanged).
            if Self::step_should_stop(
                kind, thread, base_depth, start_line, cur_thread, depth, line,
            ) {
                // (b) In `.pasta` mode (a map is present), REFINE the `.lua` stop
                //     to `.pasta` granularity (9.1–9.4). The RAW source is passed
                //     through; the map canonicalizes the chunk internally (3.4).
                //     `resolve_current_pasta` is `None` for `.lua` mode / no map,
                //     so the refinement is skipped and the `.lua` stop is taken
                //     as-is (9.5).
                let take_stop = if self.effective_mode() == SourceMode::Pasta
                    && self.source_map.is_some()
                {
                    let cur_pasta = self.resolve_current_pasta(&source, line);
                    Self::pasta_step_should_stop(
                        thread,
                        base_depth,
                        origin_pasta.as_ref(),
                        cur_thread,
                        depth,
                        cur_pasta.as_ref(),
                    )
                } else {
                    true
                };
                if take_stop {
                    return self.stop_loop(lua, debug, StopReason::Step, MAIN_THREAD_ID);
                }
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

    // =======================================================================
    // Task 4.2 — source map injection plumbing (enable → wiring → DebugSession).
    //
    // The session is the STEPPER consumer (task 5.4). These tests prove the
    // SKELETON: the gated `Arc<SourceMap>` + present mode REACH the session in
    // `.pasta` mode, and are absent (default `.lua` behavior) for `None`/`Lua`
    // (requirements 6.1 / 6.2 / 7.2). No stepping behavior is implemented here.
    // =======================================================================

    /// 6.1 / design 548: a `Some(map)` threaded in `SourceMode::Pasta` REACHES the
    /// session (the stepper-holding struct), observable via `source_map()` /
    /// `source_mode()`. This is the injection-path "arrival" assertion.
    #[test]
    fn with_source_map_pasta_threads_map_into_session() {
        let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();
        let map = Arc::new(SourceMap::new());

        let session = DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx)
            .with_source_map(Some(Arc::clone(&map)), SourceMode::Pasta);

        // The map reaches the session (Some) and the present mode is Pasta (6.1).
        assert!(
            session.source_map().is_some(),
            "Some(map) in Pasta mode must REACH the session (design 548)"
        );
        assert_eq!(session.source_mode(), SourceMode::Pasta);
        // It is the SAME shared Arc (immutable shared, design Architecture).
        assert!(Arc::ptr_eq(session.source_map().unwrap(), &map));
    }

    /// 6.2 / 7.2: a `None` map (every existing call site post-4.2) leaves the
    /// session with NO map — the default `.lua` behavior, unchanged from today.
    #[test]
    fn none_map_leaves_session_default_lua_behavior() {
        let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();

        // Default constructor: no map, default mode.
        let default_session =
            DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx);
        assert!(
            default_session.source_map().is_none(),
            "default session must hold NO map (default `.lua` behavior, 7.2)"
        );

        // Explicit None threading is likewise map-less.
        let (_c2, cmd_rx2) = mpsc::channel::<SessionCommand>();
        let (ev2, _e2) = mpsc::channel::<SessionEvent>();
        let lua_session = DebugSession::new(BreakpointSet::new(), cmd_rx2, ev2)
            .with_source_map(None, SourceMode::Lua);
        assert!(
            lua_session.source_map().is_none(),
            "None in `.lua` mode must leave the session map-less (6.2 / 7.2)"
        );
        assert_eq!(lua_session.source_mode(), SourceMode::Lua);
    }

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

    // =======================================================================
    // Task 5.4 — `.pasta`-granular stepping (requirements 9.1–9.5).
    //
    // In `SourceMode::Pasta` with a `SourceMap`, stepping is `.pasta`-line
    // granular: step over consumes all `.lua` lines mapping to the SAME `.pasta`
    // line in the origin frame and stops at the next DIFFERENT `.pasta` line
    // (9.1); unmapped `.lua` lines are passed through (9.4); step into stops at
    // the callee's first MAPPED `.pasta` line (9.2); step out stops at the first
    // MAPPED `.pasta` line in the caller (9.3). `SourceMode::Lua` (or no map)
    // keeps the existing `.lua` granularity unchanged (9.5).
    //
    // The pure stop-decision core `pasta_step_should_stop` (design 549–556) is
    // unit-tested directly; the end-to-end host-thread tests drive a real VM
    // with a controlled `SourceMap` injected via `with_source_map`.
    // =======================================================================

    // `PastaPos` / `SourceMap` are already in scope via `use super::*`; only the
    // builder type `ChunkSourceMap` and `BTreeMap` need importing here.
    use crate::debug::source_map::ChunkSourceMap;
    use std::collections::BTreeMap;

    /// Build a `PastaPos` in a fixed `.pasta` file for these tests.
    fn ppos(line: u32) -> PastaPos {
        PastaPos {
            file: "scene.pasta".to_string(),
            line,
        }
    }

    // ----------------------------------------------------------------------
    // Unit tests for the pure `.pasta` stop-decision core (design 549–556).
    // These synthesize (thread, depth, origin_pasta, cur_pasta) inputs and need
    // no VM — they pin the 4 behaviours the host-thread tests exercise E2E.
    // ----------------------------------------------------------------------

    /// 9.1/E1: same origin frame, current `.lua` line maps to the SAME `.pasta`
    /// line as the origin → CONTINUE (consume the `.pasta` line's `.lua` lines).
    #[test]
    fn pasta_decision_same_pasta_line_same_frame_continues() {
        let t = ThreadId(0xAB);
        let origin = ppos(10);
        let cur = ppos(10);
        assert!(
            !DebugSession::pasta_step_should_stop(
                t, 3, Some(&origin), t, 3, Some(&cur)
            ),
            "same `.pasta` line in the origin frame must be consumed (continue, 9.1)"
        );
    }

    /// 9.1: same origin frame, current `.lua` line maps to a DIFFERENT `.pasta`
    /// line → STOP (the next `.pasta` line is reached).
    #[test]
    fn pasta_decision_different_pasta_line_same_frame_stops() {
        let t = ThreadId(0xAB);
        let origin = ppos(10);
        let cur = ppos(11);
        assert!(
            DebugSession::pasta_step_should_stop(
                t, 3, Some(&origin), t, 3, Some(&cur)
            ),
            "a DIFFERENT `.pasta` line in the same frame must STOP (9.1)"
        );
    }

    /// 9.4/E6: current `.lua` line is `.pasta`-unmapped → CONTINUE (pass through),
    /// regardless of frame.
    #[test]
    fn pasta_decision_unmapped_line_passes_through() {
        let t = ThreadId(0xAB);
        let origin = ppos(10);
        // Same frame, unmapped current line.
        assert!(
            !DebugSession::pasta_step_should_stop(t, 3, Some(&origin), t, 3, None),
            "an unmapped `.lua` line must be passed through (continue, 9.4)"
        );
        // Different frame (deeper), unmapped current line — also passes (skip
        // unmapped callee lines for step into, 9.2/9.4).
        assert!(
            !DebugSession::pasta_step_should_stop(t, 3, Some(&origin), t, 4, None),
            "an unmapped line in a deeper frame must also be passed through (9.2/9.4)"
        );
    }

    /// 9.2/E3: step into — a mapped line in a DEEPER frame (callee) stops, and the
    /// origin `.pasta` is discarded (a callee line mapping to the SAME `.pasta`
    /// line as the origin still STOPS, because it is a different frame).
    #[test]
    fn pasta_decision_deeper_frame_mapped_line_stops_discarding_origin() {
        let t = ThreadId(0xAB);
        let origin = ppos(10);
        // Callee mapped line with a DIFFERENT `.pasta` line → stop.
        let cur_diff = ppos(20);
        assert!(
            DebugSession::pasta_step_should_stop(
                t, 3, Some(&origin), t, 4, Some(&cur_diff)
            ),
            "a mapped line in the callee frame must STOP (step into, 9.2)"
        );
        // Callee mapped line coincidentally equal to the origin `.pasta` line →
        // still STOP (different frame discards the origin; design 554).
        let cur_same = ppos(10);
        assert!(
            DebugSession::pasta_step_should_stop(
                t, 3, Some(&origin), t, 4, Some(&cur_same)
            ),
            "a callee line equal to the origin `.pasta` line still STOPS (origin \
             discarded across frames, 9.2)"
        );
    }

    /// 9.3/E4: step out — a mapped line in a SHALLOWER frame (caller) stops.
    #[test]
    fn pasta_decision_shallower_frame_mapped_line_stops() {
        let t = ThreadId(0xAB);
        let origin = ppos(20); // origin captured inside the callee
        let cur = ppos(12); // a mapped caller line after return
        assert!(
            DebugSession::pasta_step_should_stop(
                t, 4, Some(&origin), t, 3, Some(&cur)
            ),
            "a mapped line in the caller frame after return must STOP (step out, 9.3)"
        );
    }

    /// A different THREAD (host loop / another coroutine) with a mapped line:
    /// branch (3) STOPs (different frame). The thread-mismatch SKIP that protects
    /// against mis-stopping on other threads is enforced earlier by
    /// `step_should_stop` (which returns false for `cur_thread != thread`), so by
    /// the time this refinement runs the line is already on a relevant frame.
    #[test]
    fn pasta_decision_origin_none_mapped_line_stops() {
        let t = ThreadId(0xAB);
        // Unmapped start line (origin None): the first mapped line is a genuine
        // `.pasta` transition → STOP (9.1/9.4 combined).
        let cur = ppos(11);
        assert!(
            DebugSession::pasta_step_should_stop(t, 3, None, t, 3, Some(&cur)),
            "with no origin `.pasta` (unmapped start), the first mapped line STOPS"
        );
    }

    // ----------------------------------------------------------------------
    // End-to-end host-thread tests with an injected `SourceMap`.
    // ----------------------------------------------------------------------

    // `.pasta`-stepping scenario chunk. Lines (1-origin), source PASTA_SOURCE:
    //   1: local function helper(x)
    //   2:     local y = x + 1      <- callee: UNMAPPED (passed through on step in)
    //   3:     local z = y + 1      <- callee: .pasta 20 (step-in stop target)
    //   4:     return z
    //   5: end
    //   6: local a = 1              <- .pasta 10  (BREAKPOINT / step origin)
    //   7: local b = a + 1          <- .pasta 10  (SAME .pasta as 6 -> consumed)
    //   8: local c = b + 1          <- UNMAPPED   (passed through)
    //   9: local d = helper(c)      <- .pasta 11  (DIFFERENT .pasta -> step-over stop)
    //  10: return d                 <- .pasta 12  (step-out stop target)
    const PASTA_SOURCE: &str = "@pasta_step_scenario";
    const PASTA_CHUNK: &str = "\
local function helper(x)
    local y = x + 1
    local z = y + 1
    return z
end
local a = 1
local b = a + 1
local c = b + 1
local d = helper(c)
return d
";
    const PASTA_BP_LINE: u32 = 6;

    /// Build the `SourceMap` for `PASTA_CHUNK` (keyed by the hook source name,
    /// which the map canonicalizes internally — task 3.4).
    fn pasta_scenario_map() -> Arc<SourceMap> {
        let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
        // caller frame
        forward.insert(6, ppos(10));
        forward.insert(7, ppos(10)); // same .pasta line as 6
        // line 8 intentionally unmapped
        forward.insert(9, ppos(11));
        forward.insert(10, ppos(12));
        // callee frame
        // line 2 intentionally unmapped
        forward.insert(3, ppos(20));
        forward.insert(4, ppos(21));

        let mut sm = SourceMap::new();
        sm.insert_chunk(
            PASTA_SOURCE.to_string(),
            "scene.pasta".to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        Arc::new(sm)
    }

    /// Start a `StepHost` like [`start_step_host`] but inject a `SourceMap` +
    /// `SourceMode` into the `DebugSession` (task 4.2 `with_source_map`), so the
    /// stepper runs at `.pasta` granularity (5.4).
    fn start_step_host_with_map(
        breakpoints: BreakpointSet,
        chunk: &'static str,
        source: &'static str,
        source_map: Option<Arc<SourceMap>>,
        source_mode: SourceMode,
    ) -> StepHost {
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

        let last_line = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let hook_last_line = Arc::clone(&last_line);

        let handle = std::thread::spawn(move || -> Result<(), String> {
            let lua = build_all_safe_vm();
            let session = DebugSession::new(breakpoints, cmd_rx, event_tx)
                .with_source_map(source_map, source_mode);

            let handler = move |lua: &Lua, debug: &Debug| {
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

    /// Start a `StepHost` like [`start_step_host_with_map`] but thread a SHARED
    /// effective mode ([`SharedSourceMode`]) into the session via
    /// [`with_shared_mode`](DebugSession::with_shared_mode) (task 5.5). The map is
    /// ALWAYS threaded; the EFFECTIVE mode is the shared cell, so a test (standing
    /// in for the socket bridge applying a DAP `attach` `sourcePresentation`) can
    /// flip the returned [`SharedSourceMode`] to switch `.pasta`↔`.lua` step
    /// granularity for the running session. Returns `(host, shared_mode)`.
    fn start_step_host_with_shared_mode(
        breakpoints: BreakpointSet,
        chunk: &'static str,
        source: &'static str,
        source_map: Option<Arc<SourceMap>>,
        initial_mode: SourceMode,
    ) -> (StepHost, crate::debug::SharedSourceMode) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

        let shared_mode = crate::debug::SharedSourceMode::new(initial_mode);
        let session_shared = shared_mode.clone();

        let last_line = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let hook_last_line = Arc::clone(&last_line);

        let handle = std::thread::spawn(move || -> Result<(), String> {
            let lua = build_all_safe_vm();
            // `source_mode` (baked) is set to the initial mode too; the EFFECTIVE
            // mode read by the stepper is the shared cell (so an attach flip wins).
            let session = DebugSession::new(breakpoints, cmd_rx, event_tx)
                .with_source_map(source_map, initial_mode)
                .with_shared_mode(Some(session_shared));

            let handler = move |lua: &Lua, debug: &Debug| {
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

        (
            StepHost {
                cmd_tx,
                event_rx,
                last_line,
                handle: Some(handle),
            },
            shared_mode,
        )
    }

    /// 5.5 / 6.3 (attach forces `.pasta`): the resolved/baked mode starts at `Lua`,
    /// but a DAP `attach sourcePresentation="pasta"` flips the SHARED mode to
    /// `Pasta` BEFORE the VM runs. The stepper must then run at `.pasta`
    /// granularity: step over from line 6 (`.pasta` 10) consumes line 7 (same
    /// `.pasta` 10) + passes line 8 (unmapped), stopping at line 9 (`.pasta` 11) —
    /// NOT at line 7 (which `.lua` granularity would target).
    #[test]
    fn attach_pasta_switches_session_to_pasta_step_granularity() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[PASTA_BP_LINE]);
        // Server default/file mode is Lua; the map IS present (always threaded).
        let (mut host, shared_mode) = start_step_host_with_shared_mode(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Lua,
        );

        // attach sourcePresentation="pasta" applied (socket bridge writes shared).
        shared_mode.set(SourceMode::Pasta);

        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Breakpoint);
        assert_eq!(line, PASTA_BP_LINE, "must stop at the breakpoint line (6)");

        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step);
        assert_eq!(
            line, 9,
            "attach `pasta` must switch this session to `.pasta` step granularity: \
             consume line 7 (same `.pasta` 10), pass line 8 (unmapped), stop at \
             line 9 (`.pasta` 11) — NOT line 7 (5.5/6.3)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// 5.5 / 6.3 (attach forces `.lua`): the resolved/baked mode starts at `Pasta`,
    /// but a DAP `attach sourcePresentation="lua"` flips the SHARED mode to `Lua`
    /// BEFORE the VM runs. The stepper must then run at `.lua` granularity: step
    /// over from line 6 stops at line 7 (the next `.lua` line) — NOT line 9 (which
    /// `.pasta` granularity would target). attach > env/file precedence.
    #[test]
    fn attach_lua_switches_session_to_lua_step_granularity() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[PASTA_BP_LINE]);
        // Server default/file mode is Pasta (map present) → would be `.pasta`-granular.
        let (mut host, shared_mode) = start_step_host_with_shared_mode(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Pasta,
        );

        // attach sourcePresentation="lua" applied → flip to Lua.
        shared_mode.set(SourceMode::Lua);

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, PASTA_BP_LINE, "must stop at the breakpoint line (6)");

        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step);
        assert_eq!(
            line, 7,
            "attach `lua` must force `.lua` step granularity (stop at line 7), NOT \
             consume to the next `.pasta` line at 9 (5.5/6.3 — attach > env/file)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// 5.5 / design 581 (no attach override): with NO attach flip the session keeps
    /// the resolved env > file > 既定 mode. Baked `Pasta` + map → `.pasta`-granular
    /// stepping (stop at line 9), exactly as without any shared-mode plumbing.
    #[test]
    fn no_attach_keeps_resolved_pasta_step_granularity() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[PASTA_BP_LINE]);
        let (mut host, _shared_mode) = start_step_host_with_shared_mode(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Pasta,
        );
        // No flip: the env > file > 既定 resolved mode (Pasta) stands (design 581).

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, PASTA_BP_LINE, "must stop at the breakpoint line (6)");

        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step);
        assert_eq!(
            line, 9,
            "no attach override → resolved Pasta `.pasta` granularity (stop at 9)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// 9.1/E1 + 9.4/E6 (step over): stopped at `local a = 1` (line 6, `.pasta`
    /// 10), `Next` must CONSUME line 7 (also `.pasta` 10) and PASS line 8
    /// (unmapped), stopping at line 9 (`.pasta` 11) — the next DIFFERENT `.pasta`
    /// line — NOT at line 7 or 8.
    #[test]
    fn pasta_step_over_consumes_same_pasta_line_and_passes_unmapped() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[PASTA_BP_LINE]);
        let mut host = start_step_host_with_map(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Pasta,
        );

        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Breakpoint);
        assert_eq!(line, PASTA_BP_LINE, "must stop at the breakpoint line (6)");

        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "step over must stop with reason Step");
        assert_eq!(
            line, 9,
            "step over from `.pasta` 10 must consume line 7 (same `.pasta` 10) and \
             pass line 8 (unmapped), stopping at line 9 (`.pasta` 11) — the next \
             DIFFERENT `.pasta` line (9.1/9.4)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// 9.2/E3 + 9.4 (step into): stopped at `local d = helper(c)` (line 9),
    /// `StepIn` must enter `helper`, PASS the unmapped callee line 2, and stop at
    /// line 3 (`.pasta` 20) — the callee's first MAPPED `.pasta` line.
    #[test]
    fn pasta_step_into_stops_at_first_mapped_callee_line() {
        let breakpoints = BreakpointSet::new();
        // Breakpoint at the call line so we can step from there.
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[9]);
        let mut host = start_step_host_with_map(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Pasta,
        );

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, 9, "must stop at the call line (9)");

        host.cont(SessionCommand::StepIn);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "step into must stop with reason Step");
        assert_eq!(
            line, 3,
            "step into must PASS the unmapped callee line 2 and stop at line 3 \
             (`.pasta` 20) — the callee's first MAPPED `.pasta` line (9.2/9.4)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// 9.3/E4 (step out): step INTO `helper` (stop at line 3), then `StepOut` must
    /// return to the caller and stop at line 10 (`.pasta` 12) — the first MAPPED
    /// `.pasta` line in the caller after `helper` returns.
    #[test]
    fn pasta_step_out_stops_at_first_mapped_caller_line() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[9]);
        let mut host = start_step_host_with_map(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Pasta,
        );

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, 9, "must stop at the call line (9)");

        // Step into helper (stop at line 3, the first mapped callee line).
        host.cont(SessionCommand::StepIn);
        let (_reason, line) = host.recv_stop();
        assert_eq!(line, 3, "precondition: stepped into helper at line 3");

        // Step out: return to the caller, stop at the first mapped line (10).
        host.cont(SessionCommand::StepOut);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step, "step out must stop with reason Step");
        assert_eq!(
            line, 10,
            "step out must return to the caller and stop at line 10 (`.pasta` 12) — \
             the first MAPPED `.pasta` line after `helper` returns (9.3)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// E2/E5 (sub-call NOT entered on step over): a `helper` sub-call on the
    /// step-over line lives in a DEEPER frame; step over must not stop inside it.
    /// Stepping over line 9 (`.pasta` 11, which CALLS helper) lands at line 10
    /// (`.pasta` 12) in the SAME frame — NOT inside helper (lines 2/3/4).
    #[test]
    fn pasta_step_over_does_not_enter_sub_call() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[9]);
        let mut host = start_step_host_with_map(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Pasta,
        );

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, 9, "must stop at the call line (9)");

        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step);
        assert_eq!(
            line, 10,
            "step over the call line (9) must stop at line 10 in the SAME frame \
             (`.pasta` 12), NOT inside helper (E2 — sub-call not entered)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }

    /// 9.5 (`.lua` mode unchanged): with `SourceMode::Lua` AND a map present, the
    /// stepper must keep `.lua`-line granularity. Step over from line 6 stops at
    /// line 7 (the next `.lua` line) — NOT line 9 (which `.pasta` granularity
    /// would target). This guards that `.pasta` refinement is gated on the mode.
    #[test]
    fn lua_mode_keeps_lua_granularity_even_with_map() {
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[PASTA_BP_LINE]);
        let mut host = start_step_host_with_map(
            breakpoints,
            PASTA_CHUNK,
            PASTA_SOURCE,
            Some(pasta_scenario_map()),
            SourceMode::Lua,
        );

        let (_reason, line) = host.recv_stop();
        assert_eq!(line, PASTA_BP_LINE, "must stop at the breakpoint line (6)");

        host.cont(SessionCommand::Next);
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Step);
        assert_eq!(
            line, 7,
            "`.lua` mode must step at `.lua` granularity (stop at line 7), NOT \
             consume to the next `.pasta` line (9.5)"
        );

        host.cont(SessionCommand::Continue);
        host.join();
    }
}
