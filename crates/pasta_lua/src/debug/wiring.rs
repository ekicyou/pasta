//! Backend wiring: the transport↔dap↔session↔hook bridge threads (task 4.1).
//!
//! This module owns the two `Send`-only bridge threads that connect the
//! I/O-side [`Transport`](crate::debug::transport::Transport) +
//! [`DapAdapter`](crate::debug::dap::DapAdapter) to the VM-side
//! [`DebugSession`](crate::debug::session::DebugSession). The VM-side hook
//! install and the session itself live on the VM thread (the thread that calls
//! [`enable`](crate::debug::enable)); this module never touches `mlua::Lua`
//! (it is `!Send` and stays pinned to the VM thread). Only `std::sync::mpsc`
//! channels carrying `Send` payloads ([`SessionCommand`] / [`SessionEvent`] /
//! [`serde_json::Value`]) cross between the threads here.
//!
//! # Thread topology (design "Architecture" / "System Flows" / スレッドモデル)
//!
//! Three concurrent roles plus the transport's own internal reader thread,
//! connected only by channels:
//!
//! 1. **VM host thread** (caller of `enable`, owns `mlua::Lua`): the line hook
//!    drives [`DebugSession::on_line`]; when stopped, the session processes
//!    inspect/step/continue commands IN the hook loop, on this thread, calling
//!    `inspect::capture_*` on `lua.current_thread()`. It reads the session's
//!    `cmd_rx` and writes the session's `event_tx`.
//! 2. **Socket bridge thread** ([`run_socket_bridge`]): the SOLE owner of the
//!    [`Transport`]. [`Transport`] is `!Sync` (it holds a
//!    `Receiver<Value>`), so it cannot be shared across threads — exactly one
//!    thread owns it and performs BOTH socket reads and socket writes. This
//!    thread multiplexes, per iteration:
//!    - **inbound** (socket → us): a bounded `recv_timeout` poll of
//!      `transport.inbound()`. Each decoded DAP request becomes (a) immediate
//!      response/event frames written straight back, (b) a `setBreakpoints`
//!      applied DIRECTLY to the shared [`BreakpointSet`] (settable while the VM
//!      is RUNNING — design "System Flows": `Arc<Mutex>` 共有) whose DAP response
//!      is produced via the adapter and written back, or (c) a stop-context
//!      [`SessionCommand`] forwarded to the session's `cmd_tx`.
//!    - **outbound** (session → socket): drains the `out_rx` frame channel fed
//!      by the encoder thread and writes each frame to the socket.
//! 3. **Event encoder thread** ([`run_event_encoder`]): drains the session's
//!    `event_rx` ([`SessionEvent`]s), encodes each via the shared [`DapAdapter`]
//!    (`encode_event`) into DAP frames, and pushes them into the `out_tx` frame
//!    channel for the socket bridge to write. It never touches the `Transport`.
//!
//! `std::sync::mpsc` has no `select`, and `Transport` is `!Sync`, so the socket
//! bridge polls inbound with a small timeout ([`POLL_INTERVAL`]) and drains the
//! encoder's frame channel between polls — the "equivalent structure" to two
//! independent bridge loops. The poll interval is small enough to be
//! imperceptible for interactive debugging and adds no busy-spin (it blocks for
//! the interval when idle).
//!
//! # Shared `DapAdapter` (`Arc<Mutex<…>>`)
//!
//! The adapter is the single stateful correlation point (a monotonic `seq`
//! counter + per-kind FIFO `request_seq` table). It is mutated by BOTH the
//! socket bridge (decoding requests, producing the `setBreakpoints` response)
//! and the encoder thread (encoding events), so it is shared behind an
//! `Arc<Mutex<…>>`.
//!
//! # No double-response to `scopes`
//!
//! `DapAdapter::decode_request("scopes")` SELF-ANSWERS the scopes response at
//! decode time (from the frame id alone) AND still returns a
//! `SessionCommand::Scopes`. The socket bridge sends that self-answer
//! immediately and forwards the `Scopes` command; the session replies with a
//! `SessionEvent::Scopes`, but `DapAdapter::encode_event(Scopes)` is a
//! deliberate no-op (returns no frames), so the client receives EXACTLY one
//! scopes response. The same single-response guarantee holds for `threads`
//! (deferred: only the `SessionEvent::Threads` produces the wire response) and
//! `setBreakpoints` (only the bridge-applied `Breakpoints` event produces it).
//!
//! # `.pasta` presentation seam
//!
//! [`SourceMapWiring`] threads the optional shared `Arc<SourceMap>` plus the
//! shared effective [`SourceMode`] into this module's consumers: the `.pasta`
//! source resolver ([`attach_pasta_resolver`], task 5.2), the `.pasta`→`.lua`
//! breakpoint translation ([`translate_pasta_breakpoints`], task 5.3) and the
//! runtime presentation toggle / `attach` override handled in [`handle_inbound`]
//! (tasks 3.1 / 5.5). Each consumer gates on
//! [`SourceMapWiring::pasta_active`] at use time, so a mode flip is observed
//! immediately; with no map or in `Lua` mode the existing `.lua` behavior is
//! kept byte-for-byte (requirements 6.1 / 6.2 / 7.2).
//!
//! # Shutdown (no hang)
//!
//! A shared [`AtomicBool`] shutdown flag lets the owner ([`DebugHandle`]) signal
//! the socket bridge to stop without blocking: the bridge checks it each poll
//! iteration and exits within [`POLL_INTERVAL`], dropping the `Transport` (which
//! winds the transport down). The encoder thread ends when the session's
//! `event_rx` closes (the VM thread finished and dropped the session) or when
//! the frame channel's receiver is gone.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::Duration;

use serde_json::Value;

use crate::debug::{SharedSourceMode, SourceMode};
use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::{DapAdapter, pasta_source_resolver};
use crate::debug::source_map::SourceMap;
use crate::debug::transport::Transport;
use crate::debug::types::{
    Breakpoint, ResolvedBreakpoint, SessionCommand, SessionEvent, SourceRef,
};

/// The (optional) shared source map + present mode threaded from
/// [`enable`](crate::debug::enable) to the I/O-side consumers (task 4.2).
///
/// Carries the immutable `Arc<SourceMap>` and the shared effective [`SourceMode`]
/// to the socket-bridge thread, which owns the [`DapAdapter`] where the `.pasta`
/// source RESOLVER attaches ([`attach_pasta_resolver`], task 5.2) and applies
/// `setBreakpoints` where the `.pasta` BP TRANSLATION attaches
/// ([`translate_pasta_breakpoints`], task 5.3) — both consumers live in this
/// module.
///
/// `source_map` is `Some` whenever `enable` was given a map, REGARDLESS of the
/// initial mode: the mode half of the gate is decided at CONSUMPTION time by
/// [`pasta_active`](Self::pasta_active) (design 582), because a DAP `attach`
/// `sourcePresentation` can flip the mode AFTER `enable` — including Lua→Pasta,
/// which needs the map available. With no map, or while the effective mode is
/// [`SourceMode::Lua`], every consumer keeps the existing default `.lua`
/// behavior byte-for-byte (requirements 6.1 / 6.2 / 7.2).
#[derive(Clone)]
pub(crate) struct SourceMapWiring {
    /// Immutable shared map, or `None` for default `.lua` behavior.
    pub(crate) source_map: Option<Arc<SourceMap>>,
    /// The EFFECTIVE present mode for this session (requirements 6.1 default
    /// `.pasta`, 6.2 `.lua`). Shared and interior-mutable so the DAP `attach`
    /// `sourcePresentation` argument (highest precedence, design 581) can flip it
    /// at request time and have BOTH the resolver (task 5.2) and the VM-thread
    /// stepper (task 5.4) observe the same value (task 5.5 / requirement 6.3).
    /// Initialised at [`enable`](crate::debug::enable) time from the resolved
    /// `DebugConfig::source_mode` (env > file > 既定).
    pub(crate) source_mode: SharedSourceMode,
}

impl SourceMapWiring {
    /// The default (disabled) wiring: no map, effective mode default `.pasta` —
    /// but with no map the consumers behave exactly as today's `.lua` path (7.2).
    ///
    /// Test-only constructor: production (`enable` in `mod.rs`) always builds the
    /// wiring via the struct literal with the resolved map/mode, so this shorthand
    /// is exercised only by the in-file test modules.
    #[cfg(test)]
    pub(crate) fn disabled() -> Self {
        Self {
            source_map: None,
            source_mode: SharedSourceMode::new(SourceMode::default()),
        }
    }

    /// Whether the `.pasta` consumers (resolver / BP translation / stepper) should
    /// be active: a map is present AND the EFFECTIVE mode is
    /// [`SourceMode::Pasta`] (design 582). Otherwise every consumer keeps its
    /// default `.lua` behavior. Reads the shared mode each call so an `attach`
    /// flip is observed immediately (task 5.5).
    pub(crate) fn pasta_active(&self) -> bool {
        self.source_map.is_some() && self.source_mode.get() == SourceMode::Pasta
    }
}

/// Inbound poll interval for the socket bridge. `std::sync::mpsc` has no
/// `select` and [`Transport`] is `!Sync`, so the single Transport-owner thread
/// polls inbound with this timeout and drains the outbound frame channel between
/// polls. Small enough to be imperceptible interactively; it blocks (does not
/// busy-spin) for the interval when idle.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Shared DAP adapter (seq counter + per-kind FIFO `request_seq` correlation),
/// mutated by BOTH the socket bridge and the event encoder thread.
pub(crate) type SharedAdapter = Arc<Mutex<DapAdapter>>;

/// Install the `.pasta` source RESOLVER on the shared [`DapAdapter`] when the
/// `.pasta` consumers should be active (task 5.2・design 509/582).
///
/// When `source_map.pasta_active()` (a map present AND
/// [`SourceMode::Pasta`](crate::debug::SourceMode), requirements 6.1), the
/// adapter's source seam is swapped to
/// [`pasta_source_resolver`](crate::debug::dap::pasta_source_resolver) so every
/// stack frame is presented in `.pasta` coordinates (R5.1/R5.2), with unmapped
/// frames falling back to the generated `.lua` (R5.3). Otherwise
/// (`SourceMode::Lua` or no map) the adapter keeps its default `.lua` resolver
/// untouched — byte-for-byte the existing behavior (requirements 6.2 / 7.2).
///
/// Called ONCE by [`run_socket_bridge`] before the inbound/outbound loop, so the
/// resolver is in place before any `stackTrace` is encoded. A poisoned adapter
/// lock is treated as "do not attach" (the bridge never panics); the default
/// `.lua` resolver then remains, which is the safe fallback.
///
/// It is also RE-RUN when a DAP `attach` `sourcePresentation` flips the effective
/// mode (task 5.5): because [`SourceMapWiring::pasta_active`] reads the SHARED
/// effective mode, this re-installs the `.pasta` resolver on a Lua→Pasta flip AND
/// resets to the default `.lua` resolver on a Pasta→Lua flip (so the resolver
/// presentation always matches the FINAL effective mode, requirement 6.3).
fn attach_pasta_resolver(adapter: &SharedAdapter, source_map: &SourceMapWiring) {
    let resolver = if source_map.pasta_active() {
        // `pasta_active()` guarantees the map is `Some`; degrade to a no-op if it
        // is somehow absent (never panic in the bridge).
        match &source_map.source_map {
            Some(map) => pasta_source_resolver(Arc::clone(map)), // 5.1, 5.2, 5.3
            None => return,
        }
    } else {
        // Lua mode / no map → ensure the default `.lua` resolver (6.2/7.2). This
        // RESETS a previously-installed `.pasta` resolver on a Pasta→Lua `attach`
        // flip (task 5.5); on the first call (default adapter) it is a harmless
        // re-assert of the already-default resolver.
        crate::debug::dap::default_source_resolver()
    };
    if let Ok(mut dap) = adapter.lock() {
        dap.set_source_resolver(resolver);
    }
}

/// Socket bridge body: the SOLE owner of the [`Transport`]. Multiplexes inbound
/// socket frames (poll) and outbound encoded frames (`out_rx`) on one thread,
/// because the `Transport` is `!Sync` and `mpsc` has no `select`.
///
/// Per iteration it:
/// 1. checks `shutdown` (set by the handle's Drop) and exits if signalled;
/// 2. polls `transport.inbound()` with [`POLL_INTERVAL`]; on a frame, decodes it
///    via the shared adapter and writes immediate responses / applies
///    `setBreakpoints` to the shared store (writing its response) / forwards
///    stop-context commands to the session;
/// 3. drains the encoder's `out_rx` and writes each frame to the socket.
///
/// Returns (winding the transport down by dropping it) when the inbound channel
/// closes (client disconnect / shutdown) or the shutdown flag is set — never a
/// hang.
pub(crate) fn run_socket_bridge(
    transport: Transport,
    adapter: SharedAdapter,
    breakpoints: BreakpointSet,
    cmd_tx: Sender<SessionCommand>,
    out_rx: Receiver<Value>,
    shutdown: Arc<AtomicBool>,
    // The (optional) shared map + present mode delivered to the `.pasta` resolver
    // (5.2, attached just below) and BP-translation (5.3) attachment points on
    // this thread. No map / `Lua` mode → existing `.lua` behavior (6.1/6.2/7.2).
    source_map: SourceMapWiring,
) {
    // Task 5.2: install the `.pasta` source resolver on the shared adapter when
    // `pasta_active()` (map present AND `SourceMode::Pasta`, design 509/582). For
    // `Lua`/no-map this is a no-op and the default `.lua` resolver stays (6.2/7.2).
    // Done ONCE before the loop so it is in place before any `stackTrace` encode.
    attach_pasta_resolver(&adapter, &source_map);

    loop {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }

        // (1) Inbound: poll one frame (bounded so we can also service outbound).
        match transport.inbound().recv_timeout(POLL_INTERVAL) {
            Ok(req) => {
                if !handle_inbound(&transport, &adapter, &breakpoints, &cmd_tx, &req, &source_map) {
                    return; // peer gone while replying → done
                }
            }
            // Idle this interval: fall through to drain outbound.
            Err(RecvTimeoutError::Timeout) => {}
            // Inbound channel closed (client disconnected / transport reader
            // ended): flush any pending outbound frames, then stop.
            Err(RecvTimeoutError::Disconnected) => {
                drain_outbound(&transport, &out_rx);
                return;
            }
        }

        // (2) Outbound: drain all currently-available encoded frames.
        if !drain_outbound(&transport, &out_rx) {
            return; // peer gone → done
        }
    }
}

/// Decode and act on one inbound DAP request frame. Returns `false` if the peer
/// is gone (a transport write failed) so the caller stops.
fn handle_inbound(
    transport: &Transport,
    adapter: &SharedAdapter,
    breakpoints: &BreakpointSet,
    cmd_tx: &Sender<SessionCommand>,
    req: &Value,
    // The shared map + present mode, consulted by the `setBreakpoints` branch
    // below: when `pasta_active()` a `.pasta` source is routed through
    // `translate_pasta_breakpoints` (task 5.3), otherwise the `.lua` direct path
    // is unchanged (requirements 6.2 / 7.2). Also read by the presentation-toggle
    // / `attach` branches to flip the shared effective mode (tasks 3.1 / 5.5).
    source_map: &SourceMapWiring,
) -> bool {
    // The RAW request command string. Needed to disambiguate the runtime toggle
    // (task 3.1 / requirement 1.4): `decoded.requested_source_mode == None` is
    // None BOTH for a `pasta/sourcePresentation` request carrying an invalid mode
    // AND for any non-toggle request, so the toggle MUST be detected by the
    // command string, never by the `Option` alone. Likewise the attach-complete
    // event (requirement 2.5) keys off `command == "attach"`.
    let command = req.get("command").and_then(Value::as_str).unwrap_or("");

    // Decode under the shared adapter lock (seq counter / pending table).
    let decoded = {
        let mut dap = match adapter.lock() {
            Ok(g) => g,
            Err(_) => return false, // poisoned → stop (never panic in the bridge)
        };
        dap.decode_request(req)
    };

    // Task 3.1 (requirements 1.1/1.2/1.3/1.4/1.5, 2.5/2.6, 3.1/3.2/3.4/3.5,
    // 4.2/4.3 / design "DAP custom request handler"): the runtime presentation
    // TOGGLE. `handle_inbound` is the application authority. Detected by the raw
    // command string (see above): a `pasta/sourcePresentation` request is a
    // self-contained exchange (the adapter `decode_request` emits no response /
    // command for it), so it is handled here and returns directly — keeping the
    // ORDER apply → ack → event → RefreshPresentation exact and free of any
    // interleaving with the generic response/command routing below.
    if command == "pasta/sourcePresentation" {
        let request_seq = req.get("seq").and_then(Value::as_u64).unwrap_or(0);

        // (1) APPLY first, so the resolver/stepper switch is already in effect for
        // the subsequent redraw's stackTrace/source (requirements 3.1/3.2/3.4/3.5).
        // A `Some(mode)` is a valid toggle: write the SHARED effective mode cell
        // (requirements 1.1/1.2/4.2/4.3) — read by the VM-thread stepper per line —
        // and RE-RUN `attach_pasta_resolver` to swap the DAP source resolver to the
        // FINAL effective mode (`.pasta` resolver on Pasta+map, default `.lua`
        // otherwise — task 5.2). `None` is an UNRECOGNIZED mode value: make NO
        // change (requirement 1.4) — leave the cell and resolver as-is.
        if let Some(mode) = decoded.requested_source_mode {
            source_map.source_mode.set(mode);
            attach_pasta_resolver(adapter, source_map);
        }

        // The RESULTING current mode after applying (or NOT applying, for 1.4).
        let current = source_map.source_mode.get();

        // (2) Acceptance RESPONSE first, BEFORE the redraw (requirement 1.3): echo
        // the resolved current mode, correlated to the incoming request seq.
        let (response, event) = {
            let mut dap = match adapter.lock() {
                Ok(g) => g,
                Err(_) => return false,
            };
            (
                dap.source_presentation_response(request_seq, current),
                dap.source_presentation_event(current),
            )
        };
        if transport.send(response).is_err() {
            return false;
        }
        // (3) Custom EVENT carrying the current mode (the status-bar push
        // notification — requirements 2.5/2.6).
        if transport.send(event).is_err() {
            return false;
        }
        // (4) Forward `RefreshPresentation` to the session: while STOPPED it
        // re-emits the current `Stopped` so the client refetches in the new mode
        // (requirement 3.3); while RUNNING it is a no-op until the next natural
        // stop (requirement 1.5).
        if cmd_tx.send(SessionCommand::RefreshPresentation).is_err() {
            return false;
        }
        return true;
    }

    // Task 5.5 (requirement 6.3 / design 581/586): an `attach` request carrying an
    // explicit `sourcePresentation` is the HIGHEST-precedence present-mode source.
    // Apply it to THIS session BEFORE replying so the resolver/stepper switch is
    // already in effect: (1) write the SHARED effective mode (read by the VM-thread
    // stepper per line, task 5.4), and (2) RE-RUN `attach_pasta_resolver` so the
    // DAP source resolver presentation matches the FINAL effective mode (attach
    // `.pasta` resolver on Pasta+map, reset to default `.lua` otherwise — task
    // 5.2). When the `attach` arg is ABSENT (`None`) this is skipped, so the
    // resolved env > file > 既定 mode stays in effect (no client-default override).
    if let Some(mode) = decoded.attach_source_mode {
        source_map.source_mode.set(mode);
        attach_pasta_resolver(adapter, source_map);
    }

    // (a) Immediate response (acks / initialize / scopes self-answer).
    if let Some(response) = decoded.response
        && transport.send(response).is_err()
    {
        return false;
    }
    // (b) Immediate unsolicited events (the `initialized` handshake event).
    for ev in decoded.events {
        if transport.send(ev).is_err() {
            return false;
        }
    }

    // Task 3.1 case B (requirement 2.5 initial display / design "Event Contract"
    // (a)): on `attach` completion, emit the resolved initial-mode
    // `pasta/sourcePresentation` event so the extension can show the resolved
    // INITIAL mode (the single source of truth for the status bar, no query). This
    // covers BOTH the explicit-`sourcePresentation` attach (mode just applied
    // above) AND the no-arg attach (the resolved env > file > 既定 mode kept); the
    // RESULTING cell mode is read in either case. Keyed off the raw command string
    // (the no-arg case has `attach_source_mode == None`, indistinguishable by the
    // Option), and emitted AFTER the attach ack so the ack precedes the event.
    if command == "attach" {
        let current = source_map.source_mode.get();
        let event = {
            let mut dap = match adapter.lock() {
                Ok(g) => g,
                Err(_) => return false,
            };
            dap.source_presentation_event(current)
        };
        if transport.send(event).is_err() {
            return false;
        }
    }

    // (c) Command routing.
    match decoded.command {
        // `setBreakpoints` is the ONE command valid while the VM runs: apply it
        // directly to the shared store and synthesize the DAP response via the
        // adapter (correlated to the originating request seq). It is NOT
        // forwarded to the session (that would block off a stop).
        Some(SessionCommand::SetBreakpoints { source, lines }) => {
            // Task 5.3: when `pasta_active()` (a map present AND `SourceMode::
            // Pasta`, design 582) the source is treated as a `.pasta` file and
            // each requested `.pasta` line is TRANSLATED to its `.lua` execution
            // coords via `resolve_pasta_to_lua`, registering ALL of them (4.1 /
            // 8.2) and adjusting no-correspondence lines to the nearest
            // SUBSEQUENT mapped `.pasta` line (4.3). Otherwise (`Lua` mode / no
            // map) the existing `.lua` direct path is used byte-for-byte
            // (requirements 6.2 / 7.2).
            let resolved = if source_map.pasta_active() && is_pasta_source(&source.path) {
                translate_pasta_breakpoints(breakpoints, source_map, &source, &lines)
            } else {
                // `Lua` mode / no map, OR a `.lua` source presented in Pasta mode
                // (design "BpTranslator" 514: `.lua` source → direct register):
                // existing path, byte-for-byte (requirements 6.2 / 7.2).
                breakpoints.set_breakpoints(&source, &lines)
            };
            let frames = {
                let mut dap = match adapter.lock() {
                    Ok(g) => g,
                    Err(_) => return false,
                };
                dap.encode_event(SessionEvent::Breakpoints(resolved))
            };
            for frame in frames {
                if transport.send(frame).is_err() {
                    return false;
                }
            }
        }
        // Every other (stop-context) command is forwarded to the session's
        // VM-thread stop loop. If the session controller is gone, stop.
        Some(cmd) => {
            if cmd_tx.send(cmd).is_err() {
                return false;
            }
        }
        None => {}
    }
    true
}

/// Whether a DAP `setBreakpoints` source path names a `.pasta` file (design
/// "BpTranslator" 514: `.pasta` source → translate; `.lua` source → direct
/// register). Case-insensitive on the extension so `.PASTA` is also recognised.
///
/// In Pasta mode VSCode presents `.pasta` paths, but a `.lua` source can still be
/// set (e.g. the author opened a generated `.lua`); routing only `.pasta` sources
/// through the translator keeps the `.lua` direct path intact (6.2 / 7.2) and
/// avoids treating a `.lua` path as an (unmapped) `.pasta` file.
fn is_pasta_source(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pasta"))
}

/// Translate a `.pasta`-source `setBreakpoints` into `.lua` execution-coordinate
/// registrations and build the DAP `setBreakpoints` response (task 5.3・design
/// "BpTranslator" 511-528・Flow 2 215-236・requirements 4.1 / 4.2 / 4.3 / 8.2).
///
/// Only called when [`SourceMapWiring::pasta_active`] (a map present AND
/// [`SourceMode::Pasta`](crate::debug::SourceMode)); the `.lua`/`Lua`/no-map path
/// keeps the existing direct [`BreakpointSet::set_breakpoints`] (requirements 6.2
/// / 7.2). For each requested `.pasta` `line`:
///
/// 1. `resolve_pasta_to_lua(pasta_path, line)` → all `(chunk, lua_line)` exec
///    coords. If non-empty, every coord is registered (4.1; one `.pasta` line may
///    expand to MANY `.lua` lines, 8.2) and the BP is reported `verified` at the
///    ORIGINAL `line`.
/// 2. No correspondence → `nearest_pasta_line_with_mapping(pasta_path, line)`
///    finds the nearest SUBSEQUENT mapped `.pasta` line; THAT line's `.lua` coords
///    are registered and the BP is reported `verified` at the ADJUSTED line (so
///    VSCode shows it moved, 4.3). NEVER mismaps.
/// 3. No nearest mapping at all → `verified: false` at the original line (4.3:
///    only adjust to a real subsequent line; otherwise leave unverified).
///
/// ALL execution coords across ALL requested lines are accumulated into a SINGLE
/// [`BreakpointSet::register`] call tagged with the `.pasta` present source, so
/// they replace this presented source's prior set atomically (per-present-source
/// authoritative; a `.pasta`-origin and a `.lua`-origin BP in the same chunk
/// never evict each other — requirements 4.4 / 8.2). The hook reports the RAW
/// `@<.lua path>` source; [`BreakpointSet::should_pause`] canonicalizes both the
/// hook source and these stored canonical chunks, so the registered `.pasta` BP
/// fires for the runtime coord (4.2).
fn translate_pasta_breakpoints(
    breakpoints: &BreakpointSet,
    source_map: &SourceMapWiring,
    source: &SourceRef,
    lines: &[u32],
) -> Vec<ResolvedBreakpoint> {
    // `pasta_active()` guarantees the map is `Some`; degrade safely to the `.lua`
    // path if it is somehow absent (never panic in the bridge).
    let map = match &source_map.source_map {
        Some(map) => map,
        None => return breakpoints.set_breakpoints(source, lines),
    };
    let pasta_path = source.path.as_str();

    // Accumulate ALL execution coords for ALL requested lines into one register
    // call (replacing this present source's set atomically). One requested line
    // may yield many `(chunk, lua_line)` (8.2); a no-correspondence line is
    // adjusted to the nearest subsequent mapped `.pasta` line (4.3).
    let mut entries: Vec<Breakpoint> = Vec::new();
    let resolved: Vec<ResolvedBreakpoint> = lines
        .iter()
        .map(|&line| {
            // (1) Direct correspondence: register all `.lua` coords, verified at
            // the original line (4.1 / 8.2).
            let direct = map.resolve_pasta_to_lua(pasta_path, line);
            if !direct.is_empty() {
                for (chunk, lua_line) in direct {
                    entries.push(Breakpoint::new(pasta_path, chunk, lua_line));
                }
                return ResolvedBreakpoint {
                    source: source.clone(),
                    line,
                    verified: true,
                };
            }
            // (2) No correspondence: adjust to the nearest SUBSEQUENT mapped
            // `.pasta` line and register THAT line's coords, verified at the
            // adjusted line (4.3).
            if let Some(adjusted) = map.nearest_pasta_line_with_mapping(pasta_path, line) {
                for (chunk, lua_line) in map.resolve_pasta_to_lua(pasta_path, adjusted) {
                    entries.push(Breakpoint::new(pasta_path, chunk, lua_line));
                }
                return ResolvedBreakpoint {
                    source: source.clone(),
                    line: adjusted,
                    verified: true,
                };
            }
            // (3) No nearest mapping at all → unverified at the original line
            // (4.3: never mismap; only adjust to a real subsequent line).
            ResolvedBreakpoint {
                source: source.clone(),
                line,
                verified: false,
            }
        })
        .collect();

    // Replace this `.pasta` present source's prior set with the accumulated exec
    // coords (per-present-source authoritative; other sources preserved).
    breakpoints.register(pasta_path, entries);
    resolved
}

