//! `BreakpointSet`: the shared breakpoint store with a **two-tier key** —
//! present source (DAP retain/replace key) + resolved execution coordinate
//! `(chunk, lua_line)` (stop-decision key). (design "BpTranslator 二段キー"
//! 516-528, requirements 4.4 / 8.2.)
//!
//! # Shared, settable-during-execution store (design "System Flows" main 判断)
//!
//! Breakpoints must be settable *while the VM is executing*: the design states
//! "`setBreakpoints` のみ実行中でも可（`Arc<Mutex>` 共有）". [`BreakpointSet`] is
//! therefore a `#[derive(Clone)]` newtype over
//! [`Arc`]`<`[`Mutex`](std::sync::Mutex)`<`[`HashSet`]`<`[`Breakpoint`]`>>>`: the
//! VM-thread hook holds one clone and reads via [`BreakpointSet::should_pause`],
//! while the controller / transport thread holds another clone and writes via
//! [`BreakpointSet::set_breakpoints`]. Cloning is cheap (an `Arc` bump) and all
//! clones share the same inner set, so an update made on one handle is observed
//! by every other handle (this is what makes "set during execution" work). The
//! same shared `Arc` is what makes registered breakpoints **persist across the
//! short-lived multi-request debug sessions** of requirement 4.4.
//!
//! # Lock discipline (hook must never block while holding the lock)
//!
//! [`should_pause`](BreakpointSet::should_pause) is a quick lock / check /
//! unlock: it takes the mutex, runs the containment predicate, and drops the
//! guard before returning. The hook MUST NOT hold this lock across a blocking
//! stop (the blocking stop is task 2.2's `block_until_command`, which runs
//! *after* `should_pause` has already returned and released the lock).
//!
//! # Two-tier resolution semantics (the heart of this module)
//!
//! A [`Breakpoint`] carries two independent keys (design 519-524):
//!
//! - **present source** ([`Breakpoint::present_source`]) — the source path the
//!   IDE presented the breakpoint under (`.pasta` or `.lua`). This is the
//!   **retain / replace key**.
//!   [`set_breakpoints`](BreakpointSet::set_breakpoints) REPLACES the
//!   breakpoints whose `present_source` matches the request's source — and ONLY
//!   those (matching DAP semantics, where each `setBreakpoints` call is
//!   authoritative for the source it names). Breakpoints presented under any
//!   OTHER source are preserved **even if they resolve into the same chunk**
//!   (requirement 4.4: registered BPs persist; requirement 8.2: a `.pasta`- and
//!   a `.lua`-origin BP in one chunk never mutually evict).
//! - **execution coordinate** (`chunk` + `lua_line`) — the resolved `.lua`
//!   position the runtime hook actually reaches. This is the
//!   **stop-decision key**. The hook reports the executing chunk source + line
//!   and [`should_pause`](BreakpointSet::should_pause) matches against
//!   `(chunk, lua_line)`; the presented space is irrelevant to whether we stop.
//!
//! For the existing `.lua` path both tiers collapse to the same `.lua` identity
//! (the caller sets `present_source == chunk ==` the DAP `.lua` source path), so
//! the prior single-key behaviour is preserved byte-for-byte. The `.pasta`
//! translation that populates `present_source = .pasta` with a distinct
//! `chunk`/`lua_line` is task 5.3.
//!
//! [`set_breakpoints`](BreakpointSet::set_breakpoints) returns the resolved
//! breakpoints as `Vec<`[`ResolvedBreakpoint`]`>`, each `verified: true` —
//! Lua-level lines are accepted as-is in this spec; verification refinement
//! (binding to an executable location) is out of scope here.

#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::debug::source_map::canonicalize_chunk_name;
use crate::debug::types::{Breakpoint, ResolvedBreakpoint, SourceRef};

/// Shared, cheaply-cloneable breakpoint store (design "Breakpoints"), keyed by a
/// two-tier [`Breakpoint`] (present source + execution coordinate).
///
/// A `#[derive(Clone)]` newtype over `Arc<Mutex<HashSet<Breakpoint>>>` so it can
/// be shared across the VM-thread hook (reads via [`should_pause`]) and the
/// controller / transport thread (writes via [`set_breakpoints`]) without data
/// races. Every clone shares the same inner set.
///
/// [`should_pause`]: BreakpointSet::should_pause
/// [`set_breakpoints`]: BreakpointSet::set_breakpoints
#[derive(Clone, Default)]
pub(crate) struct BreakpointSet {
    inner: Arc<Mutex<HashSet<Breakpoint>>>,
}

