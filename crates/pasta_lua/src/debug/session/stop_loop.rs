//! `DebugSession` stop core and per-line hook entry: `(source, line)` extraction,
//! the UNBOUNDED STOP LOOP (inspect/step/continue/disconnect command routing on
//! the VM thread), the per-line `on_line_impl` decision, and the [`LineHook`]
//! plug (design "DebugSession（停止状態機械）" / "無期限ブロックが正"; requirements
//! 1.1 / 1.2 / 1.6 / 2.1 / 2.2 / 3.2 / 3.4 / 9.1–9.5). Split out of the `session`
//! hub (C5 production split) — child of `session`, so it reaches the
//! `DebugSession` private state via the ancestor rule (no visibility widening).

use mlua::{Debug, Lua, VmState};

use crate::debug::SourceMode;
use crate::debug::hook::LineHook;
use crate::debug::inspect::{capture_stack, capture_variables};
use crate::debug::types::{
    Scope, SessionCommand, SessionEvent, StopReason, ThreadInfo,
};

use super::{DebugSession, MAIN_THREAD_ID, RunMode, StepKind};

impl DebugSession {
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
    pub(super) fn report_error(&self, err: &mlua::Error) {
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
