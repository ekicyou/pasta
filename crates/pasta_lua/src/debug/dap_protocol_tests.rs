//! Inline test cluster externalized from `dap.rs` (Task 2.2, pure
//! behavior-invariant move). Cluster: core DAP protocol decode/encode —
//! request decoding, deferred-response correlation, event encoding, and the
//! outgoing-`seq` envelope invariants.
use super::*;
use super::dap_test_support::*;

use crate::debug::types::FrameInfo;

// --- initialize (R3.2) -------------------------------------------------

#[test]
fn initialize_advertises_capabilities_and_emits_initialized() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(1, "initialize", json!({ "adapterID": "pasta" })));

    // No session command for initialize.
    assert_eq!(decoded.command, None);

    let resp = decoded.response.expect("initialize must produce a response");
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "initialize");
    assert_eq!(resp["request_seq"], 1);
    assert_eq!(resp["success"], true);
    assert_eq!(
        resp["body"]["supportsConfigurationDoneRequest"], true,
        "R3.2: initialize must advertise supportsConfigurationDoneRequest"
    );

    // The standard handshake emits an `initialized` event after the response.
    assert_eq!(decoded.events.len(), 1, "initialize emits one event");
    let ev = &decoded.events[0];
    assert_eq!(ev["type"], "event");
    assert_eq!(ev["event"], "initialized");

    // Outgoing seq is monotonic: response seq=1, event seq=2.
    assert_eq!(resp["seq"], 1);
    assert_eq!(ev["seq"], 2);
}

// --- setBreakpoints (R3.3) ---------------------------------------------

#[test]
fn set_breakpoints_decodes_command_and_correlates_response() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        5,
        "setBreakpoints",
        json!({
            "source": { "path": "@scene.lua" },
            "breakpoints": [{ "line": 3 }, { "line": 7 }],
        }),
    ));

    assert_eq!(
        decoded.command,
        Some(SessionCommand::SetBreakpoints {
            source: SourceRef::new("@scene.lua"),
            lines: vec![3, 7],
        }),
        "R3.3: setBreakpoints → SetBreakpoints{{source,lines}}"
    );
    // setBreakpoints is deferred — no immediate response.
    assert!(decoded.response.is_none());

    // The corresponding SessionEvent::Breakpoints produces the response,
    // correlated to request_seq=5.
    let out = dap.encode_event(SessionEvent::Breakpoints(vec![
        ResolvedBreakpoint {
            source: SourceRef::new("@scene.lua"),
            line: 3,
            verified: true,
        },
        ResolvedBreakpoint {
            source: SourceRef::new("@scene.lua"),
            line: 7,
            verified: false,
        },
    ]));
    assert_eq!(out.len(), 1);
    let resp = &out[0];
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "setBreakpoints");
    assert_eq!(resp["request_seq"], 5, "deferred response carries originating seq");
    assert_eq!(resp["success"], true);
    let bps = resp["body"]["breakpoints"].as_array().expect("breakpoints array");
    assert_eq!(bps.len(), 2);
    assert_eq!(bps[0]["verified"], true);
    assert_eq!(bps[0]["line"], 3);
    assert_eq!(bps[1]["verified"], false);
    assert_eq!(bps[1]["line"], 7);
}

// --- configurationDone (R3.3) ------------------------------------------

#[test]
fn configuration_done_acks_without_command() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(2, "configurationDone", json!({})));
    assert_eq!(decoded.command, None);
    let resp = decoded.response.expect("ack response");
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "configurationDone");
    assert_eq!(resp["request_seq"], 2);
    assert_eq!(resp["success"], true);
    assert!(resp.get("body").is_none(), "ack has no body");
}

// --- threads (R3.3) ----------------------------------------------------

#[test]
fn threads_decodes_command_and_correlates_response() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(8, "threads", json!({})));
    assert_eq!(decoded.command, Some(SessionCommand::Threads));
    assert!(decoded.response.is_none());

    let out = dap.encode_event(SessionEvent::Threads(vec![ThreadInfo {
        id: 1,
        name: "main".to_string(),
    }]));
    assert_eq!(out.len(), 1);
    let resp = &out[0];
    assert_eq!(resp["command"], "threads");
    assert_eq!(resp["request_seq"], 8);
    let threads = resp["body"]["threads"].as_array().expect("threads array");
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0]["id"], 1);
    assert_eq!(threads[0]["name"], "main");
}

