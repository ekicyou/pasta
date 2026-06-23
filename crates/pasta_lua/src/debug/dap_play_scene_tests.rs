//! Inline test cluster for the position-based `pasta/playSceneAt` custom request
//! decode (kick-from-cursor task 4.1, component "PlaySceneAt transport"). A
//! custom DAP request whose decode extracts a position `(uri, line)` into
//! `Decoded.play_scene_at` (`Some`), treating a missing/empty/non-string `uri` or
//! a missing/non-number `line` as invalid (`None`) without falling into generic
//! routing.
//!
//! The OLD name-based `pasta/playScene` decode arm (and its `kick_scene` field)
//! was REMOVED in task 4.2 (requirement 5.4): the only external scene-execution
//! request is now `pasta/playSceneAt`. The `old_name_based_play_scene_is_unrecognized`
//! test below pins that removed-arm invariant.
use super::dap_test_support::*;
use super::*;

// --- removed name-based transport (task 4.2 / requirement 5.4) ------------

/// Requirement 5.4: the OLD name-based `pasta/playScene` external transport was
/// removed. A `pasta/playScene` request is no longer a recognised custom request:
/// the decode arm is GONE, so it falls through to the `_ => Decoded::default()`
/// catch-all and produces an EMPTY `Decoded` (no command, no response, no special
/// scene-kick state). This is the observable that the old arm is absent.
#[test]
fn old_name_based_play_scene_is_unrecognized() {
    let mut dap = DapAdapter::new();
    // Even a well-formed (old-style) name-based request is now unrecognised.
    let decoded = dap.decode_request(&request(70, "pasta/playScene", json!({ "scene": "intro" })));
    assert_eq!(
        decoded,
        Decoded::default(),
        "R5.4: pasta/playScene is no longer recognised — decodes to an empty Decoded"
    );
    // And specifically it does NOT populate the position-based field either.
    assert_eq!(
        decoded.play_scene_at, None,
        "R5.4: the removed name-based arm must not touch play_scene_at"
    );
}

// --- pasta/playSceneAt custom request (task 4.1 / R4.1-R4.4) --------------

/// R4.1 / R4.4: a `pasta/playSceneAt` request carrying `{ uri, line }` decodes to
/// a `Decoded` whose `play_scene_at` is `Some((uri, line))`. The decode itself
/// produces NO session command and NO response — the wiring (task 4.1) owns the
/// resolver call and the ack — so the request does not fall into generic routing:
/// every other `Decoded` field stays at default.
#[test]
fn play_scene_at_request_decodes_uri_and_line() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        80,
        "pasta/playSceneAt",
        json!({ "uri": "file:///c:/work/talk.pasta", "line": 25 }),
    ));
    assert_eq!(
        decoded.play_scene_at,
        Some(("file:///c:/work/talk.pasta".to_string(), 25)),
        "R4.1/R4.4: {{uri,line}} → Some((uri, line))"
    );
    // Decode does not route or ack: other fields are default.
    assert_eq!(
        decoded,
        Decoded {
            play_scene_at: Some(("file:///c:/work/talk.pasta".to_string(), 25)),
            ..Decoded::default()
        }
    );
}

/// R4.4 (strict parse): a MISSING `line` decodes to `None` — the request is
/// invalid and must not be dispatched to the resolver.
#[test]
fn play_scene_at_request_missing_line_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        81,
        "pasta/playSceneAt",
        json!({ "uri": "file:///c:/work/talk.pasta" }),
    ));
    assert_eq!(
        decoded.play_scene_at, None,
        "R4.4: missing line → None (invalid)"
    );
    assert_eq!(decoded, Decoded::default());
}

/// R4.4 (strict parse): a non-string `uri` is invalid (`None`).
#[test]
fn play_scene_at_request_non_string_uri_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        82,
        "pasta/playSceneAt",
        json!({ "uri": 5, "line": 25 }),
    ));
    assert_eq!(
        decoded.play_scene_at, None,
        "R4.4: non-string uri → None (invalid)"
    );
}

/// R4.4 (strict parse): an EMPTY `uri` is invalid (`None`).
#[test]
fn play_scene_at_request_empty_uri_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        83,
        "pasta/playSceneAt",
        json!({ "uri": "   ", "line": 25 }),
    ));
    assert_eq!(
        decoded.play_scene_at, None,
        "R4.4: empty/blank uri → None (invalid)"
    );
}

/// R4.4 (strict parse): a MISSING `uri` decodes to `None`.
#[test]
fn play_scene_at_request_missing_uri_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(84, "pasta/playSceneAt", json!({ "line": 25 })));
    assert_eq!(
        decoded.play_scene_at, None,
        "R4.4: missing uri → None (invalid)"
    );
    assert_eq!(decoded, Decoded::default());
}

/// R4.4 (strict parse): a non-number `line` is invalid (`None`).
#[test]
fn play_scene_at_request_non_number_line_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        85,
        "pasta/playSceneAt",
        json!({ "uri": "file:///c:/work/talk.pasta", "line": "25" }),
    ));
    assert_eq!(
        decoded.play_scene_at, None,
        "R4.4: non-number line → None (invalid)"
    );
}
