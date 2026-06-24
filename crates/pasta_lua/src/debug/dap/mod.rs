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
//! [`SessionCommand`]: crate::debug::types::SessionCommand
//! [`SessionEvent`]: crate::debug::types::SessionEvent
//!
//! # Hand-written, dependency-minimal (design "依存最小")
//!
//! DAP messages are built and parsed by hand with the already-present
//! `serde_json`. The `dap` crate (and any other heavy DAP dependency) is
//! deliberately NOT used, keeping the supply chain and distribution size small.
//!
//! # Module layout (C3 directory module)
//!
//! This hub holds the [`DapAdapter`] type and its envelope core (`new` /
//! `next_seq` / `response` / `event` and the `Debug`/`Default` impls). The
//! responsibility submodules are:
//!
//! - [`resolver`] — the DAP-presentation source seam ([`ResolvedSource`] /
//!   [`SourceResolver`] / [`default_source_resolver`] / [`pasta_source_resolver`]).
//! - [`pending`] — deferred-response correlation (`PendingKind` / `PendingTable`).
//! - [`decode`] — inbound request decode ([`Decoded`] + `decode_request` and the
//!   `pasta/sourcePresentation` envelope builders).
//! - [`encode`] — outbound event encode (`encode_event`).
//! - [`codec`] — the hand-written JSON parse/encode free-function helpers.
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
//! `seq` in a small FIFO `PendingTable`, keyed by the event KIND the request
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

mod codec;
mod decode;
mod encode;
mod pending;
mod resolver;

// `Decoded` is the public return type of `decode_request` (kept reachable as
// `crate::debug::dap::Decoded`, unchanged public API). In a non-test lib build
// the re-export has no in-crate consumer — only the externalized `#[cfg(test)]`
// dap tests reference it via `use super::*;` — so silence the false-positive
// unused-import lint without widening or narrowing the public surface.
#[allow(unused_imports)]
pub use decode::Decoded;
// `RELOAD_SENTINEL` is the reserved kick scene string for SHIORI reload (task
// 4.3); the wiring (`try_reload_shiori`) and the reload wiring tests reference it
// via `crate::debug::dap::RELOAD_SENTINEL`.
#[allow(unused_imports)]
pub use decode::RELOAD_SENTINEL;
pub use resolver::{
    ResolvedSource, SourceResolver, default_source_resolver, pasta_source_resolver,
};

use serde_json::{Value, json};

use self::pending::PendingTable;

// The externalized `#[cfg(test)]` dap test clusters resolve their referenced
// types through `use super::*;` (this `dap` hub). In the original flat `dap.rs`
// these names were brought into module scope by the production `use` statements
// that now live in the child submodules; re-introduce them here, test-gated, so
// the test glob keeps resolving them WITHOUT adding any non-test import or
// widening the public surface (Task 4.4 test re-wiring).
#[cfg(test)]
use crate::debug::SourceMode;
#[cfg(test)]
use crate::debug::types::{
    ResolvedBreakpoint, SessionCommand, SessionEvent, SourceRef, StopReason, ThreadInfo, Variable,
};

/// Hand-written DAP minimal-subset adapter (design "Transport & DapAdapter").
///
/// Translates inbound DAP request [`Value`]s into [`SessionCommand`]s (+ optional
/// immediate response) and outbound [`SessionEvent`]s into DAP response/event
/// [`Value`]s, correlating deferred responses to their originating request `seq`.
/// Stateful only in the small bookkeeping it must own: the monotonic outgoing
/// `seq` counter and the `PendingTable`.
///
/// [`SessionCommand`]: crate::debug::types::SessionCommand
/// [`SessionEvent`]: crate::debug::types::SessionEvent
pub struct DapAdapter {
    /// Monotonic outgoing sequence counter for every response/event emitted.
    out_seq: u64,
    /// Pending request seqs awaiting their deferred [`SessionEvent`].
    ///
    /// [`SessionEvent`]: crate::debug::types::SessionEvent
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
    ///
    /// [`encode_frames`]: codec::encode_frames
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
}

// Inline `#[cfg(test)] mod tests` was externalized into logical-cluster sibling
// files (Task 2.2, pure behavior-invariant move). Each sibling begins with
// `use super::*;` and keeps the same module path, preserving private/`pub(crate)`
// reachability into this production module. The cluster-shared `request` builder
// lives in `dap_test_support` (`pub(super)`); each cluster `use`s it. The set of
// leaf test-fn names and the total test count are unchanged. The test files stay
// in `debug/`, so from this `dap/mod.rs` hub the `#[path]` is `../dap_*_tests.rs`
// (Task 4.4 directory-module move).
#[cfg(test)]
#[path = "../dap_test_support.rs"]
mod dap_test_support;

#[cfg(test)]
#[path = "../dap_protocol_tests.rs"]
mod dap_protocol_tests;

#[cfg(test)]
#[path = "../dap_source_presentation_tests.rs"]
mod dap_source_presentation_tests;

#[cfg(test)]
#[path = "../dap_source_resolver_tests.rs"]
mod dap_source_resolver_tests;

#[cfg(test)]
#[path = "../dap_play_scene_tests.rs"]
mod dap_play_scene_tests;

#[cfg(test)]
#[path = "../dap_edge_tests.rs"]
mod dap_edge_tests;
