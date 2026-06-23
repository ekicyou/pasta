//! The debug backend enablement entry point and full thread wiring.
//!
//! Split out of `debug/mod.rs` (task 4.5): the [`enable`] startup entry and all
//! of its wiring. The parent `mod.rs` keeps `pub use enable::enable;` so the
//! public surface is byte-identical.
//!
//! Because `enable.rs` is a SIBLING of `handle.rs` (not a descendant), it cannot
//! build the [`DebugHandle`] struct literal directly; it constructs the handle
//! via the sanctioned `pub(crate)` [`DebugHandle::new`](super::handle::DebugHandle::new)
//! seam (design "Components / C3"), passing the same field values in the same
//! order.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use serde_json::Value;

use super::breakpoints::BreakpointSet;
use super::config::DebugConfig;
use super::dap::DapAdapter;
use super::error::DebugError;
use super::handle::DebugHandle;
use super::session::DebugSession;
use super::source_map;
use super::source_mode::SharedSourceMode;
use super::transport::Transport;
use super::types::{SessionCommand, SessionEvent};
use super::wiring;

/// Enable the debug backend for `lua` according to `cfg` (task 4.1 full wiring).
///
/// - When `cfg.enabled == false`: returns `Ok(None)`. No VM hook is installed,
///   no port is opened, no thread is spawned, and `std_debug` is NOT exposed to
///   scripts. This is the true zero-cost path (R5.2 / R5.3 / R5.5).
/// - When `cfg.enabled == true`: builds a FULLY WIRED backend and returns
///   `Ok(Some(DebugHandle))`:
///   1. a shared [`BreakpointSet`] (settable while the VM runs),
///   2. a [`DebugSession`] over the VM-thread ends of the command/event
///      channels, installed into the line hook via
///      [`hook::install`](crate::debug::hook::install) (engine-wide `jit.off()` +
///      a coroutine-crossing `EVERY_LINE` hook) — this is the VM-thread stop
///      core; inspect/step/continue are processed in its hook loop ON THIS
///      THREAD (the `mlua::Lua` never crosses a thread, R6 / `!Send`),
///   3. a [`Transport`] bound to `cfg.listen` (the OS-assigned port is readable
///      via [`DebugHandle::local_addr`] when `listen` uses port 0),
///   4. a shared [`DapAdapter`] and two bridge threads connecting the transport
///      to the session (see [`wiring`] for the thread topology).
///
/// # Thread topology (design "Architecture" / "System Flows")
///
/// One VM host thread (the caller, owns `mlua::Lua` and the session in the hook) +
/// one socket-bridge thread (sole [`Transport`] owner: multiplexes inbound
/// socket reads and outbound socket writes, since `Transport` is `!Sync` and
/// `mpsc` has no `select`) + one event-encoder thread (session events → DAP
/// frames). The socket bridge and encoder share the [`DapAdapter`] behind an
/// `Arc<Mutex<…>>` (its `seq` + correlation table). See [`wiring`] for the full
/// topology and the inbound-poll / outbound-frame-channel structure.
///
/// # SHIORI independence (R6)
///
/// This function does not import or reference `pasta_shiori`; any pasta host (or
/// a test harness) drives it directly.
///
/// # Preconditions
/// `lua` must already be constructed on the VM thread.
///
/// # Source map injection (task 4.2 — `pasta-source-map`)
///
/// `source_map` is the OPTIONAL immutable shared `.pasta`↔`.lua` map (design
/// "Architecture": `Arc<SourceMap>` 不変共有). Together with `cfg.source_mode`
/// (task 4.1) it is threaded to the three `.pasta` CONSUMERS — the DAP source
/// resolver (task 5.2), the breakpoint translator (task 5.3) and the stepper
/// (task 5.4) — via this injection path: `enable → wiring → DebugSession`
/// (design 548). The map+mode REACH those points only when BOTH a map is
/// supplied AND `cfg.source_mode == SourceMode::Pasta` (design 582, requirements
/// 6.1); for `None` or [`SourceMode::Lua`] every consumer keeps its existing
/// default `.lua` behavior byte-for-byte (requirements 6.2 / 7.2). This task
/// wires the SKELETON only — the consumer LOGIC is tasks 5.x.
///
/// # Errors
/// [`DebugError::Bind`] if the DAP listener fails to bind; [`DebugError::Vm`] if
/// the hook install fails (`mlua::Error` is stringified at the boundary, it is
/// `!Send`). The disabled path never errors.
///
/// # Scene-kick sink injection (pasta-scene-kick tasks 2.3 / 2.4 — `KickSinkSeam`)
///
/// `kick_sink` is the OPTIONAL host-injected scene-kick closure
/// ([`KickSink`](crate::debug::kick::KickSink)). When supplied AND debug is
/// enabled, it is threaded to the socket-bridge thread so an inbound
/// `pasta/playScene` request invokes it (pasta-scene-kick R2.4); `pasta_lua`
/// holds it opaquely and never references `pasta_shiori` (R2.4 dependency
/// direction). On the disabled path the sink is dropped here unused, keeping the
/// kick path inert (R2.6, zero cost).
pub fn enable(
    lua: &mlua::Lua,
    cfg: &DebugConfig,
    source_map: Option<Arc<source_map::SourceMap>>,
    kick_sink: Option<crate::debug::kick::KickSink>,
) -> Result<Option<DebugHandle>, DebugError> {
    if !cfg.enabled {
        // Zero-cost disabled path (R5.2 / R5.3 / R5.5): no hook, no port, no
        // thread, no std_debug exposure. Leave `lua` untouched. The `source_map`
        // and `kick_sink` (if any) are simply dropped here — the disabled gate
        // never consumes them, so the kick path stays inert (R2.6).
        return Ok(None);
    }

    // Effective present-mode cell (task 5.5 / requirement 6.3): initialise the
    // SHARED, interior-mutable mode from the resolved `cfg.source_mode` (env >
    // file > 既定). The socket bridge flips it when a DAP `attach`
    // `sourcePresentation` arrives (highest precedence, design 581); the resolver
    // (task 5.2) and the VM-thread stepper (task 5.4) both read it, so an `attach`
    // switches BOTH for this session. One clone goes to the wiring, one to the
    // session.
    let shared_mode = SharedSourceMode::new(cfg.source_mode);

    // Gating (design 582, requirements 6.1 / 6.2 / 6.3): the `.pasta` consumers
    // (resolver / BP translator / stepper) are reached only when a map is supplied
    // AND the EFFECTIVE mode is `SourceMode::Pasta`. The mode part is now decided
    // at CONSUMPTION time (`pasta_active()` reads the shared cell) rather than
    // frozen here, because a DAP `attach` `sourcePresentation` can flip the mode
    // AFTER `enable` (it arrives later) — including Lua→Pasta, which needs the map
    // available. So the map is ALWAYS threaded when supplied; the per-consumption
    // `pasta_active()` gate (map present AND effective mode Pasta) keeps `None`/
    // `Lua`/no-attach paths byte-for-byte (7.2). Cloning the `Arc` is a refcount
    // bump (immutable shared map).
    let source_map_wiring = wiring::SourceMapWiring {
        source_map: source_map.clone(),
        source_mode: shared_mode.clone(),
    };

    // (1) Shared breakpoint store: one clone goes to the VM-thread hook (reads),
    // one clone to the handle / socket bridge (writes — settable while running).
    let breakpoints = BreakpointSet::new();

    // (2) Channel seam (the ONLY thing that crosses the VM/transport boundary):
    //   cmd:   controller (socket bridge) → session (VM thread)
    //   event: session (VM thread) → controller (event encoder)
    let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
    let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

    // A clone of the event sender for the handle's teardown `terminated` (task
    // 4.2). The session keeps the original `event_tx`; this clone outlives the VM
    // thread (it lives on the handle) so `Drop` can emit `Terminated` even after
    // the VM has finished executing — the event-encoder thread stays alive as long
    // as ANY `Sender` (this clone) is held, then winds down when the handle drops.
    let terminate_tx = event_tx.clone();

    // (3) The stop core: a DebugSession over the VM-thread channel ends, plugged
    // into the line hook. install() applies engine-wide jit.off() and registers
    // the coroutine-crossing EVERY_LINE hook (R1.7 / R5.2). The session is moved
    // INTO the hook closure and thereafter lives on this VM thread inside `lua`.
    // The session is the STEPPER consumer (task 5.4 / 5.5): thread the map plus
    // the SHARED effective mode into it. The map is threaded whenever supplied
    // (the `effective_mode == Pasta` gate is applied per line via
    // `resolve_current_pasta`), so a DAP `attach` Lua→Pasta flip can activate
    // `.pasta` stepping; `with_shared_mode` lets the socket-bridge `attach` flip
    // be observed here. With no map / `Lua` effective mode the session keeps its
    // default `.lua` granularity (7.2). The baked `source_mode` is the `attach`-
    // absent fallback (matches the env > file > 既定 resolution).
    let session = DebugSession::new(breakpoints.clone(), cmd_rx, event_tx)
        .with_source_map(source_map.clone(), cfg.source_mode)
        .with_shared_mode(Some(shared_mode.clone()));
    crate::debug::hook::install(lua, session).map_err(|e| DebugError::Vm(e.to_string()))?;

    // (4) I/O side: bind the transport (None → no port; Some → bind + accept one
    // client). A bind failure surfaces as DebugError::Bind (R3.1 / R5.5). The
    // bound addr is read NOW and stored in the handle, because the transport is
    // moved into the socket-bridge thread (it is `!Sync`, single-owner).
    let transport = Transport::start(cfg.listen).map_err(|e| {
        // 2.1/2.3 (failure warn): name the attempted bind addr + io cause, then
        // propagate `DebugError::Bind` unchanged. `cfg.listen` is `Option`, so
        // bind it (the enabled gate guarantees `Some` — R5.5 only materialises a
        // listen addr when enabled) before applying `%` (Display).
        let Some(listen) = cfg.listen else {
            unreachable!("enabled => cfg.listen is Some (R5.5)")
        };
        tracing::warn!(addr = %listen, error = %e, "debug transport bind failed");
        e
    })?;
    let local_addr = transport.local_addr();

    // 1.1/1.3/1.4/1.5 (success info): one line carrying the REAL bound loopback
    // addr (`local_addr()`'s `Some`, defensively matched). On port 0 this is the
    // OS-assigned port read back from the transport.
    if let Some(addr) = local_addr {
        tracing::info!(addr = %addr, "debug backend listening");
    }

    // (5) Shared DAP adapter (seq counter + per-kind FIFO request correlation),
    // mutated by BOTH the socket bridge and the event encoder → Arc<Mutex<…>>.
    let adapter: wiring::SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));

    // (6) Encoded-frame channel: the event encoder produces DAP frames; the
    // socket bridge (sole Transport owner) writes them to the socket.
    let (out_tx, out_rx) = mpsc::channel::<Value>();

    // (7) Shared shutdown flag (non-blocking teardown via the handle's Drop).
    let shutdown = Arc::new(AtomicBool::new(false));

    // (8) Socket-bridge thread: owns the Transport; multiplexes inbound decode
    // (reply / apply setBreakpoints / forward stop-context commands) and
    // outbound frame writes.
    let socket_handle = {
        let adapter = Arc::clone(&adapter);
        let breakpoints = breakpoints.clone();
        let shutdown = Arc::clone(&shutdown);
        // The socket bridge owns the DapAdapter (source RESOLVER attach point,
        // task 5.2) and applies setBreakpoints (BP TRANSLATION attach point, task
        // 5.3): deliver the gated map+mode there too (task 4.2 plumbing).
        let source_map_wiring = source_map_wiring.clone();
        // pasta-scene-kick tasks 2.3 / 2.4: deliver the (optional) host kick sink
        // to the bridge so an inbound `pasta/playScene` invokes it (R2.4). `None`
        // keeps the kick path inert (R2.6). Cloning an `Arc<dyn Fn…>` is a
        // refcount bump.
        let kick_sink = kick_sink.clone();
        std::thread::spawn(move || {
            wiring::run_socket_bridge(
                transport,
                adapter,
                breakpoints,
                cmd_tx,
                out_rx,
                shutdown,
                source_map_wiring,
                kick_sink,
            );
        })
    };

    // (9) Event-encoder thread: session events → DAP frames → out_tx.
    let encoder_handle = {
        let adapter = Arc::clone(&adapter);
        std::thread::spawn(move || {
            wiring::run_event_encoder(adapter, event_rx, out_tx);
        })
    };

    Ok(Some(DebugHandle::new(
        cfg.clone(),
        local_addr,
        shutdown,
        Some(socket_handle),
        Some(encoder_handle),
        terminate_tx,
    )))
}
