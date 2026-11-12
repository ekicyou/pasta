---@module pasta.shiori.event.scene_play
--- 開発用のイベントの既定のハンドラ（OnPastaScene・OnPastaSceneAt・OnPastaDumpVariables）
---
--- OnPastaScene:   Reference0 = 大域シーンの書いた名前、Reference1 = ローカルシーンの書いた名前（省略・空なら大域だけ）
--- OnPastaSceneAt: Reference0 = ファイル（ghost/master/ からの相対パスか絶対パス）、Reference1 = 行（半角数字だけの 1 以上）
--- 2 つに共通の Reference2: 省略できる TOML のインライン表（{ '天気' = '雨' }）。シーンが見つかったときだけ、
---   各値を act.var（再生するシーンのローカル変数）に入れてから再生する。誤りなら何も入れずに失敗の表記
--- OnPastaDumpVariables: 引数なし。保存変数を profile/pasta/debug/variables.toml に書き、喋らずに 204
---
--- 3 つに共通の決まり: デバッグが有効（@pasta_debug.enabled()）で、SecurityLevel が external でないときだけ働く。
--- 断るときは「<イベント名>: debug is disabled; …」か「<イベント名>: SecurityLevel external is refused; …」の
--- 警告を 1 行書いて 204。ゴーストが先に登録していれば上書きしない。
--- 再生するときはコルーチンに STORE.co_preempt の印（はっきり捨てる）を付けて返し、差し替え・1 ビート目・実行時エラーは EVENT.fire に任せる。
--- 再生の失敗では表記とログの警告を出して act:build() を返し、場面の続きを変えない。Status は見ない。
--- event/init.lua は require しない（循環させない）。

local REG = require("pasta.shiori.event.register")
local FAILURE = require("pasta.failure")
local STORE = require("pasta.store")

--- 門: デバッグが有効で external でなければ @pasta_debug を返す。断るときは warn を出して nil
---@param act ShioriAct
---@param event string
---@param outcome string 警告の後半（"scene not played" など）
---@return table|nil debug
local function allowed(act, event, outcome)
    local log = require("@pasta_log")
    local ok, debug = pcall(require, "@pasta_debug")
    if not ok or type(debug) ~= "table" or not debug.enabled() then
        log.warn(event .. ": debug is disabled; " .. outcome)
        return nil
    end
    if act.req.security_level == "external" then
        log.warn(event .. ": SecurityLevel external is refused; " .. outcome)
        return nil
    end
    return debug
end

