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

-- Task 3.3: pasta.act uses pasta.save
describe("pasta.act - save from pasta.save", function()
    test("ACT.new() returns act with save from pasta.save", function()
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local act = ACT.new({})
        expect(act.save):toBeTruthy()
        expect(act.save):toBe(SAVE) -- same reference as pasta.save
    end)

    test("ACT.new(actors) sets actors correctly while save references pasta.save", function()
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local actors = { hero = { name = "Hero" } }
        local act = ACT.new(actors)
        expect(act.save):toBe(SAVE)
        expect(act.actors):toBe(actors)
        expect(act.actors.hero.name):toBe("Hero")
    end)

    test("ACT.new() with nil actors sets empty actors table", function()
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local act = ACT.new(nil)
        expect(act.save):toBe(SAVE)
        expect(type(act.actors)):toBe("table")
        expect(next(act.actors)):toBe(nil)
    end)
end)

-- Reference identity tests
describe("pasta.save and act.save - reference identity", function()
    test("act.save changes reflect in pasta.save", function()
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local act = ACT.new({})
        act.save.from_act = "act_value"
        expect(SAVE.from_act):toBe("act_value")
    end)

    test("pasta.save changes reflect in act.save", function()
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local act = ACT.new({})
        SAVE.from_save = "save_value"
        expect(act.save.from_save):toBe("save_value")
    end)

    test("multiple ACT instances share the same pasta.save", function()
        local SAVE = require("pasta.save")
        local ACT = require("pasta.act")
        local act1 = ACT.new({})
        local act2 = ACT.new({})
        act1.save.shared = "shared_value"
        expect(act2.save.shared):toBe("shared_value")
        expect(SAVE.shared):toBe("shared_value")
        expect(act1.save):toBe(act2.save)
        expect(act1.save):toBe(SAVE)
    end)
end)
