//! `DapAdapter`: the hand-written DAP minimal-subset translation layer (design
//! "Transport & DapAdapter", requirements 3.2 / 3.3 / 3.4 / 3.5).
//!
//! # Role in the backend
//!
//! [`DapAdapter`] is the protocol layer that sits between the [`Transport`] wire
//! boundary (raw [`serde_json::Value`] frames) and the protocol-independent
//! [`DebugSession`]. It is PURE translation: it maps inbound DAP request JSON to
//! [`SessionCommand`]s (plus any immediate DAP response), and maps
//! [`SessionEvent`]s coming back from the session to the matching DAP
//! response/event JSON. It owns NO Lua state, opens NO sockets, and never
//! touches `mlua` — that separation is the whole point of the channel seam.
//!
//! [`Transport`]: crate::debug::transport
//! [`DebugSession`]: crate::debug::session
//!
//! # Hand-written, dependency-minimal (design "依存最小")
//!
//! DAP messages are built and parsed by hand with the already-present
//! `serde_json`. The `dap` crate (and any other heavy DAP dependency) is
//! deliberately NOT used, keeping the supply chain and distribution size small.
//!
//! # DAP message envelopes
//!
//! - **Request** (inbound): `{"seq":N,"type":"request","command":"<cmd>","arguments":{…}}`.
//! - **Response** (outbound): `{"seq":<out>,"type":"response","request_seq":<req
//!   seq>,"success":true,"command":"<cmd>","body":{…}}` (the `body` is omitted
//!   for bare acks).
//! - **Event** (outbound, unsolicited): `{"seq":<out>,"type":"event","event":"<name>","body":{…}}`.
//!
//! The outgoing `seq` is a monotonic counter ([`DapAdapter::next_seq`]) shared by
//! every response and event the adapter emits.
//!
//! # Deferred responses & `request_seq` correlation
//!
//! Several requests cannot be answered until the session replies with the
//! corresponding [`SessionEvent`] (e.g. a `stackTrace` request becomes a
//! [`SessionCommand::StackTrace`], and only later does
//! [`SessionEvent::Stack`] arrive). The adapter records the originating request
//! `seq` in a small FIFO [`PendingTable`], keyed by the event KIND the request
//! will produce, so the deferred response carries the correct `request_seq`. The
//! transport is a single ordered TCP stream, so a per-kind FIFO is sufficient to
//! pair each event back to its request.
//!
//! # `frame_id` / `variablesReference` numbering (design "Implementation Notes")
//!
//! The adapter assigns these ids itself and maps them back; table deep-expansion
//! is OUT OF SCOPE (all leaf variables report `variablesReference: 0`):
//!
//! - **`frame_id` = stack index** (0-based) as ordered in [`SessionEvent::Stack`].
//!   A `scopes` request carries that `frameId` straight through into
//!   [`SessionCommand::Scopes`].
//! - **`variablesReference` = `frame_id + 1`** for the single synthetic `Locals`
//!   scope of a frame. The `+ 1` keeps it non-zero (DAP reserves `0` for "no
//!   children"), and it is trivially decoded back to the frame (`var_ref - 1`)
//!   when a subsequent `variables` request arrives. A `variables` request passes
//!   its `variablesReference` straight through into
//!   [`SessionCommand::Variables`]; the session side owns the `var_ref -> frame`
//!   decode. Note: this adapter emits the `Locals` scope itself from the frame
//!   list rather than relying on the session's [`Scope`](crate::debug::types::Scope)
//!   handles, so the scheme
//!   is self-contained and deterministic.
//!
//! # Error mapping (design "Event Contract": `output` optional)
//!
//! [`SessionEvent::Error`] is mapped to a DAP `output` event on the `stderr`
//! category. This is a sane, non-fatal surfacing: the IDE shows the message in
//! the debug console without aborting the session (a failed *response* would
//! need a request to correlate to, which an asynchronous VM/FFI error does not
//! have).

use std::collections::VecDeque;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::debug::SourceMode;
use crate::debug::source_map::SourceMap;
use crate::debug::types::{
    ResolvedBreakpoint, SessionCommand, SessionEvent, SourceRef, StopReason, ThreadInfo, Variable,
};

