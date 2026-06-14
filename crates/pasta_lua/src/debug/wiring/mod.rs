//! Backend wiring: the transport↔dap↔session↔hook bridge threads (task 4.1).
//!
//! This module owns the two `Send`-only bridge threads that connect the
//! I/O-side [`Transport`](crate::debug::transport::Transport) +
//! [`DapAdapter`](crate::debug::dap::DapAdapter) to the VM-side
//! [`DebugSession`](crate::debug::session::DebugSession). The VM-side hook
//! install and the session itself live on the VM thread (the thread that calls
//! [`enable`](crate::debug::enable)); this module never touches `mlua::Lua`
//! (it is `!Send` and stays pinned to the VM thread). Only `std::sync::mpsc`
//! channels carrying `Send` payloads ([`SessionCommand`] / [`SessionEvent`] /
//! [`serde_json::Value`]) cross between the threads here.
//!
//! # Thread topology (design "Architecture" / "System Flows" / スレッドモデル)
//!
//! Three concurrent roles plus the transport's own internal reader thread,
//! connected only by channels:
//!
//! 1. **VM host thread** (caller of `enable`, owns `mlua::Lua`): the line hook
//!    drives [`DebugSession::on_line`]; when stopped, the session processes
//!    inspect/step/continue commands IN the hook loop, on this thread, calling
//!    `inspect::capture_*` on `lua.current_thread()`. It reads the session's
//!    `cmd_rx` and writes the session's `event_tx`.
//! 2. **Socket bridge thread** ([`run_socket_bridge`]): the SOLE owner of the
//!    [`Transport`]. [`Transport`] is `!Sync` (it holds a
//!    `Receiver<Value>`), so it cannot be shared across threads — exactly one
//!    thread owns it and performs BOTH socket reads and socket writes. This
//!    thread multiplexes, per iteration:
//!    - **inbound** (socket → us): a bounded `recv_timeout` poll of
//!      `transport.inbound()`. Each decoded DAP request becomes (a) immediate
//!      response/event frames written straight back, (b) a `setBreakpoints`
//!      applied DIRECTLY to the shared [`BreakpointSet`] (settable while the VM
//!      is RUNNING — design "System Flows": `Arc<Mutex>` 共有) whose DAP response
//!      is produced via the adapter and written back, or (c) a stop-context
//!      [`SessionCommand`] forwarded to the session's `cmd_tx`.
//!    - **outbound** (session → socket): drains the `out_rx` frame channel fed
//!      by the encoder thread and writes each frame to the socket.
//! 3. **Event encoder thread** ([`run_event_encoder`]): drains the session's
//!    `event_rx` ([`SessionEvent`]s), encodes each via the shared [`DapAdapter`]
//!    (`encode_event`) into DAP frames, and pushes them into the `out_tx` frame
//!    channel for the socket bridge to write. It never touches the `Transport`.
//!
//! `std::sync::mpsc` has no `select`, and `Transport` is `!Sync`, so the socket
//! bridge polls inbound with a small timeout ([`bridge::POLL_INTERVAL`]) and
//! drains the encoder's frame channel between polls — the "equivalent structure"
//! to two independent bridge loops. The poll interval is small enough to be
//! imperceptible for interactive debugging and adds no busy-spin (it blocks for
//! the interval when idle).
//!
//! # Shared `DapAdapter` (`Arc<Mutex<…>>`)
//!
//! The adapter is the single stateful correlation point (a monotonic `seq`
//! counter + per-kind FIFO `request_seq` table). It is mutated by BOTH the
//! socket bridge (decoding requests, producing the `setBreakpoints` response)
//! and the encoder thread (encoding events), so it is shared behind an
//! `Arc<Mutex<…>>`.
//!
//! # No double-response to `scopes`
//!
//! `DapAdapter::decode_request("scopes")` SELF-ANSWERS the scopes response at
//! decode time (from the frame id alone) AND still returns a
//! `SessionCommand::Scopes`. The socket bridge sends that self-answer
//! immediately and forwards the `Scopes` command; the session replies with a
//! `SessionEvent::Scopes`, but `DapAdapter::encode_event(Scopes)` is a
//! deliberate no-op (returns no frames), so the client receives EXACTLY one
//! scopes response. The same single-response guarantee holds for `threads`
//! (deferred: only the `SessionEvent::Threads` produces the wire response) and
//! `setBreakpoints` (only the bridge-applied `Breakpoints` event produces it).
//!
//! # `.pasta` presentation seam
//!
//! [`SourceMapWiring`] threads the optional shared `Arc<SourceMap>` plus the
//! shared effective [`SourceMode`] into this module's consumers: the `.pasta`
//! source resolver ([`resolver::attach_pasta_resolver`], task 5.2), the
//! `.pasta`→`.lua` breakpoint translation
//! ([`resolver::translate_pasta_breakpoints`], task 5.3) and the runtime
//! presentation toggle / `attach` override handled in
//! [`inbound::handle_inbound`] (tasks 3.1 / 5.5). Each consumer gates on
//! [`SourceMapWiring::pasta_active`] at use time, so a mode flip is observed
//! immediately; with no map or in `Lua` mode the existing `.lua` behavior is
//! kept byte-for-byte (requirements 6.1 / 6.2 / 7.2).
//!
//! # Shutdown (no hang)
//!
//! A shared [`AtomicBool`](std::sync::atomic::AtomicBool) shutdown flag lets the
//! owner ([`DebugHandle`](crate::debug::handle::DebugHandle)) signal the socket
//! bridge to stop without blocking: the bridge checks it each poll iteration and
//! exits within [`bridge::POLL_INTERVAL`], dropping the `Transport` (which winds
//! the transport down). The encoder thread ends when the session's `event_rx`
//! closes (the VM thread finished and dropped the session) or when the frame
//! channel's receiver is gone.
//!
//! # Module layout (C5 production split)
//!
//! The flat `wiring.rs` was split into responsibility submodules without changing
//! any behavior or public reachability:
//! - [`bridge`] — the transport-owning socket bridge loop + outbound drain + the
//!   event encoder thread (`run_socket_bridge` byte-identical, requirement 4.4).
//! - [`inbound`] — `handle_inbound` and its fixed A→B→C→D→E helper sequence (the
//!   `setBreakpoints` atomic + non-forward arm preserved, requirement 4.1).
//! - [`resolver`] — the `.pasta` source-resolver attach + breakpoint translation.
//!
//! The primary `.pasta`-seam types ([`SourceMapWiring`] / [`SharedAdapter`]) and
//! the `#[path]` test decls stay in this hub; the cross-sibling free fns are
//! re-exported below so external reachability (`crate::debug::wiring::{…}`, used
//! by `enable`) and the externalized tests' `use super::{…}` are unchanged.

