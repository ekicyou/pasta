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

// ===========================================================================
// インラインテストの外出し（task 2.4・C1）。元の単一 `mod tests`（~1023行）を
// 凝集境界で 2 つの FLAT 兄弟テストファイルへ分割する（各 < 600 行）:
//   - 変数キャプチャ（`capture_variables`）→ inspect_variables_tests.rs
//   - call stack + コルーチン body-frame 検査 → inspect_stack_coroutine_tests.rs
// クラスタ跨ぎで共有するテストヘルパー（`build_jit_off_vm`/`source_and_line`/`find_var`）は
// `inspect_test_support.rs` に `pub(super)` で集約し、各クラスタが
// `use super::inspect_test_support::*;` で参照する（本番可視性は不変）。
// ===========================================================================

/// クラスタ跨ぎ共有テストヘルパー（`pub(super)`・test-only）。
#[cfg(test)]
#[path = "inspect_test_support.rs"]
mod inspect_test_support;

/// 変数キャプチャクラスタ（`capture_variables` の locals/upvalues・型判別・graceful）。
#[cfg(test)]
#[path = "inspect_variables_tests.rs"]
mod inspect_variables_tests;

/// call stack + コルーチン body-frame クラスタ（`capture_stack`・R2.4・`ThreadId`）。
#[cfg(test)]
#[path = "inspect_stack_coroutine_tests.rs"]
mod inspect_stack_coroutine_tests;
