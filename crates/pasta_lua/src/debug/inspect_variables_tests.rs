//! `inspect` モジュールの **変数キャプチャ**（`capture_variables`）インラインテスト
//! 外出し（task 2.4・C1）。停止フレームの locals/upvalues を名前・型・repr で取得、基本型
//! 判別、非対応種の graceful 記録（VM 破壊なし）、caller frame_level、範囲外 level、
//! number repr 分岐、サンドボックス保持を集約する。
//!
//! 移動のみ（振る舞い不変）。元の単一 `mod tests`（~1023行）を凝集境界で 2 兄弟へ分割した
//! 一方。共有ヘルパー（`build_jit_off_vm`/`source_and_line`/`find_var`）は
//! `inspect_test_support.rs` を `use` する（本番可視性は不変）。

use super::inspect_test_support::*;
use super::*;

use mlua::{HookTriggers, VmState};
use std::sync::{Arc, Mutex};

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
