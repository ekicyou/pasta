-- store_appearance_test.lua
-- STORE.appearance（外見状態のセッション常駐）フィールドテスト
-- actor-surface-restore Task 1.3 (Requirements: 4.1, 4.2, 4.3)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local KEYS = { "actors", "spots", "owners", "last_spots" }

--- 4 つの表だけを持ち、すべて空であることを検証する
local function expect_empty_appearance(appearance)
    expect(type(appearance)):toBe("table")
    local count = 0
    for key in pairs(appearance) do
        count = count + 1
        local known = false
        for _, k in ipairs(KEYS) do
            if k == key then known = true end
        end
        expect(known):toBe(true)
    end
    expect(count):toBe(#KEYS)
    for _, key in ipairs(KEYS) do
        expect(type(appearance[key])):toBe("table")
        expect(next(appearance[key])):toBe(nil)
    end
end

describe("STORE.appearance - 外見状態", function()
    local STORE

    local function setup()
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
    end

    test("ロード直後に 4 つの空表を持つ", function()
        setup()
        expect_empty_appearance(STORE.appearance)
    end)

    test("reset() 後に 4 つの空表を持つ", function()
        setup()
        STORE.reset()
        expect_empty_appearance(STORE.appearance)
    end)

    test("reset() が汚れた状態を同形の空状態へ置き換える", function()
        setup()
        STORE.appearance.actors["女の子"] = { surface = 5, dressup = { ["頭"] = { ["帽子"] = 1 } } }
        STORE.appearance.spots[0] = { surface = 5 }
        STORE.appearance.owners[0] = "女の子"
        STORE.appearance.last_spots["女の子"] = 0
        STORE.appearance.extra = {}
        STORE.reset()
        expect_empty_appearance(STORE.appearance)
    end)

    test("永続化データ(pasta.save)に外見状態を置かない", function()
        setup()
        local save = require("pasta.save")
        expect(save.appearance):toBe(nil)
        expect(STORE.save):toBe(nil)
    end)
end)