// --- stackTrace (R3.3) -------------------------------------------------

#[test]
fn stack_trace_decodes_command_and_correlates_response() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));
    assert_eq!(decoded.command, Some(SessionCommand::StackTrace));
    assert!(decoded.response.is_none());

    let out = dap.encode_event(SessionEvent::Stack(vec![
        FrameInfo {
            source: "@scene.lua".to_string(),
            line: 7,
            func_name: Some("talk".to_string()),
        },
        FrameInfo {
            source: "@scene.lua".to_string(),
            line: 2,
            func_name: None,
        },
    ]));
    assert_eq!(out.len(), 1);
    let resp = &out[0];
    assert_eq!(resp["command"], "stackTrace");
    assert_eq!(resp["request_seq"], 11);
    assert_eq!(resp["body"]["totalFrames"], 2);
    let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0]["id"], 0, "frame id = stack index");
    assert_eq!(frames[0]["name"], "talk");
    assert_eq!(frames[0]["source"]["path"], "@scene.lua");
    assert_eq!(frames[0]["line"], 7);
    assert_eq!(frames[0]["column"], 1);
    assert_eq!(frames[1]["id"], 1);
    assert_eq!(frames[1]["name"], "?", "missing func name → placeholder");
}

// --- scopes (R3.3) -----------------------------------------------------

#[test]
fn scopes_immediately_returns_locals_scope_with_decodable_ref() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(13, "scopes", json!({ "frameId": 2 })));
    assert_eq!(
        decoded.command,
        Some(SessionCommand::Scopes { frame_id: 2 }),
        "scopes → Scopes{{frame_id}}"
    );
    let resp = decoded.response.expect("scopes answered immediately");
    assert_eq!(resp["command"], "scopes");
    assert_eq!(resp["request_seq"], 13);
    let scopes = resp["body"]["scopes"].as_array().expect("scopes array");
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0]["name"], "Locals");
    // variablesReference = frameId + 1 (non-zero, decodable back to frame 2).
    assert_eq!(scopes[0]["variablesReference"], 3);
}

// --- variables (R3.3) --------------------------------------------------

#[test]
fn variables_decodes_command_and_maps_fields() {
    let mut dap = DapAdapter::new();
    // A scopes for frame 2 yields variablesReference 3; the client passes it
    // back in a variables request.
    let decoded = dap.decode_request(&request(15, "variables", json!({ "variablesReference": 3 })));
    assert_eq!(
        decoded.command,
        Some(SessionCommand::Variables { var_ref: 3 }),
        "variables → Variables{{var_ref}}"
    );
    assert!(decoded.response.is_none());

    let out = dap.encode_event(SessionEvent::Variables(vec![
        Variable {
            name: "x".to_string(),
            type_name: "number".to_string(),
            repr: "42".to_string(),
        },
        Variable {
            name: "s".to_string(),
            type_name: "string".to_string(),
            repr: "\"hi\"".to_string(),
        },
    ]));
    assert_eq!(out.len(), 1);
    let resp = &out[0];
    assert_eq!(resp["command"], "variables");
    assert_eq!(resp["request_seq"], 15);
    let vars = resp["body"]["variables"].as_array().expect("variables array");
    assert_eq!(vars.len(), 2);
    // repr → value, type_name → type, leaf ref = 0.
    assert_eq!(vars[0]["name"], "x");
    assert_eq!(vars[0]["value"], "42");
    assert_eq!(vars[0]["type"], "number");
    assert_eq!(vars[0]["variablesReference"], 0);
    assert_eq!(vars[1]["name"], "s");
    assert_eq!(vars[1]["value"], "\"hi\"");
    assert_eq!(vars[1]["type"], "string");
}

// --- continue / next / stepIn / stepOut (R3.3) -------------------------

#[test]
fn continue_acks_and_forwards_command() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(20, "continue", json!({ "threadId": 1 })));
    assert_eq!(decoded.command, Some(SessionCommand::Continue));
    let resp = decoded.response.expect("continue acks");
    assert_eq!(resp["command"], "continue");
    assert_eq!(resp["request_seq"], 20);
    assert_eq!(resp["body"]["allThreadsContinued"], true);
}

