//! Inline test cluster externalized from `dap.rs` (Task 2.2, pure
//! behavior-invariant move). Cluster: source-presentation negotiation — the
//! `attach` `sourcePresentation` parsing and the `pasta/sourcePresentation`
//! custom request/response/event handling.
use super::*;
use super::dap_test_support::*;

// --- attach `sourcePresentation` parsing (task 5.5 — R6.3 / design 581/586) ---

/// R6.3 / design 586: an `attach` request carrying an explicit
/// `sourcePresentation` ("lua"/"pasta") is parsed into
/// `Decoded.attach_source_mode` (highest precedence) and acked. The server
/// applies it to the session (resolver + step granularity) in the wiring.
#[test]
fn attach_parses_explicit_source_presentation() {
    for (raw, expected) in [("lua", SourceMode::Lua), ("pasta", SourceMode::Pasta)] {
        let mut dap = DapAdapter::new();
        let decoded = dap.decode_request(&request(
            3,
            "attach",
            json!({ "sourcePresentation": raw }),
        ));
        assert_eq!(
            decoded.attach_source_mode,
            Some(expected),
            "explicit sourcePresentation={raw:?} must parse to {expected:?} (R6.3)"
        );
        // attach is acked immediately (no session command).
        assert_eq!(decoded.command, None);
        let resp = decoded.response.expect("attach must ack");
        assert_eq!(resp["command"], "attach");
        assert_eq!(resp["request_seq"], 3);
        assert_eq!(resp["success"], true);
    }
}

/// design 581: an invalid `sourcePresentation` value still PARSES (the key is
/// present) but falls back to the default `pasta` (design 615) — it is NOT
/// `None` (the author DID specify presentation, just wrongly).
#[test]
fn attach_invalid_source_presentation_falls_back_to_pasta() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        3,
        "attach",
        json!({ "sourcePresentation": "garbage" }),
    ));
    assert_eq!(
        decoded.attach_source_mode,
        Some(SourceMode::Pasta),
        "invalid sourcePresentation → default pasta (design 615)"
    );
}

/// design 581 (NO client-default override): an `attach` WITHOUT
/// `sourcePresentation` leaves `attach_source_mode` `None`, so the server
/// keeps the resolved env > file > 既定 mode (a missing arg must NOT override).
#[test]
fn attach_without_source_presentation_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(3, "attach", json!({ "host": "127.0.0.1" })));
    assert_eq!(
        decoded.attach_source_mode, None,
        "absent sourcePresentation must NOT override the resolved mode (design 581)"
    );
    // Still acked so the client handshake proceeds.
    let resp = decoded.response.expect("attach must ack even without the arg");
    assert_eq!(resp["command"], "attach");
}

// --- pasta/sourcePresentation custom request (R1.1–R1.4) ---------------

/// R1.1: a `pasta/sourcePresentation` request with `mode: "lua"` decodes to a
/// `Decoded` carrying `Some(SourceMode::Lua)` as the requested runtime mode,
/// SEPARATE from `attach_source_mode` (which stays `None`). The request is
/// acked immediately so the client observes acceptance (R1.3).
#[test]
fn source_presentation_request_lua_decodes_runtime_mode() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        60,
        "pasta/sourcePresentation",
        json!({ "mode": "lua" }),
    ));
    assert_eq!(
        decoded.requested_source_mode,
        Some(SourceMode::Lua),
        "R1.1: mode=lua → Some(Lua) requested runtime mode"
    );
    // Runtime toggle does NOT populate the attach field (separate semantics).
    assert_eq!(
        decoded.attach_source_mode, None,
        "runtime toggle must not overload attach_source_mode"
    );
    // No session command is produced at decode time (wiring owns application).
    assert_eq!(decoded.command, None);
}

/// R1.2: a `pasta/sourcePresentation` request with `mode: "pasta"` decodes to
/// `Some(SourceMode::Pasta)`.
#[test]
fn source_presentation_request_pasta_decodes_runtime_mode() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        61,
        "pasta/sourcePresentation",
        json!({ "mode": "pasta" }),
    ));
    assert_eq!(
        decoded.requested_source_mode,
        Some(SourceMode::Pasta),
        "R1.2: mode=pasta → Some(Pasta) requested runtime mode"
    );
    assert_eq!(decoded.attach_source_mode, None);
}

