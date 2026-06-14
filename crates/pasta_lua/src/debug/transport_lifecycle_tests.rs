//! `transport` モジュールの **トランスポート・ライフサイクル**（無効パス R5.5・
//! bind/accept・両方向フレーム往復・no-client/connected teardown・同期 join・
//! 再 bind 反復）インラインテスト外出し（task 2.4・C1）。
//!
//! 移動のみ（振る舞い不変）。元の単一 `mod tests`（~740行）を凝集境界で 2 兄弟へ分割した
//! うちのライフサイクルクラスタ。TEST-ONLY な watchdog 付き join ヘルパーと loopback
//! クライアントコネクタを同梱する（このクラスタ専用・他クラスタ非依存）。
//! `use super::*;` で本番の private / `pub(crate)` 項目へ到達する（本番可視性は不変）。

use super::*;

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

/// `join` on a disabled transport is a no-op (no thread was ever spawned)
/// and `shutdown` stays idempotent — neither hangs nor panics.
#[test]
fn disabled_join_and_shutdown_are_noops() {
    let mut transport = Transport::start(None).expect("disabled start must succeed");
    transport.join(); // no thread → returns immediately
    transport.shutdown();
    transport.shutdown(); // idempotent
    transport.join(); // still a no-op after shutdown
    assert!(transport.local_addr().is_none());
}

/// After `shutdown` the outbound sender is gone: `send` on an ENABLED
/// transport reports `Disconnected` (the owner's clean stop signal), and
/// `shutdown` is idempotent.
#[test]
fn send_after_shutdown_reports_disconnected() {
    let mut transport =
        Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
    assert!(transport.local_addr().is_some(), "enabled transport binds");

    transport.shutdown();
    assert!(
        matches!(transport.send(json!({"x":1})), Err(DebugError::Disconnected)),
        "send after shutdown must report Disconnected"
    );
    transport.shutdown(); // idempotent

    // Unblock the listener (it is parked in accept()) so the bounded join
    // can complete: connect-and-drop makes accept return, the reader sees
    // EOF, and the writer sees the already-closed outbound channel.
    let _ = connect_client(transport.local_addr().unwrap());
    join_transport_with_watchdog(transport, WATCHDOG);
}

/// R3.1: the enabled path is built via the `socket2` (SO_REUSEADDR) chain
/// and still binds an OS-assigned loopback port, exposing a concrete
/// `local_addr`. (SO_REUSEADDR cannot be read back via getsockopt once the
/// socket is converted to `TcpListener`; the rebind/reuse behavior is
/// verified by later tasks. This pins that the socket2 bind path works.)
#[test]
fn enabled_socket2_path_binds_and_exposes_local_addr() {
    let mut transport =
        Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("socket2 bind must succeed");
    let addr = transport
        .local_addr()
        .expect("socket2-built enabled transport must expose its bound addr (R3.1)");
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_ne!(addr.port(), 0, "OS must assign a concrete port via the socket2 listener");

    // Unblock the listener (parked in accept()) so the bounded join completes.
    transport.shutdown();
    let _ = connect_client(addr);
    join_transport_with_watchdog(transport, WATCHDOG);
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
    match transport.inbound().recv_timeout(WATCHDOG) {
        Err(RecvTimeoutError::Disconnected) => {} // reader done
        Err(RecvTimeoutError::Timeout) => {
            panic!("inbound did not close after client disconnect (hang?)")
        }
        Ok(v) => panic!("unexpected inbound frame after disconnect: {v:?}"),
    }

    let mut transport = transport;
    transport.shutdown();
    join_transport_with_watchdog(transport, WATCHDOG);
}

