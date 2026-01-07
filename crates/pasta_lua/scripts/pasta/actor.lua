--- @class Action アクションオブジェクト

--- @class Actor アクターオブジェクト
--- @field name string アクター名
--- @field spot integer 立ち位置（０以上）
--- @field action Action アクションオブジェクト
--- @field has_init_script boolean 初期化スクリプトが未実行ならtrue
--- @field [string] string 動的定義
local IMPL = {}

local MOD = {}

--- 立ち位置を設定する。
--- @param spot integer 立ち位置（０以上）
function IMPL.set_spot(self, spot)
    self.spot = spot
end

--- アクションの開始。アクションオブジェクトを設定し、現在のアクターをリセットする。
--- アクター初期化スクリプトを追加する。
--- @param action Action アクションオブジェクト
function IMPL.start_action(self, action)
    self.action = action
    action.now_actor = nil
    self.has_init_script = true
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

local spot_counter = 0

-- アクターオブジェクトの新規作成
-- @return Actor アクターオブジェクト
function MOD.new_actor(name)
    local actor = {}
    actor.name = name
    actor.spot = spot_counter
    spot_counter = spot_counter + 1
    setmetatable(actor, IMPL)
    return actor
end

local actors = {}

--- アクターオブジェクトを取得または新規作成する。
--- @param name string アクター名
--- @return Actor
function MOD.create_actor(name)
    if actors[name] then
        return actors[name]
    end
    local actor = MOD.new_actor(name)
    actors[name] = actor
    return actor
end

return MOD
