//! FrameInspector: call stack and variable capture for a stopped Lua frame
//! (design "FrameInspector", requirements 2.1 / 2.2 / 2.3 / 2.5).
//!
//! This module is promoted from the validated PoC
//! (`tests/runtime/lua_debug_poc_test/frame_inspector.rs`). It captures, for a
//! stopped frame, the call stack ([`capture_stack`]) and the visible locals +
//! upvalues ([`capture_variables`]) by **name + type + value**, using only
//! `mlua::ffi` — the Lua `debug` / `std_debug` library is NEVER enabled (R5.3).
//!
//! # Thread-state design (the seam tasks 2.3 AND 2.4 share — R2.4 CONFIRMED)
//!
//! Both public fns take a `&mlua::Thread` and operate on **`thread.state()`**
//! (the raw `*mut lua_State` of the *passed* thread). Task 2.3 passes the
//! main/current thread stopped at a top-level chunk frame; task 2.4 passes a
//! *running Lua-side coroutine's* thread so the SAME traversal reaches its body
//! frame — **no rewrite**, only a caller change.
//!
//! ## R2.4: how a running coroutine's body frame is reached (empirically proven)
//!
//! From inside a `set_global_hook` line hook, `hook_lua.current_thread()`
//! resolves to the **running coroutine** (not the main thread), so passing that
//! `&Thread` makes `thread.state()` the coroutine's own `lua_State` and the
//! 2.3 traversal lands on the body frame. Verified against mlua 0.11.6 source
//! (`state/raw.rs::global_hook_proc` + `state/util.rs::callback_error_ext`):
//! the C hook trampoline receives the *running* `lua_State*` and wraps the user
//! callback in `callback_error_ext`, which installs a `StateGuard` that
//! **swaps `RawLua.state` to that running state** for the callback's duration.
//! `current_thread()` reads `RawLua.state` and `lua_pushthread`es it, hence it
//! yields the running coroutine. (mlua's own doc: "for parameters given to a
//! callback, this will be whatever Lua thread called the callback".)
//!
//! The upstream PoC's "coroutine body frame unreachable from the main state"
//! note (frame_inspector.rs R3.4) was an artifact of `exec_raw`, which runs a
//! *nested* `lua_pcall`/`do_call` on the MAIN state — NOT of `current_thread()`
//! in a hook. Direct FFI on `current_thread().state()` does not have that
//! problem, so R2.4 is satisfied by the design's intended path with no
//! fallback (no `coroutine.create` override, no per-thread hook) needed. The
//! `tests` module proves this with a Lua-side `coroutine.create`/`resume`
//! scene driver (incl. post-yield locals) plus a teeth-check confirming the
//! main thread genuinely cannot see those body locals.
//!
//! ## ThreadId stability (StepController underpinning, design R-1)
//!
//! A given coroutine's `lua_State` pointer — i.e. its
//! [`ThreadId`](super::types::ThreadId) — is **stable across `yield`/`resume`**
//! (the coroutine reuses the same state each resume) and **distinct** between
//! coroutines. This is what lets the step machine key on `(thread, base_depth)`
//! and survive the yield boundary; it is asserted by the `tests` module.
//!
//! ## Why direct FFI on `thread.state()` (and NOT the PoC's `exec_raw`)
//!
//! The PoC ran its FFI through [`mlua::Lua::exec_raw`], whose closure is invoked
//! across an internal `lua_pcall` (`do_call` C function). That interposes C
//! frames, so PoC level 0 is a C frame and it had to scan past them with
//! `find_first_lua_frame`. Operating **directly** on `thread.state()` from
//! inside the line hook has no such interposition: the hook fires synchronously
//! on the running state, so **`lua_getstack` level 0 is the stopped Lua frame**
//! (the line being executed). This unifies tasks 2.3 (main frame) and 2.4
//! (coroutine body frame) under one implementation: 2.4 only needs to pass a
//! coroutine `Thread` whose `state()` is its own `lua_State`.
//!
//! We still *walk* levels defensively (skipping any non-Lua frame) so the code
//! is robust if a C frame ever sits at the requested level, but the common case
//! is a clean level-0 Lua frame.
//!
//! # mlua 0.11 / mlua-sys 0.10 (luajit52 = lua51 ABI) FFI used
//! - [`lua_getstack`](mlua::ffi::lua_getstack)`(L, level, &ar) -> c_int`
//!   (0 = no such level).
//! - [`lua_getinfo`](mlua::ffi::lua_getinfo)`(L, what, &ar) -> c_int`. `"Snl"`
//!   fills `source`/`short_src`/`what`/`currentline`/`name`. `"f"` pushes the
//!   running function onto the stack.
//! - [`lua_getlocal`](mlua::ffi::lua_getlocal)`(L, &ar, n) -> *const c_char`
//!   (non-null => value pushed + name returned; null => end).
//! - [`lua_getupvalue`](mlua::ffi::lua_getupvalue)`(L, funcindex, n)` (same
//!   push/return shape).
//! - [`lua_type`](mlua::ffi::lua_type) + `LUA_TNUMBER/TSTRING/TBOOLEAN/TTABLE`.
//! - [`lua_tonumber`](mlua::ffi::lua_tonumber) / [`lua_toboolean`](mlua::ffi::lua_toboolean)
//!   / [`lua_tolstring`](mlua::ffi::lua_tolstring) / [`lua_topointer`](mlua::ffi::lua_topointer)
//!   / [`lua_typename`](mlua::ffi::lua_typename).
//! - [`lua_gettop`](mlua::ffi::lua_gettop) / [`lua_pop`](mlua::ffi::lua_pop)
//!   / [`lua_settop`](mlua::ffi::lua_settop).
//! - `lua_Debug` has a private field (`i_ci`) so it is constructed with
//!   [`std::mem::zeroed`].
//!
//! # Stack discipline is MANDATORY (design "Error Handling": VM 破壊回避)
//!
//! Every `unsafe` traversal records [`lua_gettop`](mlua::ffi::lua_gettop) at
//! entry, pops each value immediately after reading it, restores via
//! [`lua_settop`](mlua::ffi::lua_settop) at exit, and `debug_assert_eq!`s the
//! entry/exit depth. The capture fns return `Vec<...>` directly (NOT `Result`):
//! they are infallible and graceful (R2.5) — on any FFI shortfall they return
//! partial/empty results and never panic, error, or corrupt the VM stack.

