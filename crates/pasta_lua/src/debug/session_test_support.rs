//! Shared test-support helpers externalized from `session.rs` (Task 2.1,
//! pure behavior-invariant move). These were cluster-shared helpers inside
//! the original single `#[cfg(test)] mod tests`; they are gathered here and
//! `pub(super)`-exported so the sibling cluster modules can reach them via
//! `use super::session_test_support::*;`. The same module path is preserved,
//! so private / `pub(crate)` reachability into production code is unchanged.
use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use mlua::{Lua, LuaOptions, StdLib};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::source_map::ChunkSourceMap;
use crate::debug::types::StopReason;
use std::collections::BTreeMap;

// ---- injection / host-thread scenario helpers (from the injection cluster) ----
/// Controller-side watchdog (TEST-ONLY). The stop core stays UNBOUNDED;
/// this only keeps CI from hanging on a regression.
pub(super) const WATCHDOG: Duration = Duration::from_secs(10);

/// Source name and breakpoint line for the stop/continue scenario.
pub(super) const SCENARIO_SOURCE: &str = "@session_scenario";
pub(super) const BREAKPOINT_LINE: u32 = 3;

/// Build an ALL_SAFE VM (so `jit` exists and `debug` is excluded). The hook
/// `install` applies `jit.off()` itself. Promotes the PoC / hook-test VM.
pub(super) fn build_all_safe_vm() -> Lua {
    unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) }
}

/// A controller's view of a running host-thread session (test driver).
///
/// `mlua::Lua` is `!Send` and is NEVER held here — it lives entirely inside
/// the host thread. Only channel ends, the shared `Arc` progress counter, and
/// the join handle (returning a `Send` `Result<(), String>`) cross.
pub(super) struct HostThread {
    pub(super) cmd_tx: Sender<SessionCommand>,
    pub(super) event_rx: Receiver<SessionEvent>,
    /// Incremented once per executed line by a recording hook wrapper, so the
    /// controller can observe progress freeze (while stopped) and resume.
    pub(super) progress: Arc<AtomicUsize>,
    pub(super) handle: Option<JoinHandle<Result<(), String>>>,
}

impl HostThread {
    pub(super) fn progress(&self) -> usize {
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
pub(super) fn start_session(breakpoints: BreakpointSet) -> HostThread {
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
pub(super) fn run_host_thread(
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
pub(super) fn join_with_watchdog(
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

// ---- StepController host helpers (from the step-controller cluster) ----
/// A flexible step-test host: builds a VM, installs the hook wired to a
/// `DebugSession`, and runs a caller-supplied chunk under a caller-supplied
/// source name. The hook wrapper records the line seen just BEFORE each
/// `session.on_line` call into a shared `AtomicU32`, so when the controller
/// receives a `Stopped` event it can read the EXACT line the session
/// stopped on (the wrapper's write happens-before the session's `Stopped`
/// send, so the value is current by the time the controller observes it).
pub(super) struct StepHost {
    pub(super) cmd_tx: Sender<SessionCommand>,
    pub(super) event_rx: Receiver<SessionEvent>,
    /// Line the hook last entered `on_line` with (the stop line on a stop).
    pub(super) last_line: Arc<std::sync::atomic::AtomicU32>,
    pub(super) handle: Option<JoinHandle<Result<(), String>>>,
}

impl StepHost {
    /// Receive the next `Stopped` event (bounded by the test watchdog) and
    /// return `(reason, stop_line)` — `stop_line` read from `last_line`.
    pub(super) fn recv_stop(&self) -> (StopReason, u32) {
        match self
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("must receive a session event before the watchdog")
        {
            SessionEvent::Stopped { reason, .. } => {
                (reason, self.last_line.load(Ordering::SeqCst))
            }
            // A non-stop event is unexpected in these scenarios: fail loudly
            // (each arm diverges, so no loop is needed — clippy::never_loop).
            other => panic!("unexpected event while awaiting a stop: {other:?}"),
        }
    }

    pub(super) fn cont(&self, cmd: SessionCommand) {
        self.cmd_tx.send(cmd).expect("command send must succeed");
    }

    /// Bounded join: a stepping regression that hangs the VM thread is a
    /// test failure, not a suite-killer (the stop core stays unbounded).
    pub(super) fn join(&mut self) {
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
pub(super) fn start_step_host(
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

// ---- `.pasta` stepping scenario helpers (from the pasta-step cluster) ----
/// Build a `PastaPos` in a fixed `.pasta` file for these tests.
pub(super) fn ppos(line: u32) -> PastaPos {
    PastaPos {
        file: "scene.pasta".to_string(),
        line,
    }
}

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
pub(super) const PASTA_SOURCE: &str = "@pasta_step_scenario";
pub(super) const PASTA_CHUNK: &str = "\
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
pub(super) const PASTA_BP_LINE: u32 = 6;

/// Build the `SourceMap` for `PASTA_CHUNK` (keyed by the hook source name,
/// which the map canonicalizes internally — task 3.4).
pub(super) fn pasta_scenario_map() -> Arc<SourceMap> {
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
pub(super) fn start_step_host_with_map(
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
