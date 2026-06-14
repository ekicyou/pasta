//! Inline test cluster externalized from `session.rs` (Task 2.1, pure move).
//! Cluster: stop-loop inspect-command routing, live SetBreakpoints, controller-disconnect.
use super::*;
use super::session_test_support::*;

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::source_map::ChunkSourceMap;
use crate::debug::types::{SourceRef, StopReason};
use std::collections::BTreeMap;

// =======================================================================
// Cell 3.27 (G1) — stop-loop inspect-command ROUTING, live SetBreakpoints,
// controller-disconnect release, and pure StepController decision pins.
//
// The capture functions themselves are tested in `inspect.rs`; these tests
// pin the SESSION's responsibilities: each inspect command handled in the
// stop loop emits its RESULT EVENT on the VM thread and KEEPS BLOCKING
// (R2.1 / R2.2 / R3.3), `SetBreakpoints` mutates the shared store live,
// and a vanished controller (`recv()` Err) releases the VM without a
// `Terminated` event (design "Error Handling" fallback).
// =======================================================================

/// R2.1 / R3.3: `StackTrace` while stopped emits a `SessionEvent::Stack`
/// whose top frame is the stopped line, and does NOT resume execution.
#[test]
fn stack_trace_while_stopped_emits_stack_event_and_keeps_blocking() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);
    let mut host = start_session(breakpoints);

    host.event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit Stopped");
    let at_stop = host.progress();

    host.cmd_tx
        .send(SessionCommand::StackTrace)
        .expect("sending StackTrace must succeed");

    let event = host
        .event_rx
        .recv_timeout(WATCHDOG)
        .expect("StackTrace while stopped must emit a result event (R2.1)");
    match event {
        SessionEvent::Stack(frames) => {
            assert!(
                !frames.is_empty(),
                "the captured stack must contain at least the stopped frame"
            );
            assert_eq!(
                frames[0].line, BREAKPOINT_LINE,
                "the TOP frame must report the stopped line ({BREAKPOINT_LINE})"
            );
            assert!(
                frames[0].source.contains("session_scenario"),
                "the top frame source must be the scenario chunk: {:?}",
                frames[0].source
            );
        }
        other => panic!("expected SessionEvent::Stack, got {other:?}"),
    }

    // The capture ran ON the VM thread INSIDE the stop loop: still paused.
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        host.progress(),
        at_stop,
        "StackTrace must KEEP BLOCKING (no resume) after emitting Stack"
    );

    host.cmd_tx
        .send(SessionCommand::Continue)
        .expect("sending Continue must succeed");
    join_with_watchdog(&mut host, WATCHDOG)
        .expect("host thread must finish after Continue")
        .expect("host thread must not panic")
        .expect("scenario must complete after Continue");
}

/// R2.2 / R3.3: `Variables` while stopped emits a `SessionEvent::Variables`
/// carrying the stopped frame's locals BY NAME. Also pins the DapAdapter
/// numbering decode `frame_level = var_ref - 1` AND its saturating edge
/// (`var_ref = 0` must not underflow — it decodes to frame 0 too).
#[test]
fn variables_while_stopped_emits_locals_and_var_ref_zero_saturates() {
    let breakpoints = BreakpointSet::new();
    // Stop at line 3: locals `a` (line 1) and `b` (line 2) are in scope.
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);
    let mut host = start_session(breakpoints);

    host.event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit Stopped");

    let recv_vars = |host: &HostThread| -> Vec<crate::debug::types::Variable> {
        match host
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("Variables while stopped must emit a result event (R2.2)")
        {
            SessionEvent::Variables(vars) => vars,
            other => panic!("expected SessionEvent::Variables, got {other:?}"),
        }
    };

    // var_ref = 1 → frame_level 0 (the stopped main-chunk frame).
    host.cmd_tx
        .send(SessionCommand::Variables { var_ref: 1 })
        .expect("sending Variables must succeed");
    let vars = recv_vars(&host);
    for name in ["a", "b"] {
        assert!(
            vars.iter().any(|v| v.name == name),
            "frame-0 locals must contain `{name}` at the line-3 stop \
             (R2.2); got {vars:?}"
        );
    }

    // var_ref = 0 → saturating_sub → ALSO frame_level 0 (no underflow).
    host.cmd_tx
        .send(SessionCommand::Variables { var_ref: 0 })
        .expect("sending Variables must succeed");
    let vars0 = recv_vars(&host);
    assert!(
        vars0.iter().any(|v| v.name == "a"),
        "var_ref = 0 must saturate to frame 0 (NOT underflow); got {vars0:?}"
    );

    host.cmd_tx
        .send(SessionCommand::Continue)
        .expect("sending Continue must succeed");
    join_with_watchdog(&mut host, WATCHDOG)
        .expect("host thread must finish after Continue")
        .expect("host thread must not panic")
        .expect("scenario must complete after Continue");
}

