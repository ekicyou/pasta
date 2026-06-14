//! Outbound side: the [`DapAdapter::encode_event`] mapping from [`SessionEvent`]
//! to DAP response/event [`Value`]s (design "Transport & DapAdapter",
//! requirements 3.2 / 3.3 / 3.4 / 3.5).

use serde_json::{Value, json};

use crate::debug::types::SessionEvent;

use super::DapAdapter;
use super::codec::{
    encode_breakpoints, encode_frames, encode_threads, encode_variables, stop_reason_str,
};
use super::pending::PendingKind;

impl DapAdapter {
    /// Encode one outbound [`SessionEvent`] into DAP response/event [`Value`]s.
    ///
    /// Deferred responses ([`SessionEvent::Breakpoints`] / `Threads` / `Stack` /
    /// `Variables`) are correlated back to their originating request `seq` via
    /// the [`PendingTable`]; if no pending request is found (e.g. a spurious or
    /// out-of-band event) the correlation falls back to `request_seq = 0`.
    /// Unsolicited events ([`SessionEvent::Stopped`] / `Terminated` / `Error`)
    /// become DAP events. Each call returns zero or more frames to write to the
    /// transport.
    ///
    /// [`PendingTable`]: super::pending::PendingTable
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
