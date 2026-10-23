---@module pasta.shiori.event.scene_play
--- シーンを指名して再生する既定のハンドラ（OnPastaScene・OnPastaSceneAt）
---
--- OnPastaScene:   Reference0 = 大域シーンの書いた名前、Reference1 = ローカルシーンの書いた名前（省略・空なら大域だけ）
--- OnPastaSceneAt: Reference0 = ファイル（ghost/master/ からの相対パスか絶対パス）、Reference1 = 行（半角数字だけの 1 以上）
---
--- デバッグが有効（@pasta_debug.enabled()）で、SecurityLevel が external でないときだけ働く。
--- 再生するときはコルーチンに STORE.co_preempt の印（はっきり捨てる）を付けて返し、差し替え・1 ビート目・実行時エラーは EVENT.fire に任せる。
--- 失敗では表記とログの警告を出して act:build() を返し、場面の続きを変えない。Status は見ない。
--- ゴーストが先に登録していれば上書きしない。event/init.lua は require しない（循環させない）。

local REG = require("pasta.shiori.event.register")
local FAILURE = require("pasta.failure")
local STORE = require("pasta.store")

--- 門: デバッグが有効で external でなければ @pasta_debug を返す。断るときは warn を出して nil
---@param act ShioriAct
---@param event string
---@return table|nil debug
local function allowed(act, event)
    local log = require("@pasta_log")
    local ok, debug = pcall(require, "@pasta_debug")
    if not ok or type(debug) ~= "table" or not debug.enabled() then
        log.warn(event .. ": debug is disabled; scene not played")
        return nil
    end
    if act.req.security_level == "external" then
        log.warn(event .. ": SecurityLevel external is refused; scene not played")
        return nil
    end
    return debug
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

--- Reference0 が無い・空のときの失敗
local function bad_reference0(act, event, value)
    return fail(act, {
        label = FAILURE.LABEL.BAD_NAME,
        subject = event .. " Reference0",
        detail = FAILURE.kind_of(value),
        warning = event .. ": bad argument: Reference0=" .. quoted(value),
    })
end

local function is_name(value)
    return type(value) == "string" and value ~= ""
end

REG.OnPastaScene = REG.OnPastaScene or function(act)
    local event = "OnPastaScene"
    if not allowed(act, event) then return nil end
    local name, local_name = act.req.reference[0], act.req.reference[1]
    if not is_name(name) then return bad_reference0(act, event, name) end
    if local_name == "" then local_name = nil end

    local co, miss = require("pasta.scene").co_exec_exact(name, local_name)
    if co then
        STORE.co_preempt[co] = true
        return co
    end
    local subject, warning = "＊" .. name, "global='" .. name .. "'"
    if miss == "local" then
        subject = subject .. " ・" .. local_name
        warning = warning .. ", local='" .. local_name .. "'"
    end
    return fail(act, {
        label = FAILURE.LABEL.SCENE_NOT_FOUND,
        subject = event .. " " .. subject,
        warning = event .. ": scene not found: " .. warning,
    })
end

REG.OnPastaSceneAt = REG.OnPastaSceneAt or function(act)
    local event = "OnPastaSceneAt"
    local debug = allowed(act, event)
    if not debug then return nil end
    local file, line = act.req.reference[0], act.req.reference[1]
    if not is_name(file) then return bad_reference0(act, event, file) end
    -- 全角の数字・前後の空白・小数・符号は数にしない（要件 2.6）
    local n = type(line) == "string" and line:match("^%d+$") and tonumber(line)
    if not n or n < 1 then
        return fail(act, {
            label = FAILURE.LABEL.NOT_NUMBER,
            subject = event .. " Reference1",
            detail = FAILURE.value_of(line),
            warning = event .. ": bad argument: Reference1=" .. quoted(line),
        })
    end

    local global_name, local_name = debug.scene_at(file, n)
    local co = global_name and require("pasta.scene").co_exec_registered(global_name, local_name)
    if co then
        STORE.co_preempt[co] = true
        return co
    end
    return fail(act, {
        label = FAILURE.LABEL.SCENE_NOT_FOUND,
        subject = event .. " " .. file .. ":" .. line,
        warning = event .. ": scene not found: file='" .. file .. "', line=" .. line,
    })
end
