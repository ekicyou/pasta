-- store_last_global_scene_test.lua
-- Lua-side BDD tests for STORE.last_global_scene field
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("STORE.last_global_scene - フィールド定義", function()
    local STORE

    local function setup()
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
    end

    test("last_global_scene フィールドが nil で初期化される", function()
        setup()
        expect(STORE.last_global_scene):toBe(nil)
    end)

    test("last_global_scene に文字列を設定・取得できる", function()
        setup()
        STORE.last_global_scene = "MyScene"
        expect(STORE.last_global_scene):toBe("MyScene")
    end)

    test("reset() で last_global_scene が nil にリセットされる", function()
        setup()
        STORE.last_global_scene = "SomeScene"
        STORE.reset()
        expect(STORE.last_global_scene):toBe(nil)
    end)
end)
