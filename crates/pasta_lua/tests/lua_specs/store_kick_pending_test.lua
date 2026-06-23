-- store_kick_pending_test.lua
-- Lua-side BDD tests for STORE.kick_pending / STORE.kick_force fields
-- pasta-scene-kick Task 1.3 (Requirements: 3.1, 5.4, 5.5)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("STORE.kick_pending / STORE.kick_force - フィールド定義", function()
    local STORE

    local function setup()
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
    end

    test("kick_pending フィールドが nil で初期化される", function()
        setup()
        expect(STORE.kick_pending):toBe(nil)
    end)

    test("kick_force フィールドが false で初期化される", function()
        setup()
        expect(STORE.kick_force):toBe(false)
    end)

    test("kick_pending に文字列を設定・取得できる", function()
        setup()
        STORE.kick_pending = "intro"
        expect(STORE.kick_pending):toBe("intro")
    end)

    test("kick_force に true を設定・取得できる", function()
        setup()
        STORE.kick_force = true
        expect(STORE.kick_force):toBe(true)
    end)

    test("reset() で kick_pending が nil にリセットされる", function()
        setup()
        STORE.kick_pending = "intro"
        STORE.reset()
        expect(STORE.kick_pending):toBe(nil)
    end)

    test("reset() で kick_force が false にリセットされる", function()
        setup()
        STORE.kick_force = true
        STORE.reset()
        expect(STORE.kick_force):toBe(false)
    end)
end)