#[test]
fn step_commands_ack_and_forward() {
    for (command, expected) in [
        ("next", SessionCommand::Next),
        ("stepIn", SessionCommand::StepIn),
        ("stepOut", SessionCommand::StepOut),
    ] {
        let mut dap = DapAdapter::new();
        let decoded = dap.decode_request(&request(30, command, json!({ "threadId": 1 })));
        assert_eq!(decoded.command, Some(expected), "{command} → step command");
        let resp = decoded.response.expect("step acks");
        assert_eq!(resp["command"], command);
        assert_eq!(resp["request_seq"], 30);
        assert_eq!(resp["success"], true);
        assert!(resp.get("body").is_none(), "step ack has no body");
    }
}

// --- disconnect (R3.3 / R3.5) ------------------------------------------

#[test]
fn disconnect_acks_forwards_and_later_terminates() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(40, "disconnect", json!({})));
    assert_eq!(decoded.command, Some(SessionCommand::Disconnect));
    let resp = decoded.response.expect("disconnect acks");
    assert_eq!(resp["command"], "disconnect");
    assert_eq!(resp["request_seq"], 40);

    // The later Terminated event maps to a `terminated` DAP event (R3.5).
    let out = dap.encode_event(SessionEvent::Terminated);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["type"], "event");
    assert_eq!(out[0]["event"], "terminated");
}

// --- stopped event (R3.4) ----------------------------------------------

#[test]
fn stopped_event_maps_each_reason_and_thread() {
    for (reason, expected) in [
        (StopReason::Breakpoint, "breakpoint"),
        (StopReason::Step, "step"),
        (StopReason::Entry, "entry"),
        (StopReason::Pause, "pause"),
    ] {
        let mut dap = DapAdapter::new();
        let out = dap.encode_event(SessionEvent::Stopped {
            reason,
            thread_id: 1,
        });
        assert_eq!(out.len(), 1);
        let ev = &out[0];
        assert_eq!(ev["type"], "event");
        assert_eq!(ev["event"], "stopped");
        assert_eq!(ev["body"]["reason"], expected, "R3.4: reason mapping");
        assert_eq!(ev["body"]["threadId"], 1);
    }
}

/// Spike (Task 1.1, pasta-debug-lua-view-toggle): adapter-layer proof that a
/// SECOND `stopped` event can be emitted MID-PAUSE on the same adapter and a
/// subsequent `stackTrace` request is still served — i.e. the
/// transport/adapter has NO single-shot guard on `stopped` and does NOT
/// require a resume between two stops. This underpins the R3.3 redraw design
/// (停止中の `stopped` 再送 → クライアントが stackTrace を再フェッチ): the
/// session-side `RefreshPresentation` handler will resend `Stopped { reason,
/// thread_id }` and the adapter encodes it identically every time.
///
/// This is the adapter-level slice of the spike; the VM-thread-gated
/// "停止中のみ再送" judgement lives in `session.rs` (unbuilt here, see
/// research.md). The DAP-client refetch semantics themselves are confirmed by
/// the official DAP overview / VSCode docs (research.md "Spike 結果").
#[test]
fn stopped_can_be_resent_midpause_and_stacktrace_still_served() {
    let mut dap = DapAdapter::new();

    // First stop (e.g. breakpoint hit) — a normal `stopped` event.
    let first = dap.encode_event(SessionEvent::Stopped {
        reason: StopReason::Breakpoint,
        thread_id: 1,
    });
    assert_eq!(first.len(), 1);
    assert_eq!(first[0]["event"], "stopped");
    assert_eq!(first[0]["body"]["reason"], "breakpoint");

    // No `continue`/resume happens here. A client `stackTrace` arrives and is
    // accepted (deferred until the session replies with Stack).
    let st1 = dap.decode_request(&request(50, "stackTrace", json!({ "threadId": 1 })));
    assert_eq!(st1.command, Some(SessionCommand::StackTrace));
    let stack1 = dap.encode_event(SessionEvent::Stack(vec![]));
    assert_eq!(stack1[0]["command"], "stackTrace");
    assert_eq!(stack1[0]["request_seq"], 50, "first stackTrace correlates");

    // RE-SEND `stopped` WHILE STILL PAUSED (the redraw trigger). The adapter
    // emits it again with no error and no single-shot guard. This is exactly
    // what the session-side RefreshPresentation will drive.
    let resent = dap.encode_event(SessionEvent::Stopped {
        reason: StopReason::Breakpoint,
        thread_id: 1,
    });
    assert_eq!(resent.len(), 1, "a re-sent stopped is emitted again");
    assert_eq!(resent[0]["event"], "stopped");
    assert_eq!(resent[0]["body"]["reason"], "breakpoint");
    assert_eq!(resent[0]["body"]["threadId"], 1);
    assert_eq!(resent[0]["body"]["allThreadsStopped"], true);
    // The re-sent event is a fresh, monotonically-sequenced frame (not a
    // replay of the first) — the client treats it as a new stop and refetches.
    assert!(
        resent[0]["seq"].as_u64().unwrap() > first[0]["seq"].as_u64().unwrap(),
        "re-sent stopped carries a new monotonic seq"
    );

    // After the re-send the client refetches the stack; the adapter serves it
    // again identically (proving the refetch loop the design relies on).
    let st2 = dap.decode_request(&request(51, "stackTrace", json!({ "threadId": 1 })));
    assert_eq!(st2.command, Some(SessionCommand::StackTrace));
    let stack2 = dap.encode_event(SessionEvent::Stack(vec![]));
    assert_eq!(stack2[0]["command"], "stackTrace");
    assert_eq!(stack2[0]["request_seq"], 51, "second stackTrace correlates");
}