/// R2.2/R2.3: with NO client connecting, the listener thread is parked in
/// `accept()`. `shutdown()` must set the internal shutdown flag so the
/// non-blocking accept poll loop observes it and `serve()` returns, letting
/// the listener handle join within the bounded watchdog — proving a parked
/// accept is interruptible (the root-cause of the port-leak reload bug).
///
/// On the pre-2.2 code (blocking accept, flag not consulted) this hangs and
/// the watchdog join fails.
#[test]
fn no_client_shutdown_unblocks_serve() {
    let mut transport =
        Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
    assert!(transport.local_addr().is_some(), "enabled transport binds");

    // No client ever connects → serve() is parked in the accept poll loop.
    // Setting the shutdown flag must wind it down.
    transport.shutdown();
    join_transport_with_watchdog(transport, WATCHDOG);
}

/// R2.5: with a client CONNECTED, the connected-state writer loop must honor
/// the internal `shutdown` FLAG — not only an outbound-sender drop. Here we
/// raise the flag WHILE keeping the outbound sender alive (and the inbound
/// receiver held so the reader is not torn down by an owner-gone signal),
/// then join the listener handle under the watchdog.
///
/// On the pre-2.3 connected writer loop (`while let Ok(_) = out_rx.recv()`,
/// flag not consulted, outbound still alive) the writer never returns, the
/// reader is detached, and `serve()` hangs → the watchdog join FAILS. After
/// 2.3 the writer breaks on the flag, `writer.shutdown(Both)` EOFs the reader,
/// the reader is joined, and `serve()` returns within the watchdog.
#[test]
fn connected_writer_honors_shutdown_flag_while_outbound_alive() {
    let mut transport =
        Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
    let addr = transport.local_addr().expect("enabled transport binds");

    // Connect so serve() leaves the accept loop and enters the connected-state
    // bridge (reader sub-thread + writer loop). Keep the client alive so the
    // reader's blocking read does NOT see a peer EOF on its own — teardown
    // must come from the flag → writer.shutdown(Both), not from the client.
    let client = connect_client(addr).expect("client connect must succeed");
    client
        .set_read_timeout(Some(WATCHDOG))
        .expect("TEST-ONLY read timeout");

    // Give serve() a moment to accept and spin up the bridge before signaling.
    // (Bounded; the watchdog join below is the real timeout guard.)
    std::thread::sleep(Duration::from_millis(50));

    // Raise the FLAG ONLY — outbound sender stays alive. This is the signal a
    // pre-2.3 `while out_rx.recv()` writer loop ignores.
    transport.signal_shutdown_flag_only();

    // Join the listener handle under the watchdog WITHOUT dropping the outbound
    // sender (the whole point is that the flag alone tears the writer down).
    let handle = transport.handle.take().expect("enabled transport has a handle");
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(handle.join());
    });
    done_rx
        .recv_timeout(WATCHDOG)
        .expect("connected writer must break on the shutdown FLAG and serve() must return (R2.5)")
        .expect("listener thread must not panic");

    // Keep the client alive until after the join so its EOF could not have been
    // the thing that woke the reader.
    drop(client);
}

