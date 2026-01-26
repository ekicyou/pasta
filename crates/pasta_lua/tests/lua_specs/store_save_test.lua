-- Save persistence tests
-- Tests for store-save-persistence feature
-- Updated: STORE.save is deprecated, now using pasta.save via ctx.save
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Task 3.1: pasta.save module tests
describe("pasta.save - persistence module", function()
    test("pasta.save returns a table", function()
        local save = require("pasta.save")
        expect(type(save)):toBe("table")
    end)

    test("pasta.save can store and retrieve values", function()
        local save = require("pasta.save")
        save.test_key = "test_value"
        save.nested = { a = 1, b = 2 }
        expect(save.test_key):toBe("test_value")
        expect(save.nested.a):toBe(1)
        expect(save.nested.b):toBe(2)
    end)
end)

-- Task 3.2: pasta.store no longer has save field
describe("pasta.store - save field removed", function()
    test("STORE.save is nil (deprecated)", function()
        local STORE = require("pasta.store")
        expect(STORE.save):toBe(nil)
    end)

    test("STORE.reset() works without save field", function()
        local STORE = require("pasta.store")
        STORE.actors["test_actor"] = { name = "Test" }
        STORE.scenes["test_scene"] = { fn = function() end }
        STORE.reset()
        expect(next(STORE.actors)):toBe(nil)
        expect(next(STORE.scenes)):toBe(nil)
    end)
end)

-- Task 3.3: pasta.ctx uses pasta.save
describe("pasta.ctx - save from pasta.save", function()
    test("CTX.new() returns ctx with save from pasta.save", function()
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local ctx = CTX.new()
        expect(ctx.save):toBeTruthy()
        expect(ctx.save):toBe(SAVE) -- same reference as pasta.save
    end)

    test("CTX.new(actors) sets actors correctly while save references pasta.save", function()
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local actors = { hero = { name = "Hero" } }
        local ctx = CTX.new(actors)
        expect(ctx.save):toBe(SAVE)
        expect(ctx.actors):toBe(actors)
        expect(ctx.actors.hero.name):toBe("Hero")
    end)

    test("CTX.new() with nil actors sets empty actors table", function()
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local ctx = CTX.new(nil)
        expect(ctx.save):toBe(SAVE)
        expect(type(ctx.actors)):toBe("table")
        expect(next(ctx.actors)):toBe(nil)
    end)
end)

-- Reference identity tests
describe("pasta.save and ctx.save - reference identity", function()
    test("ctx.save changes reflect in pasta.save", function()
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local ctx = CTX.new()
        ctx.save.from_ctx = "ctx_value"
        expect(SAVE.from_ctx):toBe("ctx_value")
    end)

    test("pasta.save changes reflect in ctx.save", function()
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local ctx = CTX.new()
        SAVE.from_save = "save_value"
        expect(ctx.save.from_save):toBe("save_value")
    end)

    test("multiple CTX instances share the same pasta.save", function()
        local SAVE = require("pasta.save")
        local CTX = require("pasta.ctx")
        local ctx1 = CTX.new()
        local ctx2 = CTX.new()
        ctx1.save.shared = "shared_value"
        expect(ctx2.save.shared):toBe("shared_value")
        expect(SAVE.shared):toBe("shared_value")
        expect(ctx1.save):toBe(ctx2.save)
        expect(ctx1.save):toBe(SAVE)
    end)
end)