// --- terminated event (R3.5) -------------------------------------------

#[test]
fn terminated_event_encoded() {
    let mut dap = DapAdapter::new();
    let out = dap.encode_event(SessionEvent::Terminated);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["type"], "event");
    assert_eq!(out[0]["event"], "terminated");
}

// --- error event -------------------------------------------------------

#[test]
fn error_maps_to_output_event() {
    let mut dap = DapAdapter::new();
    let out = dap.encode_event(SessionEvent::Error("lua boom".to_string()));
    assert_eq!(out.len(), 1);
    let ev = &out[0];
    assert_eq!(ev["type"], "event");
    assert_eq!(ev["event"], "output");
    assert_eq!(ev["body"]["category"], "stderr");
    assert!(
        ev["body"]["output"].as_str().unwrap().contains("lua boom"),
        "error message surfaced in output event"
    );
}

// --- envelope / seq invariants -----------------------------------------

#[test]
fn outgoing_seq_is_monotonic_across_responses_and_events() {
    let mut dap = DapAdapter::new();
    // initialize → response (seq 1) + initialized event (seq 2).
    let init = dap.decode_request(&request(1, "initialize", json!({})));
    assert_eq!(init.response.unwrap()["seq"], 1);
    assert_eq!(init.events[0]["seq"], 2);
    // A stopped event (seq 3).
    let stopped = dap.encode_event(SessionEvent::Stopped {
        reason: StopReason::Breakpoint,
        thread_id: 1,
    });
    assert_eq!(stopped[0]["seq"], 3);
    // configurationDone response (seq 4).
    let cfg = dap.decode_request(&request(2, "configurationDone", json!({})));
    assert_eq!(cfg.response.unwrap()["seq"], 4);
}

#[test]
fn deferred_responses_correlate_in_fifo_order_per_kind() {
    let mut dap = DapAdapter::new();
    // Two stackTrace requests in flight; FIFO pairs each Stack event back.
    dap.decode_request(&request(100, "stackTrace", json!({})));
    dap.decode_request(&request(101, "stackTrace", json!({})));

    let first = dap.encode_event(SessionEvent::Stack(vec![]));
    assert_eq!(first[0]["request_seq"], 100, "first event pairs to first request");
    let second = dap.encode_event(SessionEvent::Stack(vec![]));
    assert_eq!(second[0]["request_seq"], 101, "second event pairs to second request");
}

#[test]
fn unknown_request_is_ignored() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(99, "evaluate", json!({})));
    assert_eq!(decoded, Decoded::default(), "unknown command yields empty decode");
}