use std::sync::Arc;
use std::sync::Mutex;

use crate::debug::dap::DapAdapter;
use crate::debug::source_map::SourceMap;
use crate::debug::{SharedSourceMode, SourceMode};

mod bridge;
mod inbound;
mod resolver;

// Re-export the cross-sibling / external-facing free fns so external reachability
// (`crate::debug::wiring::{…}`, used by `enable`) and the externalized test
// modules' `use super::{…}` keep resolving exactly as in the flat `wiring.rs`.
pub(crate) use bridge::{run_event_encoder, run_socket_bridge};
// The remaining items below are `wiring`-internal (`pub(super)` in their defining
// submodule, consumed by the bridge / by the externalized tests via
// `use super::{…}`). A PRIVATE `use` binds them into this hub's scope so the
// child test modules' `super::` paths resolve unchanged, WITHOUT widening their
// visibility beyond `wiring` (so they never reach the externally-public surface).
#[cfg(test)]
use bridge::drain_outbound;
#[cfg(test)]
use inbound::{handle_inbound, is_pasta_source};
#[cfg(test)]
use resolver::{attach_pasta_resolver, translate_pasta_breakpoints};

/// The (optional) shared source map + present mode threaded from
/// [`enable`](crate::debug::enable) to the I/O-side consumers (task 4.2).
///
/// Carries the immutable `Arc<SourceMap>` and the shared effective [`SourceMode`]
/// to the socket-bridge thread, which owns the [`DapAdapter`] where the `.pasta`
/// source RESOLVER attaches ([`resolver::attach_pasta_resolver`], task 5.2) and
/// applies `setBreakpoints` where the `.pasta` BP TRANSLATION attaches
/// ([`resolver::translate_pasta_breakpoints`], task 5.3) — both consumers live in
/// this module.
///
/// `source_map` is `Some` whenever `enable` was given a map, REGARDLESS of the
/// initial mode: the mode half of the gate is decided at CONSUMPTION time by
/// [`pasta_active`](Self::pasta_active) (design 582), because a DAP `attach`
/// `sourcePresentation` can flip the mode AFTER `enable` — including Lua→Pasta,
/// which needs the map available. With no map, or while the effective mode is
/// [`SourceMode::Lua`], every consumer keeps the existing default `.lua`
/// behavior byte-for-byte (requirements 6.1 / 6.2 / 7.2).
#[derive(Clone)]
pub(crate) struct SourceMapWiring {
    /// Immutable shared map, or `None` for default `.lua` behavior.
    pub(crate) source_map: Option<Arc<SourceMap>>,
    /// The EFFECTIVE present mode for this session (requirements 6.1 default
    /// `.pasta`, 6.2 `.lua`). Shared and interior-mutable so the DAP `attach`
    /// `sourcePresentation` argument (highest precedence, design 581) can flip it
    /// at request time and have BOTH the resolver (task 5.2) and the VM-thread
    /// stepper (task 5.4) observe the same value (task 5.5 / requirement 6.3).
    /// Initialised at [`enable`](crate::debug::enable) time from the resolved
    /// `DebugConfig::source_mode` (env > file > 既定).
    pub(crate) source_mode: SharedSourceMode,
}

