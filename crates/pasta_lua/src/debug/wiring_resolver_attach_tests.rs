//! Task 5.2 — the `.pasta` source RESOLVER attachment on the socket-bridge's
//! shared [`DapAdapter`]. [`attach_pasta_resolver`] installs
//! [`pasta_source_resolver`](crate::debug::dap::pasta_source_resolver) IFF
//! `pasta_active()` (a map present AND `SourceMode::Pasta`, design 509/582);
//! otherwise the adapter keeps its default `.lua` resolver (R6.2 / 7.2).
use super::*;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::debug::{SharedSourceMode, SourceMode};
use crate::debug::dap::DapAdapter;
use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
use crate::debug::types::{FrameInfo, SessionEvent};

use super::{SourceMapWiring, attach_pasta_resolver};

/// 既知 `chunk(.lua line) → .pasta` 対応を 1 件持つ集約 `SourceMap`。
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

/// `stackTrace` を 1 回エンコードし、top フレームの `source` / `line` を返す
/// 小ヘルパ（装着結果の提示を観測する）。
fn top_frame(adapter: &Arc<Mutex<DapAdapter>>, source: &str, line: u32) -> (serde_json::Value, u32) {
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
    (frame["source"].clone(), frame["line"].as_u64().unwrap() as u32)
}

/// R5.1/R5.2/6.1: map present + `SourceMode::Pasta` → resolver 装着。対応あり
/// フレームが `.pasta` 提示になる。
#[test]
fn attaches_pasta_resolver_when_active() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    let wiring = SourceMapWiring {
        source_map: Some(Arc::new(map_with(
            "C:/proj/cache/scene.lua",
            7,
            "C:/proj/scene.pasta",
            3,
        ))),
        source_mode: SharedSourceMode::new(SourceMode::Pasta),
    };
    attach_pasta_resolver(&adapter, &wiring);

    let (source, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        source,
        json!({ "path": "C:/proj/scene.pasta" }),
        "active → 対応ありフレームは `.pasta` 提示 (R5.1/R5.2)"
    );
    assert_eq!(line, 3);
}

/// R5.3: 装着済みでも対応の無いフレームは `.lua` フォールバック（判別可能）。
#[test]
fn attached_resolver_falls_back_to_lua_for_unmapped() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    let wiring = SourceMapWiring {
        source_map: Some(Arc::new(map_with(
            "C:/proj/cache/scene.lua",
            7,
            "C:/proj/scene.pasta",
            3,
        ))),
        source_mode: SharedSourceMode::new(SourceMode::Pasta),
    };
    attach_pasta_resolver(&adapter, &wiring);

    // `.lua` 行 2 は未対応 → 生成 `.lua` 提示のまま。
    let (source, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 2);
    assert_eq!(source, json!({ "path": r"@C:\proj\cache\scene.lua" }));
    assert_eq!(line, 2);
}

/// R6.2 / 7.2: `SourceMode::Lua`（map あり）→ 非装着。既定 `.lua` resolver の
/// まま、生成 `.lua` 提示（バイト不変）。
#[test]
fn does_not_attach_in_lua_mode() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    let wiring = SourceMapWiring {
        source_map: Some(Arc::new(map_with(
            "C:/proj/cache/scene.lua",
            7,
            "C:/proj/scene.pasta",
            3,
        ))),
        source_mode: SharedSourceMode::new(SourceMode::Lua),
    };
    attach_pasta_resolver(&adapter, &wiring);

    // Lua モードでは対応があっても `.pasta` 化されない（既定 `.lua` のまま）。
    let (source, line) = top_frame(&adapter, r"@C:\proj\cache\scene.lua", 7);
    assert_eq!(
        source,
        json!({ "path": r"@C:\proj\cache\scene.lua" }),
        "R6.2: Lua モードは既定 `.lua` resolver のまま（非装着）"
    );
    assert_eq!(line, 7);
}

/// 7.2: map なし → 非装着（既存呼び出し全てが `None` を渡す経路と同一の挙動）。
#[test]
fn does_not_attach_without_map() {
    let adapter = Arc::new(Mutex::new(DapAdapter::new()));
    attach_pasta_resolver(&adapter, &SourceMapWiring::disabled());

    let (source, line) = top_frame(&adapter, "@scene.lua", 7);
    assert_eq!(source, json!({ "path": "@scene.lua" }), "no map → 既定 `.lua`");
    assert_eq!(line, 7);
}
