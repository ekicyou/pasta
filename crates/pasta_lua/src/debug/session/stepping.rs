//! `DebugSession` stepping decisions: the StepController over/into/out logic and
//! its `.pasta`-granular refinement (design "DebugSession 状態機械（StepController）"
//! / "PastaStepper"; requirements 1.3 / 1.4 / 1.5 / 9.1–9.5). Split out of the
//! `session` hub (C5 production split) — child of `session`, so it reaches the
//! `DebugSession` private state via the ancestor rule (no visibility widening).

use mlua::Lua;

use crate::debug::SourceMode;
use crate::debug::inspect::capture_stack;
use crate::debug::source_map::PastaPos;
// `SourceMap` is referenced only in the `resolve_current_pasta` doc comments
// (`[`SourceMap`]` / `[`SourceMap::resolve_lua_to_pasta`]`), which the verbatim
// move preserves; the code reaches the map through the hub's private
// `source_map` field. Keep the import so the intra-doc links resolve without
// editing the moved doc text, and silence the false-positive unused-import lint.
#[allow(unused_imports)]
use crate::debug::source_map::SourceMap;
use crate::debug::types::ThreadId;

use super::{DebugSession, StepKind};

impl DebugSession {
    /// The running coroutine's identity and current Lua call depth, observed
    /// from inside the hook (design "DebugSession 状態機械").
    ///
    /// `lua.current_thread()` inside the hook resolves to the RUNNING coroutine
    /// (proven by task 2.4); its `.state()` pointer is a STABLE [`ThreadId`]
    /// across `yield`/`resume`. The depth is the number of Lua frames on that
    /// thread, reused from [`capture_stack`] (no extra FFI helper — keeps the
    /// StepController boundary inside `session.rs`).
    pub(super) fn current_thread_and_depth(lua: &Lua) -> (ThreadId, u32) {
        let thread = lua.current_thread();
        let tid = ThreadId::from_state(thread.state());
        let depth = capture_stack(lua, &thread).len() as u32;
        (tid, depth)
    }

    /// Resolve the current `(source, line)` to a `.pasta` position FOR STEPPING,
    /// gated on `.pasta` granularity being active (task 5.4 / requirements 9.5).
    ///
    /// Returns `Some(PastaPos)` ONLY when this session is in
    /// [`SourceMode::Pasta`] AND a [`SourceMap`] is present AND the current
    /// `.lua` line maps to a `.pasta` position. Otherwise `None`:
    /// - `SourceMode::Lua` or no map → `.pasta` stepping is disabled, the stepper
    ///   keeps its existing `.lua` granularity (9.5).
    /// - mapped chunk/line miss → the line is `.pasta`-unmapped (passed through;
    ///   9.4).
    ///
    /// The RAW hook `source` is passed straight through to
    /// [`SourceMap::resolve_lua_to_pasta`], which canonicalizes the chunk
    /// internally (task 3.4); the caller must NOT pre-canonicalize.
    pub(super) fn resolve_current_pasta(&self, source: &str, line: u32) -> Option<PastaPos> {
        // Read the EFFECTIVE mode (shared cell when threaded) so a DAP `attach`
        // `sourcePresentation` flip switches `.pasta` stepping for this session
        // (task 5.5 / requirement 6.3).
        if self.effective_mode() != SourceMode::Pasta {
            return None;
        }
        let map = self.source_map.as_ref()?;
        map.resolve_lua_to_pasta(source, line).cloned()
    }

    /// The step-stop decision for [`RunMode::Stepping`] (design
    /// "DebugSession 状態機械（StepController）").
    ///
    /// Returns `true` IFF the current line is the step's completion point:
    ///
    /// - **thread mismatch** (the current thread is NOT `step.thread` — i.e. the
    ///   host loop or ANOTHER coroutine): SKIP (`false`). The step only fires on
    ///   its own thread; because a coroutine's stack is preserved across `yield`,
    ///   the `(thread, base_depth)` key stays valid across `yield`→`resume`
    ///   (採択B), so the step completes at the right line after the resume.
    /// - **Over**: stop when `depth < base_depth` (returned to the caller) OR
    ///   (`depth == base_depth` AND the line changed from `start_line`) — the
    ///   next line in the same frame or shallower; lines in DEEPER frames (the
    ///   called function's body) are skipped.
    /// - **In**: stop at the next stoppable line on this thread — a deeper frame
    ///   (entered a callee) OR a changed line in the same/shallower frame.
    /// - **Out**: stop when `depth < base_depth` (returned to the caller frame).
    ///
    /// `cur_thread` / `depth` are the values from
    /// [`current_thread_and_depth`](Self::current_thread_and_depth); `line` is
    /// the current source line.
    pub(super) fn step_should_stop(
        kind: StepKind,
        thread: ThreadId,
        base_depth: u32,
        start_line: u32,
        cur_thread: ThreadId,
        depth: u32,
        line: u32,
    ) -> bool {
        // thread 不一致（ホストループ・別コルーチン）の行はスキップして継続。
        if cur_thread != thread {
            return false;
        }
        match kind {
            StepKind::Over => {
                depth < base_depth || (depth == base_depth && line != start_line)
            }
            StepKind::In => depth > base_depth || line != start_line,
            StepKind::Out => depth < base_depth,
        }
    }

