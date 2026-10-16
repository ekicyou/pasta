--- @module pasta.failure
--- 実行時の失敗の報告の唯一の出口
---
--- 失敗 1 件を、ログの警告と失敗表記（raw_script トークン）に変える。
--- 依存はログ（と設定）だけ。pasta.act・pasta.actor・pasta.word は読み込まない（循環させないため）。

local FAILURE = {}

---@class FailureRecord
---@field label string        表記の中身（【】はこのモジュールが付ける）
---@field warning string|nil  ログの警告文。nil ならログを出さない

--- act を通じた報告: 警告を出し、【label】を act.token に raw_script として積む
---
--- raw_script はそのまま出力される（句読点ウェイト・分かち書き改行は入らない）。
--- 切替タグを出さずに積んだ位置（直前に話したアクターのバルーン、
--- または出力の先頭なら現在のスコープ）へ文字を出す。act が nil なら警告だけ出す。
--- @param act table|nil アクションオブジェクト
--- @param record FailureRecord
--- @return nil
function FAILURE.report(act, record)
    if record.warning then
        -- 呼んだ時点で引く: テストがログを差し替えて act を読み直しても、その差し替えに出す
        require("@pasta_log").warn(record.warning)
    end
    if act then
        table.insert(act.token, { type = "raw_script", text = "【" .. record.label .. "】" })
    end
    return nil
end

return FAILURE