use std::ffi::{CStr, CString};
use std::os::raw::c_int;

use mlua::Lua;

use super::types::{FrameInfo, Variable};

/// Upper bound on call-stack levels walked, guarding against pathological or
/// corrupt activation records (the PoC used the same guard).
const MAX_STACK_LEVELS: c_int = 256;

/// Capture the call stack of `thread` as a `FrameInfo` per Lua frame (R2.1).
///
/// Walks `thread.state()` with `lua_getstack` from level 0 upward, calling
/// `lua_getinfo("Snl")` to read each frame's source / current line / function
/// name. Non-Lua frames (C frames, e.g. an interposed `pcall`) are skipped so
/// the returned vector contains only generated-`.lua` execution positions.
///
/// Infallible by contract (R2.5): never errors; returns whatever frames are
/// reachable. `lua_getinfo("Snl")` does not push a value, so this fn does not
/// disturb the VM stack.
///
/// `lua` is accepted to match the [`capture_variables`] signature and the
/// design's `FrameInspector` contract (and to pin the lifetime to a live VM);
/// the traversal itself runs on `thread.state()`.
pub(crate) fn capture_stack(lua: &Lua, thread: &mlua::Thread) -> Vec<FrameInfo> {
    let _ = lua; // signature/contract parity with capture_variables (design).
    let l = thread.state();
    if l.is_null() {
        return Vec::new();
    }

    let mut out: Vec<FrameInfo> = Vec::new();
    // SAFETY: `l` is a live lua_State (owned by `thread`, kept alive by `lua`).
    // `lua_getinfo("Snl")` reads into our owned `ar` and pushes nothing, so the
    // VM stack depth is unchanged across the whole walk.
    unsafe {
        let what = match CString::new("Snl") {
            Ok(c) => c,
            Err(_) => return out,
        };
        let mut level: c_int = 0;
        while level < MAX_STACK_LEVELS {
            let mut ar: mlua::ffi::lua_Debug = std::mem::zeroed();
            if mlua::ffi::lua_getstack(l, level, &mut ar as *mut _) == 0 {
                break; // no further frames
            }
            if mlua::ffi::lua_getinfo(l, what.as_ptr(), &mut ar as *mut _) != 0
                && let Some(frame) = frame_info_from_ar(&ar)
            {
                out.push(frame);
            }
            level += 1;
        }
    }
    out
}

/// Build a [`FrameInfo`] from a filled `lua_Debug`, or `None` for a non-Lua
/// (C) frame.
///
/// # Safety
/// `ar` must have been filled by `lua_getinfo("Snl")` (so `source`/`short_src`/
/// `what`/`name` C-string pointers are valid for the duration of this read).
unsafe fn frame_info_from_ar(ar: &mlua::ffi::lua_Debug) -> Option<FrameInfo> {
    unsafe {
        // Skip C frames: only generated-.lua execution positions are reported.
        if !ar.what.is_null() {
            let what = CStr::from_ptr(ar.what).to_string_lossy();
            if what == "C" {
                return None;
            }
        }

        // Prefer the chunk `source` (e.g. "@scene.lua", a `*const c_char` that
        // may be null); fall back to `short_src` (a fixed `[c_char; LUA_IDSIZE]`
        // array that `lua_getinfo("S")` always populates, NUL-terminated).
        let source = if !ar.source.is_null() {
            CStr::from_ptr(ar.source).to_string_lossy().into_owned()
        } else {
            CStr::from_ptr(ar.short_src.as_ptr())
                .to_string_lossy()
                .into_owned()
        };

        let line = if ar.currentline >= 0 {
            ar.currentline as u32
        } else {
            0
        };

        let func_name = if ar.name.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ar.name).to_string_lossy().into_owned())
        };

        Some(FrameInfo {
            source,
            line,
            func_name,
        })
    }
}

/// Capture the visible locals AND upvalues of `thread`'s frame at
/// `frame_level`, as `Variable { name, type_name, repr }` (R2.2 / R2.3 / R2.5).
///
/// Classifies `number` / `string` / `boolean` / `table` via `lua_type` so the
/// user can discriminate them (R2.3). UNSUPPORTED kinds (function / userdata /
/// thread / nil / cdata / ...) are recorded gracefully as `<unsupported T>`
/// (R2.5) — NEVER crashing, erroring, or corrupting the VM stack.
///
/// `frame_level` is the logical call-stack level on `thread.state()` where the
/// frame is found (0 = the stopped/top frame). The session passes the level
/// requested by the DAP client; for a running coroutine it passes that
/// coroutine's `Thread`, whose own `state()` likewise has its body frame at
/// the requested level.
///
/// Infallible by contract (R2.5): on any FFI shortfall it returns partial /
/// empty results. Stack discipline (design "Error Handling"): `lua_gettop` at
/// entry, `lua_pop` per value, `lua_settop` restore at exit, `debug_assert_eq!`
/// of push/pop symmetry.
pub(crate) fn capture_variables(
    lua: &Lua,
    thread: &mlua::Thread,
    frame_level: u32,
) -> Vec<Variable> {
    let _ = lua; // signature/contract parity (design); traversal uses thread.state().
    let l = thread.state();
    if l.is_null() {
        return Vec::new();
    }

    let mut out: Vec<Variable> = Vec::new();
    // SAFETY: `l` is a live lua_State. Every value pushed by lua_getlocal /
    // lua_getupvalue / lua_getinfo("f") is popped before returning; the entry
    // depth is restored with lua_settop and asserted equal (no stack corruption).
    unsafe {
        let top_at_entry = mlua::ffi::lua_gettop(l);

        let mut ar: mlua::ffi::lua_Debug = std::mem::zeroed();
        // Resolve the activation record for the requested level.
        if mlua::ffi::lua_getstack(l, frame_level as c_int, &mut ar as *mut _) != 0 {
            collect_locals(l, &ar, &mut out);
            collect_upvalues(l, &mut ar, &mut out);
        }

        // Restore the stack to its entry depth no matter what (VM 破壊回避).
        // Restore BEFORE asserting: if the symmetry invariant were ever
        // violated, the debug-build panic must not unwind past a still-dirty
        // VM stack (the hook machinery keeps the VM running after a captured
        // panic, so the stack has to be balanced first).
        let top_at_exit = mlua::ffi::lua_gettop(l);
        mlua::ffi::lua_settop(l, top_at_entry);
        debug_assert_eq!(
            top_at_entry, top_at_exit,
            "capture_variables must keep the VM stack balanced (push/pop symmetric)"
        );
    }
    out
}

