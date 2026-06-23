//! Task 4.1 — the position-based scene-kick WIRING in [`handle_inbound`]
//! (requirements 4.1 / 4.2 / 4.3 / 4.4, 2.3 / 2.6).
//!
//! A `pasta/playSceneAt` custom request is a self-contained handler (same shape
//! as `try_play_scene_kick`, NOT generic routing): when a `KickSink` is wired AND
//! a `SourceMap` index is loaded, the handler resolves the decoded `(uri, line)`
//! via `resolve_and_kick`, invokes the sink with the resolved scene exactly once,
//! and sends a success ack (R4.2). A position with no scene (`NotFound`) sends an
//! error response with a reason and never calls the sink (R4.3). With no sink
//! wired (`None`), the kick path is inert — the request is recognised but no sink
//! call and no ack are produced (R2.6).
//!
//! These tests drive [`handle_inbound`] DIRECTLY over a real loopback
//! [`Transport`] (a connected client reads the response frames) and observe the
//! sink via a recording mock, so the exact wire frames AND the sink calls are
//! both asserted (mirrors `wiring_play_scene_kick_tests.rs`).

use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::DapAdapter;
use crate::debug::kick::{KickRequest, KickSink};
use crate::debug::source_map::{SceneIdentityIndex, SourceMap};
use crate::debug::transport::{Transport, read_frame};
use crate::debug::types::SessionCommand;

use super::{SharedAdapter, SourceMapWiring, handle_inbound};

/// TEST-ONLY watchdog so a wiring test cannot hang on a frame read.
const WATCHDOG: Duration = Duration::from_secs(10);

/// A real loopback [`Transport`] plus a connected client end (mirrors the
/// scene-kick harness).
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

/// A `SourceMapWiring` carrying a loaded `SourceMap` whose scene index resolves
/// `file` line 25 to the local scene `挨拶_1` (parent `会話1` → composite
/// `:会話1:挨拶_1`) and line 15 to the global `会話1` (mirrors the
/// `playscene_tests` fixture). Lines past 40 are not found.
fn wiring_with_index(file: &str) -> SourceMapWiring {
    let mut b = SceneIdentityIndex::builder();
    b.add_scene(file, "会話1", None, 10, 40, 0);
    b.add_scene(file, "挨拶_1", Some("会話1"), 20, 30, 1);
    let index = b.finish();

    let map = SourceMap::new();
    map.set_scene_index(index)
        .expect("write-once index set must succeed");

    let mut wiring = SourceMapWiring::disabled();
    wiring.source_map = Some(Arc::new(map));
    wiring
}

/// Build a `pasta/playSceneAt` request Value with `seq`, `uri`, and `line`.
fn play_at_req(seq: u64, uri: &str, line: u32) -> Value {
    json!({
        "seq": seq,
        "type": "request",
        "command": "pasta/playSceneAt",
        "arguments": { "uri": uri, "line": line },
    })
}

/// 4.2: a `pasta/playSceneAt` at a resolvable position with a wired sink — the
/// sink is called exactly once with the resolved composite scene
/// (`:会話1:挨拶_1`) and a success ack is sent, correlated to the request seq.
#[test]
fn resolvable_position_calls_sink_once_and_acks() {
    let file = "C:/work/dic/talk.pasta";
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();
    let wiring = wiring_with_index(file);

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_at_req(50, "file:///C:/work/dic/talk.pasta", 25),
        &wiring,
        Some(&sink),
    );
    assert!(ok, "handle_inbound must not report the peer gone");

    // (a) sink invoked exactly once with the resolved composite scene (4.2).
    {
        let recorded = calls.lock().unwrap();
        assert_eq!(
            recorded.as_slice(),
            [":会話1:挨拶_1".to_string()],
            "sink called once with the resolved scene"
        );
    }

    // (b) success ack, correlated to the request seq (4.2).
    let resp = h.recv();
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/playSceneAt");
    assert_eq!(resp["request_seq"], 50);
    assert_eq!(resp["success"], true, "resolvable position → success ack");

    // No command is forwarded into generic stop-context routing.
    assert!(
        cmd_rx.try_recv().is_err(),
        "playSceneAt must not fall into routing"
    );
}