impl SourceMapWiring {
    /// The default (disabled) wiring: no map, effective mode default `.pasta` —
    /// but with no map the consumers behave exactly as today's `.lua` path (7.2).
    ///
    /// Test-only constructor: production (`enable` in `mod.rs`) always builds the
    /// wiring via the struct literal with the resolved map/mode, so this shorthand
    /// is exercised only by the in-file test modules.
    #[cfg(test)]
    pub(crate) fn disabled() -> Self {
        Self {
            source_map: None,
            source_mode: SharedSourceMode::new(SourceMode::default()),
        }
    }

    /// Whether the `.pasta` consumers (resolver / BP translation / stepper) should
    /// be active: a map is present AND the EFFECTIVE mode is
    /// [`SourceMode::Pasta`] (design 582). Otherwise every consumer keeps its
    /// default `.lua` behavior. Reads the shared mode each call so an `attach`
    /// flip is observed immediately (task 5.5).
    pub(crate) fn pasta_active(&self) -> bool {
        self.source_map.is_some() && self.source_mode.get() == SourceMode::Pasta
    }
}

/// Shared DAP adapter (seq counter + per-kind FIFO `request_seq` correlation),
/// mutated by BOTH the socket bridge and the event encoder thread.
pub(crate) type SharedAdapter = Arc<Mutex<DapAdapter>>;

#[cfg(test)]
#[path = "../wiring_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../wiring_source_map_wiring_tests.rs"]
mod source_map_wiring_tests;

#[cfg(test)]
#[path = "../wiring_resolver_attach_tests.rs"]
mod resolver_attach_tests;

#[cfg(test)]
#[path = "../wiring_attach_source_presentation_tests.rs"]
mod attach_source_presentation_tests;

#[cfg(test)]
#[path = "../wiring_bp_translator_tests.rs"]
mod bp_translator_tests;

#[cfg(test)]
#[path = "../wiring_pasta_bp_e2e.rs"]
mod pasta_bp_e2e;

#[cfg(test)]
#[path = "../wiring_pasta_step_e2e.rs"]
mod pasta_step_e2e;

#[cfg(test)]
#[path = "../wiring_pasta_mode_edge_e2e.rs"]
mod pasta_mode_edge_e2e;

#[cfg(test)]
#[path = "../wiring_pasta_break_coalesce_e2e.rs"]
mod pasta_break_coalesce_e2e;

#[cfg(test)]
#[path = "../wiring_source_presentation_toggle_tests.rs"]
mod source_presentation_toggle_tests;

#[cfg(test)]
#[path = "../wiring_bridge_lifecycle_tests.rs"]
mod bridge_lifecycle_tests;
