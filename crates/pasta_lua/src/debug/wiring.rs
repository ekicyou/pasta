//! Backend wiring: the transport↔dap↔session↔hook bridge threads (task 4.1).
//!
//! This module owns the two `Send`-only bridge threads that connect the
//! I/O-side [`Transport`](crate::debug::transport::Transport) +
//! [`DapAdapter`](crate::debug::dap::DapAdapter) to the VM-side
//! [`DebugSession`](crate::debug::session::DebugSession). The VM-side hook
//! install and the session itself live on the VM thread (the thread that calls
//! [`enable`](crate::debug::enable)); this module never touches `mlua::Lua`
//! (it is `!Send` and stays pinned to the VM thread). Only `std::sync::mpsc`
//! channels carrying `Send` payloads ([`SessionCommand`] / [`SessionEvent`] /
//! [`serde_json::Value`]) cross between the threads here.
//!
//! # Thread topology (design "Architecture" / "System Flows" / スレッドモデル)
//!
//! Three concurrent roles plus the transport's own internal reader thread,
//! connected only by channels:
//!
//! 1. **VM host thread** (caller of `enable`, owns `mlua::Lua`): the line hook
//!    drives [`DebugSession::on_line`]; when stopped, the session processes
//!    inspect/step/continue commands IN the hook loop, on this thread, calling
//!    `inspect::capture_*` on `lua.current_thread()`. It reads the session's
//!    `cmd_rx` and writes the session's `event_tx`.
//! 2. **Socket bridge thread** ([`run_socket_bridge`]): the SOLE owner of the
//!    [`Transport`]. [`Transport`] is `!Sync` (it holds a
//!    `Receiver<Value>`), so it cannot be shared across threads — exactly one
//!    thread owns it and performs BOTH socket reads and socket writes. This
//!    thread multiplexes, per iteration:
//!    - **inbound** (socket → us): a bounded `recv_timeout` poll of
//!      `transport.inbound()`. Each decoded DAP request becomes (a) immediate
//!      response/event frames written straight back, (b) a `setBreakpoints`
//!      applied DIRECTLY to the shared [`BreakpointSet`] (settable while the VM
//!      is RUNNING — design "System Flows": `Arc<Mutex>` 共有) whose DAP response
//!      is produced via the adapter and written back, or (c) a stop-context
//!      [`SessionCommand`] forwarded to the session's `cmd_tx`.
//!    - **outbound** (session → socket): drains the `out_rx` frame channel fed
//!      by the encoder thread and writes each frame to the socket.
//! 3. **Event encoder thread** ([`run_event_encoder`]): drains the session's
//!    `event_rx` ([`SessionEvent`]s), encodes each via the shared [`DapAdapter`]
//!    (`encode_event`) into DAP frames, and pushes them into the `out_tx` frame
//!    channel for the socket bridge to write. It never touches the `Transport`.
//!
//! `std::sync::mpsc` has no `select`, and `Transport` is `!Sync`, so the socket
//! bridge polls inbound with a small timeout ([`POLL_INTERVAL`]) and drains the
//! encoder's frame channel between polls — the "equivalent structure" to two
//! independent bridge loops. The poll interval is small enough to be
//! imperceptible for interactive debugging and adds no busy-spin (it blocks for
//! the interval when idle).
//!
//! # Shared `DapAdapter` (`Arc<Mutex<…>>`)
//!
//! The adapter is the single stateful correlation point (a monotonic `seq`
//! counter + per-kind FIFO `request_seq` table). It is mutated by BOTH the
//! socket bridge (decoding requests, producing the `setBreakpoints` response)
//! and the encoder thread (encoding events), so it is shared behind an
//! `Arc<Mutex<…>>`.
//!
//! # No double-response to `scopes`
//!
//! `DapAdapter::decode_request("scopes")` SELF-ANSWERS the scopes response at
//! decode time (from the frame id alone) AND still returns a
//! `SessionCommand::Scopes`. The socket bridge sends that self-answer
//! immediately and forwards the `Scopes` command; the session replies with a
//! `SessionEvent::Scopes`, but `DapAdapter::encode_event(Scopes)` is a
//! deliberate no-op (returns no frames), so the client receives EXACTLY one
//! scopes response. The same single-response guarantee holds for `threads`
//! (deferred: only the `SessionEvent::Threads` produces the wire response) and
//! `setBreakpoints` (only the bridge-applied `Breakpoints` event produces it).
//!
//! # Shutdown (no hang)
//!
//! A shared [`AtomicBool`] shutdown flag lets the owner ([`DebugHandle`]) signal
//! the socket bridge to stop without blocking: the bridge checks it each poll
//! iteration and exits within [`POLL_INTERVAL`], dropping the `Transport` (which
//! winds the transport down). The encoder thread ends when the session's
//! `event_rx` closes (the VM thread finished and dropped the session) or when
//! the frame channel's receiver is gone.

