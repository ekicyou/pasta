--- @module pasta.shiori.stop
--- 止まるエラー（シーン・ハンドラが Lua のエラーで止まった・時間切れ）の表記
---
--- 場所の見分けとエラー文の整えは純粋な関数。読み込んだ Lua のファイルの表（`@pasta_chunks`）と、
--- act が控えた行の対応（`act.line_map`）を引数で受け取る。

local STOP = {}

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

return STOP
