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

-- ============================================================================
-- tutorial-ssp-gaps 3.2: 何も出さない毎秒の回に読み取りを返す
-- Requirements: 2.1, 3.2, 7.5
-- ============================================================================

describe("OnSecondChange handler - 仮想時刻の読み取り", function()
    local PROBE = "\\![get,property,OnPastaClock,system.year,system.month,system.day,"
        .. "system.hour,system.minute,system.second]\\e"

    --- 関連モジュールを読み直し、時計を初期状態に戻し、ディスパッチの結果を差し替える
    ---@param dispatch_result thread|nil
    local function setup(dispatch_result)
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.second_change"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.store"] = nil
        local REG = require("pasta.shiori.event.register")
        for k in pairs(REG) do
            REG[k] = nil
        end
        local STORE = require("pasta.store")
        STORE.reset()
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        local original_dispatch = dispatcher.dispatch
        dispatcher.dispatch = function() return dispatch_result end
        local CLOCK = require("pasta.shiori.clock")
        CLOCK._reset()
        require("pasta.shiori.event.second_change")
        local function teardown()
            dispatcher.dispatch = original_dispatch
            CLOCK._reset()
        end
        return REG, CLOCK, STORE, teardown
    end

    --- 時機から外れた毎分の合図で読み取りを予約する
    local function arm(CLOCK)
        CLOCK.observe({ id = "OnMinuteChange", date = { unix = 30 } })
        expect(CLOCK._get_state().probe_armed):toBe(true)
    end

    local function act_of(status)
        return { req = { id = "OnSecondChange", status = status, date = { unix = 1000 } } }
    end

    test("予約ありでディスパッチが何も出さない回は読み取りを 1 回だけ返す", function()
        local REG, CLOCK, _, teardown = setup(nil)
        arm(CLOCK)

        local first = REG.OnSecondChange(act_of("idle"))
        local second = REG.OnSecondChange(act_of("idle"))

        teardown()
        expect(first):toBe(PROBE)
        expect(second):toBe(nil)
    end)

    test("予約が無ければ何も返さない（仮想時刻なしの出方を変えない）", function()
        local REG, _, _, teardown = setup(nil)
        local result = REG.OnSecondChange(act_of("idle"))
        teardown()
        expect(result):toBe(nil)
    end)

    test("ブロック中の回は何も返さず予約が残る", function()
        local REG, CLOCK, _, teardown = setup(nil)
        arm(CLOCK)

        local result = REG.OnSecondChange(act_of("talking"))
        local armed = CLOCK._get_state().probe_armed

        teardown()
        expect(result):toBe(nil)
        expect(armed):toBe(true)
    end)

    test("シーンが出る回はシーンを返し予約が残る", function()
        local co = coroutine.create(function() return "scene" end)
        local REG, CLOCK, _, teardown = setup(co)
        arm(CLOCK)

        local result = REG.OnSecondChange(act_of("idle"))
        local armed = CLOCK._get_state().probe_armed

        teardown()
        expect(result):toBe(co)
        expect(armed):toBe(true)
    end)

    test("読み取りを返しても待っているチェイントークの続きは同じまま残る", function()
        local REG, CLOCK, STORE, teardown = setup(nil)
        local EVENT = require("pasta.shiori.event")
        local pending = coroutine.create(function() coroutine.yield("a"); return "b" end)
        coroutine.resume(pending)
        STORE.co_scene = pending
        arm(CLOCK)

        local res = EVENT.fire({ id = "OnSecondChange", status = "idle", date = { unix = 1000, ns = 0 } })

        teardown()
        expect(res:find("^SHIORI/3.0 200") ~= nil):toBe(true)
        expect(res:find("OnPastaClock", 1, true) ~= nil):toBe(true)
        expect(STORE.co_scene):toBe(pending)
        expect(coroutine.status(pending)):toBe("suspended")
    end)

    test("ゴーストが先に登録したハンドラを上書きしない", function()
        package.loaded["pasta.shiori.event.second_change"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        local REG = require("pasta.shiori.event.register")
        local custom = function() return "custom" end
        REG.OnSecondChange = custom
        require("pasta.shiori.event.second_change")
        expect(REG.OnSecondChange):toBe(custom)
        REG.OnSecondChange = nil
    end)
end)
