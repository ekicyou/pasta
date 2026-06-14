//! Inline test cluster externalized from `dap.rs` (Task 2.2, pure
//! behavior-invariant move). Cluster: malformed-request graceful degradation
//! and pending-table edge behaviour (spurious/out-of-band events, per-kind
//! FIFO independence, wire no-ops).
use super::*;
use super::dap_test_support::*;

use crate::debug::types::Scope;

// --- malformed-request degradation (documented graceful fallbacks) ------

/// `parse_set_breakpoints` degradation: a `setBreakpoints` whose arguments
/// lack the `source.path` decodes to an EMPTY path (not a panic, not a
/// dropped command), and breakpoint entries without a numeric `line` are
/// SKIPPED while well-formed siblings survive (documented "missing pieces
/// degrade gracefully").
#[test]
fn set_breakpoints_malformed_args_degrade_gracefully() {
    let mut dap = DapAdapter::new();
    // No `source` at all + a mix of well-formed and malformed entries.
    let decoded = dap.decode_request(&request(
        9,
        "setBreakpoints",
        json!({
            "breakpoints": [
                { "line": 4 },
                { "noline": true },        // no `line` key → skipped
                { "line": "seven" },        // non-numeric line → skipped
                { "line": 9 },
            ],
        }),
    ));
    assert_eq!(
        decoded.command,
        Some(SessionCommand::SetBreakpoints {
            source: SourceRef::new(""),
            lines: vec![4, 9],
        }),
        "missing source.path → empty path; malformed entries skipped, \
         well-formed lines preserved in order"
    );
}

/// Envelope degradation: a request with a MISSING (or non-numeric) `seq`
/// falls back to `request_seq = 0` in the correlated response rather than
/// failing to decode.
#[test]
fn missing_or_non_numeric_seq_falls_back_to_zero() {
    let mut dap = DapAdapter::new();
    // No `seq` key at all.
    let decoded = dap.decode_request(&json!({
        "type": "request",
        "command": "configurationDone",
    }));
    let resp = decoded.response.expect("still acks without a seq");
    assert_eq!(resp["request_seq"], 0, "missing seq → request_seq 0");

    // A non-numeric seq is treated the same way.
    let decoded2 = dap.decode_request(&json!({
        "seq": "twelve",
        "type": "request",
        "command": "configurationDone",
    }));
    assert_eq!(
        decoded2.response.expect("acks")["request_seq"],
        0,
        "non-numeric seq → request_seq 0"
    );
}

/// `scopes` without a `frameId` defaults to frame 0, so the synthetic
/// `Locals` scope reports `variablesReference = 0 + 1 = 1` (still non-zero,
/// still decodable).
#[test]
fn scopes_missing_frame_id_defaults_to_frame_zero() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(14, "scopes", json!({})));
    assert_eq!(decoded.command, Some(SessionCommand::Scopes { frame_id: 0 }));
    let resp = decoded.response.expect("scopes answered immediately");
    let scopes = resp["body"]["scopes"].as_array().expect("scopes array");
    assert_eq!(
        scopes[0]["variablesReference"], 1,
        "missing frameId → frame 0 → variablesReference 1 (non-zero)"
    );
}

/// Hardening regression (trust boundary): a hostile or buggy client sending
/// `frameId: u32::MAX` must NOT panic the adapter (the synthetic scope's
/// `variablesReference = frame_id + 1` would overflow in debug builds).
/// The reference saturates at `u32::MAX` instead — still non-zero, and the
/// request is still answered immediately like any other `scopes`.
#[test]
fn scopes_huge_frame_id_does_not_overflow() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(17, "scopes", json!({ "frameId": u32::MAX })));
    assert_eq!(
        decoded.command,
        Some(SessionCommand::Scopes { frame_id: u32::MAX }),
        "the command still forwards the (extreme) frame id"
    );
    let resp = decoded.response.expect("scopes still answered immediately");
    let scopes = resp["body"]["scopes"].as_array().expect("scopes array");
    assert_eq!(
        scopes[0]["variablesReference"],
        u32::MAX,
        "variablesReference saturates at u32::MAX (non-zero, no overflow panic)"
    );
}

