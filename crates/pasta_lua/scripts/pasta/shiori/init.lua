--- SHIORI.DLL用 パスタモジュール
local MOD = {}
local new_actor = require("pasta.shiori.actor")

local actors = {}

--- アクターオブジェクトを新規作成する。
--- @param name string アクター名
--- @return Actor アクターオブジェクト
function MOD.create_actor(name)
    local actor = new_actor(name)
    actors[name] = actor
    return actor
end

return MOD
