//! VmHook: line-hook installation with engine-wide `jit.off()`, coroutine-
//! crossing firing, and hook-internal panic capture (design "VmHook").
//!
//! # Responsibilities (design "VmHook", requirements 1.7 / 5.2 / 5.4)
//!
//! [`install`] is the single seam that arms a Lua VM for debugging. It is only
//! ever called when debugging is *enabled* (the caller's responsibility — the
//! zero-cost disabled path in [`crate::debug::enable`] never calls it, R5.2).
//! When called it:
//!
//! 1. Applies **no-arg `jit.off()`** (engine-wide) so line hooks never miss a
//!    JIT-compiled line. This is verifiable via `jit.status()` returning
//!    `false`. The per-function form `jit.off(true, true)` must NOT be used:
//!    PoC knowledge proved it leaves `jit.status()` `true` and does not stop the
//!    engine, so dynamically-loaded scene chunks and coroutines would still be
//!    JIT-compiled and miss line hooks.
//! 2. Installs `lua.set_global_hook(HookTriggers::EVERY_LINE, cb)` — a
//!    coroutine-crossing line hook (R1.7). On LuaJIT, `lua_sethook` acts on the
//!    main state globally, so the hook fires for EVERY executed line across all
//!    coroutines, including ones created Lua-side via `coroutine.create`.
//! 3. The callback ALWAYS returns `Ok(VmState::Continue)` (LuaJIT cannot Yield
//!    from a hook).
//! 4. The callback delegates each line to a per-line [`LineHook`] handler seam
//!    so [`DebugSession`](crate::debug::session::DebugSession) and the wiring
//!    ([`crate::debug::enable`]) plug in WITHOUT rewriting this file. The
//!    seam's call shape is `on_line(lua, &debug)` to match the design intent
//!    `cb: move |lua, debug| { session.on_line(lua, &debug) }`.
//!
//! # Hook-internal panic capture (design "VmHook" + "Error Handling")
//!
//! A panic raised inside the handler must NOT abort the VM process. [`install`]
//! wraps each handler invocation in [`std::panic::catch_unwind`]
//! ([`AssertUnwindSafe`](std::panic::AssertUnwindSafe)); on panic it records a
//! cause string to a `Send`-safe side channel
//! ([`Arc`]`<`[`Mutex`](std::sync::Mutex)`<Option<String>>>`) and returns
//! `Ok(VmState::Continue)` so the VM is not torn down. The recorded side channel
//! is returned to the caller (see [`HookHandle::panic_cause`]) so callers/tests
//! can read recorded panics.
//!
//! ## Panic-cause constraint (PoC knowledge — payload is lost across unwind)
//!
//! On MSVC + LuaJIT the panic payload (`Box<dyn Any + Send>`) is NOT preserved
//! across the `C-unwind` boundary, so a panic message thrown deep in Lua cannot
//! be downcast back to `&str`/`String` after the fact. Our `catch_unwind` runs
//! *in-hook on the VM thread* — before any C-unwind boundary — so it CAN recover
//! the payload here. The wrapper therefore records, in order of preference:
//! 1. a cause the handler *pre-recorded* into the side channel before panicking
//!    (most specific — preserved even if the payload were later lost), else
//! 2. the `catch_unwind` payload downcast to `&str` / `String`, else
//! 3. a generic `"hook handler panicked"` marker.
//!
//! # Sandbox (R5.3)
//!
//! `std_debug` is NEVER exposed by this module. The VM is ALL_SAFE (the `debug`
//! library is excluded); this module relies solely on the Rust-side
//! `set_global_hook` API and never enables `std_debug`.

use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use mlua::{Debug, HookTriggers, Lua, VmState};

/// `Send`-safe channel a hook panic's cause string is recorded into.
///
/// `mlua`/Lua values are `!Send`, but a `String` cause is `Send`, so the cause
/// can later cross the VM/transport thread boundary (design "Error Handling":
/// hook panic → side-channel record + session end, VM process continues).
pub(crate) type PanicCause = Arc<Mutex<Option<String>>>;

/// Per-line handler seam invoked by the installed line hook.
///
/// The call shape is `on_line(lua, &debug)` to match the design intent
/// `cb: move |lua, debug| { session.on_line(lua, &debug) }`.
/// [`DebugSession`](crate::debug::session::DebugSession) implements this trait
/// so it plugs into [`install`] without rewriting this file.
///
/// Implementations should return `Ok(VmState::Continue)` (LuaJIT cannot Yield
/// from a hook). A returned `Err` is swallowed by [`install`]'s wrapper (the
/// hook contract is "always Continue"; error routing is owned by the session);
/// a *panic* is captured by the wrapper and does not abort the VM.
pub(crate) trait LineHook: 'static {
    /// Called once per executed line, on the VM thread.
    fn on_line(&self, lua: &Lua, debug: &Debug) -> mlua::Result<VmState>;
}

/// Blanket impl so a plain closure can be used as a [`LineHook`].
///
/// This lets callers (tests) pass `|lua, debug| { ... }` directly while the
/// wiring passes a `DebugSession` that implements the trait. The bound matches
/// the design seam (`FnMut`-shaped per-line callback) with a single hook entry.
impl<F> LineHook for F
where
    F: Fn(&Lua, &Debug) -> mlua::Result<VmState> + 'static,
{
    fn on_line(&self, lua: &Lua, debug: &Debug) -> mlua::Result<VmState> {
        (self)(lua, debug)
    }
}

