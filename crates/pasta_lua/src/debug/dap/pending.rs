//! Deferred-response correlation bookkeeping: [`PendingKind`] and the per-kind
//! FIFO [`PendingTable`] that pairs a deferred DAP response back to the
//! originating request `seq` (design "Deferred responses & `request_seq`
//! correlation").

use std::collections::VecDeque;

/// The kind of deferred [`SessionEvent`] a pending request is waiting for.
///
/// Used as the FIFO key in [`PendingTable`] so each deferred response is paired
/// back to the `request_seq` of the request that triggered it.
///
/// [`SessionEvent`]: crate::debug::types::SessionEvent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PendingKind {
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
pub(super) struct PendingTable {
    set_breakpoints: VecDeque<u64>,
    threads: VecDeque<u64>,
    stack_trace: VecDeque<u64>,
    variables: VecDeque<u64>,
}

impl PendingTable {
    /// Record `request_seq` as awaiting the given event `kind`.
    pub(super) fn push(&mut self, kind: PendingKind, request_seq: u64) {
        match kind {
            PendingKind::SetBreakpoints => self.set_breakpoints.push_back(request_seq),
            PendingKind::Threads => self.threads.push_back(request_seq),
            PendingKind::StackTrace => self.stack_trace.push_back(request_seq),
            PendingKind::Variables => self.variables.push_back(request_seq),
        }
    }

    /// Pop the oldest pending `request_seq` for `kind`, if any.
    pub(super) fn pop(&mut self, kind: PendingKind) -> Option<u64> {
        match kind {
            PendingKind::SetBreakpoints => self.set_breakpoints.pop_front(),
            PendingKind::Threads => self.threads.pop_front(),
            PendingKind::StackTrace => self.stack_trace.pop_front(),
            PendingKind::Variables => self.variables.pop_front(),
        }
    }
}
