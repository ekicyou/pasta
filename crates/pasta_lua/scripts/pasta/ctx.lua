--- @module pasta.ctx
--- 環境コンテキストモジュール
---
--- セッション管理とコルーチン制御を担当する。
--- save（永続変数）とactors（登録アクター）を保持する。

local ACT -- 前方宣言（循環参照回避）

--- @class CTX 環境オブジェクト
--- @field save table 永続変数（セッションが終わっても残る）
--- @field actors table<string, Actor> 登録アクター（名前→アクター）
local CTX = {}
CTX.__index = CTX

--- ACTモジュールを遅延ロード（循環参照回避）
--- @return table ACTモジュール
local function get_act()
    if not ACT then
        ACT = require("pasta.act")
    end
    return ACT
end

--- 新規CTXを作成
--- @param save table|nil 永続変数
--- @param actors table|nil 登録アクター
--- @return CTX 環境オブジェクト
function CTX.new(save, actors)
    local obj = {
        save = save or {},
        actors = actors or {},
    }
    setmetatable(obj, CTX)
    return obj
end

--- コルーチンでアクションを実行する
--- @param scene function シーン関数（第1引数にactを受け取る）
--- @return thread コルーチン
function CTX:co_action(scene, ...)
    local args = { ... }
    local act_mod = get_act()
    return coroutine.create(function()
        local act = act_mod.new(self)
        scene(act, table.unpack(args))
        if #act.token > 0 then
            self:end_action(act)
        end
    end)
end

--- アクション開始
--- @return Act アクションオブジェクト
function CTX:start_action()
    local act_mod = get_act()
    return act_mod.new(self)
end

--- yieldでトークンを出力
--- @param act Act アクションオブジェクト
function CTX:yield(act)
    local token = act.token
    act.token = {}
    act.now_actor = nil
    coroutine.yield({ type = "yield", token = token })
end

--- アクション終了
--- @param act Act アクションオブジェクト
function CTX:end_action(act)
    local token = act.token
    act.token = {}
    coroutine.yield({ type = "end_action", token = token })
end

return CTX