/// Drain all currently-available outbound frames from `out_rx` and write them.
/// Returns `false` if a transport write failed (peer gone) so the caller stops;
/// a closed `out_rx` (encoder thread ended) is NOT a stop condition here — the
/// client may still send inbound commands.
fn drain_outbound(transport: &Transport, out_rx: &Receiver<Value>) -> bool {
    loop {
        match out_rx.try_recv() {
            Ok(frame) => {
                if transport.send(frame).is_err() {
                    return false;
                }
            }
            // Nothing pending, or the encoder thread is gone: stop draining.
            Err(_) => return true,
        }
    }
}

/// Event encoder body: session `event_rx` ([`SessionEvent`]) → encode via the
/// shared [`DapAdapter`] → push DAP frames into `out_tx` for the socket bridge.
///
/// Runs on its own thread (no `mlua::Lua`, no `Transport`). Returns when the
/// session event channel closes (the session/VM is gone) or the frame channel's
/// receiver is gone — both clean, never a hang.
pub(crate) fn run_event_encoder(
    adapter: SharedAdapter,
    event_rx: Receiver<SessionEvent>,
    out_tx: Sender<Value>,
) {
    while let Ok(event) = event_rx.recv() {
        let frames = {
            let mut dap = match adapter.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            dap.encode_event(event)
        };
        for frame in frames {
            if out_tx.send(frame).is_err() {
                return; // socket bridge gone → done
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! End-to-end integration: drive the FULL attach→BP→stack→vars→step→
    //! continue→terminated path over real TCP through [`enable`], exercising
    //! every layer (transport / dap / session / hook / inspect) wired together.
    //!
    //! `mlua::Lua` is `!Send`: it is built and owned entirely on the VM host
    //! thread; only channels / the bound address (a `SocketAddr`, `Copy`) cross
    //! the thread boundary. All client-side waits use a TEST-ONLY watchdog so CI
    //! cannot hang; the stop core itself stays unbounded.

    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::transport::{read_frame, write_frame};
    use crate::debug::{DebugConfig, enable};

    /// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// The generated-`.lua` source name and breakpoint line for the scenario.
    const SCENARIO_SOURCE: &str = "@e2e_scenario";

    /// The scenario chunk: a top-level chunk that drives a coroutine whose body
    /// has the breakpoint target (a coroutine-body local must be inspectable).
    /// The breakpoint sits on a line AFTER `co_local` is assigned so the local
    /// is a live, named slot when inspected (a local on its OWN declaration line
    /// is still an unnamed `(*temporary)` slot). Lines (1-origin):
    ///   1: local function helper(x)
    ///   2:     local y = x + 1
    ///   3:     return y
    ///   4: end
    ///   5: local body = function()
    ///   6:     local co_local = 7
    ///   7:     local marker = co_local      <- BREAKPOINT (co_local is live here)
    ///   8:     local doubled = helper(marker)
    ///   9:     coroutine.yield()
    ///  10:     return doubled
    ///  11: end
    ///  12: local co = coroutine.create(body)
    ///  13: while coroutine.status(co) ~= 'dead' do
    ///  14:     coroutine.resume(co)
    ///  15: end
    const SCENARIO_CHUNK: &str = "\
local function helper(x)
    local y = x + 1
    return y
end
local body = function()
    local co_local = 7
    local marker = co_local
    local doubled = helper(marker)
    coroutine.yield()
    return doubled
end
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
";
    const BREAKPOINT_LINE: u32 = 7;

    /// A test DAP client over a real TCP socket: Content-Length framed JSON.
    struct DapClient {
        reader: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl DapClient {
        fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone socket for writing");
            Self {
                reader: BufReader::new(stream),
                writer,
            }
        }

        /// Send a DAP request with the given seq/command/arguments.
        fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
            let req = json!({
                "seq": seq,
                "type": "request",
                "command": command,
                "arguments": arguments,
            });
            write_frame(&mut self.writer, &req).expect("client write must succeed");
        }

        /// Read the next framed message (bounded by the TEST-ONLY read timeout).
        fn recv(&mut self) -> Value {
            read_frame(&mut self.reader)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present (peer did not close)")
        }

        /// Read messages until one matching `pred` arrives; returns it. Bounded
        /// by the read timeout per read so CI cannot hang.
        fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
            loop {
                let msg = self.recv();
                if pred(&msg) {
                    return msg;
                }
            }
        }
    }

    fn is_event(msg: &Value, name: &str) -> bool {
        msg["type"] == "event" && msg["event"] == name
    }

    fn is_response(msg: &Value, command: &str) -> bool {
        msg["type"] == "response" && msg["command"] == command
    }

    /// The headline integration test (task 4.1 "done"): a full DAP session over
    /// real TCP through `enable`, hitting a breakpoint inside a coroutine body,
    /// inspecting the stack and a coroutine-body local, stepping, continuing,
    /// and running to completion — all layers wired end-to-end.
    #[test]
    fn full_dap_session_over_tcp_attach_bp_stack_vars_step_continue_terminated() {
        // Coordination channels: host → main carries the bound addr; main → host
        // carries the "breakpoints are set, run the VM now" go signal. mlua::Lua
        // never crosses — only these Send values do.
        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        // VM HOST THREAD: build the VM, enable the backend, publish the bound
        // addr, wait for the client handshake, then run the scenario chunk.
        let host = std::thread::spawn(move || -> Result<(), String> {
            // ALL_SAFE VM: `jit` exists, `debug` excluded; `enable`'s hook does
            // jit.off() itself (mirrors the other debug tests' VM build).
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                // Port 0 → OS-assigned free loopback port (no fixed-port clash).
                listen: Some("127.0.0.1:0".parse().unwrap()),
                ..Default::default()
            };
            let handle = enable(&lua, &cfg, None)
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            // Wait for the client to finish initialize/setBreakpoints/
            // configurationDone before running the VM (so the BP is live).
            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            // Run the scenario. This blocks at the breakpoint until the client
            // sends continue; the VM thread processes inspect/step in the hook.
            lua.load(SCENARIO_CHUNK)
                .set_name(SCENARIO_SOURCE)
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;
            lua.remove_global_hook();

            // Keep the handle alive until the chunk has fully run, then drop it
            // (Drop signals shutdown + winds the transport/bridges down).
            drop(handle);
            Ok(())
        });

        // CLIENT (this thread): connect and drive the DAP handshake + session.
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // --- initialize → capabilities + `initialized` ---
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let init_resp = client.recv_until(|m| is_response(m, "initialize"));
        assert_eq!(init_resp["success"], true, "initialize must succeed");
        assert_eq!(init_resp["request_seq"], 1);
        assert_eq!(
            init_resp["body"]["supportsConfigurationDoneRequest"], true,
            "initialize must advertise supportsConfigurationDoneRequest"
        );
        let _initialized = client.recv_until(|m| is_event(m, "initialized"));

        // --- setBreakpoints on the `.lua` source line ---
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": SCENARIO_SOURCE },
                "breakpoints": [{ "line": BREAKPOINT_LINE }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        assert_eq!(bp_resp["request_seq"], 2, "setBreakpoints response correlates");
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0]["verified"], true);
        assert_eq!(bps[0]["line"], BREAKPOINT_LINE);

        // --- configurationDone (ack) ---
        client.send_request(3, "configurationDone", json!({}));
        let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
        assert_eq!(cfg_resp["success"], true);
        assert_eq!(cfg_resp["request_seq"], 3);

        // Breakpoints are live + config done: let the VM run.
        go_tx.send(()).expect("send go signal");

        // --- the VM hits the breakpoint → `stopped` event ---
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "must stop with reason breakpoint at the coroutine-body BP"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        // --- threads → at least the main thread ---
        client.send_request(9, "threads", json!({}));
        let threads = client.recv_until(|m| is_response(m, "threads"));
        assert_eq!(threads["request_seq"], 9);
        let thread_arr = threads["body"]["threads"].as_array().expect("threads array");
        assert!(!thread_arr.is_empty(), "threads must report at least one thread");

        // --- stackTrace → frames (top frame is the coroutine body BP line) ---
        client.send_request(10, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        assert_eq!(stack["request_seq"], 10);
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have at least the stopped frame");
        assert_eq!(
            frames[0]["source"]["path"], SCENARIO_SOURCE,
            "top frame source must be the scenario `.lua`"
        );
        assert_eq!(
            frames[0]["line"], BREAKPOINT_LINE,
            "top frame line must be the breakpoint line"
        );

        // --- scopes → a `Locals` scope with a decodable variablesReference ---
        let frame_id = frames[0]["id"].as_u64().expect("frame id");
        client.send_request(11, "scopes", json!({ "frameId": frame_id }));
        let scopes = client.recv_until(|m| is_response(m, "scopes"));
        assert_eq!(scopes["request_seq"], 11);
        let scope_arr = scopes["body"]["scopes"].as_array().expect("scopes array");
        assert_eq!(scope_arr.len(), 1, "exactly one scopes response (no double-answer)");
        assert_eq!(scope_arr[0]["name"], "Locals");
        let var_ref = scope_arr[0]["variablesReference"]
            .as_u64()
            .expect("variablesReference");
        assert!(var_ref != 0, "variablesReference must be non-zero");

        // --- variables → the coroutine-body local `co_local` (= 7) ---
        client.send_request(12, "variables", json!({ "variablesReference": var_ref }));
        let vars = client.recv_until(|m| is_response(m, "variables"));
        assert_eq!(vars["request_seq"], 12);
        let var_arr = vars["body"]["variables"].as_array().expect("variables array");
        let co_local = var_arr
            .iter()
            .find(|v| v["name"] == "co_local")
            .unwrap_or_else(|| panic!("coroutine-body local `co_local` must be present: {var_arr:?}"));
        assert_eq!(co_local["type"], "number");
        assert_eq!(co_local["value"], "7", "co_local must read its live value 7");

        // --- step over (`next`) → ack + a new `stopped(step)` ---
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let next_ack = client.recv_until(|m| is_response(m, "next"));
        assert_eq!(next_ack["request_seq"], 20);
        let step_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            step_stopped["body"]["reason"], "step",
            "after `next` the VM must re-stop with reason step"
        );

        // --- continue → run to completion ---
        client.send_request(30, "continue", json!({ "threadId": thread_id }));
        let cont_ack = client.recv_until(|m| is_response(m, "continue"));
        assert_eq!(cont_ack["request_seq"], 30);

        // The scenario runs to completion: the host VM thread finishes (drops the
        // handle → shutdown). We assert the host completed within the watchdog;
        // the chunk completing (not a disconnect) is the natural session end.
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion after continue");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    // =======================================================================
    // Task 8.1 — COMPREHENSIVE Lua-level debug session E2E (the full
    // requirement matrix in ONE cohesive DAP-over-TCP session).
    //
    // Where the 4.1 headline test above proves the path is wired end-to-end,
    // THIS test exhaustively exercises the user-facing Lua debug feature set in
    // a single session and asserts the EXACT DAP responses/events at each stage
    // so a regression in ANY layer (transport / dap / session / step / inspect)
    // fails it. It maps to:
    //   R1.1/1.2  BP set on a `.lua` line + hit → `stopped(breakpoint)`
    //   R1.3      step over (`next`)  → exact next `.lua` line
    //   R1.4      step into (`stepIn`)→ a callee's first body line
    //   R1.5      step out (`stepOut`)→ back in the caller, past the call
    //   R1.6      continue → run to completion
    //   R1.7      the BP + all steps fire ACROSS a scene coroutine body
    //   R2.1      stackTrace frames carry the `.lua` source + line
    //   R2.2/2.3  variables expose number / string / boolean / table by name+type
    //   R2.4      a coroutine-BODY-frame local is inspectable
    //   R2.5      unsupported kinds (function / nil) are surfaced gracefully,
    //             the request does NOT error, and the VM stays usable
    //   R3.4/3.5  stopped + terminated events over the wire
    //   R3.6      the DAP-over-TCP attach IS the VSCode-equivalent client target
    //             (the VSCode factory returns the same DebugAdapterServer; the
    //             config-factory wiring itself is covered by task 6.1)
    //
    // `mlua::Lua` (!Send) is built and owned solely on the VM host thread; only
    // the bound `SocketAddr` (Copy) and the go/done channels cross. All client
    // waits are bounded by the TEST-ONLY [`WATCHDOG`] so CI cannot hang; the
    // stop core stays unbounded.
    // =======================================================================

    /// Comprehensive scenario source name and breakpoint line.
    const FULL_SOURCE: &str = "@e2e_full_scenario";

    /// The comprehensive scenario chunk. A `helper` callee (for step into/out)
    /// plus a coroutine BODY (so the BP + steps cross a scene coroutine, R1.7)
    /// whose frame holds ALL basic variable types AND unsupported kinds.
    ///
    /// The breakpoint sits on a line where every local declared above it is a
    /// live, NAMED slot (a local on its OWN declaration line is still an unnamed
    /// `(*temporary)` slot, so the BP is placed AFTER all the declarations).
    ///
    /// Lines (1-origin):
    ///   1: local function helper(x)
    ///   2:     local hv = x + 1          <- step INTO target (helper body line)
    ///   3:     return hv
    ///   4: end
    ///   5: local body = function()
    ///   6:     local num = 7             -- number   (R2.2/R2.3)
    ///   7:     local str = 'hi'          -- string   (R2.2/R2.3)
    ///   8:     local flag = true         -- boolean  (R2.2/R2.3)
    ///   9:     local tbl = { 1, 2, 3 }   -- table    (R2.2/R2.3)
    ///  10:     local fn = helper         -- function (UNSUPPORTED, R2.5)
    ///  11:     local nilv = nil          -- nil      (UNSUPPORTED, R2.5)
    ///  12:     local marker = num        <- BREAKPOINT (6..=11 all live here)
    ///  13:     local doubled = helper(marker)  <- step OVER lands here; step
    ///                                             INTO from here enters helper
    ///  14:     coroutine.yield()         <- step OUT (from helper) lands here
    ///  15:     return doubled
    ///  16: end
    ///  17: local co = coroutine.create(body)
    ///  18: while coroutine.status(co) ~= 'dead' do
    ///  19:     coroutine.resume(co)
    ///  20: end
    const FULL_CHUNK: &str = "\
local function helper(x)
    local hv = x + 1
    return hv
end
local body = function()
    local num = 7
    local str = 'hi'
    local flag = true
    local tbl = { 1, 2, 3 }
    local fn = helper
    local nilv = nil
    local marker = num
    local doubled = helper(marker)
    coroutine.yield()
    return doubled
end
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
";
    /// Stop lines for the comprehensive scenario (1-origin, see [`FULL_CHUNK`]).
    const FULL_BP_LINE: u32 = 12; // `local marker = num` (all locals live).
    const FULL_STEP_OVER_LINE: u32 = 13; // same frame, next line after the BP.
    const FULL_STEP_IN_LINE: u32 = 2; // helper's first body line.
    const FULL_STEP_OUT_LINE: u32 = 14; // back in the body, past the helper call.

    /// The comprehensive task-8.1 E2E: ONE DAP-over-TCP session driving the full
    /// Lua-level debug feature matrix (R1.1–1.7, R2.1–2.5, R3.4/3.5, R3.6) with
    /// EXACT assertions at every stage. See the section comment above for the
    /// per-stage requirement mapping.
    #[test]
    fn full_lua_debug_session_all_steps_all_var_types_coroutine_body() {
        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        // VM HOST THREAD: owns `mlua::Lua` (!Send) for its whole lifetime.
        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                ..Default::default()
            };
            let handle = enable(&lua, &cfg, None)
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            // Run the scenario: this blocks at the breakpoint, then is driven by
            // the client over the wire (inspect/step processed in the hook loop
            // ON THIS THREAD). It runs through the coroutine yield/resume on
            // `continue` and returns when the coroutine is dead.
            lua.load(FULL_CHUNK)
                .set_name(FULL_SOURCE)
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;

            // Prove the VM is still usable after the whole debug session (no
            // stack corruption from any inspect; R2.5 "VM stays usable").
            let sane: i64 = lua
                .load("return 1 + 2")
                .eval()
                .map_err(|e| format!("post-session VM eval failed: {e}"))?;
            if sane != 3 {
                return Err(format!("VM stack corrupted after session: 1+2 = {sane}"));
            }

            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        // CLIENT (this thread) — the VSCode-equivalent DAP-over-TCP client (R3.6).
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // --- initialize → capabilities + `initialized` (R3.2 handshake) ---
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let init_resp = client.recv_until(|m| is_response(m, "initialize"));
        assert_eq!(init_resp["success"], true, "initialize must succeed");
        assert_eq!(init_resp["request_seq"], 1);
        assert_eq!(
            init_resp["body"]["supportsConfigurationDoneRequest"], true,
            "initialize must advertise supportsConfigurationDoneRequest"
        );
        let _initialized = client.recv_until(|m| is_event(m, "initialized"));

        // --- setBreakpoints on the `.lua` source line (R1.1) ---
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": FULL_SOURCE },
                "breakpoints": [{ "line": FULL_BP_LINE }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        assert_eq!(bp_resp["request_seq"], 2);
        let bps = bp_resp["body"]["breakpoints"].as_array().expect("bp array");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0]["verified"], true, "the `.lua` BP must be verified (R1.1)");
        assert_eq!(bps[0]["line"], FULL_BP_LINE);

        // --- configurationDone (ack) → let the VM run ---
        client.send_request(3, "configurationDone", json!({}));
        let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
        assert_eq!(cfg_resp["success"], true);
        assert_eq!(cfg_resp["request_seq"], 3);
        go_tx.send(()).expect("send go signal");

        // --- hit the breakpoint inside the coroutine body → `stopped` (R1.2,
        //     R1.7, R3.4) ---
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "must stop with reason breakpoint at the coroutine-body BP (R1.2/R3.4)"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        // --- threads (R3.3) ---
        client.send_request(10, "threads", json!({}));
        let threads = client.recv_until(|m| is_response(m, "threads"));
        assert_eq!(threads["request_seq"], 10);
        let thread_arr = threads["body"]["threads"].as_array().expect("threads array");
        assert!(!thread_arr.is_empty(), "threads must report at least one thread");

        // --- stackTrace → top frame is the coroutine body BP line (R2.1) ---
        client.send_request(11, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        assert_eq!(stack["request_seq"], 11);
        let frames = stack["body"]["stackFrames"].as_array().expect("frames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame (R2.1)");
        assert_eq!(
            frames[0]["source"]["path"], FULL_SOURCE,
            "top frame source must be the scenario `.lua` (R2.1)"
        );
        assert_eq!(
            frames[0]["line"], FULL_BP_LINE,
            "top frame line must be the breakpoint line (R2.1)"
        );
        let frame_id = frames[0]["id"].as_u64().expect("frame id");

        // --- scopes → exactly one `Locals` scope (no double-answer) ---
        client.send_request(12, "scopes", json!({ "frameId": frame_id }));
        let scopes = client.recv_until(|m| is_response(m, "scopes"));
        assert_eq!(scopes["request_seq"], 12);
        let scope_arr = scopes["body"]["scopes"].as_array().expect("scopes array");
        assert_eq!(scope_arr.len(), 1, "exactly one scopes response (no double-answer)");
        assert_eq!(scope_arr[0]["name"], "Locals");
        let var_ref = scope_arr[0]["variablesReference"]
            .as_u64()
            .expect("variablesReference");
        assert_ne!(var_ref, 0, "variablesReference must be non-zero");

        // --- variables → ALL basic types by name+type+value, the coroutine-body
        //     local (R2.2/R2.3/R2.4), AND the unsupported kinds surfaced
        //     gracefully without an error response (R2.5) ---
        client.send_request(13, "variables", json!({ "variablesReference": var_ref }));
        let vars = client.recv_until(|m| is_response(m, "variables"));
        assert_eq!(vars["request_seq"], 13);
        assert_eq!(vars["success"], true, "variables must not error (R2.5)");
        let var_arr = vars["body"]["variables"].as_array().expect("variables array");

        let find = |name: &str| -> Value {
            var_arr
                .iter()
                .find(|v| v["name"] == name)
                .unwrap_or_else(|| panic!("coroutine-body local `{name}` must be present: {var_arr:?}"))
                .clone()
        };

        // number (R2.2/R2.3) — also THE coroutine-body local proof (R2.4).
        let num = find("num");
        assert_eq!(num["type"], "number", "num must be discriminated as number (R2.3)");
        assert_eq!(num["value"], "7", "num must read its live value 7 (R2.4)");
        // string
        let s = find("str");
        assert_eq!(s["type"], "string", "str must be discriminated as string (R2.3)");
        assert_eq!(s["value"], "hi", "str must read its live value 'hi'");
        // boolean
        let flag = find("flag");
        assert_eq!(flag["type"], "boolean", "flag must be a boolean (R2.3)");
        assert_eq!(flag["value"], "true", "flag must read its live value true");
        // table
        let tbl = find("tbl");
        assert_eq!(tbl["type"], "table", "tbl must be a table (R2.3)");
        assert!(
            tbl["value"].as_str().unwrap().starts_with("table:"),
            "table value must be a readable placeholder: {:?}",
            tbl["value"]
        );

        // R2.5: an UNSUPPORTED kind (function) is RECORDED gracefully — present
        // by name, type surfaced, repr marked unsupported — never dropped and
        // never erroring the request.
        let fnval = find("fn");
        assert_eq!(fnval["type"], "function", "unsupported kind type surfaced (R2.5)");
        assert!(
            fnval["value"].as_str().unwrap().starts_with("<unsupported"),
            "an unsupported kind must carry an out-of-scope repr (R2.5): {:?}",
            fnval["value"]
        );
        // nil is likewise surfaced gracefully.
        let nilv = find("nilv");
        assert_eq!(nilv["type"], "nil", "nil kind surfaced gracefully (R2.5)");
        assert!(
            nilv["value"].as_str().unwrap().starts_with("<unsupported"),
            "nil must carry an out-of-scope repr (R2.5): {:?}",
            nilv["value"]
        );

        // --- step OVER (`next`) → ack + stopped(step) at the next `.lua` line in
        //     the SAME frame, NOT inside helper (R1.3) ---
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let next_ack = client.recv_until(|m| is_response(m, "next"));
        assert_eq!(next_ack["request_seq"], 20);
        let over_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            over_stopped["body"]["reason"], "step",
            "step over must re-stop with reason step (R1.3)"
        );
        assert_eq!(
            top_frame_line(&mut client, thread_id, 21),
            FULL_STEP_OVER_LINE,
            "step over must stop at the next line in the SAME frame (R1.3), not inside helper"
        );

        // --- step INTO (`stepIn`) → ack + stopped(step) at helper's first body
        //     line (R1.4) ---
        client.send_request(30, "stepIn", json!({ "threadId": thread_id }));
        let in_ack = client.recv_until(|m| is_response(m, "stepIn"));
        assert_eq!(in_ack["request_seq"], 30);
        let in_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(in_stopped["body"]["reason"], "step", "step in reason step (R1.4)");
        assert_eq!(
            top_frame_line(&mut client, thread_id, 31),
            FULL_STEP_IN_LINE,
            "step in must stop at the callee's first body line (R1.4)"
        );

        // --- step OUT (`stepOut`) → ack + stopped(step) back in the caller body,
        //     past the helper call (R1.5) ---
        client.send_request(40, "stepOut", json!({ "threadId": thread_id }));
        let out_ack = client.recv_until(|m| is_response(m, "stepOut"));
        assert_eq!(out_ack["request_seq"], 40);
        let out_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(out_stopped["body"]["reason"], "step", "step out reason step (R1.5)");
        assert_eq!(
            top_frame_line(&mut client, thread_id, 41),
            FULL_STEP_OUT_LINE,
            "step out must stop back in the caller body past the call (R1.5)"
        );

        // --- continue (R1.6) → the coroutine yields, the driver re-resumes, the
        //     body returns, the chunk completes → the host VM thread finishes ---
        client.send_request(50, "continue", json!({ "threadId": thread_id }));
        let cont_ack = client.recv_until(|m| is_response(m, "continue"));
        assert_eq!(cont_ack["request_seq"], 50);
        assert_eq!(cont_ack["body"]["allThreadsContinued"], true);

        // The scenario runs to completion (R1.6): the host VM thread finishes and
        // drops the handle → Drop emits a final `Terminated`, which the encoder
        // turns into a DAP `terminated` event flushed to us (R3.5).
        let terminated = client.recv_until(|m| is_event(m, "terminated"));
        assert_eq!(terminated["event"], "terminated", "natural end emits terminated (R3.5)");

        // The host completed cleanly within the watchdog (no hang) and the VM
        // stayed usable (1+2==3 asserted on the host).
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion after continue (R1.6)");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    /// Request a `stackTrace` for `thread_id` (correlated to `seq`) and return
    /// the top frame's reported line. A small client-side helper so each step's
    /// EXACT stop line is asserted over the wire (DAP reports the position via
    /// `stackTrace`, not in the `stopped` event body).
    fn top_frame_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
        client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame");
        frames[0]["line"].as_u64().expect("top frame line") as u32
    }
}

#[cfg(test)]
mod source_map_wiring_tests {
    //! Task 4.2 — the gating that decides whether the I/O-side `.pasta` consumers
    //! (resolver 5.2 / BP translation 5.3) are reached. `pasta_active()` is the
    //! single gate: a map present AND `SourceMode::Pasta` (design 582). Otherwise
    //! the bridge keeps its default `.lua` behavior (requirements 6.1/6.2/7.2).

    use std::sync::Arc;

    use crate::debug::{SharedSourceMode, SourceMode};
    use crate::debug::source_map::SourceMap;

    use super::SourceMapWiring;

    /// 6.1 / design 582: a map present in `SourceMode::Pasta` activates the
    /// `.pasta` consumers (`pasta_active() == true`).
    #[test]
    fn pasta_active_when_map_present_and_mode_pasta() {
        let wiring = SourceMapWiring {
            source_map: Some(Arc::new(SourceMap::new())),
            source_mode: SharedSourceMode::new(SourceMode::Pasta),
        };
        assert!(
            wiring.pasta_active(),
            "Some(map) + Pasta must activate the `.pasta` consumers (6.1)"
        );
    }

    /// 6.2 / 7.2: `SourceMode::Lua` keeps the default `.lua` behavior even if a map
    /// were present (the gate is false).
    #[test]
    fn not_active_in_lua_mode_even_with_map() {
        let wiring = SourceMapWiring {
            source_map: Some(Arc::new(SourceMap::new())),
            source_mode: SharedSourceMode::new(SourceMode::Lua),
        };
        assert!(
            !wiring.pasta_active(),
            "`.lua` mode must NOT activate `.pasta` consumers (6.2)"
        );
    }

    /// 7.2: with no map the gate is false regardless of mode — the existing call
    /// sites (all pass `None`) behave exactly as today.
    #[test]
    fn not_active_without_map() {
        assert!(!SourceMapWiring::disabled().pasta_active());
        let pasta_no_map = SourceMapWiring {
            source_map: None,
            source_mode: SharedSourceMode::new(SourceMode::Pasta),
        };
        assert!(
            !pasta_no_map.pasta_active(),
            "no map → default `.lua` behavior even in Pasta mode (7.2)"
        );
    }
}

#[cfg(test)]
mod resolver_attach_tests {
    //! Task 5.2 — the `.pasta` source RESOLVER attachment on the socket-bridge's
    //! shared [`DapAdapter`]. [`attach_pasta_resolver`] installs
    //! [`pasta_source_resolver`](crate::debug::dap::pasta_source_resolver) IFF
    //! `pasta_active()` (a map present AND `SourceMode::Pasta`, design 509/582);
    //! otherwise the adapter keeps its default `.lua` resolver (R6.2 / 7.2).

    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use crate::debug::{SharedSourceMode, SourceMode};
    use crate::debug::dap::DapAdapter;
    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
    use crate::debug::types::{FrameInfo, SessionEvent};

