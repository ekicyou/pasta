-- ============================================================================
-- EVENT.fire の入口で合図と返事を時計に見せるフックのテスト (tutorial-ssp-gaps 3.1)
-- Requirements: 2.1, 2.6, 2.9, 7.5
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 関連モジュールを読み直し、REG を空にし、時計を初期状態に戻す
local function reload()
    package.loaded["pasta.shiori.event"] = nil
    package.loaded["pasta.shiori.event.callback"] = nil
    package.loaded["pasta.shiori.event.register"] = nil
    package.loaded["pasta.store"] = nil
    package.loaded["pasta.shiori.event.boot"] = nil
    package.loaded["pasta.shiori.event.second_change"] = nil

    local REG = require("pasta.shiori.event.register")
    local STORE = require("pasta.store")
    STORE.reset()
    for k in pairs(REG) do
        REG[k] = nil
    end
    local EVENT = require("pasta.shiori.event")
    local CLOCK = require("pasta.shiori.clock")
    CLOCK._reset()
    return EVENT, REG, CLOCK
end

--- 秒単位の地方時から req.date を作る
local function date_of(unix)
    return { unix = unix, ns = 0 }
end

describe("EVENT.fire - 時計のフック", function()
    test("OnPastaClock は 204 を返し、ゴーストのハンドラを呼ばずにずれを決める", function()
        local EVENT, REG, CLOCK = reload()
        local called = false
        REG.OnPastaClock = function() called = true; return "x" end

        local os_unix = os.time({ year = 2026, month = 10, day = 18, hour = 12, min = 0, sec = 0 })
        local res = EVENT.fire({
            id = "OnPastaClock",
            date = date_of(os_unix),
            reference = { [0] = "2026", "10", "18", "13", "0", "0" },
        })

        expect(res:find("^SHIORI/3.0 204") ~= nil):toBe(true)
        expect(called):toBe(false)
        expect(CLOCK.offset()):toBe(3600)
        CLOCK._reset()
    end)

    test("読めない返事でもエラーにせず 204 を返し、ずれは 0", function()
        local EVENT, REG, CLOCK = reload()
        local called = false
        REG.OnPastaClock = function() called = true end

        local res = EVENT.fire({
            id = "OnPastaClock",
            date = date_of(os.time()),
            reference = { [0] = "abc" },
        })

        expect(res:find("^SHIORI/3.0 204") ~= nil):toBe(true)
        expect(called):toBe(false)
        expect(CLOCK.offset()):toBe(0)
        CLOCK._reset()
    end)

    test("時機から外れた OnMinuteChange は読み取りを予約し、ゴーストのハンドラへも進む", function()
        local EVENT, REG, CLOCK = reload()
        local called = false
        REG.OnMinuteChange = function() called = true; return "minute" end

        local unix = os.time({ year = 2026, month = 10, day = 18, hour = 12, min = 0, sec = 30 })
        local res = EVENT.fire({ id = "OnMinuteChange", date = date_of(unix) })

        expect(called):toBe(true)
        expect(res:find("minute") ~= nil):toBe(true)
        expect(CLOCK._get_state().probe_armed):toBe(true)
        CLOCK._reset()
    end)

    test("時機から外れた OnHourTimeSignal もハンドラ未登録ならシーンのフォールバックへ進む", function()
        local EVENT, _, CLOCK = reload()
        local seen = nil
        local orig = EVENT.no_entry
        EVENT.no_entry = function(act) seen = act.req.id; return nil end

        local unix = os.time({ year = 2026, month = 10, day = 18, hour = 12, min = 30, sec = 0 })
        local res = EVENT.fire({ id = "OnHourTimeSignal", date = date_of(unix) })

        EVENT.no_entry = orig
        expect(seen):toBe("OnHourTimeSignal")
        expect(res:find("^SHIORI/3.0 204") ~= nil):toBe(true)
        expect(CLOCK._get_state().probe_armed):toBe(true)
        CLOCK._reset()
    end)

    test("外から来た OnPastaClock は受け取らずゴーストのハンドラへ進む", function()
        local EVENT, REG, CLOCK = reload()
        local called = false
        REG.OnPastaClock = function() called = true; return "ext" end

        EVENT.fire({
            id = "OnPastaClock",
            security_level = "external",
            date = date_of(os.time()),
            reference = { [0] = "2026", "10", "18", "13", "0", "0" },
        })

        expect(called):toBe(true)
        expect(CLOCK.offset()):toBe(0)
        CLOCK._reset()
    end)
end)