impl BreakpointSet {
    /// Construct an empty breakpoint store.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Stop-decision predicate: does the **execution coordinate** `(chunk,
    /// line)` name a registered breakpoint? (R1.1; two-tier stop key, design
    /// 525-528.)
    ///
    /// The hook reports the executing chunk source (`chunk`) and current line;
    /// this returns `true` IFF some registered [`Breakpoint`] has
    /// `bp.chunk == chunk && bp.lua_line == line` — REGARDLESS of which
    /// `present_source` it was registered under. So a `.pasta`-presented and a
    /// `.lua`-presented breakpoint that resolve to the same `(chunk, line)` BOTH
    /// fire, and neither is masked by the other (requirement 8.2).
    ///
    /// `chunk` matching is by the **canonicalized chunk name** (task 5.3
    /// reconciliation): both the incoming hook source `chunk` AND each stored
    /// `bp.chunk` are run through [`canonicalize_chunk_name`] before comparison.
    /// This is required because the runtime hook reports the RAW `@<abs .lua
    /// path>` source while the `.pasta` BP translator (task 5.3) registers the
    /// CANONICALIZED chunk that `SourceMap::resolve_pasta_to_lua` returns — a raw
    /// vs. canonical mismatch would otherwise mean a `.pasta`-translated BP never
    /// fires (requirements 4.2 / 8.2). It is safe for existing `.lua`
    /// breakpoints: (a) two raw strings that were equal stay equal after
    /// canonicalization, and (b) task 1.1 empirically confirmed the hook source
    /// and the loader/cache key are EQUAL after `canonicalize_chunk_name`, so the
    /// `.lua` path (which stores the hook-form source verbatim) still matches
    /// (requirement 7.2).
    ///
    /// Iterates `any` (rather than building a throw-away owned key to
    /// `contains`) so it can match without allocating per entry beyond the
    /// canonical forms it must compute. The incoming `chunk` is canonicalized
    /// once up front; each entry's `chunk` is canonicalized as it is visited.
    ///
    /// Lock discipline: the mutex is taken, the predicate runs, and the guard is
    /// dropped before returning — the hook never holds this lock across a
    /// blocking stop. A poisoned lock degrades to `false` (do not pause) rather
    /// than panicking inside the VM-thread hook.
    pub(crate) fn should_pause(&self, chunk: &str, line: u32) -> bool {
        let Ok(guard) = self.inner.lock() else {
            // Poisoned lock: fail safe by not pausing (never panic in the hook).
            return false;
        };
        // Canonicalize the incoming hook source ONCE (task 5.3 reconciliation),
        // then compare against each stored entry's canonicalized chunk so a
        // `.pasta`-translated BP (canonical chunk) fires for the raw hook coord
        // (4.2) and an existing `.lua` BP keeps matching (7.2).
        let hook_canon = canonicalize_chunk_name(chunk);
        guard
            .iter()
            .any(|bp| bp.lua_line == line && canonicalize_chunk_name(&bp.chunk) == hook_canon)
    }

    /// Register breakpoints for a presented source, replacing any previously
    /// registered under THAT present source (DAP per-source authoritative
    /// semantics) and return the resolved set (R1.1, requirements 4.4 / 8.2).
    ///
    /// This is the existing `.lua` registration path: the presented source IS
    /// the `.lua` chunk, so each requested `line` becomes a [`Breakpoint`] whose
    /// `present_source`, `chunk`, and `lua_line` collapse to
    /// `(source.path, source.path, line)` — identical to the prior single-key
    /// behaviour. (The `.pasta` path, which fills a distinct `chunk`/`lua_line`
    /// per requested line via the source map, is task 5.3 and will call
    /// [`register`](Self::register) with explicit execution coordinates.)
    ///
    /// **Retain/replace is by present source only.** Entries whose
    /// `present_source` differs are preserved — even if they share a `chunk`
    /// with the replaced source — so a `.pasta`-origin and a `.lua`-origin
    /// breakpoint in the same chunk never evict one another (requirements 4.4 /
    /// 8.2).
    ///
    /// Each returned [`ResolvedBreakpoint`] is `verified: true` (Lua-level lines
    /// accepted as-is; verification refinement is out of scope for this spec).
    /// The returned `Vec` mirrors the requested `lines` order.
    pub(crate) fn set_breakpoints(
        &self,
        source: &SourceRef,
        lines: &[u32],
    ) -> Vec<ResolvedBreakpoint> {
        let path = source.path.as_str();

        // The `.lua` path: present source == execution chunk; each requested
        // line is its own execution coordinate.
        let entries: Vec<Breakpoint> = lines
            .iter()
            .map(|&line| Breakpoint::new(path, path, line))
            .collect();
        self.register(path, entries);

        // Resolve in requested order. Lua-level lines are accepted as-is, so
        // every requested line is verified.
        lines
            .iter()
            .map(|&line| ResolvedBreakpoint {
                source: source.clone(),
                line,
                verified: true,
            })
            .collect()
    }

