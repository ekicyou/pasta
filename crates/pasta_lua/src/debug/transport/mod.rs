//! Transport: TCP listener + DAP-compliant Content-Length framing on an
//! I/O-ONLY thread (design "Transport & DapAdapter", requirements 3.1 / 5.5).
//!
//! # Role in the backend
//!
//! [`Transport`] is the wire layer of the debug backend. It owns a long-lived
//! listener thread (promoted from the PoC `transport_loop::serve`) that:
//!
//! 1. binds a [`TcpListener`] to the configured address (ONLY when debugging is
//!    enabled — `listen == None` opens nothing, R5.5),
//! 2. accepts exactly ONE client connection (single-client by design),
//! 3. bridges that socket to two `std::sync::mpsc` channels, reading framed JSON
//!    from the socket into an inbound channel and writing outbound JSON from a
//!    channel back to the socket.
//!
//! Unlike the PoC, which used a bare line protocol (`stopped <src> <line>\n`),
//! production uses **DAP-compliant `Content-Length` framing** (design
//! "Transport": "PoC の素朴な行プロトコルは DAP 準拠フレーミングへ作り直す").
//!
//! # I/O ONLY — never touches Lua (design "Transport"/"Architecture")
//!
//! The transport thread uses only `std::net` / `std::io` / `std::sync::mpsc` and
//! the existing `serde_json`. It MUST NOT touch `mlua::Lua` / Lua state: the
//! `mlua::Lua` handle is `!Send` and is pinned to the VM thread. The only seam
//! the transport exposes is a pair of channels carrying raw
//! [`serde_json::Value`] frames; DAP message SEMANTICS (initialize /
//! setBreakpoints / …) are owned by the DAP adapter (task 3.2), NOT here. This
//! layer is purely the byte/JSON wire boundary.
//!
//! # Wire frame: `Content-Length: <N>\r\n\r\n<json>` (byte length)
//!
//! A frame is a header block terminated by a blank line (`\r\n\r\n`) followed by
//! exactly `N` bytes of UTF-8 JSON body, where `N` is the **byte** length of the
//! body (NOT its char count — multi-byte UTF-8 such as Japanese makes the two
//! differ). Reads are robust to extra/reordered headers; only `Content-Length`
//! is significant. See [`read_frame`] / [`write_frame`].
//!
//! # Clean shutdown (no hang on EOF / disconnect)
//!
//! The listener thread returns safely on socket EOF, an I/O error, or channel
//! disconnect (the inbound `Sender` being dropped, or the outbound `Receiver`
//! being dropped) — mirroring the PoC's "safe return on error" so the thread
//! never hangs. [`Transport::shutdown`] drops the outbound sender and unblocks
//! the writer; the reader unblocks on the next socket EOF / error. Tests use a
//! TEST-ONLY `set_read_timeout` and bounded joins so CI cannot hang; the
//! production path has no timeout baked in.
//!
//! # Module layout (C3 directory module)
//!
//! This hub holds the [`Transport`] type, its impls, [`Drop`], and the listener
//! thread body [`serve`]. The frame codec (Content-Length framing) lives in the
//! [`framing`] submodule and is re-exported here so the wire-level seam reads as
//! a single `transport` surface (`read_frame` / `write_frame`).

mod framing;

use std::io::{self, BufRead, BufReader};
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::Value;
use socket2::{Domain, Protocol, Socket, Type};

use crate::debug::DebugError;

// Re-export the codec at the `transport` module path so the wire-level seam
// (`crate::debug::transport::{read_frame, write_frame}`) reads as a single
// surface — preserving the pre-split public path used by sibling debug modules
// and their externalized tests.
pub(crate) use framing::{read_frame, write_frame};
// Re-exported for the externalized codec tests (`transport_codec_tests.rs`),
// which reach module-scope items via `use super::*`; production code does not
// reference the cap directly (the codec enforces it internally), so the
// re-export is test-only to avoid an unused-import warning in the lib build.
#[cfg(test)]
pub(crate) use framing::MAX_CONTENT_LENGTH;

