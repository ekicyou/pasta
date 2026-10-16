---@module pasta.shiori.event.mouse
--- つつき・撫での既定ハンドラの部品
---
--- 候補の名前（M.candidates）: 種類・アクター名・部位から 1〜8 段の名前を上から順に並べる。
--- アクター名（M.resolve_actor）: スコープ番号（Reference3）から pasta.toml のアクター名を引く。

local STORE = require("pasta.store")

local M = {}

--- 種類ごとの接頭辞と英字
local PREFIX = {
    poke = { "つっつき反応", "OnPoke" },
    touch = { "さわり反応", "OnTouch" },
}

--- 候補の名前（1〜8 段のうち飛ばさなかったもの。上の段から順）
--- 部位は加工せずに連結する。部位が その他・Other でも特別扱いしない。
---@param kind "poke"|"touch"
---@param actor string|nil アクター名（決まらなければ nil）
---@param part string|nil 部位（空文字列・nil は「部位なし」）
---@return string[]
function M.candidates(kind, actor, part)
    local ja, en = PREFIX[kind][1], PREFIX[kind][2]
    if part == "" then part = nil end
    local list = {}
    local function add(a, b)
        list[#list + 1] = a
        list[#list + 1] = b
    end
    if actor and part then add(ja .. actor .. part, en .. actor .. part) end
    if actor then add(ja .. actor .. "その他", en .. actor .. "Other") end
    if part then add(ja .. part, en .. part) end
    add(ja .. "その他", en .. "Other")
    return list
end

--- スコープ番号からアクター名を引く
--- spot が 1 人ならその名前、0 人なら nil、2 人以上なら そのスポットで最後に話した人
--- （STORE.appearance.owners）が集めた中にいればその名前。spot は読み込み時の値を見る。
---@param scope_ref string|nil Reference3 の生の値
---@return string|nil
function M.resolve_actor(scope_ref)
    local scope = tonumber(scope_ref)
    if not scope then return nil end
    local ok, config = pcall(require, "@pasta_config")
    if not ok or type(config) ~= "table" or type(config.actor) ~= "table" then return nil end

    local found = {}
    for name, actor in pairs(config.actor) do
        if type(actor) == "table" and actor.spot == scope then
            found[#found + 1] = name
        end
    end
    if #found == 1 then return found[1] end

    local owners = STORE.appearance and STORE.appearance.owners
    local owner = owners and owners[scope]
    for _, name in ipairs(found) do
        if name == owner then return name end
    end
    return nil
end

return M
