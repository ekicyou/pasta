---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 先にコールバックのタイムアウト掃引（CALLBACK.sweep）を行い、応答が生まれた回は
--- その応答（200 または timeout_message の 500 の全文）を返して OnHour・OnTalk を発行しない。
--- 生まれなければ仮想イベントディスパッチャを呼び出し、結果（thread|nil）をそのまま返す。
--- EVENT.fire が thread を EVENT.drive で再開し、状態管理とレスポンス生成を行う。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local CALLBACK = require("pasta.shiori.event.callback")

---OnSecondChange デフォルトハンドラ
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return string|thread|nil 掃引が応答を生んだ回はその応答（200 または 500 の全文）、それ以外はシーンコルーチンまたはnil
REG.OnSecondChange = function(act)
    -- コールバックのタイムアウト掃引を先に実行（応答は SHIORI/ で始まるため EVENT.fire が素通しする）
    local timeout_response = CALLBACK.sweep(os.time())
    if timeout_response then
        return timeout_response
    end
    -- dispatcher.dispatch()からthread|nilを受け取り、そのまま返す
    -- EVENT.fire が EVENT.drive での再開とレスポンス生成を担当
    return dispatcher.dispatch(act)
end

return REG
