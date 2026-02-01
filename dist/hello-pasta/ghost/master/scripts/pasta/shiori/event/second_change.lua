---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 仮想イベントディスパッチャを呼び出し、結果に応じてレスポンスを返す。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

---OnSecondChange デフォルトハンドラ
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return string SHIORI レスポンス
REG.OnSecondChange = function(act)
    local result = dispatcher.dispatch(act)

    if result then
        -- alpha01: シーン実行成功でも 204 を返す
        -- alpha03: さくらスクリプト変換・200 OK を返す
        return RES.no_content()
    end

    return RES.no_content()
end

return REG
