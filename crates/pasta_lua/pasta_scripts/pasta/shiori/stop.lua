--- @module pasta.shiori.stop
--- 止まるエラー（シーン・ハンドラが Lua のエラーで止まった・時間切れ）の表記
---
--- 場所の見分けとエラー文の整えは純粋な関数。読み込んだ Lua のファイルの表（`@pasta_chunks`）と、
--- act が控えた行の対応（`act.line_map`）を引数で受け取る。
---
--- 依存は FAILURE と RES だけ（読み込み時）。表記だけの act を作るときの pasta.shiori.act・pasta.store は
--- 呼んだ時点で require する（読み込み時に握ると callback.lua と循環する）。

local FAILURE = require("pasta.failure")
local RES = require("pasta.shiori.res")

local STOP = {}

--- 表記を出す応答のステータス（タスク 1 の SSP の実機の確かめで決めた。500 の Value はバルーンに出ない）
STOP.STATUS = "200 OK"

---@class StopHold
---@field act table|nil  止まったコルーチンの act（FAILURE.act_of(co)。無ければ nil）
---@field err any        EVENT.drive が受けたエラーの値（投げ直す前の元の値）

--- 1 要求に 1 つの控え（シーンの失敗）と、ハンドラの act の控え
---@type StopHold|nil
local held = nil
---@type table|nil
local held_handler = nil

--- 控えを空にする（要求の始めに呼ぶ）
function STOP.clear()
    held = nil
    held_handler = nil
end

--- シーンの失敗を控える（EVENT.drive の失敗の枝だけが呼ぶ）。前の控えは上書きする
---@param co thread
---@param err any
function STOP.hold(co, err)
    held = { act = FAILURE.act_of(co), err = err }
end

--- 控えを取り出して空にする
---@return StopHold|nil
function STOP.take()
    local h = held
    held = nil
    return h
end

--- イベントのハンドラに渡す act を控える（EVENT.fire がハンドラを呼ぶ直前に呼ぶ）。STOP.clear で空になる
---@param act table
function STOP.hold_handler(act)
    held_handler = act
end

--- 控えたハンドラの act を取り出して空にする
---@return table|nil
function STOP.take_handler()
    local act = held_handler
    held_handler = nil
    return act
end

--- act が控えた書いた名前から、止まった場所の名前を作る（"＊天気・雨の日"・"＊天気"）
--- 書いた名前が無ければ（古いキャッシュ）登録名から末尾の "_連番" を落とした名前、それも無ければ nil
---@param act table|nil
---@return string|nil
function STOP.subject(act)
    if not act then return nil end
    if act.scene_name then
        if act.local_scene_name then
            return "＊" .. act.scene_name .. "・" .. act.local_scene_name
        end
        return "＊" .. act.scene_name
    end
    local global_name = act.current_scene and act.current_scene.__global_name__
    if type(global_name) ~= "string" then return nil end
    return "＊" .. global_name:gsub("_%d+$", "")
end

--- 読み込んだ Lua のファイルの表（ローダーを通さない VM では無い）
---@return table|nil
local function default_chunks()
    return package.loaded["@pasta_chunks"]
end

--- 行の対応 "<.pasta のファイル>|<lua の行>:<pasta の行>,..." から、Lua の行を "ファイル:行" に直す
---@param line_map string|nil
---@param line number
---@return string|nil
local function pasta_location(line_map, line)
    if type(line_map) ~= "string" then return nil end
    local file, list = line_map:match("^(.*)|(.*)$")
    if not file then return nil end
    for lua_line, pasta_line in list:gmatch("(%d+):(%d+)") do
        if tonumber(lua_line) == line then
            return file:gsub("/", "\\") .. ":" .. pasta_line
        end
    end
    return nil
end

--- 位置の接頭辞のチャンクと行を、ゴーストのフォルダーから見た場所にする（純粋な関数）
--- 作者の .lua は "scripts\天気.lua:12"、変換した Lua のキャッシュは "dic\天気.pasta:15"。
--- 出さない場所・見分けられない場所（表に無い・重複・対応に無い行）は nil
---@param chunk string  エラー文の "チャンク:行: " のチャンクの部分
---@param line number
---@param chunks table|nil  読み込んだ Lua のファイルの表（省略時は @pasta_chunks）
---@param line_map string|nil  act が控えた行の対応
---@return string|nil
function STOP.location(chunk, line, chunks, line_map)
    chunks = chunks or default_chunks()
    local entry = chunks and chunks[chunk]
    if entry == true then
        return pasta_location(line_map, line)
    elseif type(entry) == "string" then
        return entry:gsub("/", "\\") .. ":" .. line
    end
    return nil
