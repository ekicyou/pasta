//! Hand-written JSON parse/encode free-function helpers shared across the
//! `dap` submodules (design "依存最小"). These are `pub(super)` because they are
//! called from the sibling [`decode`](super::decode) / [`encode`](super::encode)
//! modules; free functions — unlike type-associated items — are NOT visible to
//! siblings unless re-exported, so the minimal `pub(super)` seam is required
//! (design C3 risk note, research §8). NONE of these are re-exported outside the
//! `dap` module.

use serde_json::{Value, json};

use crate::debug::SourceMode;
use crate::debug::types::{
    ResolvedBreakpoint, SourceRef, StopReason, ThreadInfo, Variable,
};

use super::ResolvedSource;

/// Strictly parse a `pasta/sourcePresentation` `mode` token into a
/// [`SourceMode`], for the runtime presentation toggle (requirement 1.1 / 1.2 /
/// 1.4).
///
/// ONLY the exact tokens `"pasta"` / `"lua"` (case-insensitive, surrounding
/// whitespace ignored — mirroring [`SourceMode::parse`]'s tokenizing convention)
/// yield `Some`; ANY other value yields `None`.
///
/// Unlike [`SourceMode::parse`], there is NO invalid-value fallback to the
/// default `Pasta`. That fallback is correct for `attach` (the author DID specify
/// a presentation, just wrongly) but WRONG for the runtime toggle: requirement
/// 1.4 mandates that an unrecognized mode value cause NO mode change, so an
/// invalid token must map to `None` ("keep current mode"), never silently to
/// `Pasta`.
pub(super) fn parse_source_mode_strict(raw: &str) -> Option<SourceMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "pasta" => Some(SourceMode::Pasta),
        "lua" => Some(SourceMode::Lua),
        _ => None,
    }
}

/// Map a [`SourceMode`] to its `pasta/sourcePresentation` wire token
/// (`"pasta"` / `"lua"`), used in both the acceptance response and the custom
/// event body (requirement 1.3 / 2.5 / 2.6).
pub(super) fn source_mode_str(mode: SourceMode) -> &'static str {
    match mode {
        SourceMode::Pasta => "pasta",
        SourceMode::Lua => "lua",
    }
}

/// Map a [`StopReason`] to its DAP `stopped` `reason` string.
pub(super) fn stop_reason_str(reason: StopReason) -> &'static str {
    match reason {
        StopReason::Breakpoint => "breakpoint",
        StopReason::Step => "step",
        StopReason::Entry => "entry",
        StopReason::Pause => "pause",
    }
}

/// Parse a `setBreakpoints` request's `arguments` into `(SourceRef, lines)`.
///
/// DAP shape: `{"source":{"path":".."},"breakpoints":[{"line":N},..]}`. Missing
/// pieces degrade gracefully: no source path → empty path; no breakpoints →
/// empty line set (which clears the source's breakpoints, per DAP semantics).
pub(super) fn parse_set_breakpoints(args: Option<&Value>) -> (SourceRef, Vec<u32>) {
    let path = args
        .and_then(|a| a.get("source"))
        .and_then(|s| s.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let lines = args
        .and_then(|a| a.get("breakpoints"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|bp| bp.get("line").and_then(Value::as_u64).map(|l| l as u32))
                .collect()
        })
        .unwrap_or_default();

    (SourceRef { path }, lines)
}

/// Encode resolved breakpoints into the `setBreakpoints` response `body`.
pub(super) fn encode_breakpoints(bps: &[ResolvedBreakpoint]) -> Vec<Value> {
    bps.iter()
        .map(|bp| {
            json!({
                "verified": bp.verified,
                "line": bp.line,
            })
        })
        .collect()
}

/// Encode threads into the `threads` response `body`.
pub(super) fn encode_threads(threads: &[ThreadInfo]) -> Vec<Value> {
    threads
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.name }))
        .collect()
}

/// Encode stack frames into the `stackTrace` response `body`.
///
/// `frame id = stack index` (0-based), per the documented numbering. The frame's
/// `source`/`line` are produced by `resolver` rather than hard-coded, so the
/// DAP-presentation seam is swappable (R4.3): the default resolver presents the
/// generated `.lua` (`{ "path": <source> }`, `line = FrameInfo.line`)
/// byte-equivalently to task 3.2, while a future `pasta-source-map` resolver can
/// substitute a `.pasta` path and the mapped `.pasta` line without changing the
/// frame shape.
pub(super) fn encode_frames(
    frames: &[crate::debug::types::FrameInfo],
    resolver: &(dyn Fn(&str, u32) -> ResolvedSource + Send),
) -> Vec<Value> {
    frames
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            let resolved = resolver(&f.source, f.line);
            json!({
                "id": idx as u32,
                "name": f.func_name.clone().unwrap_or_else(|| "?".to_string()),
                "source": resolved.source,
                "line": resolved.line,
                "column": 1,
            })
        })
        .collect()
}

/// Encode variables into the `variables` response `body`.
///
/// Maps [`Variable::repr`] → DAP `value` and [`Variable::type_name`] → DAP
/// `type`. Leaf variables report `variablesReference: 0` (no deep table
/// expansion — out of scope for this task).
pub(super) fn encode_variables(vars: &[Variable]) -> Vec<Value> {
    vars.iter()
        .map(|v| {
            json!({
                "name": v.name,
                "value": v.repr,
                "type": v.type_name,
                "variablesReference": 0,
            })
        })
        .collect()
}
