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
