--- @class Actor アクター
---
--- @class CTX 環境

--- @class Action アクション
--- @field ctx CTX 環境オブジェクト
--- @field spot_actors [Actor] 登場アクター
--- @field now_actor Actor 現在のアクター
--- @field token [table] 構築中のスクリプトトークン
local IMPL = {}

function IMPL.actor(self, actor)
    if self.now_actor == actor then
        return
    end
    table.insert(self.token, { type = "actor", actor = actor })
    self.now_actor = actor
end

function IMPL.talk(self, actor, text)
    self:actor(actor)
    table.insert(self.token, { type = "talk", text = text })
end

function IMPL.sakura_script(self, text)
    table.insert(self.token, { type = "sakura_script", text = text })
end

function IMPL.yield(self)
    table.insert(self.token, { type = "yield" })
    self.ctx:yield(self)
end

function IMPL.end_action(self)
    table.insert(self.token, { type = "end_action" })
    self.ctx:end_action(self)
end

local MOD = {}

--- アクションの新規作成
--- @param ctx CTX 環境オブジェクト
--- @return Action アクション
function MOD.new(ctx)
    local obj = {}
    obj.ctx = ctx
    obj.token = {}
    obj.now_actor = nil
    setmetatable(obj, IMPL)
    return obj
end

return MOD
