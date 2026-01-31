---@module pasta.shiori.event.boot
--- Default implementation for OnBoot event.
---
--- This module provides a minimal default OnBoot handler that returns
--- a simple startup acknowledgment. Ghost developers can override this
--- by registering their own REG.OnBoot handler.

local REG = require "pasta.shiori.event.register"
local RES = require "pasta.shiori.res"

---Default OnBoot handler.
---Returns a simple 204 No Content response.
---Override this by setting REG.OnBoot to your own function.
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return string SHIORI response
REG.OnBoot = function(act)
    return RES.no_content()
end

return REG
