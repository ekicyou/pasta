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
        local act2 = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = 1702652400, hour = 15 },
        })
        local result = dispatcher.check_hour(act2)

        expect(type(result)):toBe("thread")
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
        local act2 = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = state.next_talk_time + 1 },
        })
        local result = dispatcher.check_talk(act2)

        expect(type(result)):toBe("thread")
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
        local act2 = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = 1702652400, hour = 15 },
        })
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

        dispatcher.check_talk(act)

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

    -- 共通手順: 指定 hour で時報を発火させ、最初に検索された候補名を返す
    -- （executor 差替→初期化→発火の実行順を厳密保存）
    local function first_candidate_for_hour(hour)
        local first_name = nil

        dispatcher._set_scene_executor(function(event_name)
            if not first_name then first_name = event_name end
            return coroutine.create(function() return "result" end)
        end)

        local act = { req = { id = "OnSecondChange", status = "idle", date = { unix = 0, hour = hour } } }
        dispatcher.check_hour(act)
        act.req.date.unix = 3600
        dispatcher.check_hour(act)

        return first_name
    end

    test("hour=0 generates candidate 時報00", function()
        setup()
        local first_name = first_candidate_for_hour(0)

        expect(first_name):toBe("時報00")
    end)

    test("hour=9 generates candidate 時報09", function()
        setup()
        local first_name = first_candidate_for_hour(9)

        expect(first_name):toBe("時報09")
    end)

    test("hour=12 generates candidate 時報12", function()
        setup()
        local first_name = first_candidate_for_hour(12)

        expect(first_name):toBe("時報12")
    end)

    test("hour=23 generates candidate 時報23", function()
        setup()
        local first_name = first_candidate_for_hour(23)

        expect(first_name):toBe("時報23")
    end)
end)

-- ============================================================================
-- ontalk-block-condition: dispatch() 経由のブロック条件テスト
-- Requirements: 3.1, 3.2, 3.3, 3.4
-- ============================================================================

describe("dispatch - status block conditions", function()
    local dispatcher

    local function setup()
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            return coroutine.create(function() return event_name .. "_result" end)
        end)
    end

    -- 共通手順: init_status で初期化 dispatch 後、fire_status で時報境界の dispatch 結果を返す
    -- （ブロック対象 status での初期化はタイマー未設定になるため、初期化 status は引数で分離）
    local function dispatch_at_next_hour(init_status, fire_status)
        local act1 = create_mock_act({ id = "OnSecondChange", status = init_status, date = { unix = 1702648800 } })
        dispatcher.dispatch(act1)
        local act2 = create_mock_act({
            id = "OnSecondChange", status = fire_status, date = { unix = 1702652400, hour = 15 },
        })
        return dispatcher.dispatch(act2)
    end

    -- ブロック対象 Status 9件（各キーワード単体）: dispatch() が nil を返す
    -- Requirements: 3.1

    test("blocks when status is 'talking'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "talking")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'choosing'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "choosing")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'online'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "online")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'opening(communicate)'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "opening(communicate)")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'passive'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "passive")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'induction'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "induction")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'timecritical'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "timecritical")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'nouserbreak'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "nouserbreak")
        expect(result):toBe(nil)
    end)

    test("blocks when status is 'minimizing'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "minimizing")
        expect(result):toBe(nil)
    end)

    -- 複合 Status 2件: dispatch() が nil を返す
    -- Requirements: 3.2

    test("blocks when compound status contains 'choosing'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "choosing,balloon(0=0)")
        expect(result):toBe(nil)
    end)

    test("blocks when compound status contains 'online'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "online,balloon(1=2)")
        expect(result):toBe(nil)
    end)

    -- 非ブロック Status 4件（初期化・発火とも同一 status）: dispatch() が nil 以外を返す
    -- Requirements: 3.3

    test("does not block when status is nil", function()
        setup()
        local result = dispatch_at_next_hour(nil, nil)
        expect(result ~= nil):toBe(true)
    end)

    test("does not block when status is empty string", function()
        setup()
        local result = dispatch_at_next_hour("", "")
        expect(result ~= nil):toBe(true)
    end)

    test("does not block when status is 'idle'", function()
        setup()
        local result = dispatch_at_next_hour("idle", "idle")
        expect(result ~= nil):toBe(true)
    end)

    test("does not block when status is 'balloon(0=0)'", function()
        setup()
        local result = dispatch_at_next_hour("balloon(0=0)", "balloon(0=0)")
        expect(result ~= nil):toBe(true)
    end)

    -- M.is_blocked() 直接呼び出し 3件
    -- Requirements: 3.4

    test("is_blocked returns true for 'talking'", function()
        setup()
        expect(dispatcher.is_blocked("talking")):toBe(true)
    end)

    test("is_blocked returns false for 'idle'", function()
        setup()
        expect(dispatcher.is_blocked("idle")):toBe(false)
    end)

    test("is_blocked returns false for nil", function()
        setup()
        expect(dispatcher.is_blocked(nil)):toBe(false)
    end)
