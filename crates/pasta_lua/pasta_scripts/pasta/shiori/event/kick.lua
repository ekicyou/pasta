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

return KICK