#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::Duration;

use serde_json::Value;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::DapAdapter;
use crate::debug::transport::Transport;
use crate::debug::types::{SessionCommand, SessionEvent};

/// Inbound poll interval for the socket bridge. `std::sync::mpsc` has no
/// `select` and [`Transport`] is `!Sync`, so the single Transport-owner thread
/// polls inbound with this timeout and drains the outbound frame channel between
/// polls. Small enough to be imperceptible interactively; it blocks (does not
/// busy-spin) for the interval when idle.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Shared DAP adapter (seq counter + per-kind FIFO `request_seq` correlation),
/// mutated by BOTH the socket bridge and the event encoder thread.
pub(crate) type SharedAdapter = Arc<Mutex<DapAdapter>>;

/// Socket bridge body: the SOLE owner of the [`Transport`]. Multiplexes inbound
/// socket frames (poll) and outbound encoded frames (`out_rx`) on one thread,
/// because the `Transport` is `!Sync` and `mpsc` has no `select`.
///
/// Per iteration it:
/// 1. checks `shutdown` (set by the handle's Drop) and exits if signalled;
/// 2. polls `transport.inbound()` with [`POLL_INTERVAL`]; on a frame, decodes it
///    via the shared adapter and writes immediate responses / applies
///    `setBreakpoints` to the shared store (writing its response) / forwards
///    stop-context commands to the session;
/// 3. drains the encoder's `out_rx` and writes each frame to the socket.
///
/// Returns (winding the transport down by dropping it) when the inbound channel
/// closes (client disconnect / shutdown) or the shutdown flag is set — never a
/// hang.
pub(crate) fn run_socket_bridge(
    transport: Transport,
    adapter: SharedAdapter,
    breakpoints: BreakpointSet,
    cmd_tx: Sender<SessionCommand>,
    out_rx: Receiver<Value>,
    shutdown: Arc<AtomicBool>,
) {
    loop {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }

        // (1) Inbound: poll one frame (bounded so we can also service outbound).
        match transport.inbound().recv_timeout(POLL_INTERVAL) {
            Ok(req) => {
                if !handle_inbound(&transport, &adapter, &breakpoints, &cmd_tx, &req) {
                    return; // peer gone while replying → done
                }
            }
            // Idle this interval: fall through to drain outbound.
            Err(RecvTimeoutError::Timeout) => {}
            // Inbound channel closed (client disconnected / transport reader
            // ended): flush any pending outbound frames, then stop.
            Err(RecvTimeoutError::Disconnected) => {
                drain_outbound(&transport, &out_rx);
                return;
            }
        }

        // (2) Outbound: drain all currently-available encoded frames.
        if !drain_outbound(&transport, &out_rx) {
            return; // peer gone → done
        }
    }
}

/// Decode and act on one inbound DAP request frame. Returns `false` if the peer
/// is gone (a transport write failed) so the caller stops.
fn handle_inbound(
    transport: &Transport,
    adapter: &SharedAdapter,
    breakpoints: &BreakpointSet,
    cmd_tx: &Sender<SessionCommand>,
    req: &Value,
) -> bool {
    // Decode under the shared adapter lock (seq counter / pending table).
    let decoded = {
        let mut dap = match adapter.lock() {
            Ok(g) => g,
            Err(_) => return false, // poisoned → stop (never panic in the bridge)
        };
        dap.decode_request(req)
    };

    // (a) Immediate response (acks / initialize / scopes self-answer).
    if let Some(response) = decoded.response {
        if transport.send(response).is_err() {
            return false;
        }
    }
    // (b) Immediate unsolicited events (the `initialized` handshake event).
    for ev in decoded.events {
        if transport.send(ev).is_err() {
            return false;
        }
    }

    // (c) Command routing.
    match decoded.command {
        // `setBreakpoints` is the ONE command valid while the VM runs: apply it
        // directly to the shared store and synthesize the DAP response via the
        // adapter (correlated to the originating request seq). It is NOT
        // forwarded to the session (that would block off a stop).
        Some(SessionCommand::SetBreakpoints { source, lines }) => {
            let resolved = breakpoints.set_breakpoints(&source, &lines);
            let frames = {
                let mut dap = match adapter.lock() {
                    Ok(g) => g,
                    Err(_) => return false,
                };
                dap.encode_event(SessionEvent::Breakpoints(resolved))
            };
            for frame in frames {
                if transport.send(frame).is_err() {
                    return false;
                }
            }
        }
        // Every other (stop-context) command is forwarded to the session's
        // VM-thread stop loop. If the session controller is gone, stop.
        Some(cmd) => {
            if cmd_tx.send(cmd).is_err() {
                return false;
            }
        }
        None => {}
    }
    true
}

