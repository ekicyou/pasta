---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 仮想イベントディスパッチャを呼び出し、結果に応じてレスポンスを返す。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

---OnSecondChange デフォルトハンドラ
---@param req table SHIORI リクエストテーブル
---@return string SHIORI レスポンス
REG.OnSecondChange = function(req)
    local result = dispatcher.dispatch(req)

    if result then
        -- alpha01: シーン実行成功でも 204 を返す
        -- alpha03: さくらスクリプト変換・200 OK を返す
        return RES.no_content()
    end

    return RES.no_content()
end

return REG
