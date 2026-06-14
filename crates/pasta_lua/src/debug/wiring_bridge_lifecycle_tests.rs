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
use super::*;

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