end

--- エラー文の最初の行から、出さない場所の接頭辞を除き、残す場所を "場所:行: " の形に直した文（純粋な関数）
--- 文字列でなければ nil。除いた後が空なら nil
---@param err any
---@param chunks table|nil  読み込んだ Lua のファイルの表（省略時は @pasta_chunks）
---@param line_map string|nil  act が控えた行の対応
---@return string|nil
function STOP.detail(err, chunks, line_map)
    if type(err) ~= "string" then return nil end
    chunks = chunks or default_chunks() or {}
    local rest = err:match("^[^\r\n]*")
    local kept = {}
    while true do
        local chunk, line, after = rest:match("^(.-):(%d+): (.*)$")
        -- 表で引けるものと、.lua・] で終わるものだけを位置の接頭辞とする（文中の "10:30: " を取らない）
        if not chunk or (chunks[chunk] == nil and not chunk:match("%.lua$") and not chunk:match("%]$")) then
            break
        end
        local loc = STOP.location(chunk, tonumber(line), chunks, line_map)
        if loc then table.insert(kept, loc .. ": ") end
        rest = after
    end
    local text = table.concat(kept) .. rest
    if text == "" then return nil end
    return text
end

---@class StopReport
---@field reason string|nil   応答の理由の行（表記を出さない場合と同じ値。そのまま使う）
---@field hold StopHold|nil   シーンの失敗の控え（STOP.take の結果）
---@field handler table|nil  ハンドラの act の控え（STOP.take_handler の結果）
---@field err any             控えが無いときのエラーの値
---@field fallback string|nil 名前が分からないときの対象（要求の ID）
---@field timeout string|nil  理由付きの時間切れなら timeout_message（種類が「時間切れ」になる）
---@field warning string|boolean|nil  ログの警告（nil・false なら出さない。true なら表記と同じ整えた中身、文字列ならそれを前に付けた中身）
---@field shown boolean       表記を出してよい要求か（ID が "On" で始まる）

--- 表記を act の末尾に raw_script として積み、組み立てた値（位置と話者は FAILURE.report と同じ）
--- act が無ければ新しい SHIORI の act に表記だけを積む。組み立てに失敗したら表記だけを値にする
---@param act table|nil
---@param text string
---@return string
local function build_value(act, text)
    local ok, value = pcall(function()
        if not act then
            act = require("pasta.shiori.act").new(require("pasta.store").actors)
        end
        table.insert(act.token, { type = "raw_script", text = text })
        return act:build()
    end)
    if ok and type(value) == "string" and value ~= "" then return value end
    return text
end

--- 止まるエラー 1 件の、表記を出す応答の全文
--- 出さないとき（shown が偽・表記を出さない設定）と組み立てに失敗したときは nil（呼び出し元が今の応答を返す）。
--- 警告（report.warning）は、表記を出すかどうかにかかわらず 1 回出す
---@param report StopReport
---@return string|nil
function STOP.response(report)
    local ok, res = pcall(function()
        local hold = report.hold
        local act = hold and hold.act or report.handler
        local label, subject, detail
        if report.timeout ~= nil then
            label, subject = FAILURE.LABEL.TIMEOUT, report.timeout
        else
            label = FAILURE.LABEL.SCRIPT_ERROR
            local err
            if hold then err = hold.err else err = report.err end
            if type(err) == "string" then
                subject = STOP.subject(act) or report.fallback
                detail = STOP.detail(err, nil, act and act.line_map)
            end
        end
        local warning = report.warning
        if warning then
            local body = FAILURE.body(label, subject, detail)
            warning = type(warning) == "string" and warning .. body or body
        else
            warning = nil
        end
        local text = FAILURE.notation({ label = label, subject = subject, detail = detail, warning = warning })
        if not text or not report.shown then return nil end
        local reason = report.reason
        if reason == nil or reason == "" then reason = "Unknown error" end
        return RES.build(STOP.STATUS, { Value = build_value(act, text), ["X-Error-Reason"] = reason })
    end)
    if ok then return res end
    return nil
end

return STOP
