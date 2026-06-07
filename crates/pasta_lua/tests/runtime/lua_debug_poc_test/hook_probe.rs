//! HookProbe: jit.off + set_global_hook の設置、pasta 風コルーチン駆動シナリオ、
//! 行発火の記録（R1 検証 / go-no-go 本丸）。
//!
//! 本モジュールは task 2.1 を実装する。兄弟モジュール `harness_types`
//! （`build_jit_off_vm()` / `LineEvent`）を再利用する。`HookStrategy` は
//! ここで定義する（hook_probe の所有）。
//!
//! ## R1 の本丸（design "Open Questions / Risks #1" / research D1）
//! `mlua::Lua::set_global_hook` が **Lua 側 `coroutine.create` 由来の動的生成
//! コルーチン群**でラインフックを撃つか。撃たない場合は D1 フォールバック
//! （Lua 側 `coroutine.create` を Rust 製の `lua.create_thread` + `Thread::set_hook`
//! へ差し替え、生成毎にフックを付与）を試行し、成立した方式を `HookStrategy`
//! として記録する（Req 1.5）。
//!
//! ## mlua 0.11.6 API（実機検証済み）
//! - `Lua::set_global_hook(triggers, |&Lua, &Debug| -> Result<VmState>) -> Result<()>`
//!   （`#[cfg(not(feature = "luau"))]`。workspace は luajit52 のため利用可）。
//! - `HookTriggers::EVERY_LINE`（associated const）。
//! - `VmState::Continue`（LuaJIT では `Yield` 不可。フックは常に `Continue` を返す）。
//! - `Debug::source() -> DebugSource { source, short_src, .. }`、
//!   `Debug::current_line() -> Option<usize>`、`Debug::names() -> DebugNames { name, .. }`。
//! - `Lua::create_thread(Function) -> Result<Thread>`、
//!   `Thread::set_hook(triggers, callback) -> Result<()>`、
//!   `Thread::state() -> *mut lua_State`（thread_ptr 取得に利用）。

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use mlua::{Debug, HookTriggers, Lua, Thread, VmState};

use super::harness_types::LineEvent;

/// 採用したフック方式（Req 1.5：成立した方式を記録）。
///
/// - `GlobalHook`: `Lua::set_global_hook` が Lua 側 `coroutine.create` 由来の
///   コルーチンを含め全コルーチンで発火した（LuaJIT は `lua_sethook` が
///   main state に対してグローバルに作用する＝issue #666 の挙動）。
/// - `PerCoroutineThreadHook`: グローバルフックがコルーチン横断で発火しないため、
///   D1 フォールバック（`coroutine.create` を Rust 製 `create_thread` +
///   `Thread::set_hook` に差し替え）で生成毎にフックを付与して成立させた。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HookStrategy {
    GlobalHook,
    PerCoroutineThreadHook,
}

/// `Debug` から `LineEvent` を構築する（フック内で呼ぶ）。
///
/// `thread_ptr` はベストエフォートのコルーチン識別子。mlua 0.11.6 のフック
/// コールバックには実行中コルーチンの生 `lua_State` が渡らず（`&Lua` は常に
/// main state を指す）、安全 API から走行スレッドを一意取得する経路が無い。
/// よって `thread_ptr` は `lua.current_thread().state()`（実質 main 固定の
/// ベストエフォート値）とする。**コルーチン横断発火の決定的証拠は
/// `thread_ptr` ではなく、各コルーチン本体に固有のソース名・行番号が
/// `LineEvent` として記録されること**（design "HookProbe Invariants" 注 +
/// task 2.1 指示）。
fn line_event_from_debug(lua: &Lua, debug: &Debug) -> LineEvent {
    let src = debug.source();
    // source（チャンク名）を優先し、無ければ short_src を使う。
    let source = src
        .source
        .as_ref()
        .map(|c| c.as_ref().to_string())
        .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
        .unwrap_or_default();
    let line = debug.current_line().unwrap_or(0) as u32;
    let thread_ptr = lua.current_thread().state() as usize;
    LineEvent {
        source,
        line,
        thread_ptr,
    }
}