/// Drain all currently-available outbound frames from `out_rx` and write them.
/// Returns `false` if a transport write failed (peer gone) so the caller stops;
/// a closed `out_rx` (encoder thread ended) is NOT a stop condition here — the
/// client may still send inbound commands.
fn drain_outbound(transport: &Transport, out_rx: &Receiver<Value>) -> bool {
    loop {
        match out_rx.try_recv() {
            Ok(frame) => {
                if transport.send(frame).is_err() {
                    return false;
                }
            }
            // Nothing pending, or the encoder thread is gone: stop draining.
            Err(_) => return true,
        }
    }
}

/// Event encoder body: session `event_rx` ([`SessionEvent`]) → encode via the
/// shared [`DapAdapter`] → push DAP frames into `out_tx` for the socket bridge.
///
/// Runs on its own thread (no `mlua::Lua`, no `Transport`). Returns when the
/// session event channel closes (the session/VM is gone) or the frame channel's
/// receiver is gone — both clean, never a hang.
pub(crate) fn run_event_encoder(
    adapter: SharedAdapter,
    event_rx: Receiver<SessionEvent>,
    out_tx: Sender<Value>,
) {
    while let Ok(event) = event_rx.recv() {
        let frames = {
            let mut dap = match adapter.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            dap.encode_event(event)
        };
        for frame in frames {
            if out_tx.send(frame).is_err() {
                return; // socket bridge gone → done
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! End-to-end integration: drive the FULL attach→BP→stack→vars→step→
    //! continue→terminated path over real TCP through [`enable`], exercising
    //! every layer (transport / dap / session / hook / inspect) wired together.
    //!
    //! `mlua::Lua` is `!Send`: it is built and owned entirely on the VM host
    //! thread; only channels / the bound address (a `SocketAddr`, `Copy`) cross
    //! the thread boundary. All client-side waits use a TEST-ONLY watchdog so CI
    //! cannot hang; the stop core itself stays unbounded.

    use std::io::BufReader;
    use std::net::{SocketAddr, TcpStream};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    use serde_json::{Value, json};

    use crate::debug::transport::{read_frame, write_frame};
    use crate::debug::{DebugConfig, enable};

    /// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// The generated-`.lua` source name and breakpoint line for the scenario.
    const SCENARIO_SOURCE: &str = "@e2e_scenario";

    /// The scenario chunk: a top-level chunk that drives a coroutine whose body
    /// has the breakpoint target (a coroutine-body local must be inspectable).
    /// The breakpoint sits on a line AFTER `co_local` is assigned so the local
    /// is a live, named slot when inspected (a local on its OWN declaration line
    /// is still an unnamed `(*temporary)` slot). Lines (1-origin):
    ///   1: local function helper(x)
    ///   2:     local y = x + 1
    ///   3:     return y
    ///   4: end
    ///   5: local body = function()
    ///   6:     local co_local = 7
    ///   7:     local marker = co_local      <- BREAKPOINT (co_local is live here)
    ///   8:     local doubled = helper(marker)
    ///   9:     coroutine.yield()
    ///  10:     return doubled
    ///  11: end
    ///  12: local co = coroutine.create(body)
    ///  13: while coroutine.status(co) ~= 'dead' do
    ///  14:     coroutine.resume(co)
    ///  15: end
    const SCENARIO_CHUNK: &str = "\
local function helper(x)
    local y = x + 1
    return y
end
local body = function()
    local co_local = 7
    local marker = co_local
    local doubled = helper(marker)
    coroutine.yield()
    return doubled
end
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
";
    const BREAKPOINT_LINE: u32 = 7;

    /// A test DAP client over a real TCP socket: Content-Length framed JSON.
    struct DapClient {
        reader: BufReader<TcpStream>,
        writer: TcpStream,
    }

    impl DapClient {
        fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
            stream
                .set_read_timeout(Some(WATCHDOG))
                .expect("TEST-ONLY read timeout");
            let writer = stream.try_clone().expect("clone socket for writing");
            Self {
                reader: BufReader::new(stream),
                writer,
            }
        }

        /// Send a DAP request with the given seq/command/arguments.
        fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
            let req = json!({
                "seq": seq,
                "type": "request",
                "command": command,
                "arguments": arguments,
            });
            write_frame(&mut self.writer, &req).expect("client write must succeed");
        }

        /// Read the next framed message (bounded by the TEST-ONLY read timeout).
        fn recv(&mut self) -> Value {
            read_frame(&mut self.reader)
                .expect("client read must succeed (TEST-ONLY timeout)")
                .expect("a frame must be present (peer did not close)")
        }

        /// Read messages until one matching `pred` arrives; returns it. Bounded
        /// by the read timeout per read so CI cannot hang.
        fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
            loop {
                let msg = self.recv();
                if pred(&msg) {
                    return msg;
                }
            }
        }
    }

    fn is_event(msg: &Value, name: &str) -> bool {
        msg["type"] == "event" && msg["event"] == name
    }

    fn is_response(msg: &Value, command: &str) -> bool {
        msg["type"] == "response" && msg["command"] == command
    }

    /// The headline integration test (task 4.1 "done"): a full DAP session over
    /// real TCP through `enable`, hitting a breakpoint inside a coroutine body,
    /// inspecting the stack and a coroutine-body local, stepping, continuing,
    /// and running to completion — all layers wired end-to-end.
    #[test]
    fn full_dap_session_over_tcp_attach_bp_stack_vars_step_continue_terminated() {
        // Coordination channels: host → main carries the bound addr; main → host
        // carries the "breakpoints are set, run the VM now" go signal. mlua::Lua
        // never crosses — only these Send values do.
        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        // VM HOST THREAD: build the VM, enable the backend, publish the bound
        // addr, wait for the client handshake, then run the scenario chunk.
        let host = std::thread::spawn(move || -> Result<(), String> {
            // ALL_SAFE VM: `jit` exists, `debug` excluded; `enable`'s hook does
            // jit.off() itself (mirrors the other debug tests' VM build).
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                // Port 0 → OS-assigned free loopback port (no fixed-port clash).
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_map_slice: false,
            };
            let handle = enable(&lua, &cfg)
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            // Wait for the client to finish initialize/setBreakpoints/
            // configurationDone before running the VM (so the BP is live).
            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            // Run the scenario. This blocks at the breakpoint until the client
            // sends continue; the VM thread processes inspect/step in the hook.
            lua.load(SCENARIO_CHUNK)
                .set_name(SCENARIO_SOURCE)
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;
            lua.remove_global_hook();

            // Keep the handle alive until the chunk has fully run, then drop it
            // (Drop signals shutdown + winds the transport/bridges down).
            drop(handle);
            Ok(())
        });

        // CLIENT (this thread): connect and drive the DAP handshake + session.
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // --- initialize → capabilities + `initialized` ---
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let init_resp = client.recv_until(|m| is_response(m, "initialize"));
        assert_eq!(init_resp["success"], true, "initialize must succeed");
        assert_eq!(init_resp["request_seq"], 1);
        assert_eq!(
            init_resp["body"]["supportsConfigurationDoneRequest"], true,
            "initialize must advertise supportsConfigurationDoneRequest"
        );
        let _initialized = client.recv_until(|m| is_event(m, "initialized"));

        // --- setBreakpoints on the `.lua` source line ---
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": SCENARIO_SOURCE },
                "breakpoints": [{ "line": BREAKPOINT_LINE }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        assert_eq!(bp_resp["request_seq"], 2, "setBreakpoints response correlates");
        let bps = bp_resp["body"]["breakpoints"]
            .as_array()
            .expect("breakpoints array");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0]["verified"], true);
        assert_eq!(bps[0]["line"], BREAKPOINT_LINE);

        // --- configurationDone (ack) ---
        client.send_request(3, "configurationDone", json!({}));
        let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
        assert_eq!(cfg_resp["success"], true);
        assert_eq!(cfg_resp["request_seq"], 3);

        // Breakpoints are live + config done: let the VM run.
        go_tx.send(()).expect("send go signal");

        // --- the VM hits the breakpoint → `stopped` event ---
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "must stop with reason breakpoint at the coroutine-body BP"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        // --- threads → at least the main thread ---
        client.send_request(9, "threads", json!({}));
        let threads = client.recv_until(|m| is_response(m, "threads"));
        assert_eq!(threads["request_seq"], 9);
        let thread_arr = threads["body"]["threads"].as_array().expect("threads array");
        assert!(!thread_arr.is_empty(), "threads must report at least one thread");

        // --- stackTrace → frames (top frame is the coroutine body BP line) ---
        client.send_request(10, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        assert_eq!(stack["request_seq"], 10);
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have at least the stopped frame");
        assert_eq!(
            frames[0]["source"]["path"], SCENARIO_SOURCE,
            "top frame source must be the scenario `.lua`"
        );
        assert_eq!(
            frames[0]["line"], BREAKPOINT_LINE,
            "top frame line must be the breakpoint line"
        );

        // --- scopes → a `Locals` scope with a decodable variablesReference ---
        let frame_id = frames[0]["id"].as_u64().expect("frame id");
        client.send_request(11, "scopes", json!({ "frameId": frame_id }));
        let scopes = client.recv_until(|m| is_response(m, "scopes"));
        assert_eq!(scopes["request_seq"], 11);
        let scope_arr = scopes["body"]["scopes"].as_array().expect("scopes array");
        assert_eq!(scope_arr.len(), 1, "exactly one scopes response (no double-answer)");
        assert_eq!(scope_arr[0]["name"], "Locals");
        let var_ref = scope_arr[0]["variablesReference"]
            .as_u64()
            .expect("variablesReference");
        assert!(var_ref != 0, "variablesReference must be non-zero");

        // --- variables → the coroutine-body local `co_local` (= 7) ---
        client.send_request(12, "variables", json!({ "variablesReference": var_ref }));
        let vars = client.recv_until(|m| is_response(m, "variables"));
        assert_eq!(vars["request_seq"], 12);
        let var_arr = vars["body"]["variables"].as_array().expect("variables array");
        let co_local = var_arr
            .iter()
            .find(|v| v["name"] == "co_local")
            .unwrap_or_else(|| panic!("coroutine-body local `co_local` must be present: {var_arr:?}"));
        assert_eq!(co_local["type"], "number");
        assert_eq!(co_local["value"], "7", "co_local must read its live value 7");

        // --- step over (`next`) → ack + a new `stopped(step)` ---
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let next_ack = client.recv_until(|m| is_response(m, "next"));
        assert_eq!(next_ack["request_seq"], 20);
        let step_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            step_stopped["body"]["reason"], "step",
            "after `next` the VM must re-stop with reason step"
        );

        // --- continue → run to completion ---
        client.send_request(30, "continue", json!({ "threadId": thread_id }));
        let cont_ack = client.recv_until(|m| is_response(m, "continue"));
        assert_eq!(cont_ack["request_seq"], 30);

        // The scenario runs to completion: the host VM thread finishes (drops the
        // handle → shutdown). We assert the host completed within the watchdog;
        // the chunk completing (not a disconnect) is the natural session end.
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion after continue");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    // =======================================================================
    // Task 8.1 — COMPREHENSIVE Lua-level debug session E2E (the full
    // requirement matrix in ONE cohesive DAP-over-TCP session).
    //
    // Where the 4.1 headline test above proves the path is wired end-to-end,
    // THIS test exhaustively exercises the user-facing Lua debug feature set in
    // a single session and asserts the EXACT DAP responses/events at each stage
    // so a regression in ANY layer (transport / dap / session / step / inspect)
    // fails it. It maps to:
    //   R1.1/1.2  BP set on a `.lua` line + hit → `stopped(breakpoint)`
    //   R1.3      step over (`next`)  → exact next `.lua` line
    //   R1.4      step into (`stepIn`)→ a callee's first body line
    //   R1.5      step out (`stepOut`)→ back in the caller, past the call
    //   R1.6      continue → run to completion
    //   R1.7      the BP + all steps fire ACROSS a scene coroutine body
    //   R2.1      stackTrace frames carry the `.lua` source + line
    //   R2.2/2.3  variables expose number / string / boolean / table by name+type
    //   R2.4      a coroutine-BODY-frame local is inspectable
    //   R2.5      unsupported kinds (function / nil) are surfaced gracefully,
    //             the request does NOT error, and the VM stays usable
    //   R3.4/3.5  stopped + terminated events over the wire
    //   R3.6      the DAP-over-TCP attach IS the VSCode-equivalent client target
    //             (the VSCode factory returns the same DebugAdapterServer; the
    //             config-factory wiring itself is covered by task 6.1)
    //
    // `mlua::Lua` (!Send) is built and owned solely on the VM host thread; only
    // the bound `SocketAddr` (Copy) and the go/done channels cross. All client
    // waits are bounded by the TEST-ONLY [`WATCHDOG`] so CI cannot hang; the
    // stop core stays unbounded.
    // =======================================================================

    /// Comprehensive scenario source name and breakpoint line.
    const FULL_SOURCE: &str = "@e2e_full_scenario";

    /// The comprehensive scenario chunk. A `helper` callee (for step into/out)
    /// plus a coroutine BODY (so the BP + steps cross a scene coroutine, R1.7)
    /// whose frame holds ALL basic variable types AND unsupported kinds.
    ///
    /// The breakpoint sits on a line where every local declared above it is a
    /// live, NAMED slot (a local on its OWN declaration line is still an unnamed
    /// `(*temporary)` slot, so the BP is placed AFTER all the declarations).
    ///
    /// Lines (1-origin):
    ///   1: local function helper(x)
    ///   2:     local hv = x + 1          <- step INTO target (helper body line)
    ///   3:     return hv
    ///   4: end
    ///   5: local body = function()
    ///   6:     local num = 7             -- number   (R2.2/R2.3)
    ///   7:     local str = 'hi'          -- string   (R2.2/R2.3)
    ///   8:     local flag = true         -- boolean  (R2.2/R2.3)
    ///   9:     local tbl = { 1, 2, 3 }   -- table    (R2.2/R2.3)
    ///  10:     local fn = helper         -- function (UNSUPPORTED, R2.5)
    ///  11:     local nilv = nil          -- nil      (UNSUPPORTED, R2.5)
    ///  12:     local marker = num        <- BREAKPOINT (6..=11 all live here)
    ///  13:     local doubled = helper(marker)  <- step OVER lands here; step
    ///                                             INTO from here enters helper
    ///  14:     coroutine.yield()         <- step OUT (from helper) lands here
    ///  15:     return doubled
    ///  16: end
    ///  17: local co = coroutine.create(body)
    ///  18: while coroutine.status(co) ~= 'dead' do
    ///  19:     coroutine.resume(co)
    ///  20: end
    const FULL_CHUNK: &str = "\
local function helper(x)
    local hv = x + 1
    return hv
