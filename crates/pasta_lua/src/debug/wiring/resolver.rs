//! `.pasta` source presentation seam wiring: the DAP source RESOLVER attach
//! ([`attach_pasta_resolver`], task 5.2) and the `.pasta`→`.lua` `setBreakpoints`
//! TRANSLATION ([`translate_pasta_breakpoints`], task 5.3). Split out of the
//! `wiring` hub (C5 production split) — child of `wiring`, so it reaches the
//! hub's [`SourceMapWiring`]/[`SharedAdapter`] and the sibling submodules through
//! `super::`/`crate::debug::wiring::` (no visibility widening of the public
//! surface). Bodies are byte-identical to the flat `wiring.rs`.

use std::sync::Arc;

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::pasta_source_resolver;
use crate::debug::types::{Breakpoint, ResolvedBreakpoint, SourceRef};

use super::{SharedAdapter, SourceMapWiring};

/// Install the `.pasta` source RESOLVER on the shared [`DapAdapter`] when the
/// `.pasta` consumers should be active (task 5.2・design 509/582).
///
/// When `source_map.pasta_active()` (a map present AND
/// [`SourceMode::Pasta`](crate::debug::SourceMode), requirements 6.1), the
/// adapter's source seam is swapped to
/// [`pasta_source_resolver`](crate::debug::dap::pasta_source_resolver) so every
/// stack frame is presented in `.pasta` coordinates (R5.1/R5.2), with unmapped
/// frames falling back to the generated `.lua` (R5.3). Otherwise
/// (`SourceMode::Lua` or no map) the adapter keeps its default `.lua` resolver
/// untouched — byte-for-byte the existing behavior (requirements 6.2 / 7.2).
///
/// Called ONCE by [`run_socket_bridge`] before the inbound/outbound loop, so the
/// resolver is in place before any `stackTrace` is encoded. A poisoned adapter
/// lock is treated as "do not attach" (the bridge never panics); the default
/// `.lua` resolver then remains, which is the safe fallback.
///
/// It is also RE-RUN when a DAP `attach` `sourcePresentation` flips the effective
/// mode (task 5.5): because [`SourceMapWiring::pasta_active`] reads the SHARED
/// effective mode, this re-installs the `.pasta` resolver on a Lua→Pasta flip AND
/// resets to the default `.lua` resolver on a Pasta→Lua flip (so the resolver
/// presentation always matches the FINAL effective mode, requirement 6.3).
pub(super) fn attach_pasta_resolver(adapter: &SharedAdapter, source_map: &SourceMapWiring) {
    let resolver = if source_map.pasta_active() {
        // `pasta_active()` guarantees the map is `Some`; degrade to a no-op if it
        // is somehow absent (never panic in the bridge).
        match &source_map.source_map {
            Some(map) => pasta_source_resolver(Arc::clone(map)), // 5.1, 5.2, 5.3
            None => return,
        }
    } else {
        // Lua mode / no map → ensure the default `.lua` resolver (6.2/7.2). This
        // RESETS a previously-installed `.pasta` resolver on a Pasta→Lua `attach`
        // flip (task 5.5); on the first call (default adapter) it is a harmless
        // re-assert of the already-default resolver.
        crate::debug::dap::default_source_resolver()
    };
    if let Ok(mut dap) = adapter.lock() {
        dap.set_source_resolver(resolver);
    }
}