/// Handle returned by [`install`], exposing the panic side channel.
///
/// The hook itself lives inside the VM (registered via `set_global_hook`); this
/// handle only carries the `Send`-safe panic-cause channel so callers/tests can
/// observe a hook-internal panic that was captured (R-coverage: hook panic must
/// be recorded without aborting the VM).
#[derive(Clone)]
pub(crate) struct HookHandle {
    // Production wiring (debug::enable → install) currently discards the
    // handle — error routing to the controller is owned by the session seam —
    // so outside test builds this observation channel is intentionally unread.
    #[cfg_attr(not(test), allow(dead_code))]
    panic_cause: PanicCause,
}

impl HookHandle {
    /// The side channel a captured hook panic's cause is recorded into.
    ///
    /// `None` until a handler panic is captured; `Some(cause)` afterwards.
    /// Read by tests (and future error routing); production wiring discards
    /// the handle, hence the non-test dead-code allowance.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn panic_cause(&self) -> &PanicCause {
        &self.panic_cause
    }
}

/// Apply engine-wide `jit.off()` to `lua` (design "Technology Stack": no-arg
/// `jit.off()`).
///
/// Uses the **no-arg** form so the whole JIT engine stops (`jit.status()` →
/// `false`). The per-function form `jit.off(true, true)` must NOT be used: it
/// leaves the engine enabled and only affects the calling function tree, so
/// later-loaded chunks and dynamically-created coroutines would still be JIT-
/// compiled and miss line hooks (R1.7 / R5.2 correctness premise).
fn apply_jit_off(lua: &Lua) -> mlua::Result<()> {
    lua.load("jit.off()").exec()
}

/// Install the debug line hook on `lua` using `handler` as the per-line seam.
///
/// Only call this when debugging is enabled (caller's responsibility; the
/// zero-cost disabled path never calls it — R5.2). Steps:
/// 1. Apply no-arg `jit.off()` (engine-wide; `jit.status()` → `false`).
/// 2. Register `set_global_hook(HookTriggers::EVERY_LINE, cb)` — coroutine-
///    crossing (R1.7).
/// 3. `cb` delegates each line to `handler.on_line(lua, &debug)`, wrapped in
///    [`catch_unwind`](std::panic::catch_unwind) so a handler panic is recorded
///    to the returned [`HookHandle::panic_cause`] side channel and does NOT
///    abort the VM. `cb` ALWAYS returns `Ok(VmState::Continue)` (LuaJIT cannot
///    Yield from a hook); on a handler `Err` it also continues — error routing
///    is owned by the session seam, not by aborting the hook — matching the
///    "always Continue" hook contract.
///
/// `std_debug` is never exposed (R5.3): the VM is ALL_SAFE and only the Rust
/// `set_global_hook` API is used.
pub(crate) fn install<H>(lua: &Lua, handler: H) -> mlua::Result<HookHandle>
where
    H: LineHook,
{
    // (1) Engine-wide jit.off() so line hooks never miss JIT-compiled lines.
    apply_jit_off(lua)?;

    let panic_cause: PanicCause = Arc::new(Mutex::new(None));
    let cb_cause = Arc::clone(&panic_cause);

    // (2) Coroutine-crossing line hook (LuaJIT: lua_sethook is global on the
    //     main state; fires across all coroutines incl. Lua-side
    //     coroutine.create — R1.7).
    lua.set_global_hook(HookTriggers::EVERY_LINE, move |lua, debug| {
        // (3) Delegate to the per-line handler seam, capturing any panic so the
        //     VM is not torn down (design "Error Handling": hook panic →
        //     side-channel record, VM continues).
        let result = panic::catch_unwind(AssertUnwindSafe(|| handler.on_line(lua, debug)));

        match result {
            // Handler returned normally. LuaJIT cannot Yield from a hook, so we
            // normalise to Continue regardless of the handler's VmState. A
            // handler Err is intentionally swallowed here (the hook contract is
            // "always Continue"); error routing to the controller is owned by a
            // later task and would go through the channel seam, not by aborting.
            Ok(Ok(_vm_state)) => Ok(VmState::Continue),
            Ok(Err(_handler_err)) => Ok(VmState::Continue),
            Err(payload) => {
                record_panic_cause(&cb_cause, payload);
                // Do NOT abort the VM: keep executing so the process survives.
                Ok(VmState::Continue)
            }
        }
    })?;

    Ok(HookHandle { panic_cause })
}

/// Record a captured panic's cause into the side channel.
///
/// Preference order (see module docs — payload may be lost across the LuaJIT
/// C-unwind boundary, but here we run in-hook before that boundary):
/// 1. a cause the handler *pre-recorded* into the side channel before panicking
///    (most specific; left untouched), else
/// 2. the `catch_unwind` payload downcast to `&str` / `String`, else
/// 3. a generic `"hook handler panicked"` marker.
fn record_panic_cause(cause: &PanicCause, payload: Box<dyn std::any::Any + Send>) {
    let Ok(mut guard) = cause.lock() else {
        // Poisoned lock: nothing more we can safely do; the VM still continues.
        return;
    };

    // (1) Respect a handler-pre-recorded, more specific cause.
    if guard.is_some() {
        return;
    }

    // (2) Best-effort payload recovery (works here: in-hook, pre C-unwind).
    let recovered = payload
        .downcast_ref::<&'static str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned());

    // (3) Fallback marker.
    *guard = Some(recovered.unwrap_or_else(|| "hook handler panicked".to_string()));
}

#[cfg(test)]
#[path = "hook_tests.rs"]
mod tests;