    use super::{SourceMapWiring, attach_pasta_resolver};

    /// 既知 `chunk(.lua line) → .pasta` 対応を 1 件持つ集約 `SourceMap`。
    fn map_with(chunk: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
        let mut forward = BTreeMap::new();
        forward.insert(
            lua_line,
            PastaPos {
                file: file.to_string(),
                line: pasta_line,
            },
        );
        let mut sm = SourceMap::new();
        sm.insert_chunk(
            chunk.to_string(),
            file.to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        sm
    }

    /// `stackTrace` を 1 回エンコードし、top フレームの `source` / `line` を返す
    /// 小ヘルパ（装着結果の提示を観測する）。
    fn top_frame(adapter: &Arc<Mutex<DapAdapter>>, source: &str, line: u32) -> (serde_json::Value, u32) {
        let mut dap = adapter.lock().unwrap();
        dap.decode_request(&json!({
            "seq": 1, "type": "request", "command": "stackTrace",
            "arguments": { "threadId": 1 },
        }));
        let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
            source: source.to_string(),
            line,
            func_name: Some("f".to_string()),
        }]));
        let frame = &out[0]["body"]["stackFrames"][0];
        (frame["source"].clone(), frame["line"].as_u64().unwrap() as u32)
    }

    /// R5.1/R5.2/6.1: map present + `SourceMode::Pasta` → resolver 装着。対応あり
    /// フレームが `.pasta` 提示になる。
    #[test]
    fn attaches_pasta_resolver_when_active() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        let wiring = SourceMapWiring {
            source_map: Some(Arc::new(map_with(
                "C:/proj/cache/scene.lua",
                7,
                "C:/proj/scene.pasta",
                3,
            ))),
            source_mode: SharedSourceMode::new(SourceMode::Pasta),
        };
        attach_pasta_resolver(&adapter, &wiring);

        let (source, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            source,
            json!({ "path": "C:/proj/scene.pasta" }),
            "active → 対応ありフレームは `.pasta` 提示 (R5.1/R5.2)"
        );
        assert_eq!(line, 3);
    }

    /// R5.3: 装着済みでも対応の無いフレームは `.lua` フォールバック（判別可能）。
    #[test]
    fn attached_resolver_falls_back_to_lua_for_unmapped() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        let wiring = SourceMapWiring {
            source_map: Some(Arc::new(map_with(
                "C:/proj/cache/scene.lua",
                7,
                "C:/proj/scene.pasta",
                3,
            ))),
            source_mode: SharedSourceMode::new(SourceMode::Pasta),
        };
        attach_pasta_resolver(&adapter, &wiring);

        // `.lua` 行 2 は未対応 → 生成 `.lua` 提示のまま。
        let (source, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 2);
        assert_eq!(source, json!({ "path": r"@C:\proj\cache\scene.lua" }));
        assert_eq!(line, 2);
    }

    /// R6.2 / 7.2: `SourceMode::Lua`（map あり）→ 非装着。既定 `.lua` resolver の
    /// まま、生成 `.lua` 提示（バイト不変）。
    #[test]
    fn does_not_attach_in_lua_mode() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        let wiring = SourceMapWiring {
            source_map: Some(Arc::new(map_with(
                "C:/proj/cache/scene.lua",
                7,
                "C:/proj/scene.pasta",
                3,
            ))),
            source_mode: SharedSourceMode::new(SourceMode::Lua),
        };
        attach_pasta_resolver(&adapter, &wiring);

        // Lua モードでは対応があっても `.pasta` 化されない（既定 `.lua` のまま）。
        let (source, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            source,
            json!({ "path": r"@C:\proj\cache\scene.lua" }),
            "R6.2: Lua モードは既定 `.lua` resolver のまま（非装着）"
        );
        assert_eq!(line, 7);
    }

    /// 7.2: map なし → 非装着（既存呼び出し全てが `None` を渡す経路と同一の挙動）。
    #[test]
    fn does_not_attach_without_map() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        attach_pasta_resolver(&adapter, &SourceMapWiring::disabled());

        let (source, line) = top_frame(&adapter, "@scene.lua", 7);
        assert_eq!(source, json!({ "path": "@scene.lua" }), "no map → 既定 `.lua`");
        assert_eq!(line, 7);
    }
}

#[cfg(test)]
mod attach_source_presentation_tests {
    //! Task 5.5 — the DAP `attach` `sourcePresentation` applied to the SESSION's
    //! resolver presentation (task 5.2) via the shared effective mode
    //! ([`SharedSourceMode`]). `handle_inbound`, on an `attach` whose `Decoded`
    //! carries `attach_source_mode`, WRITES the shared mode and RE-RUNS
    //! [`attach_pasta_resolver`] so the DAP source resolver matches the FINAL
    //! effective mode (requirement 6.3 / design 581/586). A missing arg leaves the
    //! resolved env > file > 既定 mode untouched.
    //!
    //! These tests drive the resolver-attachment + shared-mode decision DIRECTLY
    //! (the unit-testable core of the integration); the VM-thread step granularity
    //! half is covered in `session.rs` (`attach_*` tests via [`SharedSourceMode`]).

    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use crate::debug::dap::DapAdapter;
    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
    use crate::debug::types::{FrameInfo, SessionEvent};
    use crate::debug::{SharedSourceMode, SourceMode};

    use super::{SourceMapWiring, attach_pasta_resolver};

    fn map_with(chunk: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
        let mut forward = BTreeMap::new();
        forward.insert(
            lua_line,
            PastaPos {
                file: file.to_string(),
                line: pasta_line,
            },
        );
        let mut sm = SourceMap::new();
        sm.insert_chunk(
            chunk.to_string(),
            file.to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        sm
    }

    /// Top frame `source`/`line` after encoding ONE `stackTrace` (observes which
    /// resolver is installed).
    fn top_frame(adapter: &Arc<Mutex<DapAdapter>>, source: &str, line: u32) -> (serde_json::Value, u32) {
        let mut dap = adapter.lock().unwrap();
        dap.decode_request(&json!({
            "seq": 1, "type": "request", "command": "stackTrace",
            "arguments": { "threadId": 1 },
        }));
        let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
            source: source.to_string(),
            line,
            func_name: Some("f".to_string()),
        }]));
        let frame = &out[0]["body"]["stackFrames"][0];
        (frame["source"].clone(), frame["line"].as_u64().unwrap() as u32)
    }

    /// Pasta-capable wiring (map present) whose EFFECTIVE mode starts at `start`.
    fn wiring_with(map: SourceMap, start: SourceMode) -> SourceMapWiring {
        SourceMapWiring {
            source_map: Some(Arc::new(map)),
            source_mode: SharedSourceMode::new(start),
        }
    }

    /// Simulate the `handle_inbound` attach-apply step: write the shared mode and
    /// re-run the resolver attachment (the exact two operations `handle_inbound`
    /// performs when `Decoded.attach_source_mode` is `Some`).
    fn apply_attach(adapter: &Arc<Mutex<DapAdapter>>, wiring: &SourceMapWiring, mode: SourceMode) {
        wiring.source_mode.set(mode);
        attach_pasta_resolver(adapter, wiring);
    }

    /// R6.3 / precedence attach > env > file: `attach sourcePresentation="lua"`
    /// FORCES `.lua` presentation even when the server default/file is `.pasta`.
    /// After the flip, the resolver is the default `.lua` (NOT `.pasta`).
    #[test]
    fn attach_lua_forces_lua_resolver_over_pasta_default() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        // Server default is Pasta (map present): the `.pasta` resolver is attached.
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Pasta,
        );
        attach_pasta_resolver(&adapter, &wiring);
        // Precondition: Pasta default presents `.pasta`.
        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(src, json!({ "path": "C:/proj/scene.pasta" }));
        assert_eq!(line, 3);

        // attach sourcePresentation="lua" → flip to Lua.
        apply_attach(&adapter, &wiring, SourceMode::Lua);

        // Now the SAME frame is presented as the generated `.lua` (NOT `.pasta`).
        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            src,
            json!({ "path": r"@C:\proj\cache\scene.lua" }),
            "attach `lua` must force `.lua` presentation over the Pasta default (R6.3)"
        );
        assert_eq!(line, 7);
        assert_eq!(wiring.source_mode.get(), SourceMode::Lua);
        assert!(!wiring.pasta_active(), "Lua effective mode → consumers inactive");
    }

    /// R6.3: `attach sourcePresentation="pasta"` FORCES `.pasta` presentation even
    /// when the server default/file is `.lua` (a map IS present). The `.pasta`
    /// resolver is attached after the flip.
    #[test]
    fn attach_pasta_forces_pasta_resolver_over_lua_default() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        // Server default is Lua (map present but gated off): default `.lua`.
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Lua,
        );
        attach_pasta_resolver(&adapter, &wiring);
        // Precondition: Lua default presents the generated `.lua`.
        let (src, _line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(src, json!({ "path": r"@C:\proj\cache\scene.lua" }));

        // attach sourcePresentation="pasta" → flip to Pasta (map present).
        apply_attach(&adapter, &wiring, SourceMode::Pasta);

        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            src,
            json!({ "path": "C:/proj/scene.pasta" }),
            "attach `pasta` must force `.pasta` presentation over the Lua default (R6.3)"
        );
        assert_eq!(line, 3);
        assert!(wiring.pasta_active(), "Pasta effective mode + map → consumers active");
    }

    /// design 581 (no client-default override): with NO attach `sourcePresentation`
    /// the resolved env > file > 既定 mode stays in effect — the resolver is NOT
    /// touched by the (absent) attach arg.
    #[test]
    fn no_attach_arg_keeps_resolved_mode() {
        let adapter = Arc::new(Mutex::new(DapAdapter::new()));
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Pasta,
        );
        attach_pasta_resolver(&adapter, &wiring);
        // No apply_attach call (Decoded.attach_source_mode == None → skipped).
        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            src,
            json!({ "path": "C:/proj/scene.pasta" }),
            "absent attach arg keeps the resolved Pasta mode (design 581)"
        );
        assert_eq!(line, 3);
        assert_eq!(wiring.source_mode.get(), SourceMode::Pasta);
    }
}

#[cfg(test)]
mod bp_translator_tests {
    //! Task 5.3 — `.pasta` ブレークポイント翻訳と最近接調整
    //! ([`translate_pasta_breakpoints`]・design "BpTranslator" 511-528・Flow 2
    //! 215-236・requirements 4.1 / 4.2 / 4.3 / 8.2).
    //!
    //! `.pasta` 行 BP を `resolve_pasta_to_lua` で `.lua` 実行座標群へ翻訳して登録
    //! （4.1・複数 `.lua` 行は全登録 8.2）、対応なしは
    //! `nearest_pasta_line_with_mapping` で後続最近接へ調整して `verified`＋調整後
    //! 行を返す（4.3）。登録した `.pasta` BP が **生フック座標**で `should_pause`
    //! を発火することも end-to-end で検証する（5.1 review のキー再整合・4.2）。

    use std::collections::BTreeMap;
    use std::sync::Arc;

    use crate::debug::{SharedSourceMode, SourceMode};
    use crate::debug::breakpoints::BreakpointSet;
    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
    use crate::debug::types::SourceRef;

    use super::{SourceMapWiring, is_pasta_source, translate_pasta_breakpoints};

    /// design "BpTranslator" 514: `.pasta` source は翻訳経路、`.lua` source は直接
    /// 登録経路。拡張子で判別する（大小無視）。
    #[test]
    fn is_pasta_source_detects_pasta_extension() {
        assert!(is_pasta_source("C:/proj/scene.pasta"));
        assert!(is_pasta_source(r"C:\proj\scene.PASTA"), "拡張子は大小無視");
        assert!(is_pasta_source("scene.pasta"));
        // `.lua` source（フック源 `@` 付きを含む）は翻訳しない → 直接登録経路。
        assert!(!is_pasta_source("@e2e_scenario"));
        assert!(!is_pasta_source("C:/proj/cache/scene.lua"));
        assert!(!is_pasta_source(r"@C:\proj\cache\scene.lua"));
    }

    /// 集約 `SourceMap` を、`.pasta` ファイル → [(chunk, lua_line, pasta_line)] の
    /// 列から構築する小ヘルパ。各 (chunk, lua_line) を `pasta_line` へ対応づける。
    fn map_from(file: &str, entries: &[(&str, u32, u32)]) -> SourceMap {
        let mut per_chunk: BTreeMap<String, BTreeMap<u32, PastaPos>> = BTreeMap::new();
        for &(chunk, lua_line, pasta_line) in entries {
            per_chunk.entry(chunk.to_string()).or_default().insert(
                lua_line,
                PastaPos {
                    file: file.to_string(),
                    line: pasta_line,
                },
            );
        }
        let mut sm = SourceMap::new();
        for (chunk, forward) in per_chunk {
            sm.insert_chunk(chunk, file.to_string(), ChunkSourceMap::from_forward(forward));
        }
        sm
    }

    /// Pasta-active な wiring を構築する。
    fn pasta_wiring(map: SourceMap) -> SourceMapWiring {
        SourceMapWiring {
            source_map: Some(Arc::new(map)),
            source_mode: SharedSourceMode::new(SourceMode::Pasta),
        }
    }

    /// 4.1 / 8.2 / 4.2: `.pasta` 行 → 対応 `.lua` 行群を全登録し、`verified`＋元行を
    /// 返す。さらに登録された各 `.lua` 実行座標が **生フック source**（`@`・`\`）で
    /// `should_pause` を発火する（canonicalize 再整合の end-to-end 証明・4.2）。
    #[test]
    fn pasta_line_registers_all_lua_lines_and_fires_should_pause() {
        // `.pasta` 行 7 → `.lua` 12, 13（同一 chunk・1→多展開・8.2）。
        let file = "C:/proj/scene.pasta";
        let map = map_from(
            file,
            &[
                ("C:/proj/cache/scene.lua", 12, 7),
                ("C:/proj/cache/scene.lua", 13, 7),
            ],
        );
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        let resolved =
            translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[7]);

        // 応答: 1 件・verified・元の `.pasta` 行 7（4.1）。
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].verified, "対応ありは verified (4.1)");
        assert_eq!(resolved[0].line, 7, "verified 行は元の `.pasta` 行");
        assert_eq!(resolved[0].source, SourceRef::new(file));

        // 4.2 end-to-end: 生フック座標（`@`・`\`・大小違い）で両 `.lua` 行が発火する。
        // 登録 chunk は `resolve_pasta_to_lua` の正規化済み chunk だが、
        // `should_pause` が両側を canonicalize するため一致する。
        assert!(
            set.should_pause(r"@C:\proj\cache\scene.lua", 12),
            "`.pasta` 行 7 の `.lua` 行 12 が生フック座標で発火する (4.2/8.2)"
        );
        assert!(
            set.should_pause(r"@C:\proj\cache\scene.lua", 13),
            "`.pasta` 行 7 の `.lua` 行 13 が生フック座標で発火する (4.2/8.2)"
        );
        // 未登録 `.lua` 行は発火しない。
        assert!(!set.should_pause(r"@C:\proj\cache\scene.lua", 11));
    }

    /// 4.3 最近接調整: 対応の無い `.pasta` 行は後続最近接の対応行へ調整され、その
    /// `.lua` 座標で登録、応答は `verified`＋**調整後**行。
    #[test]
    fn unmapped_pasta_line_adjusts_to_nearest_subsequent() {
        // 対応 `.pasta` 行 = {3, 7}。`.pasta` 行 4（対応なし）→ 最近接 7 へ調整。
        let file = "C:/proj/scene.pasta";
        let map = map_from(
            file,
            &[
                ("C:/proj/cache/scene.lua", 10, 3),
                ("C:/proj/cache/scene.lua", 20, 7),
            ],
        );
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        let resolved =
            translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[4]);

        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].verified, "調整後は verified (4.3)");
        assert_eq!(
            resolved[0].line, 7,
            "対応なし行 4 は後続最近接の対応行 7 へ調整される (4.3)"
        );
        // 調整先 `.pasta` 行 7 の `.lua` 行 20 が登録・発火する。
        assert!(
            set.should_pause(r"@C:\proj\cache\scene.lua", 20),
            "調整後の `.pasta` 行 7 の `.lua` 座標で停止する (4.3)"
        );
        // 元の対応行 3 の `.lua` 10 は（行 4 のリクエストでは）登録されない。
        assert!(!set.should_pause(r"@C:\proj\cache\scene.lua", 10));
    }

    /// 4.3: 後続に対応行が一切無い `.pasta` 行は **誤マッピングせず** unverified を
    /// 返す（最近接が存在しない場合のみ）。
    #[test]
    fn no_subsequent_mapping_returns_unverified() {
        let file = "C:/proj/scene.pasta";
        // 対応 `.pasta` 行 = {3}。行 5 以降に対応なし。
        let map = map_from(file, &[("C:/proj/cache/scene.lua", 10, 3)]);
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        let resolved =
            translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[5]);

        assert_eq!(resolved.len(), 1);
        assert!(
            !resolved[0].verified,
            "後続最近接が無い場合は unverified（誤マッピング禁止・4.3）"
        );
        assert_eq!(resolved[0].line, 5, "unverified は元の行を保持");
        // 何も登録されない。
        assert!(!set.should_pause(r"@C:\proj\cache\scene.lua", 10));
    }

    /// 4.1 / 8.2: 複数 `.pasta` 行を 1 回の setBreakpoints で要求した場合、全行分の
    /// `.lua` 実行座標が **同一 present source** で蓄積登録され、行ごとに後勝ち
    /// eviction しない（1 register 呼び出しに集約）。
    #[test]
    fn multiple_pasta_lines_all_register_without_mutual_eviction() {
        let file = "C:/proj/scene.pasta";
        // `.pasta` 3 → `.lua` 10、`.pasta` 7 → `.lua` 20, 21。
        let map = map_from(
            file,
            &[
                ("C:/proj/cache/scene.lua", 10, 3),
                ("C:/proj/cache/scene.lua", 20, 7),
                ("C:/proj/cache/scene.lua", 21, 7),
            ],
        );
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        let resolved =
            translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[3, 7]);

        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().all(|r| r.verified));
        assert_eq!(resolved[0].line, 3);
        assert_eq!(resolved[1].line, 7);

        // 全 `.lua` 座標が同時に登録されている（行ごとの register で互いを評価
        // 退避していない）。
        assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 10), "行 3 の座標");
        assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 20), "行 7 の座標 1");
        assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 21), "行 7 の座標 2");
    }

    /// 4.1 / 多チャンク: 同一 `.pasta` 行が複数チャンクへ展開される場合も全チャンクの
    /// `.lua` 座標を登録する（design 215-236 のクロスチャンク・8.2）。
    #[test]
    fn pasta_line_spanning_multiple_chunks_registers_all() {
        let file = "C:/proj/scene.pasta";
        // `.pasta` 行 7 → chunk a の `.lua` 12, chunk b の `.lua` 5。
        let map = map_from(
            file,
            &[
                ("C:/proj/cache/a.lua", 12, 7),
                ("C:/proj/cache/b.lua", 5, 7),
            ],
        );
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        let resolved =
            translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[7]);

        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].verified);
        assert!(set.should_pause(r"@C:\proj\cache\a.lua", 12), "chunk a の座標");
        assert!(set.should_pause(r"@C:\proj\cache\b.lua", 5), "chunk b の座標");
    }

    /// `.pasta` present source は retain/置換キー: 同 `.pasta` の再 setBreakpoints は
    /// 旧座標を置換しつつ、別 present source（別 `.pasta`/`.lua`）の BP を保持する。
    /// requirements 4.4 / 8.2（present source 単位の権威的置換）。
    #[test]
    fn re_setting_pasta_source_replaces_only_its_own_coords() {
        let file = "C:/proj/scene.pasta";
        let map = map_from(
            file,
            &[
                ("C:/proj/cache/scene.lua", 10, 3),
                ("C:/proj/cache/scene.lua", 20, 7),
            ],
        );
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        // 行 3（→ `.lua` 10）を登録。
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[3]);
        assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 10));

        // 同 `.pasta` を行 7（→ `.lua` 20）で再設定。旧座標 10 は置換される。
        translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[7]);
        assert!(
            !set.should_pause(r"@C:\proj\cache\scene.lua", 10),
            "同一 present source の旧座標は権威的に置換される"
        );
        assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 20), "新座標が登録される");
    }

    /// 4.4 / 8.2: 空行リスト（DAP の「この source の BP を全消去」）での再設定は
    /// 当該 present source の既存座標を権威的に除去し、空の resolved を返す。
    /// [`BreakpointSet::register`] の置換セマンティクスが空 entries でも成立する
    /// ことを翻訳経路越しに固定する。
    #[test]
    fn re_setting_pasta_source_with_empty_lines_clears_its_coords() {
        let file = "C:/proj/scene.pasta";
        let map = map_from(file, &[("C:/proj/cache/scene.lua", 10, 3)]);
        let wiring = pasta_wiring(map);
        let set = BreakpointSet::new();

        // 行 3（→ `.lua` 10）を登録して発火を確認。
        let resolved = translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[3]);
        assert_eq!(resolved.len(), 1);
        assert!(set.should_pause(r"@C:\proj\cache\scene.lua", 10));

        // 同 `.pasta` を空行リストで再設定: resolved は空、旧座標は全除去。
        let cleared = translate_pasta_breakpoints(&set, &wiring, &SourceRef::new(file), &[]);
        assert!(cleared.is_empty(), "空要求の resolved は空集合");
        assert!(
            !set.should_pause(r"@C:\proj\cache\scene.lua", 10),
            "空再設定は当該 present source の全座標を権威的に除去する"
        );
    }

    /// [`translate_pasta_breakpoints`] 冒頭の防御分岐: `pasta_active()` が保証する
    /// はずの map が万一 `None` でも `.lua` 直接登録経路
    /// （[`BreakpointSet::set_breakpoints`]）へ安全劣化し、ブリッジは決して
    /// パニックしない（要求行どおり verified・present source 自身が実行座標）。
    #[test]
    fn translate_without_map_degrades_to_direct_lua_registration() {
        let wiring = SourceMapWiring {
            source_map: None,
            source_mode: SharedSourceMode::new(SourceMode::Pasta),
        };
        let set = BreakpointSet::new();
        let src = SourceRef::new("C:/proj/scene.pasta");

        let resolved = translate_pasta_breakpoints(&set, &wiring, &src, &[4, 9]);

        // `.lua` 直接経路の形: 要求順ミラー・全 verified・原行のまま。
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().all(|bp| bp.verified), "直接経路は全行 verified");
        assert_eq!(resolved[0].line, 4);
        assert_eq!(resolved[1].line, 9);
        // present source == chunk として直接登録される（翻訳なし）。
        assert!(set.should_pause("C:/proj/scene.pasta", 4));
        assert!(set.should_pause("C:/proj/scene.pasta", 9));
    }
}

#[cfg(test)]
mod pasta_bp_e2e {
    //! Task 7.1 — `.pasta` ブレークポイント **E2E**（実トランスパイル → 実マップ構築
    //! → 実デバッグセッション）。
    //!
    //! 既存の `.lua` E2E（[`super::tests::full_dap_session_over_tcp_*`] / 4.1）を
    //! **本番化**し、`.pasta` 座標経路を end-to-end で駆動する（design 637-639「既存
    //! スライス E2E を本番化」）。同じ DAP-over-TCP ハーネス（[`super::tests`] と同型の
    //! `DapClient`）で、ローダの本番マップ構築経路（task 4.3
    //! [`PastaLoader::build_source_map`]）が産んだ集約 `Arc<SourceMap>` を
    //! [`enable`](crate::debug::enable)（task 4.2）へ `SourceMode::Pasta` で渡し、実際の
    //! 生成 `.lua` を当該キャッシュ `.lua` パスをチャンク名（`@<path>`）として VM 上で
    //! 走らせる。
    //!
    //! 観測する「done」（task 7.1 完了状態）:
    //! 1. **`.pasta` 行 BP ヒット（4.1/4.2）**: `.pasta` のマップ済み行へ BP を張ると
    //!    `verified`＝true で登録され、対応する `.lua` 行に到達して停止する。
    //! 2. **`.pasta` 提示（5.1/5.2）**: 停止フレームの `stackTrace` が **`.pasta`** の
    //!    ファイル・行を提示する（生成 `.lua` 座標**ではない**）。これがテストの「歯」:
    //!    resolver/翻訳が無効なら `.lua` 座標が出てこのアサートが落ちる。
    //! 3. **停止中 inspect 継続（5.4）**: `.pasta` 提示中も `scopes`/`variables` が
    //!    機能する（提示モードは inspect に影響しない）。
    //! 4. **最近接調整（4.3）**: マップの無い `.pasta` 行（先頭コメント行）へ BP を張ると
    //!    後続最近接のマップ済み `.pasta` 行へ調整され、`verified`＋調整後 `line` が返る。
    //!
    //! `mlua::Lua`（`!Send`）は VM ホストスレッドにのみ生存し、バウンド `SocketAddr`
    //! （`Copy`）と go/done チャネルだけが越境する。全クライアント待機は TEST-ONLY
    //! watchdog でバウンドし CI がハングしないようにする（停止コアは無期限）。

    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::Arc;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::source_map::{MapBuilderSink, SourceMap};
    use crate::debug::transport::{read_frame, write_frame};
    use crate::debug::{DebugConfig, SourceMode, enable};
    use crate::loader::CacheManager;
    use crate::transpiler::LuaTranspiler;

    /// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// 最小 `.pasta` フィクスチャ（グローバル単語定義 3 本）。先頭行はコメント（マップ
    /// 無し → 4.3 最近接調整の対象）。各単語定義行はマップ済みで、生成 `.lua` の
    /// トップレベル文（`PASTA.create_word(...):entry(...)`）として **実行**されるため、
    /// その行に張った `.pasta` BP が実フックで発火する。
    ///
    /// `.pasta` 行（1-origin）:
    ///   1: ＃グローバル単語          <- コメント（マップ無し → 4.3 で行2へ調整）
    ///   2: ＠あいさつ：こんにちは、やあ   <- マップ済み（→ 生成 `.lua` のある行）
    ///   3: ＠べつ：A、B、C            <- マップ済み（BP 対象）
    ///   4: ＠みっつ：x、y             <- マップ済み
    const PASTA_FIXTURE: &str = "\
＃グローバル単語
＠あいさつ：こんにちは、やあ
＠べつ：A、B、C
＠みっつ：x、y
";

    /// `require "pasta"` / `require "pasta.global"` を満たす最小 Lua シム。
    /// `PASTA.create_word(name)` は `entry(...)` を持つオブジェクトを返すので、生成
    /// `.lua` のトップレベル単語定義文が **副作用付きで実行**でき、各行で実フックが
    /// 発火する（BP ヒット観測に必要）。`.pasta`↔`.lua` 変換とは無関係の純粋な実行
    /// 足場であり、提示は resolver/翻訳（5.2/5.3）が担う。
    const PASTA_SHIM: &str = "\
local word = {}
word.__index = word
function word:entry(...) self.entries = { ... } return self end
local PASTA = {}
function PASTA.create_word(name) return setmetatable({ name = name }, word) end
package.loaded['pasta'] = PASTA
package.loaded['pasta.global'] = {}
";

    /// 実 TCP ソケット越しの最小 DAP クライアント（Content-Length フレーミング）。
    /// [`super::tests::DapClient`] と同型（モジュール独立のため自前に持つ）。
    struct DapClient {
        reader: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl DapClient {
        fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone socket for writing");
            Self {
                reader: BufReader::new(stream),
                writer,
            }
        }

        fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
            let req = json!({
                "seq": seq,
                "type": "request",
                "command": command,
                "arguments": arguments,
            });
            write_frame(&mut self.writer, &req).expect("client write must succeed");
        }

        fn recv(&mut self) -> Value {
            read_frame(&mut self.reader)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present (peer did not close)")
        }

        fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
            loop {
                let msg = self.recv();
                if pred(&msg) {
                    return msg;
                }
            }
        }
    }

    fn is_event(msg: &Value, name: &str) -> bool {
        msg["type"] == "event" && msg["event"] == name
    }

    fn is_response(msg: &Value, command: &str) -> bool {
        msg["type"] == "response" && msg["command"] == command
    }

    /// 実トランスパイルの成果物（E2E が必要とする実座標を **マップから導出**）。
    struct Fixture {
        /// ローダの本番経路が産んだ集約マップ（`enable` へ渡す）。
        map: Arc<SourceMap>,
        /// 実行する生成 `.lua` 本文（map が構築されたのと同一バイト）。
        lua_source: String,
        /// VM チャンク名（`@<キャッシュ .lua 絶対パス>`）= ローダ由来チャンク名。
        chunk_name: String,
        /// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致）。
        pasta_path: String,
        /// BP を張るマップ済み `.pasta` 行（map から導出）。
        mapped_pasta_line: u32,
        /// `mapped_pasta_line` に対応する `.lua` 実行行（停止位置の根拠）。
        mapped_lua_line: u32,
        /// マップの無い `.pasta` 行（4.3 最近接調整の入力）。
        unmapped_pasta_line: u32,
        /// `unmapped_pasta_line` の後続最近接マップ済み `.pasta` 行（調整後の期待値）。
        nearest_adjusted_line: u32,
    }

    /// 実 `.pasta` をディスクへ書き、(a) 本番ローダ経路でマップを構築し
    /// （[`PastaLoader::build_source_map`]・task 4.3）、(b) 同一 `.pasta` を同一
    /// `transpile_with_source_map` で再トランスパイルして **実行する生成 `.lua` 本文**を
    /// 取得する。map のチャンク名キー（ローダ由来 `source_to_cache_path`）を VM の
    /// `set_name` に用いるため、停止座標とマップが一致する。
    ///
    /// BP 対象/調整対象の `.pasta` 行は **マップから導出**してハードコードを避ける
    /// （map の決定的順序に依存しない・回帰耐性）。
    fn build_fixture() -> Fixture {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let base_dir = temp.path().to_path_buf();
        let pasta_file = base_dir.join("dic/baseware/words.pasta");
        std::fs::create_dir_all(pasta_file.parent().unwrap()).expect("mkdir dic");
        std::fs::write(&pasta_file, PASTA_FIXTURE).expect("write .pasta");

        let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");

        // (a) 本番ローダ経路の集約マップ（sidecar=false: メモリ既定）。
        let map = crate::loader::PastaLoader::build_source_map(
            std::slice::from_ref(&pasta_file),
            &cache_manager,
            false,
        );

        // チャンク名キー = ローダ由来 `source_to_cache_path`（map のキーと同一）。
        let chunk_name = cache_manager
            .source_to_cache_path(&pasta_file)
            .to_string_lossy()
            .to_string();
        // `.pasta` ファイルキー = `build_source_map_inner` が `parse_str` / `PastaPos.file`
        // に用いる `file_path.to_string_lossy()`（VSCode source.path と一致させる側）。
        let pasta_path = pasta_file.to_string_lossy().to_string();

        // (b) 同一入力で再トランスパイルし、実行する生成 `.lua` 本文を得る（map が
        // 構築されたのと同一の決定的バイト）。
        let content = std::fs::read_to_string(&pasta_file).expect("read .pasta");
        let parsed = pasta_dsl::parse_str(&content, &pasta_path).expect("parse .pasta");
        let transpiler = LuaTranspiler::default();
        let mut sink = MapBuilderSink::new(pasta_path.clone(), chunk_name.clone());
        let mut out = Vec::new();
        transpiler
            .transpile_with_source_map(&parsed, &mut out, Some(&mut sink))
            .expect("transpile .pasta");
        let lua_source = String::from_utf8(out).expect("utf8 .lua");

        // --- 実座標をマップから導出 ---
        // マップ済み `.pasta` 行（と対応 `.lua` 行）を、`.lua`→`.pasta` 前方解決の
        // 昇順走査から収集する。
        let mut mapped: Vec<(u32, u32)> = Vec::new(); // (pasta_line, lua_line)
        for lua_line in 1u32..=200 {
            if let Some(pos) = map.resolve_lua_to_pasta(&chunk_name, lua_line) {
                mapped.push((pos.line, lua_line));
            }
        }
        mapped.sort();
        assert!(
            mapped.len() >= 2,
            "フィクスチャは少なくとも 2 つのマップ済み `.pasta` 行を持つこと: {mapped:?}"
        );

        // BP 対象: 2 番目のマップ済み `.pasta` 行（先頭以外で安定して実行される行）。
        let (mapped_pasta_line, mapped_lua_line) = mapped[1];

        // マップの無い `.pasta` 行: 最小マップ済み行の手前の行（先頭コメント行など）。
        // その後続最近接マップ済み行は最小マップ済み行になる。
        let min_mapped = mapped[0].0;
        assert!(
            min_mapped >= 2,
            "最小マップ済み `.pasta` 行の手前にマップ無し行が必要（4.3 入力）: min={min_mapped}"
        );
        let unmapped_pasta_line = min_mapped - 1; // 先頭コメント行（マップ無し）。
        // 期待される調整先 = `from_line` 以上で最初にマップを持つ `.pasta` 行。
        let nearest_adjusted_line = map
            .nearest_pasta_line_with_mapping(&pasta_path, unmapped_pasta_line)
            .expect("マップ無し行には後続最近接のマップ済み行が存在すること");
        assert_eq!(
            nearest_adjusted_line, min_mapped,
            "未マップ行の後続最近接は最小マップ済み行（4.3）"
        );
        // 未マップ行自身が本当にマップを持たないことを表明（テストの前提）。
        assert!(
            map.resolve_pasta_to_lua(&pasta_path, unmapped_pasta_line)
                .is_empty(),
            "選んだ未マップ `.pasta` 行 {unmapped_pasta_line} は対応 `.lua` を持たないこと"
        );

        // TempDir をリークさせて寿命を伸ばす（map / lua_source は既に取得済みで
        // ディスクは不要だが、念のため明示）。drop で削除されても map は in-memory。
        drop(temp);

        Fixture {
            map,
            lua_source,
            chunk_name,
            pasta_path,
            mapped_pasta_line,
            mapped_lua_line,
            unmapped_pasta_line,
            nearest_adjusted_line,
        }
    }

    /// task 7.1 の本体: `.pasta` 行 BP ヒット・`.pasta` 提示・停止中 inspect・最近接調整を
    /// 1 本の実 DAP-over-TCP セッションで end-to-end 検証する。
    ///
    /// requirements: **4.1**（`.pasta` 行 BP → 対応 `.lua` 行群登録）/ **4.2**（対応
    /// `.lua` 行到達で停止）/ **4.3**（対応なし → 後続最近接へ調整・有効位置提示）/
    /// **5.1**（停止位置 `.pasta` 提示）/ **5.2**（コールスタック各フレーム `.pasta`
    /// 提示）/ **5.4**（`.pasta` 提示中も変数/コルーチン inspect 利用可能）。
    #[test]
    fn pasta_breakpoint_hits_presents_pasta_inspects_and_nearest_adjusts_over_tcp() {
        let fx = build_fixture();

        // host へ渡す値（`!Send` を越境させない）。
        let lua_source = fx.lua_source.clone();
        let chunk_name = fx.chunk_name.clone();
        let map = Arc::clone(&fx.map);

        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        // VM HOST スレッド: `mlua::Lua`（!Send）を専有。実マップを Pasta モードで
        // `enable` へ渡し、実生成 `.lua` をローダ由来チャンク名で走らせる。
        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };
            // `require "pasta"` を満たすシムを先に読み込む（フック未装着の準備実行）。
            lua.load(PASTA_SHIM)
                .set_name("@pasta_shim")
                .exec()
                .map_err(|e| format!("shim exec failed: {e}"))?;

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_mode: SourceMode::Pasta, // 既定だが明示（6.1）。
                ..Default::default()
            };
            // 実マップを `Some` で渡す（task 4.2・design 582: map+Pasta で `.pasta`
            // resolver/BP 翻訳/stepper が装着される）。
            let handle = enable(&lua, &cfg, Some(map))
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            // クライアントが setBreakpoints/configurationDone を終えるまで待つ。
            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            // 実生成 `.lua` を **ローダ由来チャンク名**（`@<キャッシュ .lua パス>`）で
            // 走らせる。フックはこの source を報告し、`should_pause` が正規化突合する
            // ので `.pasta` BP（→ `.lua` 行登録）が発火する（4.2）。
            lua.load(&lua_source)
                .set_name(format!("@{chunk_name}"))
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;
            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        // CLIENT（このスレッド）。
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // --- initialize → capabilities + `initialized` ---
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let init_resp = client.recv_until(|m| is_response(m, "initialize"));
        assert_eq!(init_resp["success"], true, "initialize must succeed");
        let _initialized = client.recv_until(|m| is_event(m, "initialized"));

        // --- (4.3) 最近接調整: マップ無し `.pasta` 行へ BP → verified＋調整後 line ---
        // 先に最近接調整を表明する（設定リクエストの応答だけで判定でき、VM 実行に
        // 依存しない）。同一 `.pasta` source なので後段のヒット用 BP で置換される。
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": fx.pasta_path },
                "breakpoints": [{ "line": fx.unmapped_pasta_line }],
            }),
        );
        let adj_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        let adj_bps = adj_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(adj_bps.len(), 1, "1 つの BP 応答");
        assert_eq!(
            adj_bps[0]["verified"], true,
            "4.3: マップ無し行 BP は後続最近接へ調整され verified になる"
        );
        assert_eq!(
            adj_bps[0]["line"].as_u64().expect("adjusted line") as u32,
            fx.nearest_adjusted_line,
            "4.3: 調整後の有効位置（後続最近接のマップ済み `.pasta` 行）が提示される"
        );

        // --- (4.1) `.pasta` のマップ済み行へ BP → verified（元の行で確定）---
        client.send_request(
            3,
            "setBreakpoints",
            json!({
                "source": { "path": fx.pasta_path },
                "breakpoints": [{ "line": fx.mapped_pasta_line }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 1);
        assert_eq!(
            bps[0]["verified"], true,
            "4.1: マップ済み `.pasta` 行 BP は verified で登録される"
        );
        assert_eq!(
            bps[0]["line"].as_u64().expect("bp line") as u32,
            fx.mapped_pasta_line,
            "4.1: マップ済み行は調整されず元の `.pasta` 行のまま"
        );

        // --- configurationDone → VM 実行開始 ---
        client.send_request(4, "configurationDone", json!({}));
        let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
        assert_eq!(cfg_resp["success"], true);
        go_tx.send(()).expect("send go signal");

        // --- (4.2) `.pasta` 行に対応する `.lua` 行へ到達 → stopped(breakpoint) ---
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "4.2: `.pasta` 行 BP に対応する `.lua` 行で停止する"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        // --- (5.1/5.2) stackTrace は **`.pasta`** 座標を提示する（`.lua` ではない）---
        client.send_request(10, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "停止フレームが存在する（5.2）");

        // === テストの「歯」: top フレームの source.path が `.pasta`・line が `.pasta`
        // 行であること。resolver/翻訳（5.2/5.3）が無効なら、ここに生成 `.lua` の
        // チャンク名（`@<...lua>`）と `.lua` 行が出てこのアサートが落ちる。===
        let top_src = frames[0]["source"]["path"]
            .as_str()
            .expect("top frame source path");
        assert!(
            top_src.ends_with(".pasta"),
            "5.1/5.2: 停止フレームは `.pasta` を提示すること（`.lua` ではない）。actual={top_src:?}"
        );
        // 正規化突合（区切り/大小）で `.pasta` パスと一致する。
        assert_eq!(
            crate::debug::source_map::canonicalize_chunk_name(top_src),
            crate::debug::source_map::canonicalize_chunk_name(&fx.pasta_path),
            "5.1: 提示 `.pasta` パスは元 `.pasta` ファイルと一致する"
        );
        assert_eq!(
            frames[0]["line"].as_u64().expect("top frame line") as u32,
            fx.mapped_pasta_line,
            "5.1: 提示行は **`.pasta` 行**（{}）。`.lua` 行（{}）であってはならない",
            fx.mapped_pasta_line,
            fx.mapped_lua_line
        );
        // `.pasta` 行と `.lua` 行が異なることを前提として確認（歯の有効性の裏付け）:
        // もし提示が `.lua` のままなら line==mapped_lua_line になり、上の assert が
        // 落ちる。両者が異なるフィクスチャを選んでいる。
        assert_ne!(
            fx.mapped_pasta_line, fx.mapped_lua_line,
            "フィクスチャは `.pasta` 行 ≠ `.lua` 行（提示差が観測可能）"
        );

        // --- (5.4) `.pasta` 提示中でも inspect（scopes/variables）が機能する ---
        let frame_id = frames[0]["id"].as_u64().expect("frame id");
        client.send_request(11, "scopes", json!({ "frameId": frame_id }));
        let scopes = client.recv_until(|m| is_response(m, "scopes"));
        assert_eq!(scopes["success"], true, "5.4: `.pasta` 提示中も scopes が成功する");
        let scope_arr = scopes["body"]["scopes"]
            .as_array()
            .expect("scopes array");
        assert!(
            !scope_arr.is_empty(),
            "5.4: 停止フレームの scope が利用可能（提示モードは inspect に影響しない）"
        );
        let var_ref = scope_arr[0]["variablesReference"]
            .as_u64()
            .expect("variablesReference");
        // variables 要求も成功する（中身の有無に依らず、提示モードで壊れないこと）。
        client.send_request(12, "variables", json!({ "variablesReference": var_ref }));
        let vars = client.recv_until(|m| is_response(m, "variables"));
        assert_eq!(
            vars["success"], true,
            "5.4: `.pasta` 提示中でも variables 要求が成功する（inspect 継続）"
        );
        assert!(
            vars["body"]["variables"].is_array(),
            "5.4: variables 応答は配列を返す（inspect 経路が機能している）"
        );

        // --- continue → 実行完走 → host スレッド終了（terminated 観測）---
        //
        // 注意: マップ済み行（単語定義）は生成 `.lua` 上で `create_word():entry()` の
        // **複数呼び出しを含む 1 行**であり、Lua のラインフックは同一ソース行で複数回
        // 発火し得る（呼び出しから戻る際に同じ行へ再入する）。よって 1 つの `.pasta`
        // 行 BP がその行で複数回停止し得る。これは行 BP の正当な挙動なので、`continue`
        // を繰り返して流し切り、chunk 完走 → `drop(handle)` が出す `terminated`（host が
        // 完了した決定的シグナル）を観測するまでループする。各 `continue` 後に来る
        // フレームは `stopped`（再停止 → もう一度 continue）か `terminated`（完了 →
        // 終了）のいずれか。ループ回数は BP 行の再入数で有限（CI 無限ループ防止に上限）。
        let mut terminated = false;
        for continue_seq in 30u64..60u64 {
            client.send_request(continue_seq, "continue", json!({ "threadId": thread_id }));
            // continue ack の後に来る次の制御フレーム（stopped/terminated）を待つ。
            let next = client.recv_until(|m| {
                is_event(m, "stopped") || is_event(m, "terminated")
            });
            if is_event(&next, "terminated") {
                terminated = true;
                break;
            }
            // それ以外は同一 BP 行への再停止 → もう一度 continue（reason は breakpoint）。
            assert_eq!(
                next["body"]["reason"], "breakpoint",
                "再停止は同一 `.pasta` 行 BP のはず（多重呼び出し行の再入）"
            );
        }
        assert!(
            terminated,
            "chunk 完走 → `terminated` が来るまで continue で流し切れること"
        );

        // host VM スレッドが watchdog 内で完了することを確認（ハング無し）。
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion after continue");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }
}

#[cfg(test)]
mod pasta_step_e2e {
    //! Task 7.2 — `.pasta` 粒度ステップ **E2E**（E1–E8）。実デバッグセッションを
    //! 実 DAP-over-TCP トランスポート越しに駆動し、`.pasta` 行単位のステップ実行
    //! （step over/into/out）が design 640 の 8 シナリオすべてで **期待どおりの
    //! `.pasta` 停止位置**へ到達することを end-to-end で検証する（requirements
    //! **9.1**/**9.2**/**9.3**/**9.4**/**9.5**）。
    //!
    //! # ハーネス（task 7.1 [`super::pasta_bp_e2e`] / `.lua` ステップ E2E
    //! [`super::tests::full_lua_debug_session_all_steps_all_var_types_coroutine_body`]
    //! の踏襲）
    //!
    //! - 実マップ＋`SourceMode::Pasta` を [`enable`](crate::debug::enable) へ渡し
    //!   （task 4.2／design 582: map+Pasta で `.pasta` resolver/BP 翻訳/stepper が装着）、
    //!   生成 `.lua`（ここでは決定的に制御するため**素の Lua チャンク**）を VM 上で走らせる。
    //! - クライアントは実 TCP ソケット越しに `next`/`stepIn`/`stepOut` を送り、各停止の
    //!   **`.pasta` 行**を `stackTrace` の top フレーム `line` で読む（DAP は停止位置を
    //!   `stopped` body ではなく `stackTrace` で報告する・task 7.1 と同型の [`top_pasta_line`]）。
    //!   resolver（task 5.2）が装着済みなので、`stackTrace` の `line` は `.pasta` 座標
    //!   （E1–E7）。
    //!
    //! # マップは「素の Lua チャンク」へ手組みする（task 5.4 unit test 流儀）
    //!
    //! `.pasta` トランスパイラの出力に依存せず E1–E8 の行構造（複数 `.lua`→同一
    //! `.pasta`、サブ呼び出し、再帰、未対応行、コルーチン）を **決定的に**作るため、
    //! [`super::super::session`] の task 5.4 unit test と同じく `ChunkSourceMap::
    //! from_forward` で `lua_line → PastaPos` を手組みした集約 [`SourceMap`] を注入する。
    //! これは task 5.4 が stop-decision ロジックで検証した**同じ振る舞い**を、本物の
    //! セッション（transport→dap→session→hook）で合成して end-to-end に証明する 7.2 の
    //! 役割そのもの。**期待停止 `.pasta` 行は map から導出**してハードコードを避ける
    //! （[`derive`](Expected::derive)・回帰耐性）。
    //!
    //! # 「歯」（teeth）
    //!
    //! 中核シナリオ（E1/E3/E6）について、`SourceMode::Lua` へ切り替えると `.pasta`
    //! 粒度が無効化されて `.lua` 行で停止する（`.pasta` 行ではない）ことを別テスト
    //! [`teeth_lua_mode_stops_at_lua_line_not_pasta`] で示し、アサートが**本物**
    //! （恒真でない）ことを裏づける。E8（9.5）はその `.lua` 粒度回帰そのもの。
    //!
    //! `mlua::Lua`（`!Send`）は VM ホストスレッドにのみ生存し、バウンド `SocketAddr`
    //! （`Copy`）と go/done チャネルだけが越境する。全クライアント待機は TEST-ONLY
    //! watchdog でバウンドし CI がハングしないようにする（停止コアは無期限）。

    use std::collections::BTreeMap;
    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::Arc;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
    use crate::debug::transport::{read_frame, write_frame};
    use crate::debug::{DebugConfig, SourceMode, enable};

    /// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// 生成 `.lua` チャンク名（フック source = `set_name` 値）。map のキーと一致させる。
    const STEP_SOURCE: &str = "@pasta_step_e2e_scenario";

    /// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致させる側）。
    /// `.pasta` 拡張子を持たせて [`super::is_pasta_source`] の BP 翻訳経路を通す。
    const STEP_PASTA_FILE: &str = "scene_step.pasta";

    /// E1–E8 を 1 本で覆う**素の Lua チャンク**。`.pasta` 行は手組みマップ
    /// （[`step_scenario_map`]）が与える。行番号（1-origin）と各行の役割:
    ///
    /// ```text
    ///   1: local function helper(x)
    ///   2:     local hy = x + 1     -- callee: 未対応（step into で通過・E3/9.4）
    ///   3:     local hz = hy + 1    -- callee: .pasta 30（step into 停止先・E3 / step out 起点）
    ///   4:     return hz
    ///   5: end
    ///   6: local function recur(n)
    ///   7:     if n > 0 then        -- .pasta 40（再帰: 別フレームで同一 .pasta 行・E5）
    ///   8:         return recur(n-1)-- .pasta 41（再帰呼び出し）
    ///   9:     end
    ///  10:     return 0
    ///  11: end
    ///  12: local body = function()
    ///  13:     local p = 1          -- .pasta 50（コルーチン本体・E7 step 起点）
    ///  14:     coroutine.yield()    -- .pasta 51（E7: step over で yield を跨ぐ）
    ///  15:     local q = p + 1      -- .pasta 52（E7: resume 後の停止先）
    ///  16:     return q
    ///  17: end
    ///  18: local a = 1              -- .pasta 10（BP / step over 起点・単一 .lua 行）
    ///  19: local b = a + 1          -- .pasta 11（E1: 複数 .lua 行へ展開された .pasta 行の 1 本目）
    ///  20: local c = b + 1          -- .pasta 11（E1: 同一 .pasta 11 の 2 本目 → 消化・9.1）
    ///  21: local g = c + 1          -- 未対応（通過・E6/9.4）
    ///  22: local d = helper(c)      -- .pasta 12（E1 step over 停止先 / E2 サブ呼び出し / E3 step into 起点）
    ///  23: local e = recur(2)       -- .pasta 13（E5 再帰呼び出し / E2 step over 停止先）
    ///  24: local f = e + 1          -- .pasta 14（E4 step out 停止先 / E5 step over 停止先）
    ///  25: local co = coroutine.create(body)
    ///  26: while coroutine.status(co) ~= 'dead' do  -- 駆動ループ（別スレッド・E7 で skip）
    ///  27:     coroutine.resume(co)
    ///  28: end
    ///  29: return f
    /// ```
    ///
    /// E1 の「複数 `.lua` 行を生む `.pasta` 行」は**起点ではなく**行19/20（`.pasta` 11）に
    /// 置く（起点 `.pasta` 10 は単一 `.lua` 行18）。これにより `.pasta` 行 BP が起点 1 本
    /// だけを登録し、step over 中に同一 BP 行へ再入して `breakpoint` で再停止する事故を
    /// 避けつつ、step over が `.pasta` 11 の 2 本（行19/20）を消化することを観測できる。
    const STEP_CHUNK: &str = "\
local function helper(x)
    local hy = x + 1
    local hz = hy + 1
    return hz
end
local function recur(n)
    if n > 0 then
        return recur(n - 1)
    end
    return 0
end
local body = function()
    local p = 1
    coroutine.yield()
    local q = p + 1
    return q
end
local a = 1
local b = a + 1
local c = b + 1
local g = c + 1
local d = helper(c)
local e = recur(2)
local f = e + 1
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
return f
";

    /// `STEP_CHUNK` の手組み `SourceMap`（task 5.4 unit test と同流儀・
    /// `ChunkSourceMap::from_forward`）。フック source 名でキーし、map が内部で
    /// 正規化する（task 3.4）。各 `(lua_line → .pasta line)` は上記コメントの対応表。
    fn step_scenario_map() -> Arc<SourceMap> {
        let pp = |line: u32| PastaPos {
            file: STEP_PASTA_FILE.to_string(),
            line,
        };
        let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
        // helper 本体（callee）
        // 行2 は意図的に未対応（step into で通過・E3/9.4）
        forward.insert(3, pp(30)); // step into 停止先 / step out 起点
        forward.insert(4, pp(31));
        // recur 本体（再帰: 別フレームで同一 .pasta 行を踏む・E5）
        forward.insert(7, pp(40));
        forward.insert(8, pp(41));
        forward.insert(10, pp(42));
        // body（コルーチン本体・E7）
        forward.insert(13, pp(50));
        forward.insert(14, pp(51));
        forward.insert(15, pp(52));
        forward.insert(16, pp(53));
        // トップレベル（caller フレーム）
        forward.insert(18, pp(10)); // BP / step over 起点（単一 .lua 行）
        forward.insert(19, pp(11)); // E1: .pasta 11 の 1 本目（複数 .lua 行展開）
        forward.insert(20, pp(11)); // E1: .pasta 11 の 2 本目 → 消化（9.1）
        // 行21 は意図的に未対応（通過・E6/9.4）
        forward.insert(22, pp(12)); // E1 停止先 / E2 サブ呼び出し / E3 step into 起点
        forward.insert(23, pp(13)); // E5 再帰呼び出し / E2 step over 停止先
        forward.insert(24, pp(14)); // E4 step out 停止先 / E5 step over 停止先
        forward.insert(29, pp(15));

        let mut sm = SourceMap::new();
        sm.insert_chunk(
            STEP_SOURCE.to_string(),
            STEP_PASTA_FILE.to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        Arc::new(sm)
    }

    /// E1–E8 の期待座標を **map から導出**した束（ハードコード回避）。`.lua` 行を起点に
    /// `resolve_lua_to_pasta` で `.pasta` 行を引き、シナリオが要求する関係（同一/異なる/
    /// 未対応）を build 時に表明する。各テストはこの導出値に対してアサートする。
    struct Expected {
        /// BP/step over 起点（単一 `.lua` 行・`derive` 内アンカー行 18）の対応 `.pasta` 行。
        origin_pasta: u32,
        /// E1: 複数 `.lua` 行へ展開された `.pasta` 行（行19/20 = 同一 `.pasta`）と、その
        /// `.lua` 行（1 本目/2 本目）。1 回目 step over の停止先（= 起点の次の異なる
        /// `.pasta` 行）でもある。
        multi_pasta: u32,
        multi_lua_first: u32,
        multi_lua_second: u32,
        /// E6: step over がまたぐ未対応 `.lua` 行（停止しない）。
        unmapped_lua: u32,
        /// E1 の 2 回目 step over 停止先（`.pasta` 11 を消化＋未対応行通過の次の `.pasta` 行）
        /// = helper 呼び出し行（E2 サブ呼び出し / E3 step into 起点）と対応 `.pasta` 行。
        call_helper_lua: u32,
        call_helper_pasta: u32,
        /// E3: helper 内の未対応 `.lua` 行（step into で通過）。
        callee_unmapped_lua: u32,
        /// E3: helper 内の最初の対応 `.lua` 行（step into 停止先・`derive` 内アンカー
        /// 行 3）の `.pasta` 行。
        callee_first_pasta: u32,
        /// E2/E5: helper / recur 呼び出し行から step over した停止先（呼出元フレームの
        /// 次の `.pasta` 行 = 再帰呼び出し行）の `.pasta` 行。
        next_caller_pasta: u32,
        /// E4: step out が呼出元で停止する `.pasta` 行（呼出行の次の対応行）。
        step_out_pasta: u32,
        /// E5: 再帰呼び出し行（step over 起点・`derive` 内アンカー行 23）の `.pasta` 行。
        recur_call_pasta: u32,
        /// E5: 再帰呼び出し行から step over した停止先（呼出元フレームの次の `.pasta`
        /// 行 = 行24）の `.pasta` 行。recur 内の同一 `.pasta` 行（40/41）ではない。
        after_recur_pasta: u32,
        /// E7: コルーチン本体 step 起点（`.pasta` 行）。
        co_origin_pasta: u32,
        /// E7: yield 行の `.pasta` 行（first step over の停止先）。
        co_yield_pasta: u32,
        /// E7: resume 後の `.pasta` 行（yield をまたぐ step over の停止先）。
        co_post_yield_pasta: u32,
    }

    impl Expected {
        /// map から導出し、シナリオ前提（同一/異なる/未対応）を build 時に表明する。
        /// `.lua` 行番号は [`STEP_CHUNK`] のコメント対応表に由来する固定アンカーだが、
        /// `.pasta` 行は全て map から引き、行同士の関係（同一/異なる/未対応）を表明する
        /// ことで「ハードコードした `.pasta` 行で恒真になる」ことを防ぐ。
        fn derive(map: &SourceMap) -> Self {
            let lp = |lua: u32| map.resolve_lua_to_pasta(STEP_SOURCE, lua).map(|p| p.line);

            // 起点（行18）→ 単一 `.lua` 行の `.pasta` 行。
            let origin_lua = 18;
            let origin_pasta = lp(origin_lua).expect("起点 `.lua` 18 は対応 `.pasta` を持つ");

            // E1: 行19/20 が同一 `.pasta` 行（複数 `.lua` 行展開）。
            let multi_lua_first = 19;
            let multi_lua_second = 20;
            let multi_pasta = lp(multi_lua_first).expect("行19 は対応 `.pasta` を持つ");
            assert_eq!(
                lp(multi_lua_second),
                Some(multi_pasta),
                "E1: 行19/20 は同一 `.pasta` 行（複数 `.lua` 行展開・消化対象）"
            );
            assert_ne!(
                multi_pasta, origin_pasta,
                "E1: 複数 `.lua` の `.pasta` 行は起点の `.pasta` 行と異なる（1 回目 step over の停止先）"
            );

            // E6: 行21 は未対応（通過対象）。
            let unmapped_lua = 21;
            assert_eq!(lp(unmapped_lua), None, "E6: 行21 は未対応（通過する）");

            // E1 の 2 回目 step over 停止先 / E2 サブ呼び出し / E3 step into 起点:
            // 行22（helper 呼び出し）。`.pasta` は `multi_pasta` と異なる次の行。
            let call_helper_lua = 22;
            let call_helper_pasta = lp(call_helper_lua).expect("行22 は helper 呼び出しの対応 `.pasta`");
            assert_ne!(
                call_helper_pasta, multi_pasta,
                "E1: 2 回目 step over 停止先は `.pasta` 11 と異なる次の `.pasta` 行"
            );

            // E3 step into: helper（行2 未対応 → 行3 対応）。
            let callee_unmapped_lua = 2;
            assert_eq!(
                lp(callee_unmapped_lua),
                None,
                "E3: helper 内 行2 は未対応（step into で通過）"
            );
            let callee_first_lua = 3;
            let callee_first_pasta =
                lp(callee_first_lua).expect("helper 内 行3 は最初の対応 `.pasta`（step into 停止先）");

            // E5 再帰: 呼び出し行23（recur）。recur 内（行7/8）は別フレームで同一 `.pasta`
            // 行を踏むが、step over は depth で別扱い → 呼出元フレームの次行へ。
            let recur_call_lua = 23;
            let recur_call_pasta = lp(recur_call_lua).expect("行23 は再帰呼び出しの対応 `.pasta`");
            assert_ne!(
                recur_call_pasta, call_helper_pasta,
                "E2/E5: 再帰呼び出し行は helper 呼び出し行と異なる `.pasta` 行"
            );
            // E5 step over 停止先: 行24（recur 呼び出しの直後の対応行）。
            let after_recur_lua = 24;
            let after_recur_pasta = lp(after_recur_lua).expect("行24 は recur 呼び出し直後の対応 `.pasta`");
            assert_ne!(
                after_recur_pasta, recur_call_pasta,
                "E5: 再帰 step over 停止先は recur 呼び出し行と異なる次の `.pasta` 行"
            );
            // recur 内の同一 `.pasta` 行（40/41）が誤停止の罠であることを表明（前提）。
            assert!(
                lp(7).is_some() && lp(8).is_some(),
                "E5: recur 本体（行7/8）は対応 `.pasta` 行を持つ（深いフレームの誤停止候補）"
            );

            // E2: helper 呼び出し行（行22）から step over → 呼出元フレームの次の `.pasta`
            // 行 = 行23（recur 呼び出し・`.pasta` 13）。helper の `.pasta`（30/31）ではない。
            let next_caller_pasta = recur_call_pasta;

            // E4 step out: helper から戻り、呼出元の次の対応行 = 行23（recur 呼び出し）。
            // helper 呼び出し行（行22）の直後の対応行であり、呼出行の `.pasta` とは異なる。
            let step_out_pasta = recur_call_pasta;
            assert_ne!(
                step_out_pasta, call_helper_pasta,
                "E4: step out 停止先は呼出行の `.pasta` 行と異なる（次の対応行）"
            );

            // E7 コルーチン: 本体 行13 起点 → 行14 yield → 行15 resume 後。
            let co_origin_pasta = lp(13).expect("コルーチン本体 行13 の対応 `.pasta`");
            let co_yield_pasta = lp(14).expect("yield 行14 の対応 `.pasta`");
            let co_post_yield_pasta = lp(15).expect("resume 後 行15 の対応 `.pasta`");
            assert!(
                co_origin_pasta != co_yield_pasta && co_yield_pasta != co_post_yield_pasta,
                "E7: コルーチンの 3 停止位置は相異なる `.pasta` 行"
            );

            Expected {
                origin_pasta,
                multi_pasta,
                multi_lua_first,
                multi_lua_second,
                unmapped_lua,
                call_helper_lua,
                call_helper_pasta,
                callee_unmapped_lua,
                callee_first_pasta,
                next_caller_pasta,
                step_out_pasta,
                recur_call_pasta,
                after_recur_pasta,
                co_origin_pasta,
                co_yield_pasta,
                co_post_yield_pasta,
            }
        }
    }

    /// 実 TCP ソケット越しの最小 DAP クライアント（[`super::tests::DapClient`] と同型）。
    struct DapClient {
        reader: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl DapClient {
        fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone socket for writing");
            Self {
                reader: BufReader::new(stream),
                writer,
            }
        }

        fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
            let req = json!({
                "seq": seq,
                "type": "request",
                "command": command,
                "arguments": arguments,
            });
            write_frame(&mut self.writer, &req).expect("client write must succeed");
        }

        fn recv(&mut self) -> Value {
            read_frame(&mut self.reader)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present (peer did not close)")
        }

        fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
            loop {
                let msg = self.recv();
                if pred(&msg) {
                    return msg;
                }
            }
        }
    }

    fn is_event(msg: &Value, name: &str) -> bool {
        msg["type"] == "event" && msg["event"] == name
    }

    fn is_response(msg: &Value, command: &str) -> bool {
        msg["type"] == "response" && msg["command"] == command
    }

    /// `stackTrace` の top フレーム `line` を返す（task 7.1 [`super::tests::top_frame_line`]
    /// と同型）。`SourceMode::Pasta` では resolver（task 5.2）が装着済みなので、これは
    /// **`.pasta` 行**を返す（E1–E7 はこれで `.pasta` 停止位置を観測する）。`Lua` モード
    /// では `.lua` 行を返す（E8/teeth）。`seq` は stackTrace 要求の相関 seq。
    fn top_frame_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
        client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame");
        frames[0]["line"].as_u64().expect("top frame line") as u32
    }

    /// stackTrace の top フレーム `source.path` が `.pasta` を提示することを表明し、その
    /// `line`（= `.pasta` 行）を返す。`.pasta` 提示の「歯」を各停止で効かせる
    /// （resolver/翻訳が無効なら `.lua` チャンク名が出てこのアサートが落ちる）。
    fn top_pasta_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
        client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame");
        let top_src = frames[0]["source"]["path"]
            .as_str()
            .expect("top frame source path");
        assert!(
            top_src.ends_with(".pasta"),
            "`.pasta` 提示中は top フレームが `.pasta` を提示すること（`.lua` ではない）: {top_src:?}"
        );
        frames[0]["line"].as_u64().expect("top frame line") as u32
    }

    /// VM ホストスレッドの結果を watchdog 内で確認する（ハング無し）。
    fn join_host(host: std::thread::JoinHandle<Result<(), String>>) {
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    /// 実 DAP セッションを起動する共通ヘルパ。`STEP_CHUNK` を `STEP_SOURCE` で走らせ、
    /// `map`＋`mode` で `enable` する。`(host, client, thread_id)` を返す。`bp_pasta_line`
    /// は最初に張る `.pasta` 行 BP（`pasta_path` が `.pasta` なら BP 翻訳経路を通る）。
    /// `mode == Lua` のときは `.lua` 行 BP を直接張る（`lua_bp_line` を使う）。
    ///
    /// クライアントは initialize→setBreakpoints→configurationDone を済ませ、最初の停止
    /// （reason breakpoint）まで進めて `thread_id` を確定して返す。
    #[allow(clippy::too_many_arguments)]
    fn start_session(
        map: Arc<SourceMap>,
        mode: SourceMode,
        bp_source_path: &str,
        bp_line: u32,
    ) -> (std::thread::JoinHandle<Result<(), String>>, DapClient, u64) {
        let map_for_host = Arc::clone(&map);

        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_mode: mode,
                ..Default::default()
            };
            let handle = enable(&lua, &cfg, Some(map_for_host))
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            lua.load(STEP_CHUNK)
                .set_name(STEP_SOURCE)
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;
            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // initialize
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let _ = client.recv_until(|m| is_response(m, "initialize"));
        let _ = client.recv_until(|m| is_event(m, "initialized"));

        // setBreakpoints: `.pasta` 行 BP は翻訳経路（map+Pasta）で `.lua` へ展開され
        // verified になる。`.lua` 源（Lua モード）は直接登録。
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": bp_source_path },
                "breakpoints": [{ "line": bp_line }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0]["verified"], true, "BP は verified で登録される");

        // configurationDone → VM 実行開始
        client.send_request(3, "configurationDone", json!({}));
        let _ = client.recv_until(|m| is_response(m, "configurationDone"));
        go_tx.send(()).expect("send go signal");

        // 最初の停止（reason breakpoint）
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "最初の停止は BP（step 起点）"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        (host, client, thread_id)
    }

    /// `continue` を投げて完走させ、host を join する（停止が再発する行 BP の再入は無いので
    /// 1 回で `terminated`／完走）。
    fn continue_to_end(
        host: std::thread::JoinHandle<Result<(), String>>,
        client: &mut DapClient,
        thread_id: u64,
        seq: u64,
    ) {
        client.send_request(seq, "continue", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "continue"));
        join_host(host);
    }

    // =======================================================================
    // E1（9.1）+ E6（9.4）+ E2（9.1）: step over が、複数 `.lua` 行へ展開された同一
    // `.pasta` 行を消化し、未対応行を通過し、次の異なる `.pasta` 行で停止する。さらに
    // サブ呼び出しを含む行（`helper(c)`）からの step over はサブ呼び出しに入らない（E2）。
    // =======================================================================
    /// E1（9.1）/ E6（9.4）/ E2（9.1）を 1 セッションで検証する。
    ///
    /// - **1 回目 step over**（起点 `.pasta` 10・行18 = 単一 `.lua` 行）→ 次の異なる
    ///   `.pasta` 行 = `.pasta` 11（行19・複数 `.lua` 行展開の 1 本目）で停止。
    /// - **E1/E6（2 回目 step over）**: `.pasta` 11（行19）から step over → 行20（**同一
    ///   `.pasta` 11**・複数 `.lua` 行の 2 本目）を**消化**し、行21（未対応・E6/9.4）を
    ///   **通過**し、`.pasta` 12（行22）で停止する（**`.pasta` 12**・`.lua` 20/21 ではない）。
    /// - **E2（3 回目 step over）**: `.pasta` 12（`helper(c)` を**含む**行22）から step over
    ///   → サブ呼び出し helper（行2/3/4・`.pasta` 30/31）に入らず、呼出元フレームの次の
    ///   `.pasta` 行 `.pasta` 13（行23）で停止する。
    #[test]
    fn e1_e6_e2_step_over_consumes_pasta_line_passes_unmapped_and_skips_sub_call() {
        let map = step_scenario_map();
        let exp = Expected::derive(&map);

        let (host, mut client, thread_id) =
            start_session(Arc::clone(&map), SourceMode::Pasta, STEP_PASTA_FILE, exp.origin_pasta);

        // BP 停止位置は `.pasta` 起点行を提示する（5.1）。
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 10),
            exp.origin_pasta,
            "BP 停止は `.pasta` 起点行（{}）を提示する",
            exp.origin_pasta
        );

        // 1 回目 step over（起点 = 単一 `.lua` 行）→ 複数 `.lua` 行展開の `.pasta` 行（行19）。
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "1 回目 step over は reason step");
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 21),
            exp.multi_pasta,
            "1 回目 step over は次の異なる `.pasta` 行 {}（複数 `.lua` 行展開・行{}）で停止する",
            exp.multi_pasta,
            exp.multi_lua_first
        );

        // E1/E6: 2 回目 step over（`.pasta` 11・行19 起点）→ 行20（同一 `.pasta` 11）を消化＋
        // 行21（未対応）を通過 → 次の異なる `.pasta` 行（`.pasta` 12・行22）で停止。
        client.send_request(22, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "E1: step over は reason step");
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 23),
            exp.call_helper_pasta,
            "E1/E6/9.1/9.4: step over は同一 `.pasta` 行の 2 本目（.lua {}）を消化し、未対応行 \
             （.lua {}）を通過、次の異なる `.pasta` 行 {} で停止する（`.lua` 行ではない）",
            exp.multi_lua_second,
            exp.unmapped_lua,
            exp.call_helper_pasta
        );

        // E2: 3 回目 step over（`.pasta` 12・helper(c) を含む行22）→ サブ呼び出しに入らず、
        // 呼出元フレームの次の `.pasta` 行（`.pasta` 13・行23）で停止。
        client.send_request(24, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let _ = client.recv_until(|m| is_event(m, "stopped"));
        let after_sub = top_pasta_line(&mut client, thread_id, 25);
        assert_eq!(
            after_sub, exp.next_caller_pasta,
            "E2/9.1: サブ呼び出しを含む行から step over は呼び出し先 helper（`.pasta` \
             30/31）に入らず、次の `.pasta` 行 {}（呼出元フレーム）で停止する",
            exp.next_caller_pasta
        );
        assert_ne!(
            after_sub, exp.callee_first_pasta,
            "E2: helper 内の `.pasta` 行（{}）で停止してはならない",
            exp.callee_first_pasta
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }

    // =======================================================================
    // E3（9.2）+ E4（9.3）: step into で呼び出し先の最初の対応 `.pasta` 行へ、
    // step out で呼出元の次の対応 `.pasta` 行へ。
    // =======================================================================
    /// E3（9.2）/ E4（9.3）を 1 セッションで検証する。
    ///
    /// - **E3**: 行22（`helper(c)`・`.pasta` 12）で停止 → step into は helper に入り、
    ///   未対応の行2 を通過して、helper の最初の対応 `.pasta` 行（行3 = `.pasta` 30）で
    ///   停止する。
    /// - **E4**: helper 内（行3）から step out → 呼出元へ戻り、呼出行の `.pasta` 行とは
    ///   異なる**次の対応 `.pasta` 行**（行23 = `.pasta` 13）で停止する。
    #[test]
    fn e3_e4_step_into_first_callee_pasta_line_and_step_out_next_caller_pasta_line() {
        let map = step_scenario_map();
        let exp = Expected::derive(&map);

        // 呼び出し行（`.pasta` 12）に BP を張ってそこから step する。
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta,
            STEP_PASTA_FILE,
            exp.call_helper_pasta,
        );
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 10),
            exp.call_helper_pasta,
            "BP 停止は呼び出し行の `.pasta`（{}）",
            exp.call_helper_pasta
        );

        // E3: step into（stepIn）→ helper の最初の対応 `.pasta` 行（行3 = `.pasta` 30）。
        client.send_request(20, "stepIn", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "stepIn"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "E3: step into は reason step");
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 21),
            exp.callee_first_pasta,
            "E3/9.2/9.4: step into は未対応の callee 行（.lua {}）を通過し、helper の最初の \
             対応 `.pasta` 行 {} で停止する",
            exp.callee_unmapped_lua,
            exp.callee_first_pasta
        );

        // E4: step out（stepOut）→ 呼出元へ戻り、呼出行と異なる次の対応 `.pasta` 行
        // （行23 = `.pasta` 13）で停止する。
        client.send_request(30, "stepOut", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "stepOut"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "E4: step out は reason step");
        let out_line = top_pasta_line(&mut client, thread_id, 31);
        assert_eq!(
            out_line, exp.step_out_pasta,
            "E4/9.3: step out は呼出元へ戻り、次の対応 `.pasta` 行 {} で停止する",
            exp.step_out_pasta
        );
        assert_ne!(
            out_line, exp.call_helper_pasta,
            "E4: 呼出行の `.pasta` 行（{}）で再停止してはならない（次の対応行へ）",
            exp.call_helper_pasta
        );

        continue_to_end(host, &mut client, thread_id, 40);
    }

    // =======================================================================
    // E5: 再帰で別フレームの同一 `.pasta` 行に誤停止しない（depth による frame identity）。
    // =======================================================================
    /// E5 を検証する。`recur(2)`（行23・`.pasta` 13）から step over すると、recur は
    /// 自身を再帰呼び出し（行7/8 = `.pasta` 40/41 を**異なる深さのフレームで複数回**踏む）
    /// するが、step over は base フレームより深いフレームを depth で除外するため、それらの
    /// 同一 `.pasta` 行に誤停止せず、呼出元フレームの次の `.pasta` 行（行24 = `.pasta` 14）で
    /// 停止する。
    #[test]
    fn e5_recursion_does_not_mis_stop_at_same_pasta_line_in_other_frames() {
        let map = step_scenario_map();
        let exp = Expected::derive(&map);

        // 再帰呼び出し行（`.pasta` 13）に BP を張る。
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta,
            STEP_PASTA_FILE,
            exp.recur_call_pasta,
        );
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 10),
            exp.recur_call_pasta,
            "BP 停止は再帰呼び出し行の `.pasta`（{}）",
            exp.recur_call_pasta
        );

        // E5: step over → recur の深いフレーム（`.pasta` 40/41 を複数回踏む）に誤停止せず、
        // 呼出元フレームの次の `.pasta` 行（行24 = `.pasta` 14）で停止する。
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "E5: step over は reason step");
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 21),
            exp.after_recur_pasta,
            "E5: 再帰の別フレームで同一 `.pasta` 行（40/41）を踏んでも誤停止せず、呼出元 \
             フレームの次の `.pasta` 行 {} で停止する（depth による frame identity）",
            exp.after_recur_pasta
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }

    // =======================================================================
    // E7: コルーチン跨ぎ（yield/resume）の `.pasta` ステップ。
    // =======================================================================
    /// E7 を検証する。コルーチン本体内（行13・`.pasta` 50）で停止し、step over は yield 行
    /// （行14・`.pasta` 51）に到達。さらに step over で `coroutine.yield()` をまたぐと、
    /// コルーチンは中断し駆動ループ（別スレッド・行25/26）が再 resume するが、それらは
    /// thread 不一致で skip され、同一コルーチンの resume 後の `.pasta` 行（行15・
    /// `.pasta` 52）で停止する（step 鍵が yield/resume をまたいで生存・採択B）。
    #[test]
    fn e7_pasta_step_over_crosses_coroutine_yield_resume() {
        let map = step_scenario_map();
        let exp = Expected::derive(&map);

        // コルーチン本体の起点行（`.pasta` 50）に BP を張る。
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta,
            STEP_PASTA_FILE,
            exp.co_origin_pasta,
        );
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 10),
            exp.co_origin_pasta,
            "BP 停止はコルーチン本体の `.pasta` 起点行（{}）",
            exp.co_origin_pasta
        );

        // 1 回目 step over: 同一フレーム次行 = yield 行（`.pasta` 51）。
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let _ = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 21),
            exp.co_yield_pasta,
            "E7: 1 回目 step over は yield 行の `.pasta`（{}）に停止する",
            exp.co_yield_pasta
        );

        // 2 回目 step over: `coroutine.yield()` をまたぐ。駆動ループ（別スレッド）を
        // skip し、resume 後の `.pasta` 行（`.pasta` 52）で停止する（採択B 生存）。
        client.send_request(30, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "E7: step over は reason step");
        assert_eq!(
            top_pasta_line(&mut client, thread_id, 31),
            exp.co_post_yield_pasta,
            "E7: yield をまたぐ step over は駆動ループ（別スレッド）を skip し、resume 後の \
             `.pasta` 行 {} で停止する（step 鍵が yield/resume をまたいで生存）",
            exp.co_post_yield_pasta
        );

        continue_to_end(host, &mut client, thread_id, 40);
    }

    // =======================================================================
    // E8（9.5）: `.lua` モード回帰 — ステップは `.lua` 行単位（`.pasta` 消化しない）。
    // =======================================================================
    /// E8（9.5）を検証する。`SourceMode::Lua`（map は存在するが提示モードが `.lua`）の
    /// とき、step over は**`.lua` 行単位**で進む。複数 `.lua` 行へ展開された `.pasta` 行の
    /// 1 本目（行19）で停止 → step over は次の `.lua` 行20（**同一 `.pasta` 11 の 2 本目**）で
    /// 停止する。`.pasta` 粒度なら行20 を消化して行22（`.pasta` 12）まで進むが、`.lua`
    /// モードではそうならない。停止位置も `.lua` 座標で提示される。
    #[test]
    fn e8_lua_mode_steps_at_lua_granularity_regression() {
        let map = step_scenario_map();
        let exp = Expected::derive(&map);

        // Lua モード: 複数 `.lua` 展開行の 1 本目（行19）へ `.lua` 直接 BP を張る（`.pasta`
        // 翻訳経路は通さない）。`.lua`/`.pasta` 粒度が分岐する行を起点に選ぶ。
        let lua_source_path = STEP_SOURCE; // `@...`（`.pasta` 拡張子ではない）。
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Lua,
            lua_source_path,
            exp.multi_lua_first,
        );

        // `.lua` モードでは stackTrace は `.lua` 座標（`.pasta` ではない）を提示する。
        assert_eq!(
            top_frame_line(&mut client, thread_id, 10),
            exp.multi_lua_first,
            "E8/9.5: `.lua` モードの停止は `.lua` 行（{}）を提示する",
            exp.multi_lua_first
        );

        // E8: step over → 次の `.lua` 行（行20 = 同一 `.pasta` 11 の 2 本目）。`.pasta` 粒度の
        // ように行20 を消化して行22 まで進んではならない。
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "E8: step over は reason step");
        let lua_step_line = top_frame_line(&mut client, thread_id, 21);
        assert_eq!(
            lua_step_line, exp.multi_lua_second,
            "E8/9.5: `.lua` モードは `.lua` 行単位（次行 {} で停止）。`.pasta` 粒度のように \
             同一 `.pasta` 行（{}）を消化して次の `.pasta` 行まで進んではならない",
            exp.multi_lua_second, exp.multi_pasta
        );
        assert_ne!(
            lua_step_line, exp.call_helper_lua,
            "E8: `.pasta` 粒度の停止先（.lua {}）に進んではならない（`.lua` 粒度回帰）",
            exp.call_helper_lua
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }

    // =======================================================================
    // 「歯」（teeth）: 中核シナリオ（E1）で `.pasta` 粒度を無効化（`SourceMode::Lua`）
    // すると、停止が `.lua` 行になり `.pasta` 行**ではない**ことを示す。E1 の核心アサート
    // （`.pasta` 11 の複数 `.lua` 行を消化して `.pasta` 12 で停止）が `.pasta` 粒度に
    // **真に依存**していることの証左。
    // =======================================================================
    /// teeth: E1 の核心と同一の起点（複数 `.lua` 行展開の `.pasta` 11・行19）・操作でも、
    /// `SourceMode::Lua` では step over の停止が `.lua` 行20（同一 `.pasta` 11 の 2 本目）に
    /// なり、E1 が期待する `.pasta` 停止位置（`.pasta` 12 = `.lua` 22）には**到達しない**。
    /// `.pasta` ステップが無効なら E1 のアサートが落ちることの実証。
    #[test]
    fn teeth_lua_mode_stops_at_lua_line_not_pasta() {
        let map = step_scenario_map();
        let exp = Expected::derive(&map);

        let (host, mut client, thread_id) =
            start_session(Arc::clone(&map), SourceMode::Lua, STEP_SOURCE, exp.multi_lua_first);

        // E1 と同じ起点（`.pasta` 11・行19）で同じ step over を `.lua` モードで行う。
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let _ = client.recv_until(|m| is_event(m, "stopped"));
        let lua_step_line = top_frame_line(&mut client, thread_id, 21);

        // teeth: `.lua` 粒度では同一 `.pasta` 行の 2 本目（行20）で止まる（E1 の `.pasta`
        // 停止 = `.lua` 22 ではない）。つまり E1 の核心アサート（`.pasta` 12）は `.pasta`
        // 粒度が ON のときだけ通る。
        assert_eq!(
            lua_step_line, exp.multi_lua_second,
            "teeth: `.pasta` 粒度を無効化（Lua モード）すると step over は `.lua` 行 {} で \
             停止する（E1 の `.pasta` 消化後の停止位置 .lua {} ではない）",
            exp.multi_lua_second, exp.call_helper_lua
        );
        assert_ne!(
            lua_step_line, exp.call_helper_lua,
            "teeth: `.pasta` 粒度が OFF なら E1 の停止位置（.lua {}）には到達しない \
             → E1 のアサートは `.pasta` 粒度に真に依存（恒真ではない）",
            exp.call_helper_lua
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }
}

