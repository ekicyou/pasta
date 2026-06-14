//! Task 4.2 — the gating that decides whether the I/O-side `.pasta` consumers
//! (resolver 5.2 / BP translation 5.3) are reached. `pasta_active()` is the
//! single gate: a map present AND `SourceMode::Pasta` (design 582). Otherwise
//! the bridge keeps its default `.lua` behavior (requirements 6.1/6.2/7.2).

use std::sync::Arc;

use crate::debug::{SharedSourceMode, SourceMode};
use crate::debug::source_map::SourceMap;

use super::SourceMapWiring;

/// 6.1 / design 582: a map present in `SourceMode::Pasta` activates the
/// `.pasta` consumers (`pasta_active() == true`).
#[test]
fn pasta_active_when_map_present_and_mode_pasta() {
    let wiring = SourceMapWiring {
        source_map: Some(Arc::new(SourceMap::new())),
        source_mode: SharedSourceMode::new(SourceMode::Pasta),
    };
    assert!(
        wiring.pasta_active(),
        "Some(map) + Pasta must activate the `.pasta` consumers (6.1)"
    );
}

/// 6.2 / 7.2: `SourceMode::Lua` keeps the default `.lua` behavior even if a map
/// were present (the gate is false).
#[test]
fn not_active_in_lua_mode_even_with_map() {
    let wiring = SourceMapWiring {
        source_map: Some(Arc::new(SourceMap::new())),
        source_mode: SharedSourceMode::new(SourceMode::Lua),
    };
    assert!(
        !wiring.pasta_active(),
        "`.lua` mode must NOT activate `.pasta` consumers (6.2)"
    );
}

/// 7.2: with no map the gate is false regardless of mode — the existing call
/// sites (all pass `None`) behave exactly as today.
#[test]
fn not_active_without_map() {
    assert!(!SourceMapWiring::disabled().pasta_active());
    let pasta_no_map = SourceMapWiring {
        source_map: None,
        source_mode: SharedSourceMode::new(SourceMode::Pasta),
    };
    assert!(
        !pasta_no_map.pasta_active(),
        "no map → default `.lua` behavior even in Pasta mode (7.2)"
    );
}
