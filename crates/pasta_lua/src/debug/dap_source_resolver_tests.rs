//! Inline test cluster externalized from `dap.rs` (Task 2.2, pure
//! behavior-invariant move). Cluster: the DAP-presentation source-resolver
//! seam — the default/alternate resolver swap and the `pasta_source_resolver`
//! `.lua` -> `.pasta` mapping (with the local `map_with` fixture helper).
use super::*;
use super::dap_test_support::*;

use crate::debug::types::FrameInfo;

// --- source resolver seam (R4.3) ---------------------------------------

/// DEFAULT resolver: a `stackTrace` response presents each frame's `source`
/// as the generated `.lua` (path = `FrameInfo.source`, line = `FrameInfo.line`),
/// byte-equivalent to task 3.2 — R4.3 "既定の提示は生成 .lua".
#[test]
fn stack_trace_default_resolver_presents_generated_lua() {
    let mut dap = DapAdapter::new();
    dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));

    let out = dap.encode_event(SessionEvent::Stack(vec![
        FrameInfo {
            source: "@scene.lua".to_string(),
            line: 7,
            func_name: Some("talk".to_string()),
        },
        FrameInfo {
            source: "@scene.lua".to_string(),
            line: 2,
            func_name: None,
        },
    ]));
    let resp = &out[0];
    let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
    // Default presentation is the generated .lua, unchanged from 3.2.
    assert_eq!(frames[0]["source"], json!({ "path": "@scene.lua" }));
    assert_eq!(frames[0]["line"], 7);
    assert_eq!(frames[1]["source"], json!({ "path": "@scene.lua" }));
    assert_eq!(frames[1]["line"], 2);
}

/// SWAPPABLE: install an alternate resolver that maps any `.lua` source to a
/// `.pasta`-style source (and remaps the line); the same `SessionEvent::Stack`
/// now presents the `.pasta` source/line — proving the口 is genuinely
/// swappable (R4.3 "将来 .pasta パスを提示できる構造"). This stub stands in for
/// the future `pasta-source-map` resolver (wired via task 5.3 / downstream).
#[test]
fn stack_trace_alternate_resolver_presents_pasta() {
    let mut dap = DapAdapter::new();
    // A test stub resolver: every frame becomes foo.pasta with line+100.
    dap.set_source_resolver(Box::new(|_lua_source: &str, lua_line: u32| {
        ResolvedSource {
            source: json!({ "path": "foo.pasta" }),
            line: lua_line + 100,
        }
    }));

    dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));
    let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
        source: "@scene.lua".to_string(),
        line: 7,
        func_name: Some("talk".to_string()),
    }]));
    let resp = &out[0];
    let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
    // The seam is swapped: presentation is now the .pasta source + mapped line.
    assert_eq!(frames[0]["source"], json!({ "path": "foo.pasta" }));
    assert_eq!(frames[0]["line"], 107);
    // Other frame fields (id / name) are unaffected by the source seam.
    assert_eq!(frames[0]["id"], 0);
    assert_eq!(frames[0]["name"], "talk");
}

#[test]
fn set_breakpoints_with_no_breakpoints_clears_lines() {
    let mut dap = DapAdapter::new();
    let decoded = dap.decode_request(&request(
        7,
        "setBreakpoints",
        json!({ "source": { "path": "@s.lua" } }),
    ));
    assert_eq!(
        decoded.command,
        Some(SessionCommand::SetBreakpoints {
            source: SourceRef::new("@s.lua"),
            lines: vec![],
        }),
        "missing breakpoints array → empty (clears) line set"
    );
}

// --- pasta source resolver (task 5.2 — R5.1 / R5.2 / R5.3 / R6.2 / R3.3) ---

use std::sync::Arc;

use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};

/// 既知の `chunk → .pasta` 対応を 1 件持つ集約 `SourceMap` を構築する小ヘルパ。
///
/// `chunk_name`（生フック源相当）へ、最終 `.lua` 行 `lua_line` → `.pasta`
/// `{file, pasta_line}` の 1 対応を登録する。`resolve_lua_to_pasta` は chunk
/// 引数を内部で正規化する（task 3.4）ため、テストは生フック source 文字列を
/// そのまま渡す。
fn map_with(chunk_name: &str, lua_line: u32, file: &str, pasta_line: u32) -> SourceMap {
    let mut forward = std::collections::BTreeMap::new();
    forward.insert(
        lua_line,
        PastaPos {
            file: file.to_string(),
            line: pasta_line,
        },
    );
    let mut sm = SourceMap::new();
    sm.insert_chunk(
        chunk_name.to_string(),
        file.to_string(),
        ChunkSourceMap::from_forward(forward),
    );
    sm
}

