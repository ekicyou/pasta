---@module pasta.shiori.event.recommend
--- おすすめメニューの既定のハンドラと組み込み（＞おすすめ項目・＞おすすめ区切り）
---
--- 組み込みは act の欄 _recommend_items（メニューの受け手だけが作る入れ物）に項目を積む。
--- 入れ物が無ければ（メニューの外では）何もしない。項目を台詞として積まない（act.token に触れない）。
--- event/init.lua は require しない（循環させない）。

local GLOBAL = require("pasta.global")

---@class RecommendItem
---@field name string          -- 区切り線は "-"
---@field url string           -- 区切り線は ""
---@field scene string|nil     -- 選んだときのシーンの書いた名前（無ければ nil）
---@field banner string|nil    -- バナーのパス（無ければ nil）

---@class RecommendHolder      -- act の欄 _recommend_items（メニューの受け手だけが作る）
---@field id string            -- リソースの ID（ログ用）
---@field calls integer        -- ＞おすすめ項目・＞おすすめ区切り を呼んだ数（ログの「何番目」）

--- 項目の欄の名前（引数の順）。名前・URL は欠かせない
local FIELDS = { "name", "url", "scene", "banner" }
local REQUIRED = 2

--- 欄の値を確かめる
---@param value any
---@param required boolean 無し（nil・空文字列）を誤りにするか
---@return string|nil text 使う値（無しは nil）
---@return string|nil bad 誤りの説明（誤りが無ければ nil）
local function check_text(value, required)
    local kind = type(value)
    if kind == "number" then return tostring(value) end
    if value == nil or value == "" then
        if required then return nil, value == nil and "nil" or "empty string" end
        return nil
    end
    if kind ~= "string" then return nil, kind end
    -- バイト値 1・2 はメニューの値の区切りに使われるので通さない
    if value:find("\1", 1, true) then return nil, "contains byte 1" end
    if value:find("\2", 1, true) then return nil, "contains byte 2" end
    return value
end

--- 入れ物を引き、呼んだ数を 1 増やす（無ければ nil）
---@param act ShioriAct
---@return RecommendHolder|nil
local function take_holder(act)
    local holder = type(act) == "table" and rawget(act, "_recommend_items") or nil
    if holder then holder.calls = holder.calls + 1 end
    return holder
end

--- ＞おすすめ項目（名前、URL、選んだときのシーン、バナー）
---@param act ShioriAct
---@param name any
---@param url any
---@param scene any|nil
---@param banner any|nil
---@param ... any  5 つ目から後（使わずに warn）
function GLOBAL.recommend_item(act, name, url, scene, banner, ...)
    local holder = take_holder(act)
    if not holder then return end
    local log = require("@pasta_log")
    local prefix = "recommend_item: " .. tostring(holder.id) .. " item #" .. holder.calls .. ": "
    local extra = select("#", ...)
    if extra > 0 then
        log.warn(prefix .. extra .. " extra argument(s) ignored")
    end
    local values = { name, url, scene, banner }
    local item = {}
    for i, field in ipairs(FIELDS) do
        local text, bad = check_text(values[i], i <= REQUIRED)
        if bad then
            log.warn(prefix .. "bad " .. field .. " (" .. bad .. "); item skipped")
            return
        end
        item[field] = text
    end
    holder[#holder + 1] = item
end

-- 非ASCII識別子はブラケット記法で定義する（entry.lua と同じ方針）
GLOBAL["おすすめ項目"] = GLOBAL.recommend_item

--- ＞おすすめ区切り（引数は見ない）
---@param act ShioriAct
function GLOBAL.recommend_separator(act)
    local holder = take_holder(act)
    if not holder then return end
    holder[#holder + 1] = { name = "-", url = "" }
end

GLOBAL["おすすめ区切り"] = GLOBAL.recommend_separator

-- ============================================================================
-- メニューの受け手（MenuReceiver）
-- ============================================================================

local REG = require("pasta.shiori.event.register")

--- MenuMemory: リソースの ID → 最後に開いたときの並び（メモリ上だけ。OnRecommendsiteChoice が読む）
--- 読む側（OnRecommendsiteChoice の受け手）を置くまでは書くだけなので、未使用の警告を止める
---@type table<string, RecommendItem[]>
local last = {} -- luacheck: ignore 241

--- メニューのシーンをその場で 1 回だけ動かし、積んだ項目を返す（シーンが無ければ nil）
--- EVENT.drive・set_co_scene・wrap_scene を通さない。続きのコルーチンは参照を外して捨てる
---@param act ShioriAct
---@param id string
---@return RecommendItem[]|nil
local function run_menu(act, id)
    local global_name, local_name = require("@pasta_search"):search_scene_exact(id)
    if not global_name then return nil end
    local fn = require("pasta.scene").get(global_name, local_name)
    if type(fn) ~= "function" then return nil end
    local items = { id = id, calls = 0 }
    act._recommend_items = items
    local ok, err = coroutine.resume(coroutine.create(fn), act)
    -- 台詞と失敗の表記は値に混ぜない
    act.token = {}
    if not ok then error(err, 0) end
    return items
end

--- 項目を SSP の値にする（欄はバイト値 1、各項目の後ろにバイト値 2。トークの欄は常に空）
---@param items RecommendItem[]
---@return string
local function encode(items)
    local parts = {}
    for i, item in ipairs(items) do
        parts[i] = item.name .. "\1" .. item.url .. "\1" .. (item.banner or "") .. "\1\2"
    end
    return table.concat(parts)
end

--- メニューの受け手を作る（ID を閉じ込める）。項目が無ければ記憶を消して nil（204）
---@param id string
---@return fun(act: ShioriAct): string|nil
local function menu_receiver(id)
    return function(act)
        local items = run_menu(act, id)
        if not items or #items == 0 then
            last[id] = nil
            return nil
        end
        last[id] = items
        return encode(items)
    end
end

--- 受け手を置く ID: 決まった 3 つと、別名の表の置き換え先のうち charN.recommendsites
local menu_ids = { "sakura.recommendsites", "kero.recommendsites", "sakura.portalsites" }
-- @pasta_search が引けない・scene_aliases を持たない（テストの差し替えなど）なら決まった 3 つだけ
local found, aliases = pcall(function() return require("@pasta_search"):scene_aliases() end)
if found and type(aliases) == "table" then
    for target in pairs(aliases) do
        if target:match("^char%d+%.recommendsites$") then menu_ids[#menu_ids + 1] = target end
    end
end
-- 作者が先に置いた受け手を上書きしない
for _, id in ipairs(menu_ids) do
    REG[id] = REG[id] or menu_receiver(id)
end
