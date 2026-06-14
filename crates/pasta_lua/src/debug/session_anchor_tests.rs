//! Inline test cluster externalized from `session.rs` (Task 2.1, pure move).
//! Cluster: `.pasta` line-break anchor state machine.
use super::*;
use super::session_test_support::*;

use std::sync::mpsc;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::source_map::ChunkSourceMap;
use std::collections::BTreeMap;

// =======================================================================
// Task 1 — `.pasta` 行ブレークアンカーの状態機械（フィールド＋遷移ヘルパー）。
//
// These are PURE unit tests of `update_break_anchor` (no VM): they pin the 4
// transitions of the anchor state machine (design §State Management 173-191,
// §System Flows アンカーのライフサイクル 114-122) and the equality invariant
// that two DIFFERENT `.lua` lines mapping to the SAME `.pasta` line resolve to
// an EQUAL `PastaPos` (the precondition for the `anchor == cur` suppression
// check; same invariant as `pasta_step_should_stop`'s `origin_pasta ==
// Some(cur)`). Requirements 1.1, 2.1, 2.2, 2.3.
//
// The transition helper may be dead code until Task 2 (on_line_impl
// integration); these tests verify it in isolation.
// =======================================================================

/// Build a bare session (no VM, dangling channel ends) for pure anchor-helper
/// unit tests. Channels are created and the senders/receivers held by the
/// session; the test does not drive them.
fn anchor_test_session() -> DebugSession {
    // The helper under test (`update_break_anchor`) never touches the
    // channels, so the test-side ends may be dropped immediately.
    let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
    let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();
    DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx)
}

/// Read the current anchor (test observation of the private state field).
fn anchor_of(session: &DebugSession) -> Option<PastaPos> {
    session.pasta_break_anchor.borrow().clone()
}

/// 初期状態: `new` はアンカー無し（`None`）で開始する。
#[test]
fn update_break_anchor_initial_is_none() {
    let session = anchor_test_session();
    assert_eq!(
        anchor_of(&session),
        None,
        "a fresh session must start with NO anchor (design §State Management)"
    );
}

/// 遷移1（design 177: `(Some(a), Some(a))` → `true`・不変）:
/// アンカー == 現在の `.pasta` 行 → 抑制適格 true、アンカー不変（2.1）。
#[test]
fn update_break_anchor_same_pasta_line_returns_true_unchanged() {
    let session = anchor_test_session();
    *session.pasta_break_anchor.borrow_mut() = Some(ppos(10));

    let cur = ppos(10);
    let suppress = session.update_break_anchor(Some(&cur));

    assert!(
        suppress,
        "same `.pasta` line as the anchor must be suppression-eligible (true)"
    );
    assert_eq!(
        anchor_of(&session),
        Some(ppos(10)),
        "the anchor must remain UNCHANGED on the same `.pasta` line"
    );
}

/// 遷移2（design 177: `(Some(a), Some(b!=a))` → `anchor=None`, `false`）:
/// 別の対応 `.pasta` 行へ移動 → 非抑制 false、アンカー解除（2.2）。
#[test]
fn update_break_anchor_different_pasta_line_clears_returns_false() {
    let session = anchor_test_session();
    *session.pasta_break_anchor.borrow_mut() = Some(ppos(10));

    let cur = ppos(11);
    let suppress = session.update_break_anchor(Some(&cur));

    assert!(
        !suppress,
        "a DIFFERENT `.pasta` line must NOT be suppression-eligible (false)"
    );
    assert_eq!(
        anchor_of(&session),
        None,
        "moving to a different `.pasta` line must CLEAR the anchor to None (2.2)"
    );
}