/// Poll cadence for the interruptible non-blocking `accept()` loop in
/// [`serve`]. Matches the established 5ms cooperative-poll convention used by
/// the socket bridge (`wiring::POLL_INTERVAL`): small enough to be
/// imperceptible at teardown while keeping the parked accept from busy-spinning
/// (it sleeps for this interval when no client is waiting). The shutdown flag is
/// checked once per interval, so a parked accept winds down within ~one
/// `POLL_INTERVAL` of [`Transport::shutdown`] / drop (design "State Management").
const POLL_INTERVAL: Duration = Duration::from_millis(5);

// ---------------------------------------------------------------------------
// Transport: bind + accept (single client) + socket<->channel bridge
// ---------------------------------------------------------------------------

/// The wire-layer transport: a long-lived listener thread bridging one TCP
/// client to inbound/outbound [`serde_json::Value`] channels.
///
/// Built by [`Transport::start`]. When `listen == None` the constructor opens
/// NOTHING (no bind, no port, no thread — R5.5) and the inbound channel yields
/// nothing. When `listen == Some(addr)` it binds, spawns the listener thread,
/// and accepts exactly one client.
///
/// The owner reads inbound frames from [`Transport::inbound`] and pushes
/// outbound frames via [`Transport::outbound`]. Dropping the [`Transport`] (or
/// calling [`shutdown`](Transport::shutdown)) drops the outbound sender, which
/// unblocks and ends the writer side of the bridge.
pub(crate) struct Transport {
    /// Inbound frames parsed off the socket (reader → owner). `None` when
    /// disabled (`listen == None`), so the owner observes an immediately-closed
    /// channel and never blocks.
    inbound: Receiver<Value>,
    /// Outbound frames to write to the socket (owner → writer). `None` when
    /// disabled; sending is a silent no-op (the channel is already closed).
    outbound: Option<Sender<Value>>,
    /// The listener thread join handle (long-lived). `None` when disabled (no
    /// thread was spawned).
    handle: Option<JoinHandle<()>>,
    /// The bound local address, when enabled. `None` when disabled (R5.5: no
    /// port is opened, so there is no address to report).
    local_addr: Option<SocketAddr>,
    /// Internal cooperative shutdown signal (R2.2/R2.3). A clone is moved into
    /// [`serve`], whose non-blocking accept poll loop checks it once per
    /// [`POLL_INTERVAL`]; setting it (via [`shutdown`](Transport::shutdown) or
    /// [`Drop`]) interrupts a parked accept so the listener thread can wind down
    /// and its port be released. This is the fix for the parked-listener
    /// port-leak that breaks unload→reload. The same flag also drives the
    /// CONNECTED-state writer poll in [`serve`] (interruptible `recv_timeout` +
    /// flag check), after which the reader sub-thread is joined — so a connected
    /// client's socket is released synchronously at teardown too (R2.5).
    shutdown: Arc<AtomicBool>,
}

