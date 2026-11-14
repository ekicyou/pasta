---@module pasta.shiori.event.sys_resume
--- OnSysResume デフォルトハンドラ
---
--- 処理の順:
---   (1) ＊OnSysResume のシーンがあれば、そのコルーチンを返す（既定の予約はしない）。
---   (2) Reference0 が "auto"（システムのスケジュールによる自動の復帰）なら何もしない。
---   (3) スリープ明けのおしゃべりを予約する（dispatcher.wake_talk）。応答は 204。
---       発行はしない。次の OnSecondChange からの check_talk が、今の門を通ったうえで出す。
--- ゴースト開発者は REG.OnSysResume を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local SCENE = require("pasta.scene")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

---OnSysResume デフォルトハンドラ
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return thread|nil シーンが見つかればコルーチン、それ以外は nil
REG.OnSysResume = REG.OnSysResume or function(act)
    local co = SCENE.co_exec(act, act.req.id, nil, nil)
    if co then
        return co
    end
    local ref = act.req.reference
    if ref and ref[0] == "auto" then
        return nil
    end
    dispatcher.wake_talk(act)
    return nil
end

return REG
