use super::*;
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
            ..Default::default()
        };
        let handle = enable(&lua, &cfg, None)
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