#[cfg(test)]
mod pasta_mode_edge_e2e {
    //! Task 7.3 — **提示モード切替**（`.lua` モード回帰・requirements **6.2**/**9.5**）と
    //! **多対多マッピングのエッジケース確定挙動**（requirements **8.1**/**8.2**/**8.3**）の
    //! **E2E**。
    //!
    //! task 7.1 [`super::pasta_bp_e2e`] / 7.2 [`super::pasta_step_e2e`] と同型の
    //! DAP-over-TCP ハーネス（実 `Arc<SourceMap>`・実 [`enable`](crate::debug::enable)
    //! セッション）を用い、**手組みの集約 [`SourceMap`]**（`ChunkSourceMap::from_forward`・
    //! task 5.4 unit test / 7.2 流儀）に **集約行**（複数 `.pasta` 行 → 単一 `.lua` 行・8.1）と
    //! **展開行**（単一 `.pasta` 行 → 複数 `.lua` 行・8.2）の双方を仕込む。
    //!
    //! # 観測する「done」（task 7.3 完了状態）
    //!
    //! 1. **`.lua` 提示モード回帰（6.2/9.5）**: 実 DAP `attach sourcePresentation="lua"`
    //!    （VSCode 等価クライアント経路）で提示モードを `.lua` へ切替えると、BP・停止位置・
    //!    コールスタックが **`.lua` 座標**（`.pasta` ではない）で提示され、ステップも
    //!    **`.lua` 行単位**になる（[`mode_switch_lua_presents_lua_coords_and_lua_step_granularity_over_tcp`]）。
    //!    **歯**: 同一マップ・同一 `.lua` 行を `.pasta` モードで提示すると **`.pasta` 座標**が
    //!    出ることを併せて表明し、`.lua` モードのアサートが本物（恒真でない）ことを裏づける。
    //! 2. **8.1 集約 → 確定的単一 `.pasta`**: 複数 `.pasta` 行が集約された単一 `.lua` 行で
    //!    停止すると、確定的に **単一の** `.pasta` 位置を提示する（last-write-wins・task 3.3）。
    //!    セッション提示＋マップ直接（`pasta_for_lua` 反復一致）で確定性を表明
    //!    （[`edge_8_1_aggregated_lua_line_presents_deterministic_single_pasta`]）。
    //! 3. **8.2 展開 → 同一 `.pasta`**: 単一 `.pasta` 行が展開された複数 `.lua` 行の **いずれ**で
    //!    停止しても **同一の** `.pasta` 行を提示する
    //!    （[`edge_8_2_expanded_pasta_line_same_pasta_at_every_lua_line`]）。
    //! 4. **8.3 提示順安定**: 同一 `.pasta` 位置に対するマッピング（`lua_lines_for_pasta` /
    //!    `resolve_pasta_to_lua`）の提示順が反復・複数回構築をまたいで **安定（決定的）**
    //!    （[`edge_8_3_presentation_order_is_stable_deterministic`]）。
    //!
    //! `mlua::Lua`（`!Send`）は VM ホストスレッドにのみ生存し、バウンド `SocketAddr`
    //! （`Copy`）と go/done チャネルだけが越境する。全クライアント待機は TEST-ONLY
    //! watchdog でバウンドし CI がハングしないようにする（停止コアは無期限）。

