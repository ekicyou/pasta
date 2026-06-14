//! Inline test cluster externalized from `session.rs` (Task 2.1, pure move).
//! Cluster: `.pasta`-granular stepping (requirements 9.1-9.5).
use super::*;
use super::session_test_support::*;

use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;

use mlua::Lua;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::types::{SourceRef, StopReason};

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