/// `Scopes` while stopped emits exactly ONE `Locals` scope whose
/// `variables_reference` follows the DapAdapter numbering
/// `frame_id + 1` (the inverse of the Variables decode above).
#[test]
fn scopes_while_stopped_emits_single_locals_scope_with_frame_numbering() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);
    let mut host = start_session(breakpoints);

    host.event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit Stopped");

    host.cmd_tx
        .send(SessionCommand::Scopes { frame_id: 4 })
        .expect("sending Scopes must succeed");

    match host
        .event_rx
        .recv_timeout(WATCHDOG)
        .expect("Scopes while stopped must emit a result event")
    {
        SessionEvent::Scopes(scopes) => {
            assert_eq!(scopes.len(), 1, "exactly one (Locals) scope: {scopes:?}");
            assert_eq!(scopes[0].name, "Locals");
            assert_eq!(
                scopes[0].variables_reference, 5,
                "variables_reference must be frame_id + 1 (DapAdapter numbering)"
            );
        }
        other => panic!("expected SessionEvent::Scopes, got {other:?}"),
    }

    host.cmd_tx
        .send(SessionCommand::Continue)
        .expect("sending Continue must succeed");
    join_with_watchdog(&mut host, WATCHDOG)
        .expect("host thread must finish after Continue")
        .expect("host thread must not panic")
        .expect("scenario must complete after Continue");
}

/// Cell 3.28 G3 hardening boundary: `Scopes { frame_id: u32::MAX }` — an
/// attacker/buggy-client value crossing the stop-loop trust boundary — must
/// NOT panic the VM thread (pre-fix: `frame_id + 1` overflows and panics in
/// debug builds). The numbering saturates (`variables_reference =
/// u32::MAX`), the session keeps blocking, and Continue still resumes.
#[test]
fn scopes_with_max_frame_id_saturates_instead_of_overflowing() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);
    let mut host = start_session(breakpoints);

    host.event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit Stopped");

    // External-input edge: the maximum representable frame_id.
    host.cmd_tx
        .send(SessionCommand::Scopes { frame_id: u32::MAX })
        .expect("sending Scopes must succeed");

    match host
        .event_rx
        .recv_timeout(WATCHDOG)
        .expect("Scopes with frame_id = u32::MAX must emit a result, not panic")
    {
        SessionEvent::Scopes(scopes) => {
            assert_eq!(scopes.len(), 1, "exactly one (Locals) scope: {scopes:?}");
            assert_eq!(
                scopes[0].variables_reference,
                u32::MAX,
                "the frame_id + 1 numbering must SATURATE at u32::MAX \
                 (no overflow panic on the trust boundary)"
            );
        }
        other => panic!("expected SessionEvent::Scopes, got {other:?}"),
    }

    // The VM thread survived and is still paused; Continue resumes normally.
    host.cmd_tx
        .send(SessionCommand::Continue)
        .expect("sending Continue must succeed");
    join_with_watchdog(&mut host, WATCHDOG)
        .expect("host thread must finish after Continue")
        .expect("host thread must not panic on a u32::MAX frame_id")
        .expect("scenario must complete after Continue");
}

/// R3.3: `Threads` while stopped emits the single fixed main-thread
/// descriptor (`id = MAIN_THREAD_ID`, name "main").
#[test]
fn threads_while_stopped_emits_single_main_thread() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);
    let mut host = start_session(breakpoints);

    host.event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit Stopped");

    host.cmd_tx
        .send(SessionCommand::Threads)
        .expect("sending Threads must succeed");

    match host
        .event_rx
        .recv_timeout(WATCHDOG)
        .expect("Threads while stopped must emit a result event")
    {
        SessionEvent::Threads(threads) => {
            assert_eq!(threads.len(), 1, "exactly one thread descriptor");
            assert_eq!(threads[0].id, MAIN_THREAD_ID);
            assert_eq!(threads[0].name, "main");
        }
        other => panic!("expected SessionEvent::Threads, got {other:?}"),
    }

    host.cmd_tx
        .send(SessionCommand::Continue)
        .expect("sending Continue must succeed");
    join_with_watchdog(&mut host, WATCHDOG)
        .expect("host thread must finish after Continue")
        .expect("host thread must not panic")
        .expect("scenario must complete after Continue");
}