end
local body = function()
    local num = 7
    local str = 'hi'
    local flag = true
    local tbl = { 1, 2, 3 }
    local fn = helper
    local nilv = nil
    local marker = num
    local doubled = helper(marker)
    coroutine.yield()
    return doubled
end
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
";
    /// Stop lines for the comprehensive scenario (1-origin, see [`FULL_CHUNK`]).
    const FULL_BP_LINE: u32 = 12; // `local marker = num` (all locals live).
    const FULL_STEP_OVER_LINE: u32 = 13; // same frame, next line after the BP.
    const FULL_STEP_IN_LINE: u32 = 2; // helper's first body line.
    const FULL_STEP_OUT_LINE: u32 = 14; // back in the body, past the helper call.

    /// The comprehensive task-8.1 E2E: ONE DAP-over-TCP session driving the full
    /// Lua-level debug feature matrix (R1.1–1.7, R2.1–2.5, R3.4/3.5, R3.6) with
    /// EXACT assertions at every stage. See the section comment above for the
    /// per-stage requirement mapping.
    #[test]
    fn full_lua_debug_session_all_steps_all_var_types_coroutine_body() {
        let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
        let (go_tx, go_rx) = mpsc::channel::<()>();

        // VM HOST THREAD: owns `mlua::Lua` (!Send) for its whole lifetime.
        let host = std::thread::spawn(move || -> Result<(), String> {
            let lua = unsafe {
                mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            };

            let cfg = DebugConfig {
                enabled: true,
                listen: Some("127.0.0.1:0".parse().unwrap()),
                source_map_slice: false,
            };
            let handle = enable(&lua, &cfg)
                .map_err(|e| format!("enable failed: {e}"))?
                .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

            let addr = handle
                .local_addr()
                .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "did not receive go signal before running the VM".to_string())?;

            // Run the scenario: this blocks at the breakpoint, then is driven by
            // the client over the wire (inspect/step processed in the hook loop
            // ON THIS THREAD). It runs through the coroutine yield/resume on
            // `continue` and returns when the coroutine is dead.
            lua.load(FULL_CHUNK)
                .set_name(FULL_SOURCE)
                .exec()
                .map_err(|e| format!("scenario exec failed: {e}"))?;

            // Prove the VM is still usable after the whole debug session (no
            // stack corruption from any inspect; R2.5 "VM stays usable").
            let sane: i64 = lua
                .load("return 1 + 2")
                .eval()
                .map_err(|e| format!("post-session VM eval failed: {e}"))?;
            if sane != 3 {
                return Err(format!("VM stack corrupted after session: 1+2 = {sane}"));
            }

            lua.remove_global_hook();
            drop(handle);
            Ok(())
        });

        // CLIENT (this thread) — the VSCode-equivalent DAP-over-TCP client (R3.6).
        let addr = addr_rx
            .recv_timeout(WATCHDOG)
            .expect("host must publish the bound addr before the watchdog");
        let mut client = DapClient::connect(addr);

        // --- initialize → capabilities + `initialized` (R3.2 handshake) ---
        client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
        let init_resp = client.recv_until(|m| is_response(m, "initialize"));
        assert_eq!(init_resp["success"], true, "initialize must succeed");
        assert_eq!(init_resp["request_seq"], 1);
        assert_eq!(
            init_resp["body"]["supportsConfigurationDoneRequest"], true,
            "initialize must advertise supportsConfigurationDoneRequest"
        );
        let _initialized = client.recv_until(|m| is_event(m, "initialized"));

        // --- setBreakpoints on the `.lua` source line (R1.1) ---
        client.send_request(
            2,
            "setBreakpoints",
            json!({
                "source": { "path": FULL_SOURCE },
                "breakpoints": [{ "line": FULL_BP_LINE }],
            }),
        );
        let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
        assert_eq!(bp_resp["request_seq"], 2);
        let bps = bp_resp["body"]["breakpoints"].as_array().expect("bp array");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0]["verified"], true, "the `.lua` BP must be verified (R1.1)");
        assert_eq!(bps[0]["line"], FULL_BP_LINE);

        // --- configurationDone (ack) → let the VM run ---
        client.send_request(3, "configurationDone", json!({}));
        let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
        assert_eq!(cfg_resp["success"], true);
        assert_eq!(cfg_resp["request_seq"], 3);
        go_tx.send(()).expect("send go signal");

        // --- hit the breakpoint inside the coroutine body → `stopped` (R1.2,
        //     R1.7, R3.4) ---
        let stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            stopped["body"]["reason"], "breakpoint",
            "must stop with reason breakpoint at the coroutine-body BP (R1.2/R3.4)"
        );
        let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

        // --- threads (R3.3) ---
        client.send_request(10, "threads", json!({}));
        let threads = client.recv_until(|m| is_response(m, "threads"));
        assert_eq!(threads["request_seq"], 10);
        let thread_arr = threads["body"]["threads"].as_array().expect("threads array");
        assert!(!thread_arr.is_empty(), "threads must report at least one thread");

        // --- stackTrace → top frame is the coroutine body BP line (R2.1) ---
        client.send_request(11, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        assert_eq!(stack["request_seq"], 11);
        let frames = stack["body"]["stackFrames"].as_array().expect("frames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame (R2.1)");
        assert_eq!(
            frames[0]["source"]["path"], FULL_SOURCE,
            "top frame source must be the scenario `.lua` (R2.1)"
        );
        assert_eq!(
            frames[0]["line"], FULL_BP_LINE,
            "top frame line must be the breakpoint line (R2.1)"
        );
        let frame_id = frames[0]["id"].as_u64().expect("frame id");

        // --- scopes → exactly one `Locals` scope (no double-answer) ---
        client.send_request(12, "scopes", json!({ "frameId": frame_id }));
        let scopes = client.recv_until(|m| is_response(m, "scopes"));
        assert_eq!(scopes["request_seq"], 12);
        let scope_arr = scopes["body"]["scopes"].as_array().expect("scopes array");
        assert_eq!(scope_arr.len(), 1, "exactly one scopes response (no double-answer)");
        assert_eq!(scope_arr[0]["name"], "Locals");
        let var_ref = scope_arr[0]["variablesReference"]
            .as_u64()
            .expect("variablesReference");
        assert_ne!(var_ref, 0, "variablesReference must be non-zero");

        // --- variables → ALL basic types by name+type+value, the coroutine-body
        //     local (R2.2/R2.3/R2.4), AND the unsupported kinds surfaced
        //     gracefully without an error response (R2.5) ---
        client.send_request(13, "variables", json!({ "variablesReference": var_ref }));
        let vars = client.recv_until(|m| is_response(m, "variables"));
        assert_eq!(vars["request_seq"], 13);
        assert_eq!(vars["success"], true, "variables must not error (R2.5)");
        let var_arr = vars["body"]["variables"].as_array().expect("variables array");

        let find = |name: &str| -> Value {
            var_arr
                .iter()
                .find(|v| v["name"] == name)
                .unwrap_or_else(|| panic!("coroutine-body local `{name}` must be present: {var_arr:?}"))
                .clone()
        };

        // number (R2.2/R2.3) — also THE coroutine-body local proof (R2.4).
        let num = find("num");
        assert_eq!(num["type"], "number", "num must be discriminated as number (R2.3)");
        assert_eq!(num["value"], "7", "num must read its live value 7 (R2.4)");
        // string
        let s = find("str");
        assert_eq!(s["type"], "string", "str must be discriminated as string (R2.3)");
        assert_eq!(s["value"], "hi", "str must read its live value 'hi'");
        // boolean
        let flag = find("flag");
        assert_eq!(flag["type"], "boolean", "flag must be a boolean (R2.3)");
        assert_eq!(flag["value"], "true", "flag must read its live value true");
        // table
        let tbl = find("tbl");
        assert_eq!(tbl["type"], "table", "tbl must be a table (R2.3)");
        assert!(
            tbl["value"].as_str().unwrap().starts_with("table:"),
            "table value must be a readable placeholder: {:?}",
            tbl["value"]
        );

        // R2.5: an UNSUPPORTED kind (function) is RECORDED gracefully — present
        // by name, type surfaced, repr marked unsupported — never dropped and
        // never erroring the request.
        let fnval = find("fn");
        assert_eq!(fnval["type"], "function", "unsupported kind type surfaced (R2.5)");
        assert!(
            fnval["value"].as_str().unwrap().starts_with("<unsupported"),
            "an unsupported kind must carry an out-of-scope repr (R2.5): {:?}",
            fnval["value"]
        );
        // nil is likewise surfaced gracefully.
        let nilv = find("nilv");
        assert_eq!(nilv["type"], "nil", "nil kind surfaced gracefully (R2.5)");
        assert!(
            nilv["value"].as_str().unwrap().starts_with("<unsupported"),
            "nil must carry an out-of-scope repr (R2.5): {:?}",
            nilv["value"]
        );

        // --- step OVER (`next`) → ack + stopped(step) at the next `.lua` line in
        //     the SAME frame, NOT inside helper (R1.3) ---
        client.send_request(20, "next", json!({ "threadId": thread_id }));
        let next_ack = client.recv_until(|m| is_response(m, "next"));
        assert_eq!(next_ack["request_seq"], 20);
        let over_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(
            over_stopped["body"]["reason"], "step",
            "step over must re-stop with reason step (R1.3)"
        );
        assert_eq!(
            top_frame_line(&mut client, thread_id, 21),
            FULL_STEP_OVER_LINE,
            "step over must stop at the next line in the SAME frame (R1.3), not inside helper"
        );

        // --- step INTO (`stepIn`) → ack + stopped(step) at helper's first body
        //     line (R1.4) ---
        client.send_request(30, "stepIn", json!({ "threadId": thread_id }));
        let in_ack = client.recv_until(|m| is_response(m, "stepIn"));
        assert_eq!(in_ack["request_seq"], 30);
        let in_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(in_stopped["body"]["reason"], "step", "step in reason step (R1.4)");
        assert_eq!(
            top_frame_line(&mut client, thread_id, 31),
            FULL_STEP_IN_LINE,
            "step in must stop at the callee's first body line (R1.4)"
        );

        // --- step OUT (`stepOut`) → ack + stopped(step) back in the caller body,
        //     past the helper call (R1.5) ---
        client.send_request(40, "stepOut", json!({ "threadId": thread_id }));
        let out_ack = client.recv_until(|m| is_response(m, "stepOut"));
        assert_eq!(out_ack["request_seq"], 40);
        let out_stopped = client.recv_until(|m| is_event(m, "stopped"));
        assert_eq!(out_stopped["body"]["reason"], "step", "step out reason step (R1.5)");
        assert_eq!(
            top_frame_line(&mut client, thread_id, 41),
            FULL_STEP_OUT_LINE,
            "step out must stop back in the caller body past the call (R1.5)"
        );

        // --- continue (R1.6) → the coroutine yields, the driver re-resumes, the
        //     body returns, the chunk completes → the host VM thread finishes ---
        client.send_request(50, "continue", json!({ "threadId": thread_id }));
        let cont_ack = client.recv_until(|m| is_response(m, "continue"));
        assert_eq!(cont_ack["request_seq"], 50);
        assert_eq!(cont_ack["body"]["allThreadsContinued"], true);

        // The scenario runs to completion (R1.6): the host VM thread finishes and
        // drops the handle → Drop emits a final `Terminated`, which the encoder
        // turns into a DAP `terminated` event flushed to us (R3.5).
        let terminated = client.recv_until(|m| is_event(m, "terminated"));
        assert_eq!(terminated["event"], "terminated", "natural end emits terminated (R3.5)");

        // The host completed cleanly within the watchdog (no hang) and the VM
        // stayed usable (1+2==3 asserted on the host).
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(host.join());
        });
        match done_rx.recv_timeout(WATCHDOG) {
            Ok(joined) => {
                joined
                    .expect("host VM thread must not panic")
                    .expect("scenario must run to completion after continue (R1.6)");
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("host VM thread did not finish within the watchdog (hang?)");
            }
            Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
        }
    }

    /// Request a `stackTrace` for `thread_id` (correlated to `seq`) and return
    /// the top frame's reported line. A small client-side helper so each step's
    /// EXACT stop line is asserted over the wire (DAP reports the position via
    /// `stackTrace`, not in the `stopped` event body).
    fn top_frame_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
        client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
        let stack = client.recv_until(|m| is_response(m, "stackTrace"));
        let frames = stack["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames array");
        assert!(!frames.is_empty(), "stack must have the stopped frame");
        frames[0]["line"].as_u64().expect("top frame line") as u32
    }
}
