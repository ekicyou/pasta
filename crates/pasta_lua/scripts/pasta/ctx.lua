--- @module pasta.ctx
--- 環境コンテキストモジュール
---
--- セッション管理とコルーチン制御を担当する。
--- save（永続変数）とactors（登録アクター）を保持する。

local ACT = require("pasta.act")

--- @class CTX 環境オブジェクト
--- @field save table 永続変数（セッションが終わっても残る）
--- @field actors table<string, Actor> 登録アクター（名前→アクター）
local CTX = {}

--- CTX実装メタテーブル
local CTX_IMPL = {}
CTX_IMPL.__index = CTX_IMPL

--- 新規CTXを作成
--- 注意: pasta.saveのrequireはここで遅延実行される。
--- ctx.luaの読み込み時点ではloaderの初期化が完了していない可能性があるため。
--- @param actors table|nil 登録アクター
--- @return CTX 環境オブジェクト
function CTX.new(actors)
    local obj = {
        save = require("pasta.save"),
        actors = actors or {},
    }
    return setmetatable(obj, CTX_IMPL)
end

--- コルーチンでアクションを実行する
--- @param self CTX 環境オブジェクト
--- @param scene function シーン関数（第1引数にactを受け取る）
--- @param ... any 追加引数
--- @return thread コルーチン
function CTX_IMPL.co_action(self, scene, ...)
    local args = { ... }
    return coroutine.create(function()
        local act = ACT.new(self)
        scene(act, table.unpack(args))
        if #act.token > 0 then
            self:end_action(act)
        end
    end)
end

--- アクション開始
--- @param self CTX 環境オブジェクト
--- @return Act アクションオブジェクト
function CTX_IMPL.start_action(self)
    return ACT.new(self)
end

--- yieldでトークンを出力
--- @param self CTX 環境オブジェクト
--- @param act Act アクションオブジェクト
--- @return nil
function CTX_IMPL.yield(self, act)
    local token = act.token
    act.token = {}
    act.now_actor = nil
    coroutine.yield({ type = "yield", token = token })
end

--- アクション終了
--- @param self CTX 環境オブジェクト
--- @param act Act アクションオブジェクト
--- @return nil
function CTX_IMPL.end_action(self, act)
    local token = act.token
    act.token = {}
    coroutine.yield({ type = "end_action", token = token })
end

return CTX