    use std::collections::BTreeMap;
    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::Arc;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
    use crate::debug::transport::{read_frame, write_frame};
    use crate::debug::{DebugConfig, SourceMode, enable};

    /// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// 生成 `.lua` チャンク名（フック source = `set_name` 値）。map のキーと一致させる。
    const EDGE_SOURCE: &str = "@pasta_mode_edge_e2e_scenario";

    /// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致させる側）。
    /// `.pasta` 拡張子で [`super::is_pasta_source`] の BP 翻訳経路を通す。
    const EDGE_PASTA_FILE: &str = "scene_edge.pasta";

    /// エッジケースを 1 本で覆う **素の Lua チャンク**。`.pasta` 行は手組みマップ
    /// （[`edge_scenario_map`]）が与える。行番号（1-origin）と各行の役割:
    ///
    /// ```text
    ///   1: local a = 1            -- .pasta 20（集約 `.lua` 行・8.1 / `.lua` モード回帰起点）
    ///   2: local b = a + 1        -- .pasta 30（展開 `.pasta` 30 の 1 本目・8.2）
    ///   3: local c = b + 1        -- .pasta 30（展開 `.pasta` 30 の 2 本目・8.2）
    ///   4: local d = c + 1        -- .pasta 30（展開 `.pasta` 30 の 3 本目・8.2）
    ///   5: local e = d + 1        -- .pasta 40（展開行の後・異なる `.pasta`）
    ///   6: return e
    /// ```
    ///
    /// - 行1（`.pasta` 20）は **集約行**: マップ上は単一の `PastaPos`（`.pasta` 20）だが、
    ///   トランスパイル時に複数の `.pasta` 行（例 19/20）が同一 `.lua` 行へ集約された結果を
    ///   模す。確定挙動（8.1）は「単一の `.pasta` を確定的に提示」であり、`from_forward` の
    ///   `BTreeMap<lua_line, PastaPos>` が 1 `.lua` 行 → 高々 1 `.pasta` 位置を構造的に担保する。
    /// - 行2/3/4（同一 `.pasta` 30）は **展開行**: 単一 `.pasta` 行が複数 `.lua` 行へ展開された
    ///   ケース（8.2）。いずれの行で停止しても同一 `.pasta` 30 を提示する。
    const EDGE_CHUNK: &str = "\
local a = 1
local b = a + 1
local c = b + 1
local d = c + 1
local e = d + 1
return e
";

    /// `EDGE_CHUNK` の手組み `SourceMap`（task 5.4 / 7.2 流儀・`ChunkSourceMap::from_forward`）。
    /// フック source 名でキーし、map が内部で正規化する（task 3.4）。
    ///
    /// - 集約行（8.1）: `.lua` 行1 → `.pasta` 20（単一・確定的）。
    /// - 展開行（8.2）: `.lua` 行2/3/4 → 同一 `.pasta` 30。
    /// - 行5 → `.pasta` 40（展開とは異なる `.pasta`・ステップ観測の終端）。
    fn edge_scenario_map() -> Arc<SourceMap> {
        let pp = |line: u32| PastaPos {
            file: EDGE_PASTA_FILE.to_string(),
            line,
        };
        let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
        forward.insert(1, pp(20)); // 集約 `.lua` 行（8.1）: 単一 `.pasta` 20。
        forward.insert(2, pp(30)); // 展開（8.2）: `.pasta` 30 の 1 本目。
        forward.insert(3, pp(30)); // 展開（8.2）: `.pasta` 30 の 2 本目。
        forward.insert(4, pp(30)); // 展開（8.2）: `.pasta` 30 の 3 本目。
        forward.insert(5, pp(40)); // 展開後の異なる `.pasta` 行。

        let mut sm = SourceMap::new();
        sm.insert_chunk(
            EDGE_SOURCE.to_string(),
            EDGE_PASTA_FILE.to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        Arc::new(sm)
    }

    /// 実 TCP ソケット越しの最小 DAP クライアント（[`super::tests::DapClient`] と同型）。
    struct DapClient {
        reader: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl DapClient {
        fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone socket for writing");
            Self {
                reader: BufReader::new(stream),
                writer,
            }
        }

        fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
            let req = json!({
                "seq": seq,
                "type": "request",
                "command": command,
                "arguments": arguments,
            });
            write_frame(&mut self.writer, &req).expect("client write must succeed");
        }

        fn recv(&mut self) -> Value {
            read_frame(&mut self.reader)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present (peer did not close)")
        }

        fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
            loop {
                let msg = self.recv();
                if pred(&msg) {
                    return msg;
                }
            }
        }
    }

    fn is_event(msg: &Value, name: &str) -> bool {
        msg["type"] == "event" && msg["event"] == name
    }

    fn is_response(msg: &Value, command: &str) -> bool {
        msg["type"] == "response" && msg["command"] == command
    }

    /// `stackTrace` の top フレーム `(source.path, line)` を返す（task 7.1/7.2 同型）。
    /// `.pasta` モードでは resolver（task 5.2）が装着済みなので `.pasta` 座標、`.lua`
    /// モードでは `.lua` 座標（チャンク名 + `.lua` 行）を返す。
    fn top_frame(client: &mut DapClient, thread_id: u64, seq: u64) -> (String, u32) {
        client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame");
        let path = frames[0]["source"]["path"]
            .as_str()
            .expect("top frame source path")
            .to_string();
        let line = frames[0]["line"].as_u64().expect("top frame line") as u32;
        (path, line)
    }

    /// VM ホストスレッドの結果を watchdog 内で確認する（ハング無し）。
    fn join_host(host: std::thread::JoinHandle<Result<(), String>>) {
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    /// 実 DAP セッションを起動する共通ヘルパ。`EDGE_CHUNK` を `EDGE_SOURCE` で走らせ、
    /// `map`＋サーバ既定 `mode` で `enable` する。
    ///
    /// `attach_mode` が `Some(m)` のとき、initialize 後に実 DAP `attach` リクエスト
    /// （`sourcePresentation` 付き）を送って提示モードを **クライアント経路で切替える**
    /// （task 5.5・requirement 6.3）。`None` のときは attach を送らず、サーバ既定 `mode` の
    /// まま（resolved env > file > 既定）。
    ///
    /// その後 setBreakpoints（`bp_source_path` の `bp_line`）→ configurationDone を済ませ、
    /// 最初の停止（reason breakpoint）まで進めて `thread_id` を確定して返す。
    fn start_session(
        map: Arc<SourceMap>,
        server_mode: SourceMode,
        attach_mode: Option<SourceMode>,
        bp_source_path: &str,
        bp_line: u32,
    ) -> (std::thread::JoinHandle<Result<(), String>>, DapClient, u64) {
        let map_for_host = Arc::clone(&map);

        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_mode: server_mode,
                ..Default::default()
            };
            let handle = enable(&lua, &cfg, Some(map_for_host))
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            lua.load(EDGE_CHUNK)
                .set_name(EDGE_SOURCE)
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;
            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // initialize
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let _ = client.recv_until(|m| is_response(m, "initialize"));
        let _ = client.recv_until(|m| is_event(m, "initialized"));

        // attach（任意）: 実クライアント経路で `sourcePresentation` を渡し提示モードを
        // 切替える（task 5.5 / R6.3）。`attach` 応答が来たら適用済み。
        if let Some(m) = attach_mode {
            let presentation = match m {
                SourceMode::Pasta => "pasta",
                SourceMode::Lua => "lua",
            };
            client.send_request(
                4,
                "attach",
                json!({ "sourcePresentation": presentation }),
            );
            let _ = client.recv_until(|m| is_response(m, "attach"));
        }

        // setBreakpoints: `.pasta` 行 BP は翻訳経路（map+Pasta）で `.lua` へ展開され
        // verified になる。`.lua` 源（Lua モード）は直接登録。
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": bp_source_path },
                "breakpoints": [{ "line": bp_line }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0]["verified"], true, "BP は verified で登録される");

        // configurationDone → VM 実行開始
        client.send_request(3, "configurationDone", json!({}));
        let _ = client.recv_until(|m| is_response(m, "configurationDone"));
        go_tx.send(()).expect("send go signal");

        // 最初の停止（reason breakpoint）
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "最初の停止は BP（観測起点）"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        (host, client, thread_id)
    }

    /// `continue` を投げて完走させ、host を join する。
    ///
    /// 展開 `.pasta` 行 BP（8.2）は複数 `.lua` 行を登録するため、`continue` 後に同一
    /// `.pasta` 行 BP の別 `.lua` 行で **再停止**し得る（task 7.1 と同じ多重ヒット挙動）。
    /// よって `stopped`（再停止 → もう一度 `continue`）／host 完了（`terminated` 相当の
    /// 決定的シグナル）のいずれかを観測するまでループする。`continue` を送ったら、その
    /// 後に来る次の制御フレーム（`stopped`／`terminated`）を待つ。再入数で有限（CI 無限
    /// ループ防止に上限）。
    fn continue_to_end(
        host: std::thread::JoinHandle<Result<(), String>>,
        client: &mut DapClient,
        thread_id: u64,
        mut seq: u64,
    ) {
        for _ in 0..30u64 {
            client.send_request(seq, "continue", json!({ "threadId": thread_id }));
            seq += 1;
            let next =
                client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
            if is_event(&next, "terminated") {
                break;
            }
            // それ以外は同一 BP 行（別 `.lua` 行）への再停止 → もう一度 continue。
        }
        join_host(host);
    }

    // =======================================================================
    // 6.2 / 9.5 — `.lua` 提示モード回帰（実 DAP `attach sourcePresentation="lua"`）。
    // BP・停止・コールスタックが `.lua` 座標、ステップが `.lua` 行単位。
    // 「歯」: 同一 `.lua` 行を `.pasta` モードで提示すると `.pasta` 座標になることを
    // 併せて表明し、`.lua` モードのアサートが本物（恒真でない）ことを裏づける。
    // =======================================================================
    /// requirement **6.2**（`.lua` モードで BP・停止位置・コールスタックを `.lua` 座標で
    /// 提示）/ **9.5**（`.lua` モードのステップは `.lua` 行単位）を、実 DAP-over-TCP
    /// `attach sourcePresentation="lua"`（VSCode 等価クライアント経路・task 5.5/R6.3）で
    /// 提示モードを切替えて end-to-end 検証する。
    ///
    /// サーバ既定は `.pasta`（map present・6.1）だが、`attach` が `.lua` を **強制**する。
    /// `.lua` 源（`@...`）への BP を行2（展開 `.pasta` 30 の 1 本目）に張り、停止・
    /// コールスタックが `.lua` 行2（チャンク名提示）であること、step over が次の `.lua`
    /// 行3（**同一 `.pasta` 30 の 2 本目**）で止まる（`.pasta` 粒度なら同一 `.pasta` を
    /// 消化して行5 まで進むがそうならない）ことを表明する。
    #[test]
    fn mode_switch_lua_presents_lua_coords_and_lua_step_granularity_over_tcp() {
        let map = edge_scenario_map();

        // サーバ既定は Pasta（6.1）。attach で `.lua` を強制（R6.3）。BP は `.lua` 源・
        // 行2（展開 `.pasta` 30 の 1 本目）。`.lua`/`.pasta` 粒度が分岐する行を起点に選ぶ。
        let lua_bp_line = 2u32;
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta,            // サーバ既定（6.1）。
            Some(SourceMode::Lua),        // attach で `.lua` へ切替（6.2/6.3）。
            EDGE_SOURCE,                  // `.lua` 源（`@...`・`.pasta` 拡張子ではない）。
            lua_bp_line,
        );

        // 6.2: `.lua` モードの停止・コールスタックは `.lua` 座標を提示する（`.pasta`
        // ではない）。top フレーム source.path はチャンク名（`@...`・`.pasta` で終わらない）、
        // line は `.lua` 行。
        let (lua_src, lua_line) = top_frame(&mut client, thread_id, 10);
        assert!(
            !lua_src.ends_with(".pasta"),
            "6.2: `.lua` モードのコールスタックは `.lua` 座標（チャンク名）を提示すること \
             （`.pasta` ではない）。actual={lua_src:?}"
        );
        assert_eq!(
            crate::debug::source_map::canonicalize_chunk_name(&lua_src),
            crate::debug::source_map::canonicalize_chunk_name(EDGE_SOURCE),
            "6.2: `.lua` モードの提示 source は生成 `.lua` チャンク（{EDGE_SOURCE}）"
        );
        assert_eq!(
            lua_line, lua_bp_line,
            "6.2: `.lua` モードの停止行は `.lua` 行（{lua_bp_line}）。`.pasta` 行（30）ではない"
        );

        // 9.5: `.lua` モードの step over は `.lua` 行単位。行2 → 行3（同一 `.pasta` 30 の
        // 2 本目）で停止する。`.pasta` 粒度なら行3/4 を消化して行5（`.pasta` 40）まで
        // 進むが、`.lua` モードではそうならない。
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "next"));
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(stopped["body"]["reason"], "step", "9.5: step over は reason step");
        let (step_src, step_line) = top_frame(&mut client, thread_id, 21);
        assert!(
            !step_src.ends_with(".pasta"),
            "9.5: `.lua` モードのステップ後も `.lua` 座標提示。actual={step_src:?}"
        );
        assert_eq!(
            step_line, 3,
            "9.5: `.lua` モードの step over は次の `.lua` 行（3 = 同一 `.pasta` 30 の 2 本目）で \
             停止する。`.pasta` 粒度のように同一 `.pasta` 行を消化して `.pasta` 40（.lua 5）まで \
             進んではならない"
        );
        assert_ne!(
            step_line, 5,
            "9.5: `.pasta` 粒度の停止先（.lua 5 = `.pasta` 40）に進んではならない（`.lua` 粒度回帰）"
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }

    /// 「歯」（6.2 の teeth）: `.lua` モード回帰の核心アサート（`.lua` 座標・`.lua` 粒度）が
    /// **本物**であることを、同一 map・同一 `.lua` 行2 を **`.pasta` モード**で提示すると
    /// **`.pasta` 30**（`.lua` 行ではない）が出ることで裏づける。提示モード切替が効いて
    /// いなければ、`.lua` モードのテストも `.pasta` を提示して落ちるはず。
    #[test]
    fn teeth_same_lua_line_in_pasta_mode_presents_pasta_not_lua() {
        let map = edge_scenario_map();

        // `.pasta` モード（既定・attach なし）。`.pasta` 30（展開行）に BP を張り、最初の
        // 対応 `.lua` 行（行2）で停止する。
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta,
            None,
            EDGE_PASTA_FILE,
            30,
        );

        // 歯: 同一 `.lua` 行2 が、`.pasta` モードでは `.pasta` 30 を提示する（`.lua` 行
        // ではない）。`.lua` モードのテストはここが `.lua` 座標に変わることを表明している。
        let (pasta_src, pasta_line) = top_frame(&mut client, thread_id, 10);
        assert!(
            pasta_src.ends_with(".pasta"),
            "歯: `.pasta` モードは `.pasta` を提示する（`.lua` モードのテストの差分が観測可能）。\
             actual={pasta_src:?}"
        );
        assert_eq!(
            pasta_line, 30,
            "歯: `.pasta` モードの提示行は `.pasta` 30（`.lua` 行2 ではない）。`.lua` モードの \
             テストの `.lua` 座標アサートは恒真でない"
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }

    // =======================================================================
    // 8.1 — 集約行に確定的単一 `.pasta` 提示。複数 `.pasta` 行が集約された単一 `.lua`
    // 行で停止すると、確定的に単一の `.pasta` 位置を提示する（last-write-wins・3.3）。
    // =======================================================================
    /// requirement **8.1**: 複数 `.pasta` 行が同一 `.lua` 行へ集約された場合、当該 `.lua`
    /// 行の停止について **確定的な単一の `.pasta` 位置**を提示する。
    ///
    /// (a) マップ直接: 集約行 `.lua` 行1 の `resolve_lua_to_pasta` は **単一**の `.pasta`
    ///     位置（20）を返し、反復しても同一（決定的）。`from_forward` の
    ///     `BTreeMap<lua_line, PastaPos>` が 1 `.lua` 行 → 高々 1 `.pasta` 位置を構造的に
    ///     担保する（design 286 last-write-wins）。
    /// (b) セッション提示: 集約 `.lua` 行で停止すると stackTrace の top フレームが その
    ///     単一 `.pasta` 20 を提示する。
    #[test]
    fn edge_8_1_aggregated_lua_line_presents_deterministic_single_pasta() {
        let map = edge_scenario_map();

        // (a) マップ直接: 集約 `.lua` 行1 → 単一 `.pasta` 20。反復一致（確定的）。
        let first = map
            .resolve_lua_to_pasta(EDGE_SOURCE, 1)
            .expect("集約 `.lua` 行1 は対応 `.pasta` を持つ")
            .clone();
        assert_eq!(
            first.line, 20,
            "8.1: 集約 `.lua` 行1 の `.pasta` 位置は確定的単一（20・last-write-wins）"
        );
        // 反復して同一位置を返す（確定性・8.1）。
        for _ in 0..8 {
            let again = map
                .resolve_lua_to_pasta(EDGE_SOURCE, 1)
                .expect("集約行は反復しても対応 `.pasta` を持つ");
            assert_eq!(
                (&again.file, again.line),
                (&first.file, first.line),
                "8.1: 集約行の `.pasta` 位置は反復しても確定的に同一の単一位置"
            );
        }

        // (b) セッション提示: `.pasta` 20 に BP を張り、集約 `.lua` 行1 で停止 → top
        // フレームは単一 `.pasta` 20 を提示する。
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta,
            None,
            EDGE_PASTA_FILE,
            20,
        );
        let (src, line) = top_frame(&mut client, thread_id, 10);
        assert!(
            src.ends_with(".pasta"),
            "8.1: 集約行の停止は `.pasta` を提示する。actual={src:?}"
        );
        assert_eq!(
            line, 20,
            "8.1: 集約 `.lua` 行1 の停止は確定的単一の `.pasta` 20 を提示する"
        );

        continue_to_end(host, &mut client, thread_id, 30);
    }

    // =======================================================================
    // 8.2 — 展開行は同一 `.pasta` 提示。単一 `.pasta` 行が複数 `.lua` 行へ展開された
    // 場合、それら `.lua` 行のいずれで停止しても同一の `.pasta` 行を提示する。
    // =======================================================================
    /// requirement **8.2**: 単一 `.pasta` 行（30）が複数 `.lua` 行（2/3/4）へ展開された
    /// とき、**いずれの `.lua` 行で停止しても**同一の `.pasta` 30 を提示する。
    ///
    /// (a) マップ直接: `resolve_lua_to_pasta` が 行2/3/4 すべてで同一 `.pasta` 30 を返す。
    /// (b) セッション提示: 各展開 `.lua` 行に **`.lua` 直接 BP**（`.lua` 源）を張り、それぞれ
    ///     `.pasta` モードで停止させて、top フレームが毎回同一 `.pasta` 30 を提示すること
    ///     を end-to-end で確認する（`.lua` 源 BP でも `.pasta` 提示は resolver が担う）。
    #[test]
    fn edge_8_2_expanded_pasta_line_same_pasta_at_every_lua_line() {
        let map = edge_scenario_map();

        // (a) マップ直接: 展開 `.lua` 行2/3/4 はすべて同一 `.pasta` 30。
        for lua_line in [2u32, 3, 4] {
            let pos = map
                .resolve_lua_to_pasta(EDGE_SOURCE, lua_line)
                .unwrap_or_else(|| panic!("展開 `.lua` 行{lua_line} は対応 `.pasta` を持つ"));
            assert_eq!(
                pos.line, 30,
                "8.2: 展開 `.lua` 行{lua_line} は単一 `.pasta` 30 へ写像する"
            );
        }

        // (b) セッション提示: 各展開 `.lua` 行で停止 → 毎回同一 `.pasta` 30 を提示する。
        // `.lua` 直接 BP（`.lua` 源）を使い、停止を当該 `.lua` 行に確定させつつ、提示は
        // `.pasta` モードの resolver（task 5.2）で `.pasta` 30 になることを確認する。
        for lua_line in [2u32, 3, 4] {
            let (host, mut client, thread_id) = start_session(
                Arc::clone(&map),
                SourceMode::Pasta, // 提示は `.pasta`（resolver 装着）。
                None,
                EDGE_SOURCE,       // `.lua` 直接 BP（停止行を当該 `.lua` 行に確定）。
                lua_line,
            );
            let (src, presented) = top_frame(&mut client, thread_id, 10);
            assert!(
                src.ends_with(".pasta"),
                "8.2: 展開 `.lua` 行{lua_line} の停止は `.pasta` を提示する。actual={src:?}"
            );
            assert_eq!(
                presented, 30,
                "8.2: 展開 `.lua` 行{lua_line} のいずれで停止しても同一 `.pasta` 30 を提示する"
            );
            continue_to_end(host, &mut client, thread_id, 30);
        }
    }

    // =======================================================================
    // 8.3 — 提示順安定（決定的）。同一 `.pasta` 位置に対するマッピングの提示順が
    // 反復・複数回構築をまたいで安定（決定的）。
    // =======================================================================
    /// requirement **8.3**: 同一 `.pasta` 位置に対するマッピングの提示順序を安定
    /// （決定的）に保つ。
    ///
    /// (a) `ChunkSourceMap::lua_lines_for_pasta` が展開 `.pasta` 30 に対し `.lua` 行の
    ///     **昇順**（[2, 3, 4]）を返し、反復しても同一順（決定的）。
    /// (b) 集約 `SourceMap::resolve_pasta_to_lua` が `.pasta` 30 に対し
    ///     `(chunk, lua_line)` の決定的順序（チャンク名昇順 → `.lua` 行昇順）を返し、
    ///     反復しても同一。
    /// (c) マップを **複数回構築**（`edge_scenario_map` を再呼び出し）しても同一順序
    ///     （ビルド非依存の決定論）。
    #[test]
    fn edge_8_3_presentation_order_is_stable_deterministic() {
        let map = edge_scenario_map();

        // (a) チャンクレベルの逆引きは `.lua` 行昇順で決定的。反復一致。
        // チャンクへ直接アクセスはできない（private）ため、集約 `SourceMap` 経由の
        // 逆引きで順序の安定を確認する（resolve_pasta_to_lua は同一決定的順序を返す）。
        let baseline: Vec<u32> = map
            .resolve_pasta_to_lua(EDGE_PASTA_FILE, 30)
            .into_iter()
            .map(|(_chunk, lua_line)| lua_line)
            .collect();
        assert_eq!(
            baseline,
            vec![2, 3, 4],
            "8.3: 展開 `.pasta` 30 の逆引きは `.lua` 行昇順 [2, 3, 4]（決定的）"
        );

        // (b) 反復しても完全一致（`(chunk, lua_line)` 列ごと）。
        let baseline_full = map.resolve_pasta_to_lua(EDGE_PASTA_FILE, 30);
        for _ in 0..16 {
            let again = map.resolve_pasta_to_lua(EDGE_PASTA_FILE, 30);
            assert_eq!(
                again, baseline_full,
                "8.3: 同一 `.pasta` 位置の提示順は反復しても安定（決定的）"
            );
        }

        // (c) マップを複数回構築しても同一順序（ビルド非依存の決定論）。
        for _ in 0..4 {
            let rebuilt = edge_scenario_map();
            let order: Vec<u32> = rebuilt
                .resolve_pasta_to_lua(EDGE_PASTA_FILE, 30)
                .into_iter()
                .map(|(_chunk, lua_line)| lua_line)
                .collect();
            assert_eq!(
                order, baseline,
                "8.3: 複数回構築しても提示順は安定（決定的・BTreeMap 由来）"
            );
        }
    }
}

#[cfg(test)]
mod pasta_break_coalesce_e2e {
    //! Task 3.2 — 多対1 Continue の実 DAP-over-TCP E2E
    //! (spec: `pasta-debug-break-coalesce`, requirements **1.1** / **1.2** / **3.2** / **6.2(a)**).
    //!
    //! # 観測する「done」
    //!
    //! 1つの `.pasta` 行が複数の `.lua` 行へ展開される（多対1）構成で、その `.pasta` 行へ
    //! BP を張って停止させ、**1回だけ** `continue` を送ると、実行は **次の `.pasta` 行**
    //! （次の停止点）へ進み、**同一 `.pasta` 行に対する 2 回目の `stopped` は来ない**こと
    //! を実ソケットで証明する。これが design "System Flows" / "Testing Strategy →
    //! Integration / E2E（multi-to-one Continue）" のシナリオであり、fix（`session.rs`
    //! `on_line_impl` の break-anchor coalescing）が無ければ、同じ `.pasta` 行へマップする
    //! 次の `.lua` 行で再停止してこのテストが落ちる。
    //!
    //! # フィクスチャ（Task 3.1 で committed）
    //!
    //! `tests/fixtures/debug_break_coalesce.pasta` のトーク行
    //! `合計は＠加算ループ()　です。`（`.pasta` 行 21）は、生成 `.lua` の
    //! `SCENE.__start__` 本体で **3 本の `talk(...)`/`expr_fn(...)` 文**（`.lua` 行 11/12/13）
    //! へ展開される（多対1）。次行 `「おはよう」「げんき」`（`.pasta` 行 22）は `.lua` 行 14
    //! へ対応し、これが「1 回の continue で到達すべき次の停止点」。実 `.pasta`/`.lua` 行は
    //! **ローダの本番マップ構築経路**（[`PastaLoader::build_source_map`]・Task 4.3）から
    //! 導出してハードコードを避ける（[`build_fixture`] と同流儀）。
    //!
    //! # ハーネス（task 7.1 [`super::pasta_bp_e2e`] の踏襲）
    //!
    //! 実マップを `SourceMode::Pasta` で [`enable`](crate::debug::enable) へ渡し、生成
    //! `.lua` を **ローダ由来チャンク名**（`@<キャッシュ .lua パス>`）で VM 上に走らせる。
    //! 関数定義は BP 設定前に行い（チャンク本文実行で `SCENE.__start__` 等を定義）、その後
    //! クライアントが `.pasta` 行 21 へ BP を張ってから `SCENE.__start__(act)` を呼ぶ。
    //! こうすると BP 有効下で実行されるのは `__start__` 本体（`.lua` 行 11→12→13→14）だけ
    //! になり、多対1 行の **連続実行**中の coalescing を決定的に観測できる（関数定義行
    //! `.lua` 7 もまた `.pasta` 21 へマップするが、それは定義時＝BP 設定前に実行済み）。
    //!
    //! `mlua::Lua`（`!Send`）は VM ホストスレッドにのみ生存し、バウンド `SocketAddr`
    //! （`Copy`）と go/done チャネルだけが越境する。全クライアント待機は TEST-ONLY
    //! watchdog でバウンドし CI がハングしないようにする（停止コアは無期限）。

    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::Arc;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::source_map::{MapBuilderSink, SourceMap};
    use crate::debug::transport::{read_frame, write_frame};
    use crate::debug::{DebugConfig, SourceMode, enable};
    use crate::loader::CacheManager;
    use crate::transpiler::LuaTranspiler;

