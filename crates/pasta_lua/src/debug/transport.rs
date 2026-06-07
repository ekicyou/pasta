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

#![allow(dead_code)]

use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use serde_json::Value;

use crate::debug::DebugError;

/// The DAP header that carries the body byte length.
const CONTENT_LENGTH: &str = "Content-Length";

// ---------------------------------------------------------------------------
// Frame codec (Content-Length framing) — pure, Lua-free, unit-testable
// ---------------------------------------------------------------------------

/// Serialize `value` into a `Content-Length`-framed DAP wire frame and write it
/// to `out`.
///
/// The body is compact UTF-8 JSON; the header reports its **byte** length
/// (`buf.len()` of the UTF-8 encoding, NOT the char count), then a blank
/// `\r\n\r\n` separates the header block from the body. The whole frame is
/// flushed so the peer can read it immediately.
///
/// I/O only — never touches Lua.
pub(crate) fn write_frame<W: Write>(out: &mut W, value: &Value) -> io::Result<()> {
    // Compact JSON body. `to_vec` yields the exact UTF-8 bytes; the header MUST
    // use this byte length (multi-byte UTF-8 makes bytes != chars).
    let body = serde_json::to_vec(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(out, "{CONTENT_LENGTH}: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()
}

/// Read one `Content-Length`-framed DAP wire frame from `reader` and parse the
/// body into a [`serde_json::Value`].
///
/// Parsing is robust to header ordering and to extra headers: the header block
/// is read line by line until a blank line (the `\r\n\r\n` separator), and only
/// the `Content-Length` header is significant (its name is matched
/// case-insensitively, surrounding whitespace trimmed). Then EXACTLY that many
/// body bytes are read (no over- or under-read), decoded as UTF-8, and parsed.
///
/// Returns `Ok(None)` on a clean EOF *before* any header bytes (the peer closed
/// the connection between frames). Any malformed frame (missing
/// `Content-Length`, truncated body, non-UTF-8, invalid JSON) is an
/// [`io::Error`].
///
/// I/O only — never touches Lua.
pub(crate) fn read_frame<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    let mut saw_any_header_byte = false;

    // (1) Read the header block, line by line, until a blank line.
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            // EOF. If it landed exactly between frames (no header bytes read),
            // it's a clean close; otherwise the frame was truncated.
            if saw_any_header_byte {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF in the middle of a frame header block",
                ));
            }
            return Ok(None);
        }
        saw_any_header_byte = true;

        // The blank line (`\r\n` or `\n`) terminates the header block.
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        // Parse `Header-Name: value`; only Content-Length matters. Robust to
        // ordering and to additional headers (which are ignored).
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case(CONTENT_LENGTH) {
                let parsed = value.trim().parse::<usize>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid Content-Length value: {value:?}"),
                    )
                })?;
                content_length = Some(parsed);
            }
        }
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "frame header block missing Content-Length",
        )
    })?;

    // (2) Read EXACTLY `len` body bytes (no over/under-read).
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;

    // (3) Decode UTF-8 and parse JSON.
    let text = String::from_utf8(body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let value = serde_json::from_str::<Value>(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(value))
}

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
            });
        };

        // Enabled: bind (bind failure → DebugError::Bind, R3.1/R5.5).
        let listener = TcpListener::bind(addr).map_err(DebugError::Bind)?;
        let local_addr = listener.local_addr().map_err(DebugError::Bind)?;

        // Channels are the ONLY seam. The transport thread owns the socket ends;
        // the owner keeps the other ends.
        let (in_tx, in_rx) = std::sync::mpsc::channel::<Value>();
        let (out_tx, out_rx) = std::sync::mpsc::channel::<Value>();

        // Long-lived listener thread (socket I/O only — no Lua).
        let handle = std::thread::spawn(move || {
            serve(listener, in_tx, out_rx);
        });

        Ok(Self {
            inbound: in_rx,
            outbound: Some(out_tx),
            handle: Some(handle),
            local_addr: Some(local_addr),
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
    pub(crate) fn shutdown(&mut self) {
        // Dropping the outbound Sender closes the channel; the writer loop sees a
        // disconnect and returns. The reader loop returns on socket EOF/error.
        self.outbound = None;
    }

    /// Join the listener thread (used by tests / orderly teardown). No-op when
    /// disabled (no thread was spawned).
    pub(crate) fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        // Best-effort: drop the outbound sender to unblock the writer. We do NOT
        // block-join in Drop (the reader may be parked on a blocking socket read
        // in production, which has no timeout); tests join explicitly.
        self.outbound = None;
    }
}

