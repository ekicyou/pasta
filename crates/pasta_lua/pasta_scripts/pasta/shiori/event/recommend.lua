---@module pasta.shiori.event.recommend
--- おすすめメニュー・ボタンの名前の既定のハンドラと組み込み（＞おすすめ項目・＞おすすめ区切り）
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
local CALLBACK = require("pasta.shiori.event.callback")
local STORE = require("pasta.store")
local ACT = require("pasta.act")

--- MenuMemory: リソースの ID → 最後に開いたときの並び（メモリ上だけ。OnRecommendsiteChoice が読む）
---@type table<string, RecommendItem[]>
local last = {}

--- 区切り: 自動トークを再開せず、組み立てずに止める（resume は 1 回だけなので続きは捨てる）
local function cut()
    coroutine.yield()
end

--- メニューのシーンをその場で 1 回だけ動かし、積んだ項目を返す（シーンが無ければ nil）
--- EVENT.drive・set_co_scene・wrap_scene を通さない。続きのコルーチンは参照を外して捨てる。
--- STORE は last_global_scene の控えと戻しのほかに触れない（変数の代入は取り消さない）
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
    -- インスタンスの欄で区切りを差し替える: 持ち越し（立ち位置・表情）に触れない組み立てと、
    -- 自動トークを再開しない区切り。get_property も self:build()・yield で区切る
    act.build = ACT.IMPL.build
    act.yield = cut
    local last_global_scene = STORE.last_global_scene
    local ok, err = coroutine.resume(coroutine.create(fn), act)
    -- 成否にかかわらず: この要求で積まれたコールバック待ちを捨て、控えを戻す
    local waited = CALLBACK.has_staged()
    CALLBACK.discard_staged()
    STORE.last_global_scene = last_global_scene
    -- 台詞と失敗の表記は値に混ぜない
    act.token = {}
    if not ok then error(err, 0) end
    if waited then
        require("@pasta_log").warn("recommend menu: " .. id
            .. ": get_property is not available in a menu scene; the scene was cut there")
    end
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

--- 受け手を置く ID: 決まった 3 つずつと、別名の表の置き換え先のうち charN.recommendsites（.caption）
local menu_ids = { "sakura.recommendsites", "kero.recommendsites", "sakura.portalsites" }
local caption_ids = { "sakura.recommendbuttoncaption", "kero.recommendbuttoncaption", "sakura.portalbuttoncaption" }
-- @pasta_search が引けない・scene_aliases を持たない（テストの差し替えなど）なら決まった 3 つだけ
local found, aliases = pcall(function() return require("@pasta_search"):scene_aliases() end)
if not (found and type(aliases) == "table") then aliases = {} end
for target in pairs(aliases) do
    if target:match("^char%d+%.recommendsites$") then menu_ids[#menu_ids + 1] = target end
    if target:match("^char%d+%.recommendsites%.caption$") then caption_ids[#caption_ids + 1] = target end
end
-- 作者が先に置いた受け手を上書きしない
for _, id in ipairs(menu_ids) do
    REG[id] = REG[id] or menu_receiver(id)
end

-- ============================================================================
-- ボタンの名前の受け手（CaptionReceiver）
-- ============================================================================

--- ボタンの名前の受け手を作る。別名の辞書順に ＠名前 と同じ前方一致で単語を引き、
--- 最初に見つかった値をそのまま返す（無ければ nil で 204）。act:word は使わない（失敗の表記を積まない）
---@param id string
---@return fun(act: ShioriAct): string|nil
local function caption_receiver(id)
    local names = {}
    for i, name in ipairs(aliases[id] or {}) do names[i] = name end
    table.sort(names)
    return function()
        local SEARCH = require("@pasta_search")
        for _, name in ipairs(names) do
            local word = SEARCH:search_word(name)
            if word ~= nil then return word end
        end
        return nil
    end
end

for _, id in ipairs(caption_ids) do
    REG[id] = REG[id] or caption_receiver(id)
end

-- ============================================================================
-- 選んだときの受け手（ChoiceReceiver）
-- ============================================================================

--- 推す ID（Reference3: recommend／portal、Reference4: 0・1・char の番号）。分からなければ nil
---@param ref table<integer, string>
---@return string|nil
local function preferred_id(ref)
    if ref[3] == "portal" then return "sakura.portalsites" end
    local owner = ref[3] == "recommend" and tostring(ref[4]):match("^%d+$")
    if owner == "0" then return "sakura.recommendsites" end
    if owner == "1" then return "kero.recommendsites" end
    return owner and ("char" .. owner .. ".recommendsites") or nil
end

--- 名前（Reference0）と URL（Reference1）が一致する項目か（区切り線は URL が空なので当たらない）
local function matches(item, ref)
    return item ~= nil and item.url ~= "" and item.name == ref[0] and item.url == ref[1]
end

--- 並びから名前と URL が一致する最初の項目
local function first_match(items, ref)
    for _, item in ipairs(items or {}) do
        if matches(item, ref) then return item end
    end
    return nil
end

--- 開いたときの並びから選ばれた項目を探す
---@param ref table<integer, string> act.req.reference
---@return RecommendItem|nil
local function find_chosen(ref)
    local items = last[preferred_id(ref) or ""]
    if items then
        -- Reference5 は手がかり（区切り線も 1 つと数える位置）。外れれば同じ並びを探す
        local hinted = items[(tonumber(ref[5]) or -1) + 1]
        if matches(hinted, ref) then return hinted end
        local item = first_match(items, ref)
        if item then return item end
    end
    -- 覚えているほかの並びを決まった順に探す（char* は ID の文字列の昇順）
    local chars = {}
    for id in pairs(last) do
        if id:match("^char") then chars[#chars + 1] = id end
    end
    table.sort(chars)
    local order = { "sakura.recommendsites", "kero.recommendsites", "sakura.portalsites" }
    for _, id in ipairs(chars) do order[#order + 1] = id end
    for _, id in ipairs(order) do
        local item = first_match(last[id], ref)
        if item then return item end
    end
    return nil
end

--- 選んだ項目のシーンを完全一致で引き、コルーチンを返す（EVENT.drive がふつうの割り込みの決まりで再開する）
REG.OnRecommendsiteChoice = REG.OnRecommendsiteChoice or function(act)
    local SCENE = require("pasta.scene")
    local item = find_chosen(act.req.reference or {})
    if not (item and item.scene) then
        -- EVENT.no_entry と同じ前方一致（無ければ nil で 204）
        return SCENE.co_exec(act, act.req.id)
    end
    local co = SCENE.co_exec_exact(item.scene)
    if co then return co end
    -- 書き間違いを ＊OnRecommendsiteChoice で隠さない
    local FAILURE = require("pasta.failure")
    FAILURE.report(act, {
        label = FAILURE.LABEL.SCENE_NOT_FOUND,
        subject = "＊" .. item.scene,
        warning = "OnRecommendsiteChoice: scene not found: name='" .. item.scene .. "'",
    })
    return act:build()
end
