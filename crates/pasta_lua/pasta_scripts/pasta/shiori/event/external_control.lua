---@module pasta.shiori.event.external_control
--- 外からゴーストを操作する既定のハンドラ（OnPastaCallFunction・OnPastaSetSaveVariable）
---
--- OnPastaCallFunction: Reference0 = 呼ぶ名前（＞名前 と同じ探し方）、Reference1 以降 = 引数（文字列のまま）
--- OnPastaSetSaveVariable: Reference0 = ＄＊ の後ろの名前（区切らない）、Reference1 = JSON の値 1 つ（null で消す）
---
--- デバッグの有無によらず働く。SecurityLevel が external なら警告だけ書いて何もしない（204）。
--- 呼ぶときはコルーチンを返すだけにし、1 ビート目・続き・実行時エラーは EVENT.fire に任せる。
--- 失敗では表記とログの警告を出して act:build() を返し、場面の続きを変えない。Status は見ない。
--- ゴーストが先に登録していれば上書きしない。event/init.lua は require しない（循環させない）。

local REG = require("pasta.shiori.event.register")
local FAILURE = require("pasta.failure")

--- 門: external なら warn を出して true（断る）
---@param act ShioriAct
---@param event string
---@return boolean
local function refused(act, event)
    if act.req.security_level ~= "external" then return false end
    require("@pasta_log").warn(event .. ": SecurityLevel external is refused; nothing done")
    return true
end

--- 失敗: 表記とログの警告を出し、act:build() を返す（表記を出さない設定なら nil）
---@param act ShioriAct
---@param record FailureRecord
---@return string|nil
local function fail(act, record)
    FAILURE.report(act, record)
    return act:build()
end

--- ログの警告に出す引数の値（nil はそのまま、それ以外は引用符で囲む）
local function quoted(value)
    return value == nil and "nil" or ("'" .. tostring(value) .. "'")
end

local function is_name(value)
    return type(value) == "string" and value ~= ""
end

--- Reference0 が無い・空のときの失敗
local function bad_reference0(act, event, value)
    return fail(act, {
        label = FAILURE.LABEL.BAD_NAME,
        subject = event .. " Reference0",
        detail = FAILURE.kind_of(value),
        warning = event .. ": bad argument: Reference0=" .. quoted(value),
    })
end

--- 引数: Reference1 から、値の無い最初の Reference の手前まで（文字列のまま。空も渡す）
---@param reference table<integer, string>
---@return string[] args, integer n
local function collect_args(reference)
    local args, n = {}, 0
    while reference[n + 1] ~= nil do
        n = n + 1
        args[n] = reference[n]
    end
    return args, n
end

REG.OnPastaCallFunction = REG.OnPastaCallFunction or function(act)
    local event = "OnPastaCallFunction"
    if refused(act, event) then return nil end
    local name = act.req.reference[0]
    if not is_name(name) then return bad_reference0(act, event, name) end

    -- ＞名前 と同じ探し方（skip_methods は渡さない。act のメソッドにも届く）
    local handler = act:find_handler("scene", name)
    if type(handler) ~= "function" then
        return fail(act, {
            label = FAILURE.LABEL.SCENE_NOT_FOUND,
            subject = event .. " ＞" .. name,
            warning = event .. ": handler not found: name='" .. name .. "'",
        })
    end
    local args, n = collect_args(act.req.reference)
    -- 呼び先はコルーチンの中で呼ぶ（区切っても続きが EVENT.fire の決まりで進む）。戻り値は捨てる
    return coroutine.create(function(a)
        handler(a, table.unpack(args, 1, n))
        return a:build()
    end)
end

--- Reference1 を JSON の値 1 つとして読む（decode の失敗も、投げられたエラーも「読めない」）
---@param json table  require("@json")
---@param text string|nil  Reference1
---@return "value"|"null"|"table"|"unreadable" kind
---@return number|string|boolean|nil value  kind == "value" のときだけ
local function read_json(json, text)
    if type(text) ~= "string" then return "unreadable" end
    local ok, value = pcall(json.decode, text)
    if not ok then return "unreadable" end
    -- and/or で書かない（JSON の false が消えるため）
    local t = type(value)
    if t == "number" or t == "string" or t == "boolean" then return "value", value end
    if t == "userdata" then return "null" end -- JSON の null（mlua の light userdata）
    if t == "table" then return "table" end
    return "unreadable"
end

REG.OnPastaSetSaveVariable = REG.OnPastaSetSaveVariable or function(act)
    local event = "OnPastaSetSaveVariable"
    if refused(act, event) then return nil end
    local name = act.req.reference[0]
    if not is_name(name) then return bad_reference0(act, event, name) end

    local subject = event .. " ＄＊" .. name
    local loaded, json = pcall(require, "@json")
    if not loaded then
        return fail(act, {
            label = FAILURE.LABEL.NOT_JSON,
            subject = subject,
            detail = "JSON の部品が無い",
            warning = event .. ": @json is not available; variable not set: name='" .. name .. "'",
        })
    end
    local text = act.req.reference[1]
    local kind, value = read_json(json, text)
    if kind == "unreadable" then
        -- 読めた文字列は引用符で包まずにそのまま説明に出す（"01" と送ったときと見分けるため）
        local detail = text
        if not is_name(text) then detail = FAILURE.kind_of(text) end
        return fail(act, {
            label = FAILURE.LABEL.NOT_JSON,
            subject = subject,
            detail = detail,
            warning = event .. ": not JSON: name='" .. name .. "', Reference1=" .. quoted(text),
        })
    end
    if kind == "table" then
        return fail(act, {
            label = FAILURE.LABEL.NOT_ASSIGNABLE,
            subject = subject,
            detail = "表",
            warning = event .. ": JSON array or object is refused: name='" .. name .. "', Reference1=" .. quoted(text),
        })
    end
    -- 名前は区切らず、予約変数も除かない。null は value が nil なのでキーが消える。書き出しは終了時の自動保存に任せる
    require("pasta.save")[name] = value
    return nil
end
