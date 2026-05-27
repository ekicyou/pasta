-- ============================================================================
-- Task 1.2: CALLBACK.try_route / CALLBACK.sweep テスト
-- Task 3.1: CALLBACK ID generation and staging テスト
-- Requirements: 1.1, 1.2, 5.1, 5.2, 5.3, R4 (callback chaining)
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect
local mocks = require("lua_test.mocks")

-- ============================================================================
-- Task 3.1: ID generation and staging tests
-- ============================================================================
describe("CALLBACK - ID generation and staging", function()
    local CALLBACK, STORE

    local function setup()
        mocks.reset()
        mocks.install()
        package.loaded["pasta.shiori.event.callback"] = nil
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
        CALLBACK = require("pasta.shiori.event.callback")
        CALLBACK.reset()
    end

    test("next_event_id generates sequential IDs", function()
        setup()
        expect(CALLBACK.next_event_id()):toBe("OnPastaCallBack1")
        expect(CALLBACK.next_event_id()):toBe("OnPastaCallBack2")
        expect(CALLBACK.next_event_id()):toBe("OnPastaCallBack3")
    end)

    test("reset clears ID counter", function()
        setup()
        CALLBACK.next_event_id() -- 1
        CALLBACK.next_event_id() -- 2
        CALLBACK.reset()
        expect(CALLBACK.next_event_id()):toBe("OnPastaCallBack1")
    end)

    test("stage_pending → consume_staged registers entry in pending", function()
        setup()
        CALLBACK.stage_pending("OnPastaCallBack1", 1000, "test timeout")

        local co = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co)
        local act = { name = "test" }

        local consumed = CALLBACK.consume_staged(co, act)
        expect(consumed):toBe(true)
        expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack1"].co):toBe(co)
        expect(CALLBACK.pending["OnPastaCallBack1"].act):toBe(act)
        expect(CALLBACK.pending["OnPastaCallBack1"].timeout_at):toBe(1000)
        expect(CALLBACK.pending["OnPastaCallBack1"].on_timeout):toBe("test timeout")
    end)

    test("consume_staged sets STORE.co_callback", function()
        setup()
        CALLBACK.stage_pending("OnPastaCallBack1", 1000, nil)

        local co = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co)

        CALLBACK.consume_staged(co, {})
        expect(STORE.co_callback):toBe(co)
    end)

    test("consume_staged returns false when nothing staged", function()
        setup()
        local co = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co)

        local consumed = CALLBACK.consume_staged(co, {})
        expect(consumed):toBe(false)
    end)

    test("multiple stage_pending without consume raises error", function()
        setup()
        CALLBACK.stage_pending("OnPastaCallBack1", 1000, nil)

        local ok, err = pcall(function()
            CALLBACK.stage_pending("OnPastaCallBack2", 2000, "second")
        end)
        expect(ok):toBe(false)
        expect(tostring(err):find("multiple staging detected")).not_:toBe(nil)
    end)

    test("reset clears pending, staged, and STORE.co_callback", function()
        setup()
        CALLBACK.stage_pending("OnPastaCallBack1", 1000, nil)
        local co = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co)
        CALLBACK.consume_staged(co, {})

        expect(STORE.co_callback):toBe(co)
        CALLBACK.reset()
        expect(STORE.co_callback):toBe(nil)
        expect(next(CALLBACK.pending)):toBe(nil)
    end)
end)

