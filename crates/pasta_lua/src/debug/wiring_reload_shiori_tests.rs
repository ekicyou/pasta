//! Task 4.3 — the SHIORI-reload WIRING in [`handle_inbound`] (requirement 9.2).
//!
//! A `pasta/reloadShiori` custom request is a self-contained handler (same shape
//! as `try_play_scene_at`, NOT generic routing): when a `KickSink` is wired the
//! handler invokes the sink ONCE with the reserved [`RELOAD_SENTINEL`] scene
//! string and sends a success ack. With no sink wired (`None`) the path is inert —
//! the request is recognised but no sink call and no ack are produced (R2.6).
//!
//! The engine output `\![reload,shiori]` is produced LATER, in Lua, when the
//! sentinel kick is dispatched (see `kick_reload_shiori_test.lua`). The wiring
//! only delivers the sentinel via the single existing `KickSink`.
//!
//! These tests drive [`handle_inbound`] DIRECTLY over a real loopback
//! [`Transport`] and observe the sink via a recording mock (mirrors
//! `wiring_play_scene_at_tests.rs`).

use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::{DapAdapter, RELOAD_SENTINEL};
use crate::debug::kick::{KickRequest, KickSink};
use crate::debug::transport::{Transport, read_frame};
use crate::debug::types::SessionCommand;

use super::{SharedAdapter, SourceMapWiring, handle_inbound};

/// TEST-ONLY watchdog so a wiring test cannot hang on a frame read.
const WATCHDOG: Duration = Duration::from_secs(10);

/// A real loopback [`Transport`] plus a connected client end.
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

    fn recv(&mut self) -> Value {
        read_frame(&mut self.client)
            .expect("client read must succeed (TEST-ONLY timeout)")
            .expect("a frame must be present")
    }

    /// True if the client has no immediately-available frame (a brief grace
    /// read window): used to assert the inert path emits NO response.
    fn recv_none(&mut self) -> bool {
        self.client
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(200)))
            .expect("TEST-ONLY short timeout");
        let got = read_frame(&mut self.client);
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

/// Build a bare `pasta/reloadShiori` request Value with `seq`.
fn reload_req(seq: u64) -> Value {
    json!({
        "seq": seq,
        "type": "request",
        "command": "pasta/reloadShiori",
        "arguments": {},
    })
}

/// R9.2: a `pasta/reloadShiori` with a wired sink — the sink is called exactly
/// once with the reserved `RELOAD_SENTINEL` scene and a success ack is sent,
/// correlated to the request seq.
#[test]
fn reload_request_kicks_sentinel_once_and_acks() {
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
        &reload_req(60),
        &SourceMapWiring::disabled(),
        Some(&sink),
    );
    assert!(ok, "handle_inbound must not report the peer gone");

    // (a) sink invoked exactly once with the reserved sentinel scene (R9.2).
    {
        let recorded = calls.lock().unwrap();
        assert_eq!(
            recorded.as_slice(),
            [RELOAD_SENTINEL.to_string()],
            "sink called once with the reload sentinel"
        );
    }

    // (b) success ack, correlated to the request seq.
    let resp = h.recv();
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/reloadShiori");
    assert_eq!(resp["request_seq"], 60);
    assert_eq!(resp["success"], true, "reload → success ack");

    // No command is forwarded into generic stop-context routing.
    assert!(
        cmd_rx.try_recv().is_err(),
        "reloadShiori must not fall into routing"
    );
}

/// R2.6: NO sink wired (`None`) — the reload path is inert. The request is
/// recognised (it does NOT fall into generic routing) but no sink is called and
/// no response frame is produced.
#[test]
fn no_sink_keeps_reload_path_inert() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &reload_req(61),
        &SourceMapWiring::disabled(),
        None, // R2.6: sink not injected → path non-activated
    );
    assert!(ok);

    assert!(h.recv_none(), "no sink → no response frame (R2.6 inert)");
    assert!(cmd_rx.try_recv().is_err(), "no sink → no routed command");
}
