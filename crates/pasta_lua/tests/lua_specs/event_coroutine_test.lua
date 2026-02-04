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

-- ============================================================================
-- Task 3.1: resume_until_valid() 単体テスト
-- Requirements: 1.1, 1.2, 1.3
-- ============================================================================

describe("resume_until_valid - nil yieldスキップループ", function()
    local EVENT, STORE

    local function setup()
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.event.boot"] = nil
        package.loaded["pasta.shiori.event.second_change"] = nil

        STORE = require("pasta.store")
        STORE.reset()

        EVENT = require("pasta.shiori.event")
    end

    test("nil yieldしてsuspendedのコルーチンが再resumeされる (1.1)", function()
        setup()
        local resume_count = 0
        local co = coroutine.create(function()
            resume_count = resume_count + 1
            coroutine.yield(nil) -- nil yield (should continue)
            resume_count = resume_count + 1
            coroutine.yield(nil) -- nil yield again
            resume_count = resume_count + 1
            return "valid_result"
        end)

        local ok, value = EVENT._resume_until_valid(co)

        expect(ok):toBe(true)
        expect(value):toBe("valid_result")
        expect(resume_count):toBe(3)
    end)

    test("有効値（nil以外）を返したらループ終了 (1.2)", function()
        setup()
        local resume_count = 0
        local co = coroutine.create(function()
            resume_count = resume_count + 1
            return "immediate_result"
        end)

        local ok, value = EVENT._resume_until_valid(co)

        expect(ok):toBe(true)
        expect(value):toBe("immediate_result")
        expect(resume_count):toBe(1)
    end)

    test("dead状態でnilを返したらループ終了（空シーン） (1.2)", function()
        setup()
        local resume_count = 0
        local co = coroutine.create(function()
            resume_count = resume_count + 1
            -- 終了時にnilを返す（空シーン）
            return nil
        end)

        local ok, value = EVENT._resume_until_valid(co)

        expect(ok):toBe(true)
        expect(value):toBe(nil) -- dead状態のnilは有効値
        expect(resume_count):toBe(1)
        expect(coroutine.status(co)):toBe("dead")
    end)

    test("エラー発生時にok=false、エラーメッセージを返す (1.3)", function()
        setup()
        local co = coroutine.create(function()
            error("intentional_error")
        end)

        local ok, value = EVENT._resume_until_valid(co)

        expect(ok):toBe(false)
        expect(type(value)):toBe("string")
        expect(value:find("intentional_error")):toBeTruthy()
    end)

    test("初回resume引数がコルーチンに渡される", function()
        setup()
        local received_arg = nil
        local co = coroutine.create(function(arg)
            received_arg = arg
            return "done"
        end)

        local ok, value = EVENT._resume_until_valid(co, "test_arg")

        expect(ok):toBe(true)
        expect(received_arg):toBe("test_arg")
    end)

    test("2回目以降のresumeは引数なしで呼ばれる", function()
        setup()
        local args_received = {}
        local co = coroutine.create(function(first_arg)
            table.insert(args_received, first_arg or "nil")
            local second_arg = coroutine.yield(nil) -- nil yield
            table.insert(args_received, second_arg or "nil")
            return "done"
        end)

        local ok, value = EVENT._resume_until_valid(co, "first")

        expect(ok):toBe(true)
        expect(args_received[1]):toBe("first")
        expect(args_received[2]):toBe("nil") -- 2回目は引数なし
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

-- ============================================================================
-- Task 3.2: EVENT.fire統合テスト（nil yieldスキップ対応）
-- Requirements: 2.1, 2.4, 3.1, 3.2
-- ============================================================================

describe("EVENT.fire - nil yieldスキップ統合", function()
    local EVENT, REG, STORE, RES

    local function setup()
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

        for k in pairs(REG) do
            REG[k] = nil
        end

        EVENT = require("pasta.shiori.event")
    end

    test("nil yieldするthreadがresume_until_validで処理される (2.1)", function()
        setup()
        local resume_count = 0
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                resume_count = resume_count + 1
                coroutine.yield(nil)            -- スキップされる
                resume_count = resume_count + 1
                coroutine.yield("valid_result") -- これが返る
            end)
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(resume_count):toBe(2)
        expect(result:find("valid_result")):toBeTruthy()
    end)

    test("有効値を返すthreadハンドラが正常動作 (2.1)", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                return "immediate_result"
            end)
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("immediate_result")):toBeTruthy()
    end)

    test("エラー発生時にset_co_sceneでcloseされる (3.1, 3.2)", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                error("test_error")
            end)
        end

        local req = { id = "TestEvent" }
        local ok, err = pcall(function()
            EVENT.fire(req)
        end)

        expect(ok):toBe(false)
        expect(err:find("test_error")):toBeTruthy()
        expect(STORE.co_scene):toBe(nil)
    end)

    test("ループ終了後にsuspendedならset_co_sceneで保存 (2.4)", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                coroutine.yield(nil)     -- スキップ
                coroutine.yield("valid") -- 有効値
                -- まだ終了していない
                coroutine.yield("more")
            end)
        end

        local req = { id = "TestEvent" }
        EVENT.fire(req)

        expect(type(STORE.co_scene)):toBe("thread")
        expect(coroutine.status(STORE.co_scene)):toBe("suspended")
    end)

    test("空シーン（dead + nil）が204 No Contentに変換 (2.3)", function()
        setup()
        REG.TestEvent = function(act)
            return coroutine.create(function(act)
                -- 何もyieldせず終了（空シーン）
                return nil
            end)
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("204 No Content")):toBeTruthy()
    end)
end)

-- ============================================================================
-- Task 3.3: 後方互換性テスト
-- Requirements: 2.2, 2.3
-- ============================================================================

describe("EVENT.fire - 後方互換性", function()
    local EVENT, REG, STORE

    local function setup()
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.res"] = nil
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

    test("既存の文字列ハンドラが正常動作 (2.2)", function()
        setup()
        REG.TestEvent = function(act)
            return "string_response"
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("string_response")):toBeTruthy()
    end)

    test("既存のnil戻り値が204 No Contentに変換 (2.3)", function()
        setup()
        REG.TestEvent = function(act)
            return nil
        end

        local req = { id = "TestEvent" }
        local result = EVENT.fire(req)

        expect(result:find("204 No Content")):toBeTruthy()
    end)
end)

return true
