-- ============================================================================
-- Task 4: EVENT.no_entry のthread返却対応テスト
-- Requirements: 3.2
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("EVENT.no_entry thread return", function()
    local EVENT
    local SCENE
    local SHIORI_ACT
    local original_search

    local function setup()
        -- Reset modules
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.scene"] = nil
        package.loaded["pasta.shiori.act"] = nil

        EVENT = require("pasta.shiori.event")
        SCENE = require("pasta.scene")
        SHIORI_ACT = require("pasta.shiori.act")

        -- Store original search function
        original_search = SCENE.search
    end

    local function teardown()
        -- Restore original search function
        if SCENE and original_search then
            SCENE.search = original_search
        end
    end

    test("returns thread when scene function is found", function()
        setup()

        -- Mock SCENE.search to return a SceneSearchResult-like table
        -- (SCENE.search returns {func = scene_fn, global_name = ..., local_name = ...})
        local scene_called = false
        SCENE.search = function(event_name, arg1, arg2)
            return {
                func = function()
                    scene_called = true
                    return "scene result"
                end,
                global_name = "test",
                local_name = "__start__"
            }
        end

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, { id = "OnUnknownEvent" })

        local result = EVENT.no_entry(act)

        -- Should return a thread (coroutine)
        expect(type(result)):toBe("thread")

        -- Thread should not be executed yet
        expect(scene_called):toBe(false)

        -- Thread should be in suspended state (not started)
        expect(coroutine.status(result)):toBe("suspended")

        teardown()
    end)

    test("returns nil when scene function is not found", function()
        setup()

        -- Mock SCENE.search to return nil
        SCENE.search = function(event_name, arg1, arg2)
            return nil
        end

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, { id = "OnNotFoundEvent" })

        local result = EVENT.no_entry(act)

        -- Should return nil when no scene is found
        expect(result):toBe(nil)

        teardown()
    end)

    test("passes event_name to SCENE.search", function()
        setup()

        local searched_event_name = nil
        SCENE.search = function(event_name, arg1, arg2)
            searched_event_name = event_name
            return nil
        end

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, { id = "OnTestEvent" })

        EVENT.no_entry(act)

        expect(searched_event_name):toBe("OnTestEvent")

        teardown()
    end)

    test("returned thread produces scene result when resumed", function()
        setup()

        -- Mock SCENE.search to return a SceneSearchResult-like table
        SCENE.search = function(event_name, arg1, arg2)
            return {
                func = function()
                    return "expected scene result"
                end,
                global_name = "test",
                local_name = "__start__"
            }
        end

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, { id = "OnTestEvent" })

        local thread = EVENT.no_entry(act)

        -- Resume the thread
        local ok, result = coroutine.resume(thread)

        expect(ok):toBe(true)
        expect(result):toBe("expected scene result")

        teardown()
    end)

    test("returned thread can be resumed with act parameter", function()
        setup()

        local received_act = nil
        SCENE.search = function(event_name, arg1, arg2)
            return {
                func = function(act)
                    received_act = act
                    return "result"
                end,
                global_name = "test",
                local_name = "__start__"
            }
        end

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, { id = "OnTestEvent" })

        local thread = EVENT.no_entry(act)

        -- Resume the thread with act parameter (as EVENT.fire does)
        coroutine.resume(thread, act)

        expect(received_act):toBe(act)

        teardown()
    end)
end)
