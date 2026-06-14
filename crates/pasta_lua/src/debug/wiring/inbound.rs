//! Inbound DAP request handling: [`handle_inbound`] and its FIXED A→B→C→D→E
//! helper sequence (the `pasta/sourcePresentation` toggle, the `attach`-mode
//! apply, the immediate response+events, the attach-completion event, and the
//! command routing), plus the `.pasta` source-extension probe. Split out of the
//! `wiring` hub (C5 production split) — child of `wiring`, so it reaches the
//! hub's [`SharedAdapter`]/[`SourceMapWiring`] and the resolver sibling through
//! `super::`. Bodies are byte-identical to the flat `wiring.rs`; the A→B→C→D→E
//! order and the `setBreakpoints` atomic+non-forward arm are preserved exactly
//! (requirement 4.1).

use std::sync::mpsc::Sender;

use serde_json::Value;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::Decoded;
use crate::debug::transport::Transport;
use crate::debug::types::{SessionCommand, SessionEvent};

use super::resolver::{attach_pasta_resolver, translate_pasta_breakpoints};
use super::{SharedAdapter, SourceMapWiring};

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
pub(super) fn handle_inbound(
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
pub(super) fn is_pasta_source(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pasta"))
}
