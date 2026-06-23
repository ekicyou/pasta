//! Inbound side: the [`Decoded`] outcome type and the [`DapAdapter`] request
//! decode / custom `pasta/sourcePresentation` envelope methods (design
//! "Transport & DapAdapter", requirements 3.2 / 3.3).

use serde_json::{Value, json};

use crate::debug::SourceMode;
use crate::debug::types::SessionCommand;

use super::DapAdapter;
use super::codec::{
    parse_scene_strict, parse_set_breakpoints, parse_source_mode_strict, source_mode_str,
};
use super::pending::PendingKind;

/// The outcome of decoding one inbound DAP request.
///
/// A request can produce a [`SessionCommand`] to forward to the session, an
/// immediate DAP response to send straight back, or BOTH (e.g. `continue`
/// forwards the command AND immediately acks). `initialize` additionally needs a
/// follow-up `initialized` event, carried in `events`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Decoded {
    /// Command to forward to the session, if the request maps to one.
    pub command: Option<SessionCommand>,
    /// An immediate DAP response to send back now (acks and `initialize`).
    pub response: Option<Value>,
    /// Any immediate unsolicited events to emit after the response (the
    /// `initialized` event of the `initialize` handshake).
    pub events: Vec<Value>,
    /// The `attach` request's explicit `sourcePresentation` override, parsed to a
    /// [`SourceMode`] — `Some` ONLY when the `attach` arguments carried the key
    /// (task 5.5 / requirement 6.3 / design 581/586). The socket bridge applies
    /// it to the current session (resolver + step granularity), overriding the
    /// `enable`-time resolved env > file > 既定 mode. When the key is ABSENT this
    /// stays `None` so the resolved mode is kept (NO client-default override).
    pub attach_source_mode: Option<SourceMode>,
    /// The runtime presentation-toggle mode requested by a
    /// `pasta/sourcePresentation` custom request (requirement 1.1 / 1.2), parsed
    /// STRICTLY: `Some` ONLY for the exact valid tokens `"pasta"`/`"lua"`
    /// (case-insensitive); ANY other value — an unrecognized string, a missing
    /// key, or a non-string — yields `None`.
    ///
    /// This is DELIBERATELY separate from [`attach_source_mode`]: the two have
    /// different semantics. `attach` uses [`SourceMode::parse`], whose invalid
    /// fallback is the default `Pasta`; the runtime toggle must NOT fall back,
    /// because requirement 1.4 mandates that an unrecognized mode value cause NO
    /// mode change (a `Pasta` fallback would silently CHANGE the mode and violate
    /// 1.4). `None` therefore means "keep the current mode"; the wiring (task 3.1)
    /// keeps the current mode and echoes it.
    ///
    /// [`attach_source_mode`]: Decoded::attach_source_mode
    pub requested_source_mode: Option<SourceMode>,
    /// The scene name extracted from a `pasta/playScene` custom request
    /// (pasta-scene-kick requirements 2.1 / 2.2 / 2.5), parsed STRICTLY: `Some`
    /// ONLY for a non-empty (post-trim) string `scene` argument; a missing key,
    /// a non-string value, an empty string, or a whitespace-only string all
    /// yield `None`.
    ///
    /// `None` means the request carried no usable scene name and the kick must
    /// NOT be issued (R2.5). The decode itself produces no command/response — the
    /// wiring (task 2.2) owns sink invocation and the ack — so a `pasta/playScene`
    /// request never falls into generic routing (R2.3).
    pub kick_scene: Option<String>,
}

