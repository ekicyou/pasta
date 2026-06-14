//! Task 3.1 — the runtime presentation-toggle WIRING in [`handle_inbound`]
//! (requirements 1.1/1.2/1.4/1.5, 2.5/2.6, 3.1/3.2/3.4/3.5, 4.2/4.3).
//!
//! A `pasta/sourcePresentation` custom request must, IN ORDER: apply the mode
//! to the shared cell + swap the resolver (valid mode only), send the
//! acceptance response, emit the `pasta/sourcePresentation` custom event, and
//! forward `SessionCommand::RefreshPresentation`. An UNRECOGNIZED mode value
//! makes NO cell change (1.4) but still acks + echoes the current mode. The
//! `attach` completion (explicit AND no-arg) emits the resolved initial-mode
//! event (2.5 initial display).
//!
//! These tests drive [`handle_inbound`] DIRECTLY over a real loopback
//! [`Transport`] (a connected client reads the response/event frames), so the
//! exact wire frames, the cell state, the swapped resolver (observed via a
//! subsequent `stackTrace` encode), and the forwarded command are all asserted.

use std::collections::BTreeMap;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::DapAdapter;
use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
use crate::debug::transport::{Transport, read_frame};
use crate::debug::types::{FrameInfo, SessionCommand, SessionEvent};
use crate::debug::{SharedSourceMode, SourceMode};

use super::{SharedAdapter, SourceMapWiring, handle_inbound};

/// TEST-ONLY watchdog so a wiring test cannot hang on a frame read.
const WATCHDOG: Duration = Duration::from_secs(10);

/// A `chunk(.lua line) → .pasta` map with a single known correspondence, so a
/// `stackTrace` encode observably switches presentation when the `.pasta`
/// resolver is installed.
fn map_with(chunk: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
    let mut forward = BTreeMap::new();
    forward.insert(
        lua_line,
        PastaPos {
            file: file.to_string(),
            line: pasta_line,
        },
    );
    let mut sm = SourceMap::new();
    sm.insert_chunk(
        chunk.to_string(),
        file.to_string(),
        ChunkSourceMap::from_forward(forward),
    );
    sm
}

/// Pasta-capable wiring (map present) whose EFFECTIVE mode starts at `start`.
fn wiring_with(map: SourceMap, start: SourceMode) -> SourceMapWiring {
    SourceMapWiring {
        source_map: Some(Arc::new(map)),
        source_mode: SharedSourceMode::new(start),
    }
}

/// A real loopback [`Transport`] plus a connected client end. The bridge owns
/// the `Transport` (its writer thread serializes `send()`s to the socket); the
/// `client` reads those frames back. Lets a wiring test observe the EXACT wire
/// frames `handle_inbound` emits via `transport.send`.
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
}

/// Build a `pasta/sourcePresentation` request Value with `seq` and a `mode`.
fn toggle_req(seq: u64, mode: &str) -> Value {
    json!({
        "seq": seq,
        "type": "request",
        "command": "pasta/sourcePresentation",
        "arguments": { "mode": mode },
    })
}

/// Encode ONE `stackTrace` over `adapter` for `(source, line)` and return the
/// top frame's presented `source`/`line` — observes which resolver is live.
fn top_frame(adapter: &SharedAdapter, source: &str, line: u32) -> (Value, u32) {
    let mut dap = adapter.lock().unwrap();
    dap.decode_request(&json!({
        "seq": 9, "type": "request", "command": "stackTrace",
        "arguments": { "threadId": 1 },
    }));
    let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
        source: source.to_string(),
        line,
        func_name: Some("f".to_string()),
    }]));
    let frame = &out[0]["body"]["stackFrames"][0];
    (frame["source"].clone(), frame["line"].as_u64().unwrap() as u32)
}

