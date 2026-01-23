-- Actor word dictionary runtime tests
-- Task 3.6: Runtime layer unit tests for actor-word-dictionary feature
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Test word.lua module with actor word registry
describe("word module - actor registry", function()
    test("create_actor returns word builder", function()
        local word = require("pasta.word")
        local builder = word.create_actor("テスト太郎", "test_key")
        expect(builder):toBeTruthy()
        expect(type(builder)):toBe("table")
        expect(type(builder.entry)):toBe("function") -- WordBuilder has entry method
    end)

    test("create_actor registers actor in registry", function()
        local word = require("pasta.word")
        word.create_actor("テスト花子", "挨拶"):entry("こんにちは")
        local retrieved = word.get_actor_words("テスト花子") -- actor_name, not key
        expect(retrieved):toBeTruthy()
        expect(retrieved["挨拶"]):toBeTruthy()
    end)

    test("get_actor_words returns nil for unknown actor", function()
        local word = require("pasta.word")
        local result = word.get_actor_words("unknown_actor_xyz")
        expect(result):toBe(nil)
    end)

    test("get_global_words returns GLOBAL table", function()
        local word = require("pasta.word")
        local global = word.get_global_words()
        expect(global):toBeTruthy()
        expect(type(global)):toBe("table")
    end)

    test("get_local_words returns nil for unknown scene", function()
        local word = require("pasta.word")
        local result = word.get_local_words("unknown_scene_xyz")
        expect(result):toBe(nil)
    end)
end)