end)

-- ============================================================================
-- tutorial-ssp-gaps 3.3: 時報の判定を仮想の時刻で行い、ずれが変わったら次の正時を決め直す
-- Requirements: 2.1, 2.3, 2.4, 2.5, 7.3
-- ============================================================================

describe("check_hour / check_talk - 仮想の時刻", function()
    local dispatcher
    local CLOCK = require("pasta.shiori.clock")

    -- 正時ちょうどの Unix 時刻（unix % 3600 == 0）。R0 はその 21 分後（「10:21」）
    local BASE = 497861 * 3600
    local R0 = BASE + 21 * 60
    local H = 3600

    local found -- 探した候補の名前

    local function mk_date(unix)
        local t = os.date("*t", unix)
        return {
            unix = unix, year = t.year, month = t.month, day = t.day,
            hour = t.hour, min = t.min, sec = t.sec, ns = 0,
            yday = t.yday, ordinal = t.yday, wday = t.wday - 1, num_days_from_sunday = t.wday - 1,
        }
    end

    --- OnSecondChange の act を作る。EVENT.fire と同じく、ハンドラより前に CLOCK.observe を通す
    local function mk_act(real, dic)
        local req = { id = "OnSecondChange", status = "idle", date = mk_date(real), dic = dic or {} }
        CLOCK.observe(req)
        return create_mock_act(req)
    end

    --- from から to まで（両端を含む）1 秒ずつ OnSecondChange を見せる（時が流れる）
    local function pass(from, to)
        for t = from, to do
            CLOCK.observe({ id = "OnSecondChange", date = mk_date(t), dic = {} })
        end
    end

    local function hh_of(unix)
        return string.format("%02d", os.date("*t", unix).hour)
    end

    --- 実際の時刻 real のとき、仮想の時刻が virtual になるようにずれを決める
    local function set_virtual(real, virtual)
        local t = os.date("*t", virtual)
        CLOCK.observe({
            id = "OnPastaClock",
            date = mk_date(real),
            dic = {},
            security_level = "local",
            reference = {
                [0] = tostring(t.year), [1] = tostring(t.month), [2] = tostring(t.day),
                [3] = tostring(t.hour), [4] = tostring(t.min), [5] = tostring(t.sec),
            },
        })
    end

    --- 時報の候補を探すと、"時報HH" の名前を found に残して thread を返す
    local function vtest(name, fn)
        test(name, function()
            dispatcher = require("pasta.shiori.event.virtual_dispatcher")
            dispatcher._reset()
            CLOCK._reset()
            found = nil
            dispatcher._set_scene_executor(function(event_name)
                if event_name:sub(1, #"時報") == "時報" and event_name ~= "時報その他" then
                    found = event_name
                    return coroutine.create(function() end)
                end
                if event_name == "OnTalk" then
                    return coroutine.create(function() end)
                end
                return nil
            end)
            local ok, err = pcall(fn)
            CLOCK._reset()
            dispatcher._reset()
            if not ok then error(err, 0) end
        end)
    end

    vtest("仮想の正時ちょうどにずれが変わったら、すぐ時報HHが出る", function()
        dispatcher.check_hour(mk_act(R0)) -- 初回: 実時間の 11:00 を記録
        -- 10:59:30 を入れて、仮想の 11:00:09 に気づいた
        local real = R0 + 39
        set_virtual(real, BASE + H + 9)
        expect(type(dispatcher.check_hour(mk_act(real)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(BASE + H))
        expect(dispatcher._get_internal_state().seen_epoch):toBe(CLOCK.epoch())
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + 2 * H)
    end)

    vtest("初回の判定より前の飛びでは、次の回に決め直さない（実際の正時の直後でも余分な時報が出ない）", function()
        CLOCK.observe({ id = "OnSecondChange", date = mk_date(BASE + H - 300), dic = {} })
        -- 初回の判定の前に飛び（+330 秒）が数えられ、初回は 11:00:30 に記録だけする
        expect(dispatcher.check_hour(mk_act(BASE + H + 30))):toBe(nil)
        expect(CLOCK.epoch()):toBe(1)
        expect(dispatcher._get_internal_state().seen_epoch):toBe(1)
        expect(dispatcher.check_hour(mk_act(BASE + H + 31))):toBe(nil)
        expect(found):toBe(nil)
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + 2 * H)
    end)

    vtest("正時をまたいで先へ設定し直しても、余分な時報は出ず、次の正時に出る", function()
        dispatcher.check_hour(mk_act(R0))
        set_virtual(R0, BASE + 2 * H - 30) -- 11:59:30
        expect(dispatcher.check_hour(mk_act(R0))):toBe(nil)
        expect(found):toBe(nil)
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + 2 * H)
        expect(dispatcher.check_hour(mk_act(R0 + 29))):toBe(nil)
        expect(type(dispatcher.check_hour(mk_act(R0 + 30)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(BASE + 2 * H))
    end)

    vtest("戻したら、戻った先の次の正時に出る", function()
        set_virtual(R0, BASE + 2 * H - 30)
        dispatcher.check_hour(mk_act(R0)) -- 初回
        dispatcher.check_hour(mk_act(R0 + 1)) -- ずれを見て決め直す
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + 2 * H)
        set_virtual(R0 + 2, R0 + 2) -- 実際の 10:21 に戻す（ずれ 0）
        expect(CLOCK.offset()):toBe(0)
        expect(dispatcher.check_hour(mk_act(R0 + 2))):toBe(nil)
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + H)
        expect(dispatcher._get_internal_state().seen_epoch):toBe(CLOCK.epoch())
        expect(dispatcher.check_hour(mk_act(BASE + H - 1))):toBe(nil)
        expect(type(dispatcher.check_hour(mk_act(BASE + H)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(BASE + H))
    end)

    vtest("ずれ 0 のまま OS の時計が 1 時間戻ると、戻った先の次の正時に出る（要件 7.3 の例外）", function()
        dispatcher.check_hour(mk_act(R0)) -- 10:21
        expect(dispatcher.check_hour(mk_act(R0 - H))):toBe(nil) -- 9:21 へ戻す
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE)
        pass(R0 - H + 1, BASE - 1)
        expect(type(dispatcher.check_hour(mk_act(BASE)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(BASE))
    end)

    vtest("時が流れて 2 時間進んでも 1 回だけで、HH は進んだ先（仮想）", function()
        local off = 5 * H
        set_virtual(R0, R0 + off)
        dispatcher.check_hour(mk_act(R0))
        dispatcher.check_hour(mk_act(R0 + 1))
        found = nil
        CLOCK.observe({ id = "OnSysResume", date = mk_date(R0 + 1), dic = {} }) -- スリープ明け
        local real = R0 + 2 * H + 9 * 60 -- 実際の 12:30・仮想の 17:30
        expect(type(dispatcher.check_hour(mk_act(real)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(real + off))
        found = nil
        expect(dispatcher.check_hour(mk_act(real + 60))):toBe(nil)
        expect(found):toBe(nil)
    end)

    vtest("実機の順番のスリープ明け（OnSysSuspend → 最初の秒 → OnSysResume）でも時報は 1 回出て、おしゃべりはすぐ出る", function()
        dispatcher.check_hour(mk_act(R0)) -- 初回: 11:00 を記録
        dispatcher.check_talk(mk_act(R0)) -- 初回: 次のおしゃべりを記録
        local talk_at = dispatcher._get_internal_state().next_talk_time
        local real = R0 + 2 * H -- 2 時間眠って 12:21
        CLOCK.observe({ id = "OnSysSuspend", date = mk_date(real), dic = {} }) -- 復帰してから届く
        expect(type(dispatcher.check_hour(mk_act(real)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(real))
        expect(talk_at < CLOCK.steady({ date = mk_date(real), dic = {} })):toBe(true)
        expect(type(dispatcher.check_talk(mk_act(real + 1)))):toBe("thread")
        CLOCK.observe({ id = "OnSysResume", date = mk_date(real + 4), dic = {} }) -- 4 秒遅れて届く
        found = nil
        expect(dispatcher.check_hour(mk_act(real + 5))):toBe(nil)
        expect(found):toBe(nil)
    end)

    vtest("hour_margin は仮想の正時までの残りで見る", function()
        dispatcher.check_hour(mk_act(R0))
        dispatcher.check_talk(mk_act(R0))
        local talk_at = dispatcher._get_internal_state().next_talk_time
        local real = R0 + 400 -- 実際は 10:27:40（おしゃべりの時刻は過ぎた）
        pass(R0 + 1, real - 1)
        set_virtual(real, BASE + H - 15) -- 仮想は 10:59:45
        expect(dispatcher.check_hour(mk_act(real))):toBe(nil)
        expect(dispatcher.check_talk(mk_act(real))):toBe(nil)
        expect(dispatcher._get_internal_state().next_talk_time):toBe(talk_at)
    end)

    vtest("next_talk_time はなめらかな時計で数える", function()
        set_virtual(R0, R0 + 5 * H)
        local act = mk_act(R0)
        dispatcher.check_talk(act)
        local t = dispatcher._get_internal_state().next_talk_time - CLOCK.steady(act.req)
        expect(t >= 180 and t <= 300):toBe(true)
    end)

    vtest("X-Pasta-Time 付きでは、ずれを無視する", function()
        local XPT = { ["X-Pasta-Time"] = "2026-10-18T10:21:00+09:00" }
        set_virtual(R0, BASE + H - 30) -- ずれ 38 分 30 秒
        dispatcher.check_hour(mk_act(R0, XPT))
        expect(dispatcher.check_hour(mk_act(R0 + 39, XPT))):toBe(nil)
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + H)
        expect(type(dispatcher.check_hour(mk_act(BASE + H, XPT)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(BASE + H))
    end)

    -- req.date だけが飛ぶ（ずれ 0。SSP 2.9.18 の TimeMachine が OS の時計まで書き換える場合。タスク 7.2 の実機）

    vtest("飛び: 14:00:24 から 16:59:29 へ入れ直すと、時報16は出ず、17:00 に時報17が出る", function()
        local T14 = BASE + 4 * H
        dispatcher.check_hour(mk_act(T14 + 24)) -- 次は 15:00
        expect(dispatcher.check_hour(mk_act(T14 + 3 * H - 31))):toBe(nil)
        expect(found):toBe(nil)
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(T14 + 3 * H)
        expect(dispatcher._get_internal_state().seen_epoch):toBe(CLOCK.epoch())
        pass(T14 + 3 * H - 30, T14 + 3 * H - 1)
        expect(type(dispatcher.check_hour(mk_act(T14 + 3 * H)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(T14 + 3 * H))
    end)

    vtest("飛び: 10:21:10 に 10:59:30 を入れると、30 秒後に時報11が出る", function()
        dispatcher.check_hour(mk_act(R0 + 10))
        expect(dispatcher.check_hour(mk_act(BASE + H - 30))):toBe(nil)
        expect(found):toBe(nil)
        pass(BASE + H - 29, BASE + H - 1)
        expect(type(dispatcher.check_hour(mk_act(BASE + H)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(BASE + H))
    end)

    vtest("飛び: 17:01 から 13:11 へ戻すと、時報は出ず、次は 14:00", function()
        local T13 = BASE + 3 * H
        dispatcher.check_hour(mk_act(T13 + 4 * H + 60)) -- 17:01。次は 18:00
        expect(dispatcher.check_hour(mk_act(T13 + 11 * 60))):toBe(nil) -- 13:11
        expect(found):toBe(nil)
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(T13 + H)
        pass(T13 + 11 * 60 + 1, T13 + H - 1)
        expect(type(dispatcher.check_hour(mk_act(T13 + H)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(T13 + H))
    end)

    vtest("スリープ明けに 2 時間進んでも、飛びにせず時報は 1 回、進んだ先の時刻で出る", function()
        dispatcher.check_hour(mk_act(R0))
        CLOCK.observe({ id = "OnSysResume", date = mk_date(R0 + 1), dic = {} })
        local real = R0 + 2 * H + 9 * 60
        expect(type(dispatcher.check_hour(mk_act(real)))):toBe("thread")
        expect(found):toBe("時報" .. hh_of(real))
        expect(CLOCK.epoch()):toBe(0)
        found = nil
        expect(dispatcher.check_hour(mk_act(real + 1))):toBe(nil)
        expect(found):toBe(nil)
    end)

    vtest("おしゃべりの間隔は、+3 時間の飛びの分だけずれない", function()
        dispatcher.check_talk(mk_act(R0))
        local n = dispatcher._get_internal_state().next_talk_time - R0
        pass(R0 + 1, R0 + 10)
        local jumped = R0 + 11 + 3 * H -- 1 秒たって +3 時間飛んだ
        expect(dispatcher.check_talk(mk_act(jumped))):toBe(nil)
        pass(jumped + 1, jumped + n - 12)
        expect(dispatcher.check_talk(mk_act(jumped + n - 11 - 1))):toBe(nil)
        expect(type(dispatcher.check_talk(mk_act(jumped + n - 11)))):toBe("thread")
    end)
end)
