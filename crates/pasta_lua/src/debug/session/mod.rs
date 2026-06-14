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

use crate::debug::SourceMode;
use crate::debug::breakpoints::BreakpointSet;
use crate::debug::source_map::{PastaPos, SourceMap};
use crate::debug::types::{SessionCommand, SessionEvent};

// Responsibility submodules (C5 production split). The primary types
// (`DebugSession` / `RunMode` / `StepKind`), the constructor + injection seam,
// and the `#[path]` test decls stay in this hub; the method-body `impl
// DebugSession` blocks live in the children below. Each child is a child module
// of `session`, so it reaches the `DebugSession` private state via Rust's
// ancestor-private rule — NO visibility widening of any field.
mod anchor;
mod stepping;
mod stop_loop;

// The externalized `#[cfg(test)]` session test clusters resolve their referenced
// production types through `use super::*;` (this `session` hub). In the original
// flat `session.rs` these names were brought into module scope by the production
// `use` statements that now live in the child submodules; re-introduce them
// here, test-gated, so the test glob keeps resolving them WITHOUT adding any
// non-test import or widening the public surface (C5 test re-wiring; mirrors the
// dap hub).
#[cfg(test)]
use mlua::Debug;
#[cfg(test)]
use crate::debug::hook::LineHook;
#[cfg(test)]
use crate::debug::types::ThreadId;

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
#[path = "../session_test_support.rs"]
mod session_test_support;

#[cfg(test)]
#[path = "../session_injection_tests.rs"]
mod session_injection_tests;

#[cfg(test)]
#[path = "../session_step_controller_tests.rs"]
mod session_step_controller_tests;

#[cfg(test)]
#[path = "../session_pasta_step_tests.rs"]
mod session_pasta_step_tests;

#[cfg(test)]
#[path = "../session_anchor_tests.rs"]
mod session_anchor_tests;

#[cfg(test)]
#[path = "../session_hook_integration_tests.rs"]
mod session_hook_integration_tests;

#[cfg(test)]
#[path = "../session_stop_loop_tests.rs"]
mod session_stop_loop_tests;