/// 1.1 / 4.2 / 4.3 / 3.1 / 3.4 / 2.6 / 1.3: a valid `pasta/sourcePresentation`
/// toggle to `lua` from a `.pasta` default — updates the cell, swaps the
/// resolver to default `.lua` (observed via a subsequent stackTrace), sends the
/// acceptance response echoing `lua`, emits the custom event `lua`, and
/// forwards `RefreshPresentation`, in that exact order.
#[test]
fn valid_toggle_to_lua_applies_acks_events_and_forwards_refresh() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Pasta,
    );
    // Install the initial (`.pasta`) resolver, mirroring run_socket_bridge.
    super::attach_pasta_resolver(&adapter, &wiring);
    // Precondition: Pasta presents the `.pasta` source.
    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(src, json!({ "path": "C:/proj/scene.pasta" }));
    assert_eq!(line, 3);

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &toggle_req(70, "lua"),
        &wiring,
    );
    assert!(ok, "handle_inbound must not report the peer gone");

    // (a) the cell is updated to Lua (1.1 / 4.2 / 4.3).
    assert_eq!(wiring.source_mode.get(), SourceMode::Lua, "cell set to lua (1.1)");

    // (b) the resolver swapped to default `.lua` — the SAME frame now presents
    // the generated `.lua` (3.1 / 3.4).
    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        src,
        json!({ "path": r"@C:\proj\cache\scene.lua" }),
        "resolver swapped → generated `.lua` presentation (3.1/3.4)"
    );
    assert_eq!(line, 7);

    // (c) acceptance response FIRST (1.3), echoing the resolved current mode.
    let resp = h.recv();
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/sourcePresentation");
    assert_eq!(resp["request_seq"], 70, "ack correlates to the request seq (1.3)");
    assert_eq!(resp["success"], true);
    assert_eq!(resp["body"]["mode"], "lua", "ack echoes the resolved mode (1.3)");

    // (d) THEN the custom event with the current mode (2.6 push notification).
    let ev = h.recv();
    assert_eq!(ev["type"], "event");
    assert_eq!(ev["event"], "pasta/sourcePresentation");
    assert_eq!(ev["body"]["mode"], "lua", "event carries the new mode (2.6)");

    // (e) RefreshPresentation forwarded to the session (3.3 path).
    let cmd = cmd_rx
        .recv_timeout(WATCHDOG)
        .expect("RefreshPresentation must be forwarded to the session");
    assert_eq!(cmd, SessionCommand::RefreshPresentation);
    assert!(cmd_rx.try_recv().is_err(), "exactly one command forwarded");
}

/// 1.2 / 4.2 / 3.2 / 3.5: a valid toggle to `pasta` from a `.lua` default —
/// updates the cell to Pasta and swaps in the `.pasta` resolver (observed via a
/// subsequent stackTrace presenting `.pasta`).
#[test]
fn valid_toggle_to_pasta_swaps_in_pasta_resolver() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Lua,
    );
    super::attach_pasta_resolver(&adapter, &wiring);
    // Precondition: Lua presents the generated `.lua`.
    let (src, _line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(src, json!({ "path": r"@C:\proj\cache\scene.lua" }));

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &toggle_req(71, "pasta"),
        &wiring,
    );
    assert!(ok);
    assert_eq!(wiring.source_mode.get(), SourceMode::Pasta, "cell set to pasta (1.2)");

    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        src,
        json!({ "path": "C:/proj/scene.pasta" }),
        "resolver swapped → `.pasta` presentation (3.2/3.5)"
    );
    assert_eq!(line, 3);

    // ack `pasta`, event `pasta`, RefreshPresentation.
    let resp = h.recv();
    assert_eq!(resp["request_seq"], 71);
    assert_eq!(resp["body"]["mode"], "pasta");
    let ev = h.recv();
    assert_eq!(ev["event"], "pasta/sourcePresentation");
    assert_eq!(ev["body"]["mode"], "pasta");
    assert_eq!(
        cmd_rx.recv_timeout(WATCHDOG).unwrap(),
        SessionCommand::RefreshPresentation
    );
}

