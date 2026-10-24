---@module pasta.shiori.event.external_control
--- 外からゴーストを操作する既定のハンドラ（OnPastaCallFunction・OnPastaSetSaveVariable）
---
--- OnPastaCallFunction: Reference0 = 呼ぶ名前（＞名前 と同じ探し方）、Reference1 以降 = 引数（文字列のまま）
--- OnPastaSetSaveVariable: Reference0 = ＄＊ の後ろの名前（区切らない）、Reference1 = 値（届いた文字列のまま）、
---   Reference2 = 型（number・string・boolean・delete。無い・空は string。delete は Reference1 を見ずに消す）
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

--- 型の名前（英小文字の完全一致）。無い・空は string として扱う
local TYPES = { string = true, number = true, boolean = true, delete = true }

--- Reference1 を型ごとに読む。delete は Reference1 を見ずに nil（キーが消える）
---@param kind "string"|"number"|"boolean"|"delete"
---@param text string|nil  Reference1
---@return string|number|boolean|nil value  label が nil のときだけ
---@return string|nil label  Reference1 の誤りの種類
---@return string|nil what  ログの警告の言い回し
local function read_value(kind, text)
    if kind == "string" then
        if text == nil then return nil, FAILURE.LABEL.NOT_ASSIGNABLE, "no value" end
        return text
    elseif kind == "number" then
        local n = type(text) == "string" and tonumber(text)
        -- 無限大・非数は断る（n ~= n は非数）
        if type(n) ~= "number" or n ~= n or n == math.huge or n == -math.huge then
            return nil, FAILURE.LABEL.NOT_NUMBER, "not a number"
        end
        return n
    elseif kind == "boolean" then
        -- and/or で書かない（false が消えるため）
        if text == "true" then return true end
        if text == "false" then return false end
        return nil, FAILURE.LABEL.NOT_ASSIGNABLE, "not a boolean"
    end
    return nil
end

REG.OnPastaSetSaveVariable = REG.OnPastaSetSaveVariable or function(act)
    local event = "OnPastaSetSaveVariable"
    if refused(act, event) then return nil end
    local name = act.req.reference[0]
    if not is_name(name) then return bad_reference0(act, event, name) end

    local subject = event .. " ＄＊" .. name
    -- 型の名前を Reference1 より先に見る
    local kind = act.req.reference[2]
    if kind == nil or kind == "" then kind = "string" end
    if not TYPES[kind] then
        return fail(act, {
            label = FAILURE.LABEL.BAD_TYPE,
            subject = subject,
            detail = kind,
            warning = event .. ": unknown type: name='" .. name .. "', Reference2=" .. quoted(kind),
        })
    end
    local text = act.req.reference[1]
    local value, label, what = read_value(kind, text)
    if label then
        -- 値は引用符で包まずにそのまま説明に出す（無い・空は種類の説明）
        local detail = text
        if not is_name(text) then detail = FAILURE.kind_of(text) end
        return fail(act, {
            label = label,
            subject = subject,
            detail = detail,
            warning = event .. ": " .. what .. ": name='" .. name .. "', Reference1=" .. quoted(text),
        })
    end
    -- 名前は区切らず、予約変数も除かない。delete は value が nil なのでキーが消える。書き出しは終了時の自動保存に任せる
    require("pasta.save")[name] = value
    return nil
end