/// Design "System Flows" (`Arc<Mutex>` 共有): `SetBreakpoints` sent WHILE
/// stopped is applied LIVE to the shared store, keeps blocking (no event,
/// no resume), and the just-set breakpoint is observed after Continue.
/// The replacement also REMOVES the old line-2 breakpoint, so the next stop
/// is line 5 — proving replace semantics through the stop loop.
#[test]
fn set_breakpoints_while_stopped_applies_live_and_keeps_blocking() {
    const LIVE_SOURCE: &str = "@live_bp_scenario";
    const LIVE_CHUNK: &str = "\
local a = 1
local b = a + 1
local c = b + 1
local d = c + 1
local e = d + 1
return e
";
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(LIVE_SOURCE), &[2]);
    let mut host = start_step_host(breakpoints, LIVE_CHUNK, LIVE_SOURCE);

    let (reason, line) = host.recv_stop();
    assert_eq!(reason, StopReason::Breakpoint);
    assert_eq!(line, 2, "must stop at the initial breakpoint (line 2)");

    // Replace the set for this source while stopped: {2} -> {5}.
    host.cont(SessionCommand::SetBreakpoints {
        source: SourceRef::new(LIVE_SOURCE),
        lines: vec![5],
    });

    // KEEP BLOCKING: no event and no resume — were the session to resume,
    // it would hit line 5 and emit a Stopped within this window.
    let quiet = host.event_rx.recv_timeout(Duration::from_millis(150));
    assert!(
        matches!(quiet, Err(RecvTimeoutError::Timeout)),
        "SetBreakpoints must apply silently and keep blocking; got {quiet:?}"
    );

    // Continue: the LIVE set is observed — next stop is line 5 (the new
    // breakpoint), NOT a re-stop on line 2 (replaced away).
    host.cont(SessionCommand::Continue);
    let (reason, line) = host.recv_stop();
    assert_eq!(
        reason,
        StopReason::Breakpoint,
        "the live-set breakpoint must stop with reason Breakpoint"
    );
    assert_eq!(
        line, 5,
        "after Continue the session must stop at the LIVE-set line 5 \
         (old line-2 breakpoint replaced away)"
    );

    host.cont(SessionCommand::Continue);
    host.join();
}

/// Design "Error Handling" fallback: when the CONTROLLER VANISHES while the
/// session is stopped (all `cmd_tx` ends dropped → `recv()` Err), the stop
/// loop must release the VM so the host completes — WITHOUT emitting a
/// `Terminated` event (that is the explicit-`Disconnect` path only).
#[test]
fn controller_drop_while_stopped_releases_vm_without_terminated() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);
    let host = start_session(breakpoints);

    host.event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit Stopped");

    // Drop the ONLY command sender: the blocked `recv()` returns Err.
    let HostThread {
        cmd_tx,
        event_rx,
        progress: _,
        mut handle,
    } = host;
    drop(cmd_tx);

    // The host thread must run to completion (VM released, no hang).
    let join_handle = handle.take().expect("handle present");
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(join_handle.join());
    });
    done_rx
        .recv_timeout(WATCHDOG)
        .expect("host thread must finish after the controller is dropped")
        .expect("host thread must not panic")
        .expect("scenario must run to completion after the controller drop");

    // NO Terminated event on this path: the channel is now closed with
    // nothing buffered (Terminated belongs to the explicit Disconnect path).
    let next = event_rx.recv_timeout(Duration::from_millis(0));
    assert!(
        next.is_err(),
        "a vanished controller must NOT produce a Terminated event; got {next:?}"
    );
}

// ----------------------------------------------------------------------
// Pure StepController decision pins (`step_should_stop`). The behaviours
// are e2e-proven above; these unit pins make the formula mutation-
// resistant per kind/branch without a VM.
// ----------------------------------------------------------------------

/// Thread mismatch (host loop / another coroutine) SKIPS for EVERY kind,
/// even when depth/line would otherwise be a completion point.
#[test]
fn step_should_stop_skips_thread_mismatch_for_all_kinds() {
    let target = ThreadId(0x10);
    let other = ThreadId(0x20);
    for kind in [StepKind::Over, StepKind::In, StepKind::Out] {
        assert!(
            !DebugSession::step_should_stop(kind, target, 3, 6, other, 2, 7),
            "{kind:?}: a line on a DIFFERENT thread must be skipped"
        );
    }
}

