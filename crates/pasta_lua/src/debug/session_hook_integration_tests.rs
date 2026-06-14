//! Inline test cluster externalized from `session.rs` (Task 2.1, pure move).
//! Cluster: line-hook decision integration (re-break consumption).
use super::*;
use super::session_test_support::*;

use std::sync::Arc;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::source_map::ChunkSourceMap;
use crate::debug::types::{SourceRef, StopReason};
use std::collections::BTreeMap;

// =======================================================================
// Task 2 — 行フック判定への統合（再ブレーク消化の本体）。
//
// Session-level (host-thread) aggregates over `on_line_impl`:
//   - the BUG fix: a `.pasta`-line breakpoint registered on MULTIPLE mapped
//     `.lua` lines stops ONCE; one Continue does NOT re-stop on the SAME
//     `.pasta` line but advances to the next mapped stop point (1.1–1.3,
//     2.4, 3.1, 3.2, 5.2);
//   - loop re-visit re-stops (2.2);
//   - the gating invariant: `.lua` mode + no source map keep the existing
//     `.lua`-granularity behavior unchanged (4.1, 4.2).
// =======================================================================

/// 1.1 / 1.2 / 2.4 / 3.2 / 5.2 (the bug fix): `.pasta` line 10 maps to BOTH
/// `.lua` lines 6 AND 7 (`pasta_scenario_map`). With a breakpoint on BOTH (and
/// on line 9 = `.pasta` 11, a DIFFERENT line), the session must stop ONCE on
/// `.pasta` 10 (at line 6), and a single Continue must NOT re-stop on line 7
/// (same `.pasta` 10 — consumed) but advance to line 9 (`.pasta` 11).
///
/// BEFORE the integration this FAILS: line 7's breakpoint re-stops on the same
/// `.pasta` line 10 (the user cannot escape the line with one Continue).
#[test]
fn continue_escapes_pasta_line_with_breakpoints_on_multiple_mapped_lua_lines() {
    let breakpoints = BreakpointSet::new();
    // BP on BOTH `.lua` lines mapping to `.pasta` 10 (lines 6, 7) AND on
    // line 9 (`.pasta` 11, the next DIFFERENT `.pasta` line).
    breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[6, 7, 9]);
    let mut host = start_step_host_with_map(
        breakpoints,
        PASTA_CHUNK,
        PASTA_SOURCE,
        Some(pasta_scenario_map()),
        SourceMode::Pasta,
    );

    // First (and only) stop on `.pasta` 10: at line 6, reason Breakpoint (2.4,
    // 3.1).
    let (reason, line) = host.recv_stop();
    assert_eq!(reason, StopReason::Breakpoint);
    assert_eq!(line, 6, "must stop once on `.pasta` 10 at line 6 (2.4)");

    // One Continue must ESCAPE `.pasta` 10: line 7 (same `.pasta` 10) is
    // consumed (no extra Stopped event — 3.2), line 8 (unmapped) is passed,
    // and the next stop is line 9 (`.pasta` 11) — NOT line 7 (1.1, 1.2, 5.2).
    host.cont(SessionCommand::Continue);
    let (reason, line) = host.recv_stop();
    assert_eq!(
        reason,
        StopReason::Breakpoint,
        "the next stop after escaping `.pasta` 10 is a Breakpoint (3.1)"
    );
    assert_eq!(
        line, 9,
        "one Continue must escape `.pasta` 10: line 7 (same `.pasta` 10) is \
         consumed and the next stop is line 9 (`.pasta` 11) — NOT a re-stop on \
         line 7 (1.1, 1.2, 5.2)"
    );

    host.cont(SessionCommand::Continue);
    host.join();
}