--- Reference2（TOML のインライン表）を読み、検査を通ったローカル変数の表か、失敗の応答を返す。act.var には触れない
---@param act ShioriAct
---@param event string
---@param debug table 門が返した @pasta_debug（is_local_var_name を持つ）
---@return boolean ok
---@return table|string|nil  ok なら入れる表（Reference2 が無ければ nil）、ok でなければ FAILURE.reply の戻り値
local function read_locals(act, event, debug)
    local raw = act.req.reference[2]
    if raw == nil or raw == "" then return true, nil end
    -- @toml は Lua の仕様のテストのランナーに無いので、Reference2 があるときだけ読む
    local doc, err = require("@toml").decode("v = " .. raw)
    local reason
    if not doc then
        -- 位置の行は "v = " の分だけ列がずれるので、最後の空でない行（誤りの文）だけを載せる
        for l in err:gmatch("[^\r\n]+") do
            if l:match("%S") then reason = l end
        end
    -- @toml は配列も表にするので、インライン表かは先頭の { で見分ける（= の後に改行は置けないので %s は空白とタブだけになる）
    elseif type(doc.v) ~= "table" or not raw:find("^%s*{") then
        reason = "not an inline table"
    else
        for k in pairs(doc) do
            if k ~= "v" then reason = "not an inline table" end
        end
    end
    if reason then
        return false, FAILURE.reply(act, {
            label = FAILURE.LABEL.NOT_TABLE,
            subject = event .. " Reference2",
            detail = FAILURE.value_of(raw),
            warning = event .. ": bad argument: Reference2=" .. FAILURE.quoted(raw) .. " (" .. reason .. ")",
        })
    end

    local keys = {}
    for k in pairs(doc.v) do keys[#keys + 1] = k end
    table.sort(keys)
    for _, k in ipairs(keys) do
        if not debug.is_local_var_name(k) then
            return false, FAILURE.reply(act, {
                label = FAILURE.LABEL.BAD_NAME,
                subject = event .. " Reference2",
                detail = FAILURE.value_of(k),
                warning = event .. ": bad argument: Reference2 key=" .. FAILURE.quoted(k)
                    .. " (not a local variable name)",
            })
        end
        if type(doc.v[k]) == "table" then
            return false, FAILURE.reply(act, {
                label = FAILURE.LABEL.NOT_ASSIGNABLE,
                subject = event .. " Reference2 ＄" .. k,
                detail = FAILURE.kind_of(doc.v[k]),
                warning = event .. ": bad argument: Reference2 key=" .. FAILURE.quoted(k)
                    .. " (table is not assignable)",
            })
        end
    end
    return true, doc.v
end

REG.OnPastaScene = REG.OnPastaScene or function(act)
    local event = "OnPastaScene"
    local debug = allowed(act, event, "scene not played")
    if not debug then return nil end
    local name, local_name = act.req.reference[0], act.req.reference[1]
    if not FAILURE.is_name(name) then return FAILURE.bad_reference0(act, event, name) end
    if local_name == "" then local_name = nil end
    local ok, locals = read_locals(act, event, debug)
    if not ok then return locals end

    local co, miss = require("pasta.scene").co_exec_exact(name, local_name)
    if co then
        for k, v in pairs(locals or {}) do act.var[k] = v end
        STORE.co_preempt[co] = true
        return co
    end
    local subject, warning = "＊" .. name, "global='" .. name .. "'"
    if miss == "local" then
        subject = subject .. " ・" .. local_name
        warning = warning .. ", local='" .. local_name .. "'"
    end
    return FAILURE.reply(act, {
        label = FAILURE.LABEL.SCENE_NOT_FOUND,
        subject = event .. " " .. subject,
        warning = event .. ": scene not found: " .. warning,
    })
end

REG.OnPastaSceneAt = REG.OnPastaSceneAt or function(act)
    local event = "OnPastaSceneAt"
    local debug = allowed(act, event, "scene not played")
    if not debug then return nil end
    local file, line = act.req.reference[0], act.req.reference[1]
    if not FAILURE.is_name(file) then return FAILURE.bad_reference0(act, event, file) end
    -- 全角の数字・前後の空白・小数・符号は数にしない（要件 2.6）
    local n = type(line) == "string" and line:match("^%d+$") and tonumber(line)
    if not n or n < 1 then
        return FAILURE.reply(act, {
            label = FAILURE.LABEL.NOT_NUMBER,
            subject = event .. " Reference1",
            detail = FAILURE.value_of(line),
            warning = event .. ": bad argument: Reference1=" .. FAILURE.quoted(line),
        })
    end
    local ok, locals = read_locals(act, event, debug)
    if not ok then return locals end

    local global_name, local_name = debug.scene_at(file, n)
    local co = global_name and require("pasta.scene").co_exec_registered(global_name, local_name)
    if co then
        for k, v in pairs(locals or {}) do act.var[k] = v end
        STORE.co_preempt[co] = true
        return co
    end
    return FAILURE.reply(act, {
        label = FAILURE.LABEL.SCENE_NOT_FOUND,
        subject = event .. " " .. file .. ":" .. line,
        warning = event .. ": scene not found: file='" .. file .. "', line=" .. line,
    })
end

REG.OnPastaDumpVariables = REG.OnPastaDumpVariables or function(act)
    local debug = allowed(act, "OnPastaDumpVariables", "variables not dumped")
    if debug then debug.dump_variables(require("pasta.save")) end
    return nil -- 喋らない（204）
end