/// Collect the locals of the frame described by `ar` into `out`.
///
/// `lua_getlocal(L, &ar, n)` for n = 1.. pushes each local's value and returns
/// its name; null terminates. Each pushed value is popped immediately after
/// reading (stack-neutral per local).
///
/// # Safety
/// `l` is a live `lua_State`; `ar` was filled by `lua_getstack` for the target
/// level on `l`.
unsafe fn collect_locals(
    l: *mut mlua::ffi::lua_State,
    ar: &mlua::ffi::lua_Debug,
    out: &mut Vec<Variable>,
) {
    unsafe {
        let mut n: c_int = 1;
        while n < c_int::MAX {
            let name_ptr = mlua::ffi::lua_getlocal(l, ar as *const _, n);
            if name_ptr.is_null() {
                break; // no more locals; nothing was pushed
            }
            let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
            let (type_name, repr) = read_value_at_top(l);
            mlua::ffi::lua_pop(l, 1); // pop the value lua_getlocal pushed
            out.push(Variable {
                name,
                type_name,
                repr,
            });
            n += 1;
        }
    }
}

/// Collect the upvalues of the function running in the frame described by `ar`
/// into `out`.
///
/// `lua_getinfo(L, "f", &ar)` pushes the running function; `lua_getupvalue(L,
/// funcindex, n)` for n = 1.. pushes each upvalue value and returns its name.
/// Each upvalue value is popped after reading, and the function itself is popped
/// at the end (stack-neutral overall).
///
/// # Safety
/// `l` is a live `lua_State`; `ar` was filled by `lua_getstack` for the target
/// level on `l`. `ar` is taken `&mut` because `lua_getinfo("f")` writes through
/// it while pushing the function.
unsafe fn collect_upvalues(
    l: *mut mlua::ffi::lua_State,
    ar: &mut mlua::ffi::lua_Debug,
    out: &mut Vec<Variable>,
) {
    unsafe {
        let what_f = match CString::new("f") {
            Ok(c) => c,
            Err(_) => return,
        };
        // Push the frame's running function (idx -1 afterwards).
        if mlua::ffi::lua_getinfo(l, what_f.as_ptr(), ar as *mut _) == 0 {
            return; // could not resolve the function; nothing pushed
        }
        let func_index = mlua::ffi::lua_gettop(l); // absolute idx of the pushed function

        let mut n: c_int = 1;
        while n < c_int::MAX {
            let name_ptr = mlua::ffi::lua_getupvalue(l, func_index, n);
            if name_ptr.is_null() {
                break; // no more upvalues; nothing pushed this round
            }
            let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
            let (type_name, repr) = read_value_at_top(l);
            mlua::ffi::lua_pop(l, 1); // pop the upvalue value
            out.push(Variable {
                name,
                type_name,
                repr,
            });
            n += 1;
        }

        // Pop the function pushed by lua_getinfo("f").
        mlua::ffi::lua_pop(l, 1);
    }
}

