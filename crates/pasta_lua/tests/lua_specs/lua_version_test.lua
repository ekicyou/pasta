-- pasta.lua_version module tests
-- Tests for pasta.lua_version - 実行中ランタイム種別＋版を単一整数で返す
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local lua_version = require("pasta.lua_version")

-- ============================================================================
-- Requirement 6: Lua ランタイムバージョンの数値取得
-- ============================================================================

describe("pasta.lua_version - get()", function()
    test("get() が整数を返す (6.1, 6.5)", function()
        local v = lua_version.get()
        expect(type(v)):toBe("number")
        -- 整数であること（小数部なし）
        expect(v == math.floor(v)):toBe(true)
    end)

    test("本 LuaJIT 2.1 ランタイムで get() == 221 (6.2)", function()
        expect(lua_version.get()):toBe(221)
    end)

    test("get() >= 200（LuaJIT 判定, 6.4）", function()
        expect(lua_version.get()):toBeGraterThanOrEqual(200)
    end)
end)