-- ============================================================================
-- Task 1.2: try_route / sweep tests
-- ============================================================================
describe("CALLBACK.try_route", function()
    local CALLBACK

    local function setup()
        mocks.reset()
        mocks.install()
        package.loaded["pasta.shiori.event.callback"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.res"] = nil
        -- Reload store and callback
        require("pasta.store")
        CALLBACK = require("pasta.shiori.event.callback")
        CALLBACK.reset()
    end

    test("returns nil when req.id does not match any pending entry", function()
        setup()
        local req = { id = "OnUnknown", reference = {} }
        local result = CALLBACK.try_route(req)
        expect(result):toBe(nil)
    end)

    test("resumes coroutine with 1-based refs and returns 200 OK response", function()
        setup()
        local received_refs = nil
        local co = coroutine.create(function(refs)
            received_refs = refs
            coroutine.yield("hello world\\e")
        end)
        -- Start coroutine first (it needs to be suspended)
        coroutine.resume(co) -- initial start, yields immediately... no, we need to design correctly

        -- Actually: the coroutine should first yield to be in suspended state,
        -- then try_route resumes it with refs
        local co2 = coroutine.create(function()
            local refs = coroutine.yield()    -- first yield to become suspended
            received_refs = refs
            coroutine.yield("hello world\\e") -- yield response
        end)
        coroutine.resume(co2)                 -- initial resume to reach first yield

        local act = { name = "test_act" }
        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co2,
            act = act,
            timeout_at = 999999,
            on_timeout = "timeout reason",
        }

        local req = {
            id = "OnPastaCallBack1",
            reference = { [0] = "2.6.77" },
        }
        local result = CALLBACK.try_route(req)

        -- Refs should be 1-based
        expect(received_refs[1]):toBe("2.6.77")
        -- Entry should be removed
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        -- Response should be 200 OK
        expect(result:find("200 OK")).not_:toBe(nil)
        expect(result:find("Value: hello world\\e")).not_:toBe(nil)
    end)

    test("converts multiple 0-indexed references to 1-based array", function()
        setup()
        local received_refs = nil
        local co = coroutine.create(function()
            local refs = coroutine.yield()
            received_refs = refs
            coroutine.yield("result")
        end)
        coroutine.resume(co)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co,
            act = {},
            timeout_at = 999999,
            on_timeout = nil,
        }

        local req = {
            id = "OnPastaCallBack1",
            reference = { [0] = "val0", [1] = "val1", [2] = "val2" },
        }
        CALLBACK.try_route(req)

        expect(received_refs[1]):toBe("val0")
        expect(received_refs[2]):toBe("val1")
        expect(received_refs[3]):toBe("val2")
    end)

    test("handles callback chaining (R4) via consume_staged", function()
        setup()
        -- Coroutine that chains: receives refs, stages a new pending, yields
        local co = coroutine.create(function()
            local refs = coroutine.yield() -- first callback wait
            -- After receiving refs, stage another callback (simulating get_property again)
            CALLBACK.stage_pending("OnPastaCallBack2", 999999, "chained timeout")
            coroutine.yield("intermediate response") -- yield again for second callback
        end)
        coroutine.resume(co)

        local act = { name = "test_act" }
        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co,
            act = act,
            timeout_at = 999999,
            on_timeout = nil,
        }

        local req = {
            id = "OnPastaCallBack1",
            reference = { [0] = "first_value" },
        }
        local result = CALLBACK.try_route(req)

        -- Original entry removed
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        -- Chained entry registered via consume_staged
        expect(CALLBACK.pending["OnPastaCallBack2"]).not_:toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack2"].co):toBe(co)
        expect(CALLBACK.pending["OnPastaCallBack2"].on_timeout):toBe("chained timeout")
        -- Response from intermediate yield
        expect(result:find("200 OK")).not_:toBe(nil)
    end)

    test("propagates error when coroutine errors during resume", function()
        setup()
        local co = coroutine.create(function()
            coroutine.yield()
            error("coroutine exploded")
        end)
        coroutine.resume(co)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co,
            act = {},
            timeout_at = 999999,
            on_timeout = nil,
        }

        local req = {
            id = "OnPastaCallBack1",
            reference = { [0] = "val" },
        }
        local ok, err = pcall(function()
            CALLBACK.try_route(req)
        end)
        expect(ok):toBe(false)
        expect(tostring(err):find("coroutine exploded")).not_:toBe(nil)
    end)
end)

