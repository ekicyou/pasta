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
    SCRIPT_ERROR       = "スクリプトのエラー",
    TIMEOUT            = "時間切れ",
    NOT_JSON           = "JSON として読めない値",
    NOT_ASSIGNABLE     = "代入できない値",
}

local ESCAPE = { ["\\"] = "\\\\", ["%"] = "\\%", ["\r"] = " ", ["\n"] = " " }

--- さくらスクリプトとして働く文字を無効にする: "\" → "\\"、"%" → "\%"、CR・LF → 空白
---@param s string
---@return string
function FAILURE.escape(s)
    return (s:gsub("[\\%%\r\n]", ESCAPE))
end

--- 表記の中身（枠なし・エスケープなし）。ログの警告に表記と同じ中身を出すのに使う純粋な関数
---
--- 形は `label：subject（detail）`。detail が無ければ（…）を付けず、
--- subject が無ければ detail を対象の位置に出し、両方無ければ `label`。
---@param label string
---@param subject string|nil
---@param detail string|nil
---@return string
function FAILURE.body(label, subject, detail)
    local body = tostring(label)
    if subject ~= nil then
        body = body .. "：" .. tostring(subject)
        if detail ~= nil then
            body = body .. "（" .. tostring(detail) .. "）"
        end
    elseif detail ~= nil then
        body = body .. "：" .. tostring(detail)
    end
    return body
end

--- 表記の文字列（枠付き・エスケープ済み）。切り替えを見ない純粋な関数
---
--- 中身は FAILURE.body の形。エスケープは中身全体にかけ、枠はその後で付ける。行番号は含めない。
---@param label string
---@param subject string|nil
---@param detail string|nil
---@return string  例 "【数にできない値：＄y（\"abc\"）】"
function FAILURE.format(label, subject, detail)
    return "【" .. FAILURE.escape(FAILURE.body(label, subject, detail)) .. "】"
end

local FULLWIDTH_DIGIT = {
    ["0"] = "０", ["1"] = "１", ["2"] = "２", ["3"] = "３", ["4"] = "４",
    ["5"] = "５", ["6"] = "６", ["7"] = "７", ["8"] = "８", ["9"] = "９",
}

--- Lua の書き方の変数パスを辞書の書き方へ。知らない形はそのまま返す
--- "var.x" → "＄x"、"save.x" → "＄＊x"、"args[1]" → "＄０"（添字 - 1 を全角数字で）
---@param var_path string
---@return string
function FAILURE.var(var_path)
    local name = var_path:match("^var%.(.+)$")
    if name then return "＄" .. name end
    name = var_path:match("^save%.(.+)$")
    if name then return "＄＊" .. name end
    local index = tonumber(var_path:match("^args%[(%d+)%]$"))
    if index and index >= 1 then
        return "＄" .. tostring(index - 1):gsub("%d", FULLWIDTH_DIGIT)
    end
    return var_path
end

--- 被演算子・関数の説明を辞書の書き方へ。知らない形はそのまま返す
--- "@名前()" → "＠名前（）"、"@*名前()" → "＠＊名前（）"、"@$var.x()" → "＠＄x（）"、
--- 変数パス（"var.x" など）は FAILURE.var と同じ
---@param desc string
---@return string
function FAILURE.fn(desc)
    local mark, name = desc:match("^@([%$%*]?)(.*)%(%)$")
    if not name or name == "" then return FAILURE.var(desc) end
    if mark == "*" then return "＠＊" .. name .. "（）" end
    if mark == "" then return "＠" .. name .. "（）" end
    local converted = FAILURE.var(name)
    -- 変数パスでなければ知らない形
    if converted == name then return desc end
    return "＠" .. converted .. "（）"
end

local KIND = { boolean = "真偽値", table = "表", ["function"] = "関数" }

--- 名前にならない値の説明: nil → "値なし"、"" → "空文字列"、真偽値 → "真偽値"、表 → "表"、関数 → "関数"、
--- それ以外 → Lua の型の名前（読み手は DSL の作者のため日本語にする）
---@param v any
---@return string
function FAILURE.kind_of(v)
    if v == nil then return "値なし" end
    if v == "" then return "空文字列" end
    local t = type(v)
    return KIND[t] or t
end