/// 遷移3（design 178: `(_, None)` → `false`・不変）:
/// 未対応行（`cur==None`）→ 非抑制 false、アンカー不変（同一展開内の未対応行で
/// 誤解除しない・2.1）。
#[test]
fn update_break_anchor_unmapped_line_returns_false_unchanged() {
    let session = anchor_test_session();
    *session.pasta_break_anchor.borrow_mut() = Some(ppos(10));

    let suppress = session.update_break_anchor(None);

    assert!(
        !suppress,
        "an unmapped (`None`) line must NOT be suppression-eligible (false)"
    );
    assert_eq!(
        anchor_of(&session),
        Some(ppos(10)),
        "an unmapped line must keep the anchor UNCHANGED (no false clear, 2.1)"
    );
}

/// 遷移4（design 178: `(None, Some)` → `false`・不変）:
/// アンカー無し起点 → 非抑制 false、アンカー不変（確立は呼び出し側の責務）。
#[test]
fn update_break_anchor_no_anchor_returns_false_unchanged() {
    let session = anchor_test_session();
    assert_eq!(anchor_of(&session), None, "precondition: no anchor");

    let cur = ppos(10);
    let suppress = session.update_break_anchor(Some(&cur));

    assert!(
        !suppress,
        "with NO anchor the line must NOT be suppression-eligible (false)"
    );
    assert_eq!(
        anchor_of(&session),
        None,
        "the helper must NOT establish the anchor — that is the CALLER's job \
         at stop time (design 178)"
    );
}

/// 等価不変条件（design Testing Strategy §Unit Tests, line 207 / 1.1, 2.1）:
/// 同一 `.pasta` 行へマップする2つの DIFFERENT `.lua` 行に対し
/// `resolve_current_pasta` が EQUAL な `PastaPos`（同一 file・同一 line）を返す。
/// これがアンカー抑制 `anchor == cur` の前提（既存 `pasta_step_should_stop` の
/// `origin_pasta == Some(cur)` と同一不変条件）。
#[test]
fn two_lua_lines_for_same_pasta_line_resolve_equal_pastapos() {
    // Build a map where two DIFFERENT `.lua` lines (20, 21) both map to the
    // SAME `.pasta` line (10) — the multi-to-one expansion (8.2).
    let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
    forward.insert(20, ppos(10));
    forward.insert(21, ppos(10));
    let mut sm = SourceMap::new();
    sm.insert_chunk(
        PASTA_SOURCE.to_string(),
        "scene.pasta".to_string(),
        ChunkSourceMap::from_forward(forward),
    );
    let map = Arc::new(sm);

    let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
    let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();
    let session = DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx)
        .with_source_map(Some(map), SourceMode::Pasta);

    let a = session
        .resolve_current_pasta(PASTA_SOURCE, 20)
        .expect("lua line 20 must map to a `.pasta` position");
    let b = session
        .resolve_current_pasta(PASTA_SOURCE, 21)
        .expect("lua line 21 must map to a `.pasta` position");

    assert_eq!(
        a, b,
        "two DIFFERENT `.lua` lines mapping to the SAME `.pasta` line must \
         resolve to EQUAL `PastaPos` (file + line) — the `anchor == cur` \
         suppression precondition (1.1, 2.1)"
    );
    assert_eq!(a, ppos(10), "both must resolve to the shared `.pasta` line 10");
}

/// `with_source_map` / `with_shared_mode` MUST NOT touch the anchor state
/// (design File Structure Plan line 90: 「`with_source_map`/`with_shared_mode`
/// は不変」). The anchor stays `None` through both injections.
#[test]
fn injection_helpers_do_not_touch_anchor() {
    let (_cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
    let (event_tx, _event_rx) = mpsc::channel::<SessionEvent>();
    let session = DebugSession::new(BreakpointSet::new(), cmd_rx, event_tx)
        .with_source_map(Some(Arc::new(SourceMap::new())), SourceMode::Pasta)
        .with_shared_mode(None);

    assert_eq!(
        anchor_of(&session),
        None,
        "with_source_map / with_shared_mode must leave the anchor as None \
         (design File Structure Plan: 「不変」)"
    );
}