/// R1.4: an UNRECOGNIZED `mode` value must yield `None` (NO change). Unlike
/// `attach` (which falls back to `Pasta` on garbage via `SourceMode::parse`),
/// the runtime toggle parses STRICTLY: an invalid token must NOT silently
/// switch the mode to `Pasta` — `None` means "keep current mode".
#[test]
fn source_presentation_request_invalid_mode_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        62,
        "pasta/sourcePresentation",
        json!({ "mode": "xml" }),
    ));
    assert_eq!(
        decoded.requested_source_mode, None,
        "R1.4: unrecognized mode → None (no change), NOT a Pasta fallback"
    );
}

/// R1.4: a MISSING (or non-string) `mode` likewise yields `None` (no change).
#[test]
fn source_presentation_request_missing_mode_is_none() {
    let mut dap = DapAdapter::new();
    let decoded =
        dap.decode_request(&request(63, "pasta/sourcePresentation", json!({})));
    assert_eq!(
        decoded.requested_source_mode, None,
        "R1.4: absent mode → None (no change)"
    );

    // A non-string mode is also rejected strictly.
    let mut dap2 = DapAdapter::new();
    let decoded2 = dap2.decode_request(&request(
        64,
        "pasta/sourcePresentation",
        json!({ "mode": 1 }),
    ));
    assert_eq!(
        decoded2.requested_source_mode, None,
        "R1.4: non-string mode → None (no change)"
    );
}

/// The valid tokens are matched case-insensitively, mirroring the existing
/// `SourceMode::parse` convention, but WITHOUT its invalid-value fallback.
#[test]
fn source_presentation_request_mode_is_case_insensitive() {
    for (raw, expected) in [
        ("LUA", SourceMode::Lua),
        ("Pasta", SourceMode::Pasta),
        ("  lua  ", SourceMode::Lua),
    ] {
        let mut dap = DapAdapter::new();
        let decoded = dap.decode_request(&request(
            65,
            "pasta/sourcePresentation",
            json!({ "mode": raw }),
        ));
        assert_eq!(
            decoded.requested_source_mode,
            Some(expected),
            "mode={raw:?} must parse case-insensitively to {expected:?}"
        );
    }
}

/// Event-builder: `source_presentation_event` produces the custom event
/// `pasta/sourcePresentation` with body `{ "mode": "lua"|"pasta" }`, reusing
/// the existing `event(...)` envelope (R2.5/R2.6 push notification body).
#[test]
fn source_presentation_event_builds_custom_event() {
    let mut dap = DapAdapter::new();
    let ev = dap.source_presentation_event(SourceMode::Lua);
    assert_eq!(ev["type"], "event");
    assert_eq!(ev["event"], "pasta/sourcePresentation");
    assert_eq!(ev["body"]["mode"], "lua");
    // Reuses the monotonic seq counter from event().
    assert_eq!(ev["seq"], 1);

    let ev2 = dap.source_presentation_event(SourceMode::Pasta);
    assert_eq!(ev2["body"]["mode"], "pasta");
    assert_eq!(ev2["seq"], 2, "event() seq is monotonic");
}

/// Response-builder: `source_presentation_response` echoes the given resolved
/// mode in body `{ "mode": ... }` and correlates to the request seq (R1.3
/// acceptance response), reusing the existing `response(...)` envelope.
#[test]
fn source_presentation_response_echoes_resolved_mode() {
    let mut dap = DapAdapter::new();
    let resp = dap.source_presentation_response(70, SourceMode::Lua);
    assert_eq!(resp["type"], "response");
    assert_eq!(resp["command"], "pasta/sourcePresentation");
    assert_eq!(resp["request_seq"], 70);
    assert_eq!(resp["success"], true);
    assert_eq!(resp["body"]["mode"], "lua", "R1.3: echo resolved mode");

    let resp2 = dap.source_presentation_response(71, SourceMode::Pasta);
    assert_eq!(resp2["body"]["mode"], "pasta");
    assert_eq!(resp2["request_seq"], 71);
}