impl Transport {
    /// Start the transport for `listen`.
    ///
    /// - `listen == None` → **opens nothing** (no bind, no port, no thread —
    ///   R5.5). Returns a disabled [`Transport`] whose `inbound` is an
    ///   already-closed channel and whose `outbound` is `None`. [`local_addr`]
    ///   is `None`. This is the zero-network-footprint disabled path.
    /// - `listen == Some(addr)` → binds a [`TcpListener`] (a bind failure maps
    ///   to [`DebugError::Bind`]), records the bound [`local_addr`], spawns the
    ///   long-lived listener thread, and accepts exactly one client.
    ///
    /// I/O only — never touches Lua.
    ///
    /// [`local_addr`]: Transport::local_addr
    pub(crate) fn start(listen: Option<SocketAddr>) -> Result<Self, DebugError> {
        let Some(addr) = listen else {
            // R5.5: disabled → open nothing. Hand back a Transport whose inbound
            // channel is already closed (the Sender is dropped here) so the owner
            // never blocks waiting on a port that will never exist.
            let (_dead_tx, inbound) = std::sync::mpsc::channel::<Value>();
            return Ok(Self {
                inbound,
                outbound: None,
                handle: None,
                local_addr: None,
                shutdown: Arc::new(AtomicBool::new(false)),
            });
        };

        // Enabled: build the listener via socket2 so SO_REUSEADDR is set BEFORE
        // bind (R3.1). std `TcpListener::bind` cannot set SO_REUSEADDR pre-bind,
        // so we drive the raw socket: SO_REUSEADDR → bind → listen → convert to a
        // std `TcpListener`, then set it NON-BLOCKING. Non-blocking is the
        // precondition for `serve()`'s interruptible accept poll loop: a parked
        // accept must yield `WouldBlock` so the loop can check the shutdown flag
        // and wind down (R2.2/R2.3). Any socket2 / nonblocking step failing maps
        // to `DebugError::Bind` (same as today's bind-failure path, R3.1).
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .map_err(DebugError::Bind)?;
        socket.set_reuse_address(true).map_err(DebugError::Bind)?; // SO_REUSEADDR (R3.1/R3.2)
        socket.bind(&addr.into()).map_err(DebugError::Bind)?;
        socket.listen(1).map_err(DebugError::Bind)?; // single-client design → tiny backlog
        let listener = TcpListener::from(socket);
        let local_addr = listener.local_addr().map_err(DebugError::Bind)?;
        // Interruptible accept (R2.2): the poll loop in `serve` relies on this.
        listener.set_nonblocking(true).map_err(DebugError::Bind)?;

        // Channels are the ONLY seam. The transport thread owns the socket ends;
        // the owner keeps the other ends.
        let (in_tx, in_rx) = std::sync::mpsc::channel::<Value>();
        let (out_tx, out_rx) = std::sync::mpsc::channel::<Value>();

        // Internal shutdown signal: a clone is moved into `serve` so the accept
        // poll loop can be interrupted; the `Transport` retains the original to
        // set on `shutdown()` / `Drop`.
        let shutdown = Arc::new(AtomicBool::new(false));
        let serve_shutdown = Arc::clone(&shutdown);

        // Long-lived listener thread (socket I/O only — no Lua).
        let handle = std::thread::spawn(move || {
            serve(listener, in_tx, out_rx, serve_shutdown);
        });

        Ok(Self {
            inbound: in_rx,
            outbound: Some(out_tx),
            handle: Some(handle),
            local_addr: Some(local_addr),
            shutdown,
        })
    }

    /// The bound local address, or `None` when disabled (R5.5: no port opened).
    pub(crate) fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Receiver of inbound frames parsed off the socket (reader → owner).
    ///
    /// When disabled this channel is already closed, so `recv()` returns `Err`
    /// immediately and the owner never blocks on a non-existent connection.
    pub(crate) fn inbound(&self) -> &Receiver<Value> {
        &self.inbound
    }

    /// Send `value` as an outbound frame (owner → writer → socket).
    ///
    /// Returns `Ok(())` if it was queued for the writer. When disabled, or after
    /// the writer has gone (peer disconnected / shut down), returns
    /// [`DebugError::Disconnected`] so the owner can stop the session cleanly.
    pub(crate) fn send(&self, value: Value) -> Result<(), DebugError> {
        match &self.outbound {
            Some(tx) => tx.send(value).map_err(|_| DebugError::Disconnected),
            None => Err(DebugError::Disconnected),
        }
    }

    /// Signal shutdown: drop the outbound sender so the writer side unblocks and
    /// the listener thread can wind down. Idempotent; the reader side completes
    /// on the next socket EOF / error.
    ///
    /// Production teardown goes through [`Drop`] (same effect); this explicit
    /// form is exercised by the `#[cfg(test)]` teardown paths.
    #[allow(dead_code)] // test-facing; production uses Drop (kept per design seam)
    pub(crate) fn shutdown(&mut self) {
        // Set the internal shutdown flag so a parked non-blocking accept poll
        // loop in `serve` observes it within one POLL_INTERVAL and returns even
        // when NO client ever connects (R2.2/R2.3). `Release` pairs with the
        // loop's `Acquire` load.
        self.shutdown.store(true, Ordering::Release);
        // Dropping the outbound Sender closes the channel; the writer loop sees a
        // disconnect and returns. The reader loop returns on socket EOF/error.
        self.outbound = None;
    }

