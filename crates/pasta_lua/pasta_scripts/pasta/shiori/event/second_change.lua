---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 処理の順:
---   (1) コールバックのタイムアウト掃引（CALLBACK.sweep）。応答が生まれた回は
---       その応答（200 または timeout_message の 500 の全文）を返して OnHour・OnTalk を発行しない。
---   (2) 仮想イベントディスパッチャ。シーンのコルーチンが出ればそれを返す
---       （EVENT.fire が EVENT.drive で再開し、状態管理とレスポンス生成を行う）。
---   (3) ブロック対象の Status（talking 等）なら何も返さない（読み取りの予約は残る）。
---   (4) 時計に予約された仮想時刻の読み取りがあれば、そのスクリプトを返す。
---       文字列は EVENT.fire が RES.ok にし set_co_scene を通らないため、
---       待っているチェイントークの続き（STORE.co_scene）はそのまま残る。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local CALLBACK = require("pasta.shiori.event.callback")
local CLOCK = require("pasta.shiori.clock")

---OnSecondChange デフォルトハンドラ
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return string|thread|nil 掃引が応答を生んだ回はその応答（200 または 500 の全文）、シーンが出た回はコルーチン、読み取りの回はそのスクリプト、それ以外は nil
REG.OnSecondChange = REG.OnSecondChange or function(act)
    -- コールバックのタイムアウト掃引を先に実行（応答は SHIORI/ で始まるため EVENT.fire が素通しする）
    -- 今は なめらかな時計（get_property の締め切りと同じ物差し）。req.date が無ければ OS の時計
    local timeout_response = CALLBACK.sweep(CLOCK.steady(act.req) or os.time())
    if timeout_response then
        return timeout_response
    end
    local co = dispatcher.dispatch(act)
    if co then
        return co
    end
    if dispatcher.is_blocked(act.req.status) then
        return nil
    end
    return CLOCK.take_probe(act.req)
end

return REG
