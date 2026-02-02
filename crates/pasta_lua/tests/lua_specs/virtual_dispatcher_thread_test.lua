-- virtual_dispatcher_thread_test.lua
-- Lua-side BDD tests for virtual_dispatcher thread return support
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Helper: create mock act object with req field
local function create_mock_act(req)
    return { req = req }
end

-- ============================================================================
-- Task 3.1: create_scene_thread() ヘルパー関数
-- Task 3.2: check_hour() thread返却形式
-- Task 3.3: check_talk() チェイントーク継続ロジック
-- Task 3.4: dispatch() thread返却対応
-- Requirements: 3.1, 4.1-4.4, 5.1-5.5
-- ============================================================================

describe("virtual_dispatcher - thread返却", function()
    local dispatcher, STORE

    local function setup()
        package.loaded["pasta.shiori.event.virtual_dispatcher"] = nil
        package.loaded["pasta.store"] = nil
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        STORE = require("pasta.store")
        dispatcher._reset()
        STORE.reset()
    end

    test("check_hour() が thread を返す", function()
        setup()
        -- シーン関数が見つかるモック
        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function(act)
                return "hour_result"
            end)
        end)

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 1702648800 }
        })

        -- Initialize
        dispatcher.check_hour(act)

        -- Fire at next hour
        act.req.date.unix = 1702652400
        local result = dispatcher.check_hour(act)

        expect(type(result)):toBe("thread")
    end)

    test("check_talk() が thread を返す", function()
        setup()
        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function(act)
                return "talk_result"
            end)
        end)

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 }
        })

        -- Initialize
        dispatcher.check_hour(act) -- Set next_hour_unix
        dispatcher.check_talk(act) -- Set next_talk_time

        -- Fire after interval
        local state = dispatcher._get_internal_state()
        act.req.date.unix = state.next_talk_time + 1

        local result = dispatcher.check_talk(act)

        expect(type(result)):toBe("thread")
    end)

    test("dispatch() が thread を返す", function()
        setup()
        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function(act)
                return event_name .. "_result"
            end)
        end)

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 }
        })

        -- Initialize
        dispatcher.dispatch(act)

        -- Fire
        local state = dispatcher._get_internal_state()
        act.req.date.unix = state.next_talk_time + 1

        local result = dispatcher.dispatch(act)

        expect(type(result)):toBe("thread")
    end)

    test("シーンが見つからない場合 nil を返す", function()
        setup()
        dispatcher._set_scene_executor(function(event_name, act)
            return nil
        end)

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 }
        })

        -- Initialize
        dispatcher.check_hour(act)
        dispatcher.check_talk(act)

        -- Fire
        local state = dispatcher._get_internal_state()
        act.req.date.unix = state.next_talk_time + 1

        local result = dispatcher.check_talk(act)

        expect(result):toBe(nil)
    end)
end)

describe("virtual_dispatcher - チェイントーク継続", function()
    local dispatcher, STORE

    local function setup()
        package.loaded["pasta.shiori.event.virtual_dispatcher"] = nil
        package.loaded["pasta.store"] = nil
        dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        STORE = require("pasta.store")
        dispatcher._reset()
        STORE.reset()
    end

    test("STORE.co_scene が存在する場合 check_talk() は継続用 thread を返す", function()
        setup()
        -- 既存のsuspendedコルーチンを設定
        local existing_co = coroutine.create(function(act)
            coroutine.yield("first")
            return "final"
        end)
        coroutine.resume(existing_co) -- suspended状態に

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 }
        })

        -- 初期化: next_hour_unixとnext_talk_timeを設定
        dispatcher.check_hour(act)
        dispatcher.check_talk(act)

        -- OnTalk発動タイミングにする（next_talk_timeを過ぎる）
        local state = dispatcher._get_internal_state()
        act.req.date.unix = state.next_talk_time + 1

        -- ここでSTORE.co_sceneを設定（OnTalk発動タイミングで継続確認される）
        STORE.co_scene = existing_co

        -- check_talkは既存のコルーチンを返すべき
        local result = dispatcher.check_talk(act)

        expect(result):toBe(existing_co)
    end)

    test("STORE.co_scene が nil の場合 新規シーンを検索", function()
        setup()
        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function(act)
                return "new_scene"
            end)
        end)

        -- STORE.co_scene は nil
        expect(STORE.co_scene):toBe(nil)

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 }
        })

        -- Initialize
        dispatcher.check_hour(act)
        dispatcher.check_talk(act)

        -- Fire
        local state = dispatcher._get_internal_state()
        act.req.date.unix = state.next_talk_time + 1

        local result = dispatcher.check_talk(act)

        -- 新しいthreadが返される
        expect(type(result)):toBe("thread")
    end)

    test("OnHour は STORE.co_scene を無視して常に新規 thread を返す", function()
        setup()
        -- 既存のsuspendedコルーチンを設定
        local existing_co = coroutine.create(function(act)
            coroutine.yield("first")
        end)
        coroutine.resume(existing_co)
        STORE.co_scene = existing_co

        dispatcher._set_scene_executor(function(event_name, act)
            return coroutine.create(function(act)
                return "hour_scene"
            end)
        end)

        local act = create_mock_act({
            id = "OnSecondChange",
            status = "idle",
            date = { unix = 0 }
        })

        -- Initialize
        dispatcher.check_hour(act)

        -- Fire
        act.req.date.unix = 3600

        local result = dispatcher.check_hour(act)

        -- 新しいthreadが返される（既存のco_sceneではなく）
        expect(type(result)):toBe("thread")
        expect(result ~= existing_co):toBe(true)
    end)
end)

return true