impl DapAdapter {
    /// Decode one inbound DAP request [`Value`] into a [`Decoded`] outcome.
    ///
    /// Recognises exactly the minimal subset (design "API Contract"). An
    /// unknown command yields an empty [`Decoded`] (no command, no response) so
    /// the caller can choose to ignore it; malformed-but-known requests fall
    /// back to sane defaults (e.g. missing breakpoint lines → empty set).
    pub fn decode_request(&mut self, req: &Value) -> Decoded {
        let request_seq = req.get("seq").and_then(Value::as_u64).unwrap_or(0);
        let command = req.get("command").and_then(Value::as_str).unwrap_or("");
        let args = req.get("arguments");

        match command {
            "initialize" => {
                let response = self.response(
                    request_seq,
                    "initialize",
                    json!({
                        "supportsConfigurationDoneRequest": true,
                    }),
                );
                // Standard DAP handshake: an `initialized` event follows the
                // initialize response (configurationDone then follows from the
                // client).
                let initialized = self.event("initialized", json!({}));
                Decoded {
                    response: Some(response),
                    events: vec![initialized],
                    ..Decoded::default()
                }
            }
            "setBreakpoints" => {
                let (source, lines) = parse_set_breakpoints(args);
                // Deferred: the verified breakpoints come back as
                // SessionEvent::Breakpoints; remember this request's seq.
                self.pending.push(PendingKind::SetBreakpoints, request_seq);
                Decoded {
                    command: Some(SessionCommand::SetBreakpoints { source, lines }),
                    ..Decoded::default()
                }
            }
            "configurationDone" => {
                let response = self.response(request_seq, "configurationDone", Value::Null);
                Decoded {
                    response: Some(response),
                    ..Decoded::default()
                }
            }
            "threads" => {
                self.pending.push(PendingKind::Threads, request_seq);
                Decoded {
                    command: Some(SessionCommand::Threads),
                    ..Decoded::default()
                }
            }
            "stackTrace" => {
                self.pending.push(PendingKind::StackTrace, request_seq);
                Decoded {
                    command: Some(SessionCommand::StackTrace),
                    ..Decoded::default()
                }
            }
            "scopes" => {
                // `scopes` is answered immediately from the frame id alone: one
                // synthetic `Locals` scope whose variablesReference = frameId+1
                // (non-zero, decodable back to the frame). We still forward the
                // Scopes command so the session can prepare frame state, but the
                // response does not wait on SessionEvent::Scopes.
                let frame_id = args
                    .and_then(|a| a.get("frameId"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u32;
                // Saturating: `frameId` is untrusted client JSON, and a plain
                // `+ 1` on `u32::MAX` would overflow-panic in debug builds.
                // Saturation keeps the reference non-zero (DAP reserves 0).
                let var_ref = frame_id.saturating_add(1);
                let response = self.response(
                    request_seq,
                    "scopes",
                    json!({
                        "scopes": [{
                            "name": "Locals",
                            "variablesReference": var_ref,
                            "expensive": false,
                        }],
                    }),
                );
                Decoded {
                    command: Some(SessionCommand::Scopes { frame_id }),
                    response: Some(response),
                    ..Decoded::default()
                }
            }
            "variables" => {
                let var_ref = args
                    .and_then(|a| a.get("variablesReference"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u32;
                self.pending.push(PendingKind::Variables, request_seq);
                Decoded {
                    command: Some(SessionCommand::Variables { var_ref }),
                    ..Decoded::default()
                }
            }
            "continue" => {
                let response = self.response(
                    request_seq,
                    "continue",
                    json!({ "allThreadsContinued": true }),
                );
                Decoded {
                    command: Some(SessionCommand::Continue),
                    response: Some(response),
                    ..Decoded::default()
                }
            }
            "next" => self.step_ack(request_seq, "next", SessionCommand::Next),
            "stepIn" => self.step_ack(request_seq, "stepIn", SessionCommand::StepIn),
            "stepOut" => self.step_ack(request_seq, "stepOut", SessionCommand::StepOut),
            "attach" => {
                // The server SIDE of `sourcePresentation` (task 5.5 / requirement
                // 6.3 / design 581/586): when the client put an explicit
                // `sourcePresentation` ("pasta"/"lua") on the `attach` arguments,
                // parse it (highest precedence) so the socket bridge can apply it
                // to THIS session — switching the `.pasta` resolver presentation
                // (task 5.2) AND the step granularity (task 5.4). An invalid value
                // falls back to the default `pasta` with a warning (design 615),
                // via [`SourceMode::parse`]. When the key is ABSENT, leave the
                // resolved env > file > 既定 mode in effect (NO client-default
                // override, design 581) — so `attach_source_mode` stays `None`.
                let attach_source_mode = args
                    .and_then(|a| a.get("sourcePresentation"))
                    .and_then(Value::as_str)
                    .map(SourceMode::parse);
                let response = self.response(request_seq, "attach", Value::Null);
                Decoded {
                    response: Some(response),
                    attach_source_mode,
                    ..Decoded::default()
                }
            }
            "pasta/sourcePresentation" => {
                // The decode SIDE of the runtime presentation toggle (requirement
                // 1.1 / 1.2 / 1.4). Parse the `mode` argument STRICTLY into
                // `requested_source_mode`: ONLY the exact tokens "pasta"/"lua"
                // (case-insensitive, mirroring `SourceMode::parse`'s convention)
                // yield `Some`; ANY other value — an unrecognized string, a
                // missing key, or a non-string — yields `None`.
                //
                // We DELIBERATELY do NOT use `SourceMode::parse` here: its invalid
                // fallback is the default `Pasta`, which would silently CHANGE the
                // mode and violate requirement 1.4 ("認識できない提示モード値" →
                // 現在の提示モードを変更せず). `None` means "no change"; the wiring
                // (task 3.1) keeps the current mode and echoes it in the response.
                //
                // No immediate response/command is produced here: the wiring owns
                // mode application, the acceptance response (built via
                // `source_presentation_response`), the `RefreshPresentation`
                // forwarding, and the `pasta/sourcePresentation` event (built via
                // `source_presentation_event`).
                let requested_source_mode = args
                    .and_then(|a| a.get("mode"))
                    .and_then(Value::as_str)
                    .and_then(parse_source_mode_strict);
                Decoded {
                    requested_source_mode,
                    ..Decoded::default()
                }
            }
            "pasta/playScene" => {
                // The decode SIDE of the scene kick (pasta-scene-kick R2.1 / R2.2
                // / R2.3 / R2.5). Extract `args.scene` STRICTLY into `kick_scene`:
                // a non-empty (post-trim) string yields `Some(name)`; a missing
                // key, a non-string, an empty string, or a whitespace-only string
                // yields `None` (invalid — the kick must NOT be issued, R2.5).
                //
                // No immediate response/command is produced here (R2.3): the
                // wiring (task 2.2) owns sink invocation and the ack/error
                // response. Leaving every other field at default keeps the request
                // out of generic routing.
                let kick_scene = parse_scene_strict(args);
                Decoded {
                    kick_scene,
                    ..Decoded::default()
                }
            }
            "disconnect" => {
                let response = self.response(request_seq, "disconnect", Value::Null);
                Decoded {
                    command: Some(SessionCommand::Disconnect),
                    response: Some(response),
                    ..Decoded::default()
                }
            }
            _ => Decoded::default(),
        }
    }

    /// Shared shape for `next`/`stepIn`/`stepOut`: ack immediately and forward
    /// the step command; the later `stopped` event reports the new position.
    fn step_ack(&mut self, request_seq: u64, command: &str, cmd: SessionCommand) -> Decoded {
        let response = self.response(request_seq, command, Value::Null);
        Decoded {
            command: Some(cmd),
            response: Some(response),
            ..Decoded::default()
        }
    }

    /// Build the `pasta/sourcePresentation` custom EVENT [`Value`] for `mode`
    /// (body `{ "mode": "pasta"|"lua" }`), reusing the existing [`event`](Self::event)
    /// envelope.
    ///
    /// This is the current-mode push notification (requirement 2.5 / 2.6): the
    /// wiring (task 3.1) emits it on attach-complete (resolved initial mode) and
    /// after a runtime toggle changes the mode, so the VSCode extension can keep
    /// its status-bar display in sync without polling.
    pub(crate) fn source_presentation_event(&mut self, mode: SourceMode) -> Value {
        self.event(
            "pasta/sourcePresentation",
            json!({ "mode": source_mode_str(mode) }),
        )
    }

    /// Build the `pasta/sourcePresentation` acceptance RESPONSE [`Value`] echoing
    /// the resolved `mode` (body `{ "mode": ... }`), correlated to `request_seq`,
    /// reusing the existing [`response`](Self::response) envelope.
    ///
    /// This is the acceptance confirmation (requirement 1.3): the wiring (task
    /// 3.1) passes the RESOLVED current mode — the newly applied mode on a valid
    /// request, or the UNCHANGED current mode when the request carried an
    /// unrecognized value (requirement 1.4) — and this helper echoes it back.
    pub(crate) fn source_presentation_response(&mut self, request_seq: u64, mode: SourceMode) -> Value {
        self.response(
            request_seq,
            "pasta/sourcePresentation",
            json!({ "mode": source_mode_str(mode) }),
        )
    }
}