/// `variables` without a `variablesReference` degrades to `var_ref = 0`
/// and still forwards the command (the session side owns the decode).
#[test]
fn variables_missing_reference_defaults_to_zero() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(16, "variables", json!({})));
    assert_eq!(
        decoded.command,
        Some(SessionCommand::Variables { var_ref: 0 }),
        "missing variablesReference → var_ref 0"
    );
    assert!(decoded.response.is_none(), "variables stays deferred");
}

/// `attach` with a NON-STRING `sourcePresentation` (e.g. a number) yields
/// `None` — the value never reaches `SourceMode::parse`, so unlike an
/// invalid STRING (which falls back to `Pasta`, design 615) the resolved
/// env > file > 既定 mode is kept. This pins the documented type gate.
#[test]
fn attach_non_string_source_presentation_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        4,
        "attach",
        json!({ "sourcePresentation": 1 }),
    ));
    assert_eq!(
        decoded.attach_source_mode, None,
        "non-string sourcePresentation → None (no Pasta fallback)"
    );
    // The handshake still proceeds.
    assert!(decoded.response.is_some(), "attach still acks");
}

// --- pending-table edge behaviour ---------------------------------------

/// A deferred-kind event arriving with NO outstanding request (spurious /
/// out-of-band) is still encoded, correlated to the documented fallback
/// `request_seq = 0` instead of panicking or being dropped.
#[test]
fn spurious_deferred_event_correlates_to_request_seq_zero() {
    let mut dap = DapAdapter::new();
    // No threads request was ever decoded.
    let out = dap.encode_event(SessionEvent::Threads(vec![]));
    assert_eq!(out.len(), 1, "spurious event still produces a frame");
    assert_eq!(out[0]["type"], "response");
    assert_eq!(out[0]["command"], "threads");
    assert_eq!(out[0]["request_seq"], 0, "no pending request → fallback 0");
}

/// `SessionEvent::Scopes` is documented as a deliberate NO-OP on the wire
/// (the `scopes` response was already answered at decode time): encoding it
/// must yield zero frames and must not disturb the seq counter.
#[test]
fn scopes_event_is_a_wire_no_op() {
    let mut dap = DapAdapter::new();
    let out = dap.encode_event(SessionEvent::Scopes(vec![Scope {
        name: "Locals".to_string(),
        variables_reference: 1,
    }]));
    assert!(out.is_empty(), "Scopes event emits nothing on the wire");

    // The seq counter was not consumed: the next emission starts at 1.
    let ev = dap.encode_event(SessionEvent::Terminated);
    assert_eq!(ev[0]["seq"], 1, "no-op event must not consume a seq");
}

/// The pending FIFO is keyed PER KIND: with one `threads` and one
/// `stackTrace` outstanding, delivering the events in the OPPOSITE order
/// still pairs each response to its own request's seq (no cross-kind
/// stealing).
#[test]
fn pending_kinds_are_independent_fifos() {
    let mut dap = DapAdapter::new();
    dap.decode_request(&request(200, "threads", json!({})));
    dap.decode_request(&request(201, "stackTrace", json!({})));

    // Stack arrives FIRST even though threads was requested first.
    let stack = dap.encode_event(SessionEvent::Stack(vec![]));
    assert_eq!(stack[0]["command"], "stackTrace");
    assert_eq!(
        stack[0]["request_seq"], 201,
        "Stack pairs to the stackTrace request, not the older threads one"
    );

    let threads = dap.encode_event(SessionEvent::Threads(vec![]));
    assert_eq!(threads[0]["command"], "threads");
    assert_eq!(
        threads[0]["request_seq"], 200,
        "Threads still pairs to its own kind's pending seq"
    );
}