/// The DAP `source` presentation for one frame: the `source` JSON object and
/// the (possibly remapped) line to report (design "SourceMapSeam", R4.3).
///
/// A [`SourceResolver`] returns this for a frame's `(lua_source, lua_line)`. The
/// [default resolver](default_source_resolver) returns the generated `.lua`
/// unchanged (`{ "path": <lua source> }`, `line = lua_line`); a future
/// `pasta-source-map` resolver returns a `.pasta` path and the mapped `.pasta`
/// line instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSource {
    /// The DAP `source` object to embed in the stack frame (e.g.
    /// `{ "path": "@scene.lua" }` or `{ "path": "scene.pasta" }`).
    pub source: Value,
    /// The line to report for the frame, after any source mapping.
    pub line: u32,
}

/// The DAP-presentation source seam (design "SourceMapSeam", R4.3).
///
/// Maps a frame's generated `(lua_source, lua_line)` to the DAP [`ResolvedSource`]
/// to present. Installed on a [`DapAdapter`] via
/// [`set_source_resolver`](DapAdapter::set_source_resolver); the default is
/// [`default_source_resolver`] (presents the generated `.lua` unchanged, R4.3
/// "既定の提示は生成 .lua").
///
/// This is the DAP-PRESENTATION seam and is deliberately independent of the
/// code_gen producer seam (`SourceMapSink`). The `pasta-source-map` spec connects
/// the two: it builds a [`SourceMap`](crate::debug::source_map::SourceMap) via the
/// producer seam and installs a resolver here (see
/// [`pasta_source_resolver`]) that consults that map to present `.pasta`
/// paths/lines. No `.pasta` mapping is implemented in this layer — only the
/// swappable口.
pub type SourceResolver = Box<dyn Fn(&str, u32) -> ResolvedSource + Send>;

/// The default [`SourceResolver`]: present the generated `.lua` unchanged.
///
/// Returns `{ "path": <lua_source> }` with `line = lua_line`, byte-equivalent to
/// task 3.2's hard-coded behavior (R4.3 "本仕様の既定提示は生成 .lua").
pub fn default_source_resolver() -> SourceResolver {
    Box::new(|lua_source: &str, lua_line: u32| ResolvedSource {
        source: json!({ "path": lua_source }),
        line: lua_line,
    })
}

/// `.pasta` 提示用の [`SourceResolver`]（task 5.2・R5.1/R5.2/R5.3/R6.2/R3.3）。
///
/// 各フレームの生成 `(lua_source, lua_line)` を、集約 [`SourceMap`] の
/// [`resolve_lua_to_pasta`](SourceMap::resolve_lua_to_pasta) で `.pasta` 位置へ
/// 写像する（R3.3 双方向変換はマップ経由）:
///
/// - `Some(pos)`: `.pasta` `{ path: pos.file, line: pos.line }` を提示する
///   （停止位置・各コールスタックフレーム＝R5.1/R5.2）。
/// - `None`: 既定 `.lua` resolver（[`default_source_resolver`]）へ委譲し、生成
///   `.lua` を **判別可能**に提示する（対応なしフォールバック＝R5.3）。誤った
///   `.pasta` 対応づけ（mismap）は決して行わない（design "Error Handling"
///   610/617・整合性エラーも `.lua` フォールバック）。
///
/// `lua_source` は **フック報告の生 chunk 名**（`@<絶対 .lua パス>` 想定）であり、
/// [`SourceMap::resolve_lua_to_pasta`] が内部で
/// [`canonicalize_chunk_name`](crate::debug::source_map) による正規化を行う
/// （task 3.4）。したがって本 resolver は `lua_source` を **そのまま**渡す
/// （二重正規化しない）。
///
/// この resolver は提示モード `Pasta`（既定）時に
/// [`DapAdapter::set_source_resolver`] で装着する。`Lua` 時やマップ未提供時は
/// 既定 `.lua` resolver のままにする（R6.2・7.2 ゼロ劣化）— 装着判断は wiring 側
/// の `pasta_active()` ゲートが担う。
pub fn pasta_source_resolver(map: Arc<SourceMap>) -> SourceResolver {
    Box::new(move |lua_source: &str, lua_line: u32| {
        match map.resolve_lua_to_pasta(lua_source, lua_line) {
            // R5.1 / R5.2: 対応ありフレームは `.pasta` `{path, line}` を提示。
            Some(pos) => ResolvedSource {
                source: json!({ "path": pos.file }),
                line: pos.line,
            },
            // R5.3: 対応なし（行ミス／chunk ミス）は既定 `.lua` resolver と同一の
            // 提示へフォールバック（判別可能・誤マッピング禁止）。
            None => default_source_resolver()(lua_source, lua_line),
        }
    })
}

