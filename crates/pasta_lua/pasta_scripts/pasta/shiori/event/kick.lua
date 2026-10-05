--- @module pasta.shiori.event.kick
--- シーンキック入口モジュール（保留フラグ設置のみ・非ブロッキング）
---
--- VSCode 拡張からのシーンキック要求（`SHIORI.kick`）を受けて、
--- 次の OnSecondChange で強制再生するための保留フラグを設置する。
---
--- 本モジュールはフラグを立てるだけで、シーン解決・resume・レンダリングは
--- 一切行わない（GET をブロックしない・R3.1）。実行・継続・配信は次 tick の
--- 既存 OnSecondChange dispatch 機構が担う（`STORE.kick_force` は
--- virtual_dispatcher の dispatch 入口が、`STORE.kick_pending` は本モジュールの
--- `KICK.try_dispatch` が消費する）。
---
--- 即時単一モード（モードフラグを持たない・R5.4）。連続キックは
--- `STORE.kick_pending` を上書きし、最後のキックが次 tick で起動する。

local STORE = require("pasta.store")
local log = require("@pasta_log")

--- SHIORI リロード予約 sentinel（debug 限定・task 4.3 / R9.2）
---
--- bridge 側（`try_reload_shiori`）が既存 KickSink ただ一つにこの文字列を載せて
--- リロードを要求する。`try_dispatch` で **完全一致** 判定し（シーン表の引き当てより前）、
--- `act:raw_script("\\![reload,shiori]")` → `act:build()` を
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

--- シーン表から引き当てたシーン関数をコルーチン化する
---
--- `SCENE.co_exec`／`choice_select.create_scene_coroutine` と同一のラッパーパターン。
--- キックは `act:find_scene`（`SCENE.co_exec` 経由）を通らず、`SCENE.get`／
--- `SCENE.get_start` で得た関数を直接コルーチン化する。
---
--- @param fn function `SCENE.get`／`SCENE.get_start` で得たシーン関数
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

--- 保留キックシーンをシーン表から完全一致で引き当てシーンコルーチンを返す
---
--- `STORE.kick_pending` を消費（クリア）したうえで、シーン名をシーン表
--- （`STORE.scenes`）から完全一致で引き当ててコルーチンを生成する。`scene_name` の
--- 形式で 2 分岐する（composite-string 方式・debug 限定）:
---
--- - **先頭が `:`**（local-composite `:親:ローカル`）: `^:([^:]+):(.+)$` で親と
---   ローカルの登録名に分解し、`SCENE.get(親, ローカル)` で引く。
--- - **それ以外**（グローバルの登録名）: `SCENE.get_start(名前)` で開始関数を引く。
---
--- 得た値が関数なら `wrap_local_func` でコルーチン化し、関数でなければ解決不能とする。
--- `act:find_scene` の 5 段探索・`SCENE.search` の前方一致・`GLOBAL` の関数は通らず、
--- シーン表のシーンだけを再生する（`会話_1` のキックで `会話_10` を再生しない）。
---
--- 再生時の ctx 合成はキック専用構築をせず通常トーク再生と同一の手順による（R3.2）。
--- キック専用の出力キューは設けず、解決した co の継続は既存機構へ委譲する（R3.3）。
---
--- 解決不能シーンは co を据えず破棄し、診断ログを残す（前会話を保持・R3.5）。
--- いずれの場合も `kick_pending` を消費して再発火を防ぐ。
---
--- 注意: `set_co_scene` / resume / preempt は行わない（co を「返す」だけ）。
--- 据える・resume・force ゲートは呼び出し側（dispatch フック）の責務。
---
--- @param _act Act アクションオブジェクト（解決には使わない。シグネチャを保つ）
--- @return thread|nil シーンコルーチン、または nil（保留無し・解決不能）
function KICK.try_dispatch(_act)
    -- 1. 保留が無ければ何もしない（通常 dispatch へ素通り）
    local scene_name = STORE.kick_pending
    if scene_name == nil then
        return nil
    end

    -- 2. フラグ消費（成功・失敗いずれでも再発火させない）
    STORE.kick_pending = nil

    -- 2.5. SHIORI リロード sentinel（完全一致・task 4.3 / R9.2）。
    -- シーン表の引き当てより前で判定する。bridge 側
    -- `try_reload_shiori` が KickSink に載せた予約文字列に完全一致したら、
    -- `\![reload,shiori]` を出すコルーチンを返す（シーン解決は行わない）。
    if scene_name == RELOAD_SENTINEL then
        return wrap_reload_func()
    end

    -- 3. シーン表から完全一致で引き当て、シーンコルーチンを生成（遅延ロードで循環参照を回避）
    local SCENE = require("pasta.scene")
    local co

    -- local-composite（先頭 `:` = `:親:ローカル`）か判定して分岐。
    -- グローバルの登録名は sanitize_name により `:` を含まないため衝突しない。
    -- 親は最初の `:` 〜次の `:` まで、ローカルは残り全部（`挨拶_1` 等の `_` を含む）。
    local parent, local_name = string.match(scene_name, "^:([^:]+):(.+)$")
    local fn
    if parent ~= nil then
        fn = SCENE.get(parent, local_name)
    else
        fn = SCENE.get_start(scene_name)
    end
    if type(fn) == "function" then
        co = wrap_local_func(fn)
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
