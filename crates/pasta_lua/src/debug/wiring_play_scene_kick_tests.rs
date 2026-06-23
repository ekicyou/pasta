//! Tasks 2.2 + 2.3 — the scene-kick WIRING in [`handle_inbound`]
//! (pasta-scene-kick requirements 2.3 / 2.4 / 2.5 / 2.6).
//!
//! A `pasta/playScene` custom request is a self-contained handler (same shape
//! as `try_source_presentation_toggle`, NOT generic routing): when a `KickSink`
//! is wired AND the decoded `kick_scene` is `Some(name)`, the handler invokes
//! the sink with `KickRequest { scene: name }` exactly once and sends a success
//! ack. An empty/invalid scene name (`None`) sends an error response and never
//! calls the sink (R2.5). With no sink wired (`None`), the kick path is inert —
//! the request is recognised but no sink call and no ack are produced (R2.6).
//!
//! These tests drive [`handle_inbound`] DIRECTLY over a real loopback
//! [`Transport`] (a connected client reads the response frames) and observe the
//! sink via a recording mock, so the exact wire frames AND the sink calls are
//! both asserted.

use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::DapAdapter;
use crate::debug::kick::{KickRequest, KickSink};
use crate::debug::transport::{Transport, read_frame};
use crate::debug::types::SessionCommand;

use super::{SharedAdapter, SourceMapWiring, handle_inbound};

/// TEST-ONLY watchdog so a wiring test cannot hang on a frame read.
const WATCHDOG: Duration = Duration::from_secs(10);

/// A real loopback [`Transport`] plus a connected client end (mirrors the
/// source-presentation toggle harness).
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

    /// True if the client has no immediately-available frame (a brief grace
    /// read window): used to assert the inert path emits NO response.
    fn recv_none(&mut self) -> bool {
        // A short timeout so an (unexpected) frame is still observed but the
        // test does not block the full watchdog when nothing is sent.
        self.client
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(200)))
            .expect("TEST-ONLY short timeout");
        let got = read_frame(&mut self.client);
        // restore
        self.client
            .get_ref()
            .set_read_timeout(Some(WATCHDOG))
            .expect("TEST-ONLY read timeout");
        matches!(got, Ok(None) | Err(_))
    }
}

/// A recording mock sink: stores the scenes it was called with.
fn recording_sink() -> (KickSink, Arc<Mutex<Vec<String>>>) {
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls2 = Arc::clone(&calls);
    let sink: KickSink = Arc::new(move |req: KickRequest| {
        calls2.lock().unwrap().push(req.scene);
    });
    (sink, calls)
}

/// Build a `pasta/playScene` request Value with `seq` and a `scene` argument.
fn play_req(seq: u64, scene: &str) -> Value {
    json!({
        "seq": seq,
        "type": "request",
        "command": "pasta/playScene",
        "arguments": { "scene": scene },
    })
}

/// 2.3 / 2.4: a valid `pasta/playScene` with a wired sink — the sink is called
/// exactly once with `KickRequest { scene: "intro" }` and a success ack frame is
/// sent, correlated to the request seq.
#[test]
fn valid_play_scene_calls_sink_once_and_acks() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_req(40, "intro"),
        &SourceMapWiring::disabled(),
        Some(&sink),
    );
    assert!(ok, "handle_inbound must not report the peer gone");

    // (a) sink invoked exactly once with the decoded scene name (2.4).
    {
        let recorded = calls.lock().unwrap();
        assert_eq!(
            recorded.as_slice(),
            ["intro"],
            "sink called once with the scene"
        );
    }

    // (b) success ack, correlated to the request seq (2.3).
    let resp = h.recv();
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/playScene");
    assert_eq!(resp["request_seq"], 40);
    assert_eq!(resp["success"], true, "valid kick is acked success=true");

    // No command is forwarded into generic stop-context routing (2.3).
    assert!(
        cmd_rx.try_recv().is_err(),
        "playScene must not fall into routing"
    );
}

/// 2.5: an EMPTY scene name with a wired sink — the kick is NOT issued (sink
/// untouched) and an error response (`success: false`) is returned.
#[test]
fn empty_scene_does_not_call_sink_and_returns_error() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_req(41, "   "), // whitespace-only → decode yields None
        &SourceMapWiring::disabled(),
        Some(&sink),
    );
    assert!(ok);

    // sink NOT called (2.5).
    assert!(
        calls.lock().unwrap().is_empty(),
        "empty name must not kick (2.5)"
    );

    // error response (2.5).
    let resp = h.recv();
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/playScene");
    assert_eq!(resp["request_seq"], 41);
    assert_eq!(resp["success"], false, "empty name → error response (2.5)");

    assert!(
        cmd_rx.try_recv().is_err(),
        "playScene must not fall into routing"
    );
}

/// 2.6: NO sink wired (`None`) — the kick path is inert. The request is
/// recognised (it does NOT fall into generic routing) but no sink is called and
/// no response frame is produced.
#[test]
fn no_sink_keeps_kick_path_inert() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_req(42, "intro"),
        &SourceMapWiring::disabled(),
        None, // R2.6: sink not injected → path non-activated
    );
    assert!(ok);

    // No frame is sent back, and nothing is forwarded into routing.
    assert!(h.recv_none(), "no sink → no response frame (R2.6 inert)");
    assert!(cmd_rx.try_recv().is_err(), "no sink → no routed command");
}
