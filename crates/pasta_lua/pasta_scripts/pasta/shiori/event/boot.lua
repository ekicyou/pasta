---@module pasta.shiori.event.boot
--- Default implementation for OnBoot event.
---
--- This module provides a default OnBoot handler that looks up Pasta DSL scenes.
--- If a scene is found, it executes the scene and returns the SakuraScript.
--- If no scene is found, it returns 204 No Content.
--- Ghost developers can override this by registering their own REG.OnBoot handler.

local REG = require "pasta.shiori.event.register"
local SCENE = require "pasta.scene"

---Default OnBoot handler.
---Looks up Pasta DSL scenes first, falls back to 204 No Content.
---Override this by setting REG.OnBoot to your own function.
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return thread|nil シーンコルーチン、またはnil
REG.OnBoot = function(act)
    -- Try to find a Pasta DSL scene for OnBoot
    return SCENE.co_exec(act.req.id, nil, nil)
end

return REG
