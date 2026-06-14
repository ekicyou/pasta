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


#[cfg(test)]
#[path = "wiring_tests_a.rs"]
mod a;

#[cfg(test)]
#[path = "wiring_tests_b.rs"]
mod b;
