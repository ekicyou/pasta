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
//! # Implemented scope (tasks 2.2 / 2.5 / 4.1 / 5.4 / 5.5)
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
//!
//! In [`SourceMode::Pasta`] with a [`SourceMap`], stops are further refined to
//! `.pasta` granularity: a step consumes all `.lua` lines mapping to the SAME
//! `.pasta` line (task 5.4, [`RunMode::Stepping`]'s `origin_pasta`), and
//! breakpoint re-hits on the anchored `.pasta` line are coalesced (the
//! `pasta_break_anchor` field). The EFFECTIVE present mode follows the shared
//! cell when threaded, so a DAP `attach` `sourcePresentation` flip switches the
//! granularity live (task 5.5).

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
    /// `.lua` granularity (requirements 6.1 / 6.2). Consumed (task 5.4 / break
    /// anchor) via [`resolve_current_pasta`](Self::resolve_current_pasta) by the
    /// `.pasta`-granular stepper and the breakpoint coalescing in
    /// [`on_line_impl`](Self::on_line_impl). `Arc` is the immutable shared form
    /// (design "Architecture": `Arc<SourceMap>` 不変共有).
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
    /// The `.pasta` line of the MOST RECENT stop, used to coalesce the breakpoint
    /// re-hits a single `.pasta` line produces across its many mapped `.lua` lines
    /// (design "State Management"; requirements 1.1 / 2.1 / 2.2 / 2.4). `None` =
    /// no anchor (no recent stop / left the anchored line); `Some(p)` = stopped on
    /// (or resuming from, not yet left) `.pasta` position `p`.
    ///
    /// Interior-mutable via `RefCell` with the SAME thread discipline as `mode`:
    /// the line hook calls `&self` and the session is single-threaded (VM thread),
    /// so this is race-free without a lock. Maintained / referenced ONLY in
    /// [`SourceMode::Pasta`] with a `source_map` (task 2 integration); the
    /// `with_source_map` / `with_shared_mode` injection helpers do NOT touch it.
    pasta_break_anchor: std::cell::RefCell<Option<PastaPos>>,
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
            // No anchor at construction (design "State Management": 初期値 = None).
            pasta_break_anchor: std::cell::RefCell::new(None),
        }
    }

    /// Thread the (optional) shared source map and present mode into the session
    /// (task 4.2 injection point: `enable → wiring → DebugSession`, design 548).
    ///
    /// It STORES the map+mode read by the `.pasta`-granular stepper and the
    /// breakpoint coalescing (task 5.4 / break-anchor integration).
    /// The gating decision lives in [`enable`](crate::debug::enable):
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

    /// The threaded shared source map, if any (task 4.2 plumbing observation /
    /// task 5.4 stepper consumer). `Some` only when `enable` was given a map in
    /// [`SourceMode::Pasta`]; `None` for the default `.lua` behavior.
    ///
    /// Test-only observation of the injection path (no production caller).
    #[cfg(test)]
    pub(crate) fn source_map(&self) -> Option<&Arc<SourceMap>> {
        self.source_map.as_ref()
    }

    /// The EFFECTIVE present mode (requirements 6.1 / 6.2 / 6.3). Reads the shared
    /// cell when threaded (so a DAP `attach` flip is reflected, task 5.5), else
    /// the baked resolved mode. Default [`SourceMode::Pasta`] until `enable`
    /// threads the resolved mode.
    ///
    /// Test-only observation (production code reads `effective_mode` directly).
    #[cfg(test)]
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

    /// Advance the `.pasta` break ANCHOR by one line and report whether the
    /// current line is suppression-eligible (design "State Management" 175-178,
    /// "System Flows → アンカーのライフサイクル" 114-122; requirements 1.1 / 2.1 /
    /// 2.2 / 2.3).
    ///
    /// The return value is **suppression-eligibility**: `true` IFF the current
    /// line sits on the SAME `.pasta` line the session last stopped on (the
    /// anchor), so a breakpoint hit here should be CONSUMED rather than re-stop
    /// (this is the `anchor == cur` test — same invariant as
    /// [`pasta_step_should_stop`](Self::pasta_step_should_stop)'s `origin_pasta ==
    /// Some(cur)`). Transitions over `(anchor, cur)`:
    ///
    /// - `(Some(a), Some(a))` — same `.pasta` line → `true`, anchor UNCHANGED.
    /// - `(Some(a), Some(b))`, `b != a` — moved to a DIFFERENT mapped `.pasta`
    ///   line → CLEAR the anchor to `None`, `false` (2.2: leaving the line; the
    ///   next re-visit re-stops because the anchor is gone).
    /// - `(_, None)` — the current `.lua` line is `.pasta`-unmapped → `false`,
    ///   anchor UNCHANGED (2.1: an unmapped line within the SAME `.pasta` line's
    ///   expansion must NOT falsely clear the anchor).
    /// - `(None, _)` — no anchor → `false`, anchor UNCHANGED.
    ///
    /// The ONLY side effect is clearing on a move to a different `.pasta` line.
    /// ESTABLISHING the anchor (`*anchor = Some(cur)`) is the CALLER's job at stop
    /// time (design 178), NOT done here.
    fn update_break_anchor(&self, cur: Option<&PastaPos>) -> bool {
        let mut anchor = self.pasta_break_anchor.borrow_mut();
        match (anchor.as_ref(), cur) {
            // Same `.pasta` line as the anchor: suppression-eligible, anchor kept.
            (Some(a), Some(c)) if a == c => true,
            // Moved to a DIFFERENT mapped `.pasta` line: clear (left the line).
            (Some(_), Some(_)) => {
                *anchor = None;
                false
            }
            // Unmapped line (`cur == None`): keep the anchor, not eligible (2.1).
            // No anchor (`anchor == None`): nothing to suppress.
            _ => false,
        }
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
                // `frame_id` is EXTERNAL input crossing the stop-loop trust
                // boundary: saturate the +1 so a hostile/buggy client sending
                // u32::MAX cannot panic the VM thread (debug-build overflow) —
                // mirrors the Variables decode's `saturating_sub` (cell 3.28 G3).
                Ok(SessionCommand::Scopes { frame_id }) => {
                    let scopes = vec![Scope {
                        name: "Locals".to_string(),
                        variables_reference: frame_id.saturating_add(1),
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

                // RefreshPresentation (requirement 3.3): a `pasta/sourcePresentation`
                // toggle arrived while paused. RE-SEND the CURRENT stop reusing the
                // in-scope `reason`/`thread_id` — NO new snapshot state — so the
                // client re-fetches the stack and re-renders under the (already
                // swapped) present resolver. KEEP BLOCKING: this does NOT resume and
                // does NOT change `RunMode`, so step granularity continues to follow
                // `effective_mode()` per line (requirement 5.3, satisfied by the
                // existing per-line read; no extra logic here). Only ever drained
                // here in `stop_loop`, so "ignore while running" is automatic.
                Ok(SessionCommand::RefreshPresentation) => {
                    let _ = self.event_tx.send(SessionEvent::Stopped { reason, thread_id });
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
    /// boundary as a raw error (helper `report_error`, test-only until a
    /// production consumer appears).
    fn on_line_impl(&self, lua: &Lua, debug: &Debug) -> mlua::Result<VmState> {
        let (source, line) = Self::source_and_line(debug);

        // `.pasta` break-anchor processing (task 2 / design §State Management
        // 183-191). ONLY active in `SourceMode::Pasta` WITH a `source_map` — the
        // single gate that keeps `.lua` mode / no-map / OFF byte-identical to
        // before (4.1, 4.2, 4.3). Computed ONCE per line so `update_break_anchor`
        // runs EVERY pasta+map line (this is what guarantees the anchor is CLEARED
        // when execution leaves the anchored `.pasta` line). `should_pause` is
        // evaluated only once below — `cur`/`suppress` are computed first.
        // `pasta`/`cur` are ALSO the single per-line reads reused by the stepping
        // refinement further down (no second gate/resolution for the same line).
        let pasta = self.effective_mode() == SourceMode::Pasta && self.source_map.is_some();
        let cur = if pasta {
            self.resolve_current_pasta(&source, line)
        } else {
            None
        };
        let suppress = if pasta {
            self.update_break_anchor(cur.as_ref())
        } else {
            false
        };

        // Breakpoint-first: a breakpoint ALWAYS stops (reason Breakpoint), even
        // while Stepping — stepping must not mask breakpoints.
        if self.breakpoints.should_pause(&source, line) {
            // Suppression-eligible (the current line is on the SAME `.pasta` line
            // the session last stopped on): CONSUME the re-hit — no additional
            // Stopped event (1.1, 3.2). Returning here escapes the breakpoint-first
            // branch exactly as a non-matching line would.
            if suppress {
                return Ok(VmState::Continue);
            }
            // Otherwise this is a genuine stop on a NEW `.pasta` line: ESTABLISH
            // the anchor at the stop point (the caller's job per design 178) so the
            // remaining `.lua` lines of THIS `.pasta` line are consumed on the next
            // Continue. In `.lua` mode / no map (`cur == None`) the anchor is NOT
            // set → `.lua`-granularity stop (design §State Management 179).
            if let Some(p) = cur {
                *self.pasta_break_anchor.borrow_mut() = Some(p);
            }
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
                // (b) In `.pasta` mode (a map is present — the `pasta` gate from
                //     line entry), REFINE the `.lua` stop to `.pasta` granularity
                //     (9.1–9.4) using the SAME `cur` resolved at line entry (RAW
                //     source through the map, which canonicalizes the chunk
                //     internally — 3.4). `cur` is `None` for `.lua` mode / no map,
                //     so the refinement is skipped and the `.lua` stop is taken
                //     as-is (9.5).
                let take_stop = if pasta {
                    Self::pasta_step_should_stop(
                        thread,
                        base_depth,
                        origin_pasta.as_ref(),
                        cur_thread,
                        depth,
                        cur.as_ref(),
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
    /// No production caller has materialized (the stop path stayed infallible),
    /// so the seam is compiled for tests only until a real consumer appears.
    #[cfg(test)]
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

// Inline `#[cfg(test)] mod tests` was externalized into logical-cluster sibling
// files (Task 2.1, pure behavior-invariant move). Each sibling begins with
// `use super::*;` and keeps the same module path, preserving private/`pub(crate)`
// reachability into this production module. Cross-cluster-shared test helpers
// live in `session_test_support` (`pub(super)`); each cluster `use`s them. The
// set of leaf test-fn names and the total test count are unchanged.
// Test-only helper METHODS on production types above remain in place (they were
// never inside the `mod tests` block).
#[cfg(test)]
#[path = "session_test_support.rs"]
mod session_test_support;

#[cfg(test)]
#[path = "session_injection_tests.rs"]
mod session_injection_tests;

#[cfg(test)]
#[path = "session_step_controller_tests.rs"]
mod session_step_controller_tests;

#[cfg(test)]
#[path = "session_pasta_step_tests.rs"]
mod session_pasta_step_tests;

#[cfg(test)]
#[path = "session_anchor_tests.rs"]
mod session_anchor_tests;

#[cfg(test)]
#[path = "session_hook_integration_tests.rs"]
mod session_hook_integration_tests;

#[cfg(test)]
#[path = "session_stop_loop_tests.rs"]
mod session_stop_loop_tests;