/// 1.4: an UNRECOGNIZED mode value makes NO cell change and does NOT swap the
/// resolver, but STILL acks + echoes the CURRENT (unchanged) mode and emits the
/// event + forwards RefreshPresentation (the request is still accepted; only
/// the mode is left as-is).
#[test]
fn invalid_mode_leaves_cell_unchanged_but_acks_current() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Pasta,
    );
    super::attach_pasta_resolver(&adapter, &wiring);

    let ok = handle_inbound(
        &h.transport,
        &adapter,
        &breakpoints,
        &cmd_tx,
        &toggle_req(72, "bogus"),
        &wiring,
    );
    assert!(ok);

    // 1.4: NO change — still Pasta, and the `.pasta` resolver is still live.
    assert_eq!(
        wiring.source_mode.get(),
        SourceMode::Pasta,
        "1.4: unrecognized mode must NOT change the cell"
    );
    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        src,
        json!({ "path": "C:/proj/scene.pasta" }),
        "1.4: resolver unchanged (still `.pasta`)"
    );
    assert_eq!(line, 3);

    // Still acks + echoes the CURRENT mode (pasta), event, RefreshPresentation.
    let resp = h.recv();
    assert_eq!(resp["request_seq"], 72);
    assert_eq!(resp["body"]["mode"], "pasta", "1.4: echo the current (unchanged) mode");
    let ev = h.recv();
    assert_eq!(ev["body"]["mode"], "pasta");
    assert_eq!(
        cmd_rx.recv_timeout(WATCHDOG).unwrap(),
        SessionCommand::RefreshPresentation
    );
}

/// 2.5 (initial display): `attach` WITH an explicit `sourcePresentation` emits
/// the resolved initial-mode `pasta/sourcePresentation` event AFTER the attach
/// ack, so the extension can show the resolved initial mode.
#[test]
fn attach_with_explicit_mode_emits_initial_event() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Pasta,
    );
    super::attach_pasta_resolver(&adapter, &wiring);

    let attach = json!({
        "seq": 5, "type": "request", "command": "attach",
        "arguments": { "sourcePresentation": "lua" },
    });
    let ok = handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &attach, &wiring);
    assert!(ok);
    assert_eq!(wiring.source_mode.get(), SourceMode::Lua, "explicit attach mode applied");

    // attach ack first, then the resolved initial-mode event.
    let ack = h.recv();
    assert_eq!(ack["type"], "response");
    assert_eq!(ack["command"], "attach");
    let ev = h.recv();
    assert_eq!(ev["type"], "event");
    assert_eq!(ev["event"], "pasta/sourcePresentation");
    assert_eq!(ev["body"]["mode"], "lua", "2.5: event carries the resolved initial mode");
}

/// 2.5 (initial display, no-arg attach): `attach` WITHOUT `sourcePresentation`
/// keeps the resolved env/file/既定 mode and STILL emits the resolved
/// initial-mode event (the no-arg path must also publish the initial display).
#[test]
fn attach_without_mode_emits_resolved_initial_event() {
    let mut h = Harness::new();
    let adapter: SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));
    let breakpoints = BreakpointSet::new();
    let (cmd_tx, _cmd_rx): (_, Receiver<SessionCommand>) = mpsc::channel();
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Pasta,
    );
    super::attach_pasta_resolver(&adapter, &wiring);

    let attach = json!({
        "seq": 6, "type": "request", "command": "attach",
        "arguments": {},
    });
    let ok = handle_inbound(&h.transport, &adapter, &breakpoints, &cmd_tx, &attach, &wiring);
    assert!(ok);
    // No-arg → the resolved mode (Pasta) is kept.
    assert_eq!(wiring.source_mode.get(), SourceMode::Pasta, "no-arg attach keeps resolved mode");

    let ack = h.recv();
    assert_eq!(ack["command"], "attach");
    let ev = h.recv();
    assert_eq!(ev["event"], "pasta/sourcePresentation");
    assert_eq!(
        ev["body"]["mode"], "pasta",
        "2.5: no-arg attach still publishes the resolved initial mode"
    );
}