/// 4.2 (global): a position inside the global scene body (line 15, outside the
/// local span) resolves to the plain scene id `会話1`.
#[test]
fn resolvable_global_position_kicks_plain_scene() {
    let file = "C:/work/dic/talk.pasta";
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();
    let wiring = wiring_with_index(file);

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_at_req(51, "file:///C:/work/dic/talk.pasta", 15),
        &wiring,
        Some(&sink),
    );
    assert!(ok);

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["会話1".to_string()],
        "global position resolves to plain scene id"
    );
    let resp = h.recv();
    assert_eq!(resp["success"], true);
    assert_eq!(resp["command"], "pasta/playSceneAt");
}

/// 4.3 / 2.6: a position with NO scene (`NotFound`) — the kick is NOT issued
/// (sink untouched) and an error response (`success: false`) with a reason
/// `message` is returned.
#[test]
fn not_found_position_does_not_call_sink_and_returns_error() {
    let file = "C:/work/dic/talk.pasta";
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();
    let wiring = wiring_with_index(file);

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_at_req(52, "file:///C:/work/dic/talk.pasta", 100), // past all scenes
        &wiring,
        Some(&sink),
    );
    assert!(ok);

    // sink NOT called (4.3 / 2.6).
    assert!(
        calls.lock().unwrap().is_empty(),
        "not-found position must not kick (4.3)"
    );

    // error response with a reason message (4.3).
    let resp = h.recv();
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/playSceneAt");
    assert_eq!(resp["request_seq"], 52);
    assert_eq!(resp["success"], false, "not-found → error response (4.3)");
    assert!(
        resp["message"].is_string() && !resp["message"].as_str().unwrap().is_empty(),
        "error response carries a non-empty reason (4.3): {resp}"
    );

    assert!(
        cmd_rx.try_recv().is_err(),
        "playSceneAt must not fall into routing"
    );
}

/// 4.4: an INVALID request (strict parse failure — e.g. missing line) sends an
/// error response and never calls the sink.
#[test]
fn invalid_request_returns_error_and_does_not_kick() {
    let file = "C:/work/dic/talk.pasta";
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();
    let wiring = wiring_with_index(file);

    // Missing `line` → decode yields play_scene_at = None.
    let req = json!({
        "seq": 53,
        "type": "request",
        "command": "pasta/playSceneAt",
        "arguments": { "uri": "file:///C:/work/dic/talk.pasta" },
    });
    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &req,
        &wiring,
        Some(&sink),
    );
    assert!(ok);

    assert!(
        calls.lock().unwrap().is_empty(),
        "invalid request must not kick (4.4)"
    );
    let resp = h.recv();
    assert_eq!(resp["command"], "pasta/playSceneAt");
    assert_eq!(resp["request_seq"], 53);
    assert_eq!(resp["success"], false, "invalid request → error response (4.4)");
}

/// No loaded map: a `pasta/playSceneAt` with a wired sink but no `SourceMap`
/// (`source_map == None`) cannot resolve — an error response is sent and the
/// sink is never called.
#[test]
fn no_loaded_map_returns_error_and_does_not_kick() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let (sink, calls) = recording_sink();

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_at_req(54, "file:///C:/work/dic/talk.pasta", 25),
        &SourceMapWiring::disabled(), // no map
        Some(&sink),
    );
    assert!(ok);

    assert!(
        calls.lock().unwrap().is_empty(),
        "no map → cannot resolve → no kick"
    );
    let resp = h.recv();
    assert_eq!(resp["command"], "pasta/playSceneAt");
    assert_eq!(resp["request_seq"], 54);
    assert_eq!(resp["success"], false, "no map → error response");
}

/// 2.6: NO sink wired (`None`) — the kick path is inert. The request is
/// recognised (it does NOT fall into generic routing) but no sink is called and
/// no response frame is produced.
#[test]
fn no_sink_keeps_play_scene_at_path_inert() {
    let file = "C:/work/dic/talk.pasta";
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let wiring = wiring_with_index(file);

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &play_at_req(55, "file:///C:/work/dic/talk.pasta", 25),
        &wiring,
        None, // R2.6: sink not injected → path non-activated
    );
    assert!(ok);

    assert!(h.recv_none(), "no sink → no response frame (R2.6 inert)");
    assert!(cmd_rx.try_recv().is_err(), "no sink → no routed command");
}
