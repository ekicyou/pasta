-- auto_talk_pause_test.lua
-- 自動トークの停止・再開（AutoTalkSwitch）のテスト (ontalk-suppression 2.3)
-- Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 1.7, 3.4, 4.1, 4.2
--
-- entry の依存閉包を同一インスタンス集合として一括新規ロードする（shiori_entry_test と同方針）。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect
local mocks = require("lua_test.mocks")

mocks.reset()
mocks.install()

local RELOAD = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.virtual_dispatcher",
    "pasta.shiori.res",
    "pasta.shiori.stop",
    "pasta.shiori.act",
    "pasta.shiori.sakura_builder",
    "pasta.act",
    "pasta.actor",
    "pasta.scene",
    "pasta.word",
    "pasta.global",
    "pasta.store",
    "pasta.config",
    "pasta.save",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

require("pasta.shiori.entry")
local ACT = require("pasta.act")
local GLOBAL = require("pasta.global")
local STORE = require("pasta.store")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

local BASE = 1702648800 -- 正時ちょうど

local function act_at(unix)
    return { token = {}, req = { id = "OnSecondChange", status = "idle", date = { unix = unix, hour = 15 } } }
end

-- 時計を進めた状態（next_talk_time ~= 0）に整える
local function setup()
    STORE.reset()
    dispatcher._reset()
    dispatcher._set_scene_executor(function() return nil end)
    dispatcher.dispatch(act_at(BASE))
    dispatcher.dispatch(act_at(BASE))
end

describe("AutoTalkSwitch - 4 つの名前", function()
    test("停止・再開が関数として登録されている（1.1, 1.2）", function()
        expect(type(GLOBAL.pause_auto_talk)):toBe("function")
        expect(type(GLOBAL.resume_auto_talk)):toBe("function")
    end)

    test("日本語の別名が同じ関数（1.3）", function()
        expect(GLOBAL["自動トーク停止"]):toBe(GLOBAL.pause_auto_talk)
        expect(GLOBAL["自動トーク再開"]):toBe(GLOBAL.resume_auto_talk)
    end)
end)

describe("AutoTalkSwitch - 停止と再開", function()
    test("停止で印が立ち、act に積まない（1.1, 1.5, 1.6）", function()
        setup()
        local act = act_at(BASE)
        GLOBAL.pause_auto_talk(act)
        expect(STORE.auto_talk_paused):toBe(true)
        expect(#act.token):toBe(0)
    end)

    test("act が nil でも停止・再開できる（1.7）", function()
        setup()
        GLOBAL.pause_auto_talk(nil)
        expect(STORE.auto_talk_paused):toBe(true)
        GLOBAL.resume_auto_talk(nil)
        expect(STORE.auto_talk_paused):toBe(false)
    end)

    test("重ねて止めても 1 回の再開で解ける（4.1, 4.2）", function()
        setup()
        GLOBAL.pause_auto_talk(nil)
        GLOBAL.pause_auto_talk(nil)
        GLOBAL.resume_auto_talk(nil)
        expect(STORE.auto_talk_paused):toBe(false)
    end)

    test("止めていたときの再開は時計を決め直し、act に積まない（1.2, 1.5）", function()
        setup()
        expect(dispatcher._get_internal_state().next_talk_time).not_:toBe(0)
        GLOBAL.pause_auto_talk(nil)
        local act = act_at(BASE + 10)
        GLOBAL.resume_auto_talk(act)
        expect(STORE.auto_talk_paused):toBe(false)
        expect(dispatcher._get_internal_state().next_talk_time):toBe(0)
        expect(#act.token):toBe(0)
    end)

    test("止めていないときの再開は何もしない（3.4）", function()
        setup()
        local before = dispatcher._get_internal_state()
        GLOBAL.resume_auto_talk(act_at(BASE + 10))
        local after = dispatcher._get_internal_state()
        expect(STORE.auto_talk_paused):toBe(false)
        expect(after.next_talk_time):toBe(before.next_talk_time)
        expect(after.next_hour_unix):toBe(before.next_hour_unix)
        expect(after.seen_epoch):toBe(before.seen_epoch)
    end)
end)

describe("AutoTalkSwitch - DSL の Call の道", function()
    test("act:call で ＞自動トーク停止／＞自動トーク再開 が届く（1.1, 1.2）", function()
        setup()
        local act = ACT.new({ actors = {}, save = {} })
        act:call("global", "自動トーク停止", {})
        expect(STORE.auto_talk_paused):toBe(true)
        act:call("global", "自動トーク再開", {})
        expect(STORE.auto_talk_paused):toBe(false)
    end)
end)

STORE.reset()
dispatcher._reset()
