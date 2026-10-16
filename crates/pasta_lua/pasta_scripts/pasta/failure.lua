--- @module pasta.failure
--- 実行時の失敗の報告の唯一の出口
---
--- 失敗 1 件を、ログの警告と失敗表記（raw_script トークン）に変える。
--- 依存はログ（と設定）だけ。pasta.act・pasta.actor・pasta.word は読み込まない（循環させないため）。

local FAILURE = {}

--- 種類の名前の表（失敗表記の【】の先頭に出す。後続 spec は自分の名前を足す）
---@type table<string, string>
FAILURE.LABEL = {
    UNDEFINED_VAR      = "値の無い変数",
    WORD_NOT_FOUND     = "単語が見つからない",
    FN_NOT_FOUND       = "関数が見つからない",
    BAD_NAME           = "名前にならない値",
    UNREGISTERED_ACTOR = "未登録アクター",
    NOT_NUMBER         = "数にできない値",
    NOT_STRING         = "文字列にできない値",
    SCENE_NOT_FOUND    = "シーンが見つからない",
}

local ESCAPE = { ["\\"] = "\\\\", ["%"] = "\\%", ["\r"] = " ", ["\n"] = " " }

--- さくらスクリプトとして働く文字を無効にする: "\" → "\\"、"%" → "\%"、CR・LF → 空白
---@param s string
---@return string
function FAILURE.escape(s)
    return (s:gsub("[\\%%\r\n]", ESCAPE))
end

--- 表記の文字列（枠付き・エスケープ済み）。切り替えを見ない純粋な関数
---
--- 形は `【label：subject（detail）】`。detail が無ければ（…）を付けず、
--- subject が無ければ detail を対象の位置に出し、両方無ければ `【label】`。
--- エスケープは中身全体にかけ、枠はその後で付ける。行番号は含めない。
---@param label string
---@param subject string|nil
---@param detail string|nil
---@return string  例 "【数にできない値：＄y（\"abc\"）】"
function FAILURE.format(label, subject, detail)
    local body = tostring(label)
    if subject ~= nil then
        body = body .. "：" .. tostring(subject)
        if detail ~= nil then
            body = body .. "（" .. tostring(detail) .. "）"
        end
    elseif detail ~= nil then
        body = body .. "：" .. tostring(detail)
    end
    return "【" .. FAILURE.escape(body) .. "】"
end

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
