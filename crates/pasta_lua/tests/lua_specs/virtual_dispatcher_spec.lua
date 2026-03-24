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
            if event_name == "OnHourOther" then
                return coroutine.create(function() return "hour_result" end)
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
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702652400, hour = 15 } })
        local result = dispatcher.check_hour(act2)

        expect(type(result)):toBe("thread")
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
                return coroutine.create(function() return "talk_result" end)
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

        expect(type(result)):toBe("thread")
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
            return coroutine.create(function() return event_name .. "_result" end)
        end)
    end

    test("OnHour has priority over OnTalk", function()
        setup()
        -- Initialize both
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        -- At next hour (OnHour should fire, OnTalk should not)
        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702652400, hour = 15 } })
        local result = dispatcher.dispatch(act2)

        -- Should return thread from check_hour
        expect(type(result)):toBe("thread")
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
        -- 常にシーン実行成功を返すモック（thread返却）
        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function() return "scene_result" end)
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

        expect(type(result)):toBe("thread")
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

        expect(type(result)):toBe("thread")
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
            return coroutine.create(function() return "scene_result" end)
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

        expect(received_event_name):toBe("時報14")
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
            return coroutine.create(function() return "scene_result" end)
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
            return coroutine.create(function() return "scene_result" end)
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
        if type(result) == "thread" then
            expect(received_act):toBe(act)
        end
    end)
end)

-- ============================================================================
-- Task 3.1: フォールバック順序テスト
-- Requirements: 1.1
-- ============================================================================

describe("check_hour - fallback chain order", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
    end

    test("searches candidates in order: 時報{HH} → OnHour{HH} → 時報その他 → OnHourOther", function()
        setup()
        local called_names = {}

        -- すべての候補で nil を返し、呼び出し順序を記録
        dispatcher._set_scene_executor(function(event_name, act)
            table.insert(called_names, event_name)
            return nil
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 12 } } }
        -- Initialize
        dispatcher.check_hour(act)
        -- Fire
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        expect(#called_names):toBe(4)
        expect(called_names[1]):toBe("時報12")
        expect(called_names[2]):toBe("OnHour12")
        expect(called_names[3]):toBe("時報その他")
        expect(called_names[4]):toBe("OnHourOther")
    end)
end)

-- ============================================================================
-- Task 3.2: 早期打ち切りテスト
-- Requirements: 1.3
-- ============================================================================

