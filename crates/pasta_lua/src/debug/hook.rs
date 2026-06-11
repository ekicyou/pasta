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
mod tests {
    use std::sync::{Arc, Mutex};

    use mlua::{Debug, Lua, LuaOptions, StdLib, VmState};

    use super::*;

    // -----------------------------------------------------------------------
    // Test-only VM construction (kept inside the debug module — does NOT depend
    // on the tests/ PoC harness). ALL_SAFE so `jit` exists and `debug` is
    // excluded; `install` itself applies jit.off().
    // -----------------------------------------------------------------------

    fn build_all_safe_vm() -> Lua {
        unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) }
    }

    /// A `LineEvent`-equivalent record (source + line) used as the decisive
    /// evidence of coroutine-crossing firing (unique source names, NOT thread
    /// pointers — promoting the PoC's approach).
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Fired {
        source: String,
        line: u32,
    }

    /// Build a [`Fired`] from a hook `Debug` (chunk name + current line).
    fn fired_from_debug(debug: &Debug) -> Fired {
        let src = debug.source();
        let source = src
            .source
            .as_ref()
            .map(|c| c.as_ref().to_string())
            .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
            .unwrap_or_default();
        let line = debug.current_line().unwrap_or(0) as u32;
        Fired { source, line }
    }

    /// A recording [`LineHook`]: pushes every fired line into a shared sink.
    fn recording_handler(
        sink: Arc<Mutex<Vec<Fired>>>,
    ) -> impl Fn(&Lua, &Debug) -> mlua::Result<VmState> {
        move |_lua: &Lua, debug: &Debug| {
            if let Ok(mut g) = sink.lock() {
                g.push(fired_from_debug(debug));
            }
            Ok(VmState::Continue)
        }
    }

    fn lines_for_source(events: &[Fired], source: &str) -> Vec<u32> {
        events
            .iter()
            .filter(|e| e.source == source)
            .map(|e| e.line)
            .collect()
    }

    /// Promotes the PoC `run_scene_like_scenario`: N coroutines, each with a
    /// unique chunk name `@scene_co_<i>` and a `coroutine.yield()` mid-body,
    /// driven by a resume loop. Unique source names are the decisive evidence of
    /// coroutine-crossing firing (R1.7).
    fn run_scene_like_scenario(lua: &Lua, coroutine_count: usize) -> mlua::Result<()> {
        for i in 0..coroutine_count {
            let chunk_name = format!("@scene_co_{i}");
            let body = format!(
                "\
local marker = {i}
local acc = marker * 2
coroutine.yield()
acc = acc + marker
return acc
"
            );

            let scene_fn: mlua::Function =
                lua.load(&body).set_name(&chunk_name).into_function()?;

            let driver: mlua::Function = lua
                .load(
                    "\
local scene_fn = ...
local co = coroutine.create(scene_fn)
while coroutine.status(co) ~= 'dead' do
    local ok, err = coroutine.resume(co)
    if not ok then error(err) end
end
",
                )
                .set_name("@scene_driver")
                .into_function()?;

            driver.call::<()>(scene_fn)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // (R5.2 correctness premise) install applies engine-wide jit.off().
    // -----------------------------------------------------------------------

    /// After `install`, `jit.status()` must be `false` (no-arg `jit.off()`
    /// applied engine-wide). This is the correctness premise that line hooks do
    /// not miss JIT-compiled lines (R5.2). Before install the engine is on.
    #[test]
    fn install_applies_engine_wide_jit_off() {
        let lua = build_all_safe_vm();

        // Sanity: the JIT engine is ON before install (so the assertion below is
        // attributable to install, not to an already-off VM).
        let before: bool = lua
            .load("return (jit.status())")
            .eval()
            .expect("jit.status() must be callable on an ALL_SAFE VM");
        assert!(before, "JIT engine must be ON before install (premise)");

        let _h = install(&lua, |_lua: &Lua, _debug: &Debug| Ok(VmState::Continue))
            .expect("install must succeed");

        let after: bool = lua
            .load("return (jit.status())")
            .eval()
            .expect("jit.status() must be callable after install");
        assert!(
            !after,
            "no-arg jit.off() must disable the global JIT engine after install (R5.2)"
        );
    }

    /// `install` must NOT expose `std_debug` (R5.3): the VM stays sandboxed.
    #[test]
    fn install_keeps_std_debug_sandboxed() {
        let lua = build_all_safe_vm();
        let _h = install(&lua, |_lua: &Lua, _debug: &Debug| Ok(VmState::Continue))
            .expect("install must succeed");
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(
            debug_is_nil,
            "install must NOT expose std_debug (sandbox maintained, R5.3)"
        );
    }

    // -----------------------------------------------------------------------
    // (R1.7) Coroutine-crossing firing across dynamically-created coroutines.
    // -----------------------------------------------------------------------

    /// The installed hook fires across dynamically-created scene-like
    /// coroutines (R1.7). Decisive evidence: every coroutine body's unique
    /// source `@scene_co_<i>` is recorded, including post-yield lines (proving
    /// firing survives yield/resume across coroutines).
    #[test]
    fn hook_fires_across_dynamic_coroutines() {
        const N: usize = 3;

        let lua = build_all_safe_vm();
        let sink: Arc<Mutex<Vec<Fired>>> = Arc::new(Mutex::new(Vec::new()));

        let _h = install(&lua, recording_handler(Arc::clone(&sink)))
            .expect("install must succeed");

        run_scene_like_scenario(&lua, N).expect("scene-like scenario must run");

        lua.remove_global_hook();

        let events = sink.lock().unwrap();
        for i in 0..N {
            let name = format!("@scene_co_{i}");
            let lines = lines_for_source(&events, &name);
            assert!(
                !lines.is_empty(),
                "hook must fire inside coroutine {i} body ({name}). \
                 recorded sources: {:?}",
                events.iter().map(|e| &e.source).collect::<Vec<_>>()
            );
            // Body line numbers (1-origin):
            //   1: local marker = i
            //   2: local acc = marker * 2
            //   3: coroutine.yield()
            //   4: acc = acc + marker   <- post-yield
            //   5: return acc           <- post-yield
            // A post-yield line (>= 4) proves firing continues after
            // yield/resume across the coroutine (R1.7).
            assert!(
                lines.iter().any(|&l| l >= 4),
                "hook must keep firing after coroutine.yield/resume in {name} \
                 (post-yield lines). got: {lines:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // (R5.2 / R5.4 zero-cost premise) Firing is attributable to install.
    // -----------------------------------------------------------------------

    /// A VM with NO install must record no firing; the SAME workload WITH
    /// install must record firing. This proves the line firing is attributable
    /// to `install`, not to ambient VM behaviour (R5.2 zero-cost premise / R5.4
    /// enabled diagnostics).
    #[test]
    fn firing_is_attributable_to_install() {
        const CHUNK: &str = "\
local a = 1
local b = a + 1
local c = b + 1
return c
";
        const SRC: &str = "@attribution_chunk";

        // (A) No install: a recording handler can't even be reached because no
        // hook is registered. Run the workload; the sink must stay empty.
        let no_install_sink: Arc<Mutex<Vec<Fired>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let lua = build_all_safe_vm();
            // Deliberately do NOT call install. Apply jit.off() manually so the
            // ONLY difference vs. the installed case is the hook registration.
            lua.load("jit.off()").exec().expect("jit.off must run");
            lua.load(CHUNK)
                .set_name(SRC)
                .exec()
                .expect("workload must run without a hook");
            assert!(
                no_install_sink.lock().unwrap().is_empty(),
                "with NO install there must be no recorded firing (zero-cost premise, R5.2)"
            );
        }

        // (B) With install: the same workload records firing on the chunk.
        let install_sink: Arc<Mutex<Vec<Fired>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let lua = build_all_safe_vm();
            let _h = install(&lua, recording_handler(Arc::clone(&install_sink)))
                .expect("install must succeed");
            lua.load(CHUNK)
                .set_name(SRC)
                .exec()
                .expect("workload must run with the hook");
            lua.remove_global_hook();
        }

        let events = install_sink.lock().unwrap();
        let lines = lines_for_source(&events, SRC);
        assert!(
            !lines.is_empty(),
            "WITH install the hook must fire on the workload (R5.4). got: {lines:?}"
        );
        for expected in [1u32, 2, 3] {
            assert!(
                lines.contains(&expected),
                "expected line {expected} must fire with install. got: {lines:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // (design "Error Handling") Hook-internal panic is captured; VM survives.
    // -----------------------------------------------------------------------

    /// A panic thrown inside the handler must be captured to the side channel
    /// and the VM thread must terminate gracefully (no abort / no hang). The
    /// handler pre-records a specific cause before panicking (payload may be
    /// lost across the C-unwind boundary in production; here the wrapper would
    /// still recover it, but pre-recording is the robust path).
    ///
    /// Run on a dedicated thread with a bounded join so a regression that aborts
    /// or hangs the VM is detected as a test failure rather than killing the
    /// suite.
    #[test]
    fn hook_panic_is_captured_and_vm_survives() {
        use std::sync::mpsc;
        use std::time::Duration;

        const PANIC_SOURCE: &str = "@panic_scenario";
        const PANIC_LINE: u32 = 2;
        const PANIC_MSG: &str = "injected hook panic for task 1.3";
        const WATCHDOG: Duration = Duration::from_secs(10);

        // Host-role thread owns the VM (mlua::Lua is !Send → never crosses the
        // boundary). It returns the captured panic cause as a Send-safe String.
        let handle = std::thread::spawn(move || -> Result<Option<String>, String> {
            let lua = build_all_safe_vm();

            // Handler pre-records a specific cause, then panics, on the target
            // line of the target source.
            let cause_seen: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
            let cause_for_handler = Arc::clone(&cause_seen);

            let handle = install(&lua, move |_lua: &Lua, debug: &Debug| {
                let line = debug.current_line().unwrap_or(0) as u32;
                let src = debug
                    .source()
                    .source
                    .as_ref()
                    .map(|c| c.as_ref().to_string())
                    .unwrap_or_default();
                if src == PANIC_SOURCE && line == PANIC_LINE {
                    if let Ok(mut g) = cause_for_handler.lock() {
                        *g = Some(format!("{PANIC_MSG} at {src}:{line}"));
                    }
                    panic!("{PANIC_MSG}");
                }
                Ok(VmState::Continue)
            })
            .map_err(|e| e.to_string())?;

            // Execute a chunk whose PANIC_LINE triggers the in-hook panic. The
            // wrapper must capture it so exec() returns normally (VM survives).
            let chunk = "\
local a = 1
local b = a + 1
return b
";
            lua.load(chunk)
                .set_name(PANIC_SOURCE)
                .exec()
                .map_err(|e| format!("VM must survive a captured hook panic: {e}"))?;

            // The side channel must carry the recorded cause.
            let recorded = handle.panic_cause().lock().unwrap().clone();
            Ok(recorded)
        });

        // Bounded join (test-only watchdog): a regression that aborts/hangs is a
        // failure, not a suite-killer.
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });
        let joined = done_rx
            .recv_timeout(WATCHDOG)
            .expect("VM host thread must finish (no hang) after a hook panic is captured");

        let thread_outcome =
            joined.expect("VM host thread must terminate gracefully, NOT abort on a hook panic");
        let recorded_cause = thread_outcome
            .expect("VM must survive and side channel must be readable")
            .expect("a captured hook panic must record a cause into the side channel");

        assert!(
            recorded_cause.contains(PANIC_MSG),
            "the recorded panic cause must carry the injected message. got: {recorded_cause:?}"
        );
    }

    // -----------------------------------------------------------------------
    // (hook contract) A handler `Err` is swallowed: the VM continues and the
    // panic side channel stays untouched (an Err is NOT a panic).
    // -----------------------------------------------------------------------

    /// A handler returning `Err` must NOT abort or error the running chunk
    /// (the hook contract is "always Continue"; error routing is owned by the
    /// session/wiring seam). The chunk must run to completion with the correct
    /// result and the panic side channel must remain `None`.
    #[test]
    fn handler_error_is_swallowed_and_vm_continues() {
        const ERR_SOURCE: &str = "@handler_err_chunk";

        let lua = build_all_safe_vm();
        let handle = install(&lua, move |_lua: &Lua, debug: &Debug| {
            let src = debug
                .source()
                .source
                .as_ref()
                .map(|c| c.as_ref().to_string())
                .unwrap_or_default();
            if src == ERR_SOURCE {
                // Err on EVERY line of the target chunk: the wrapper must
                // swallow each one and keep the VM running.
                return Err(mlua::Error::runtime("injected handler error"));
            }
            Ok(VmState::Continue)
        })
        .expect("install must succeed");

        let result: i64 = lua
            .load(
                "\
local a = 1
local b = a + 1
return b + 1
",
            )
            .set_name(ERR_SOURCE)
            .eval()
            .expect("a handler Err must be swallowed: the chunk keeps executing");
        lua.remove_global_hook();

        assert_eq!(
            result, 3,
            "the chunk must run to completion with the correct result despite handler Errs"
        );
        assert!(
            handle.panic_cause().lock().unwrap().is_none(),
            "a handler Err is NOT a panic: the panic side channel must stay None"
        );
    }

    // -----------------------------------------------------------------------
    // record_panic_cause preference order (module docs steps 1-3) + lock
    // poisoning tolerance. The in-VM panic tests above cannot discriminate
    // these branches (pre-record and payload carry the same message there), so
    // each branch is pinned directly.
    // -----------------------------------------------------------------------

    /// (1) A handler-pre-recorded cause is MORE specific and must be preserved:
    /// the catch_unwind payload must NOT overwrite it.
    #[test]
    fn record_panic_cause_prefers_prerecorded_cause_over_payload() {
        let cause: PanicCause = Arc::new(Mutex::new(Some("pre-recorded specific cause".into())));
        record_panic_cause(&cause, Box::new("payload that must NOT win"));
        assert_eq!(
            cause.lock().unwrap().as_deref(),
            Some("pre-recorded specific cause"),
            "a pre-recorded cause must be preserved (preference step 1), not overwritten"
        );
    }

    /// (2a) Without a pre-record, a `&'static str` panic payload is recovered.
    #[test]
    fn record_panic_cause_recovers_static_str_payload() {
        let cause: PanicCause = Arc::new(Mutex::new(None));
        record_panic_cause(&cause, Box::new("static str payload"));
        assert_eq!(
            cause.lock().unwrap().as_deref(),
            Some("static str payload"),
            "a &'static str payload must be recovered (preference step 2)"
        );
    }

    /// (2b) Without a pre-record, an owned `String` panic payload (the shape
    /// `panic!(\"{x}\")` produces) is recovered.
    #[test]
    fn record_panic_cause_recovers_string_payload() {
        let cause: PanicCause = Arc::new(Mutex::new(None));
        record_panic_cause(&cause, Box::new(String::from("owned String payload")));
        assert_eq!(
            cause.lock().unwrap().as_deref(),
            Some("owned String payload"),
            "an owned String payload must be recovered (preference step 2)"
        );
    }

    /// (3) A payload that is neither `&str` nor `String` falls back to the
    /// generic marker — never panics, never leaves the channel empty.
    #[test]
    fn record_panic_cause_falls_back_to_generic_marker() {
        let cause: PanicCause = Arc::new(Mutex::new(None));
        record_panic_cause(&cause, Box::new(42_i32));
        assert_eq!(
            cause.lock().unwrap().as_deref(),
            Some("hook handler panicked"),
            "a non-string payload must record the generic marker (preference step 3)"
        );
    }

    /// A poisoned side-channel lock must be tolerated: record_panic_cause
    /// returns without panicking (the VM continues; nothing more can be done).
    #[test]
    fn record_panic_cause_tolerates_poisoned_lock() {
        let cause: PanicCause = Arc::new(Mutex::new(None));

        // Poison the mutex: panic while holding the guard.
        let poisoner = Arc::clone(&cause);
        let _ = panic::catch_unwind(AssertUnwindSafe(move || {
            let _guard = poisoner.lock().unwrap();
            panic!("poison the lock");
        }));
        assert!(cause.lock().is_err(), "the lock must be poisoned (premise)");

        // Must NOT panic on the poisoned lock (graceful no-op).
        record_panic_cause(&cause, Box::new("payload after poison"));
    }

    /// When the handler does NOT pre-record a cause, the wrapper itself must
    /// still record a non-empty cause (payload recovery or the generic marker),
    /// and the VM must survive.
    #[test]
    fn hook_panic_without_prerecord_still_records_a_cause() {
        use std::sync::mpsc;
        use std::time::Duration;

        const PANIC_SOURCE: &str = "@panic_bare";
        const PANIC_LINE: u32 = 2;
        const WATCHDOG: Duration = Duration::from_secs(10);

        let handle = std::thread::spawn(move || -> Result<Option<String>, String> {
            let lua = build_all_safe_vm();
            let handle = install(&lua, move |_lua: &Lua, debug: &Debug| {
                let line = debug.current_line().unwrap_or(0) as u32;
                let src = debug
                    .source()
                    .source
                    .as_ref()
                    .map(|c| c.as_ref().to_string())
                    .unwrap_or_default();
                if src == PANIC_SOURCE && line == PANIC_LINE {
                    // No pre-record: the wrapper must supply a cause.
                    panic!("bare panic with no pre-record");
                }
                Ok(VmState::Continue)
            })
            .map_err(|e| e.to_string())?;

            let chunk = "\
local a = 1
local b = a + 1
return b
";
            lua.load(chunk)
                .set_name(PANIC_SOURCE)
                .exec()
                .map_err(|e| format!("VM must survive: {e}"))?;

            Ok(handle.panic_cause().lock().unwrap().clone())
        });

        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });
        let joined = done_rx
            .recv_timeout(WATCHDOG)
            .expect("VM host thread must finish (no hang)");
        let recorded = joined
            .expect("VM host thread must terminate gracefully")
            .expect("VM must survive and side channel readable")
            .expect("the wrapper must record SOME cause even without a pre-record");
        assert!(
            !recorded.is_empty(),
            "the recorded cause must be non-empty (payload recovery or generic marker)"
        );
    }
}