    /// The `.pasta`-granular stop refinement layered on top of
    /// [`step_should_stop`](Self::step_should_stop) for [`SourceMode::Pasta`]
    /// (task 5.4 / requirements 9.1–9.5; design "PastaStepper" 549–556, Flow 4).
    ///
    /// This is ONLY consulted AFTER the existing `.lua`-granularity
    /// `step_should_stop` has already returned `true` for the current line — it
    /// can therefore only DEMOTE a `.lua` stop to "keep going", never invent a
    /// stop the `.lua` machinery would not have produced. The decision (design
    /// 549–556, reconciling branch 1 「異フレームは `.lua` 判定」 with 554/555
    /// 「step into/out は最初の `.pasta` 対応行で停止」 and 9.4 「未対応行は通過」):
    ///
    /// 1. **Current `.lua` line is `.pasta`-unmapped** (`cur_pasta == None`):
    ///    CONTINUE (`false`) — pass through unmapped lines, in ANY frame. This
    ///    realizes 9.4/E6 for the origin frame AND the "stop at the first
    ///    *mapped* `.pasta` line" of step into (9.2/E3 — skip unmapped callee
    ///    lines) and step out (9.3/E4 — skip unmapped caller lines). Sub-call /
    ///    recursion lines that reach here would only be in the origin frame,
    ///    because `step_should_stop` already excluded DEEPER frames for
    ///    `Over`/`Out` (E2/E5 are handled structurally by depth before this
    ///    function is consulted).
    /// 2. **Same frame as the step origin** (`cur_thread == thread` AND
    ///    `depth == base_depth`) **AND current `.pasta` == origin** (same FILE +
    ///    LINE): CONTINUE (`false`) — consume all `.lua` lines of the SAME
    ///    `.pasta` line in the origin frame (9.1/E1). The same-frame guard makes
    ///    step into "discard" the origin (design 554): a callee line is a
    ///    DIFFERENT frame, so even if it coincidentally maps to the origin's
    ///    `.pasta` line it is NOT consumed — it stops (step 3).
    /// 3. **Otherwise** — a mapped line that is either a DIFFERENT `.pasta` line
    ///    in the origin frame (9.1 next line) OR any mapped line in a different
    ///    frame (step into callee / step out caller — 9.2/9.3): STOP (`true`).
    ///
    /// `cur_thread`/`depth` come from
    /// [`current_thread_and_depth`](Self::current_thread_and_depth). `cur_pasta`
    /// is `resolve_lua_to_pasta(RAW source, line)` for the current line (the map
    /// canonicalizes the chunk internally — task 3.4 — so the RAW hook source is
    /// passed straight through). `origin_pasta` is the position captured when the
    /// step began (`None` if the start line was itself unmapped, in which case a
    /// mapped current line is a genuine `.pasta` transition and stops — step 3).
    pub(super) fn pasta_step_should_stop(
        thread: ThreadId,
        base_depth: u32,
        origin_pasta: Option<&PastaPos>,
        cur_thread: ThreadId,
        depth: u32,
        cur_pasta: Option<&PastaPos>,
    ) -> bool {
        // (1) 現 `.lua` 行が `.pasta` 未対応 → 通過（継続・9.4/E6）。フレームに依らず
        //     未対応行は飛ばし、step into/out は最初の「対応行」で止める（9.2/9.3）。
        let Some(cur) = cur_pasta else {
            return false;
        };
        // (2) 同一起点フレーム かつ 現 `.pasta` 位置が起点と同一（同 file・同 line）→
        //     同一 `.pasta` 行を消化（継続・9.1/E1）。同フレーム条件により step into は
        //     起点 `.pasta` を破棄（呼び出し先は別フレームなので消化対象にしない）。
        let same_frame = cur_thread == thread && depth == base_depth;
        if same_frame && origin_pasta == Some(cur) {
            return false;
        }
        // (3) それ以外（同フレームで異なる `.pasta` 対応行、または別フレームの対応行）
        //     → 停止（9.1 次行 / 9.2 step into / 9.3 step out）。
        true
    }
}