/// The kind of deferred [`SessionEvent`] a pending request is waiting for.
///
/// Used as the FIFO key in [`PendingTable`] so each deferred response is paired
/// back to the `request_seq` of the request that triggered it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PendingKind {
    /// Awaiting [`SessionEvent::Breakpoints`] for a `setBreakpoints` request.
    SetBreakpoints,
    /// Awaiting [`SessionEvent::Threads`] for a `threads` request.
    Threads,
    /// Awaiting [`SessionEvent::Stack`] for a `stackTrace` request.
    StackTrace,
    /// Awaiting [`SessionEvent::Variables`] for a `variables` request.
    Variables,
}

/// FIFO store of pending request seqs keyed by the [`PendingKind`] they await.
///
/// The transport delivers events in TCP order, so the oldest outstanding request
/// of a given kind is the one a freshly-arrived matching event answers.
#[derive(Debug, Default)]
struct PendingTable {
    set_breakpoints: VecDeque<u64>,
    threads: VecDeque<u64>,
    stack_trace: VecDeque<u64>,
    variables: VecDeque<u64>,
}

impl PendingTable {
    /// Record `request_seq` as awaiting the given event `kind`.
    fn push(&mut self, kind: PendingKind, request_seq: u64) {
        match kind {
            PendingKind::SetBreakpoints => self.set_breakpoints.push_back(request_seq),
            PendingKind::Threads => self.threads.push_back(request_seq),
            PendingKind::StackTrace => self.stack_trace.push_back(request_seq),
            PendingKind::Variables => self.variables.push_back(request_seq),
        }
    }

    /// Pop the oldest pending `request_seq` for `kind`, if any.
    fn pop(&mut self, kind: PendingKind) -> Option<u64> {
        match kind {
            PendingKind::SetBreakpoints => self.set_breakpoints.pop_front(),
            PendingKind::Threads => self.threads.pop_front(),
            PendingKind::StackTrace => self.stack_trace.pop_front(),
            PendingKind::Variables => self.variables.pop_front(),
        }
    }
}

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
}

/// Hand-written DAP minimal-subset adapter (design "Transport & DapAdapter").
///
/// Translates inbound DAP request [`Value`]s into [`SessionCommand`]s (+ optional
/// immediate response) and outbound [`SessionEvent`]s into DAP response/event
/// [`Value`]s, correlating deferred responses to their originating request `seq`.
/// Stateful only in the small bookkeeping it must own: the monotonic outgoing
/// `seq` counter and the [`PendingTable`].
pub struct DapAdapter {
    /// Monotonic outgoing sequence counter for every response/event emitted.
    out_seq: u64,
    /// Pending request seqs awaiting their deferred [`SessionEvent`].
    pending: PendingTable,
    /// The DAP-presentation source seam consulted per stack frame (R4.3).
    ///
    /// Defaults to [`default_source_resolver`] (generated `.lua` unchanged); a
    /// future `.pasta` resolver is installed via
    /// [`set_source_resolver`](DapAdapter::set_source_resolver).
    source_resolver: SourceResolver,
}

impl std::fmt::Debug for DapAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `source_resolver` is a boxed closure (no Debug); summarise it instead.
        f.debug_struct("DapAdapter")
            .field("out_seq", &self.out_seq)
            .field("pending", &self.pending)
            .field("source_resolver", &"<SourceResolver>")
            .finish()
    }
}

impl Default for DapAdapter {
    fn default() -> Self {
        Self {
            out_seq: 0,
            pending: PendingTable::default(),
            source_resolver: default_source_resolver(),
        }
    }
}

impl DapAdapter {
    /// Construct a fresh adapter with an empty pending table, `seq` at 0, and the
    /// default `.lua` source resolver (R4.3).
    pub fn new() -> Self {
        Self::default()
    }

    /// Install an alternate [`SourceResolver`] for stack-frame `source`
    /// presentation, replacing the default generated-`.lua` resolver (R4.3).
    ///
    /// This is the swappable口 the downstream `pasta-source-map` spec uses to
    /// present `.pasta` paths/lines instead of the generated `.lua`, without
    /// changing the response shape. The resolver is consulted per frame by
    /// [`encode_frames`]; only this DAP-presentation layer is affected, leaving
    /// the code_gen producer seam (`SourceMapSink`) independent.
    pub fn set_source_resolver(&mut self, resolver: SourceResolver) {
        self.source_resolver = resolver;
    }

    /// Allocate the next monotonic outgoing `seq` (1, 2, 3, …).
    fn next_seq(&mut self) -> u64 {
        self.out_seq += 1;
        self.out_seq
    }

