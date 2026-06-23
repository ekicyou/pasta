-- virtual_dispatcher_kick_force_test.lua
-- Lua-side BDD tests for KickForceGate (pasta-scene-kick Task 4.3)
-- Requirements: 5.5, 6.4
--
-- 検証契約（virtual_dispatcher.lua dispatch 入口の is_blocked 抑制ゲート）:
--   - STORE.kick_force=true + status=talking -> dispatch が抑制されない（is_blocked 突破）
--     ＋直後に STORE.kick_force==false（ワンショット消費）。
--   - STORE.kick_force=false + status=talking -> 従来どおり抑制（nil）。
--   - force 突破は 1 回限り（消費後の talking tick は通常ブロック）。
--
-- 注意: dispatch は pasta.store を require するため、テスト側 STORE と
-- dispatcher 側 STORE を同一インスタンスにそろえて一括新規ロードする。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local RELOAD = {
    "pasta.shiori.event.virtual_dispatcher",
    "pasta.store",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local STORE = require("pasta.store")

local function create_mock_act(req)
    return { req = req }
end

describe("KickForceGate - dispatch 入口の割り込み許可ゲート", function()
    -- 共通セットアップ: 内部タイマを初期化し、必ず thread を返す scene_executor を装着。
    -- 初期化 dispatch は idle status で行い OnHour/OnTalk タイマを設定する。
    local function setup()
        STORE.reset()
        dispatcher._reset()
        dispatcher._set_scene_executor(function(event_name)
            return coroutine.create(function() return event_name .. "_result" end)
        end)
        -- 初期化 dispatch（idle・非ブロック）で next_hour_unix を設定
        local init_act = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = 1702648800 },
        })
        dispatcher.dispatch(init_act)
    end

    -- 正時境界 + 指定 status + kick_force で dispatch した結果を返す
    local function dispatch_at_next_hour(status)
        local act = create_mock_act({
            id = "OnSecondChange", status = status, date = { unix = 1702652400, hour = 15 },
        })
        return dispatcher.dispatch(act)
    end

    test("kick_force=true + status=talking で dispatch が抑制されない（is_blocked 突破）", function()
        setup()
        STORE.kick_force = true

        local result = dispatch_at_next_hour("talking")

        -- talking でも force により突破され、OnHour thread が返る
        expect(type(result)):toBe("thread")
    end)

    test("force 突破の直後に kick_force が消費され false 化する", function()
        setup()
        STORE.kick_force = true

        dispatch_at_next_hour("talking")

        expect(STORE.kick_force):toBe(false)
    end)

    test("kick_force=false + status=talking は従来どおり抑制（nil）", function()
        setup()
        -- kick_force は既定 false（setup の STORE.reset で false 化済み）
        expect(STORE.kick_force):toBe(false)

        local result = dispatch_at_next_hour("talking")

        expect(result):toBe(nil)
    end)

    test("force 突破は 1 回限り（消費後の talking tick は通常ブロック）", function()
        setup()
        STORE.kick_force = true

        -- 1 回目: force で突破（thread）
        local first = dispatch_at_next_hour("talking")
        expect(type(first)):toBe("thread")
        expect(STORE.kick_force):toBe(false)

        -- 2 回目: 消費済みのため talking が通常ブロック（nil）
        local second = dispatch_at_next_hour("talking")
        expect(second):toBe(nil)
    end)
end)
