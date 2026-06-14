//! The DAP-presentation source-resolver seam (design "SourceMapSeam", R4.3):
//! [`ResolvedSource`], the [`SourceResolver`] alias, and the
//! [`default_source_resolver`] / [`pasta_source_resolver`] constructors.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::debug::source_map::SourceMap;

/// The DAP `source` presentation for one frame: the `source` JSON object and
/// the (possibly remapped) line to report (design "SourceMapSeam", R4.3).
///
/// A [`SourceResolver`] returns this for a frame's `(lua_source, lua_line)`. The
/// [default resolver](default_source_resolver) returns the generated `.lua`
/// unchanged (`{ "path": <lua source> }`, `line = lua_line`); a future
/// `pasta-source-map` resolver returns a `.pasta` path and the mapped `.pasta`
/// line instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSource {
    /// The DAP `source` object to embed in the stack frame (e.g.
    /// `{ "path": "@scene.lua" }` or `{ "path": "scene.pasta" }`).
    pub source: Value,
    /// The line to report for the frame, after any source mapping.
    pub line: u32,
}

/// The DAP-presentation source seam (design "SourceMapSeam", R4.3).
///
/// Maps a frame's generated `(lua_source, lua_line)` to the DAP [`ResolvedSource`]
/// to present. Installed on a [`DapAdapter`] via
/// [`set_source_resolver`](DapAdapter::set_source_resolver); the default is
/// [`default_source_resolver`] (presents the generated `.lua` unchanged, R4.3
/// "既定の提示は生成 .lua").
///
/// This is the DAP-PRESENTATION seam and is deliberately independent of the
/// code_gen producer seam (`SourceMapSink`). The `pasta-source-map` spec connects
/// the two: it builds a [`SourceMap`](crate::debug::source_map::SourceMap) via the
/// producer seam and installs a resolver here (see
/// [`pasta_source_resolver`]) that consults that map to present `.pasta`
/// paths/lines. No `.pasta` mapping is implemented in this layer — only the
/// swappable口.
pub type SourceResolver = Box<dyn Fn(&str, u32) -> ResolvedSource + Send>;

/// The default [`SourceResolver`]: present the generated `.lua` unchanged.
///
/// Returns `{ "path": <lua_source> }` with `line = lua_line`, byte-equivalent to
/// task 3.2's hard-coded behavior (R4.3 "本仕様の既定提示は生成 .lua").
pub fn default_source_resolver() -> SourceResolver {
    Box::new(|lua_source: &str, lua_line: u32| ResolvedSource {
        source: json!({ "path": lua_source }),
        line: lua_line,
    })
}

/// `.pasta` 提示用の [`SourceResolver`]（task 5.2・R5.1/R5.2/R5.3/R6.2/R3.3）。
///
/// 各フレームの生成 `(lua_source, lua_line)` を、集約 [`SourceMap`] の
/// [`resolve_lua_to_pasta`](SourceMap::resolve_lua_to_pasta) で `.pasta` 位置へ
/// 写像する（R3.3 双方向変換はマップ経由）:
///
/// - `Some(pos)`: `.pasta` `{ path: pos.file, line: pos.line }` を提示する
///   （停止位置・各コールスタックフレーム＝R5.1/R5.2）。
/// - `None`: 既定 `.lua` resolver（[`default_source_resolver`]）へ委譲し、生成
///   `.lua` を **判別可能**に提示する（対応なしフォールバック＝R5.3）。誤った
///   `.pasta` 対応づけ（mismap）は決して行わない（design "Error Handling"
///   610/617・整合性エラーも `.lua` フォールバック）。
///
/// `lua_source` は **フック報告の生 chunk 名**（`@<絶対 .lua パス>` 想定）であり、
/// [`SourceMap::resolve_lua_to_pasta`] が内部で
/// [`canonicalize_chunk_name`](crate::debug::source_map) による正規化を行う
/// （task 3.4）。したがって本 resolver は `lua_source` を **そのまま**渡す
/// （二重正規化しない）。
///
/// この resolver は提示モード `Pasta`（既定）時に
/// [`DapAdapter::set_source_resolver`] で装着する。`Lua` 時やマップ未提供時は
/// 既定 `.lua` resolver のままにする（R6.2・7.2 ゼロ劣化）— 装着判断は wiring 側
/// の `pasta_active()` ゲートが担う。
pub fn pasta_source_resolver(map: Arc<SourceMap>) -> SourceResolver {
    Box::new(move |lua_source: &str, lua_line: u32| {
        match map.resolve_lua_to_pasta(lua_source, lua_line) {
            // R5.1 / R5.2: 対応ありフレームは `.pasta` `{path, line}` を提示。
            Some(pos) => ResolvedSource {
                source: json!({ "path": pos.file }),
                line: pos.line,
            },
            // R5.3: 対応なし（行ミス／chunk ミス）は既定 `.lua` resolver と同一の
            // 提示へフォールバック（判別可能・誤マッピング禁止）。
            None => default_source_resolver()(lua_source, lua_line),
        }
    })
}
