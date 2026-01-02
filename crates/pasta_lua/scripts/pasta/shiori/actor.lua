--- @class Action アクションオブジェクト
--- @field now_actor Actor 現在のアクターオブジェクト


--- @class Actor アクターオブジェクト
--- @field action Action アクションオブジェクト
local IMPL = {}


--- アクションオブジェクトを設定し、現在のアクターをリセットする。
--- アクター初期化スクリプトを追加する。
--- @param action Action アクションオブジェクト
--- @param scope integer 立ち位置（０以上）
function IMPL.set_action(self, action, scope)
    self.action = action
    self.scope = scope or 0
    action.now_actor = nil
    self:set_init_script()
end

function IMPL.set_init_script(self)
    self.init_script = true
end

function IMPL.change_actor(self)
    if self.now_actor == self then
        return
    end
    self.now_actor = self
end

function IMPL.talk(self, text)
    self:change_actor()
end

function IMPL.word(self, key)

end

-- アクターオブジェクトの新規作成
-- @return table アクターオブジェクト
local function new()
    local actor = {}
    setmetatable(actor, IMPL)
    return actor
end

return new
