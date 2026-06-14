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
use crate::debug::dap::{DapAdapter, Decoded, pasta_source_resolver};
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
///
/// # Fixed `apply → response → event → command` order (requirement 4.2)
///
/// After the inline poison/decode guard, the work is a FIXED sequence of helper
/// calls A→B→C→D→E that MUST NOT be reordered — the order is the load-bearing
/// contract (design "System Flows / C4"):
///
/// - **A** [`try_source_presentation_toggle`]: the self-contained
///   `pasta/sourcePresentation` runtime toggle (`apply → ack → event →
///   RefreshPresentation`); when it handles the request it returns directly.
/// - **B** [`apply_attach_source_mode`]: APPLY an explicit `attach`
///   `sourcePresentation` to the shared effective mode BEFORE replying.
/// - **C** [`send_immediate_response_and_events`]: the immediate RESPONSE
///   (acks / initialize / scopes self-answer) then the immediate handshake
///   EVENTS — response strictly before events.
/// - **D** [`emit_attach_initial_presentation_event`]: the `attach`-completion
///   initial-presentation EVENT, emitted AFTER the attach ack (ack before event).
/// - **E** [`route_command`]: COMMAND routing. `setBreakpoints` is applied
///   ATOMICALLY to the shared breakpoint store (apply + encode + send) and is
///   NEVER forwarded to the session `cmd_tx` (that would block off a stop);
///   every other stop-context command is forwarded as-is (requirement 4.1).
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
    if let Some(done) =
        try_source_presentation_toggle(transport, adapter, cmd_tx, req, source_map, command, &decoded)
    {
        return done;
    }

    apply_attach_source_mode(adapter, source_map, &decoded);

    // (a)+(b) Immediate response (acks / initialize / scopes self-answer) followed
    // by the immediate unsolicited handshake events (task 5.3 Helper C). Peer gone
    // while replying → stop.
    if !send_immediate_response_and_events(transport, &decoded) {
        return false;
    }

    // Attach-completion initial-presentation event, emitted AFTER the attach ack so
    // the ack precedes the event (task 5.3 Helper D). Peer gone → stop.
    if !emit_attach_initial_presentation_event(transport, adapter, source_map, command) {
        return false;
    }

    // (c) Command routing (Helper E). `setBreakpoints` is applied atomically and
    // NOT forwarded; every other stop-context command is forwarded as-is. `decoded`
    // is consumed here since E is the last step.
    route_command(transport, adapter, breakpoints, cmd_tx, source_map, decoded)
}

/// Helper E (task 5.4 / design "Components / C4" Service Interface・"System Flows /
/// C4" step E): the command-routing `match decoded.command { ... }` block extracted
/// verbatim from `handle_inbound`. It is the LAST step, so it takes `decoded` BY
/// VALUE (the `match decoded.command` consumes it). Returns the same bool the inline
/// match produced: `true` for a handled / `None` command, `false` on a peer- or
/// session-gone send failure (so the bridge stops).
///
/// CRITICAL invariants (requirements 4.1 / 4.4), byte-identical to the inlined
/// branch:
/// - The `SetBreakpoints` arm stays a SINGLE atomic unit — apply to the shared
///   store + encode + send — and is NEVER forwarded to `cmd_tx`.
/// - The generic `Some(cmd) => cmd_tx.send(...)` forward and the `None` no-op are
///   unchanged.
fn route_command(
    transport: &Transport,
    adapter: &SharedAdapter,
    breakpoints: &BreakpointSet,
    cmd_tx: &Sender<SessionCommand>,
    source_map: &SourceMapWiring,
    decoded: Decoded,
) -> bool {
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

/// Helper A (task 5.2 / design "Components / C4" Service Interface・"System Flows /
/// C4" step A): the self-contained `pasta/sourcePresentation` runtime TOGGLE
/// exchange extracted verbatim from `handle_inbound`. Returns `Some(true)` /
/// `Some(false)` exactly where the original branch returned `true` / `false`
/// (handled), and `None` when `command != "pasta/sourcePresentation"` so
/// `handle_inbound` falls through to the attach-apply / response / event / command
/// branches. The internal `apply → response → event → command` order and the
/// poison/peer-gone `Some(false)` propagation are byte-identical to the inlined
/// branch.
fn try_source_presentation_toggle(
    transport: &Transport,
    adapter: &SharedAdapter,
    cmd_tx: &Sender<SessionCommand>,
    req: &Value,
    source_map: &SourceMapWiring,
    command: &str,
    decoded: &Decoded,
) -> Option<bool> {
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
                Err(_) => return Some(false),
            };
            (
                dap.source_presentation_response(request_seq, current),
                dap.source_presentation_event(current),
            )
        };
        if transport.send(response).is_err() {
            return Some(false);
        }
        // (3) Custom EVENT carrying the current mode (the status-bar push
        // notification — requirements 2.5/2.6).
        if transport.send(event).is_err() {
            return Some(false);
        }
        // (4) Forward `RefreshPresentation` to the session: while STOPPED it
        // re-emits the current `Stopped` so the client refetches in the new mode
        // (requirement 3.3); while RUNNING it is a no-op until the next natural
        // stop (requirement 1.5).
        if cmd_tx.send(SessionCommand::RefreshPresentation).is_err() {
            return Some(false);
        }
        return Some(true);
    }

    None
}

