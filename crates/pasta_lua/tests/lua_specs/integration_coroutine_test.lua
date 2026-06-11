-- ============================================================================
-- Task 7: 統合テスト - コルーチン実行とチェイントーク
-- Requirements: 6.1, 6.2, 6.3, 6.4, 8.1, 8.2, 8.3, 8.4, 8.5
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 共通セットアップ: 関連モジュールを package.loaded から一括リロードする（スイート分離規約）。
-- pasta.shiori.res は pasta.shiori.event のリロード時に内部 require で追従する（純モジュール）。
local function reload_modules()
    package.loaded["pasta.store"] = nil
    package.loaded["pasta.shiori.event"] = nil
    package.loaded["pasta.shiori.event.register"] = nil
    package.loaded["pasta.shiori.res"] = nil
    package.loaded["pasta.shiori.act"] = nil

    local STORE = require("pasta.store")
    local EVENT = require("pasta.shiori.event")
    local REG = require("pasta.shiori.event.register")

    -- Setup actors for SHIORI_ACT
    STORE.actors = { sakura = { name = "さくら", spot = "sakura" } }

    -- Ensure clean state
    STORE.co_scene = nil

    return EVENT, REG, STORE
end

-- ============================================================================
-- Task 7.1: E2Eテストでact:yield()とチェイントークを検証
-- ============================================================================

describe("Integration - act:yield() and chain talk", function()
    local EVENT
    local STORE
    local REG

    local function setup()
        EVENT, REG, STORE = reload_modules()
    end

    test("yield() returns value and sets STORE.co_scene", function()
        setup()

        -- Register a handler that yields using coroutine.yield directly
        REG.OnYieldTest = function(handler_act)
            return coroutine.create(function(act)
                coroutine.yield("First message\\e")
                return "Second message\\e"
            end)
        end

        -- Fire the event
        local response = EVENT.fire({ id = "OnYieldTest" })

        -- First response should contain "First message"
        expect(response:find("First message")).not_:toBe(nil)
        expect(response:find("200 OK")).not_:toBe(nil)

        -- STORE.co_scene should be set (suspended)
        expect(STORE.co_scene).not_:toBe(nil)
        expect(coroutine.status(STORE.co_scene)):toBe("suspended")
    end)

    test("chain talk continues suspended coroutine", function()
        setup()

        local call_count = 0

        -- Register a handler that yields
        REG.OnChainTest = function(handler_act)
            return coroutine.create(function(act)
                call_count = call_count + 1
                coroutine.yield("Message " .. call_count .. "\\e")
                call_count = call_count + 1
                return "Message " .. call_count .. "\\e"
            end)
        end

        -- First fire
        local response1 = EVENT.fire({ id = "OnChainTest" })
        expect(response1:find("Message 1")).not_:toBe(nil)
        expect(call_count):toBe(1)

        -- STORE.co_scene should be set
        local co = STORE.co_scene
        expect(co).not_:toBe(nil)
        expect(coroutine.status(co)):toBe("suspended")

        -- Create a new handler that returns the suspended coroutine
        -- (simulating what check_talk() does for chain talk)
        REG.OnChainContinue = function(handler_act)
            -- Return existing suspended coroutine for continuation
            return STORE.co_scene
        end

        -- Second fire should resume the coroutine
        local response2 = EVENT.fire({ id = "OnChainContinue" })
        expect(response2:find("Message 2")).not_:toBe(nil)
        expect(call_count):toBe(2)

        -- STORE.co_scene should be cleared after completion
        expect(STORE.co_scene):toBe(nil)
    end)

    test("completed coroutine clears STORE.co_scene", function()
        setup()

        -- Register a handler that completes without yielding
        REG.OnCompleteTest = function(handler_act)
            return coroutine.create(function(act)
                return "Only message\\e"
            end)
        end

        -- Fire the event
        local response = EVENT.fire({ id = "OnCompleteTest" })

        -- Response should contain message
        expect(response:find("Only message")).not_:toBe(nil)

        -- STORE.co_scene should be nil (coroutine completed)
        expect(STORE.co_scene):toBe(nil)
    end)

    test("multiple yields work correctly", function()
        setup()

        local step = 0

        REG.OnMultiYield = function(handler_act)
            return coroutine.create(function(act)
                step = 1
                coroutine.yield("Step 1\\e")
                step = 2
                coroutine.yield("Step 2\\e")
                step = 3
                return "Step 3\\e"
            end)
        end

        -- Simulate continuation by returning suspended coroutine
        REG.OnMultiContinue = function(handler_act)
            return STORE.co_scene
        end

        -- First fire
        EVENT.fire({ id = "OnMultiYield" })
        expect(step):toBe(1)
        expect(STORE.co_scene).not_:toBe(nil)

        -- Second fire (continue)
        EVENT.fire({ id = "OnMultiContinue" })
        expect(step):toBe(2)
        expect(STORE.co_scene).not_:toBe(nil)

        -- Third fire (complete)
        EVENT.fire({ id = "OnMultiContinue" })
        expect(step):toBe(3)
        expect(STORE.co_scene):toBe(nil)
    end)
end)

