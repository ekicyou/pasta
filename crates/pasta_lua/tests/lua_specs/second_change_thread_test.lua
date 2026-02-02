-- ============================================================================
-- Task 5: OnSecondChange ハンドラのthread橋渡しテスト
-- Requirements: 3.3
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("OnSecondChange handler - thread passthrough", function()
    local REG
    local dispatcher
    local SHIORI_ACT

    local function setup()
        -- Reset modules
        package.loaded["pasta.shiori.event.second_change"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.event.virtual_dispatcher"] = nil
        package.loaded["pasta.shiori.act"] = nil

        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        SHIORI_ACT = require("pasta.shiori.act")
        dispatcher._reset()

        -- Load second_change to register the handler
        require("pasta.shiori.event.second_change")
        REG = require("pasta.shiori.event.register")
    end

    test("returns thread when dispatcher returns thread", function()
        setup()

        -- Mock scene executor to return a thread
        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function() return "scene result" end)
        end)

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0, year = 2026, month = 2, day = 1, hour = 14, min = 0, sec = 0, wday = 0 },
        })

        -- Initialize hour check
        dispatcher.check_hour(act)

        -- Trigger OnHour by setting time to next hour
        act.req.date.unix = 3600
        act.req.date.hour = 15

        local result = REG.OnSecondChange(act)

        -- Handler should return thread directly
        expect(type(result)):toBe("thread")
    end)

    test("returns nil when dispatcher returns nil", function()
        setup()

        -- Mock scene executor that never returns a scene
        dispatcher._set_scene_executor(function(event_name, act)
            return nil
        end)

        -- Ensure STORE.co_scene is nil
        local STORE = require("pasta.store")
        STORE.co_scene = nil

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0, year = 2026, month = 2, day = 1, hour = 14, min = 0, sec = 0, wday = 0 },
        })

        -- Just check time without triggering
        dispatcher.check_hour(act)
        dispatcher.check_talk(act)

        -- Advance time slightly (not enough to trigger)
        act.req.date.unix = 1

        local result = REG.OnSecondChange(act)

        -- Handler should return nil (no event fired)
        expect(result):toBe(nil)
    end)

    test("passthrough preserves thread from dispatcher", function()
        setup()

        local expected_thread = coroutine.create(function() return "test value" end)

        -- Mock dispatch to return our specific thread
        local original_dispatch = dispatcher.dispatch
        dispatcher.dispatch = function(act)
            return expected_thread
        end

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 },
        })

        local result = REG.OnSecondChange(act)

        -- Should be the exact same thread object
        expect(result):toBe(expected_thread)

        -- Restore original
        dispatcher.dispatch = original_dispatch
    end)
end)
