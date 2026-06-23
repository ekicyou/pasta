--- @module pasta.shiori.event.kick
--- シーンキック入口モジュール（保留フラグ設置のみ・非ブロッキング）
---
--- VSCode 拡張からのシーンキック要求（`SHIORI.kick`）を受けて、
--- 次の OnSecondChange で強制再生するための保留フラグを設置する。
---
--- 本モジュールはフラグを立てるだけで、シーン解決・resume・レンダリングは
--- 一切行わない（GET をブロックしない・R3.1）。実行・継続・配信は次 tick の
--- 既存 OnSecondChange dispatch 機構が担う（kick の消費は別タスク）。
---
--- 即時単一モード（モードフラグを持たない・R5.4）。連続キックは
--- `STORE.kick_pending` を上書きし、最後のキックが次 tick で起動する。

local STORE = require("pasta.store")
local log = require("@pasta_log")

--- SHIORI リロード予約 sentinel（debug 限定・task 4.3 / R9.2）
---
--- bridge 側（`try_reload_shiori`）が既存 KickSink ただ一つにこの文字列を載せて
--- リロードを要求する。`try_dispatch` で **完全一致** 判定し（`:` local-composite 分岐や
--- global co_exec より前）、`act:raw_script("\\![reload,shiori]")` → `act:build()` を
--- 行うコルーチンを返す。
---
--- sentinel は実シーン名・local-composite と衝突しない:
---   global 実名は sanitize_name 済み（英数字/`_` のみ）、local は先頭 `:`。
---   本値は `@` と `/`（sanitize 対象）を含み先頭 `:` でもないため、完全一致は安全。
---
--- 重要: この文字列は Rust 側 `crate::debug::dap::RELOAD_SENTINEL`
--- （`crates/pasta_lua/src/debug/dap/decode.rs`）とバイト一致させること。
--- 片方だけ変更するとリロード配送が壊れる。
local RELOAD_SENTINEL = "@@pasta/reloadShiori@@"

--- @class KICK
local KICK = {}

--- キック保留フラグを設置する（非ブロッキング）
---
--- `STORE.kick_pending`（保留シーン名）と `STORE.kick_force`（割り込み許可）を
--- 立てるのみ。シーン解決・resume・レンダリングは行わず、進行中シーン状態
--- （`STORE.co_scene`）も変更しない。
---
--- @param scene_name string キック対象のシーン名
--- @return nil
function KICK.install(scene_name)
    STORE.kick_pending = scene_name
    STORE.kick_force = true
end

--- `SCENE.search` の結果関数をコルーチン化する（local-composite kick 専用）
---
--- `SCENE.co_exec`／`choice_select.create_scene_coroutine` と同一のラッパーパターン。
--- `co_exec`/`find_scene` は `global_scene_name`（parent）を透過しないため、local を
--- 狙う場合は `SCENE.search(local, parent)`（local 分岐 → `resolve_scene_id_unified`）
--- で得た func を直接コルーチン化する必要がある。
---
--- @param fn function `SCENE.search` 結果の `.func`
--- @return thread シーンコルーチン（act は初回 resume 時に渡される）
local function wrap_local_func(fn)
    -- 引数名は coroutine.resume 経由で渡される act を受ける（co_exec の wrapped_fn を踏襲）
    local function wrapped_fn(resumed_act, ...)
        fn(resumed_act, ...)
        local result = resumed_act:build()
        if result ~= nil then
            return result
        end
    end
    return coroutine.create(wrapped_fn)
end

--- SHIORI リロード用さくらスクリプトを出すコルーチンを生成する（reload sentinel 専用）
---
--- `wrap_local_func`／`SCENE.co_exec` の `wrapped_fn` と同一のラッパーパターン。
--- 初回 resume で渡される act に `\![reload,shiori]`（SHIORI のみ再読み込み・非同期）を
--- raw_script として積み、`build()` 結果（nil でなければ）を返す（R9.2）。
---
--- @return thread リロードさくらスクリプトを出すコルーチン
local function wrap_reload_func()
    -- 引数名は coroutine.resume 経由で渡される act を受ける（co_exec の wrapped_fn を踏襲）
    local function wrapped_fn(resumed_act, ...)
        resumed_act:raw_script("\\![reload,shiori]")
        local result = resumed_act:build()
        if result ~= nil then
            return result
        end
    end
    return coroutine.create(wrapped_fn)
