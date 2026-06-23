//! Task 4.2 (requirement 5.4) — the OLD name-based `pasta/playScene` external
//! transport has been REMOVED. The only external scene-execution entry is now
//! the position-based `pasta/playSceneAt` (see `wiring_play_scene_at_tests.rs`).
//!
//! This file used to drive the name-based scene-kick wiring (`try_play_scene_kick`):
//! a `pasta/playScene` request invoked the injected `KickSink` and acked. That
//! handler is gone. The remaining test below pins the removal INVARIANT: an
//! external name-based `pasta/playScene` request is NO LONGER handled as a
//! scene-kick — even with a sink wired, the sink is NEVER invoked and no scene-kick
//! ack is produced. The request is now an unrecognised custom request (it decodes
//! to an empty `Decoded`) and is dropped by generic routing.
//!
//! These tests drive [`handle_inbound`] DIRECTLY over a real loopback
//! [`Transport`] (a connected client reads any response frames) and observe the
//! sink via a recording mock, so both the wire frames AND the sink calls are
//! asserted.

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

    /// True if the client has no immediately-available frame (a brief grace
    /// read window): used to assert the removed path emits NO response.
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

/// Build a (now-defunct) `pasta/playScene` request Value with `seq` and a
/// `scene` argument.
fn play_req(seq: u64, scene: &str) -> Value {
    json!({
        "seq": seq,
        "type": "request",
        "command": "pasta/playScene",
        "arguments": { "scene": scene },
    })
}

/// R5.4: the external name-based `pasta/playScene` transport is GONE. Even with a
/// `KickSink` wired AND a valid (non-empty) scene name, the request is NOT handled
/// as a scene-kick: the sink is NEVER invoked, no scene-kick ack/error frame is
/// produced, and nothing special is routed. (Before task 4.2 this exact request
/// invoked the sink once and acked `success: true` — that path no longer exists.)
#[test]
fn name_based_play_scene_is_not_accepted_as_scene_kick() {
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
        &play_req(40, "intro"), // a valid name that the OLD path would have kicked
        &SourceMapWiring::disabled(),
        Some(&sink), // sink IS wired — the old path would have invoked it
    );
    assert!(ok, "handle_inbound must not report the peer gone");

    // The sink is NEVER invoked from a name-based external request (R5.4).
    assert!(
        calls.lock().unwrap().is_empty(),
        "R5.4: name-based pasta/playScene must NOT invoke the kick sink"
    );

    // No scene-kick response frame is produced (the old `pasta/playScene` ack is
    // gone); the request decodes to an empty `Decoded` and routes to nothing.
    assert!(
        h.recv_none(),
        "R5.4: removed name-based path must emit no scene-kick response frame"
    );

    // Nothing is forwarded into generic stop-context routing either.
    assert!(
        cmd_rx.try_recv().is_err(),
        "R5.4: unrecognised pasta/playScene routes to no command"
    );
}