/// R5.1 / R5.2 / R3.3: 対応のあるフレームは `.pasta` `{path, line}` を提示する。
/// resolver は **生フック source**（`@` 付き・区切り混在）をそのまま
/// `resolve_lua_to_pasta` へ渡し、内部正規化（task 3.4）で突合される。
#[test]
fn pasta_resolver_maps_frame_to_pasta_source_and_line() {
    // 格納時とは異なる等価形（`@` 付き・大小違い）の生フック source で引く。
    let raw_hook_source = r"@C:\proj\cache\scene.lua";
    let map = map_with("C:/proj/cache/scene.lua", 12, "C:/proj/scene.pasta", 7);
    let resolver = pasta_source_resolver(Arc::new(map));

    let resolved = resolver(raw_hook_source, 12);
    // 提示は `.pasta` ファイル・行（pos.file / pos.line）。
    assert_eq!(
        resolved.source,
        json!({ "path": "C:/proj/scene.pasta" }),
        "R5.1/R5.2: 対応ありフレームは `.pasta` パスを提示する"
    );
    assert_eq!(resolved.line, 7, "R5.1/R5.2: 提示行は `.pasta` 行 (pos.line)");
}

/// R5.3: 対応の無い `(source, line)` は既定 `.lua` resolver と **完全に同一**の
/// 提示（生成 `.lua` の `{path, line}`）へフォールバックし、誤った `.pasta`
/// 対応づけ（mismap）を行わない。判別可能（`.pasta` ではなく `.lua`）。
#[test]
fn pasta_resolver_falls_back_to_lua_for_unmapped() {
    // chunk は一致するが `.lua` 行 99 は未対応 → フォールバック。
    let map = map_with("C:/proj/cache/scene.lua", 12, "C:/proj/scene.pasta", 7);
    let resolver = pasta_source_resolver(Arc::new(map));

    let resolved = resolver(r"@C:\proj\cache\scene.lua", 99);
    let expected = default_source_resolver()(r"@C:\proj\cache\scene.lua", 99);
    assert_eq!(
        resolved, expected,
        "R5.3: 未対応行は既定 `.lua` resolver と同一の提示へフォールバックする"
    );
    // 念のため：誤った `.pasta` ではなく生成 `.lua` source を保持している。
    assert_eq!(resolved.source, json!({ "path": r"@C:\proj\cache\scene.lua" }));
    assert_eq!(resolved.line, 99);
}

/// R5.3（整合性エラー・design 610/617）: chunk 名がマップに無い場合も `.lua`
/// フォールバック（誤マッピング禁止）。
#[test]
fn pasta_resolver_falls_back_to_lua_for_unknown_chunk() {
    let map = map_with("C:/proj/cache/scene.lua", 12, "C:/proj/scene.pasta", 7);
    let resolver = pasta_source_resolver(Arc::new(map));

    let resolved = resolver("@C:/proj/cache/other.lua", 12);
    let expected = default_source_resolver()("@C:/proj/cache/other.lua", 12);
    assert_eq!(
        resolved, expected,
        "R5.3: 未知 chunk は `.lua` フォールバック（誤マッピング禁止）"
    );
}

/// R5.2: `pasta_source_resolver` を装着した `DapAdapter` で `stackTrace` を
/// エンコードすると、各フレームが個別に `.pasta`／`.lua` で提示される
/// （対応ありは `.pasta`、対応なしは `.lua` フォールバック）。
#[test]
fn stack_trace_with_pasta_resolver_presents_each_frame() {
    let map = map_with("C:/proj/cache/scene.lua", 7, "C:/proj/scene.pasta", 3);
    let mut dap = DapAdapter::new();
    dap.set_source_resolver(pasta_source_resolver(Arc::new(map)));

    dap.decode_request(&request(11, "stackTrace", json!({ "threadId": 1 })));
    let out = dap.encode_event(SessionEvent::Stack(vec![
        // 対応あり（`.lua` 7 → `.pasta` 3）。
        FrameInfo {
            source: r"@C:\proj\cache\scene.lua".to_string(),
            line: 7,
            func_name: Some("talk".to_string()),
        },
        // 対応なし（`.lua` 2 は未登録）→ `.lua` フォールバック。
        FrameInfo {
            source: r"@C:\proj\cache\scene.lua".to_string(),
            line: 2,
            func_name: None,
        },
    ]));
    let resp = &out[0];
    let frames = resp["body"]["stackFrames"].as_array().expect("stackFrames array");
    // フレーム 0: `.pasta` 提示（R5.2）。
    assert_eq!(frames[0]["source"], json!({ "path": "C:/proj/scene.pasta" }));
    assert_eq!(frames[0]["line"], 3);
    // フレーム 1: 対応なし → 生成 `.lua` 提示（R5.3 判別可能フォールバック）。
    assert_eq!(frames[1]["source"], json!({ "path": r"@C:\proj\cache\scene.lua" }));
    assert_eq!(frames[1]["line"], 2);
}
