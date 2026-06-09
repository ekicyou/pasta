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
//!   list rather than relying on the session's [`Scope`] handles, so the scheme
//!   is self-contained and deterministic.
//!
//! # Error mapping (design "Event Contract": `output` optional)
//!
//! [`SessionEvent::Error`] is mapped to a DAP `output` event on the `stderr`
//! category. This is a sane, non-fatal surfacing: the IDE shows the message in
//! the debug console without aborting the session (a failed *response* would
//! need a request to correlate to, which an asynchronous VM/FFI error does not
//! have).

#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::debug::SourceMode;
use crate::debug::source_map::SourceMap;
use crate::debug::types::{
    ResolvedBreakpoint, Scope, SessionCommand, SessionEvent, SourceRef, StopReason, ThreadInfo,
    Variable,
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
                    command: None,
                    response: Some(response),
                    events: vec![initialized],
                    attach_source_mode: None,
                    requested_source_mode: None,
                }
            }
            "setBreakpoints" => {
                let (source, lines) = parse_set_breakpoints(args);
                // Deferred: the verified breakpoints come back as
                // SessionEvent::Breakpoints; remember this request's seq.
                self.pending.push(PendingKind::SetBreakpoints, request_seq);
                Decoded {
                    command: Some(SessionCommand::SetBreakpoints { source, lines }),
                    response: None,
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
                }
            }
            "configurationDone" => {
                let response = self.response(request_seq, "configurationDone", Value::Null);
                Decoded {
                    command: None,
                    response: Some(response),
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
                }
            }
            "threads" => {
                self.pending.push(PendingKind::Threads, request_seq);
                Decoded {
                    command: Some(SessionCommand::Threads),
                    response: None,
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
                }
            }
            "stackTrace" => {
                self.pending.push(PendingKind::StackTrace, request_seq);
                Decoded {
                    command: Some(SessionCommand::StackTrace),
                    response: None,
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
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
                let var_ref = frame_id + 1;
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
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
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
                    response: None,
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
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
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
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
                    command: None,
                    response: Some(response),
                    events: Vec::new(),
                    attach_source_mode,
                    requested_source_mode: None,
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
                    command: None,
                    response: None,
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode,
                }
            }
            "disconnect" => {
                let response = self.response(request_seq, "disconnect", Value::Null);
                Decoded {
                    command: Some(SessionCommand::Disconnect),
                    response: Some(response),
                    events: Vec::new(),
                    attach_source_mode: None,
                    requested_source_mode: None,
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
            events: Vec::new(),
            attach_source_mode: None,
            requested_source_mode: None,
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

/// Encode a session [`Scope`] (unused on the immediate-scopes path; retained for
/// completeness so a future richer scopes flow can reuse it).
fn encode_scope(scope: &Scope) -> Value {
    json!({
        "name": scope.name,
        "variablesReference": scope.variables_reference,
        "expensive": false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::debug::types::FrameInfo;

    /// Build a minimal DAP request value.
    fn request(seq: u64, command: &str, arguments: Value) -> Value {
        json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        })
    }

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

    // --- source resolver seam (R4.3) ---------------------------------------

    /// DEFAULT resolver: a `stackTrace` response presents each frame's `source`
    /// as the generated `.lua` (path = `FrameInfo.source`, line = `FrameInfo.line`),
    /// byte-equivalent to task 3.2 — R4.3 "既定の提示は生成 .lua".
    #[test]
    fn stack_trace_default_resolver_presents_generated_lua() {
        let mut dap = DapAdapter::new();
        dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));

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
        let resp = &out[0];
        let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
        // Default presentation is the generated .lua, unchanged from 3.2.
        assert_eq!(frames[0]["source"], json!({ "path": "@scene.lua" }));
        assert_eq!(frames[0]["line"], 7);
        assert_eq!(frames[1]["source"], json!({ "path": "@scene.lua" }));
        assert_eq!(frames[1]["line"], 2);
    }

    /// SWAPPABLE: install an alternate resolver that maps any `.lua` source to a
    /// `.pasta`-style source (and remaps the line); the same `SessionEvent::Stack`
    /// now presents the `.pasta` source/line — proving the口 is genuinely
    /// swappable (R4.3 "将来 .pasta パスを提示できる構造"). This stub stands in for
    /// the future `pasta-source-map` resolver (wired via task 5.3 / downstream).
    #[test]
    fn stack_trace_alternate_resolver_presents_pasta() {
        let mut dap = DapAdapter::new();
        // A test stub resolver: every frame becomes foo.pasta with line+100.
        dap.set_source_resolver(Box::new(|_lua_source: &str, lua_line: u32| {
            ResolvedSource {
                source: json!({ "path": "foo.pasta" }),
                line: lua_line + 100,
            }
        }));

        dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));
        let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
            source: "@scene.lua".to_string(),
            line: 7,
            func_name: Some("talk".to_string()),
        }]));
        let resp = &out[0];
        let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
        // The seam is swapped: presentation is now the .pasta source + mapped line.
        assert_eq!(frames[0]["source"], json!({ "path": "foo.pasta" }));
        assert_eq!(frames[0]["line"], 107);
        // Other frame fields (id / name) are unaffected by the source seam.
        assert_eq!(frames[0]["id"], 0);
        assert_eq!(frames[0]["name"], "talk");
    }

    #[test]
    fn set_breakpoints_with_no_breakpoints_clears_lines() {
        let mut dap = DapAdapter::new();
        let decoded = dap.decode_request(&request(
            7,
            "setBreakpoints",
            json!({ "source": { "path": "@s.lua" } }),
        ));
        assert_eq!(
            decoded.command,
            Some(SessionCommand::SetBreakpoints {
                source: SourceRef::new("@s.lua"),
                lines: vec![],
            }),
            "missing breakpoints array → empty (clears) line set"
        );
    }

    // --- pasta source resolver (task 5.2 — R5.1 / R5.2 / R5.3 / R6.2 / R3.3) ---

    use std::sync::Arc;

    use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};

    /// 既知の `chunk → .pasta` 対応を 1 件持つ集約 `SourceMap` を構築する小ヘルパ。
    ///
    /// `chunk_name`（生フック源相当）へ、最終 `.lua` 行 `lua_line` → `.pasta`
    /// `{file, pasta_line}` の 1 対応を登録する。`resolve_lua_to_pasta` は chunk
    /// 引数を内部で正規化する（task 3.4）ため、テストは生フック source 文字列を
    /// そのまま渡す。
    fn map_with(chunk_name: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
        let mut forward = std::collections::BTreeMap::new();
        forward.insert(
            lua_line,
            PastaPos {
                file: file.to_string(),
                line: pasta_line,
            },
        );
        let mut sm = SourceMap::new();
        sm.insert_chunk(
            chunk_name.to_string(),
            file.to_string(),
            ChunkSourceMap::from_forward(forward),
        );
        sm
    }

    /// R5.1 / R5.2 / R3.3: 対応のあるフレームは `.pasta` `{path, line}` を提示する。
    /// resolver は **生フック source**（`@` 付き・区切り混在）をそのまま
    /// `resolve_lua_to_pasta` へ渡し、内部正規化（task 3.4）で突合される。
    #[test]
    fn pasta_resolver_maps_frame_to_pasta_source_and_line() {
        // 格納時とは異なる等価形（`@` 付き・大小違い）の生フック source で引く。
        let raw_hook_source = r"@C:\proj\cache\scene.lua";
        let map = map_with("C:/proj/cache/scene.lua", 12, "C:/proj/scene.pasta", 7);
        let resolver = pasta_source_resolver(Arc::new(map));

        let resolved = resolver(raw_hook_source, 12);
        // 提示は `.pasta` ファイル・行（pos.file / pos.line）。
        assert_eq!(
            resolved.source,
            json!({ "path": "C:/proj/scene.pasta" }),
            "R5.1/R5.2: 対応ありフレームは `.pasta` パスを提示する"
        );
        assert_eq!(resolved.line, 7, "R5.1/R5.2: 提示行は `.pasta` 行 (pos.line)");
    }

    /// R5.3: 対応の無い `(source, line)` は既定 `.lua` resolver と **完全に同一**の
    /// 提示（生成 `.lua` の `{path, line}`）へフォールバックし、誤った `.pasta`
    /// 対応づけ（mismap）を行わない。判別可能（`.pasta` ではなく `.lua`）。
    #[test]
    fn pasta_resolver_falls_back_to_lua_for_unmapped() {
        // chunk は一致するが `.lua` 行 99 は未対応 → フォールバック。
        let map = map_with("C:/proj/cache/scene.lua", 12, "C:/proj/scene.pasta", 7);
        let resolver = pasta_source_resolver(Arc::new(map));

        let resolved = resolver(r"@C:\proj\cache\scene.lua", 99);
        let expected = default_source_resolver()(r"@C:\proj\cache\scene.lua", 99);
        assert_eq!(
            resolved, expected,
            "R5.3: 未対応行は既定 `.lua` resolver と同一の提示へフォールバックする"
        );
        // 念のため：誤った `.pasta` ではなく生成 `.lua` source を保持している。
        assert_eq!(resolved.source, json!({ "path": r"@C:\proj\cache\scene.lua" }));
        assert_eq!(resolved.line, 99);
    }

    /// R5.3（整合性エラー・design 610/617）: chunk 名がマップに無い場合も `.lua`
    /// フォールバック（誤マッピング禁止）。
    #[test]
    fn pasta_resolver_falls_back_to_lua_for_unknown_chunk() {
        let map = map_with("C:/proj/cache/scene.lua", 12, "C:/proj/scene.pasta", 7);
        let resolver = pasta_source_resolver(Arc::new(map));

        let resolved = resolver("@C:/proj/cache/other.lua", 12);
        let expected = default_source_resolver()("@C:/proj/cache/other.lua", 12);
        assert_eq!(
            resolved, expected,
            "R5.3: 未知 chunk は `.lua` フォールバック（誤マッピング禁止）"
        );
    }

    /// R5.2: `pasta_source_resolver` を装着した `DapAdapter` で `stackTrace` を
    /// エンコードすると、各フレームが個別に `.pasta`／`.lua` で提示される
    /// （対応ありは `.pasta`、対応なしは `.lua` フォールバック）。
    #[test]
    fn stack_trace_with_pasta_resolver_presents_each_frame() {
        let map = map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3);
        let mut dap = DapAdapter::new();
        dap.set_source_resolver(pasta_source_resolver(Arc::new(map)));

        dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));
        let out = dap.encode_event(SessionEvent::Stack(vec![
            // 対応あり（`.lua` 7 → `.pasta` 3）。
            FrameInfo {
                source: r"@C:\proj\cache\scene.lua".to_string(),
                line: 7,
                func_name: Some("talk".to_string()),
            },
            // 対応なし（`.lua` 2 は未登録）→ `.lua` フォールバック。
            FrameInfo {
                source: r"@C:\proj\cache\scene.lua".to_string(),
                line: 2,
                func_name: None,
            },
        ]));
        let resp = &out[0];
        let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
        // フレーム 0: `.pasta` 提示（R5.2）。
        assert_eq!(frames[0]["source"], json!({ "path": "C:/proj/scene.pasta" }));
        assert_eq!(frames[0]["line"], 3);
        // フレーム 1: 対応なし → 生成 `.lua` 提示（R5.3 判別可能フォールバック）。
        assert_eq!(frames[1]["source"], json!({ "path": r"@C:\proj\cache\scene.lua" }));
        assert_eq!(frames[1]["line"], 2);
    }
}