    /// Replace every breakpoint registered under `present_source` with `entries`
    /// (the per-present-source authoritative core), preserving all breakpoints
    /// from OTHER present sources (requirements 4.4 / 8.2).
    ///
    /// This is the shared registration primitive the `.lua` path
    /// ([`set_breakpoints`](Self::set_breakpoints)) and the future `.pasta`
    /// translation (task 5.3) both funnel through: the caller resolves a
    /// presented source into zero-or-more execution-coordinate [`Breakpoint`]s
    /// (one `.pasta` line may yield MANY, requirement 8.2) and hands them here
    /// tagged with the same `present_source`. Retain/replace keys ONLY on
    /// `present_source`, so a different presented source's breakpoints survive
    /// even when they resolve into the same `chunk` (no cross-source eviction).
    ///
    /// `entries` MUST all carry `present_source == present_source` (the caller
    /// builds them that way); this method does not re-tag them.
    ///
    /// A poisoned lock is a no-op (never panic across the shared store).
    pub(crate) fn register(&self, present_source: &str, entries: Vec<Breakpoint>) {
        if let Ok(mut guard) = self.inner.lock() {
            // Per-PRESENT-SOURCE authoritative: drop only this present source's
            // prior entries; every other present source's entries are kept,
            // even if they target the same chunk.
            guard.retain(|bp| bp.present_source != present_source);
            for bp in entries {
                guard.insert(bp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(path: &str) -> SourceRef {
        SourceRef::new(path)
    }

    /// R1.1 (stop-decision predicate): `should_pause` matches by the EXECUTION
    /// COORDINATE `(chunk, lua_line)`. An exact `(chunk, line)` match pauses,
    /// while same-chunk/different-line AND same-line/different-chunk do NOT
    /// (non-vacuous: both true and false). The presented source is irrelevant
    /// to the stop decision (design 525-528).
    #[test]
    fn should_pause_matches_by_execution_coordinate() {
        let set = BreakpointSet::new();
        // A `.pasta`-presented BP whose RESOLVED execution coord is
        // (@C:/proj/a.lua, 5).
        set.register(
            "x.pasta",
            vec![Breakpoint::new("x.pasta", "@C:/proj/a.lua", 5)],
        );

        // Exact (chunk, line) match → true (the hook reports the chunk source).
        assert!(
            set.should_pause("@C:/proj/a.lua", 5),
            "exact (chunk,line) execution-coord match must pause"
        );
        // Same chunk, different line → false.
        assert!(
            !set.should_pause("@C:/proj/a.lua", 4),
            "same chunk but different line must NOT pause"
        );
        // Same line, different chunk → false.
        assert!(
            !set.should_pause("@C:/proj/b.lua", 5),
            "same line but different chunk must NOT pause"
        );
    }

    /// The COMPLETION CRITERION (requirements 4.4 / 8.2): a `.pasta`-origin BP
    /// and a `.lua`-origin BP that resolve into the SAME chunk must NOT evict
    /// each other when one present source is re-set. They are keyed by present
    /// source for retain/replace, by execution coord for stopping — so both
    /// remain registered and both fire.
    #[test]
    fn no_cross_present_source_eviction_when_sharing_a_chunk() {
        let set = BreakpointSet::new();
        let chunk = "@C:/proj/scene.lua";

        // A `.lua`-origin BP at (scene.lua, 10), presented under "scene.lua".
        set.register("scene.lua", vec![Breakpoint::new("scene.lua", chunk, 10)]);
        // A `.pasta`-origin BP that ALSO resolved into the same chunk at the
        // same line, presented under "x.pasta".
        set.register("x.pasta", vec![Breakpoint::new("x.pasta", chunk, 10)]);

        // Both fire (same execution coord).
        assert!(set.should_pause(chunk, 10), "both BPs fire at the shared coord");

        // Re-set the `.pasta` present source authoritatively (replace its set):
        // this must NOT remove the `.lua`-origin BP in the same chunk.
        set.register("x.pasta", vec![Breakpoint::new("x.pasta", chunk, 20)]);

        assert!(
            set.should_pause(chunk, 10),
            "the `.lua`-origin BP in the shared chunk must survive replacing the \
             `.pasta` present source (4.4/8.2 no cross-source eviction)"
        );
        assert!(
            set.should_pause(chunk, 20),
            "the `.pasta` present source's new coord is registered"
        );

        // Symmetric: re-setting the `.lua` present source must not touch the
        // `.pasta` BP at line 20.
        set.register("scene.lua", vec![Breakpoint::new("scene.lua", chunk, 30)]);
        assert!(
            set.should_pause(chunk, 20),
            "the `.pasta`-origin BP must survive replacing the `.lua` present source"
        );
        assert!(set.should_pause(chunk, 30), "the `.lua` present source's new coord is registered");
        assert!(
            !set.should_pause(chunk, 10),
            "the `.lua` present source's OLD coord was authoritatively replaced"
        );
    }

    /// R8.2: a single presented line expanding to MULTIPLE execution
    /// coordinates (multiple `.lua` lines in one chunk) registers ALL of them,
    /// and stopping at ANY one of them fires.
    #[test]
    fn one_present_line_registers_multiple_execution_coords() {
        let set = BreakpointSet::new();
        let chunk = "@C:/proj/expanded.lua";

        // One `.pasta` line → three `.lua` lines (a macro-style expansion).
        set.register(
            "x.pasta",
            vec![
                Breakpoint::new("x.pasta", chunk, 7),
                Breakpoint::new("x.pasta", chunk, 8),
                Breakpoint::new("x.pasta", chunk, 9),
            ],
        );

        assert!(set.should_pause(chunk, 7), "each expanded `.lua` line fires (8.2)");
        assert!(set.should_pause(chunk, 8), "each expanded `.lua` line fires (8.2)");
        assert!(set.should_pause(chunk, 9), "each expanded `.lua` line fires (8.2)");
        assert!(!set.should_pause(chunk, 6), "non-registered line does not fire");
    }

    /// Task 5.3 reconciliation (requirement 4.2): a `.pasta`-translated BP is
    /// registered with the CANONICALIZED chunk that `resolve_pasta_to_lua`
    /// returns, while the runtime hook reports the RAW `@<abs .lua path>` source.
    /// `should_pause` must canonicalize BOTH sides so the BP still fires for the
    /// raw hook coordinate. Without the canonicalization in `should_pause` the
    /// raw hook source would not equal the stored canonical chunk and this would
    /// fail (the `.pasta` BP would never stop).
    #[test]
    fn should_pause_matches_canonicalized_chunk_for_raw_hook_source() {
        let set = BreakpointSet::new();
        // A `.pasta`-translated BP whose stored chunk is the CANONICAL form that
        // `SourceMap::resolve_pasta_to_lua` yields (no `@`, `/` separators, and
        // lowercase on Windows).
        let canonical_chunk = if cfg!(windows) {
            "c:/proj/cache/scene.lua"
        } else {
            "C:/proj/cache/scene.lua"
        };
        set.register(
            "scene.pasta",
            vec![Breakpoint::new("scene.pasta", canonical_chunk, 12)],
        );

        // The hook reports the RAW source (`@` prefix, backslash separators,
        // possibly different case) — it must still match after canonicalization.
        assert!(
            set.should_pause(r"@C:\proj\cache\scene.lua", 12),
            "a `.pasta`-translated BP (canonical chunk) must fire for the raw hook \
             source after `should_pause` canonicalizes both sides (4.2)"
        );
        // Same chunk, different line → no stop.
        assert!(
            !set.should_pause(r"@C:\proj\cache\scene.lua", 11),
            "same chunk, different line must not pause"
        );
        // Different chunk, same line → no stop.
        assert!(
            !set.should_pause(r"@C:\proj\cache\other.lua", 12),
            "different chunk must not pause even with canonicalization"
        );
    }

    /// Task 5.3 / requirement 7.2: an existing `.lua` BP registered with the
    /// RAW hook-form source (as `set_breakpoints` does) STILL fires after the
    /// `should_pause` canonicalization change — equal raw strings stay equal
    /// once canonicalized, so no `.lua` regression.
    #[test]
    fn should_pause_lua_path_unchanged_after_canonicalization() {
        let set = BreakpointSet::new();
        // The `.lua` path stores present_source == chunk == the raw source.
        set.set_breakpoints(&src("@e2e_scenario"), &[7]);
        assert!(
            set.should_pause("@e2e_scenario", 7),
            "an existing `.lua` BP must still fire after canonicalization (7.2)"
        );
        assert!(!set.should_pause("@e2e_scenario", 6));
    }

    /// An empty store pauses for nothing (baseline).
    #[test]
    fn empty_set_never_pauses() {
        let set = BreakpointSet::new();
        assert!(!set.should_pause("@s", 1));
        assert!(!set.should_pause("", 0));
    }

    /// The existing `.lua` path: `set_breakpoints` registers each requested line
    /// as an execution coordinate under the `.lua` present source, and the line
    /// fires by execution coordinate — preserving prior single-key behaviour
    /// (R1.1, requirement 7.2). present_source == chunk == the `.lua` path here.
    #[test]
    fn set_breakpoints_lua_path_registers_execution_coords() {
        let set = BreakpointSet::new();
        set.set_breakpoints(&src("@s"), &[3]);

        // Exact (chunk == "@s", line == 3) match → true.
        assert!(set.should_pause("@s", 3), "exact .lua (chunk,line) match must pause");
        // Same chunk, different line → false.
        assert!(!set.should_pause("@s", 2), "same .lua chunk but different line must NOT pause");
        // Same line, different chunk → false.
        assert!(!set.should_pause("@other", 3), "same line but different .lua chunk must NOT pause");
    }

    /// `set_breakpoints` replaces ONLY the target source's lines, preserves
    /// other sources', and returns the resolved (source, line) verified entries
    /// (R1.1; DAP per-source authoritative semantics; requirement 7.2 `.lua`).
    #[test]
    fn set_breakpoints_replaces_only_target_source_and_preserves_others() {
        let set = BreakpointSet::new();

        // Seed two sources.
        set.set_breakpoints(&src("@a"), &[1, 2, 3]);
        let resolved_b = set.set_breakpoints(&src("@b"), &[10, 20]);

        // Resolved output for @b carries the requested (source, line) verified.
        assert_eq!(
            resolved_b,
            vec![
                ResolvedBreakpoint {
                    source: src("@b"),
                    line: 10,
                    verified: true
                },
                ResolvedBreakpoint {
                    source: src("@b"),
                    line: 20,
                    verified: true
                },
            ],
            "resolved breakpoints must mirror requested lines as verified"
        );

        // Both sources are independently active (matched by execution coord,
        // which == the `.lua` source path here).
        assert!(set.should_pause("@a", 2));
        assert!(set.should_pause("@b", 10));

        // Replace @a's set authoritatively: line 2 (old) is gone, 5 (new) is in.
        let resolved_a = set.set_breakpoints(&src("@a"), &[5]);
        assert_eq!(
            resolved_a,
            vec![ResolvedBreakpoint {
                source: src("@a"),
                line: 5,
                verified: true
            }]
        );
        assert!(
            !set.should_pause("@a", 2),
            "replaced source must drop its previous lines"
        );
        assert!(
            !set.should_pause("@a", 1),
            "replaced source must drop ALL its previous lines"
        );
        assert!(set.should_pause("@a", 5), "replaced source gets the new line");

        // @b must be untouched by replacing @a.
        assert!(
            set.should_pause("@b", 10) && set.should_pause("@b", 20),
            "replacing one source must preserve the other source's breakpoints"
        );
    }

    /// Clearing a source (empty `lines`) removes its breakpoints but leaves
    /// other sources intact (DAP clears a file by sending zero lines).
    #[test]
    fn set_breakpoints_with_empty_lines_clears_that_source_only() {
        let set = BreakpointSet::new();
        set.set_breakpoints(&src("@a"), &[1]);
        set.set_breakpoints(&src("@b"), &[2]);

        let resolved = set.set_breakpoints(&src("@a"), &[]);
        assert!(resolved.is_empty(), "clearing returns no resolved entries");
        assert!(!set.should_pause("@a", 1), "@a is cleared");
        assert!(set.should_pause("@b", 2), "@b is preserved");
    }

    /// Concurrency smoke (proves `Arc<Mutex>` sharing for the
    /// "settable during execution" requirement AND the 4.4 persistence vehicle):
    /// a clone observed from another thread sees an update made via the original
    /// handle.
    #[test]
    fn clone_observes_cross_thread_update() {
        use std::sync::mpsc;

        let original = BreakpointSet::new();
        let reader = original.clone();

        // Reader thread waits for a go signal, then reads the SHARED set by
        // execution coordinate (chunk == "@s", line == 7).
        let (go_tx, go_rx) = mpsc::channel::<()>();
        let handle = std::thread::spawn(move || {
            go_rx.recv().expect("go signal");
            reader.should_pause("@s", 7)
        });

        // The original handle sets a breakpoint AFTER the clone crossed the
        // thread boundary — simulating "set during execution".
        original.set_breakpoints(&src("@s"), &[7]);
        go_tx.send(()).expect("send go");

        let observed = handle.join().expect("reader thread must not panic");
        assert!(
            observed,
            "a clone on another thread must observe an update made via the \
             original handle (Arc<Mutex> sharing)"
        );
    }
}
