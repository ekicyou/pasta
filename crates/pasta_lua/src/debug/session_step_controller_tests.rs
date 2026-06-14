//! Inline test cluster externalized from `session.rs` (Task 2.1, pure move).
//! Cluster: StepController (step over / into / out) integration tests.
use super::*;
use super::session_test_support::*;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::types::{SourceRef, StopReason};

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