/// Translate a `.pasta`-source `setBreakpoints` into `.lua` execution-coordinate
/// registrations and build the DAP `setBreakpoints` response (task 5.3・design
/// "BpTranslator" 511-528・Flow 2 215-236・requirements 4.1 / 4.2 / 4.3 / 8.2).
///
/// Only called when [`SourceMapWiring::pasta_active`] (a map present AND
/// [`SourceMode::Pasta`](crate::debug::SourceMode)); the `.lua`/`Lua`/no-map path
/// keeps the existing direct [`BreakpointSet::set_breakpoints`] (requirements 6.2
/// / 7.2). For each requested `.pasta` `line`:
///
/// 1. `resolve_pasta_to_lua(pasta_path, line)` → all `(chunk, lua_line)` exec
///    coords. If non-empty, every coord is registered (4.1; one `.pasta` line may
///    expand to MANY `.lua` lines, 8.2) and the BP is reported `verified` at the
///    ORIGINAL `line`.
/// 2. No correspondence → `nearest_pasta_line_with_mapping(pasta_path, line)`
///    finds the nearest SUBSEQUENT mapped `.pasta` line; THAT line's `.lua` coords
///    are registered and the BP is reported `verified` at the ADJUSTED line (so
///    VSCode shows it moved, 4.3). NEVER mismaps.
/// 3. No nearest mapping at all → `verified: false` at the original line (4.3:
///    only adjust to a real subsequent line; otherwise leave unverified).
///
/// ALL execution coords across ALL requested lines are accumulated into a SINGLE
/// [`BreakpointSet::register`] call tagged with the `.pasta` present source, so
/// they replace this presented source's prior set atomically (per-present-source
/// authoritative; a `.pasta`-origin and a `.lua`-origin BP in the same chunk
/// never evict each other — requirements 4.4 / 8.2). The hook reports the RAW
/// `@<.lua path>` source; [`BreakpointSet::should_pause`] canonicalizes both the
/// hook source and these stored canonical chunks, so the registered `.pasta` BP
/// fires for the runtime coord (4.2).
pub(super) fn translate_pasta_breakpoints(
    breakpoints: &BreakpointSet,
    source_map: &SourceMapWiring,
    source: &SourceRef,
    lines: &[u32],
) -> Vec<ResolvedBreakpoint> {
    // `pasta_active()` guarantees the map is `Some`; degrade safely to the `.lua`
    // path if it is somehow absent (never panic in the bridge).
    let map = match &source_map.source_map {
        Some(map) => map,
        None => return breakpoints.set_breakpoints(source, lines),
    };
    let pasta_path = source.path.as_str();

    // Accumulate ALL execution coords for ALL requested lines into one register
    // call (replacing this present source's set atomically). One requested line
    // may yield many `(chunk, lua_line)` (8.2); a no-correspondence line is
    // adjusted to the nearest subsequent mapped `.pasta` line (4.3).
    let mut entries: Vec<Breakpoint> = Vec::new();
    let resolved: Vec<ResolvedBreakpoint> = lines
        .iter()
        .map(|&line| {
            // (1) Direct correspondence: register all `.lua` coords, verified at
            // the original line (4.1 / 8.2).
            let direct = map.resolve_pasta_to_lua(pasta_path, line);
            if !direct.is_empty() {
                for (chunk, lua_line) in direct {
                    entries.push(Breakpoint::new(pasta_path, chunk, lua_line));
                }
                return ResolvedBreakpoint {
                    source: source.clone(),
                    line,
                    verified: true,
                };
            }
            // (2) No correspondence: adjust to the nearest SUBSEQUENT mapped
            // `.pasta` line and register THAT line's coords, verified at the
            // adjusted line (4.3).
            if let Some(adjusted) = map.nearest_pasta_line_with_mapping(pasta_path, line) {
                for (chunk, lua_line) in map.resolve_pasta_to_lua(pasta_path, adjusted) {
                    entries.push(Breakpoint::new(pasta_path, chunk, lua_line));
                }
                return ResolvedBreakpoint {
                    source: source.clone(),
                    line: adjusted,
                    verified: true,
                };
            }
            // (3) No nearest mapping at all → unverified at the original line
            // (4.3: never mismap; only adjust to a real subsequent line).
            ResolvedBreakpoint {
                source: source.clone(),
                line,
                verified: false,
            }
        })
        .collect();

    // Replace this `.pasta` present source's prior set with the accumulated exec
    // coords (per-present-source authoritative; other sources preserved).
    breakpoints.register(pasta_path, entries);
    resolved
}
