local ACT = require("pasta.shiori.act")

--- @class CTX 環境オブジェクト
--- @field actors [Actor] 登場アクター
--- @field var table セッション変数（セッションが終わると消える）
--- @field save table 永続変数（セッションが終わっても残る）
--- @field act Action 現在のアクション
local IMPL = {}

--- コルーチンでアクションを実行する。
--- @param scene function シーン関数
function IMPL.co_action(self, scene, ...)
    local args = { ... }
    return coroutine.create(function()
        local act = self:start_action()
        scene(self, table.unpack(args))
        if #act.token > 0 then
            act:end_action()
        end
    end)
end

--- アクションを開始する。新しいアクションオブジェクトを作成し、現在のアクションに設定する。
--- @return Action アクションオブジェクト
function IMPL.start_action(self)
    local act = ACT.new(self)
    self.act = act
    return act
end

--- アクションを継続する。トークを区切り、次のタイミングで再開する。
--- @param act Action アクションオブジェクト
function IMPL.yield(self, act)
    local token = act.token
    act.token = {}
    act.now_actor = nil
    local mes = { type = "yield", token = token }
    coroutine.yield(mes)
end

--- アクションを終了する。
--- @param act Action アクションオブジェクト
function IMPL.end_action(self, act)
    local token = act.token
    act.token = {}
    local mes = { type = "end_action", token = token }
    coroutine.yield(mes)
    self.act = nil
end

local MOD = {}

--- 環境オブジェクトの新規作成
--- @param save table 永続変数（セッションが終わっても残る）
--- @param actors [Actor] 登場アクター
--- @return CTX 環境オブジェクト
function MOD.new(save, actors)
    local obj = {}
    obj.save = save or {}
    obj.actors = actors or {}
    obj.var = {}
    obj.act = nil
    setmetatable(obj, IMPL)
    return obj
end

return MOD