/// Over: deeper frame (callee body) skipped; same frame same line skipped;
/// same frame changed line stops; shallower frame (returned) stops.
#[test]
fn step_over_decision_per_depth_and_line() {
    let t = ThreadId(0x10);
    let f = |depth, line| DebugSession::step_should_stop(StepKind::Over, t, 3, 6, t, depth, line);
    assert!(!f(4, 2), "deeper frame (callee) must be skipped");
    assert!(!f(3, 6), "same frame, same line must not re-trigger");
    assert!(f(3, 7), "same frame, changed line is the completion point");
    assert!(f(2, 6), "shallower frame (returned to caller) stops");
}

/// In: a deeper frame stops (entered the callee); a changed line in the
/// same frame stops; the unchanged start line does not.
#[test]
fn step_in_decision_per_depth_and_line() {
    let t = ThreadId(0x10);
    let f = |depth, line| DebugSession::step_should_stop(StepKind::In, t, 3, 6, t, depth, line);
    assert!(f(4, 6), "deeper frame stops even on the same source line");
    assert!(f(3, 7), "changed line in the same frame stops");
    assert!(!f(3, 6), "the unchanged start line must not stop");
}

/// Out: ONLY a shallower frame stops; same or deeper frames are skipped
/// regardless of the line.
#[test]
fn step_out_decision_per_depth() {
    let t = ThreadId(0x10);
    let f = |depth, line| DebugSession::step_should_stop(StepKind::Out, t, 3, 6, t, depth, line);
    assert!(f(2, 6), "shallower frame (returned to caller) stops");
    assert!(!f(3, 7), "same frame must be skipped even on a changed line");
    assert!(!f(4, 9), "deeper frame must be skipped");
}

// ----------------------------------------------------------------------
// Effective-mode observation and `.pasta` resolution gating (unit, no VM).
// ----------------------------------------------------------------------

/// 6.3 / task 5.5: `source_mode()` reads the SHARED cell when threaded —
/// a flip (the socket bridge applying an `attach` override) is observed
/// immediately, overriding the baked resolved mode in BOTH directions.
#[test]
fn source_mode_observes_shared_cell_flip_over_baked_mode() {
    let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
    let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();
    let shared = crate::debug::SharedSourceMode::new(SourceMode::Pasta);
    // Baked mode Lua; shared cell Pasta — the shared cell must win.
    let session = DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx)
        .with_source_map(None, SourceMode::Lua)
        .with_shared_mode(Some(shared.clone()));

    assert_eq!(
        session.source_mode(),
        SourceMode::Pasta,
        "the shared cell must override the baked mode (attach precedence)"
    );
    shared.set(SourceMode::Lua);
    assert_eq!(
        session.source_mode(),
        SourceMode::Lua,
        "a shared-cell flip must be observed immediately by source_mode()"
    );
}

/// 9.5 gating: `resolve_current_pasta` returns `Some` ONLY in `Pasta` mode
/// WITH a map AND a mapped line — `Lua` mode with the SAME map, a missing
/// map, and an unmapped line each resolve to `None`.
#[test]
fn resolve_current_pasta_requires_pasta_mode_map_and_mapped_line() {
    let build_map = || {
        let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
        forward.insert(20, ppos(10));
        let mut sm = SourceMap::new();
        sm.insert_chunk(
            PASTA_SOURCE.to_string(),
            "scene.pasta".to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        Arc::new(sm)
    };
    let session_with = |map: Option<Arc<SourceMap>>, mode: SourceMode| {
        let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();
        DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx).with_source_map(map, mode)
    };

    // Pasta + map + mapped line → Some (the only resolving combination).
    let resolving = session_with(Some(build_map()), SourceMode::Pasta);
    assert_eq!(
        resolving.resolve_current_pasta(PASTA_SOURCE, 20),
        Some(ppos(10)),
        "Pasta mode + map + mapped line must resolve"
    );
    // Pasta + map + UNMAPPED line → None (9.4 pass-through input).
    assert_eq!(
        resolving.resolve_current_pasta(PASTA_SOURCE, 99),
        None,
        "an unmapped line must resolve to None even in Pasta mode with a map"
    );
    // Lua mode with the SAME map → None (mode gate, 9.5).
    let lua_gated = session_with(Some(build_map()), SourceMode::Lua);
    assert_eq!(
        lua_gated.resolve_current_pasta(PASTA_SOURCE, 20),
        None,
        "Lua mode must gate `.pasta` resolution OFF even with a map (9.5)"
    );
    // Pasta mode with NO map → None (map gate).
    let mapless = session_with(None, SourceMode::Pasta);
    assert_eq!(
        mapless.resolve_current_pasta(PASTA_SOURCE, 20),
        None,
        "Pasta mode without a map must resolve to None"
    );
}
