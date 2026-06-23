//! Inline test cluster for the `pasta/playScene` custom request decode
//! (pasta-scene-kick task 2.1, component PlaySceneDecode). Mirrors the
//! `pasta/sourcePresentation` decode cluster: a custom DAP request whose decode
//! extracts a scene name into `Decoded.kick_scene` (`Some`), treating an
//! empty/missing/non-string name as invalid (`None`) without falling into
//! generic routing.
use super::dap_test_support::*;
use super::*;

// --- pasta/playScene custom request (R2.1 / R2.2 / R2.3 / R2.5) ----------

/// R2.1 / R2.2: a `pasta/playScene` request carrying `scene: "intro"` decodes to
/// a `Decoded` whose `kick_scene` is `Some("intro")` (the extracted scene name).
/// The decode itself produces NO session command and NO response — the wiring
/// (task 2.2) owns sink invocation and the ack — so the request does not fall
/// into generic routing (R2.3): every other `Decoded` field stays at default.
#[test]
fn play_scene_request_decodes_scene_name() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(70, "pasta/playScene", json!({ "scene": "intro" })));
    assert_eq!(
        decoded.kick_scene,
        Some("intro".to_string()),
        "R2.1/R2.2: scene=intro → Some(\"intro\") kick scene"
    );
    // Decode does not route or ack: other fields are default (R2.3 — no generic
    // routing / stop loop fallthrough).
    assert_eq!(
        decoded,
        Decoded {
            kick_scene: Some("intro".to_string()),
            ..Decoded::default()
        }
    );
}

/// R2.5: an EMPTY scene name (`""`) is invalid and decodes to `None` — the kick
/// must NOT be issued for an empty name, and the request must not fall into
/// generic routing (other fields default).
#[test]
fn play_scene_request_empty_scene_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(71, "pasta/playScene", json!({ "scene": "" })));
    assert_eq!(
        decoded.kick_scene, None,
        "R2.5: empty scene name → None (invalid)"
    );
    assert_eq!(decoded, Decoded::default());
}

/// R2.5: a whitespace-only scene name is likewise invalid (`None`).
#[test]
fn play_scene_request_blank_scene_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(72, "pasta/playScene", json!({ "scene": "   " })));
    assert_eq!(
        decoded.kick_scene, None,
        "R2.5: whitespace-only scene name → None (invalid)"
    );
}

/// R2.5: a MISSING `scene` key decodes to `None` (no kick), other fields default.
#[test]
fn play_scene_request_missing_scene_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(73, "pasta/playScene", json!({})));
    assert_eq!(
        decoded.kick_scene, None,
        "R2.5: absent scene key → None (invalid)"
    );
    assert_eq!(decoded, Decoded::default());

    // A non-string scene is also rejected.
    let mut dap2 = DapAdapter::new();
    let decoded2 = dap2.decode_request(&request(74, "pasta/playScene", json!({ "scene": 1 })));
    assert_eq!(
        decoded2.kick_scene, None,
        "R2.5: non-string scene → None (invalid)"
    );
}

/// R2.3 (no regression): an existing `pasta/sourcePresentation` decode does NOT
/// populate `kick_scene` — the two custom requests stay independent.
#[test]
fn source_presentation_request_does_not_set_kick_scene() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        75,
        "pasta/sourcePresentation",
        json!({ "mode": "lua" }),
    ));
    assert_eq!(
        decoded.kick_scene, None,
        "sourcePresentation must not overload kick_scene"
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

/// 5.4 (coexistence): the old `pasta/playScene` decode still works alongside the
/// new arm (task 4.1 keeps both; 4.2 removes the old one later). A
/// `pasta/playSceneAt` request must NOT populate `kick_scene`.
#[test]
fn play_scene_at_request_does_not_set_kick_scene() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        86,
        "pasta/playSceneAt",
        json!({ "uri": "file:///c:/work/talk.pasta", "line": 25 }),
    ));
    assert_eq!(
        decoded.kick_scene, None,
        "playSceneAt must not overload kick_scene"
    );
}
