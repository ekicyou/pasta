//! Inline test cluster externalized from `session.rs` (Task 2.1, pure move).
//! Cluster: source-map injection plumbing (enable -> wiring -> DebugSession).
use super::*;
use super::session_test_support::*;

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::types::{SourceRef, StopReason};

// =======================================================================
// Task 4.2 — source map injection plumbing (enable → wiring → DebugSession).
//
// The session is the STEPPER consumer (task 5.4). These tests prove the
// injection path only: the gated `Arc<SourceMap>` + present mode REACH the
// session in `.pasta` mode, and are absent (default `.lua` behavior) for
// `None`/`Lua` (requirements 6.1 / 6.2 / 7.2). Stepping BEHAVIOR over the
// injected map is covered by the `.pasta` step granularity tests below.
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

/// 3.3 (停止中の即時再描画): `RefreshPresentation` while stopped RE-SENDS the
/// CURRENT stop (`Stopped` with the SAME `reason`/`thread_id`) so the client
/// re-fetches the stack and re-renders under the swapped resolver — WITHOUT
/// resuming. Mirrors `non_resuming_command_keeps_blocking_until_continue`:
/// the second `Stopped` arrives, the session stays paused (a later `Continue`
/// drives normal completion).
#[test]
fn refresh_presentation_resends_current_stop_and_keeps_paused() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(SCENARIO_SOURCE), &[BREAKPOINT_LINE]);

    let mut host = start_session(breakpoints);

    // (1) Reach the breakpoint and capture the FIRST Stopped (R3.4).
    let first = host
        .event_rx
        .recv_timeout(WATCHDOG)
        .expect("must reach the breakpoint and emit the first Stopped");
    assert_eq!(
        first,
        SessionEvent::Stopped {
            reason: StopReason::Breakpoint,
            thread_id: MAIN_THREAD_ID,
        },
        "the first Stopped must report Breakpoint on the main thread id"
    );

    let at_stop = host.progress();

    // (2) Send RefreshPresentation while stopped (3.3): the session must
    // RE-SEND the SAME stop, reusing the in-scope reason/thread_id.
    host.cmd_tx
        .send(SessionCommand::RefreshPresentation)
        .expect("sending RefreshPresentation must succeed");

    let second = host
        .event_rx
        .recv_timeout(WATCHDOG)
        .expect("RefreshPresentation must RE-SEND a Stopped event (3.3)");
    assert_eq!(
        second, first,
        "the re-sent Stopped must carry the SAME reason and thread_id as the \
         original stop (3.3 — no new snapshot state, in-scope values reused)"
    );

    // (3) The session is STILL paused: it did NOT resume (progress frozen).
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        host.progress(),
        at_stop,
        "RefreshPresentation must NOT resume execution — the session stays paused"
    );

    // (4) Continue still drives normal completion (the re-send did not break
    // the resume path).
    host.cmd_tx
        .send(SessionCommand::Continue)
        .expect("sending Continue must succeed");
    let joined = join_with_watchdog(&mut host, WATCHDOG)
        .expect("host thread must finish after Continue following RefreshPresentation");
    joined
        .expect("host thread must not panic")
        .expect("scenario must complete after Continue");
    assert!(
        host.progress() > at_stop,
        "progress must advance after Continue following RefreshPresentation"
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