/// jit.off を効かせた VM に line トリガのグローバルフックを設置し、各実行行で
/// `LineEvent` を `sink` に記録する。採用したフック方式を返す（Req 1.1, 1.4, 1.5）。
///
/// まず `Lua::set_global_hook(EVERY_LINE, ..)` を設置する（`std_debug` 非露出の
/// まま：Req 5.3）。LuaJIT では `lua_sethook` が main state に対してグローバルに
/// 作用するため、Lua 側 `coroutine.create` 由来コルーチンにも波及することが
/// 期待される（issue #666 / mlua `test_global_hook` の挙動）。
///
/// グローバルフックがコルーチン横断で発火するかは `run_scene_like_scenario` の
/// 実行結果で判定する。本関数は `GlobalHook` を第一候補として設置し、もし発火
/// しないことが観測された場合の D1 フォールバックは
/// [`install_per_coroutine_hook_fallback`] が担う。判定は呼び出し側テストが行う
/// （横断発火が観測できたら `GlobalHook` を確定）。
pub(crate) fn install_global_line_hook(
    lua: &Lua,
    sink: Arc<Mutex<Vec<LineEvent>>>,
) -> mlua::Result<HookStrategy> {
    let hook_sink = Arc::clone(&sink);
    lua.set_global_hook(HookTriggers::EVERY_LINE, move |lua, debug| {
        // LuaJIT では VmState::Yield 不可。常に Continue を返す。
        let ev = line_event_from_debug(lua, debug);
        if let Ok(mut guard) = hook_sink.lock() {
            guard.push(ev);
        }
        Ok(VmState::Continue)
    })?;
    Ok(HookStrategy::GlobalHook)
}

/// D1 フォールバック（Req 1.5）: Lua 側 `coroutine.create` を Rust 製
/// `lua.create_thread` + `Thread::set_hook` へ差し替えるラッパを globals に
/// インストールする。これにより、ラッパ経由で生成される各コルーチンへ
/// 個別にラインフックが付与される（グローバルフックがコルーチン横断で
/// 発火しない LuaJIT 構成への保険）。
///
/// ラッパは `coroutine.create` を上書きし、Rust 側コールバックを通じて
/// `create_thread` でスレッドを生成、即座に `Thread::set_hook` を張ってから
/// 返す。`coroutine.resume` / `coroutine.yield` 等の他 API は変更しない
/// （生成された thread は通常のコルーチンとして駆動できる）。
pub(crate) fn install_per_coroutine_hook_fallback(
    lua: &Lua,
    sink: Arc<Mutex<Vec<LineEvent>>>,
) -> mlua::Result<HookStrategy> {
    // Rust 製 coroutine.create 相当: Function を受け取り、フック付き Thread を返す。
    let create_with_hook = lua.create_function(move |lua, func: mlua::Function| {
        let thread: Thread = lua.create_thread(func)?;
        let hook_sink = Arc::clone(&sink);
        thread.set_hook(HookTriggers::EVERY_LINE, move |lua, debug| {
            let ev = line_event_from_debug(lua, debug);
            if let Ok(mut guard) = hook_sink.lock() {
                guard.push(ev);
            }
            Ok(VmState::Continue)
        })?;
        Ok(thread)
    })?;

    // coroutine.create を差し替え（生成毎にフック付与）。
    let coroutine: mlua::Table = lua.globals().get("coroutine")?;
    coroutine.set("create", create_with_hook)?;
    Ok(HookStrategy::PerCoroutineThreadHook)
}

