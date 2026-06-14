//! The debug backend handle: owns the bridge threads and drives teardown.
//!
//! Split out of `debug/mod.rs` (task 4.5): the [`DebugHandle`] struct, its
//! `Debug` impl, the `config` / `local_addr` accessors and the `Drop` teardown.
//! The parent `mod.rs` keeps `pub use handle::DebugHandle;` so the public surface
//! is byte-identical.
//!
//! A `pub(crate) fn new()` constructor is ADDED here so the sibling
//! [`enable`](super::enable) module (which is NOT a descendant and therefore
//! cannot build the struct literal with private fields) can construct a handle
//! while the fields stay private (design "Components / C3" seam).

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::JoinHandle;

use super::config::DebugConfig;
use super::types::SessionEvent;

/// Owner of the debug backend's bridge threads and shared state (task 4.1 full
/// wiring).
///
/// Constructed by [`enable`] when debugging is active. It holds:
/// - the bound listen address (read back from the transport so a caller using
///   port 0 can discover the OS-assigned port),
/// - a shared shutdown flag and the socket-bridge / event-encoder join handles.
///
/// The shared [`BreakpointSet`] is NOT held here: it is owned by the VM-thread
/// hook (reads) and the socket-bridge thread (writes — settable while running);
/// the handle needs no clone of it for task 4.1. (Runtime integration, task 4.2,
/// may surface it on the handle when it actually consumes it.)
///
/// The [`Transport`] itself is `!Sync` (it holds a `Receiver`), so it is owned
/// solely by the socket-bridge thread (see [`wiring`]); the handle never holds
/// it. The VM-thread line hook (installed by [`enable`] via
/// [`hook::install`](crate::debug::hook::install)) owns the [`DebugSession`] and
/// the session ends of the command/event channels; `mlua::Lua` never crosses a
/// thread (it is `!Send`).
///
/// # Teardown (synchronous port release, bounded)
///
/// [`Drop`] sets the shared shutdown flag and then SYNCHRONOUSLY JOINS the
/// socket-bridge thread (task 3.1): the bridge observes the flag within one
/// `POLL_INTERVAL`, returns, and drops its by-value [`Transport`], whose own
/// `Drop` joins the `serve()` listener thread — so the listening port is
/// RELEASED before this `Drop` returns. This makes a SHIORI unload free the
/// fixed DAP port deterministically before the next reload re-binds it (R1.x /
/// R2.x). The join is bounded because every downstream blocking point is an
/// interruptible `POLL_INTERVAL` poll, so teardown cannot hang. The
/// event-encoder thread (which owns no socket/port) is left DETACHED — joining
/// it while this `Drop` still holds `terminate_tx` would deadlock. The backend
/// also winds down naturally when the VM thread finishes Lua execution (the
/// session's channel ends drop, closing the encoder) or the DAP client
/// disconnects (the transport closes the inbound channel).
pub struct DebugHandle {
    /// Resolved configuration this handle was created from.
    config: DebugConfig,
    /// The bound listen address (read from the transport at construction), or
    /// `None` when no listener was opened.
    local_addr: Option<SocketAddr>,
    /// Shared shutdown flag: setting it makes the socket bridge stop and drop
    /// the transport (non-blocking teardown).
    shutdown: Arc<AtomicBool>,
    /// Socket-bridge thread join handle (sole `Transport` owner: reads + writes).
    socket_handle: Option<JoinHandle<()>>,
    /// Event-encoder thread join handle (session events → DAP frames).
    encoder_handle: Option<JoinHandle<()>>,
    /// A clone of the session's event sender, used SOLELY to emit a final
    /// [`SessionEvent::Terminated`] on teardown (task 4.2).
    ///
    /// In the long-lived SHIORI runtime there is no per-request "execution end":
    /// the debuggee is the runtime ITSELF, so a per-request `exec()` return must
    /// NOT terminate the session (R3.5's "execution end" maps to RUNTIME
    /// TEARDOWN, not request end). On `Drop` we send `Terminated` through this
    /// clone BEFORE signalling shutdown, so the event-encoder thread can encode a
    /// DAP `terminated` frame for the socket bridge to flush to any connected
    /// client (best-effort; the encoder/bridge channels then wind down). The
    /// existing disconnect→terminated path (the session's `Disconnect` handler)
    /// remains for the client-initiated case.
    terminate_tx: mpsc::Sender<SessionEvent>,
}

