//! `BreakpointSet`: the shared breakpoint store and `(source, line)` resolution
//! (design "Breakpoints" component, requirement 1.1).
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
//! by every other handle (this is what makes "set during execution" work).
//!
//! # Lock discipline (hook must never block while holding the lock)
//!
//! [`should_pause`](BreakpointSet::should_pause) is a quick lock / check /
//! unlock: it takes the mutex, runs the containment predicate, and drops the
//! guard before returning. The hook MUST NOT hold this lock across a blocking
//! stop (the blocking stop is task 2.2's `block_until_command`, which runs
//! *after* `should_pause` has already returned and released the lock).
//!
//! # Resolution semantics (DAP `setBreakpoints` is per-source authoritative)
//!
//! [`set_breakpoints`](BreakpointSet::set_breakpoints) REPLACES the breakpoints
//! for the named source (matching DAP semantics, where each `setBreakpoints`
//! call is the authoritative set for that source) while PRESERVING breakpoints
//! belonging to other sources. It returns the resolved breakpoints as
//! `Vec<`[`ResolvedBreakpoint`]`>`, each `verified: true` — Lua-level lines are
//! accepted as-is in this spec; verification refinement (binding to an
//! executable location) is out of scope here.

#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::debug::types::{Breakpoint, ResolvedBreakpoint, SourceRef};

/// Shared, cheaply-cloneable breakpoint store (design "Breakpoints").
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

    /// Containment predicate: does `(source, line)` name a registered
    /// breakpoint? (Promoted from the PoC `PauseGate::is_breakpoint`; R1.1.)
    ///
    /// Returns `true` IFF `(source, line)` is in the set. `Breakpoint` is a
    /// `(String, u32)` tuple, so to match on a borrowed key WITHOUT allocating a
    /// throw-away owned tuple, this iterates `any` (the PoC approach) rather than
    /// building `(source.to_string(), line)` just to `contains` it.
    ///
    /// Lock discipline: the mutex is taken, the predicate runs, and the guard is
    /// dropped before returning — the hook never holds this lock across a
    /// blocking stop. A poisoned lock degrades to `false` (do not pause) rather
    /// than panicking inside the VM-thread hook.
    pub(crate) fn should_pause(&self, source: &str, line: u32) -> bool {
        let Ok(guard) = self.inner.lock() else {
            // Poisoned lock: fail safe by not pausing (never panic in the hook).
            return false;
        };
        guard.iter().any(|(s, l)| s == source && *l == line)
    }

    /// Replace the breakpoints for `source` with `lines` (DAP `setBreakpoints`
    /// per-source authoritative semantics) and return the resolved set (R1.1).
    ///
    /// Only breakpoints belonging to `source` are replaced; breakpoints for
    /// every OTHER source are preserved. Each returned [`ResolvedBreakpoint`] is
    /// `verified: true` (Lua-level lines accepted as-is; verification refinement
    /// is out of scope for this spec).
    ///
    /// The returned `Vec` mirrors the requested `lines` order (the controller
    /// reports these back to the IDE in the order they were requested).
    pub(crate) fn set_breakpoints(
        &self,
        source: &SourceRef,
        lines: &[u32],
    ) -> Vec<ResolvedBreakpoint> {
        let path = source.path.as_str();

        if let Ok(mut guard) = self.inner.lock() {
            // Per-source authoritative: drop this source's prior breakpoints,
            // keep every other source's.
            guard.retain(|(s, _)| s != path);
            for &line in lines {
                guard.insert((path.to_string(), line));
            }
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(path: &str) -> SourceRef {
        SourceRef::new(path)
    }

    /// R1.1 (containment predicate, promoting the PoC
    /// `should_pause_matches_only_target_lines`): an exact `(source, line)`
    /// match pauses, while same-source/different-line AND
    /// same-line/different-source do NOT (non-vacuous: both true and false).
    #[test]
    fn should_pause_matches_only_target_lines() {
        let set = BreakpointSet::new();
        set.set_breakpoints(&src("@s"), &[3]);

        // Exact (source, line) match → true.
        assert!(
            set.should_pause("@s", 3),
            "exact (source,line) match must pause"
        );
        // Same source, different line → false.
        assert!(
            !set.should_pause("@s", 2),
            "same source but different line must NOT pause"
        );
        // Same line, different source → false.
        assert!(
            !set.should_pause("@other", 3),
            "same line but different source must NOT pause"
        );
    }

    /// An empty store pauses for nothing (baseline).
    #[test]
    fn empty_set_never_pauses() {
        let set = BreakpointSet::new();
        assert!(!set.should_pause("@s", 1));
        assert!(!set.should_pause("", 0));
    }

    /// `set_breakpoints` replaces ONLY the target source's lines, preserves
    /// other sources', and returns the resolved (source, line) verified entries
    /// (R1.1; DAP per-source authoritative semantics).
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

        // Both sources are independently active.
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
    /// "settable during execution" requirement): a clone observed from another
    /// thread sees an update made via the original handle.
    #[test]
    fn clone_observes_cross_thread_update() {
        use std::sync::mpsc;

        let original = BreakpointSet::new();
        let reader = original.clone();

        // Reader thread waits for a go signal, then reads the SHARED set.
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