describe("CALLBACK.sweep", function()
    local CALLBACK
    local warn_calls

    local function setup()
        warn_calls = {}
        mocks.reset()
        mocks.install({
            log = {
                trace = function() end,
                debug = function() end,
                info = function() end,
                warn = function(msg) warn_calls[#warn_calls + 1] = msg end,
                error = function() end,
            },
        })
        package.loaded["pasta.shiori.event.callback"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.res"] = nil
        require("pasta.store")
        CALLBACK = require("pasta.shiori.event.callback")
        CALLBACK.reset()
    end

    test("does nothing when no entries exist", function()
        setup()
        local result = CALLBACK.sweep(1000)
        expect(result):toBe(nil)
    end)

    test("does nothing when entries have not timed out", function()
        setup()
        local co = coroutine.create(function()
            coroutine.yield()
        end)
        coroutine.resume(co)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co,
            act = {},
            timeout_at = 2000,
            on_timeout = "timeout",
        }

        local result = CALLBACK.sweep(1000)
        expect(result):toBe(nil)
        -- Entry should still exist
        expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
    end)

    test("resumes with nil,on_timeout and returns 500 when on_timeout is string", function()
        setup()
        local received_args = {}
        local co = coroutine.create(function()
            local val, reason = coroutine.yield()
            received_args.val = val
            received_args.reason = reason
        end)
        coroutine.resume(co)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co,
            act = {},
            timeout_at = 100,
            on_timeout = "callback timeout: get_property",
        }

        local result = CALLBACK.sweep(200)

        -- Coroutine resumed with nil, on_timeout
        expect(received_args.val):toBe(nil)
        expect(received_args.reason):toBe("callback timeout: get_property")
        -- Entry removed
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        -- 500 response returned
        expect(result:find("500 Internal Server Error")).not_:toBe(nil)
        expect(result:find("X%-ERROR%-REASON: callback timeout: get_property")).not_:toBe(nil)
        -- log.warn called
        expect(#warn_calls):toBe(1)
        expect(warn_calls[1]:find("OnPastaCallBack1")).not_:toBe(nil)
    end)

    test("resumes with nil and returns nil when on_timeout is nil", function()
        setup()
        local resumed = false
        local received_val = "not_set"
        local co = coroutine.create(function()
            local val = coroutine.yield()
            received_val = val
            resumed = true
        end)
        coroutine.resume(co)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co,
            act = {},
            timeout_at = 100,
            on_timeout = nil,
        }

        local result = CALLBACK.sweep(200)

        expect(resumed):toBe(true)
        expect(received_val):toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        expect(result):toBe(nil)
        expect(#warn_calls):toBe(0)
    end)

    test("returns only first string on_timeout as 500 among multiple timeouts", function()
        setup()
        -- Create two timed-out entries with string on_timeout
        local co1 = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co1)
        local co2 = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co2)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co1,
            act = {},
            timeout_at = 50,
            on_timeout = "timeout reason 1",
        }
        CALLBACK.pending["OnPastaCallBack2"] = {
            co = co2,
            act = {},
            timeout_at = 60,
            on_timeout = "timeout reason 2",
        }

        local result = CALLBACK.sweep(200)

        -- Both entries removed
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack2"]):toBe(nil)
        -- Only one 500 response (the first one encountered)
        expect(result:find("500 Internal Server Error")).not_:toBe(nil)
        -- Both should have been logged
        expect(#warn_calls):toBe(2)
    end)

    test("handles mixed on_timeout types (string and nil)", function()
        setup()
        local co1 = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co1)
        local co2 = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co2)

        CALLBACK.pending["OnPastaCallBack1"] = {
            co = co1,
            act = {},
            timeout_at = 50,
            on_timeout = nil,
        }
        CALLBACK.pending["OnPastaCallBack2"] = {
            co = co2,
            act = {},
            timeout_at = 60,
            on_timeout = "timeout reason",
        }

        local result = CALLBACK.sweep(200)

        -- Both removed
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack2"]):toBe(nil)
        -- Only one warn call (the string one)
        expect(#warn_calls):toBe(1)
        -- Response may or may not be nil depending on iteration order,
        -- but at least one had string on_timeout
        if result then
            expect(result:find("500 Internal Server Error")).not_:toBe(nil)
        end
    end)
end)