/// R2.5 (normal path): with a client still CONNECTED, `shutdown()` (which
/// drops the outbound sender) must tear the connected bridge down
/// synchronously — the writer breaks on the outbound disconnect and the
/// reader is joined — so `serve()` returns within the watchdog even though
/// the peer never closed its end. This is the everyday unload-while-connected
/// case (R2.5); the flag-only test above isolates the OTHER teardown trigger.
#[test]
fn connected_shutdown_tears_down_synchronously_with_client_alive() {
    let transport =
        Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
    let addr = transport.local_addr().expect("enabled transport binds");

    // Connect and round-trip one frame each way so we are definitely in the
    // connected-state bridge, then leave the client ALIVE.
    let client = connect_client(addr).expect("client connect must succeed");
    client
        .set_read_timeout(Some(WATCHDOG))
        .expect("TEST-ONLY read timeout");
    let mut client_writer = client.try_clone().expect("clone client");
    let mut client_reader = BufReader::new(client.try_clone().expect("clone client"));

    write_frame(&mut client_writer, &json!({ "seq": 1, "command": "ping" }))
        .expect("client write");
    let delivered = transport
        .inbound()
        .recv_timeout(WATCHDOG)
        .expect("inbound frame delivered");
    assert_eq!(delivered["command"], json!("ping"));

    transport.send(json!({ "seq": 1, "type": "response" })).expect("send");
    let received = read_frame(&mut client_reader)
        .expect("client read")
        .expect("client frame");
    assert_eq!(received["type"], json!("response"));

    // Teardown via shutdown() (drops outbound) with the client STILL ALIVE.
    // The writer must break on the outbound disconnect and the reader must be
    // joined → serve returns within the watchdog.
    let mut transport = transport;
    transport.shutdown();
    let handle = transport.handle.take().expect("enabled transport has a handle");
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(handle.join());
    });
    done_rx
        .recv_timeout(WATCHDOG)
        .expect("connected shutdown() must tear down synchronously (R2.5)")
        .expect("listener thread must not panic");

    // Client kept alive until after the join, proving teardown did not depend
    // on a peer EOF.
    drop(client_writer);
    drop(client_reader);
    drop(client);
}

/// R2.4 (teardown 後の再 start 可能性, masking-aware): dropping a
/// no-client `Transport` must SYNCHRONOUSLY join the serve listener thread
/// so the bound port is released BEFORE `drop` returns. We prove this with a
/// PLAIN `TcpListener::bind` (NO socket2, NO `SO_REUSEADDR`) on the very same
/// port immediately after the drop: a plain rebind has no Windows
/// `SO_REUSEADDR` hijack semantics, so it can ONLY succeed if the previous
/// listener was truly gone (design Security Considerations — `SO_REUSEADDR`
/// must not be allowed to MASK a still-held leaked listener).
///
/// On the pre-2.4 code (`Drop` sets the flag and drops outbound but does NOT
/// join the serve handle) `drop` returns while `serve()` may still hold the
/// listener, so the immediate plain rebind fails with `AddrInUse`
/// (`os error 10048`). After 2.4 `Drop` block-joins the interruptible
/// `serve()`, so the rebind deterministically succeeds.
#[test]
fn drop_synchronously_frees_port_for_plain_rebind() {
    use std::net::TcpListener as PlainTcpListener;

    // Start enabled with an OS-assigned loopback port; capture the port P.
    let transport =
        Transport::start(Some("127.0.0.1:0".parse().unwrap())).expect("bind must succeed");
    let addr = transport
        .local_addr()
        .expect("enabled transport must expose its bound addr");
    let port = addr.port();
    assert_ne!(port, 0, "OS must assign a concrete port");

    // No client connects → serve() is parked in the interruptible accept
    // poll loop, holding the listener on port P. Drop the Transport DIRECTLY
    // (not via the watchdog helper that pre-takes/joins the handle): Drop
    // must set the flag AND synchronously join serve() before returning.
    drop(transport);

    // Immediately rebind the SAME port with a PLAIN listener — no socket2, no
    // SO_REUSEADDR. This must SUCCEED, proving the serve listener was joined
    // and the port freed before drop() returned. A plain bind deliberately
    // avoids the Windows SO_REUSEADDR hijack that could otherwise mask a
    // still-held listener (design Security Considerations / R2.4).
    let rebind = PlainTcpListener::bind(("127.0.0.1", port));
    assert!(
        rebind.is_ok(),
        "plain rebind of port {port} immediately after drop must succeed \
         (synchronous serve join freed the port); got: {:?}",
        rebind.err()
    );
}