    /// Join the listener thread (used by tests / orderly teardown). No-op when
    /// disabled (no thread was spawned).
    #[allow(dead_code)] // test-facing bounded-teardown helper (used by #[cfg(test)])
    pub(crate) fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// TEST-ONLY: raise the internal cooperative shutdown flag WITHOUT dropping
    /// the outbound sender.
    ///
    /// Production teardown ([`shutdown`](Transport::shutdown) / [`Drop`]) always
    /// raises the flag AND drops the outbound sender together, so either signal
    /// alone is enough to stop the writer. This helper isolates the FLAG signal
    /// so a test can prove the connected writer loop breaks on the flag even
    /// while the outbound channel is still open (R2.5) — the property that the
    /// pre-2.3 `while out_rx.recv()` loop did not have.
    #[cfg(test)]
    fn signal_shutdown_flag_only(&self) {
        self.shutdown.store(true, Ordering::Release);
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        // Synchronous teardown (R2.1/R2.2/R2.4/R2.5): set the internal shutdown
        // flag FIRST so a `serve` parked in the interruptible accept poll loop —
        // or in the connected-state writer poll — observes it within one
        // POLL_INTERVAL, then drop the outbound sender to unblock the writer.
        // ORDER MATTERS: the flag must be set BEFORE the join, otherwise the join
        // would wait on a `serve()` that has not yet been told to stop.
        self.shutdown.store(true, Ordering::Release);
        self.outbound = None;

        // Block-JOIN the serve listener thread so the bound port is released
        // BEFORE drop returns (no detached-listener port leak — the root cause of
        // the unload→reload 10048; design "State Management" invariant: after
        // teardown the handle is joined, not detached). This join is BOUNDED, not
        // a hang risk: every blocking point inside `serve()` is an interruptible
        // POLL_INTERVAL poll (accept poll, connected writer poll, and the reader
        // sub-thread's own flag poll), so `serve()` winds down within ~one
        // POLL_INTERVAL of the flag being set above. If the handle was already
        // taken (e.g. by the test-only `join()` / watchdog helper, or a prior
        // `shutdown()`+`join()`), this is a no-op — no double-join.
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// The listener thread body (long-lived, socket I/O only — never touches Lua).
///
/// Accepts exactly one client (single-client by design), then runs the
/// socket↔channel bridge with two sub-threads:
/// - **reader** (sub-thread, `JoinHandle` kept): parse `Content-Length` frames
///   off the socket and forward each as a [`serde_json::Value`] to `in_tx`.
///   Returns on EOF / I/O error / when `in_tx`'s receiver is gone, OR when the
///   `shutdown` flag is observed between frames. To make the flag interruption
///   reliable cross-platform the reader's socket carries a [`POLL_INTERVAL`]
///   read timeout: it polls for inbound data at each frame boundary (a timeout
///   yields no data → re-check the flag), then parses a full frame with blocking
///   reads once data is present (so framing is never split by the timeout).
/// - **writer** (this thread): an interruptible poll of `out_rx` — each
///   iteration checks the `shutdown` flag, then `recv_timeout(POLL_INTERVAL)`;
///   it writes each value as a frame and returns when the flag is set, the
///   outbound channel disconnects (the [`Transport`]'s sender dropped), or a
///   socket write fails.
///
/// At teardown the writer breaks (flag OR disconnect), then `shutdown(Both)` is
/// called and the reader is JOINED before `serve` returns — so a connected
/// client's socket is released synchronously (R2.5). The reader join is bounded
/// because the reader winds down within one `POLL_INTERVAL` of the flag even
/// when the peer keeps the connection open (a local socket `shutdown` does NOT
/// reliably cancel an in-flight blocking recv on Windows, so the flag poll — not
/// the `shutdown(Both)` EOF — is the load-bearing interrupt for the connected
/// path; design "Transport" Risks). Mirrors the PoC's "safe return on error" so
/// neither side hangs.
///
/// The listener is NON-BLOCKING: `accept()` is polled on a [`POLL_INTERVAL`]
/// cadence so the `shutdown` flag can interrupt a parked accept (no client
/// connected). Once a client connects the listener is dropped immediately
/// (single-client design → no further accepts, earliest possible port release).
fn serve(
    listener: TcpListener,
    in_tx: Sender<Value>,
    out_rx: Receiver<Value>,
    shutdown: Arc<AtomicBool>,
) {
    // Interruptible accept poll loop (R2.2/R2.3): the listener is non-blocking,
    // so a parked accept yields `WouldBlock`; we sleep one POLL_INTERVAL and
    // re-check the shutdown flag. This is what lets a no-client teardown wind the
    // listener thread down (and release the port) instead of leaking it.
    let stream = loop {
        // Shutdown requested while waiting for a client → drop the listener (the
        // `match` binding `listener` is moved out at the end of `serve`; an early
        // return drops it here) and return with no client.
        if shutdown.load(Ordering::Acquire) {
            return;
        }
        match listener.accept() {
            // Got the single client → stop accepting. Drop the listener NOW so
            // the port is released at the earliest point (single-client design).
            Ok((s, _peer)) => {
                // The accepted stream inherits the listener's non-blocking flag
                // (Windows/BSD). The CONNECTED-state bridge below expects a
                // BLOCKING stream (the reader parks on a blocking read, the
                // writer blocks on write); restore blocking so a transient
                // `WouldBlock` is never mistaken for an error/EOF that would
                // abort the freshly accepted connection. (Interruptible poll for
                // the connected bridge is a later task; this keeps the existing
                // blocking bridge unchanged in behavior.)
                if s.set_nonblocking(false).is_err() {
                    return;
                }
                drop(listener);
                break s;
            }
            // No client yet → sleep one interval and re-check the flag.
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(POLL_INTERVAL);
                continue;
            }
            // Any other accept error → safe return (no hang), as today.
            Err(_) => return,
        }
    };
    // Read and write halves share the socket; clone so the reader thread owns one
    // half (BufReader) and the writer the other.
    let write_half = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };

