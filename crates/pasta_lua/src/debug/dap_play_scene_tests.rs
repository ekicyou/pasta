//! Inline test cluster for the `pasta/playScene` custom request decode
//! (pasta-scene-kick task 2.1, component PlaySceneDecode). Mirrors the
//! `pasta/sourcePresentation` decode cluster: a custom DAP request whose decode
//! extracts a scene name into `Decoded.kick_scene` (`Some`), treating an
//! empty/missing/non-string name as invalid (`None`) without falling into
//! generic routing.
use super::*;
use super::dap_test_support::*;

// --- pasta/playScene custom request (R2.1 / R2.2 / R2.3 / R2.5) ----------

/// R2.1 / R2.2: a `pasta/playScene` request carrying `scene: "intro"` decodes to
/// a `Decoded` whose `kick_scene` is `Some("intro")` (the extracted scene name).
/// The decode itself produces NO session command and NO response — the wiring
/// (task 2.2) owns sink invocation and the ack — so the request does not fall
/// into generic routing (R2.3): every other `Decoded` field stays at default.
#[test]
fn play_scene_request_decodes_scene_name() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        70,
        "pasta/playScene",
        json!({ "scene": "intro" }),
    ));
    assert_eq!(
        decoded.kick_scene,
        Some("intro".to_string()),
        "R2.1/R2.2: scene=intro → Some(\"intro\") kick scene"
    );
    // Decode does not route or ack: other fields are default (R2.3 — no generic
    // routing / stop loop fallthrough).
    assert_eq!(decoded, Decoded {
        kick_scene: Some("intro".to_string()),
        ..Decoded::default()
    });
}

/// R2.5: an EMPTY scene name (`""`) is invalid and decodes to `None` — the kick
/// must NOT be issued for an empty name, and the request must not fall into
/// generic routing (other fields default).
#[test]
fn play_scene_request_empty_scene_is_none() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        71,
        "pasta/playScene",
        json!({ "scene": "" }),
    ));
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
    let decoded = dap.decode_request(&request(
        72,
        "pasta/playScene",
        json!({ "scene": "   " }),
    ));
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
    let decoded2 = dap2.decode_request(&request(
        74,
        "pasta/playScene",
        json!({ "scene": 1 }),
    ));
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