/// R1.3/R2.4 (robustness, Transport unit level): a REPEATED teardown→rebind
/// cycle on the SAME fixed port must succeed every time. The production reload
/// bug is a SECOND (and Nth) bind onto the SAME port failing because the prior
/// listener thread was detached and still held it; a single rebind
/// (`drop_synchronously_frees_port_for_plain_rebind`) proves one cycle, but the
/// reload contract (R1.3: "最低でも約15秒間隔で2回以上") is about MULTIPLE
/// cycles holding deterministically. This drives the no-client teardown path
/// through ~3 `Transport::start(Some(P)) → drop` cycles on one OS-assigned port
/// and asserts each successive `start` on the SAME port succeeds.
///
/// Mechanism under test: the synchronous Drop join (R2.4) makes each rebind
/// deterministic — every cycle's listener thread is joined (port released)
/// BEFORE `drop` returns, so the next `start` cannot race a still-held port.
/// On detached/non-joining teardown the 2nd `start` would fail with `AddrInUse`
/// (`os error 10048`); here `start` returning `DebugError::Bind` is the failure
/// surface. Each cycle is bounded by the WATCHDOG so a hung teardown fails fast
/// instead of stalling CI. We use a `socket2`-built `Transport::start` rebind
/// (not a plain bind) deliberately — this exercises the REAL production rebind
/// path repeatedly; the masking-aware plain-bind detector is the separate
/// `drop_synchronously_frees_port_for_plain_rebind` test.
#[test]
fn repeated_teardown_rebind_same_port_succeeds() {
    // Capture an OS-assigned loopback port P via a throwaway transport, then
    // drop it (synchronous join frees P) so we can reuse the SAME fixed port.
    let port = {
        let throwaway = Transport::start(Some("127.0.0.1:0".parse().unwrap()))
            .expect("throwaway bind must succeed");
        let p = throwaway
            .local_addr()
            .expect("enabled transport must expose its bound addr")
            .port();
        assert_ne!(p, 0, "OS must assign a concrete port");
        drop(throwaway); // synchronous join releases P before the next start
        p
    };
    let fixed: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    // ~3 cycles of start(SAME port) → no client → bounded teardown. Each start
    // must succeed; a leaked/held listener from the previous cycle would make
    // the next start fail with DebugError::Bind (AddrInUse / 10048).
    for cycle in 0..3 {
        let mut transport = Transport::start(Some(fixed)).unwrap_or_else(|e| {
            panic!(
                "cycle {cycle}: rebind of fixed port {port} must succeed \
                 (prior cycle's listener was synchronously joined); got {e:?}"
            )
        });
        assert_eq!(
            transport.local_addr().map(|a| a.port()),
            Some(port),
            "cycle {cycle}: must rebind the SAME fixed port",
        );

        // No client connects → serve() is parked in the interruptible accept
        // poll loop. Tear down via the bounded watchdog join so a regression
        // that fails to wind the listener down fails fast (no CI stall) and the
        // port is provably released before the next cycle's start.
        let handle = transport.handle.take();
        transport.shutdown.store(true, Ordering::Release);
        transport.outbound = None;
        if let Some(handle) = handle {
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = done_tx.send(handle.join());
            });
            done_rx
                .recv_timeout(WATCHDOG)
                .unwrap_or_else(|_| panic!(
                    "cycle {cycle}: listener must wind down within the watchdog \
                     (no hang) so the port is freed for the next rebind"
                ))
                .unwrap_or_else(|_| panic!("cycle {cycle}: listener thread must not panic"));
        }
        // `transport` is dropped here; the handle was already taken, so Drop's
        // join is a no-op (no double-join) — the port is already released.
    }
}

/// Join the transport's listener thread bounded by a TEST-ONLY watchdog so a
/// regression that hangs the thread fails the test instead of the suite.
fn join_transport_with_watchdog(mut transport: Transport, timeout: Duration) {
    let handle = transport.handle.take();
    // Set the internal shutdown flag so a `serve` parked in the accept poll
    // loop (no client connected) winds down, and drop the outbound sender so
    // the writer unblocks.
    transport.shutdown.store(true, Ordering::Release);
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