--- 数・文字列にできない値の説明: 文字列 → "\"abc\""、真偽値 → "true"/"false"、それ以外 → 型の名前
---@param v any
---@return string
function FAILURE.value_of(v)
    local t = type(v)
    if t == "string" then return '"' .. v .. '"' end
    if t == "boolean" then return tostring(v) end
    return FAILURE.kind_of(v)
end

---@class FailureRecord
---@field label string        種類の名前（FAILURE.LABEL の値。後続 spec は自分の名前を足す）
---@field subject string|nil  対象（辞書の書き方に変換済み。無ければ説明を対象の位置に出す）
---@field detail string|nil   説明（値なし・空文字列・型の名前・値）
---@field warning string|nil  ログの警告文（現行の文言のまま。nil ならログを出さない）
---@field actor table|nil     未登録アクターの目印のとき、そのアクター

--- 警告を 1 回出す（nil なら出さない）
---@param warning any
local function warn(warning)
    if warning ~= nil then
        -- 呼んだ時点で引く: テストがログを差し替えて act を読み直しても、その差し替えに出す
        require("@pasta_log").warn(tostring(warning))
    end
end

--- [logging] balloon_warning（バルーンに失敗表記を出すか）。真偽値でなければ既定値 true
---
--- 報告のたびに読む（キャッシュしない）。ログの警告はこの設定にかかわらず出す。
---@return boolean
function FAILURE.enabled()
    -- 呼んだ時点で引く（ログと同じ。テストが設定を差し替えられるように）
    local value = require("pasta.config").get("logging", "balloon_warning", true)
    if type(value) ~= "boolean" then return true end
    return value
end

--- act の無い場所からの報告: 警告を出し、表記の文字列を返す（出さない設定なら nil）。トークンに積まない
---@param record FailureRecord
---@return string|nil
function FAILURE.notation(record)
    warn(record.warning)
    if not FAILURE.enabled() then return nil end
    return FAILURE.format(record.label, record.subject, record.detail)
end

--- act を通じた報告: 警告を出し、表記を act.token に raw_script として積む。act が nil なら notation と同じ
---
--- raw_script はそのまま出力される（句読点ウェイト・分かち書き改行は入らない）。
--- 切替タグを出さずに積んだ位置（直前に話したアクターのバルーン、
--- または出力の先頭なら現在のスコープ）へ文字を出す。
--- 出さない設定では表記を積まない。ただし未登録アクターの目印（record.actor あり）は、
--- 話者の切り替えを出す設定と同じにするため空の文字で積む。
---@param act table|nil
---@param record FailureRecord
---@return nil
function FAILURE.report(act, record)
    local text = FAILURE.notation(record)
    if act and (text or record.actor) then
        table.insert(act.token, { type = "raw_script", text = text or "", actor = record.actor })
    end
    return nil
end

--- 実行中の act の対応表（スレッド → act）
local RUNNING = setmetatable({}, { __mode = "k" })
--- 直前の関数呼び出しが報告済みか（act → 真偽値）
local FN_FAILED = setmetatable({}, { __mode = "k" })
--- 未登録アクターの印（アクター → true）。アクターの表のフィールドには書かない
local UNREGISTERED = setmetatable({}, { __mode = "k" })

--- 実行中のスレッドに act を対応させる（coroutine.running() が nil なら何もしない）
---@param act table
function FAILURE.bind(act)
    local co = coroutine.running()
    if co then RUNNING[co] = act end
end

--- 実行中のスレッドに対応する act（無ければ nil）
---@return table|nil
function FAILURE.running_act()
    local co = coroutine.running()
    return co and RUNNING[co] or nil
end

--- コルーチンに対応する act（無ければ nil）。読むだけ（止まったコルーチンの act を外から引く）
---@param co thread
---@return table|nil
function FAILURE.act_of(co)
    return RUNNING[co]
end

---@param act table
---@param failed boolean
function FAILURE.set_fn_failed(act, failed)
    FN_FAILED[act] = failed and true or nil
end

--- 印を読んで偽に戻す
---@param act table
---@return boolean
function FAILURE.take_fn_failed(act)
    local failed = FN_FAILED[act] == true
    FN_FAILED[act] = nil
    return failed
end

---@param actor table
function FAILURE.mark_unregistered_actor(actor)
    UNREGISTERED[actor] = true
end

---@param actor table|nil
---@return boolean
function FAILURE.is_unregistered_actor(actor)
    return actor ~= nil and UNREGISTERED[actor] == true
end

return FAILURE
