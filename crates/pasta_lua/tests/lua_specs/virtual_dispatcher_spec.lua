-- virtual_dispatcher_spec.lua
-- Lua-side BDD tests for pasta.shiori.event.virtual_dispatcher module
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("pasta.shiori.event.virtual_dispatcher", function()
    local dispatcher

    -- before each: reset module state
    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        -- Set up mock scene executor for testing
        dispatcher._set_scene_executor(function(event_name)
            if event_name == "OnHour" then
                return "hour_result"
            elseif event_name == "OnTalk" then
                return "talk_result"
            end
            return nil
        end)
    end

    test("module is loadable", function()
        setup()
        expect(type(dispatcher)):toBe("table")
    end)

    test("exports required functions", function()
        setup()
        expect(type(dispatcher.dispatch)):toBe("function")
        expect(type(dispatcher.check_hour)):toBe("function")
        expect(type(dispatcher.check_talk)):toBe("function")
        expect(type(dispatcher._reset)):toBe("function")
        expect(type(dispatcher._get_internal_state)):toBe("function")
        expect(type(dispatcher._set_scene_executor)):toBe("function")
    end)

    test("initial state is zeroed", function()
        setup()
        local state = dispatcher._get_internal_state()
        expect(state.next_hour_unix):toBe(0)
        expect(state.next_talk_time):toBe(0)
        expect(state.cached_config):toBe(nil)
    end)
end)

describe("dispatch function", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            return event_name .. "_result"
        end)
    end

    test("returns nil when req.date is missing", function()
        setup()
        local req = { id = "OnSecondChange", status = "idle" }
        local result = dispatcher.dispatch(req)
        expect(result):toBe(nil)
    end)

    test("returns nil when req.date is nil", function()
        setup()
        local req = { id = "OnSecondChange", status = "idle", date = nil }
        local result = dispatcher.dispatch(req)
        expect(result):toBe(nil)
    end)
end)

describe("check_hour function", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            if event_name == "OnHour" then
                return "hour_result"
            end
            return nil
        end)
    end

    test("first call initializes next_hour_unix and returns nil", function()
        setup()
        local req = {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 } -- 14:00:00
        }
        local result = dispatcher.check_hour(req)
        local state = dispatcher._get_internal_state()

        expect(result):toBe(nil)
        expect(state.next_hour_unix > 0):toBe(true)
    end)

    test("fires at hour boundary", function()
        setup()
        -- Initialize
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.check_hour(req1)

        -- At next hour
        local req2 = { id = "OnSecondChange", status = "idle", date = { unix = 1702652400 } }
        local result = dispatcher.check_hour(req2)

        expect(result):toBe("fired")
    end)

    test("skips when status is talking", function()
        setup()
        -- Initialize
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.check_hour(req1)

        -- At next hour but talking
        local req2 = { id = "OnSecondChange", status = "talking", date = { unix = 1702652400 } }
        local result = dispatcher.check_hour(req2)

        expect(result):toBe(nil)
    end)

    test("returns nil before hour boundary", function()
        setup()
        -- Initialize
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.check_hour(req1)

        -- Not yet at next hour
        local req2 = { id = "OnSecondChange", status = "idle", date = { unix = 1702649000 } }
        local result = dispatcher.check_hour(req2)

        expect(result):toBe(nil)
    end)
end)

describe("check_talk function", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            if event_name == "OnTalk" then
                return "talk_result"
            end
            return nil
        end)
    end

    test("first call initializes next_talk_time and returns nil", function()
        setup()
        local req = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        local result = dispatcher.check_talk(req)
        local state = dispatcher._get_internal_state()

        expect(result):toBe(nil)
        expect(state.next_talk_time > 0):toBe(true)
    end)

    test("fires after interval", function()
        setup()
        -- Initialize
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.dispatch(req1) -- Initialize both timers

        local state = dispatcher._get_internal_state()

        -- After interval
        local req2 = { id = "OnSecondChange", status = "idle", date = { unix = state.next_talk_time + 1 } }
        local result = dispatcher.check_talk(req2)

        expect(result):toBe("fired")
    end)

    test("skips when status is talking", function()
        setup()
        -- Initialize
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.dispatch(req1)

        local state = dispatcher._get_internal_state()

        -- After interval but talking
        local req2 = { id = "OnSecondChange", status = "talking", date = { unix = state.next_talk_time + 1 } }
        local result = dispatcher.check_talk(req2)

        expect(result):toBe(nil)
    end)

    test("skips before interval", function()
        setup()
        -- Initialize
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.dispatch(req1)

        -- Before interval
        local req2 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648810 } }
        local result = dispatcher.check_talk(req2)

        expect(result):toBe(nil)
    end)
end)

describe("priority and integration", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            return event_name .. "_result"
        end)
    end

    test("OnHour has priority over OnTalk", function()
        setup()
        -- Initialize both
        local req1 = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.dispatch(req1)

        -- At next hour (OnHour should fire, OnTalk should not)
        local req2 = { id = "OnSecondChange", status = "idle", date = { unix = 1702652400 } }
        local result = dispatcher.dispatch(req2)

        -- Should return "fired" from check_hour
        expect(result):toBe("fired")
    end)

    test("_reset clears all state", function()
        setup()
        -- Set some state
        local req = { id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } }
        dispatcher.dispatch(req)

        local state_before = dispatcher._get_internal_state()
        expect(state_before.next_hour_unix > 0):toBe(true)

        -- Reset
        dispatcher._reset()

        local state_after = dispatcher._get_internal_state()
        expect(state_after.next_hour_unix):toBe(0)
        expect(state_after.next_talk_time):toBe(0)
        expect(state_after.cached_config):toBe(nil)
    end)
end)
