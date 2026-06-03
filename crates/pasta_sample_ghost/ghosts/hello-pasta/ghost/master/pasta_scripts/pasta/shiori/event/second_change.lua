---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 仮想イベントディスパッチャを呼び出し、結果（thread|nil）をそのまま返す。
--- EVENT.fireがthreadをresumeし、状態管理とレスポンス生成を行う。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local CALLBACK = require("pasta.shiori.event.callback")

---OnSecondChange デフォルトハンドラ
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return string|thread|nil タイムアウト時は500レスポンス文字列、それ以外はシーンコルーチンまたはnil
REG.OnSecondChange = function(act)
    -- コールバックのタイムアウトsweepを先に実行
    local timeout_response = CALLBACK.sweep(os.time())
    if timeout_response then
        return timeout_response
    end
    -- dispatcher.dispatch()からthread|nilを受け取り、そのまま返す
    -- EVENT.fireがresumeとレスポンス生成を担当
    return dispatcher.dispatch(act)
end

return REG
