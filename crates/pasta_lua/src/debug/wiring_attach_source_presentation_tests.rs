//! Task 5.5 — the DAP `attach` `sourcePresentation` applied to the SESSION's
//! resolver presentation (task 5.2) via the shared effective mode
//! ([`SharedSourceMode`]). `handle_inbound`, on an `attach` whose `Decoded`
//! carries `attach_source_mode`, WRITES the shared mode and RE-RUNS
//! [`attach_pasta_resolver`] so the DAP source resolver matches the FINAL
//! effective mode (requirement 6.3 / design 581/586). A missing arg leaves the
//! resolved env > file > 既定 mode untouched.
//!
//! These tests drive the resolver-attachment + shared-mode decision DIRECTLY
//! (the unit-testable core of the integration); the VM-thread step granularity
//! half is covered in `session.rs` (`attach_*` tests via [`SharedSourceMode`]).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::debug::dap::DapAdapter;
use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
use crate::debug::types::{FrameInfo, SessionEvent};
use crate::debug::{SharedSourceMode, SourceMode};

use super::{SourceMapWiring, attach_pasta_resolver};

fn map_with(chunk: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
    let mut forward = BTreeMap::new();
    forward.insert(
        lua_line,
        PastaPos {
            file: file.to_string(),
            line: pasta_line,
        },
    );
    let mut sm = SourceMap::new();
    sm.insert_chunk(
        chunk.to_string(),
        file.to_string(),
        ChunkSourceMap::from_forward(forward),
    );
    sm
}

/// Top frame `source`/`line` after encoding ONE `stackTrace` (observes which
/// resolver is installed).
fn top_frame(
    adapter: &Arc<Mutex<DapAdapter>>,
    source: &str,
    line: u32,
) -> (serde_json::Value, u32) {
    let mut dap = adapter.lock().unwrap();
    dap.decode_request(&json!({
        "seq": 1, "type": "request", "command": "stackTrace",
        "arguments": { "threadId": 1 },
    }));
    let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
        source: source.to_string(),
        line,
        func_name: Some("f".to_string()),
    }]));
    let frame = &out[0]["body"]["stackFrames"][0];
    (
        frame["source"].clone(),
        frame["line"].as_u64().unwrap() as u32,
    )
}

/// Pasta-capable wiring (map present) whose EFFECTIVE mode starts at `start`.
fn wiring_with(map: SourceMap, start: SourceMode) -> SourceMapWiring {
    SourceMapWiring {
        source_map: Some(Arc::new(map)),
        source_mode: SharedSourceMode::new(start),
    }
}

/// Simulate the `handle_inbound` attach-apply step: write the shared mode and
/// re-run the resolver attachment (the exact two operations `handle_inbound`
/// performs when `Decoded.attach_source_mode` is `Some`).
fn apply_attach(adapter: &Arc<Mutex<DapAdapter>>, wiring: &SourceMapWiring, mode: SourceMode) {
    wiring.source_mode.set(mode);
    attach_pasta_resolver(adapter, wiring);
}

/// R6.3 / precedence attach > env > file: `attach sourcePresentation="lua"`
/// FORCES `.lua` presentation even when the server default/file is `.pasta`.
/// After the flip, the resolver is the default `.lua` (NOT `.pasta`).
#[test]
fn attach_lua_forces_lua_resolver_over_pasta_default() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    // Server default is Pasta (map present): the `.pasta` resolver is attached.
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Pasta,
    );
    attach_pasta_resolver(&adapter, &wiring);
    // Precondition: Pasta default presents `.pasta`.
    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(src, json!({ "path": "C:/proj/scene.pasta" }));
    assert_eq!(line, 3);

    // attach sourcePresentation="lua" → flip to Lua.
    apply_attach(&adapter, &wiring, SourceMode::Lua);

    // Now the SAME frame is presented as the generated `.lua` (NOT `.pasta`).
    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        src,
        json!({ "path": r"@C:\proj\cache\scene.lua" }),
        "attach `lua` must force `.lua` presentation over the Pasta default (R6.3)"
    );
    assert_eq!(line, 7);
    assert_eq!(wiring.source_mode.get(), SourceMode::Lua);
    assert!(
        !wiring.pasta_active(),
        "Lua effective mode → consumers inactive"
    );
}

/// R6.3: `attach sourcePresentation="pasta"` FORCES `.pasta` presentation even
/// when the server default/file is `.lua` (a map IS present). The `.pasta`
/// resolver is attached after the flip.
#[test]
fn attach_pasta_forces_pasta_resolver_over_lua_default() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    // Server default is Lua (map present but gated off): default `.lua`.
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Lua,
    );
    attach_pasta_resolver(&adapter, &wiring);
    // Precondition: Lua default presents the generated `.lua`.
    let (src, _line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(src, json!({ "path": r"@C:\proj\cache\scene.lua" }));

    // attach sourcePresentation="pasta" → flip to Pasta (map present).
    apply_attach(&adapter, &wiring, SourceMode::Pasta);

    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        src,
        json!({ "path": "C:/proj/scene.pasta" }),
        "attach `pasta` must force `.pasta` presentation over the Lua default (R6.3)"
    );
    assert_eq!(line, 3);
    assert!(
        wiring.pasta_active(),
        "Pasta effective mode + map → consumers active"
    );
}

/// design 581 (no client-default override): with NO attach `sourcePresentation`
/// the resolved env > file > 既定 mode stays in effect — the resolver is NOT
/// touched by the (absent) attach arg.
#[test]
fn no_attach_arg_keeps_resolved_mode() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    let wiring = wiring_with(
        map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3),
        SourceMode::Pasta,
    );
    attach_pasta_resolver(&adapter, &wiring);
    // No apply_attach call (Decoded.attach_source_mode == None → skipped).
    let (src, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        src,
        json!({ "path": "C:/proj/scene.pasta" }),
        "absent attach arg keeps the resolved Pasta mode (design 581)"
    );
    assert_eq!(line, 3);
    assert_eq!(wiring.source_mode.get(), SourceMode::Pasta);
}
