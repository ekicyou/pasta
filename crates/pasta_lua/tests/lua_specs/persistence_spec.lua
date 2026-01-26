-- persistence_spec.lua
-- Lua-side persistence tests for store-save-persistence feature
-- Additional tests for @pasta_persistence low-level API
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("@pasta_persistence module", function()
    test("module is loadable", function()
        local p = require("@pasta_persistence")
        expect(type(p)):toBe("table")
        expect(type(p.load)):toBe("function")
        expect(type(p.save)):toBe("function")
    end)

    test("load returns a table", function()
        local p = require("@pasta_persistence")
        local data = p.load()
        expect(type(data)):toBe("table")
    end)

    test("save accepts a table and returns success", function()
        local p = require("@pasta_persistence")
        local ok, err = p.save({ test = true })
        expect(ok):toBe(true)
    end)

    test("save and load preserve data", function()
        local p = require("@pasta_persistence")

        local original = {
            string_val = "hello",
            number_val = 42,
            bool_val = true,
            nested = {
                a = 1,
                b = 2
            }
        }

        local ok, err = p.save(original)
        expect(ok):toBe(true)

        local loaded = p.load()
        expect(loaded.string_val):toBe("hello")
        expect(loaded.number_val):toBe(42)
        expect(loaded.bool_val):toBe(true)
        expect(loaded.nested.a):toBe(1)
        expect(loaded.nested.b):toBe(2)
    end)
end)

describe("pasta.save module (via @pasta_persistence)", function()
    test("pasta.save is loadable", function()
        local SAVE = require("pasta.save")
        expect(type(SAVE)):toBe("table")
    end)

    test("pasta.save persists data across requires", function()
        local save1 = require("pasta.save")
        save1.persist_test = "persist_value"

        local save2 = require("pasta.save")
        expect(save2.persist_test):toBe("persist_value")
    end)

    test("pasta.save supports nested tables", function()
        local SAVE = require("pasta.save")
        SAVE.player = {
            name = "Hero",
            stats = {
                hp = 100,
                mp = 50
            }
        }

        expect(SAVE.player.name):toBe("Hero")
        expect(SAVE.player.stats.hp):toBe(100)
        expect(SAVE.player.stats.mp):toBe(50)
    end)

    test("pasta.save supports arrays", function()
        local SAVE = require("pasta.save")
        SAVE.inventory = { "sword", "shield", "potion" }

        expect(#SAVE.inventory):toBe(3)
        expect(SAVE.inventory[1]):toBe("sword")
        expect(SAVE.inventory[2]):toBe("shield")
        expect(SAVE.inventory[3]):toBe("potion")
    end)
end)

describe("ctx.save integration with persistence", function()
    test("ctx.save references pasta.save", function()
        local CTX = require("pasta.ctx")
        local SAVE = require("pasta.save")
        local ctx = CTX.new()

        -- ctx.save should be the same table as pasta.save
        expect(ctx.save):toBe(SAVE)
    end)

    test("changes in ctx.save reflect in pasta.save", function()
        local CTX = require("pasta.ctx")
        local SAVE = require("pasta.save")
        local ctx = CTX.new()

        -- Modify via ctx.save
        ctx.save.ctx_persist_key = "ctx_persist_value"
        expect(SAVE.ctx_persist_key):toBe("ctx_persist_value")
    end)

    test("changes in pasta.save reflect in ctx.save", function()
        local CTX = require("pasta.ctx")
        local SAVE = require("pasta.save")
        local ctx = CTX.new()

        -- Modify via SAVE
        SAVE.save_persist_key = "save_persist_value"
        expect(ctx.save.save_persist_key):toBe("save_persist_value")
    end)
end)

describe("STORE.save deprecation", function()
    test("STORE.save is nil (removed)", function()
        local STORE = require("pasta.store")
        expect(STORE.save):toBe(nil)
    end)
end)