    /// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// 多対1＋ループ再訪を含む committed フィクスチャ（Task 3.1）。3.2/3.3 が同一ファイルを
    /// ロードする。`include_str!` で取り込み、移動時はコンパイルエラーで検知する。
    const FIXTURE: &str = include_str!("../../tests/fixtures/debug_break_coalesce.pasta");

    /// 多対1 トーク行（`合計は＠加算ループ()　です。`）の `.pasta` 行（1-origin）を
    /// FIXTURE から一意に求める（[`debug_break_coalesce_fixture_test`] の同名マーカー）。
    const MULTI_TO_ONE_MARKER: &str = "合計は＠加算ループ()";

    /// ループ本体行（`SCENE.加算ループ` 内 `total = total + i`）の `.pasta` 行を FIXTURE から
    /// 一意に求める（[`debug_break_coalesce_fixture_test`] の同名マーカー / Task 3.3 用）。
    /// ループは `GLOBAL.ループ回数 or 3` 回まわるため、この `.pasta` 行は実行時に再訪される。
    const LOOP_BODY_MARKER: &str = "total = total + i";

    /// フィクスチャのループ反復回数（`GLOBAL.ループ回数` 未設定時の既定 = ループ本体行の再訪回数）。
    /// shim は `package.loaded['pasta.global'] = {}`（= `GLOBAL.ループ回数` が nil）を与えるため、
    /// `加算ループ` は `for i = 1, 3 do ... end` を回し、ループ本体 `.pasta` 行を N 回再訪する。
    const LOOP_VISITS: usize = 3;

    /// `require "pasta"` / `require "pasta.global"` を満たし、`SCENE.__start__` を実行可能に
    /// する最小シム。`create_scene` は素のテーブルを返し、`act` スタブは `init_scene` /
    /// `expr_fn` / `さくら:talk` を no-op で提供する（`.pasta`↔`.lua` 変換とは無関係の純粋な
    /// 実行足場であり、BP/coalescing は実セッションが担う）。
    const PASTA_SHIM: &str = "\
local PASTA = {}
function PASTA.create_scene(name)
    local s = { name = name }
    PASTA.__last_scene = s   -- 生成 scene を捕捉（chunk の do...end ローカルを橋渡し）
    return s
end
package.loaded['pasta'] = PASTA
package.loaded['pasta.global'] = {}
local actor = {}
actor.__index = actor
function actor:talk(_s) return self end
function actor:expr_fn(_name) return 0 end
ACT = setmetatable({}, {
    __index = function(_t, _k)
        return setmetatable({}, actor)
    end,
})
function ACT.init_scene(_self, _scene) return {}, {} end
";

    /// 実 TCP ソケット越しの最小 DAP クライアント（[`super::tests::DapClient`] と同型）。
    struct DapClient {
        reader: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl DapClient {
        fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone socket for writing");
            Self {
                reader: BufReader::new(stream),
                writer,
            }
        }

        fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
            let req = json!({
                "seq": seq,
                "type": "request",
                "command": command,
                "arguments": arguments,
            });
            write_frame(&mut self.writer, &req).expect("client write must succeed");
        }

        fn recv(&mut self) -> Value {
            read_frame(&mut self.reader)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present (peer did not close)")
        }

        fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
            loop {
                let msg = self.recv();
                if pred(&msg) {
                    return msg;
                }
            }
        }
    }

    fn is_event(msg: &Value, name: &str) -> bool {
        msg["type"] == "event" && msg["event"] == name
    }

    fn is_response(msg: &Value, command: &str) -> bool {
        msg["type"] == "response" && msg["command"] == command
    }

    /// FIXTURE 内で `needle` を含む唯一の **非コメント** 行の 1-origin 行番号を返す。
    /// コメント行（`＃`/`#`）は除外する（マーカー文字列を解説で含み得るため）。
    fn unique_pasta_line(needle: &str) -> u32 {
        let hits: Vec<u32> = FIXTURE
            .lines()
            .enumerate()
            .filter(|(_, l)| {
                let t = l.trim_start();
                !t.starts_with('＃') && !t.starts_with('#') && l.contains(needle)
            })
            .map(|(i, _)| i as u32 + 1)
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "fixture invariant: marker {needle:?} must appear on exactly one line, got {hits:?}"
        );
        hits[0]
    }

    /// 実トランスパイル成果物（実座標を **マップから導出**）。
    struct Fixture {
        /// ローダ本番経路の集約マップ（`enable` へ渡す）。
        map: Arc<SourceMap>,
        /// 実行する生成 `.lua` 本文（map 構築と同一バイト）。
        lua_source: String,
        /// VM チャンク名（`@<キャッシュ .lua 絶対パス>`）= ローダ由来チャンク名。
        chunk_name: String,
        /// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致）。
        pasta_path: String,
        /// 多対1 トーク行の `.pasta` 行（BP 対象）。
        multi_pasta_line: u32,
        /// `multi_pasta_line` が展開される `.lua` 行群（≥2 で多対1 を裏づける）。
        multi_lua_lines: Vec<u32>,
        /// 多対1 行の **次に到達すべき** `.pasta` 行（continue 後の停止点・歯）。
        next_pasta_line: u32,
    }

    /// committed `.pasta` をディスクへ書き、(a) 本番ローダ経路でマップを構築し、(b) 同一入力を
    /// 同一マップ構築経路で再トランスパイルして実行用 `.lua` 本文を得る（バイト一致）。BP 対象・
    /// 次行・`.lua` 展開はすべて map から導出する。
    fn build_fixture() -> Fixture {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let base_dir = temp.path().to_path_buf();
        let pasta_file = base_dir.join("dic/test/debug_break_coalesce.pasta");
        std::fs::create_dir_all(pasta_file.parent().unwrap()).expect("mkdir dic");
        std::fs::write(&pasta_file, FIXTURE).expect("write .pasta");

        let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");

        // (a) 本番ローダ経路の集約マップ（sidecar=false: メモリ既定）。
        let map = crate::loader::PastaLoader::build_source_map(
            std::slice::from_ref(&pasta_file),
            &cache_manager,
            false,
        );

        // チャンク名キー = ローダ由来 `source_to_cache_path`（map のキーと同一）。
        let chunk_name = cache_manager
            .source_to_cache_path(&pasta_file)
            .to_string_lossy()
            .to_string();
        // `.pasta` ファイルキー = `parse_str` / `PastaPos.file`（VSCode source.path 側）。
        let pasta_path = pasta_file.to_string_lossy().to_string();

        // (b) `build_source_map` は生成 `.lua` を破棄するため、同一入力・同一マップ構築経路で
        //     再トランスパイルして **実行する `.lua` 本文**を得る（map と同一の決定的バイト）。
        let content = std::fs::read_to_string(&pasta_file).expect("read .pasta");
        let parsed = pasta_dsl::parse_str(&content, &pasta_path).expect("parse .pasta");
        let transpiler = LuaTranspiler::default();
        let mut sink = MapBuilderSink::new(pasta_path.clone(), chunk_name.clone());
        let mut out = Vec::new();
        transpiler
            .transpile_with_source_map(&parsed, &mut out, Some(&mut sink))
            .expect("transpile .pasta");
        let lua_source = String::from_utf8(out).expect("utf8 .lua");

        // 多対1 行（BP 対象）の `.pasta` 行と、その `.lua` 展開（≥2）。
        let multi_pasta_line = unique_pasta_line(MULTI_TO_ONE_MARKER);
        let mut multi_lua_lines: Vec<u32> = map
            .resolve_pasta_to_lua(&pasta_path, multi_pasta_line)
            .into_iter()
            .map(|(_chunk, lua_line)| lua_line)
            .collect();
        multi_lua_lines.sort_unstable();
        assert!(
            multi_lua_lines.len() >= 2,
            "6.2(a): 多対1 行 {multi_pasta_line} は ≥2 の `.lua` 行へ展開されること（前提）: \
             {multi_lua_lines:?}"
        );

        // continue 後に到達すべき **次の** `.pasta` 行 = 多対1 行の `.lua` 展開の **最大** 行の
        // 直後で初めて現れる、`multi_pasta_line` と異なる `.pasta` 行を map から導出する。
        let last_multi_lua = *multi_lua_lines.last().unwrap();
        let next_pasta_line = (last_multi_lua + 1..=last_multi_lua + 200)
            .find_map(|lua_line| {
                map.resolve_lua_to_pasta(&chunk_name, lua_line)
                    .filter(|pos| pos.line != multi_pasta_line)
                    .map(|pos| pos.line)
            })
            .expect("多対1 行の後に別の `.pasta` 行が存在すること（next stop point）");
        assert_ne!(
            next_pasta_line, multi_pasta_line,
            "次の停止点は多対1 行とは異なる `.pasta` 行であること（歯の有効性）"
        );

        drop(temp); // map / lua_source は取得済み。ディスクは不要。

        Fixture {
            map,
            lua_source,
            chunk_name,
            pasta_path,
            multi_pasta_line,
            multi_lua_lines,
            next_pasta_line,
        }
    }

    /// 多対1 Continue: `.pasta` 行（→ 複数 `.lua` 行）に BP を張り、停止後 **1 回だけ**
    /// `continue` すると、(1) 同一 `.pasta` 行に対する 2 回目の `stopped` は来ず、(2) 実行は
    /// 次の `.pasta` 行（次の停止点）へ進む — を実 DAP-over-TCP で証明する
    /// （requirements 1.1 / 1.2 / 3.2 / 6.2(a)）。
    #[test]
    fn one_continue_escapes_multi_to_one_pasta_line_over_tcp() {
        let fx = build_fixture();

        // 前提の自己点検: 多対1 行が ≥2 の `.lua` 行へ展開される（テストが非空虚である土台）。
        assert!(
            fx.multi_lua_lines.len() >= 2,
            "多対1 前提: `.pasta` 行 {} → `.lua` 行 {:?}",
            fx.multi_pasta_line,
            fx.multi_lua_lines
        );

        let lua_source = fx.lua_source.clone();
        let chunk_name = fx.chunk_name.clone();
        let map = Arc::clone(&fx.map);

        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (loaded_tx, loaded_rx) = mpsc::channel::<()>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        // VM HOST スレッド。
        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_mode: SourceMode::Pasta, // 既定だが明示（6.1）。
                ..Default::default()
            };
            let handle = enable(&lua, &cfg, Some(map))
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            // `require "pasta"` を満たすシムを先に読み込む（BP 設定前・関数定義準備）。
            lua.load(PASTA_SHIM)
                .set_name("@pasta_shim")
                .exec()
                .map_err(|e| format!("shim exec failed: {e}"))?;

            // 生成 `.lua` 本文を **ローダ由来チャンク名**で実行し、`SCENE.__start__` 等を
            // 定義する。多対1 行 BP はまだ張られていないので、定義時に `.lua` 行 7（→ `.pasta`
            // 21）を通過しても停止しない（BP 設定前）。
            lua.load(&lua_source)
                .set_name(format!("@{chunk_name}"))
                .exec()
                .map_err(|e| format!("chunk (definitions) exec failed: {e}"))?;

            // 定義完了をクライアントへ通知し、BP+configurationDone 完了の go を待つ。
            loaded_tx
                .send(())
                .map_err(|_| "loaded signal send failed".to_string())?;
            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before calling __start__".to_string())?;

            // ここで初めて多対1 行が **連続実行**される: 捕捉した SCENE の `__start__(ACT)` を
            // 呼ぶと `.lua` 行 11→12→13（すべて `.pasta` 21）→ 14（`.pasta` 22）が走る。
            // SCENE は chunk の `do ... end` ローカルなので、shim が捕捉した `__last_scene`
            // 経由で取得する。
            lua.load("local s = package.loaded['pasta'].__last_scene; return s.__start__(ACT)")
                .set_name("@invoke_start")
                .exec()
                .map_err(|e| format!("__start__ call failed: {e}"))?;

            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        // CLIENT（このスレッド）。
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // initialize ハンドシェイク。
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let _ = client.recv_until(|m| is_response(m, "initialize"));
        let _ = client.recv_until(|m| is_event(m, "initialized"));

        // 関数定義（チャンク本文実行）が終わるのを待ってから BP を張る — こうすると BP 有効
        // 下で実行されるのは `__start__` 本体だけになり、多対1 行の連続実行を観測できる。
        loaded_rx
            .recv_timeout(WATCHDOG)
            .expect("host must finish definitions before the watchdog");

        // 多対1 `.pasta` 行（翻訳経路 map+Pasta で複数 `.lua` 行へ展開）と、その **次行**の
        // 両方へ BP を張る。次行 BP は「1 回の continue が次の停止点まで進む」ことを観測する
        // ための停止点であると同時に、coalescing が無ければ continue が **次行へ着く前に**
        // 同一 `.pasta` 行 21 の別 `.lua` 行で再停止することを暴く（テストの歯）。
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": fx.pasta_path },
                "breakpoints": [
                    { "line": fx.multi_pasta_line },
                    { "line": fx.next_pasta_line },
                ],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 2, "2 つの BP 応答（多対1 行＋次行）");
        assert!(
            bps.iter().all(|b| b["verified"] == true),
            "両 `.pasta` 行 BP は verified で登録される: {bps:?}"
        );

        client.send_request(3, "configurationDone", json!({}));
        let _ = client.recv_until(|m| is_response(m, "configurationDone"));
        go_tx.send(()).expect("send go signal");

        // (1) 最初の停止: 多対1 `.pasta` 行で stop（reason breakpoint）。
        let stopped1 = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped1["body"]["reason"], "breakpoint",
            "1.2/3.2: 多対1 `.pasta` 行 BP に対応する `.lua` 行で停止する"
        );
        let thread_id = stopped1["body"]["threadId"].as_u64().expect("threadId");
        // 停止位置は多対1 `.pasta` 行（resolver 装着で `.pasta` 座標を提示する）。
        let stop1_line = top_pasta_line(&mut client, thread_id, 10);
        assert_eq!(
            stop1_line, fx.multi_pasta_line,
            "最初の停止は多対1 `.pasta` 行 {} であること",
            fx.multi_pasta_line
        );

        // === 「歯」: ここで **1 回だけ** continue を送る。fix（break-anchor coalescing）が
        // あれば、同じ `.pasta` 行へマップする残りの `.lua` 行は消化され、次の `stopped` は
        // **別の** `.pasta` 行（次の停止点）になる。fix が無ければ、同一 `.pasta` 行 {} の
        // 次の `.lua` 行で再停止し（reason breakpoint・同じ提示行）、下のアサートが落ちる。===
        client.send_request(20, "continue", json!({ "threadId": thread_id }));
        let _ = client.recv_until(|m| is_response(m, "continue"));

        // 1 回の continue 後に来る次の制御フレーム（stopped か terminated）。
        let next = client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));

        if is_event(&next, "stopped") {
            // 次の停止が **同一 `.pasta` 行** であってはならない（2 回目の同一行 stop の禁止 =
            // requirement 1.1 / 3.2）。停止しているなら次の `.pasta` 停止点であること。
            let next_tid = next["body"]["threadId"].as_u64().unwrap_or(thread_id);
            let stop2_line = top_pasta_line(&mut client, next_tid, 21);
            assert_ne!(
                stop2_line, fx.multi_pasta_line,
                "1.1/3.2: 1 回の continue が同一 `.pasta` 行 {} で **再停止してはならない** \
                 （coalescing）。actual={stop2_line}",
                fx.multi_pasta_line
            );
            assert_eq!(
                stop2_line, fx.next_pasta_line,
                "1.2: 1 回の continue は **次の** `.pasta` 行 {} へ進むこと。actual={stop2_line}",
                fx.next_pasta_line
            );
            // 後片付け: 残りを流し切って host を完走させる（CI 無限ループ防止に上限）。
            let mut done = false;
            for seq in 30u64..60u64 {
                client.send_request(seq, "continue", json!({ "threadId": next_tid }));
                let m =
                    client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
                if is_event(&m, "terminated") {
                    done = true;
                    break;
                }
            }
            assert!(done, "残りを continue で流し切れること");
        } else {
            // 次行（`.pasta` {next_pasta_line}）には BP を張ってあるので、正常時はここで停止する
            // はず。stopped 無しで terminated したのは、1 回の continue が次の停止点へ到達できて
            // いない（早期完走）ことを意味するので失敗扱いとする。
            panic!(
                "1.2: 1 回の continue は次の `.pasta` 行 {} で停止すべきだが、stopped 無しで \
                 terminated した（次の停止点へ到達できていない）",
                fx.next_pasta_line
            );
        }

        // host VM スレッドが watchdog 内で完了することを確認（ハング無し）。
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion after continue");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    /// Task 3.3 — ループ再訪の実 DAP-over-TCP E2E
    /// （spec: `pasta-debug-break-coalesce`, requirements **2.2** / **6.2(b)**）。
    ///
    /// # 観測する「done」
    ///
    /// 同一 `.pasta` 行（`SCENE.加算ループ` 本体の `total = total + i`）をループで N 回
    /// 再訪する構成で、その `.pasta` 行へ BP を張る。停止のたびに `continue` を送ると、ループ
    /// 反復ごとに **同一 `.pasta` 行で再び停止** し、合計 **ちょうど N 回**（= `LOOP_VISITS`）の
    /// `stopped`（reason breakpoint・同一提示行）を観測できることを実ソケットで証明する。
    ///
    /// アンカー coalescing（fix）は **同一 `.pasta` 行に連続して留まる間**だけ再停止を抑止する。
    /// ループは反復ごとに `for ... do`（条件/増分 = 本体とは別の `.pasta` 行）を経由するため、
    /// 本体行へ再入するたびにアンカーがクリアされ、再び停止する。したがって観測される停止数は
    ///   - over-suppression（coalescing がループをまたいで効いてしまう）なら **< N**、
    ///   - 壊れた coalescing（同一行内で複数停止）なら **> N**、
    ///
    /// となり、**ちょうど N** を要求することで両誤りを弁別する（テストの「歯」）。
    ///
    /// # ハーネス
    ///
    /// [`build_fixture`] と同一の本番ローダ経路でマップ＋生成 `.lua` を得るが、本テストは
    /// **ループ本体 `.pasta` 行**（[`LOOP_BODY_MARKER`]）へ BP を張り、`SCENE.加算ループ(ACT)`
    /// を呼んでループ本体を N 回実行させる（多対1 行ではなくループ行が観測対象）。
    #[test]
    fn loop_revisit_yields_one_stop_per_iteration_over_tcp() {
        // --- フィクスチャ（本番ローダ経路）を build_fixture と同流儀で構築し、ループ本体行を導出 ---
        let temp = tempfile::TempDir::new().expect("temp dir");
        let base_dir = temp.path().to_path_buf();
        let pasta_file = base_dir.join("dic/test/debug_break_coalesce.pasta");
        std::fs::create_dir_all(pasta_file.parent().unwrap()).expect("mkdir dic");
        std::fs::write(&pasta_file, FIXTURE).expect("write .pasta");

        let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
        let map = crate::loader::PastaLoader::build_source_map(
            std::slice::from_ref(&pasta_file),
            &cache_manager,
            false,
        );
        let chunk_name = cache_manager
            .source_to_cache_path(&pasta_file)
            .to_string_lossy()
            .to_string();
        let pasta_path = pasta_file.to_string_lossy().to_string();

        // 生成 `.lua` 本文（map 構築とバイト一致）。
        let content = std::fs::read_to_string(&pasta_file).expect("read .pasta");
        let parsed = pasta_dsl::parse_str(&content, &pasta_path).expect("parse .pasta");
        let transpiler = LuaTranspiler::default();
        let mut sink = MapBuilderSink::new(pasta_path.clone(), chunk_name.clone());
        let mut out = Vec::new();
        transpiler
            .transpile_with_source_map(&parsed, &mut out, Some(&mut sink))
            .expect("transpile .pasta");
        let lua_source = String::from_utf8(out).expect("utf8 .lua");

        // ループ本体 `.pasta` 行は単一の `.lua` 実行座標へ対応する（前提＝再訪は同一行）。
        let loop_pasta_line = unique_pasta_line(LOOP_BODY_MARKER);
        let loop_lua_lines: Vec<u32> = map
            .resolve_pasta_to_lua(&pasta_path, loop_pasta_line)
            .into_iter()
            .map(|(_chunk, lua_line)| lua_line)
            .collect();
        assert_eq!(
            loop_lua_lines.len(),
            1,
            "6.2(b) 前提: ループ本体 `.pasta` 行 {loop_pasta_line} は単一 `.lua` 座標を持つ: \
             {loop_lua_lines:?}"
        );

        drop(temp);

        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (loaded_tx, loaded_rx) = mpsc::channel::<()>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        let map_host = Arc::clone(&map);

        // VM HOST スレッド。
        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_mode: SourceMode::Pasta,
                ..Default::default()
            };
            let handle = enable(&lua, &cfg, Some(map_host))
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            // require シム（BP 設定前）。
            lua.load(PASTA_SHIM)
                .set_name("@pasta_shim")
                .exec()
                .map_err(|e| format!("shim exec failed: {e}"))?;

            // 生成 `.lua` 本文をローダ由来チャンク名で実行し、`SCENE.加算ループ` 等を定義する。
            // BP 未設定なので定義時の通過では停止しない。
            lua.load(&lua_source)
                .set_name(format!("@{chunk_name}"))
                .exec()
                .map_err(|e| format!("chunk (definitions) exec failed: {e}"))?;

            loaded_tx
                .send(())
                .map_err(|_| "loaded signal send failed".to_string())?;
            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before calling 加算ループ".to_string())?;

            // ここで初めてループ本体行が **反復実行**される: 捕捉した SCENE の `加算ループ(ACT)`
            // を呼ぶと `for i = 1, 3 do total = total + i end` がループ本体行を N 回再訪する。
            lua.load("local s = package.loaded['pasta'].__last_scene; return s['加算ループ'](ACT)")
                .set_name("@invoke_loop")
                .exec()
                .map_err(|e| format!("加算ループ call failed: {e}"))?;

            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        // CLIENT。
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let _ = client.recv_until(|m| is_response(m, "initialize"));
        let _ = client.recv_until(|m| is_event(m, "initialized"));

        loaded_rx
            .recv_timeout(WATCHDOG)
            .expect("host must finish definitions before the watchdog");

        // ループ本体 `.pasta` 行のみへ BP を張る。
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": pasta_path },
                "breakpoints": [ { "line": loop_pasta_line } ],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 1, "1 つの BP 応答（ループ本体行）");
        assert!(
            bps.iter().all(|b| b["verified"] == true),
            "ループ本体 `.pasta` 行 BP は verified で登録される: {bps:?}"
        );

        client.send_request(3, "configurationDone", json!({}));
        let _ = client.recv_until(|m| is_response(m, "configurationDone"));
        go_tx.send(()).expect("send go signal");

        // === 「歯」: 停止のたびに continue。ループは N 回まわるので、同一 `.pasta` 行で
        // ちょうど N 回 stop する。< N なら coalescing がループをまたいで効きすぎ、> N なら
        // 同一行内で過剰停止（coalescing 不全）。両者を「ちょうど N」で弁別する。===
        let mut stop_count = 0usize;
        let mut seq = 10u64;
        loop {
            let ev = client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
            if is_event(&ev, "terminated") {
                break;
            }
            // stopped: ループ本体 `.pasta` 行であることを検証して数える。
            assert_eq!(
                ev["body"]["reason"], "breakpoint",
                "2.2: ループ本体行 BP に対応する `.lua` 行で停止する"
            );
            let tid = ev["body"]["threadId"].as_u64().expect("threadId");
            let stop_line = top_pasta_line(&mut client, tid, seq);
            seq += 1;
            assert_eq!(
                stop_line, loop_pasta_line,
                "2.2/6.2(b): 各停止はループ本体 `.pasta` 行 {loop_pasta_line} であること。actual={stop_line}"
            );
            stop_count += 1;
            assert!(
                stop_count <= LOOP_VISITS,
                "2.2: 停止数がループ反復回数 {LOOP_VISITS} を超えた（coalescing 不全 / 同一行で過剰停止）: \
                 既に {stop_count} 回停止"
            );
            // 次の反復へ。
            client.send_request(seq, "continue", json!({ "threadId": tid }));
            let _ = client.recv_until(|m| is_response(m, "continue"));
            seq += 1;
        }

        // ちょうど N 回停止（再訪ごとに 1 回、再訪ごとにアンカーがクリアされ再停止する）。
        assert_eq!(
            stop_count, LOOP_VISITS,
            "2.2/6.2(b): ループ本体 `.pasta` 行はループ反復回数 {LOOP_VISITS} と同数だけ停止すること \
             （N 訪問 → N 停止）。actual={stop_count}"
        );

        // host VM スレッドが watchdog 内で完了することを確認（ハング無し）。
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("loop scenario must run to completion");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    /// stackTrace の top フレーム `line`（`.pasta` 提示中は `.pasta` 行）を返し、`.pasta`
    /// 提示の「歯」を効かせる（[`super::pasta_step_e2e::top_pasta_line`] と同型）。
    fn top_pasta_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
        client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame");
        let top_src = frames[0]["source"]["path"]
            .as_str()
            .expect("top frame source path");
        assert!(
            top_src.ends_with(".pasta"),
            "`.pasta` 提示中は top フレームが `.pasta` を提示すること: {top_src:?}"
        );
        frames[0]["line"].as_u64().expect("top frame line") as u32
    }
}

#[cfg(test)]
mod source_presentation_toggle_tests {
    //! Task 3.1 — the runtime presentation-toggle WIRING in [`handle_inbound`]
    //! (requirements 1.1/1.2/1.4/1.5, 2.5/2.6, 3.1/3.2/3.4/3.5, 4.2/4.3).
    //!
    //! A `pasta/sourcePresentation` custom request must, IN ORDER: apply the mode
    //! to the shared cell + swap the resolver (valid mode only), send the
    //! acceptance response, emit the `pasta/sourcePresentation` custom event, and
    //! forward `SessionCommand::RefreshPresentation`. An UNRECOGNIZED mode value
    //! makes NO cell change (1.4) but still acks + echoes the current mode. The
    //! `attach` completion (explicit AND no-arg) emits the resolved initial-mode
    //! event (2.5 initial display).
    //!
    //! These tests drive [`handle_inbound`] DIRECTLY over a real loopback
    //! [`Transport`] (a connected client reads the response/event frames), so the
    //! exact wire frames, the cell state, the swapped resolver (observed via a
    //! subsequent `stackTrace` encode), and the forwarded command are all asserted.

    use std::collections::BTreeMap;
    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::mpsc::{self, Receiver};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::breakpoints::BreakpointSet;
    use crate::debug::dap::DapAdapter;
    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
    use crate::debug::transport::{Transport, read_frame};
    use crate::debug::types::{FrameInfo, SessionCommand, SessionEvent};
    use crate::debug::{SharedSourceMode, SourceMode};