/// Helper B (task 5.2 / design "Components / C4" Service Interface・"System Flows /
/// C4" step B): the explicit `attach`-mode apply extracted verbatim from
/// `handle_inbound`. Sets the SHARED effective mode and re-runs the resolver when
/// `decoded.attach_source_mode` is `Some`; never sends, never returns.
fn apply_attach_source_mode(
    adapter: &SharedAdapter,
    source_map: &SourceMapWiring,
    decoded: &Decoded,
) {
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
}

/// Helper C (task 5.3 / design "Components / C4" Service Interface・"System Flows /
/// C4" step C): the immediate response + handshake events send extracted verbatim
/// from `handle_inbound`. Returns `false` where the original branch returned
/// `false` on peer-gone (a transport write failed), else `true` to continue. The
/// response-before-events order is byte-identical to the inlined branch.
fn send_immediate_response_and_events(transport: &Transport, decoded: &Decoded) -> bool {
    // (a) Immediate response (acks / initialize / scopes self-answer).
    if let Some(response) = &decoded.response
        && transport.send(response.clone()).is_err()
    {
        return false;
    }
    // (b) Immediate unsolicited events (the `initialized` handshake event).
    for ev in &decoded.events {
        if transport.send(ev.clone()).is_err() {
            return false;
        }
    }
    true
}

/// Helper D (task 5.3 / design "Components / C4" Service Interface・"System Flows /
/// C4" step D): the `attach`-completion initial-presentation event emit extracted
/// verbatim from `handle_inbound`. Returns `false` on peer-gone (a transport write
/// failed) else `true`. Emitted AFTER the attach ack so the ack precedes the event
/// (ack-before-event preserved); keyed off the raw command string exactly as the
/// inlined branch.
fn emit_attach_initial_presentation_event(
    transport: &Transport,
    adapter: &SharedAdapter,
    source_map: &SourceMapWiring,
    command: &str,
) -> bool {
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
#[path = "wiring_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "wiring_source_map_wiring_tests.rs"]
mod source_map_wiring_tests;

#[cfg(test)]
#[path = "wiring_resolver_attach_tests.rs"]
mod resolver_attach_tests;

#[cfg(test)]
#[path = "wiring_attach_source_presentation_tests.rs"]
mod attach_source_presentation_tests;

#[cfg(test)]
#[path = "wiring_bp_translator_tests.rs"]
mod bp_translator_tests;

#[cfg(test)]
#[path = "wiring_pasta_bp_e2e.rs"]
mod pasta_bp_e2e;

#[cfg(test)]
#[path = "wiring_pasta_step_e2e.rs"]
mod pasta_step_e2e;

#[cfg(test)]
#[path = "wiring_pasta_mode_edge_e2e.rs"]
mod pasta_mode_edge_e2e;

#[cfg(test)]
#[path = "wiring_pasta_break_coalesce_e2e.rs"]
mod pasta_break_coalesce_e2e;

#[cfg(test)]
#[path = "wiring_source_presentation_toggle_tests.rs"]
mod source_presentation_toggle_tests;

#[cfg(test)]
#[path = "wiring_bridge_lifecycle_tests.rs"]
mod bridge_lifecycle_tests;