    /// Build a DAP response envelope for `command`/`request_seq` with `body`.
    ///
    /// `body` may be [`Value::Null`] for a bare ack, in which case the `body`
    /// field is omitted entirely (an empty ack response).
    fn response(&mut self, request_seq: u64, command: &str, body: Value) -> Value {
        let seq = self.next_seq();
        let mut msg = json!({
            "seq": seq,
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": command,
        });
        if !body.is_null() {
            msg["body"] = body;
        }
        msg
    }

    /// Build a DAP event envelope named `event` with `body`.
    fn event(&mut self, event: &str, body: Value) -> Value {
        let seq = self.next_seq();
        json!({
            "seq": seq,
            "type": "event",
            "event": event,
            "body": body,
        })
    }

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

    /// Encode one outbound [`SessionEvent`] into DAP response/event [`Value`]s.
    ///
    /// Deferred responses ([`SessionEvent::Breakpoints`] / `Threads` / `Stack` /
    /// `Variables`) are correlated back to their originating request `seq` via
    /// the [`PendingTable`]; if no pending request is found (e.g. a spurious or
    /// out-of-band event) the correlation falls back to `request_seq = 0`.
    /// Unsolicited events ([`SessionEvent::Stopped`] / `Terminated` / `Error`)
    /// become DAP events. Each call returns zero or more frames to write to the
    /// transport.
    pub fn encode_event(&mut self, event: SessionEvent) -> Vec<Value> {
        match event {
            SessionEvent::Stopped { reason, thread_id } => {
                let body = json!({
                    "reason": stop_reason_str(reason),
                    "threadId": thread_id,
                    "allThreadsStopped": true,
                });
                vec![self.event("stopped", body)]
            }
            SessionEvent::Terminated => vec![self.event("terminated", json!({}))],
            SessionEvent::Breakpoints(bps) => {
                let request_seq = self.pending.pop(PendingKind::SetBreakpoints).unwrap_or(0);
                let body = json!({ "breakpoints": encode_breakpoints(&bps) });
                vec![self.response(request_seq, "setBreakpoints", body)]
            }
            SessionEvent::Threads(threads) => {
                let request_seq = self.pending.pop(PendingKind::Threads).unwrap_or(0);
                let body = json!({ "threads": encode_threads(&threads) });
                vec![self.response(request_seq, "threads", body)]
            }
            SessionEvent::Stack(frames) => {
                let request_seq = self.pending.pop(PendingKind::StackTrace).unwrap_or(0);
                let total = frames.len();
                let body = json!({
                    "stackFrames": encode_frames(&frames, self.source_resolver.as_ref()),
                    "totalFrames": total,
                });
                vec![self.response(request_seq, "stackTrace", body)]
            }
            SessionEvent::Scopes(scopes) => {
                // `scopes` is answered immediately at decode time (from the frame
                // id), so a SessionEvent::Scopes carries no request to correlate.
                // It is intentionally a no-op on the wire. The synthetic scope is
                // documented on DapAdapter; see decode_request("scopes").
                let _ = scopes;
                Vec::new()
            }
            SessionEvent::Variables(vars) => {
                let request_seq = self.pending.pop(PendingKind::Variables).unwrap_or(0);
                let body = json!({ "variables": encode_variables(&vars) });
                vec![self.response(request_seq, "variables", body)]
            }
            SessionEvent::Error(msg) => {
                // Surface asynchronous VM/FFI errors as a non-fatal `output`
                // event on stderr (design "Event Contract": output optional).
                let body = json!({
                    "category": "stderr",
                    "output": format!("{msg}\n"),
                });
                vec![self.event("output", body)]
            }
        }
    }
}

/// Strictly parse a `pasta/sourcePresentation` `mode` token into a
/// [`SourceMode`], for the runtime presentation toggle (requirement 1.1 / 1.2 /
/// 1.4).
///
/// ONLY the exact tokens `"pasta"` / `"lua"` (case-insensitive, surrounding
/// whitespace ignored — mirroring [`SourceMode::parse`]'s tokenizing convention)
/// yield `Some`; ANY other value yields `None`.
///
/// Unlike [`SourceMode::parse`], there is NO invalid-value fallback to the
/// default `Pasta`. That fallback is correct for `attach` (the author DID specify
/// a presentation, just wrongly) but WRONG for the runtime toggle: requirement
/// 1.4 mandates that an unrecognized mode value cause NO mode change, so an
/// invalid token must map to `None` ("keep current mode"), never silently to
/// `Pasta`.
fn parse_source_mode_strict(raw: &str) -> Option<SourceMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "pasta" => Some(SourceMode::Pasta),
        "lua" => Some(SourceMode::Lua),
        _ => None,
    }
}