    use super::{SharedAdapter, SourceMapWiring, handle_inbound};

    /// TEST-ONLY watchdog so a wiring test cannot hang on a frame read.
    const WATCHDOG: Duration = Duration::from_secs(10);

    /// A `chunk(.lua line) → .pasta` map with a single known correspondence, so a
    /// `stackTrace` encode observably switches presentation when the `.pasta`
    /// resolver is installed.
    fn map_with(chunk: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
        let mut forward = BTreeMap::new();
        forward.insert(
            lua_line,
            PastaPos {
                file: file.to_string(),
                line: pasta_line,
            },
        );
        let mut sm = SourceMap::new();
        sm.insert_chunk(
            chunk.to_string(),
            file.to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        sm
    }

    /// Pasta-capable wiring (map present) whose EFFECTIVE mode starts at `start`.
    fn wiring_with(map: SourceMap, start: SourceMode) -> SourceMapWiring {
        SourceMapWiring {
            source_map: Some(Arc::new(map)),
            source_mode: SharedSourceMode::new(start),
        }
    }

    /// A real loopback [`Transport`] plus a connected client end. The bridge owns
    /// the `Transport` (its writer thread serializes `send()`s to the socket); the
    /// `client` reads those frames back. Lets a wiring test observe the EXACT wire
    /// frames `handle_inbound` emits via `transport.send`.
    struct Harness {
        transport: Transport,
        client: BufReader<TcpStream>,
    }

    impl Harness {
        fn new() -> Self {
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let transport = Transport::start(Some(addr)).expect("bind loopback transport");
            let bound = transport.local_addr().expect("bound addr");
            let stream = TcpStream::connect(bound).expect("client connects to bridge");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            Self {
                transport,
                client: BufReader::new(stream),
            }
        }

        /// Read the next framed message the bridge wrote (bounded by the timeout).
        fn recv(&mut self) -> Value {
            read_frame(&mut self.client)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present")
        }
    }

    /// Build a `pasta/sourcePresentation` request Value with `seq` and a `mode`.
    fn toggle_req(seq: u64, mode: &str) -> Value {
        json!({
            "seq": seq,
            "type": "request",
            "command": "pasta/sourcePresentation",
            "arguments": { "mode": mode },
        })
    }

    /// Encode ONE `stackTrace` over `adapter` for `(source, line)` and return the
    /// top frame's presented `source`/`line` — observes which resolver is live.
    fn top_frame(adapter: &SharedAdapter, source: &str, line: u32) -> (Value, u32) {
        let mut dap = adapter.lock().unwrap();
        dap.decode_request(&json!({
            "seq": 9, "type": "request", "command": "stackTrace",
            "arguments": { "threadId": 1 },
        }));
        let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
            source: source.to_string(),
            line,
            func_name: Some("f".to_string()),
        }]));
        let frame = &out[0]["body"]["stackFrames"][0];
        (frame["source"].clone(), frame["line"].as_u64().unwrap() as u32)
    }

    /// 1.1 / 4.2 / 4.3 / 3.1 / 3.4 / 2.6 / 1.3: a valid `pasta/sourcePresentation`
    /// toggle to `lua` from a `.pasta` default — updates the cell, swaps the
    /// resolver to default `.lua` (observed via a subsequent stackTrace), sends the
    /// acceptance response echoing `lua`, emits the custom event `lua`, and
    /// forwards `RefreshPresentation`, in that exact order.
    #[test]
    fn valid_toggle_to_lua_applies_acks_events_and_forwards_refresh() {
        let mut h = Harness::new();
        let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Pasta,
        );
        // Install the initial (`.pasta`) resolver, mirroring run_socket_bridge.
        super::attach_pasta_resolver(&adapter, &wiring);
        // Precondition: Pasta presents the `.pasta` source.
        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(src, json!({ "path": "C:/proj/scene.pasta" }));
        assert_eq!(line, 3);

        let ok = handle_inbound(
            &h.transport,
            &adapter,
            &breakpoints,
            &cmd_tx,
            &toggle_req(70, "lua"),
            &wiring,
        );
        assert!(ok, "handle_inbound must not report the peer gone");

        // (a) the cell is updated to Lua (1.1 / 4.2 / 4.3).
        assert_eq!(wiring.source_mode.get(), SourceMode::Lua, "cell set to lua (1.1)");

        // (b) the resolver swapped to default `.lua` — the SAME frame now presents
        // the generated `.lua` (3.1 / 3.4).
        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            src,
            json!({ "path": r"@C:\proj\cache\scene.lua" }),
            "resolver swapped → generated `.lua` presentation (3.1/3.4)"
        );
        assert_eq!(line, 7);

        // (c) acceptance response FIRST (1.3), echoing the resolved current mode.
        let resp = h.recv();
        assert_eq!(resp["type"], "response");
        assert_eq!(resp["command"], "pasta/sourcePresentation");
        assert_eq!(resp["request_seq"], 70, "ack correlates to the request seq (1.3)");
        assert_eq!(resp["success"], true);
        assert_eq!(resp["body"]["mode"], "lua", "ack echoes the resolved mode (1.3)");

        // (d) THEN the custom event with the current mode (2.6 push notification).
        let ev = h.recv();
        assert_eq!(ev["type"], "event");
        assert_eq!(ev["event"], "pasta/sourcePresentation");
        assert_eq!(ev["body"]["mode"], "lua", "event carries the new mode (2.6)");

        // (e) RefreshPresentation forwarded to the session (3.3 path).
        let cmd = cmd_rx
            .recv_timeout(WATCHDOG)
            .expect("RefreshPresentation must be forwarded to the session");
        assert_eq!(cmd, SessionCommand::RefreshPresentation);
        assert!(cmd_rx.try_recv().is_err(), "exactly one command forwarded");
    }

    /// 1.2 / 4.2 / 3.2 / 3.5: a valid toggle to `pasta` from a `.lua` default —
    /// updates the cell to Pasta and swaps in the `.pasta` resolver (observed via a
    /// subsequent stackTrace presenting `.pasta`).
    #[test]
    fn valid_toggle_to_pasta_swaps_in_pasta_resolver() {
        let mut h = Harness::new();
        let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Lua,
        );
        super::attach_pasta_resolver(&adapter, &wiring);
        // Precondition: Lua presents the generated `.lua`.
        let (src, _line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(src, json!({ "path": r"@C:\proj\cache\scene.lua" }));

        let ok = handle_inbound(
            &h.transport,
            &adapter,
            &breakpoints,
            &cmd_tx,
            &toggle_req(71, "pasta"),
            &wiring,
        );
        assert!(ok);
        assert_eq!(wiring.source_mode.get(), SourceMode::Pasta, "cell set to pasta (1.2)");

        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            src,
            json!({ "path": "C:/proj/scene.pasta" }),
            "resolver swapped → `.pasta` presentation (3.2/3.5)"
        );
        assert_eq!(line, 3);

        // ack `pasta`, event `pasta`, RefreshPresentation.
        let resp = h.recv();
        assert_eq!(resp["request_seq"], 71);
        assert_eq!(resp["body"]["mode"], "pasta");
        let ev = h.recv();
        assert_eq!(ev["event"], "pasta/sourcePresentation");
        assert_eq!(ev["body"]["mode"], "pasta");
        assert_eq!(
            cmd_rx.recv_timeout(WATCHDOG).unwrap(),
            SessionCommand::RefreshPresentation
        );
    }

    /// 1.4: an UNRECOGNIZED mode value makes NO cell change and does NOT swap the
    /// resolver, but STILL acks + echoes the CURRENT (unchanged) mode and emits the
    /// event + forwards RefreshPresentation (the request is still accepted; only
    /// the mode is left as-is).
    #[test]
    fn invalid_mode_leaves_cell_unchanged_but_acks_current() {
        let mut h = Harness::new();
        let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Pasta,
        );
        super::attach_pasta_resolver(&adapter, &wiring);

        let ok = handle_inbound(
            &h.transport,
            &adapter,
            &breakpoints,
            &cmd_tx,
            &toggle_req(72, "bogus"),
            &wiring,
        );
        assert!(ok);

        // 1.4: NO change — still Pasta, and the `.pasta` resolver is still live.
        assert_eq!(
            wiring.source_mode.get(),
            SourceMode::Pasta,
            "1.4: unrecognized mode must NOT change the cell"
        );
        let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
        assert_eq!(
            src,
            json!({ "path": "C:/proj/scene.pasta" }),
            "1.4: resolver unchanged (still `.pasta`)"
        );
        assert_eq!(line, 3);

        // Still acks + echoes the CURRENT mode (pasta), event, RefreshPresentation.
        let resp = h.recv();
        assert_eq!(resp["request_seq"], 72);
        assert_eq!(resp["body"]["mode"], "pasta", "1.4: echo the current (unchanged) mode");
        let ev = h.recv();
        assert_eq!(ev["body"]["mode"], "pasta");
        assert_eq!(
            cmd_rx.recv_timeout(WATCHDOG).unwrap(),
            SessionCommand::RefreshPresentation
        );
    }

    /// 2.5 (initial display): `attach` WITH an explicit `sourcePresentation` emits
    /// the resolved initial-mode `pasta/sourcePresentation` event AFTER the attach
    /// ack, so the extension can show the resolved initial mode.
    #[test]
    fn attach_with_explicit_mode_emits_initial_event() {
        let mut h = Harness::new();
        let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Pasta,
        );
        super::attach_pasta_resolver(&adapter, &wiring);

        let attach = json!({
            "seq": 5, "type": "request", "command": "attach",
            "arguments": { "sourcePresentation": "lua" },
        });
        let ok = handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &attach, &wiring);
        assert!(ok);
        assert_eq!(wiring.source_mode.get(), SourceMode::Lua, "explicit attach mode applied");

        // attach ack first, then the resolved initial-mode event.
        let ack = h.recv();
        assert_eq!(ack["type"], "response");
        assert_eq!(ack["command"], "attach");
        let ev = h.recv();
        assert_eq!(ev["type"], "event");
        assert_eq!(ev["event"], "pasta/sourcePresentation");
        assert_eq!(ev["body"]["mode"], "lua", "2.5: event carries the resolved initial mode");
    }

    /// 2.5 (initial display, no-arg attach): `attach` WITHOUT `sourcePresentation`
    /// keeps the resolved env/file/既定 mode and STILL emits the resolved
    /// initial-mode event (the no-arg path must also publish the initial display).
    #[test]
    fn attach_without_mode_emits_resolved_initial_event() {
        let mut h = Harness::new();
        let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
        let wiring = wiring_with(
            map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
            SourceMode::Pasta,
        );
        super::attach_pasta_resolver(&adapter, &wiring);

        let attach = json!({
            "seq": 6, "type": "request", "command": "attach",
            "arguments": {},
        });
        let ok = handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &attach, &wiring);
        assert!(ok);
        // No-arg → the resolved mode (Pasta) is kept.
        assert_eq!(wiring.source_mode.get(), SourceMode::Pasta, "no-arg attach keeps resolved mode");

        let ack = h.recv();
        assert_eq!(ack["command"], "attach");
        let ev = h.recv();
        assert_eq!(ev["event"], "pasta/sourcePresentation");
        assert_eq!(
            ev["body"]["mode"], "pasta",
            "2.5: no-arg attach still publishes the resolved initial mode"
        );
    }
}

#[cfg(test)]
mod bridge_lifecycle_tests {
    //! Bridge-thread LIFECYCLE and degraded paths that the E2E suites above
    //! never reach in isolation: [`run_event_encoder`] termination conditions
    //! (event channel close / frame receiver gone / poisoned adapter),
    //! [`drain_outbound`]'s "closed `out_rx` is NOT a stop condition" contract,
    //! [`run_socket_bridge`]'s preset-shutdown fast exit and its
    //! flush-pending-outbound-on-client-disconnect branch, and
    //! [`handle_inbound`]'s error/fallback paths (missing `seq` → `request_seq`
    //! 0, session gone → `false` after the already-written ack frames,
    //! unknown/missing command silently ignored, poisoned adapter → stop, never
    //! panic).
    //!
    //! All sockets bind `127.0.0.1:0` (ephemeral); every wait is a bounded
    //! blocking wait on a channel/socket (no sleeps-as-sync).

    use std::io::BufReader;
    use std::net::{Shutdown, SocketAddr, TcpStream};
    use std::sync::atomic::AtomicBool;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::SourceMode;
    use crate::debug::breakpoints::BreakpointSet;
    use crate::debug::dap::DapAdapter;
    use crate::debug::source_map::SourceMap;
    use crate::debug::transport::{Transport, read_frame, write_frame};
    use crate::debug::types::{SessionCommand, SessionEvent, StopReason};

    use super::{
        SharedAdapter, SourceMapWiring, attach_pasta_resolver, drain_outbound, handle_inbound,
        run_event_encoder, run_socket_bridge,
    };

    /// TEST-ONLY watchdog so a lifecycle test cannot hang on a read/join.
    const WATCHDOG: Duration = Duration::from_secs(10);

    fn shared_adapter() -> SharedAdapter {
        Arc::new(Mutex::new(DapAdapter::new()))
    }

    /// Poison `adapter`'s mutex by panicking a scratch thread while it holds the
    /// lock (the only way a `Mutex` becomes poisoned).
    fn poison(adapter: &SharedAdapter) {
        let poisoner = Arc::clone(adapter);
        let _ = std::thread::spawn(move || {
            let _guard = poisoner.lock().unwrap();
            panic!("TEST-ONLY: poison the shared adapter lock");
        })
        .join();
        assert!(adapter.lock().is_err(), "the adapter lock must now be poisoned");
    }

    /// A real loopback [`Transport`] plus a connected client end (same pattern as
    /// `source_presentation_toggle_tests::Harness`, with a write half so a test
    /// can also push requests toward the bridge).
    struct Harness {
        transport: Transport,
        client: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl Harness {
        fn new() -> Self {
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let transport = Transport::start(Some(addr)).expect("bind loopback transport");
            let bound = transport.local_addr().expect("bound addr");
            let stream = TcpStream::connect(bound).expect("client connects to bridge");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone client socket for writing");
            Self {
                transport,
                client: BufReader::new(stream),
                writer,
            }
        }

        /// Read the next framed message the bridge wrote (bounded by the timeout).
        fn recv(&mut self) -> Value {
            read_frame(&mut self.client)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present")
        }
    }

    /// [`run_event_encoder`] happy path + clean termination: a session event is
    /// encoded into a DAP frame on `out_tx`, and closing the session's event
    /// channel ends the encoder thread (never a hang).
    #[test]
    fn event_encoder_forwards_frames_and_ends_when_event_channel_closes() {
        let adapter = shared_adapter();
        let (event_tx, event_rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel();
        let encoder = std::thread::spawn(move || run_event_encoder(adapter, event_rx, out_tx));

        event_tx
            .send(SessionEvent::Stopped {
                reason: StopReason::Breakpoint,
                thread_id: 1,
            })
            .expect("encoder is alive");
        let frame = out_rx
            .recv_timeout(WATCHDOG)
            .expect("the encoded frame must arrive on out_tx");
        assert_eq!(frame["type"], "event");
        assert_eq!(frame["event"], "stopped");
        assert_eq!(frame["body"]["reason"], "breakpoint");
        assert_eq!(frame["body"]["threadId"], 1);

        drop(event_tx);
        encoder
            .join()
            .expect("closing the event channel must end the encoder cleanly");
    }

    /// [`run_event_encoder`] ends when the frame channel's receiver is gone (the
    /// socket bridge exited first): the next encoded frame fails to send and the
    /// encoder returns instead of looping or panicking.
    #[test]
    fn event_encoder_ends_when_frame_receiver_is_gone() {
        let adapter = shared_adapter();
        let (event_tx, event_rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel::<Value>();
        drop(out_rx); // socket bridge already gone
        let encoder = std::thread::spawn(move || run_event_encoder(adapter, event_rx, out_tx));

        event_tx
            .send(SessionEvent::Terminated)
            .expect("queued before the encoder observes the closed frame channel");
        encoder
            .join()
            .expect("a gone frame receiver must end the encoder, not panic it");
        // event_tx is still alive: termination was driven by the frame channel.
        drop(event_tx);
    }

    /// [`run_event_encoder`] under a POISONED shared adapter: the encoder returns
    /// (no panic propagation, no busy loop) and emits no frame.
    #[test]
    fn event_encoder_ends_on_poisoned_adapter_without_panic() {
        let adapter = shared_adapter();
        poison(&adapter);
        let (event_tx, event_rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel::<Value>();
        let encoder = std::thread::spawn(move || run_event_encoder(adapter, event_rx, out_tx));

        event_tx
            .send(SessionEvent::Terminated)
            .expect("encoder is blocked on recv, channel is open");
        encoder
            .join()
            .expect("a poisoned adapter must end the encoder cleanly, not panic it");
        assert!(
            out_rx.try_recv().is_err(),
            "no frame may be emitted under a poisoned adapter"
        );
    }

    /// [`drain_outbound`]'s documented contract: pending frames are flushed to
    /// the socket, and a CLOSED `out_rx` (encoder thread ended) is NOT a stop
    /// condition — the bridge must keep serving inbound, so it returns `true`.
    #[test]
    fn drain_outbound_flushes_pending_and_treats_closed_channel_as_ok() {
        let mut h = Harness::new();
        let (out_tx, out_rx) = mpsc::channel();
        out_tx.send(json!({ "type": "event", "event": "first" })).unwrap();
        out_tx.send(json!({ "type": "event", "event": "second" })).unwrap();
        drop(out_tx); // encoder thread is gone; frames are still queued

        assert!(
            drain_outbound(&h.transport, &out_rx),
            "a closed out_rx must NOT stop the bridge (inbound may still arrive)"
        );
        assert_eq!(h.recv()["event"], "first", "queued frames are flushed in order");
        assert_eq!(h.recv()["event"], "second");
    }

    /// [`run_socket_bridge`] honours a PRESET shutdown flag: it exits before
    /// serving anything — a request already queued by the client gets no
    /// response; the client just observes the connection winding down.
    #[test]
    fn socket_bridge_exits_on_preset_shutdown_without_serving() {
        let Harness {
            transport,
            mut client,
            mut writer,
        } = Harness::new();
        write_frame(
            &mut writer,
            &json!({ "seq": 1, "type": "request", "command": "initialize", "arguments": {} }),
        )
        .expect("client queues a request before the bridge starts");

        let (cmd_tx, _cmd_rx) = mpsc::channel();
        let (_out_tx, out_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(true)); // PRESET before the loop
        let adapter = shared_adapter();
        let bridge = std::thread::spawn(move || {
            run_socket_bridge(
                transport,
                adapter,
                BreakpointSet::new(),
                cmd_tx,
                out_rx,
                shutdown,
                SourceMapWiring::disabled(),
            )
        });
        bridge
            .join()
            .expect("a preset shutdown flag must end the bridge immediately");

        // The bridge exited BEFORE answering: EOF/reset (`Ok(None)`/`Err` are
        // both a clean wind-down), never an initialize response frame.
        if let Ok(Some(frame)) = read_frame(&mut client) {
            panic!("bridge must exit before serving the queued request; got {frame}");
        }
    }

    /// [`run_socket_bridge`]'s inbound-`Disconnected` arm: when the client
    /// disconnects (write half closed → reader EOF → inbound channel closes),
    /// frames still pending in `out_rx` are FLUSHED to the socket before the
    /// bridge returns — the client can read them up to EOF.
    #[test]
    fn socket_bridge_flushes_pending_outbound_on_client_disconnect() {
        let Harness {
            transport,
            mut client,
            writer,
        } = Harness::new();

        // Half-close the client's write side, then wait (bounded, channel-close
        // based) until the transport reader saw EOF — the bridge's FIRST inbound
        // poll is then deterministically `Disconnected`.
        writer
            .shutdown(Shutdown::Write)
            .expect("half-close the client write side");
        match transport.inbound().recv_timeout(WATCHDOG) {
            Err(RecvTimeoutError::Disconnected) => {}
            other => panic!("inbound must close on client write-shutdown, got {other:?}"),
        }

        // Frames the (already finished) encoder left behind.
        let (out_tx, out_rx) = mpsc::channel();
        out_tx.send(json!({ "type": "event", "event": "pending-1" })).unwrap();
        out_tx.send(json!({ "type": "event", "event": "pending-2" })).unwrap();
        drop(out_tx);

        let (cmd_tx, _cmd_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let adapter = shared_adapter();
        let bridge = std::thread::spawn(move || {
            run_socket_bridge(
                transport,
                adapter,
                BreakpointSet::new(),
                cmd_tx,
                out_rx,
                shutdown,
                SourceMapWiring::disabled(),
            )
        });
        bridge
            .join()
            .expect("the bridge must end on inbound disconnect, not hang");

        // The pending frames were flushed before the transport wound down.
        let first = read_frame(&mut client)
            .expect("read flushed frame")
            .expect("pending-1 must be flushed on disconnect");
        assert_eq!(first["event"], "pending-1");
        let second = read_frame(&mut client)
            .expect("read flushed frame")
            .expect("pending-2 must be flushed on disconnect");
        assert_eq!(second["event"], "pending-2");
        // Then the connection winds down (EOF or reset), with no further frames.
        if let Ok(Some(frame)) = read_frame(&mut client) {
            panic!("no further frames expected; got {frame}");
        }
    }

    /// A `pasta/sourcePresentation` toggle WITHOUT a `seq` field degrades to
    /// `request_seq` 0 in the acceptance response (the documented fallback at
    /// the toggle entry) — the toggle itself still fully applies.
    #[test]
    fn toggle_missing_seq_acks_with_request_seq_zero() {
        let mut h = Harness::new();
        let adapter = shared_adapter();
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let wiring = SourceMapWiring::disabled(); // effective mode starts Pasta

        let req = json!({
            "type": "request",
            "command": "pasta/sourcePresentation",
            "arguments": { "mode": "lua" },
        }); // NO "seq"
        assert!(handle_inbound(
            &h.transport,
            &adapter,
            &breakpoints,
            &cmd_tx,
            &req,
            &wiring
        ));

        assert_eq!(wiring.source_mode.get(), SourceMode::Lua, "the toggle still applies");
        let ack = h.recv();
        assert_eq!(ack["command"], "pasta/sourcePresentation");
        assert_eq!(ack["request_seq"], 0, "missing seq must degrade to request_seq 0");
        assert_eq!(ack["body"]["mode"], "lua");
        let ev = h.recv();
        assert_eq!(ev["event"], "pasta/sourcePresentation");
        assert_eq!(
            cmd_rx.recv_timeout(WATCHDOG).expect("RefreshPresentation forwarded"),
            SessionCommand::RefreshPresentation
        );
    }

    /// A valid toggle when the SESSION IS GONE (`cmd_rx` dropped): the ack and
    /// the custom event are already on the wire (they precede the forward), and
    /// `handle_inbound` then reports `false` so the bridge stops.
    #[test]
    fn toggle_with_session_gone_still_acks_then_reports_stop() {
        let mut h = Harness::new();
        let adapter = shared_adapter();
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        drop(cmd_rx); // session controller gone
        let wiring = SourceMapWiring::disabled();

        let req = json!({
            "seq": 5,
            "type": "request",
            "command": "pasta/sourcePresentation",
            "arguments": { "mode": "lua" },
        });
        assert!(
            !handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &req, &wiring),
            "a gone session must stop the bridge (false)"
        );

        // Ack + event were written BEFORE the failed RefreshPresentation forward.
        let ack = h.recv();
        assert_eq!(ack["command"], "pasta/sourcePresentation");
        assert_eq!(ack["request_seq"], 5);
        let ev = h.recv();
        assert_eq!(ev["event"], "pasta/sourcePresentation");
    }

    /// A stop-context command (here `continue`) when the SESSION IS GONE: the
    /// immediate ack response is already on the wire, then the failed forward
    /// makes `handle_inbound` report `false`.
    #[test]
    fn stop_context_command_with_session_gone_reports_stop_after_ack() {
        let mut h = Harness::new();
        let adapter = shared_adapter();
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx) = mpsc::channel();
        drop(cmd_rx); // session controller gone

        let req = json!({ "seq": 3, "type": "request", "command": "continue", "arguments": {} });
        assert!(
            !handle_inbound(
                &h.transport,
                &adapter,
                &breakpoints,
                &cmd_tx,
                &req,
                &SourceMapWiring::disabled()
            ),
            "a gone session must stop the bridge (false)"
        );

        let ack = h.recv();
        assert_eq!(ack["command"], "continue", "the ack precedes the failed forward");
        assert_eq!(ack["request_seq"], 3);
    }

    /// Unknown / missing `command` requests decode to an empty outcome and are
    /// silently ignored (`true`): no frame, no session command — and the bridge
    /// keeps serving subsequent valid requests on the same connection.
    #[test]
    fn unknown_and_missing_command_are_ignored_and_bridge_keeps_serving() {
        let mut h = Harness::new();
        let adapter = shared_adapter();
        let breakpoints = BreakpointSet::new();
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let wiring = SourceMapWiring::disabled();

        // (1) No "command" key at all → unwrap_or("") → empty Decoded → ignored.
        let no_command = json!({ "seq": 1, "type": "request" });
        assert!(handle_inbound(
            &h.transport,
            &adapter,
            &breakpoints,
            &cmd_tx,
            &no_command,
            &wiring
        ));
        // (2) An unrecognized command → empty Decoded → ignored.
        let bogus = json!({ "seq": 2, "type": "request", "command": "bogusCommand" });
        assert!(handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &bogus, &wiring));
        assert!(
            cmd_rx.try_recv().is_err(),
            "ignored requests must forward no session command"
        );

        // (3) The bridge still serves: a valid initialize round-trips, and its
        // response is the FIRST frame on the wire (the ignored requests wrote
        // nothing before it — TCP preserves order).
        let init = json!({ "seq": 3, "type": "request", "command": "initialize", "arguments": {} });
        assert!(handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &init, &wiring));
        let resp = h.recv();
        assert_eq!(resp["command"], "initialize", "first wire frame is the initialize response");
        assert_eq!(resp["request_seq"], 3);
        let ev = h.recv();
        assert_eq!(ev["event"], "initialized");
    }

    /// POISONED shared adapter: [`attach_pasta_resolver`] must not panic on
    /// either gate outcome (active and inactive), and [`handle_inbound`] reports
    /// `false` (stop the bridge) instead of panicking.
    #[test]
    fn poisoned_adapter_never_panics_resolver_attach_and_stops_inbound() {
        let h = Harness::new();
        let adapter = shared_adapter();
        poison(&adapter);

        // Inactive gate (no map): the default-resolver reset path is skipped
        // silently on a poisoned lock.
        attach_pasta_resolver(&adapter, &SourceMapWiring::disabled());
        // Active gate (map + Pasta): the `.pasta`-resolver install path is
        // likewise skipped silently.
        let active = SourceMapWiring {
            source_map: Some(Arc::new(SourceMap::new())),
            source_mode: crate::debug::SharedSourceMode::new(SourceMode::Pasta),
        };
        attach_pasta_resolver(&adapter, &active);

        // handle_inbound cannot decode under a poisoned lock → stop, no panic.
        let (cmd_tx, _cmd_rx) = mpsc::channel();
        let req = json!({ "seq": 1, "type": "request", "command": "initialize", "arguments": {} });
        assert!(
            !handle_inbound(
                &h.transport,
                &adapter,
                &BreakpointSet::new(),
                &cmd_tx,
                &req,
                &SourceMapWiring::disabled()
            ),
            "a poisoned adapter must stop the bridge, never panic it"
        );
    }
}