/// 2.2 (loop re-visit re-stops): a `.pasta` line visited again via a loop must
/// re-stop on the new visit. The loop body line maps to one `.pasta` line; a
/// DIFFERENT `.pasta` line inside the loop clears the anchor each turn, so the
/// breakpoint fires once PER visit.
#[test]
fn loop_revisit_restops_on_same_pasta_line() {
    // Chunk: a 3-iteration loop. Lines (1-origin):
    //   1: local s = 0
    //   2: for i = 1, 3 do        <- loop header
    //   3:     s = s + i          <- `.pasta` 50 (BREAKPOINT — once per visit)
    //   4:     s = s + 0          <- `.pasta` 51 (DIFFERENT — clears the anchor)
    //   5: end
    //   6: return s
    const LOOP_SOURCE: &str = "@pasta_loop_scenario";
    const LOOP_CHUNK: &str = "\
local s = 0
for i = 1, 3 do
s = s + i
s = s + 0
end
return s
";
    let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
    forward.insert(3, PastaPos { file: "loop.pasta".to_string(), line: 50 });
    forward.insert(4, PastaPos { file: "loop.pasta".to_string(), line: 51 });
    let mut sm = SourceMap::new();
    sm.insert_chunk(
        LOOP_SOURCE.to_string(),
        "loop.pasta".to_string(),
        ChunkSourceMap::from_forward(forward),
    );

    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(LOOP_SOURCE), &[3]);
    let mut host = start_step_host_with_map(
        breakpoints,
        LOOP_CHUNK,
        LOOP_SOURCE,
        Some(Arc::new(sm)),
        SourceMode::Pasta,
    );

    // The breakpoint on `.pasta` 50 (line 3) must fire ONCE PER loop iteration
    // (3 visits) — the anchor is cleared by line 4 (`.pasta` 51) each turn (2.2).
    for visit in 1..=3 {
        let (reason, line) = host.recv_stop();
        assert_eq!(reason, StopReason::Breakpoint);
        assert_eq!(
            line, 3,
            "loop visit {visit} must re-stop on `.pasta` 50 (line 3) (2.2)"
        );
        host.cont(SessionCommand::Continue);
    }

    host.join();
}

/// 4.1 (`.lua` mode unchanged): with `SourceMode::Lua` AND a map present, the
/// breakpoint-first path keeps `.lua` granularity — NO `.pasta` aggregation.
/// A breakpoint on BOTH `.lua` lines 6 and 7 (which map to the SAME `.pasta`
/// line 10) stops on EACH (line 6, then on Continue line 7) — the anchor is
/// never touched in `.lua` mode.
#[test]
fn lua_mode_does_not_coalesce_breakpoints_even_with_map() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[6, 7]);
    let mut host = start_step_host_with_map(
        breakpoints,
        PASTA_CHUNK,
        PASTA_SOURCE,
        Some(pasta_scenario_map()),
        SourceMode::Lua,
    );

    let (reason, line) = host.recv_stop();
    assert_eq!(reason, StopReason::Breakpoint);
    assert_eq!(line, 6, "first stop at line 6");

    // In `.lua` mode the SAME-`.pasta`-line line 7 is NOT coalesced: it
    // re-stops at `.lua` granularity (4.1 — aggregation does not apply).
    host.cont(SessionCommand::Continue);
    let (reason, line) = host.recv_stop();
    assert_eq!(reason, StopReason::Breakpoint);
    assert_eq!(
        line, 7,
        "`.lua` mode must stop at EACH `.lua` line (7), NOT coalesce by \
         `.pasta` line (4.1)"
    );

    host.cont(SessionCommand::Continue);
    host.join();
}

/// 4.2 (no source map → existing behavior unchanged): with `SourceMode::Pasta`
/// but NO map, the anchor path is inert (`source_map.is_some()` is false) — the
/// breakpoint-first path is byte-identical to before. A breakpoint on lines 6
/// and 7 stops on EACH, exactly as the pre-spec `.lua`-granularity behavior.
#[test]
fn no_source_map_keeps_existing_breakpoint_behavior() {
    let breakpoints = BreakpointSet::new();
    breakpoints.set_breakpoints(&SourceRef::new(PASTA_SOURCE), &[6, 7]);
    // Pasta mode but NO map → gating disables the anchor (4.2).
    let mut host = start_step_host_with_map(
        breakpoints,
        PASTA_CHUNK,
        PASTA_SOURCE,
        None,
        SourceMode::Pasta,
    );

    let (reason, line) = host.recv_stop();
    assert_eq!(reason, StopReason::Breakpoint);
    assert_eq!(line, 6, "first stop at line 6");

    host.cont(SessionCommand::Continue);
    let (reason, line) = host.recv_stop();
    assert_eq!(reason, StopReason::Breakpoint);
    assert_eq!(
        line, 7,
        "with no source map the breakpoint path is unchanged: stop on EACH \
         `.lua` line (4.2)"
    );

    host.cont(SessionCommand::Continue);
    host.join();
}