/// pasta の実シーン実行モデル（`scene.lua:212` の `coroutine.create(wrapped_fn)`）
/// を忠実に模した N コルーチン生成＋resume 駆動シナリオ（Req 1.2, 1.3）。
///
/// pasta では「シーン 1 つ＝コルーチン 1 つを動的生成し、駆動ループ
/// （`EVENT.fire` / `resume_until_valid`）で resume する」。本関数は各コルーチンに
/// **固有のソース名（`@scene_co_<i>`）と固有の複数行本体**を与え、駆動ループで
/// 順次 resume する。固有ソース・行が `LineEvent` に現れることが、コルーチン
/// 横断発火の決定的証拠となる。
///
/// 各コルーチン本体は 3 つの実行行（途中に `coroutine.yield()` を挟む）を持ち、
/// 駆動ループは finished になるまで resume する。
pub(crate) fn run_scene_like_scenario(lua: &Lua, coroutine_count: usize) -> mlua::Result<()> {
    for i in 0..coroutine_count {
        // 各コルーチンに固有のチャンク名を付与（`@scene_co_<i>`）。
        // 本体は複数行＋途中 yield（pasta のシーン yield 駆動を模す）。
        let chunk_name = format!("@scene_co_{i}");
        let body = format!(
            "\
local marker = {i}
local acc = marker * 2
coroutine.yield()
acc = acc + marker
return acc
"
        );

        // scene.lua:212 と同様、wrapped_fn を coroutine.create に渡す。
        // wrapped_fn 内で本体関数を呼ぶ（pasta の `wrapped_fn` 構造を模写）。
        let scene_fn: mlua::Function = lua
            .load(&body)
            .set_name(&chunk_name)
            .into_function()?;

        // `coroutine.create(wrapped_fn)` 相当を Lua 側で実行する。
        // ここで使われる `coroutine.create` は、フォールバック適用時は
        // Rust 製差し替え版（フック付与）になる。
        let driver: mlua::Function = lua
            .load(
                "\
local scene_fn = ...
local co = coroutine.create(scene_fn)
-- 駆動ループ: finished になるまで resume（pasta の resume_until_valid を模す）
while coroutine.status(co) ~= 'dead' do
    local ok, err = coroutine.resume(co)
    if not ok then error(err) end
end
",
            )
            .set_name("@scene_driver")
            .into_function()?;

        driver.call::<()>(scene_fn)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_debug_poc_test::harness_types::build_jit_off_vm;

    /// 記録済み LineEvent から、指定ソースに属する行番号系列を抽出する。
    fn lines_for_source(events: &[LineEvent], source: &str) -> Vec<u32> {
        events
            .iter()
            .filter(|e| e.source == source)
            .map(|e| e.line)
            .collect()
    }

    /// 記録済み LineEvent に出現するソース名集合。
    fn sources(events: &[LineEvent]) -> std::collections::BTreeSet<String> {
        events.iter().map(|e| e.source.clone()).collect()
    }

    /// R1.1 / R1.4: jit.off 済み VM で line hook が **各行で**発火し、期待行系列に
    /// 一致する（取りこぼし無し）ことを実証する。これが R1.4「jit.off 後の
    /// 未発火行なし」の中核証拠。
    #[test]
    fn hook_fires_on_single_chunk() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let sink: Arc<Mutex<Vec<LineEvent>>> = Arc::new(Mutex::new(Vec::new()));

        let strategy = install_global_line_hook(&lua, Arc::clone(&sink))
            .expect("install_global_line_hook should succeed");
        assert_eq!(
            strategy,
            HookStrategy::GlobalHook,
            "single-chunk hook should install as GlobalHook"
        );

        // 既知の 3 行チャンク（1 行ずつ実行される）。固有チャンク名を付与。
        let chunk = "\
local x = 1
local y = x + 1
local z = y + 1
";
        lua.load(chunk)
            .set_name("@single_chunk")
            .exec()
            .expect("chunk should execute");

        // フック解除（以降の比較を安定させる）。
        lua.remove_global_hook();

        let events = sink.lock().unwrap();
        let lines = lines_for_source(&events, "@single_chunk");

        // 各行（1,2,3）が**順序通り・取りこぼし無し**で発火していること。
        // LuaJIT 2.1 では末尾に return 相当の追加行イベントが付くことがあるため、
        // 「期待行が欠落なく出現し、かつ単調」を厳密に検証する。
        assert!(
            !lines.is_empty(),
            "hook must fire on the single chunk (R1.1). recorded events: {:?}",
            *events
        );
        for expected in [1u32, 2, 3] {
            assert!(
                lines.contains(&expected),
                "expected line {expected} must fire with no missed lines (R1.4). got: {lines:?}"
            );
        }
        // 行 1→2→3 がこの相対順序で出現する（取りこぼし＝順序の飛びが無い）。
        let pos = |l: u32| lines.iter().position(|&x| x == l).unwrap();
        assert!(
            pos(1) < pos(2) && pos(2) < pos(3),
            "lines must fire in source order 1<2<3 (no missed lines, R1.4). got: {lines:?}"
        );
    }

    /// R1.2 / R1.3 / R1.5: フック設定後、`coroutine.create` で動的生成した複数
    /// コルーチン（scene.lua の co_exec を模写）を駆動したとき、**全コルーチン
    /// 本体の行**でフックが発火することを実証し、採用 `HookStrategy` を記録する。
    ///
    /// 決定的証拠: 記録 LineEvent に N 個すべての固有ソース `@scene_co_<i>` の
    /// 行が含まれること（thread_ptr ではなくソース・行内容で横断発火を判定）。
    #[test]
    fn hook_fires_across_dynamic_coroutines() {
        const N: usize = 3;

        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let sink: Arc<Mutex<Vec<LineEvent>>> = Arc::new(Mutex::new(Vec::new()));

        // 第一手: グローバルフック。
        let mut strategy = install_global_line_hook(&lua, Arc::clone(&sink))
            .expect("install_global_line_hook should succeed");

        run_scene_like_scenario(&lua, N).expect("scenario (global hook) should run");

        // 全コルーチン固有ソースが記録されたか判定。
        let crossing_ok = {
            let events = sink.lock().unwrap();
            (0..N).all(|i| {
                let name = format!("@scene_co_{i}");
                !lines_for_source(&events, &name).is_empty()
            })
        };

        if !crossing_ok {
            // D1 フォールバック: coroutine.create を Rust 製差し替え（生成毎フック）。
            // グローバルフックを外し、sink をクリアして再試行する。
            lua.remove_global_hook();
            sink.lock().unwrap().clear();

            strategy = install_per_coroutine_hook_fallback(&lua, Arc::clone(&sink))
                .expect("install_per_coroutine_hook_fallback should succeed");

            run_scene_like_scenario(&lua, N).expect("scenario (fallback) should run");
        } else {
            lua.remove_global_hook();
        }

        let events = sink.lock().unwrap();

        // R1.2: すべてのコルーチン本体の行でフックが発火していること。
        for i in 0..N {
            let name = format!("@scene_co_{i}");
            let lines = lines_for_source(&events, &name);
            assert!(
                !lines.is_empty(),
                "hook must fire inside coroutine {i} body ({name}) — strategy={strategy:?}. \
                 recorded sources: {:?}",
                sources(&events)
            );
            // 各コルーチン本体は marker / acc / (yield 後) acc / return を含む複数行。
            // 少なくとも yield をまたいだ前後（行 1〜2 と 4 以降）が記録されること
            // ＝コルーチン横断 resume でも取りこぼさない（R1.2）。
            assert!(
                lines.iter().any(|&l| l >= 4),
                "hook must keep firing after coroutine.yield/resume in {name} \
                 (post-yield lines) — strategy={strategy:?}. got: {lines:?}"
            );
        }

        // R1.5: 採用したフック方式を記録・出力（cargo test ログに残す）。
        println!(
            "[R1] HookStrategy adopted = {strategy:?} (coroutine-crossing line hook fired \
             across all {N} dynamically-created coroutines)"
        );

        // strategy は GlobalHook か PerCoroutineThreadHook のいずれか。
        assert!(
            matches!(
                strategy,
                HookStrategy::GlobalHook | HookStrategy::PerCoroutineThreadHook
            ),
            "an adopted HookStrategy must be recorded (R1.5)"
        );
    }
}
