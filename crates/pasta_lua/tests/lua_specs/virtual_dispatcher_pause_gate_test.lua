-- virtual_dispatcher_pause_gate_test.lua
-- 止める門（DispatcherGate）と時計の決め直し（TimerRestart）のテスト (ontalk-suppression 2.1)
-- Requirements: 2.1, 2.2, 2.3, 2.4, 2.7, 3.1, 3.2, 3.3, 5.1
--
-- dispatch は pasta.store を require するため、テスト側 STORE と dispatcher 側 STORE を
-- 同一インスタンスにそろえて一括新規ロードする。
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
local CLOCK = require("pasta.shiori.clock")

local BASE = 1702648800  -- 正時ちょうど
local HOUR = 3600

local function act_at(unix, status)
    return { req = { id = "OnSecondChange", status = status or "idle", date = { unix = unix, hour = 15 } } }
end

-- 内部タイマを初期化し、必ず thread を返す scene_executor を装着する
local function setup()
    STORE.reset()
    CLOCK._reset()
    dispatcher._reset()
    dispatcher._set_scene_executor(function(event_name)
        return coroutine.create(function() return event_name end)
    end)
    dispatcher.dispatch(act_at(BASE)) -- next_hour_unix を設定
    dispatcher.dispatch(act_at(BASE)) -- next_talk_time を設定
end

describe("DispatcherGate - 止めている間は時報・トーク・続きを出さない", function()
    test("止めていなければ正時で時報が出る（5.1）", function()
        setup()
        expect(type(dispatcher.dispatch(act_at(BASE + HOUR)))):toBe("thread")
    end)

    test("止めている間は正時を過ぎても nil（2.2）", function()
        setup()
        STORE.auto_talk_paused = true
        expect(dispatcher.dispatch(act_at(BASE + HOUR))):toBe(nil)
    end)

    test("止めている間はトーク時刻を過ぎても nil（2.1）", function()
        setup()
        STORE.auto_talk_paused = true
        -- 正時の手前 10 分（時報マージンの外）、トーク間隔の最大を十分に過ぎた時刻
        expect(dispatcher.dispatch(act_at(BASE + HOUR - 600))):toBe(nil)
    end)

    test("止めている間は続きを返さず、捨てもしない（2.3・2.7）", function()
        setup()
        local co = coroutine.create(function() end)
        STORE.co_scene = co
        STORE.auto_talk_paused = true
        expect(dispatcher.dispatch(act_at(BASE + HOUR - 600))):toBe(nil)
        expect(STORE.co_scene):toBe(co)
    end)

    test("止めている間もキックのシーンは返る（2.4）", function()
        setup()
        local KICK = require("pasta.shiori.event.kick")
        local orig = KICK.try_dispatch
        local kick_co = coroutine.create(function() end)
        KICK.try_dispatch = function() return kick_co end
        STORE.auto_talk_paused = true
        local result = dispatcher.dispatch(act_at(BASE + HOUR))
        KICK.try_dispatch = orig
        expect(result):toBe(kick_co)
    end)

    test("止めている間に過ぎた時報は再開の後に出ない（3.2）", function()
        setup()
        STORE.auto_talk_paused = true
        dispatcher.dispatch(act_at(BASE + HOUR + 60))
        STORE.auto_talk_paused = false
        dispatcher.restart_timers(act_at(BASE + HOUR + 60))
        expect(dispatcher.dispatch(act_at(BASE + HOUR + 61))):toBe(nil)
        -- 次の正時からは今どおり（3.3）
        expect(type(dispatcher.dispatch(act_at(BASE + 2 * HOUR)))):toBe("thread")
    end)
end)

describe("TimerRestart - 再開したときの時計の決め直し", function()
    test("next_talk_time を初回扱いに戻す（3.1）", function()
        setup()
        expect(dispatcher._get_internal_state().next_talk_time > 0):toBe(true)
        dispatcher.restart_timers(act_at(BASE + 100))
        expect(dispatcher._get_internal_state().next_talk_time):toBe(0)
    end)

    test("次の正時を今の時刻から決め直し、基準を記録する（3.2）", function()
        setup()
        dispatcher.restart_timers(act_at(BASE + HOUR + 60))
        local s = dispatcher._get_internal_state()
        expect(s.next_hour_unix):toBe(BASE + 2 * HOUR)
        expect(s.seen_epoch):toBe(CLOCK.epoch())
    end)

    test("古い時刻の act では next_hour_unix を戻さない（前へだけ）", function()
        setup()
        dispatcher.restart_timers(act_at(BASE + HOUR + 60))
        dispatcher.restart_timers(act_at(BASE + 60))
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(BASE + 2 * HOUR)
    end)

    test("仮想時刻の基準が変わっていれば古い時刻でも置き換える", function()
        setup()
        dispatcher.restart_timers(act_at(BASE + HOUR + 60))
        local orig = CLOCK.epoch
        CLOCK.epoch = function() return 7 end
        dispatcher.restart_timers(act_at(BASE + 60))
        local s = dispatcher._get_internal_state()
        CLOCK.epoch = orig
        expect(s.next_hour_unix):toBe(BASE + HOUR)
        expect(s.seen_epoch):toBe(7)
    end)

    test("act が nil なら時報も初回扱い", function()
        setup()
        dispatcher.restart_timers(nil)
        local s = dispatcher._get_internal_state()
        expect(s.next_hour_unix):toBe(0)
        expect(s.next_talk_time):toBe(0)
    end)

    test("date の無い act でも時報は初回扱い", function()
        setup()
        dispatcher.restart_timers({ req = {} })
        expect(dispatcher._get_internal_state().next_hour_unix):toBe(0)
    end)
end)
