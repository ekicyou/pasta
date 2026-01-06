--- @class Talk トーク組み立てオブジェクト
--- @field sakura_script string 現在構築中のスクリプト
local IMPL = {}

local MOD = {}

function IMPL.append_script(self, script)
    self.sakura_script = self.sakura_script .. script
end

---- トーク組み立てオブジェクトの新規作成
--- @return Talk トーク組み立てオブジェクト
function MOD.new()
    local talk = {}
    talk.sakura_script = ""
    setmetatable(talk, IMPL)
    return talk
end

return MOD
