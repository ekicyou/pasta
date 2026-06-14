//! `DebugSession` `.pasta` break-anchor state machine: advancing/clearing the
//! anchor that coalesces breakpoint re-hits across the many `.lua` lines a single
//! `.pasta` line expands to (design "State Management" / "System Flows →
//! アンカーのライフサイクル"; requirements 1.1 / 2.1 / 2.2 / 2.3). Split out of the
//! `session` hub (C5 production split) — child of `session`, so it reaches the
//! `DebugSession` private `pasta_break_anchor` via the ancestor rule.

use crate::debug::source_map::PastaPos;

use super::DebugSession;

impl DebugSession {
    /// Advance the `.pasta` break ANCHOR by one line and report whether the
    /// current line is suppression-eligible (design "State Management" 175-178,
    /// "System Flows → アンカーのライフサイクル" 114-122; requirements 1.1 / 2.1 /
    /// 2.2 / 2.3).
    ///
    /// The return value is **suppression-eligibility**: `true` IFF the current
    /// line sits on the SAME `.pasta` line the session last stopped on (the
    /// anchor), so a breakpoint hit here should be CONSUMED rather than re-stop
    /// (this is the `anchor == cur` test — same invariant as
    /// [`pasta_step_should_stop`](Self::pasta_step_should_stop)'s `origin_pasta ==
    /// Some(cur)`). Transitions over `(anchor, cur)`:
    ///
    /// - `(Some(a), Some(a))` — same `.pasta` line → `true`, anchor UNCHANGED.
    /// - `(Some(a), Some(b))`, `b != a` — moved to a DIFFERENT mapped `.pasta`
    ///   line → CLEAR the anchor to `None`, `false` (2.2: leaving the line; the
    ///   next re-visit re-stops because the anchor is gone).
    /// - `(_, None)` — the current `.lua` line is `.pasta`-unmapped → `false`,
    ///   anchor UNCHANGED (2.1: an unmapped line within the SAME `.pasta` line's
    ///   expansion must NOT falsely clear the anchor).
    /// - `(None, _)` — no anchor → `false`, anchor UNCHANGED.
    ///
    /// The ONLY side effect is clearing on a move to a different `.pasta` line.
    /// ESTABLISHING the anchor (`*anchor = Some(cur)`) is the CALLER's job at stop
    /// time (design 178), NOT done here.
    pub(super) fn update_break_anchor(&self, cur: Option<&PastaPos>) -> bool {
        let mut anchor = self.pasta_break_anchor.borrow_mut();
        match (anchor.as_ref(), cur) {
            // Same `.pasta` line as the anchor: suppression-eligible, anchor kept.
            (Some(a), Some(c)) if a == c => true,
            // Moved to a DIFFERENT mapped `.pasta` line: clear (left the line).
            (Some(_), Some(_)) => {
                *anchor = None;
                false
            }
            // Unmapped line (`cur == None`): keep the anchor, not eligible (2.1).
            // No anchor (`anchor == None`): nothing to suppress.
            _ => false,
        }
    }
}