/// The listener thread body (long-lived, socket I/O only — never touches Lua).
///
/// Accepts exactly one client (single-client by design), then runs the
/// socket↔channel bridge with two sub-threads:
/// - **reader**: parse `Content-Length` frames off the socket and forward each
///   as a [`serde_json::Value`] to `in_tx`. Returns on EOF / I/O error / when
///   `in_tx`'s receiver is gone.
/// - **writer**: drain `out_rx` and write each value to the socket as a frame.
///   Returns when `out_rx` disconnects (the [`Transport`]'s sender dropped) or
///   on a socket write error.
///
/// Mirrors the PoC's "safe return on error" so neither side hangs.
fn serve(listener: TcpListener, in_tx: Sender<Value>, out_rx: Receiver<Value>) {
    // Accept exactly ONE client (single-client per design).
    let stream = match listener.accept() {
        Ok((s, _peer)) => s,
        Err(_) => return,
    };
    // Read and write halves share the socket; clone so the reader thread owns one
    // half (BufReader) and the writer the other.
    let write_half = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };

    // Reader sub-thread: socket → in_tx. DETACHED: it self-terminates on EOF /
    // I/O error / when `in_tx`'s receiver is gone. We never block-join it (a
    // production reader has no timeout and may be parked on a blocking read), so
    // `serve` cannot hang on the reader. Dropping `in_tx` here closes the inbound
    // channel, which is the owner's "reader done" signal.
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        loop {
            match read_frame(&mut reader) {
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

    // Writer loop runs on THIS (listener) thread: out_rx → socket. It ends when
    // the outbound channel disconnects (the `Transport`'s sender dropped on
    // shutdown / drop) or on a socket write error.
    let mut writer = write_half;
    while let Ok(value) = out_rx.recv() {
        if write_frame(&mut writer, &value).is_err() {
            // Socket write failed (peer gone) → stop writing.
            break;
        }
    }

    // Outbound channel closed (shutdown) or write error: best-effort shutdown of
    // the socket so the detached reader observes EOF and self-terminates. We do
    // NOT join the reader (it is detached) — `serve` returns immediately so the
    // listener thread never hangs.
    let _ = writer.shutdown(std::net::Shutdown::Both);
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;
    use std::net::TcpStream;
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;

    use serde_json::json;

    /// TEST-ONLY watchdog / read timeout so CI cannot hang. The production
    /// transport has NO timeout baked in (design "スレッドモデル ④").
    const WATCHDOG: Duration = Duration::from_secs(10);

    /// Connect a test client to `addr`. The production peer is the external DAP
    /// client, so this loopback connector is test-only.
    fn connect_client(addr: SocketAddr) -> io::Result<TcpStream> {
        TcpStream::connect(addr)
    }

    // -----------------------------------------------------------------------
    // Frame codec unit tests (byte-length framing, header robustness, exactness)
    // -----------------------------------------------------------------------

    /// `write_frame` then `read_frame` round-trips an arbitrary JSON value.
    #[test]
    fn frame_round_trip_ascii() {
        let value = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": { "adapterID": "pasta" }
        });

        let mut buf: Vec<u8> = Vec::new();
        write_frame(&mut buf, &value).expect("write_frame must succeed");

        let mut reader = Cursor::new(buf);
        let read = read_frame(&mut reader)
            .expect("read_frame must succeed")
            .expect("a frame must be present");
        assert_eq!(read, value, "round-trip must preserve the JSON value");
    }

    /// The `Content-Length` header MUST be the UTF-8 BYTE length, not the char
    /// count. A multi-byte payload (Japanese) proves byte-vs-char correctness.
    #[test]
    fn content_length_is_byte_length_not_char_count() {
        // "こんにちは" is 5 chars but 15 UTF-8 bytes.
        let payload = "こんにちは";
        assert_eq!(payload.chars().count(), 5);
        assert_eq!(payload.len(), 15);

        let value = json!({ "text": payload });

        let mut buf: Vec<u8> = Vec::new();
        write_frame(&mut buf, &value).expect("write must succeed");

        // The emitted header must carry the BYTE length of the JSON body.
        let text = String::from_utf8(buf.clone()).expect("frame is UTF-8");
        let body = serde_json::to_vec(&value).unwrap();
        let expected_header = format!("Content-Length: {}\r\n\r\n", body.len());
        assert!(
            text.starts_with(&expected_header),
            "header must report the BYTE length ({}), got frame starting: {:?}",
            body.len(),
            &text[..expected_header.len().min(text.len())]
        );

        // And it must round-trip exactly (no over/under-read of the multi-byte body).
        let mut reader = Cursor::new(buf);
        let read = read_frame(&mut reader)
            .expect("read must succeed")
            .expect("a frame must be present");
        assert_eq!(read, value, "multi-byte body must round-trip intact");
        assert_eq!(read["text"], json!(payload));
    }

    /// `read_frame` is robust to extra and reordered headers; only
    /// `Content-Length` is significant, matched case-insensitively.
    #[test]
    fn read_frame_tolerates_extra_and_reordered_headers() {
        let body = br#"{"ok":true}"#;
        // Extra header BEFORE Content-Length, a different-cased name, and a
        // trailing extra header — all must be ignored except Content-Length.
        let mut frame: Vec<u8> = Vec::new();
        frame.extend_from_slice(b"X-Extra: hello\r\n");
        frame.extend_from_slice(format!("content-length: {}\r\n", body.len()).as_bytes());
        frame.extend_from_slice(b"X-Another: world\r\n");
        frame.extend_from_slice(b"\r\n");
        frame.extend_from_slice(body);

        let mut reader = Cursor::new(frame);
        let read = read_frame(&mut reader)
            .expect("read must succeed with extra/reordered headers")
            .expect("a frame must be present");
        assert_eq!(read, json!({ "ok": true }));
    }

    /// `read_frame` reads EXACTLY N body bytes and does not consume the start of
    /// a following frame (no over-read).
    #[test]
    fn read_frame_reads_exactly_n_bytes_and_leaves_the_next_frame() {
        let first = json!({ "a": 1 });
        let second = json!({ "b": 2 });

        let mut buf: Vec<u8> = Vec::new();
        write_frame(&mut buf, &first).unwrap();
        write_frame(&mut buf, &second).unwrap();

        let mut reader = Cursor::new(buf);
        let r1 = read_frame(&mut reader).unwrap().expect("first frame");
        assert_eq!(r1, first, "first frame parsed");
        let r2 = read_frame(&mut reader).unwrap().expect("second frame");
        assert_eq!(r2, second, "second frame intact (no over-read of the first)");
        // A third read hits clean EOF between frames.
        assert!(
            read_frame(&mut reader).unwrap().is_none(),
            "clean EOF between frames yields Ok(None)"
        );
    }

    /// A missing `Content-Length` header is a framing error (not silent).
    #[test]
    fn read_frame_missing_content_length_is_error() {
        let mut frame: Vec<u8> = Vec::new();
        frame.extend_from_slice(b"X-Only: nope\r\n\r\n");
        frame.extend_from_slice(br#"{"x":1}"#);
        let mut reader = Cursor::new(frame);
        let err = read_frame(&mut reader).expect_err("missing Content-Length must error");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    /// A truncated body (fewer than `Content-Length` bytes) is an error, not a
    /// short/partial parse.
    #[test]
    fn read_frame_truncated_body_is_error() {
        let mut frame: Vec<u8> = Vec::new();
        // Claim 20 bytes but provide far fewer.
        frame.extend_from_slice(b"Content-Length: 20\r\n\r\n");
        frame.extend_from_slice(br#"{"x":1}"#);
        let mut reader = Cursor::new(frame);
        let err = read_frame(&mut reader).expect_err("truncated body must error");
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    // -----------------------------------------------------------------------
    // Disabled path: listen == None opens NO port (R5.5)
    // -----------------------------------------------------------------------

    /// R5.5: `listen == None` opens nothing — no bind, no port, no address. A
    /// connection attempt to any would-be port cannot reach this transport
    /// because it never bound one.
    #[test]
    fn disabled_listen_none_opens_no_port() {
        let transport = Transport::start(None).expect("disabled start must succeed");

        // No bound address is exposed (nothing was opened).
        assert!(
            transport.local_addr().is_none(),
            "disabled transport must not bind a port (R5.5)"
        );

        // The inbound channel is already closed → the owner never blocks on a
        // connection that will never arrive.
        match transport.inbound().recv_timeout(Duration::from_millis(50)) {
            Err(RecvTimeoutError::Disconnected) => {}
            other => panic!("disabled inbound must be a closed channel, got {other:?}"),
        }

        // Sending is a clean Disconnected (no socket exists).
        assert!(
            matches!(transport.send(json!({"x":1})), Err(DebugError::Disconnected)),
            "disabled send must report Disconnected (no socket)"
        );
    }

    // -----------------------------------------------------------------------
    // Enabled path: bind 127.0.0.1:0, one client, framed round-trip both ways
    // -----------------------------------------------------------------------

    /// R3.1 + framing: bind an OS-assigned loopback port, connect a test client,
    /// and round-trip a Content-Length-framed JSON message BOTH directions:
    /// client → transport (delivered on `inbound`) and transport → client
    /// (received by the client as a correctly framed frame).
    #[test]
    fn enabled_round_trips_framed_json_both_directions() {
        let transport =
            Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
        let addr = transport
            .local_addr()
            .expect("enabled transport must expose its bound addr (R3.1)");
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_ne!(addr.port(), 0, "OS must assign a concrete port");

        // Test client connects (single-client accept).
        let client = connect_client(addr).expect("client connect must succeed");
        client
            .set_read_timeout(Some(WATCHDOG))
            .expect("TEST-ONLY read timeout");
        let client_write = client.try_clone().expect("clone client");
        let mut client_reader = BufReader::new(client);
        let mut client_writer = client_write;

        // (A) client → transport: send a framed request; transport delivers it
        // parsed on `inbound`.
        let request = json!({ "seq": 7, "command": "setBreakpoints" });
        write_frame(&mut client_writer, &request).expect("client write must succeed");

        let delivered = transport
            .inbound()
            .recv_timeout(WATCHDOG)
            .expect("transport must deliver the inbound frame (R3.1)");
        assert_eq!(delivered, request, "inbound JSON must match what the client sent");

        // (B) transport → client: push an outbound frame; the client reads it as
        // a correctly framed frame.
        let response = json!({ "seq": 7, "type": "response", "success": true });
        transport.send(response.clone()).expect("transport send must succeed");

        let received = read_frame(&mut client_reader)
            .expect("client read must succeed")
            .expect("client must receive a frame");
        assert_eq!(received, response, "outbound JSON must match what the transport sent");

        // Clean teardown: drop the client (peer EOF) and the transport, then
        // join the listener thread bounded by the watchdog.
        drop(client_reader);
        drop(client_writer);
        let mut transport = transport;
        transport.shutdown();
        join_transport_with_watchdog(transport, WATCHDOG);
    }

    /// The transport thread carries RAW frames only (no DAP semantics): an
    /// arbitrary, non-DAP JSON shape round-trips unchanged. Proves this layer is
    /// the byte/JSON wire boundary, not a DAP parser (task 3.2 owns semantics).
    #[test]
    fn transport_carries_raw_values_without_interpreting_dap() {
        let transport =
            Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
        let addr = transport.local_addr().unwrap();

        let client = connect_client(addr).expect("connect");
        client.set_read_timeout(Some(WATCHDOG)).unwrap();
        let mut client_writer = client.try_clone().unwrap();

        // A shape that is NOT a DAP message at all.
        let weird = json!([1, "two", { "three": [true, null] }, "日本語"]);
        write_frame(&mut client_writer, &weird).expect("client write");

        let delivered = transport
            .inbound()
            .recv_timeout(WATCHDOG)
            .expect("inbound frame delivered");
        assert_eq!(delivered, weird, "raw value must pass through uninterpreted");

        drop(client);
        let mut transport = transport;
        transport.shutdown();
        join_transport_with_watchdog(transport, WATCHDOG);
    }

    /// Clean shutdown: dropping the client (peer EOF) lets the listener thread
    /// wind down without hanging. The owner observes the inbound channel close.
    #[test]
    fn client_disconnect_winds_down_without_hang() {
        let transport =
            Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
        let addr = transport.local_addr().unwrap();

        let client = connect_client(addr).expect("connect");
        // Immediately drop the client → peer EOF.
        drop(client);

        // The inbound channel must close (reader returned on EOF) within the
        // watchdog — proving no hang.
        loop {
            match transport.inbound().recv_timeout(WATCHDOG) {
                Err(RecvTimeoutError::Disconnected) => break, // reader done
                Err(RecvTimeoutError::Timeout) => {
                    panic!("inbound did not close after client disconnect (hang?)")
                }
                Ok(v) => panic!("unexpected inbound frame after disconnect: {v:?}"),
            }
        }

        let mut transport = transport;
        transport.shutdown();
        join_transport_with_watchdog(transport, WATCHDOG);
    }

    // -----------------------------------------------------------------------
    // Test helpers (TEST-ONLY bounded join; production has no timeout)
    // -----------------------------------------------------------------------

    /// Join the transport's listener thread bounded by a TEST-ONLY watchdog so a
    /// regression that hangs the thread fails the test instead of the suite.
    fn join_transport_with_watchdog(mut transport: Transport, timeout: Duration) {
        let handle = transport.handle.take();
        // Drop the outbound sender so the writer unblocks.
        transport.outbound = None;
        if let Some(handle) = handle {
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = done_tx.send(handle.join());
            });
            done_rx
                .recv_timeout(timeout)
                .expect("listener thread must wind down within the watchdog (no hang)")
                .expect("listener thread must not panic");
        }
    }
}