-- ============================================================================
-- Task 7.2: エラー処理の統合テスト
-- ============================================================================

describe("Integration - error handling", function()
    local EVENT
    local STORE
    local REG

    local function setup()
        EVENT, REG, STORE = reload_modules()
    end

    test("coroutine error clears STORE.co_scene", function()
        setup()

        REG.OnErrorTest = function(act)
            return coroutine.create(function()
                error("Test error")
            end)
        end

        -- Fire should propagate error
        local ok, err = pcall(function()
            EVENT.fire({ id = "OnErrorTest" })
        end)

        expect(ok):toBe(false)
        expect(tostring(err):find("Test error")).not_:toBe(nil)

        -- STORE.co_scene should be cleared
        expect(STORE.co_scene):toBe(nil)
    end)

    test("new coroutine closes existing suspended coroutine", function()
        setup()

        local old_co

        -- First handler that yields
        REG.OnFirst = function(act)
            old_co = coroutine.create(function()
                coroutine.yield("first")
            end)
            return old_co
        end

        -- Fire first to set up suspended coroutine
        EVENT.fire({ id = "OnFirst" })
        expect(STORE.co_scene):toBe(old_co)
        expect(coroutine.status(old_co)):toBe("suspended")

        -- Second handler that returns a NEW coroutine
        REG.OnSecond = function(act)
            return coroutine.create(function()
                return "second result"
            end)
        end

        -- Fire second - should close the old coroutine
        EVENT.fire({ id = "OnSecond" })

        if coroutine.close then
            -- Old coroutine should be dead (closed)
            expect(coroutine.status(old_co)):toBe("dead")
        else
            -- LuaJIT/Lua 5.1 does not provide coroutine.close().
            expect(coroutine.status(old_co)):toBe("suspended")
        end

        -- STORE.co_scene should be nil (new coroutine completed)
        expect(STORE.co_scene):toBe(nil)
    end)

    test("STORE.reset() closes suspended coroutine", function()
        setup()

        local co

        REG.OnResetTest = function(act)
            co = coroutine.create(function()
                coroutine.yield("message")
            end)
            return co
        end

        -- Fire to create suspended coroutine
        EVENT.fire({ id = "OnResetTest" })
        expect(STORE.co_scene):toBe(co)
        expect(coroutine.status(co)):toBe("suspended")

        -- Reset should close and clear
        STORE.reset()

        expect(STORE.co_scene):toBe(nil)
        if coroutine.close then
            expect(coroutine.status(co)):toBe("dead")
        else
            -- LuaJIT/Lua 5.1 does not provide coroutine.close().
            expect(coroutine.status(co)):toBe("suspended")
        end
    end)
end)
