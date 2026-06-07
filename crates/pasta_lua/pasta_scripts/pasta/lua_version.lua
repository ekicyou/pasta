--- @module pasta.lua_version
--- 実行中ランタイム（標準Lua/LuaJIT）の種別＋版を単一整数で返す葉ユーティリティ。
---
--- 戻り値の規約:
---   1xy = 標準Lua x.y（例: Lua 5.4 → 154、5.5 → 155）
---   2xy = LuaJIT x.y（例: LuaJIT 2.1 → 221、2.0 → 220）
--- `>= 200` で LuaJIT 判定が可能。
---
--- ランタイム種別は `rawget(_G, "jit")` の有無で判定する。
--- `_VERSION` は LuaJIT 上でも "Lua 5.1" を返すため種別判定には使用しない（6.4）。
---
--- @usage
--- local lua_version = require("pasta.lua_version")
--- local v = lua_version.get()  -- 本 LuaJIT 2.1 ランタイムでは 221

--- @class pasta.lua_version
--- @field get fun(): integer   -- 1xy=標準Lua x.y / 2xy=LuaJIT x.y（例: 154, 221）
local M = {}

-- "5.4" のような "major.minor" 文字列から整数 major, minor を抽出する。
-- 抽出できない場合は nil を返す（呼び出し側で防御的既定値を採用）。
local function parse_major_minor(s)
    if type(s) ~= "string" then
        return nil, nil
    end
    local major, minor = string.match(s, "(%d+)%.(%d+)")
    if major == nil then
        return nil, nil
    end
    return tonumber(major), tonumber(minor)
end

-- LuaJIT のランタイム版を 2xy 整数で算出する（jit テーブル既知前提）。
local function luajit_version(jit)
    -- jit.version_num（例: 20100）からの抽出を優先する。
    local vn = jit.version_num
    if type(vn) == "number" then
        local major = math.floor(vn / 10000)
        local minor = math.floor((vn % 10000) / 100)
        return 200 + major * 10 + minor
    end

    -- フォールバック: "LuaJIT 2.1.x" のような version 文字列を解析する。
    local major, minor = parse_major_minor(jit.version)
    if major ~= nil then
        return 200 + major * 10 + minor
    end

    -- 判定不能時の防御的既定値（本ランタイムでは到達しない）。LuaJIT 2.1 を採用。
    return 221
end

-- 標準 Lua のランタイム版を 1xy 整数で算出する。
local function lua_version()
    -- _VERSION（例: "Lua 5.4"）を解析する。
    local major, minor = parse_major_minor(_VERSION)
    if major ~= nil then
        return 100 + major * 10 + minor
    end

    -- 判定不能時の防御的既定値（本ランタイムでは到達しない）。Lua 5.1 を採用。
    return 151
end

--- 実行中ランタイムを表す単一整数を返す。
---
--- 例外を送出せず常に整数を返す（6.5）。
---
--- @return integer 1xy=標準Lua / 2xy=LuaJIT
function M.get()
    local jit = rawget(_G, "jit")
    if jit ~= nil then
        return luajit_version(jit)
    end
    return lua_version()
end

return M
