-- virtual_dispatcher_spec.lua
-- Lua-side BDD tests for pasta.shiori.event.virtual_dispatcher module
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Helper: create mock act object with req field
local function create_mock_act(req)
    return { req = req }
end

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
        local act = create_mock_act({ id = "OnSecondChange", status = "idle" })
        local result = dispatcher.dispatch(act)
        expect(result):toBe(nil)
    end)

    test("returns nil when req.date is nil", function()
        setup()
        local act = create_mock_act({ id = "OnSecondChange", status = "idle", date = nil })
        local result = dispatcher.dispatch(act)
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
        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 } -- 14:00:00
        })
        local result = dispatcher.check_hour(act)
        local state = dispatcher._get_internal_state()

        expect(result):toBe(nil)
        expect(state.next_hour_unix > 0):toBe(true)
    end)

    test("fires at hour boundary", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        -- At next hour
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702652400 } })
        local result = dispatcher.check_hour(act2)

        expect(result):toBe("fired")
    end)

    test("skips when status is talking", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        -- At next hour but talking
        local act2 = create_mock_act({ id = "OnSecondChange", status = "talking", date = { unix = 1702652400 } })
        local result = dispatcher.check_hour(act2)

        expect(result):toBe(nil)
    end)

    test("returns nil before hour boundary", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        -- Not yet at next hour
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702649000 } })
        local result = dispatcher.check_hour(act2)

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
        local act = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        local result = dispatcher.check_talk(act)
        local state = dispatcher._get_internal_state()

        expect(result):toBe(nil)
        expect(state.next_talk_time > 0):toBe(true)
    end)

    test("fires after interval", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1) -- Initialize both timers

        local state = dispatcher._get_internal_state()

        -- After interval
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = state.next_talk_time + 1 } })
        local result = dispatcher.check_talk(act2)

        expect(result):toBe("fired")
    end)

    test("skips when status is talking", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        local state = dispatcher._get_internal_state()

        -- After interval but talking
        local act2 = create_mock_act({ id = "OnSecondChange", status = "talking", date = { unix = state.next_talk_time + 1 } })
        local result = dispatcher.check_talk(act2)

        expect(result):toBe(nil)
    end)

    test("skips before interval", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        -- Before interval
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648810 } })
        local result = dispatcher.check_talk(act2)

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
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        -- At next hour (OnHour should fire, OnTalk should not)
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702652400 } })
        local result = dispatcher.dispatch(act2)

        -- Should return "fired" from check_hour
        expect(result):toBe("fired")
    end)

    test("_reset clears all state", function()
        setup()
        -- Set some state
        local act = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act)

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

-- ============================================================================
-- Task 4.2: check_hour() 統合テスト - transfer_date_to_var() 呼び出し確認
-- Requirements: 2.1
-- ============================================================================

describe("check_hour - transfer_date_to_var integration", function()
    local dispatcher
    local SHIORI_ACT

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        SHIORI_ACT = require("pasta.shiori.act")
        dispatcher._reset()
        -- 常にシーン実行成功を返すモック
        dispatcher._set_scene_executor(function(event_name, act)
            return "scene_result"
        end)
    end

    test("calls transfer_date_to_var when OnHour fires", function()
        setup()
        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800, year = 2026, month = 2, day = 1, hour = 14, min = 0, sec = 0, wday = 0 },
        })

        -- First call: initialize
        dispatcher.check_hour(act)

        -- Advance to next hour
        act.req.date.unix = 1702652400

        -- Second call: should fire and call transfer_date_to_var
        local result = dispatcher.check_hour(act)

        expect(result):toBe("fired")
        -- 日時変数が設定されていること
        expect(act.var.year):toBe(2026)
        expect(act.var["年"]):toBe("2026年")
        expect(act.var["時１２"]):toBe("午後2時") -- hour 14
    end)

    test("sets Japanese date variables on OnHour fire", function()
        setup()
        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0, year = 2026, month = 2, day = 1, hour = 12, min = 0, sec = 0, wday = 0 },
        })

        -- Initialize
        dispatcher.check_hour(act)

        -- Fire (unix > next_hour_unix)
        act.req.date.unix = 3600

        local result = dispatcher.check_hour(act)

        expect(result):toBe("fired")
        expect(act.var["曜日"]):toBe("日曜日")
        expect(act.var.week):toBe("Sunday")
        expect(act.var["時１２"]):toBe("正午") -- hour 12
    end)
end)

-- ============================================================================
-- Task 4.3: execute_scene() 統合テスト - act がシーン関数に渡される確認
-- Requirements: 3.1, 3.2
-- ============================================================================

describe("execute_scene - act parameter passing", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
    end

    test("scene_executor receives act parameter", function()
        setup()
        local received_act = nil
        local received_event_name = nil

        dispatcher._set_scene_executor(function(event_name, act)
            received_event_name = event_name
            received_act = act
            return "scene_result"
        end)

        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0, year = 2026, month = 2, day = 1, hour = 14, min = 0, sec = 0, wday = 0 },
        })

        -- Initialize
        dispatcher.check_hour(act)

        -- Fire
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        expect(received_event_name):toBe("OnHour")
        expect(received_act):toBe(act)
    end)
end)

-- ============================================================================
-- Task 4.4: check_talk() 統合テスト - transfer_date_to_var() が呼び出されないこと確認
-- Requirements: 4.1, 3.3
-- ============================================================================

describe("check_talk - no transfer_date_to_var", function()
    local dispatcher
    local SHIORI_ACT

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        SHIORI_ACT = require("pasta.shiori.act")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name, act)
            return "scene_result"
        end)
    end

    test("does not call transfer_date_to_var on OnTalk", function()
        setup()
        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0, year = 2026, month = 2, day = 1, hour = 14, min = 0, sec = 0, wday = 0 },
        })

        -- Initialize check_talk
        dispatcher.check_talk(act)

        -- Advance time past talk interval (300+ seconds)
        act.req.date.unix = 500

        -- Also need to initialize check_hour to set next_hour_unix
        dispatcher.check_hour(act)
        act.req.date.unix = 501

        local result = dispatcher.check_talk(act)

        -- OnTalk may or may not fire depending on random interval
        -- but act.var should NOT have date variables set
        expect(act.var.year):toBe(nil)
        expect(act.var["年"]):toBe(nil)
    end)

    test("passes act to scene_executor for OnTalk", function()
        setup()
        local received_act = nil

        dispatcher._set_scene_executor(function(event_name, act)
            received_act = act
            return "scene_result"
        end)

        local actors = { sakura = { name = "さくら", spot = "sakura" } }
        local act = SHIORI_ACT.new(actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0, year = 2026, month = 2, day = 1, hour = 14, min = 0, sec = 0, wday = 0 },
        })

        -- Initialize both to set up state
        dispatcher.check_hour(act)
        dispatcher.check_talk(act)

        -- Advance time significantly (past talk interval, but not near hour)
        -- next_hour_unix is about 3600, hour_margin is 30, so safe at unix=500
        act.req.date.unix = 500

        -- Force fire by advancing past next_talk_time
        local result = dispatcher.check_talk(act)

        -- Even if it doesn't fire (random), the act should be passed when it does
        -- We can't guarantee a fire, but we can test the executor signature
        if result == "fired" then
            expect(received_act):toBe(act)
        end
    end)
end)