    // Give the reader's socket a POLL_INTERVAL read timeout so its frame-boundary
    // poll can observe the shutdown flag within one interval even while the peer
    // keeps the connection open. A local `shutdown(Both)` does NOT reliably cancel
    // an in-flight blocking recv on Windows, so this cooperative poll — mirroring
    // the writer's `recv_timeout(POLL_INTERVAL)` — is what makes the reader join
    // bounded at teardown (R2.5). If setting the timeout fails we cannot guarantee
    // a bounded reader join, so return without spawning a reader we could not
    // interrupt (safe: no client bridge, serve winds down).
    if stream.set_read_timeout(Some(POLL_INTERVAL)).is_err() {
        return;
    }

    // Reader sub-thread: socket → in_tx. Its `JoinHandle` is KEPT (not detached)
    // and joined after the writer loop ends, so the accepted connection is
    // released SYNCHRONOUSLY before `serve` returns (R2.5). The reader returns on
    // EOF / I/O error / when `in_tx`'s receiver is gone, or when the `shutdown`
    // flag is observed between frames. Dropping `in_tx` here (when the reader
    // returns) closes the inbound channel — the owner's "reader done" signal.
    let reader_shutdown = Arc::clone(&shutdown);
    let reader_handle = std::thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        loop {
            // (1) Teardown requested → stop between frames (bounded by one
            // POLL_INTERVAL of the read timeout below).
            if reader_shutdown.load(Ordering::Acquire) {
                return;
            }
            // (2) Frame-boundary poll: wait for inbound data with the read
            // timeout. No data this interval (Timeout/WouldBlock) → re-check the
            // flag. Clean EOF (empty fill) → peer closed → done. Other error →
            // safe return.
            match reader.fill_buf() {
                Ok([]) => return, // EOF between frames → peer closed
                Ok(_) => {}       // data buffered → a full frame can be parsed
                Err(e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(_) => return,
            }
            // (3) Data is present at a frame boundary → parse ONE full frame with
            // BLOCKING reads. The read timeout is cleared for the duration of the
            // parse so a frame split across TCP segments is never mis-read as a
            // truncation error, then restored for the next boundary poll. (The
            // flag is only polled BETWEEN frames; a frame, once started, is read to
            // completion — DAP frames are tiny, so this window is negligible.)
            if reader.get_ref().set_read_timeout(None).is_err() {
                return;
            }
            let parsed = read_frame(&mut reader);
            if reader.get_ref().set_read_timeout(Some(POLL_INTERVAL)).is_err() {
                return;
            }
            match parsed {
                Ok(Some(value)) => {
                    // Owner gone → stop reading (clean shutdown).
                    if in_tx.send(value).is_err() {
                        return;
                    }
                }
                // Clean EOF between frames → peer closed → done.
                Ok(None) => return,
                // Malformed frame / I/O error → safe return (no hang).
                Err(_) => return,
            }
        }
    });

    // Connected-state writer loop runs on THIS (listener) thread: out_rx →
    // socket. It is an INTERRUPTIBLE poll (R2.5) so teardown breaks it on EITHER
    // the internal `shutdown` flag OR the outbound sender being dropped:
    // - flag set (e.g. `shutdown()`/`Drop`, or the flag alone) → FLUSH any
    //   already-queued frames, then break (the outbound owner — the socket bridge
    //   — relies on pending frames being flushed before teardown);
    // - `recv_timeout` `Ok(frame)` → write it (stop on a socket write error);
    // - `Timeout` → re-check the flag (this is the `POLL_INTERVAL` cadence);
    // - `Disconnected` (outbound sender dropped) → drain the rest, then done.
    let mut writer = write_half;
    'writer: loop {
        if shutdown.load(Ordering::Acquire) {
            // Teardown via the flag: the outbound sender may still be alive with
            // frames already queued (e.g. the bridge enqueued a final flush then
            // dropped the Transport, racing the flag). Drain whatever is currently
            // buffered so those frames still reach the peer, then break.
            while let Ok(value) = out_rx.try_recv() {
                if write_frame(&mut writer, &value).is_err() {
                    break;
                }
            }
            break;
        }
        match out_rx.recv_timeout(POLL_INTERVAL) {
            Ok(value) => {
                if write_frame(&mut writer, &value).is_err() {
                    // Socket write failed (peer gone) → stop writing.
                    break 'writer;
                }
            }
            // No outbound frame this interval → loop and re-check the flag.
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            // Outbound sender dropped (shutdown / drop) → `recv_timeout` has
            // already yielded every queued frame above, so nothing remains; done.
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break 'writer,
        }
    }

    // Writer loop ended (flag, disconnect, or write error). Shut the socket down
    // (best-effort EOF for the reader on the peer-alive write path), THEN join the
    // reader so the accepted connection is released synchronously before `serve`
    // returns (R2.5). The join is bounded because the reader also winds down within
    // one POLL_INTERVAL via its own `shutdown` flag poll — on Windows a local
    // `shutdown(Both)` does NOT reliably cancel an in-flight blocking recv, so the
    // flag poll (not this shutdown) is the load-bearing interrupt. `serve` thus
    // never hangs on the reader.
    let _ = writer.shutdown(std::net::Shutdown::Both);
    let _ = reader_handle.join();
}

// ===========================================================================
// インラインテストの外出し（task 2.4・C1）。元の単一 `mod tests`（~740行）を
// 凝集境界で 2 つの FLAT 兄弟テストファイルへ分割する（各 < 600 行）:
//   - frame コーデック単体（純粋・Lua非依存）→ transport_codec_tests.rs
//   - トランスポート・ライフサイクル（bind/accept/teardown/再bind）→ transport_lifecycle_tests.rs
// 各兄弟は先頭に `use super::*;` を持つ。クラスタ間で共有するヘルパーは無いため
// `_test_support.rs` は設けず、各クラスタが必要なヘルパーを自クラスタ内に保持する。
// テストファイルは `debug/` 直下に留まるため、この `transport/` ハブからは `../` で参照する。
// ===========================================================================

/// frame コーデック単体クラスタ（`write_frame`/`read_frame` の純粋単体仕様）。
#[cfg(test)]
#[path = "../transport_codec_tests.rs"]
mod transport_codec_tests;

/// トランスポート・ライフサイクルクラスタ（bind/accept・両方向往復・teardown・再bind）。
#[cfg(test)]
#[path = "../transport_lifecycle_tests.rs"]
mod transport_lifecycle_tests;
