//! The transport-owning bridge threads: the socket bridge loop
//! ([`run_socket_bridge`]) that solely owns the [`Transport`] and multiplexes
//! inbound polls + outbound frame drains ([`drain_outbound`]), and the event
//! encoder thread ([`run_event_encoder`]). Split out of the `wiring` hub (C5
//! production split) — child of `wiring`, so it reaches the hub's
//! [`SharedAdapter`]/[`SourceMapWiring`] and the inbound/resolver siblings
//! through `super::`. Bodies are byte-identical to the flat `wiring.rs`;
//! [`run_socket_bridge`] is preserved exactly (requirement 4.4).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use serde_json::Value;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::kick::KickSink;
use crate::debug::transport::Transport;
use crate::debug::types::{SessionCommand, SessionEvent};

use super::inbound::handle_inbound;
use super::resolver::attach_pasta_resolver;
use super::{SharedAdapter, SourceMapWiring};

/// Inbound poll interval for the socket bridge. `std::sync::mpsc` has no
/// `select` and [`Transport`] is `!Sync`, so the single Transport-owner thread
/// polls inbound with this timeout and drains the outbound frame channel between
/// polls. Small enough to be imperceptible interactively; it blocks (does not
/// busy-spin) for the interval when idle.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

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
// The bridge is the sole Transport owner and the single multiplexing point, so it
// legitimately threads the full set of shared seams (adapter / breakpoints / the
// two channels / shutdown / source-map wiring / kick sink). Grouping them into a
// struct would only obscure the flat ownership transfer into the spawned thread.
#[allow(clippy::too_many_arguments)]
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
    // The (optional) host-injected scene-kick sink (pasta-scene-kick tasks 2.2 /
    // 2.3 / R2.4 / R2.6). Passed to `handle_inbound` so an inbound `pasta/playScene`
    // request invokes it. `None` keeps the kick path inert (R2.6).
    kick_sink: Option<KickSink>,
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
                if !handle_inbound(
                    &transport,
                    &adapter,
                    &breakpoints,
                    &cmd_tx,
                    &req,
                    &source_map,
                    kick_sink.as_ref(),
                ) {
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

/// Drain all currently-available outbound frames from `out_rx` and write them.
/// Returns `false` if a transport write failed (peer gone) so the caller stops;
/// a closed `out_rx` (encoder thread ended) is NOT a stop condition here — the
/// client may still send inbound commands.
pub(super) fn drain_outbound(transport: &Transport, out_rx: &Receiver<Value>) -> bool {
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
