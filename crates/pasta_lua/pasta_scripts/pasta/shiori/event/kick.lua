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

--- 保留キックシーンを当該 act 流用で解決しシーンコルーチンを返す
---
--- `STORE.kick_pending` を消費（クリア）したうえで、当該 OnSecondChange の `act` を
--- そのまま流用して `SCENE.co_exec(act, scene_name)` でシーンコルーチンを生成する。
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

    -- 3. 当該 act を流用してシーン名を解決し、シーンコルーチンを生成
    --    （遅延ロードで循環参照を回避）
    local SCENE = require("pasta.scene")
    local co = SCENE.co_exec(act, scene_name)

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
