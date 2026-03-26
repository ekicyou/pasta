---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 仮想イベントディスパッチャを呼び出し、結果（thread|nil）をそのまま返す。
--- EVENT.fireがthreadをresumeし、状態管理とレスポンス生成を行う。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

---OnSecondChange デフォルトハンドラ
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return thread|nil シーンコルーチン、またはnil
REG.OnSecondChange = function(act)
    -- dispatcher.dispatch()からthread|nilを受け取り、そのまま返す
    -- EVENT.fireがresumeとレスポンス生成を担当
    return dispatcher.dispatch(act)
end

return REG
