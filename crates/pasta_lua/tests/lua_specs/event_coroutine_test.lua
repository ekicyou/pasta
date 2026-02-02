-- event_coroutine_test.lua
-- Lua-side BDD tests for EVENT.fire coroutine management
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- ============================================================================
-- Task 2.1: set_co_scene() ローカル関数のテスト
-- Task 2.2: EVENT.fire thread判定とresume処理
-- Task 2.3: EVENT.fire コルーチン状態管理
-- Requirements: 1.1-1.6, 2.1-2.8
-- ============================================================================

describe("EVENT.fire - コルーチン対応", function()
    local EVENT, REG, STORE, RES

    local function setup()
        -- パッケージキャッシュをクリア
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.event.boot"] = nil
        package.loaded["pasta.shiori.event.second_change"] = nil

        REG = require("pasta.shiori.event.register")
        STORE = require("pasta.store")
        RES = require("pasta.shiori.res")
        STORE.reset()

        -- 登録をクリア
        for k in pairs(REG) do
            REG[k] = nil
        end

        EVENT = require("pasta.shiori.event")
    end

    test("handler が thread を返した場合 resume が実行される", function()
        setup()
        local resumed = false
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                resumed = true
                return "test_result"
            end)
        end

        local req = { id = "TestEvent" }
        EVENT.fire(req)

        expect(resumed):toBe(true)
    end)

    test("threadがyieldした場合 STORE.co_scene が設定される", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                coroutine.yield("first_yield")
                return "final"
            end)
        end

        local req = { id = "TestEvent" }
        EVENT.fire(req)

        expect(type(STORE.co_scene)):toBe("thread")
        expect(coroutine.status(STORE.co_scene)):toBe("suspended")
    end)

    test("threadが正常終了した場合 STORE.co_scene が nil にクリアされる", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                return "completed"
            end)
        end

        local req = { id = "TestEvent" }
        EVENT.fire(req)

        expect(STORE.co_scene):toBe(nil)
    end)

    test("handler が string を返した場合 そのまま RES.ok() で返す", function()
        setup()
        REG.TestEvent = function(act)
            return "string_result"
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("string_result")):toBeTruthy()
    end)

    test("handler が nil を返した場合 RES.no_content() が返る", function()
        setup()
        REG.TestEvent = function(act)
            return nil
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("204 No Content")):toBeTruthy()
    end)

    test("yield値が RES.ok() で返される", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                coroutine.yield("yielded_value")
            end)
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("yielded_value")):toBeTruthy()
    end)

    test("coroutine.resume() エラー時に STORE.co_scene がクリアされる", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                error("intentional_error")
            end)
        end

        local req = { id = "TestEvent" }
        -- EVENT.fire はエラーを伝搬する
        local ok, err = pcall(function()
            EVENT.fire(req)
        end)

        expect(ok):toBe(false)
        expect(STORE.co_scene):toBe(nil)
    end)
end)

describe("EVENT.fire - 既存コルーチン置換", function()
    local EVENT, REG, STORE

    local function setup()
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.event.boot"] = nil
        package.loaded["pasta.shiori.event.second_change"] = nil

        REG = require("pasta.shiori.event.register")
        STORE = require("pasta.store")
        STORE.reset()

        for k in pairs(REG) do
            REG[k] = nil
        end

        EVENT = require("pasta.shiori.event")
    end

    test("新しいコルーチンが設定されると既存のコルーチンがcloseされる", function()
        setup()

        -- 既存のsuspendedコルーチンを設定
        local old_co = coroutine.create(function()
            coroutine.yield("old")
        end)
        coroutine.resume(old_co) -- suspended状態に
        STORE.co_scene = old_co

        -- 新しいthreadを返すハンドラ
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                coroutine.yield("new")
            end)
        end

        local req = { id = "TestEvent" }
        EVENT.fire(req)

        -- 旧コルーチンはcloseされてdead状態
        expect(coroutine.status(old_co)):toBe("dead")
        -- 新しいコルーチンがSTORE.co_sceneに設定されている
        expect(type(STORE.co_scene)):toBe("thread")
        expect(STORE.co_scene ~= old_co):toBe(true)
    end)

    test("同じコルーチンを再設定しようとしてもcloseしない", function()
        setup()

        local call_count = 0
        local the_co

        REG.TestEvent = function(act)
            the_co = coroutine.create(function(act)
                coroutine.yield("first")
                coroutine.yield("second")
            end)
            return the_co
        end

        -- 最初の呼び出し
        local req = { id = "TestEvent" }
        EVENT.fire(req)

        -- the_coがSTORE.co_sceneに設定されている
        expect(STORE.co_scene):toBe(the_co)
        expect(coroutine.status(the_co)):toBe("suspended")
    end)
end)

return true