/// Read `(type_name, repr)` of the value at stack top (idx -1) WITHOUT changing
/// the stack depth (the caller pops).
///
/// number / string / boolean / table are discriminated by `lua_type` (R2.3);
/// every other kind (function / userdata / thread / nil / cdata / ...) is
/// recorded gracefully via `lua_typename` with an `<unsupported T>` repr (R2.5)
/// — never crashing.
///
/// # Safety
/// `l` is a live `lua_State` with at least one value at idx -1. `lua_tolstring`
/// is only called on `string` (already a string, so no coercion mutates the
/// value). This fn is read-only and stack-neutral.
unsafe fn read_value_at_top(l: *mut mlua::ffi::lua_State) -> (String, String) {
    unsafe {
        let t = mlua::ffi::lua_type(l, -1);
        match t {
            mlua::ffi::LUA_TNUMBER => {
                let n = mlua::ffi::lua_tonumber(l, -1);
                // Integer-valued numbers print without a decimal point.
                let repr = if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", n as i64)
                } else {
                    format!("{n}")
                };
                ("number".to_string(), repr)
            }
            mlua::ffi::LUA_TSTRING => {
                let mut len: usize = 0;
                let ptr = mlua::ffi::lua_tolstring(l, -1, &mut len as *mut usize);
                let repr = if ptr.is_null() {
                    String::new()
                } else {
                    let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
                    String::from_utf8_lossy(bytes).into_owned()
                };
                ("string".to_string(), repr)
            }
            mlua::ffi::LUA_TBOOLEAN => {
                let b = mlua::ffi::lua_toboolean(l, -1) != 0;
                ("boolean".to_string(), b.to_string())
            }
            mlua::ffi::LUA_TTABLE => {
                // Table contents expansion is out of scope here; use the address
                // as a readable placeholder repr (table:-prefixed).
                let addr = mlua::ffi::lua_topointer(l, -1);
                ("table".to_string(), format!("table: {addr:p}"))
            }
            other => {
                // Unsupported kind: record its type name, never crash (R2.5).
                let name_ptr = mlua::ffi::lua_typename(l, other);
                let type_name = if name_ptr.is_null() {
                    format!("type#{other}")
                } else {
                    CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
                };
                let repr = format!("<unsupported {type_name}>");
                (type_name, repr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ThreadId;
    use mlua::{HookTriggers, Lua, LuaOptions, StdLib, VmState};
    use std::sync::{Arc, Mutex};

    // -----------------------------------------------------------------------
    // Test-only VM construction (kept inside the debug module — does NOT depend
    // on the tests/ PoC harness). ALL_SAFE so `jit` exists and `debug` is
    // excluded; we apply jit.off() so line hooks never miss JIT-compiled lines.
    // -----------------------------------------------------------------------
    fn build_jit_off_vm() -> Lua {
        let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) };
        lua.load("jit.off()").exec().expect("jit.off() must run");
        lua
    }

    /// Read the chunk name + current line from a hook `Debug` (safe API), to
    /// locate the stop line for the test (mirrors hook.rs's pattern).
    fn source_and_line(debug: &mlua::Debug) -> (String, u32) {
        let src = debug.source();
        let source = src
            .source
            .as_ref()
            .map(|c| c.as_ref().to_string())
            .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
            .unwrap_or_default();
        let line = debug.current_line().unwrap_or(0) as u32;
        (source, line)
    }

    fn find_var<'a>(vars: &'a [Variable], name: &str) -> Option<&'a Variable> {
        vars.iter().find(|v| v.name == name)
    }

    /// R2.2 / R2.3: stop at a known top-level chunk line via a line hook and
    /// prove `capture_variables` returns locals of each basic type BY NAME with
    /// the correct `type_name` and a readable `repr`.
    #[test]
    fn capture_variables_basic_types_by_name() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        let target_line: u32 = 6;

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@locals_chunk" && line == target_line {
                // Direct FFI on the running thread's state (level 0 = stopped frame).
                let thread = hook_lua.current_thread();
                let vars = capture_variables(hook_lua, &thread, 0);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_empty()
                {
                    *g = vars;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        // 4 locals visible by the marker line (line 6).
        let chunk = "\
local num = 42
local str = 'hello'
local flag = true
local tbl = { 1, 2, 3 }
local marker = num
return marker
";
        lua.load(chunk)
            .set_name("@locals_chunk")
            .exec()
            .expect("chunk should execute");
        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "capture_variables must capture locals at the breakpoint (R2.2). got empty"
        );

        let num = find_var(&vars, "num")
            .unwrap_or_else(|| panic!("local 'num' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(num.type_name, "number", "num must be discriminated as number (R2.3)");
        assert_eq!(num.repr, "42", "num value must be readable as 42");

        let s = find_var(&vars, "str")
            .unwrap_or_else(|| panic!("local 'str' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(s.type_name, "string", "str must be discriminated as string (R2.3)");
        assert_eq!(s.repr, "hello", "str value must be readable as 'hello'");

        let flag = find_var(&vars, "flag")
            .unwrap_or_else(|| panic!("local 'flag' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(flag.type_name, "boolean", "flag must be discriminated as boolean (R2.3)");
        assert_eq!(flag.repr, "true", "flag value must be readable as true");

        let tbl = find_var(&vars, "tbl")
            .unwrap_or_else(|| panic!("local 'tbl' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(tbl.type_name, "table", "tbl must be discriminated as table (R2.3)");
        assert!(
            tbl.repr.starts_with("table:"),
            "table repr must be a readable placeholder. got: {}",
            tbl.repr
        );

        // R5.3: std_debug stayed unexposed during FFI inspection.
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(
            debug_is_nil,
            "std_debug must remain unexposed during FFI inspection (sandbox preserved, R5.3)"
        );
    }

    /// R2.2 (upvalue path): a closure capturing an upvalue is stopped, and
    /// `capture_variables` retrieves the captured upvalue BY NAME.
    #[test]
    fn capture_variables_includes_upvalue_by_name() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, _line) = source_and_line(debug);
            if source == "@upvalue_closure"
                && let Ok(mut g) = captured_hook.lock()
                && g.is_empty()
            {
                let thread = hook_lua.current_thread();
                let vars = capture_variables(hook_lua, &thread, 0);
                // Keep only the first frame that actually exposes the upvalue.
                if vars.iter().any(|v| v.name == "captured_num") {
                    *g = vars;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let closure: mlua::Function = lua
            .load(
                "\
local captured_num = 7
return function()
    local doubled = captured_num * 2
    return doubled
end
",
            )
            .set_name("@upvalue_closure")
            .eval::<mlua::Function>()
            .expect("named closure factory should produce a function");

        closure.call::<i64>(()).expect("closure call should succeed");
        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "capture_variables must capture the captured upvalue (R2.2). got empty"
        );
        let up = find_var(&vars, "captured_num").unwrap_or_else(|| {
            panic!(
                "upvalue 'captured_num' must be retrieved by name. got: {:?}",
                *vars
            )
        });
        assert_eq!(
            up.type_name, "number",
            "captured upvalue must be discriminated as number (R2.3)"
        );
        assert_eq!(up.repr, "7", "captured upvalue value must be readable as 7");
    }

    /// R2.5: a frame mixing a basic type (number) with UNSUPPORTED kinds
    /// (function + nil) must: (1) not crash, (2) still retrieve the number,
    /// (3) record the unsupported kinds gracefully, and (4) leave the VM usable
    /// (`return 1+2` == 3 afterwards — proving no stack corruption).
    #[test]
    fn capture_variables_unsupported_kinds_graceful_and_vm_usable() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        let target_line: u32 = 5;

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@unsupported_chunk" && line == target_line {
                let thread = hook_lua.current_thread();
                let vars = capture_variables(hook_lua, &thread, 0);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_empty()
                {
                    *g = vars;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let chunk = "\
local basic = 42
local fnval = function() return 1 end
local nilval = nil
local marker = basic
return marker
";
        lua.load(chunk)
            .set_name("@unsupported_chunk")
            .exec()
            .expect("chunk should execute despite unsupported-typed locals (R2.5)");
        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "capture_variables must capture locals at the breakpoint (R2.5). got empty"
        );

        // (1)+(2) basic number still retrieved by name.
        let basic = find_var(&vars, "basic").unwrap_or_else(|| {
            panic!(
                "basic-typed local 'basic' must still be obtained alongside unsupported kinds (R2.5). got: {:?}",
                *vars
            )
        });
        assert_eq!(basic.type_name, "number");
        assert_eq!(basic.repr, "42");

        // (3) function recorded gracefully.
        let fnval = find_var(&vars, "fnval").unwrap_or_else(|| {
            panic!(
                "function-typed local 'fnval' must be RECORDED (out-of-scope), not dropped (R2.5). got: {:?}",
                *vars
            )
        });
        assert_eq!(fnval.type_name, "function");
        assert!(
            fnval.repr.starts_with("<unsupported"),
            "an unsupported kind must carry an out-of-scope repr placeholder (R2.5): {:?}",
            fnval.repr
        );

        // nil recorded gracefully.
        let nilval = find_var(&vars, "nilval").unwrap_or_else(|| {
            panic!(
                "nil-typed local 'nilval' must be RECORDED as out-of-scope (R2.5). got: {:?}",
                *vars
            )
        });
        assert_eq!(nilval.type_name, "nil");
        assert!(
            nilval.repr.starts_with("<unsupported"),
            "nil must carry an out-of-scope repr placeholder (R2.5): {:?}",
            nilval.repr
        );

        // (4) VM remains usable: stack was not corrupted.
        let sane: i64 = lua
            .load("return 1 + 2")
            .eval()
            .expect("VM must remain usable after inspecting unsupported kinds (R2.5)");
        assert_eq!(sane, 3, "VM stack must remain balanced after R2.5 inspection");
    }

    /// R2.1: `capture_stack` returns at least the stopped frame with the correct
    /// source + line.
    #[test]
    fn capture_stack_reports_stopped_frame_source_and_line() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<FrameInfo>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        let target_line: u32 = 2;

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@stack_chunk" && line == target_line {
                let thread = hook_lua.current_thread();
                let frames = capture_stack(hook_lua, &thread);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_empty()
                {
                    *g = frames;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let chunk = "\
local a = 1
local b = a + 1
return b
";
        lua.load(chunk)
            .set_name("@stack_chunk")
            .exec()
            .expect("chunk should execute");
        lua.remove_global_hook();

        let frames = captured.lock().unwrap();
        assert!(
            !frames.is_empty(),
            "capture_stack must return at least the stopped frame (R2.1). got empty"
        );
        // The stopped frame must be the first Lua frame reported.
        let stopped = &frames[0];
        assert_eq!(
            stopped.source, "@stack_chunk",
            "capture_stack must report the stopped frame source (R2.1). got: {:?}",
            *frames
        );
        assert_eq!(
            stopped.line, target_line,
            "capture_stack must report the stopped frame line (R2.1). got: {:?}",
            *frames
        );
    }

    /// R5.3: `debug` (std_debug) is nil after FFI inspection — the sandbox is
    /// preserved (we never enable the Lua debug library).
    #[test]
    fn inspection_keeps_std_debug_unexposed() {
        let lua = build_jit_off_vm();
        let captured = Arc::new(Mutex::new(false));
        let captured_hook = Arc::clone(&captured);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, _line) = source_and_line(debug);
            if source == "@sandbox_chunk" {
                let thread = hook_lua.current_thread();
                let _ = capture_stack(hook_lua, &thread);
                let _ = capture_variables(hook_lua, &thread, 0);
                if let Ok(mut g) = captured_hook.lock() {
                    *g = true;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        lua.load("local a = 1\nreturn a\n")
            .set_name("@sandbox_chunk")
            .exec()
            .expect("chunk should execute");
        lua.remove_global_hook();

        assert!(
            *captured.lock().unwrap(),
            "the hook must have run the inspection on @sandbox_chunk"
        );
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(
            debug_is_nil,
            "std_debug must remain nil after inspection (sandbox preserved, R5.3)"
        );
    }

    // =======================================================================
    // Task 2.4 — running Lua-side coroutine BODY-frame inspection (R2.4) +
    // ThreadId resume-crossing stability (StepController underpinning).
    //
    // These exercise the design's R2.4 seam: from inside a GLOBAL line hook
    // that fires while a Lua-side `coroutine.create`/`coroutine.resume` body is
    // running, `hook_lua.current_thread()` must resolve to the RUNNING
    // coroutine (not the main thread), so the SAME `capture_variables`/
    // `capture_stack` from 2.3 — operating on `thread.state()` — reaches the
    // coroutine body frame's locals. See the module-level "Thread-state design"
    // note: 2.4 is a CALLER change (pass the running coroutine `&Thread`), not a
    // rewrite of the traversal.
    //
    // Why this works (empirically confirmed against mlua 0.11.6 source):
    // mlua's C `global_hook_proc` receives the *running* `lua_State*` and wraps
    // the user callback in `callback_error_ext`, which installs a `StateGuard`
    // that swaps `RawLua.state` to that running state for the callback's
    // duration. `Lua::current_thread()` reads `RawLua.state` and
    // `lua_pushthread`es it, so inside the hook it yields the running
    // coroutine. (The upstream PoC's "coroutine frame unreachable from main
    // state" note was an artifact of `exec_raw`, which runs a NESTED
    // `lua_pcall`/`do_call` on the MAIN state — not of `current_thread()` in a
    // hook.)
    // =======================================================================

    /// Drive a pasta-style coroutine body to a known stop line and capture, via
    /// `current_thread()`, both the body-frame variables (`capture_variables`)
    /// and the `ThreadId` (`current_thread().state()` address) observed at that
    /// stop. Returns `(vars, thread_id, stack)` from the FIRST time the hook
    /// fires on `target_source` at `target_line`.
    ///
    /// The coroutine is created with Lua-side `coroutine.create` and driven by a
    /// resume loop (exactly the pasta scene execution model), so this proves the
    /// real production path, not a Rust-created `lua.create_thread`.
    /// Everything captured at the stop line by [`run_coroutine_and_capture_at`]:
    /// body-frame variables, the running coroutine's `ThreadId`, and the stack.
    type CoroutineCapture = (Vec<Variable>, ThreadId, Vec<FrameInfo>);

    fn run_coroutine_and_capture_at(
        lua: &Lua,
        body: &str,
        body_name: &str,
        target_line: u32,
    ) -> (Vec<Variable>, Option<ThreadId>, Vec<FrameInfo>) {
        let captured: Arc<Mutex<Option<CoroutineCapture>>> = Arc::new(Mutex::new(None));
        let captured_hook = Arc::clone(&captured);
        let want_source = body_name.to_string();

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == want_source && line == target_line {
                // R2.4 seam: resolve the RUNNING coroutine from inside the hook.
                let thread = hook_lua.current_thread();
                let tid = ThreadId::from_state(thread.state());
                let vars = capture_variables(hook_lua, &thread, 0);
                let stack = capture_stack(hook_lua, &thread);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_none()
                {
                    *g = Some((vars, tid, stack));
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        // pasta scene model: load the body as a function, create a coroutine
        // from it on the Lua side, and resume-until-dead.
        let scene_fn: mlua::Function = lua
            .load(body)
            .set_name(body_name)
            .into_function()
            .expect("body should load into a function");

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
            .into_function()
            .expect("driver should load");

        driver
            .call::<()>(scene_fn)
            .expect("coroutine driver should run to completion");
        lua.remove_global_hook();

        match Arc::try_unwrap(captured)
            .expect("hook dropped after remove_global_hook")
            .into_inner()
            .unwrap()
        {
            Some((vars, tid, stack)) => (vars, Some(tid), stack),
            None => (Vec::new(), None, Vec::new()),
        }
    }

    /// R2.4: stopped at a line INSIDE a running Lua-side coroutine body, the
    /// body frame's locals are retrieved BY NAME with correct types from
    /// `current_thread().state()` — proving the coroutine body frame is
    /// reachable (the #1 feature risk, R-2).
    #[test]
    fn capture_variables_reaches_running_coroutine_body_locals() {
        let lua = build_jit_off_vm();
        // a=1 (num), b='x' (str), c=true (bool), t={..} (table) all visible by
        // the marker line (line 5), which sits BEFORE the yield.
        let body = "\
local a = 1
local b = 'x'
local c = true
local t = { 10, 20 }
local marker = a
coroutine.yield()
marker = marker + 1
return marker
";
        let (vars, tid, _stack) =
            run_coroutine_and_capture_at(&lua, body, "@co_body_locals", 5);

        assert!(
            tid.is_some(),
            "the hook must have fired inside the coroutine body at the marker line (R2.4)"
        );
        assert!(
            !vars.is_empty(),
            "capture_variables must reach the running coroutine body frame's locals (R2.4). \
             got empty — the body frame was UNREACHABLE (this is the R-2 failure mode)"
        );

        let a = find_var(&vars, "a").unwrap_or_else(|| {
            panic!("coroutine-body local 'a' must be retrieved by name (R2.4). got: {vars:?}")
        });
        assert_eq!(a.type_name, "number", "body local 'a' must be a number (R2.3/R2.4)");
        assert_eq!(a.repr, "1", "body local 'a' must read as 1");

        let b = find_var(&vars, "b").unwrap_or_else(|| {
            panic!("coroutine-body local 'b' must be retrieved by name (R2.4). got: {vars:?}")
        });
        assert_eq!(b.type_name, "string", "body local 'b' must be a string");
        assert_eq!(b.repr, "x", "body local 'b' must read as 'x'");

        let c = find_var(&vars, "c").unwrap_or_else(|| {
            panic!("coroutine-body local 'c' must be retrieved by name (R2.4). got: {vars:?}")
        });
        assert_eq!(c.type_name, "boolean", "body local 'c' must be a boolean");
        assert_eq!(c.repr, "true", "body local 'c' must read as true");

        let t = find_var(&vars, "t").unwrap_or_else(|| {
            panic!("coroutine-body local 't' must be retrieved by name (R2.4). got: {vars:?}")
        });
        assert_eq!(t.type_name, "table", "body local 't' must be a table");

        // VM remains usable after coroutine inspection (stack balanced).
        let sane: i64 = lua
            .load("return 1 + 2")
            .eval()
            .expect("VM must remain usable after coroutine body inspection (R2.5)");
        assert_eq!(sane, 3, "VM stack must stay balanced after coroutine inspection");

        // std_debug stays nil (R5.3).
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(debug_is_nil, "std_debug must remain nil during coroutine inspection (R5.3)");
    }

    /// R2.4 (post-yield): stopped on a line AFTER `coroutine.yield()` (i.e. on a
    /// subsequent `resume`), the coroutine body locals — including state mutated
    /// before/after the yield — are still reachable via `current_thread()`.
    /// This proves inspection survives the yield/resume boundary, not only the
    /// first resume.
    #[test]
    fn capture_variables_reaches_coroutine_body_after_yield() {
        let lua = build_jit_off_vm();
        // Marker line 7 (`return acc`) executes only on the SECOND resume, after
        // the yield on line 4. `acc` is mutated post-yield (line 6) so we prove
        // we read the live post-yield value, not a pre-yield snapshot.
        let body = "\
local seed = 5
local acc = seed * 2
coroutine.yield()
acc = acc + seed
local done = true
local sentinel = acc
return sentinel
";
        let (vars, tid, _stack) =
            run_coroutine_and_capture_at(&lua, body, "@co_post_yield", 7);

        assert!(
            tid.is_some(),
            "the hook must have fired post-yield inside the coroutine body (R2.4)"
        );
        assert!(
            !vars.is_empty(),
            "post-yield coroutine body locals must be reachable via current_thread() (R2.4). \
             got empty"
        );

        let acc = find_var(&vars, "acc").unwrap_or_else(|| {
            panic!("post-yield body local 'acc' must be retrieved by name (R2.4). got: {vars:?}")
        });
        assert_eq!(acc.type_name, "number");
        // seed=5; acc=10 then acc=acc+seed=15 after the yield resumes.
        assert_eq!(
            acc.repr, "15",
            "post-yield 'acc' must read its LIVE mutated value 15 (not a pre-yield snapshot)"
        );

        let done = find_var(&vars, "done").unwrap_or_else(|| {
            panic!("post-yield body local 'done' must be retrieved by name (R2.4). got: {vars:?}")
        });
        assert_eq!(done.type_name, "boolean");
        assert_eq!(done.repr, "true");
    }

    /// R2.4 (call stack): `capture_stack` on the running coroutine reports the
    /// coroutine body frame at top with the body's own source/line.
    #[test]
    fn capture_stack_reports_running_coroutine_body_frame() {
        let lua = build_jit_off_vm();
        let body = "\
local a = 1
local b = a + 1
coroutine.yield()
return b
";
        let (_vars, tid, stack) =
            run_coroutine_and_capture_at(&lua, body, "@co_stack", 2);

        assert!(tid.is_some(), "the hook must have fired in the coroutine body");
        assert!(
            !stack.is_empty(),
            "capture_stack must report at least the coroutine body frame (R2.1/R2.4). got empty"
        );
        let top = &stack[0];
        assert_eq!(
            top.source, "@co_stack",
            "top frame must be the coroutine body source (R2.4). got: {stack:?}"
        );
        assert_eq!(
            top.line, 2,
            "top frame must be the coroutine body stop line (R2.4). got: {stack:?}"
        );
    }

    /// StepController underpinning (design "DebugSession 状態機械" / R-1): the
    /// `ThreadId` derived from `current_thread().state()` of a SINGLE coroutine
    /// is STABLE across `yield`→`resume` (the same `lua_State` pointer is reused
    /// by the coroutine each time it is resumed). This is what lets the step
    /// machine key on `(thread, base_depth)` and survive the yield boundary.
    #[test]
    fn thread_id_is_stable_across_resume_of_same_coroutine() {
        let lua = build_jit_off_vm();
        // The body yields twice; the hook fires once per resume on the two
        // distinct marker lines (1 and 4). Record the ThreadId at each.
        let seen: Arc<Mutex<Vec<ThreadId>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_hook = Arc::clone(&seen);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@co_resume_stable" && (line == 1 || line == 4) {
                let tid = ThreadId::from_state(hook_lua.current_thread().state());
                if let Ok(mut g) = seen_hook.lock() {
                    g.push(tid);
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let scene_fn: mlua::Function = lua
            .load(
                "\
local first = 1
coroutine.yield()
local second = 2
local third = second + 1
",
            )
            .set_name("@co_resume_stable")
            .into_function()
            .expect("body loads");

        // Drive the SAME coroutine across multiple resumes.
        lua.load(
            "\
local scene_fn = ...
local co = coroutine.create(scene_fn)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
",
        )
        .set_name("@scene_driver")
        .into_function()
        .expect("driver loads")
        .call::<()>(scene_fn)
        .expect("driver runs");
        lua.remove_global_hook();

        let seen = seen.lock().unwrap();
        assert!(
            seen.len() >= 2,
            "the hook must fire on both pre-yield (line 1) and post-yield (line 4) markers — \
             proving we observed the coroutine across a resume. got: {seen:?}"
        );
        let first = seen[0];
        assert!(
            seen.iter().all(|&t| t == first),
            "the coroutine's ThreadId (current_thread().state() addr) must be STABLE across \
             yield/resume (StepController keys on it). got: {seen:?}"
        );
        // Sanity: a coroutine's state pointer is non-null and distinct from main.
        let main_tid = ThreadId::from_state(lua.current_thread().state());
        assert_ne!(
            first, main_tid,
            "a running coroutine's ThreadId must differ from the main thread's (R2.4 reached the \
             coroutine, not main). coroutine={first:?} main={main_tid:?}"
        );
    }

    // =======================================================================
    // Cell 3.25 (G1) additions — previously-unasserted public behaviour:
    // multi-frame stack walk + func_name resolution, caller-frame variable
    // capture (frame_level > 0), out-of-range frame_level, and the
    // non-integer / negative number repr branches of read_value_at_top.
    // =======================================================================

    /// R2.1: stopped INSIDE a named local function, `capture_stack` must walk
    /// MULTIPLE Lua frames in callee→caller order, resolve the callee's
    /// `func_name` ("inner"), and report the top-level chunk frame (whose
    /// `func_name` is unresolvable → None) at the call line.
    #[test]
    fn capture_stack_walks_nested_frames_and_resolves_func_name() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<FrameInfo>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        // Line 4 = `return inner_var` (inside inner); line 6 = the call site.
        let target_line: u32 = 4;

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@nested_stack_chunk" && line == target_line {
                let thread = hook_lua.current_thread();
                let frames = capture_stack(hook_lua, &thread);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_empty()
                {
                    *g = frames;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let chunk = "\
local outer_var = 99
local function inner()
    local inner_var = 7
    return inner_var
end
local result = inner()
return result
";
        lua.load(chunk)
            .set_name("@nested_stack_chunk")
            .exec()
            .expect("chunk should execute");
        lua.remove_global_hook();

        let frames = captured.lock().unwrap();
        assert!(
            frames.len() >= 2,
            "stopped inside inner(), capture_stack must walk BOTH Lua frames \
             (callee + caller chunk) (R2.1). got: {:?}",
            *frames
        );

        // Frame 0 (callee): inner's stop line, func_name resolved as "inner".
        let callee = &frames[0];
        assert_eq!(callee.source, "@nested_stack_chunk");
        assert_eq!(
            callee.line, target_line,
            "top frame must be the stopped line inside inner. got: {:?}",
            *frames
        );
        assert_eq!(
            callee.func_name.as_deref(),
            Some("inner"),
            "the callee frame's func_name must be resolved as 'inner' (R2.1). got: {:?}",
            *frames
        );

        // Frame 1 (caller = top-level chunk): the call-site line; a main chunk
        // has no resolvable function name.
        let caller = &frames[1];
        assert_eq!(caller.source, "@nested_stack_chunk");
        assert_eq!(
            caller.line, 6,
            "the caller frame must sit on the call-site line. got: {:?}",
            *frames
        );
        assert_eq!(
            caller.func_name, None,
            "a top-level chunk frame has no resolvable func_name. got: {:?}",
            *frames
        );
    }

    /// R2.2 (frame_level seam): `frame_level = 1` must capture the CALLER
    /// frame's locals (not the stopped callee's): the caller's `outer_var` is
    /// present and the callee's `inner_var` is NOT.
    #[test]
    fn capture_variables_at_caller_frame_level() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        let target_line: u32 = 4; // `return inner_var` inside inner

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@caller_frame_chunk" && line == target_line {
                let thread = hook_lua.current_thread();
                // Level 1 = the CALLER of the stopped frame.
                let vars = capture_variables(hook_lua, &thread, 1);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_empty()
                {
                    *g = vars;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let chunk = "\
local outer_var = 99
local function inner()
    local inner_var = 7
    return inner_var
end
local result = inner()
return result
";
        lua.load(chunk)
            .set_name("@caller_frame_chunk")
            .exec()
            .expect("chunk should execute");
        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "frame_level=1 must capture the caller frame's variables (R2.2). got empty"
        );

        let outer = find_var(&vars, "outer_var").unwrap_or_else(|| {
            panic!(
                "caller local 'outer_var' must be visible at frame_level=1. got: {:?}",
                *vars
            )
        });
        assert_eq!(outer.type_name, "number");
        assert_eq!(outer.repr, "99", "caller local must carry the caller's value");

        assert!(
            find_var(&vars, "inner_var").is_none(),
            "the CALLEE's local 'inner_var' must NOT appear at frame_level=1 \
             (proves the caller frame, not the stopped frame, was read). got: {:?}",
            *vars
        );
    }

    /// R2.5 (graceful): a `frame_level` beyond the call stack has no activation
    /// record (`lua_getstack` returns 0) — the capture must return EMPTY, not
    /// crash, and the VM must remain usable (stack balanced).
    #[test]
    fn capture_variables_out_of_range_level_returns_empty() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Option<Vec<Variable>>>> = Arc::new(Mutex::new(None));
        let captured_hook = Arc::clone(&captured);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, _line) = source_and_line(debug);
            if source == "@oor_level_chunk" {
                let thread = hook_lua.current_thread();
                let vars = capture_variables(hook_lua, &thread, 200);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_none()
                {
                    *g = Some(vars);
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        lua.load("local a = 1\nreturn a\n")
            .set_name("@oor_level_chunk")
            .exec()
            .expect("chunk should execute despite an out-of-range capture level (R2.5)");
        lua.remove_global_hook();

        let vars = captured
            .lock()
            .unwrap()
            .clone()
            .expect("the hook must have attempted the out-of-range capture");
        assert!(
            vars.is_empty(),
            "an out-of-range frame_level must yield an EMPTY capture (graceful, R2.5). \
             got: {vars:?}"
        );

        // VM remains usable: the failed lookup must not unbalance the stack.
        let sane: i64 = lua
            .load("return 1 + 2")
            .eval()
            .expect("VM must remain usable after an out-of-range capture (R2.5)");
        assert_eq!(sane, 3);
    }

    /// R2.3 (number repr branches): non-integer numbers keep their fraction
    /// ("2.5"), negative integers print without a decimal point ("-3"), and
    /// non-finite numbers take the non-integer formatting path ("inf") —
    /// pinning both arms of read_value_at_top's integer/fraction split.
    #[test]
    fn capture_variables_number_repr_fractional_negative_and_nonfinite() {
        let lua = build_jit_off_vm();
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        let target_line: u32 = 4; // all three locals visible by `marker`

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if source == "@number_repr_chunk" && line == target_line {
                let thread = hook_lua.current_thread();
                let vars = capture_variables(hook_lua, &thread, 0);
                if let Ok(mut g) = captured_hook.lock()
                    && g.is_empty()
                {
                    *g = vars;
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let chunk = "\
local frac = 2.5
local neg = -3
local huge = math.huge
local marker = frac
return marker
";
        lua.load(chunk)
            .set_name("@number_repr_chunk")
            .exec()
            .expect("chunk should execute");
        lua.remove_global_hook();

        let vars = captured.lock().unwrap();

        let frac = find_var(&vars, "frac").unwrap_or_else(|| {
            panic!("local 'frac' must be retrieved by name. got: {:?}", *vars)
        });
        assert_eq!(frac.type_name, "number");
        assert_eq!(
            frac.repr, "2.5",
            "a fractional number must keep its fraction in repr (R2.3)"
        );

        let neg = find_var(&vars, "neg")
            .unwrap_or_else(|| panic!("local 'neg' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(neg.type_name, "number");
        assert_eq!(
            neg.repr, "-3",
            "a negative integer-valued number must print without a decimal point"
        );

        let huge = find_var(&vars, "huge")
            .unwrap_or_else(|| panic!("local 'huge' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(huge.type_name, "number");
        assert_eq!(
            huge.repr, "inf",
            "math.huge is non-finite and must take the non-integer formatting path"
        );
    }

    /// StepController underpinning: DISTINCT coroutines have DISTINCT
    /// `ThreadId`s, so the step machine can tell scene coroutines apart and
    /// skip lines belonging to a different thread.
    #[test]
    fn distinct_coroutines_have_distinct_thread_ids() {
        let lua = build_jit_off_vm();
        let seen: Arc<Mutex<Vec<(String, ThreadId)>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_hook = Arc::clone(&seen);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let (source, line) = source_and_line(debug);
            if (source == "@co_distinct_0" || source == "@co_distinct_1") && line == 1 {
                let tid = ThreadId::from_state(hook_lua.current_thread().state());
                if let Ok(mut g) = seen_hook.lock()
                    && !g.iter().any(|(s, _)| s == &source)
                {
                    g.push((source, tid));
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        for i in 0..2usize {
            let name = format!("@co_distinct_{i}");
            let body = format!("local marker = {i}\nreturn marker\n");
            let scene_fn: mlua::Function = lua
                .load(&body)
                .set_name(&name)
                .into_function()
                .expect("body loads");
            lua.load(
                "\
local scene_fn = ...
local co = coroutine.create(scene_fn)
while coroutine.status(co) ~= 'dead' do
    coroutine.resume(co)
end
",
            )
            .set_name("@scene_driver")
            .into_function()
            .expect("driver loads")
            .call::<()>(scene_fn)
            .expect("driver runs");
        }
        lua.remove_global_hook();

        let seen = seen.lock().unwrap();
        assert_eq!(
            seen.len(),
            2,
            "both coroutines' bodies must have been observed (R2.4). got: {seen:?}"
        );
        assert_ne!(
            seen[0].1, seen[1].1,
            "distinct coroutines must have distinct ThreadIds so the StepController can tell \
             them apart. got: {seen:?}"
        );
    }
}