end

--- 保留キックシーンを当該 act 流用で解決しシーンコルーチンを返す
---
--- `STORE.kick_pending` を消費（クリア）したうえで、シーン名を解決してコルーチンを
--- 生成する。`scene_name` の形式で 2 分岐する（composite-string 方式・debug 限定）:
---
--- - **先頭が `:`**（local-composite `:parent:local`）: resolver が確定した
---   `(scene_id, parent)` を載せた形式。`^:([^:]+):(.+)$` で parent と local_name に
---   分解し、`SCENE.search(local_name, parent)` の **local 分岐**で func を取得して
---   コルーチン化する。global 分岐（parent=nil）が local_name を捨て `__start__` へ
---   潰す挙動を回避し、local の同一性を保持する（design「local シーン kick の方式決定」）。
--- - **それ以外**（global・従来通り）: 当該 act を流用して `SCENE.co_exec(act, scene)`
---   でシーンコルーチンを生成する（挙動不変）。
---
--- ctx 合成はキック専用構築をせず通常トーク再生と同一の合成手順を流用する（R3.2）。
--- キック専用の出力キューは設けず、解決した co の継続は既存機構へ委譲する（R3.3）。
---
--- 解決不能シーンは co を据えず破棄し、診断ログを残す（前会話を保持・R3.5）。
--- いずれの場合も `kick_pending` を消費して再発火を防ぐ。
---
--- 注意: `set_co_scene` / resume / preempt は行わない（co を「返す」だけ）。
--- 据える・resume・force ゲートは呼び出し側（dispatch フック）の責務。
---
--- @param act Act アクションオブジェクト（find_scene による名前解決に流用）
--- @return thread|nil シーンコルーチン、または nil（保留無し・解決不能）
function KICK.try_dispatch(act)
    -- 1. 保留が無ければ何もしない（通常 dispatch へ素通り）
    local scene_name = STORE.kick_pending
    if scene_name == nil then
        return nil
    end

    -- 2. フラグ消費（成功・失敗いずれでも再発火させない）
    STORE.kick_pending = nil

    -- 2.5. SHIORI リロード sentinel（完全一致・task 4.3 / R9.2）。
    -- `:` local-composite 分岐や global co_exec より前で判定する。bridge 側
    -- `try_reload_shiori` が KickSink に載せた予約文字列に完全一致したら、
    -- `\![reload,shiori]` を出すコルーチンを返す（シーン解決は行わない）。
    if scene_name == RELOAD_SENTINEL then
        return wrap_reload_func()
    end

    -- 3. シーン名を解決し、シーンコルーチンを生成（遅延ロードで循環参照を回避）
    local SCENE = require("pasta.scene")
    local co

    -- local-composite（先頭 `:` = `:parent:local`）か判定して分岐。
    -- global 実名は sanitize_name により `:` を含まないため衝突しない。
    -- parent は最初の `:` 〜次の `:` まで、local_name は残り全部（`挨拶_1` 等の `_` を含む）。
    local parent, local_name = string.match(scene_name, "^:([^:]+):(.+)$")
    if parent ~= nil then
        -- local 分岐: SCENE.search(local, parent) で local 同一性を保持して解決
        local result = SCENE.search(local_name, parent)
        if result and type(result.func) == "function" then
            co = wrap_local_func(result.func)
        end
    else
        -- global 分岐: 従来通り当該 act を流用（挙動不変）
        co = SCENE.co_exec(act, scene_name)
    end

    -- 4. 解決不能: co を据えず破棄＋診断ログ（前会話は保持）
    if co == nil then
        log.warn(string.format(
            "seam=kick.unresolved scene=%s: kick scene unresolved, dropped",
            tostring(scene_name)))
        return nil
    end

    -- 5. 解決成功: シーンコルーチンを返す（据える・resume は呼び出し側）
    return co
end

return KICK
