//! `inspect` モジュールの **call stack 取得とコルーチン body-frame 検査**インライン
//! テスト外出し（task 2.4・C1）。`capture_stack` の停止/多段フレーム walk と func_name
//! 解決、running Lua-side coroutine の body-frame 到達（R2.4）、yield/resume 跨ぎの locals、
//! `ThreadId` の resume 跨ぎ安定性と coroutine 間の区別を集約する。
//!
//! 移動のみ（振る舞い不変）。元の単一 `mod tests`（~1023行）を凝集境界で 2 兄弟へ分割した
//! 一方。共有ヘルパー（`build_jit_off_vm`/`source_and_line`/`find_var`）は
//! `inspect_test_support.rs` を `use` する。`ThreadId` は本クラスタ専用に局所 `use` する。

use super::inspect_test_support::*;
use super::super::types::ThreadId;
use super::*;

use mlua::{HookTriggers, Lua, VmState};
use std::sync::{Arc, Mutex};

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