/// Map a [`SourceMode`] to its `pasta/sourcePresentation` wire token
/// (`"pasta"` / `"lua"`), used in both the acceptance response and the custom
/// event body (requirement 1.3 / 2.5 / 2.6).
fn source_mode_str(mode: SourceMode) -> &'static str {
    match mode {
        SourceMode::Pasta => "pasta",
        SourceMode::Lua => "lua",
    }
}

/// Map a [`StopReason`] to its DAP `stopped` `reason` string.
fn stop_reason_str(reason: StopReason) -> &'static str {
    match reason {
        StopReason::Breakpoint => "breakpoint",
        StopReason::Step => "step",
        StopReason::Entry => "entry",
        StopReason::Pause => "pause",
    }
}

/// Parse a `setBreakpoints` request's `arguments` into `(SourceRef, lines)`.
///
/// DAP shape: `{"source":{"path":".."},"breakpoints":[{"line":N},..]}`. Missing
/// pieces degrade gracefully: no source path → empty path; no breakpoints →
/// empty line set (which clears the source's breakpoints, per DAP semantics).
fn parse_set_breakpoints(args: Option<&Value>) -> (SourceRef, Vec<u32>) {
    let path = args
        .and_then(|a| a.get("source"))
        .and_then(|s| s.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let lines = args
        .and_then(|a| a.get("breakpoints"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|bp| bp.get("line").and_then(Value::as_u64).map(|l| l as u32))
                .collect()
        })
        .unwrap_or_default();

    (SourceRef { path }, lines)
}

/// Encode resolved breakpoints into the `setBreakpoints` response `body`.
fn encode_breakpoints(bps: &[ResolvedBreakpoint]) -> Vec<Value> {
    bps.iter()
        .map(|bp| {
            json!({
                "verified": bp.verified,
                "line": bp.line,
            })
        })
        .collect()
}

/// Encode threads into the `threads` response `body`.
fn encode_threads(threads: &[ThreadInfo]) -> Vec<Value> {
    threads
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.name }))
        .collect()
}

/// Encode stack frames into the `stackTrace` response `body`.
///
/// `frame id = stack index` (0-based), per the documented numbering. The frame's
/// `source`/`line` are produced by `resolver` rather than hard-coded, so the
/// DAP-presentation seam is swappable (R4.3): the default resolver presents the
/// generated `.lua` (`{ "path": <source> }`, `line = FrameInfo.line`)
/// byte-equivalently to task 3.2, while a future `pasta-source-map` resolver can
/// substitute a `.pasta` path and the mapped `.pasta` line without changing the
/// frame shape.
fn encode_frames(
    frames: &[crate::debug::types::FrameInfo],
    resolver: &(dyn Fn(&str, u32) -> ResolvedSource + Send),
) -> Vec<Value> {
    frames
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            let resolved = resolver(&f.source, f.line);
            json!({
                "id": idx as u32,
                "name": f.func_name.clone().unwrap_or_else(|| "?".to_string()),
                "source": resolved.source,
                "line": resolved.line,
                "column": 1,
            })
        })
        .collect()
}

/// Encode variables into the `variables` response `body`.
///
/// Maps [`Variable::repr`] → DAP `value` and [`Variable::type_name`] → DAP
/// `type`. Leaf variables report `variablesReference: 0` (no deep table
/// expansion — out of scope for this task).
fn encode_variables(vars: &[Variable]) -> Vec<Value> {
    vars.iter()
        .map(|v| {
            json!({
                "name": v.name,
                "value": v.repr,
                "type": v.type_name,
                "variablesReference": 0,
            })
        })
        .collect()
}

// Inline `#[cfg(test)] mod tests` was externalized into logical-cluster sibling
// files (Task 2.2, pure behavior-invariant move). Each sibling begins with
// `use super::*;` and keeps the same module path, preserving private/`pub(crate)`
// reachability into this production module. The cluster-shared `request` builder
// lives in `dap_test_support` (`pub(super)`); each cluster `use`s it. The set of
// leaf test-fn names and the total test count are unchanged.
#[cfg(test)]
#[path = "dap_test_support.rs"]
mod dap_test_support;

#[cfg(test)]
#[path = "dap_protocol_tests.rs"]
mod dap_protocol_tests;

#[cfg(test)]
#[path = "dap_source_presentation_tests.rs"]
mod dap_source_presentation_tests;

#[cfg(test)]
#[path = "dap_source_resolver_tests.rs"]
mod dap_source_resolver_tests;

#[cfg(test)]
#[path = "dap_edge_tests.rs"]
mod dap_edge_tests;
