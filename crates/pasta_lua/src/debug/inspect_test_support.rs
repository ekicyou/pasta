//! `inspect` インラインテスト分割（task 2.4・C1）で **クラスタ跨ぎ**に共有する
//! テストヘルパー。`inspect_variables_tests.rs`（main-thread フレーム検査）と
//! `inspect_stack_coroutine_tests.rs`（running coroutine body-frame 検査）が
//! `use super::inspect_test_support::*;` で参照する。
//!
//! 本番可視性は不変。ここに集約するのは test-only ヘルパーのみで `pub(super)` に留める。

use super::*;

use mlua::{LuaOptions, StdLib};

// -----------------------------------------------------------------------
// Test-only VM construction (kept inside the debug module — does NOT depend
// on the tests/ PoC harness). ALL_SAFE so `jit` exists and `debug` is
// excluded; we apply jit.off() so line hooks never miss JIT-compiled lines.
// -----------------------------------------------------------------------
pub(super) fn build_jit_off_vm() -> Lua {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) };
    lua.load("jit.off()").exec().expect("jit.off() must run");
    lua
}

/// Read the chunk name + current line from a hook `Debug` (safe API), to
/// locate the stop line for the test (mirrors hook.rs's pattern).
pub(super) fn source_and_line(debug: &mlua::Debug) -> (String, u32) {
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

pub(super) fn find_var<'a>(vars: &'a [Variable], name: &str) -> Option<&'a Variable> {
    vars.iter().find(|v| v.name == name)
}
