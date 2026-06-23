use super::*;
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
            ..Default::default()
        };
        let handle = enable(&lua, &cfg, None, None)
            .map_err(|e| format!("enable failed: {e}"))?
            .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

        let addr = handle
            .local_addr()
            .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
        addr_tx
            .send(addr)
            .map_err(|_| "addr send failed".to_string())?;

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
    assert_eq!(
        bps[0]["verified"], true,
        "the `.lua` BP must be verified (R1.1)"
    );
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
    let thread_arr = threads["body"]["threads"]
        .as_array()
        .expect("threads array");
    assert!(
        !thread_arr.is_empty(),
        "threads must report at least one thread"
    );

    // --- stackTrace → top frame is the coroutine body BP line (R2.1) ---
    client.send_request(11, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    assert_eq!(stack["request_seq"], 11);
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("frames array");
    assert!(
        !frames.is_empty(),
        "stack must have the stopped frame (R2.1)"
    );
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
    assert_eq!(
        scope_arr.len(),
        1,
        "exactly one scopes response (no double-answer)"
    );
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
    let var_arr = vars["body"]["variables"]
        .as_array()
        .expect("variables array");

    let find = |name: &str| -> Value {
        var_arr
            .iter()
            .find(|v| v["name"] == name)
            .unwrap_or_else(|| panic!("coroutine-body local `{name}` must be present: {var_arr:?}"))
            .clone()
    };

    // number (R2.2/R2.3) — also THE coroutine-body local proof (R2.4).
    let num = find("num");
    assert_eq!(
        num["type"], "number",
        "num must be discriminated as number (R2.3)"
    );
    assert_eq!(num["value"], "7", "num must read its live value 7 (R2.4)");
    // string
    let s = find("str");
    assert_eq!(
        s["type"], "string",
        "str must be discriminated as string (R2.3)"
    );
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
    assert_eq!(
        fnval["type"], "function",
        "unsupported kind type surfaced (R2.5)"
    );
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
    assert_eq!(
        in_stopped["body"]["reason"], "step",
        "step in reason step (R1.4)"
    );
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
    assert_eq!(
        out_stopped["body"]["reason"], "step",
        "step out reason step (R1.5)"
    );
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
    assert_eq!(
        terminated["event"], "terminated",
        "natural end emits terminated (R3.5)"
    );

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