describe("check_hour - early termination", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
    end

    test("stops at first matching candidate (candidate 1)", function()
        setup()
        local call_count = 0

        dispatcher._set_scene_executor(function(event_name, act)
            call_count = call_count + 1
            -- 候補1（時報12）でヒット
            return coroutine.create(function() return "scene_result" end)
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 12 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600

        local result = dispatcher.check_hour(act)

        expect(type(result)):toBe("thread")
        expect(call_count):toBe(1)
    end)

    test("stops at candidate 3 when candidates 1-2 miss", function()
        setup()
        local called_names = {}

        dispatcher._set_scene_executor(function(event_name, act)
            table.insert(called_names, event_name)
            if event_name == "時報その他" then
                return coroutine.create(function() return "scene_result" end)
            end
            return nil
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 9 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600

        local result = dispatcher.check_hour(act)

        expect(type(result)):toBe("thread")
        expect(#called_names):toBe(3)
        expect(called_names[3]):toBe("時報その他")
    end)
end)

-- ============================================================================
-- Task 3.3: 全候補未発見テスト
-- Requirements: 1.2
-- ============================================================================

describe("check_hour - all candidates miss", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
    end

    test("returns nil when no candidate matches", function()
        setup()

        dispatcher._set_scene_executor(function(event_name, act)
            return nil
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 12 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600

        local result = dispatcher.check_hour(act)

        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- Task 3.4: HH フォーマットテスト
-- Requirements: 2.1, 2.2
-- ============================================================================

describe("check_hour - HH format", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
    end

    test("hour=0 generates candidate 時報00", function()
        setup()
        local first_name = nil

        dispatcher._set_scene_executor(function(event_name, act)
            if not first_name then first_name = event_name end
            return coroutine.create(function() return "result" end)
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 0 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        expect(first_name):toBe("時報00")
    end)

    test("hour=9 generates candidate 時報09", function()
        setup()
        local first_name = nil

        dispatcher._set_scene_executor(function(event_name, act)
            if not first_name then first_name = event_name end
            return coroutine.create(function() return "result" end)
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 9 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        expect(first_name):toBe("時報09")
    end)

    test("hour=12 generates candidate 時報12", function()
        setup()
        local first_name = nil

        dispatcher._set_scene_executor(function(event_name, act)
            if not first_name then first_name = event_name end
            return coroutine.create(function() return "result" end)
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 12 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        expect(first_name):toBe("時報12")
    end)

    test("hour=23 generates candidate 時報23", function()
        setup()
        local first_name = nil

        dispatcher._set_scene_executor(function(event_name, act)
            if not first_name then first_name = event_name end
            return coroutine.create(function() return "result" end)
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = 23 } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        expect(first_name):toBe("時報23")
    end)
end)

-- ============================================================================
-- Task 4.1: choosing 状態での OnTalk スキップ・CSV 対応
-- Requirements: 4.1, 4.3, 4.4
-- ============================================================================

describe("check_talk - choosing/CSV status guard", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            if event_name == "OnTalk" then
                return coroutine.create(function() return "talk_result" end)
            end
            return nil
        end)
    end

    test("skips when status is 'choosing' (T1)", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        local state = dispatcher._get_internal_state()

        -- After interval but choosing
        local act2 = create_mock_act({ id = "OnSecondChange", status = "choosing", date = { unix = state.next_talk_time + 1 } })
        local result = dispatcher.check_talk(act2)

        expect(result):toBe(nil)
    end)

    test("skips when status is CSV 'talking,choosing,balloon(0=2)' (T3)", function()
        setup()
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        local state = dispatcher._get_internal_state()

        local act2 = create_mock_act({ id = "OnSecondChange", status = "talking,choosing,balloon(0=2)", date = { unix = state.next_talk_time + 1 } })
        local result = dispatcher.check_talk(act2)

        expect(result):toBe(nil)
    end)

    test("skips when status is CSV 'talking,balloon(0=0)' (T5)", function()
        setup()
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        local state = dispatcher._get_internal_state()

        local act2 = create_mock_act({ id = "OnSecondChange", status = "talking,balloon(0=0)", date = { unix = state.next_talk_time + 1 } })
        local result = dispatcher.check_talk(act2)

        expect(result):toBe(nil)
    end)

    test("does NOT skip when status is 'idle' (T7)", function()
        setup()
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        local state = dispatcher._get_internal_state()

        local act2 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = state.next_talk_time + 1 } })
        local result = dispatcher.check_talk(act2)

        expect(type(result)):toBe("thread")
    end)
end)

-- ============================================================================
-- Task 4.2: choosing 状態での OnHour スキップ
-- Requirements: 4.2, 4.3, 4.4
-- ============================================================================

describe("check_hour - choosing/CSV status guard", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            if event_name == "OnHourOther" then
                return coroutine.create(function() return "hour_result" end)
            end
            return nil
        end)
    end

    test("skips when status is 'choosing' (T2)", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        -- At next hour but choosing
        local act2 = create_mock_act({ id = "OnSecondChange", status = "choosing", date = { unix = 1702652400, hour = 15 } })
        local result = dispatcher.check_hour(act2)

        expect(result):toBe(nil)
    end)

    test("skips when status is CSV 'talking,choosing,balloon(0=2)' (T4)", function()
        setup()
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        local act2 = create_mock_act({ id = "OnSecondChange", status = "talking,choosing,balloon(0=2)", date = { unix = 1702652400, hour = 15 } })
        local result = dispatcher.check_hour(act2)

        expect(result):toBe(nil)
    end)

    test("skips when status is CSV 'talking,balloon(0=0)' (T6)", function()
        setup()
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        local act2 = create_mock_act({ id = "OnSecondChange", status = "talking,balloon(0=0)", date = { unix = 1702652400, hour = 15 } })
        local result = dispatcher.check_hour(act2)

        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- Task 4.3: タイマー非消費の検証
-- Requirements: 1.2, 2.2
-- ============================================================================

describe("choosing status - timer non-consumption", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            return coroutine.create(function() return event_name .. "_result" end)
        end)
    end

    test("check_talk does not consume next_talk_time when choosing (T8)", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)

        local state_before = dispatcher._get_internal_state()
        local saved_talk_time = state_before.next_talk_time

        -- Call with choosing status after interval
        local act2 = create_mock_act({ id = "OnSecondChange", status = "choosing", date = { unix = saved_talk_time + 1 } })
        dispatcher.check_talk(act2)

        local state_after = dispatcher._get_internal_state()

        expect(state_after.next_talk_time):toBe(saved_talk_time)
    end)

    test("check_hour does not consume next_hour_unix when choosing (T9)", function()
        setup()
        -- Initialize
        local act1 = create_mock_act({ id = "OnSecondChange", status = "idle", date = { unix = 1702648800 } })
        dispatcher.check_hour(act1)

        local state_before = dispatcher._get_internal_state()
        local saved_hour_unix = state_before.next_hour_unix

        -- Call with choosing status at next hour
        local act2 = create_mock_act({ id = "OnSecondChange", status = "choosing", date = { unix = saved_hour_unix + 1, hour = 15 } })
        dispatcher.check_hour(act2)

        local state_after = dispatcher._get_internal_state()

        expect(state_after.next_hour_unix):toBe(saved_hour_unix)
    end)
end)
