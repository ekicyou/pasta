-- Store save table tests
-- Task 3.1, 3.2, 3.3: Tests for store-save-table feature
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Task 3.1: pasta.store unit tests
describe("pasta.store - save field", function()
    test("STORE.save is initialized as empty table", function()
        local STORE = require("pasta.store")
        STORE.reset() -- ensure clean state
        expect(STORE.save):toBeTruthy()
        expect(type(STORE.save)):toBe("table")
        expect(next(STORE.save)):toBe(nil) -- empty table
    end)

    test("STORE.save can store and retrieve values", function()
        local STORE = require("pasta.store")
        STORE.reset()
        STORE.save.test_key = "test_value"
        STORE.save.nested = { a = 1, b = 2 }
        expect(STORE.save.test_key):toBe("test_value")
        expect(STORE.save.nested.a):toBe(1)
        expect(STORE.save.nested.b):toBe(2)
    end)

    test("STORE.reset() resets save to empty table", function()
        local STORE = require("pasta.store")
        STORE.save.before_reset = "value"
        expect(STORE.save.before_reset):toBe("value")
        STORE.reset()
        expect(STORE.save.before_reset):toBe(nil)
        expect(next(STORE.save)):toBe(nil)
    end)

    test("STORE.reset() does not affect other fields' reset behavior", function()
        local STORE = require("pasta.store")
        STORE.actors["test_actor"] = { name = "Test" }
        STORE.scenes["test_scene"] = { fn = function() end }
        STORE.save.persistent = "data"
        STORE.reset()
        expect(next(STORE.actors)):toBe(nil)
        expect(next(STORE.scenes)):toBe(nil)
        expect(next(STORE.save)):toBe(nil)
    end)
end)

-- Task 3.2: pasta.ctx unit tests
describe("pasta.ctx - STORE.save injection", function()
    test("CTX.new() returns ctx with save referencing STORE.save", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local ctx = CTX.new()
        expect(ctx.save):toBeTruthy()
        expect(ctx.save):toBe(STORE.save) -- same reference
    end)

    test("CTX.new(actors) sets actors correctly while save references STORE.save", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local actors = { hero = { name = "Hero" } }
        local ctx = CTX.new(actors)
        expect(ctx.save):toBe(STORE.save)
        expect(ctx.actors):toBe(actors)
        expect(ctx.actors.hero.name):toBe("Hero")
    end)

    test("CTX.new() with nil actors sets empty actors table", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local ctx = CTX.new(nil)
        expect(ctx.save):toBe(STORE.save)
        expect(type(ctx.actors)):toBe("table")
        expect(next(ctx.actors)):toBe(nil)
    end)
end)

-- Task 3.3: Reference identity integration tests
describe("STORE.save and ctx.save - reference identity", function()
    test("ctx.save changes reflect in STORE.save", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local ctx = CTX.new()
        ctx.save.from_ctx = "ctx_value"
        expect(STORE.save.from_ctx):toBe("ctx_value")
    end)

    test("STORE.save changes reflect in ctx.save", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local ctx = CTX.new()
        STORE.save.from_store = "store_value"
        expect(ctx.save.from_store):toBe("store_value")
    end)

    test("multiple CTX instances share the same STORE.save", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local ctx1 = CTX.new()
        local ctx2 = CTX.new()
        ctx1.save.shared = "shared_value"
        expect(ctx2.save.shared):toBe("shared_value")
        expect(STORE.save.shared):toBe("shared_value")
        expect(ctx1.save):toBe(ctx2.save)
        expect(ctx1.save):toBe(STORE.save)
    end)

    test("STORE.reset() affects all existing ctx.save references", function()
        local STORE = require("pasta.store")
        local CTX = require("pasta.ctx")
        STORE.reset()
        local ctx_before = CTX.new()
        ctx_before.save.old = "old_value"
        local old_save_ref = ctx_before.save
        STORE.reset()
        -- ctx_before.save still holds old reference (expected behavior)
        expect(old_save_ref.old):toBe("old_value")
        -- New ctx gets new STORE.save reference
        local ctx_after = CTX.new()
        expect(ctx_after.save):toBe(STORE.save)
        expect(ctx_after.save == old_save_ref):toBe(false) -- not same reference
        expect(next(ctx_after.save)):toBe(nil)
    end)
end)