impl std::fmt::Debug for DebugHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugHandle")
            .field("config", &self.config)
            .field("local_addr", &self.local_addr)
            .finish_non_exhaustive()
    }
}

impl DebugHandle {
    /// Construct a fully wired [`DebugHandle`] from its parts.
    ///
    /// This `pub(crate)` constructor exists so the SIBLING [`enable`](super::enable)
    /// module can build the handle without seeing the private fields directly (a
    /// sibling cannot use the struct literal — only descendants can). The fields
    /// stay private; only this in-crate seam is added (design "Components / C3").
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        config: DebugConfig,
        local_addr: Option<SocketAddr>,
        shutdown: Arc<AtomicBool>,
        socket_handle: Option<JoinHandle<()>>,
        encoder_handle: Option<JoinHandle<()>>,
        terminate_tx: mpsc::Sender<SessionEvent>,
    ) -> Self {
        Self {
            config,
            local_addr,
            shutdown,
            socket_handle,
            encoder_handle,
            terminate_tx,
        }
    }

    /// The resolved [`DebugConfig`] this handle owns.
    pub fn config(&self) -> &DebugConfig {
        &self.config
    }

    /// The bound DAP listen address, or `None` when no listener is active
    /// (R3.1: the OS-assigned port is read back from the transport so a caller
    /// using port 0 can discover the concrete bound port).
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }
}

impl Drop for DebugHandle {
    fn drop(&mut self) {
        // (1) Natural-end `terminated` (task 4.2): emit a final `Terminated`
        // session event so the event-encoder thread encodes a DAP `terminated`
        // frame and the socket bridge flushes it to any connected client. In the
        // long-lived SHIORI runtime the "execution end" of R3.5 is RUNTIME
        // TEARDOWN (this Drop), NOT a per-request `exec()` return — the debuggee
        // is the runtime itself, so per-request returns deliberately do not
        // terminate the session. A send failure (encoder already gone) is ignored.
        let _ = self.terminate_tx.send(SessionEvent::Terminated);

        // (2) Give the encoder + socket bridge a brief, BOUNDED window to encode
        // and flush that frame before we tear the bridge down. The socket bridge
        // polls/drains every `wiring::POLL_INTERVAL` (5ms); a few intervals is
        // enough for the `Terminated` frame to traverse encoder → out channel →
        // socket while staying effectively non-blocking for teardown.
        std::thread::sleep(std::time::Duration::from_millis(30));

        // (3) Synchronous teardown (task 3.1, R1.1/R1.2/R1.3/R2.1/R2.2): signal
        // the socket bridge to stop FIRST (it observes the flag within one
        // `POLL_INTERVAL`, returns, and drops the `Transport`), THEN JOIN it. The
        // flag MUST be set before the join, otherwise the join would wait on a
        // bridge that was never told to stop. Joining the socket bridge waits for
        // `run_socket_bridge` to return → the by-value `Transport` is dropped →
        // `Transport::drop` synchronously joins its `serve()` listener thread
        // (tasks 2.1-2.4) → the listening port is RELEASED before this `drop`
        // returns. The join is bounded: every blocking point downstream is an
        // interruptible `POLL_INTERVAL` poll, so this never hangs (a hang would
        // wedge the test; production teardown is watchdog-free by design).
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(h) = self.socket_handle.take() {
            // Synchronous JOIN: blocks until the bridge returns → Transport drop →
            // serve join → port freed. A panicked bridge yields `Err`; ignore it,
            // teardown still completed (the thread is no longer running).
            let _ = h.join();
        }

        // The event-encoder owns no socket / port, so joining it is unnecessary
        // for releasing the port. Keep it DETACHED: this `Drop` still holds
        // `terminate_tx` (a `Sender` clone of the encoder's `event_rx`), so the
        // encoder cannot observe channel disconnect and exit until AFTER this
        // method returns and drops `terminate_tx`; joining it here would deadlock.
        let _ = self.encoder_handle.take();
    }
}
