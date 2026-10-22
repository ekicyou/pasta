-- auto_talk_pause_test.lua
-- 自動トークの停止・再開（AutoTalkSwitch）のテスト (ontalk-suppression 2.3)
-- Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 1.7, 2.2, 2.6, 2.8, 3.1, 3.2, 3.4, 4.1, 4.2, 4.5, 5.1
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

describe("AutoTalkSwitch - DSL の単語参照の道", function()
    test("act:word で ＠自動トーク停止／＠自動トーク再開 が届き、見える文字を足さない（1.4, 1.5）", function()
        setup()
        local act = ACT.new({ actors = {}, save = {} })
        act:talk(nil, act:word("自動トーク停止"))
        expect(STORE.auto_talk_paused):toBe(true)
        act:talk(nil, act:word("自動トーク再開"))
        expect(STORE.auto_talk_paused):toBe(false)
        expect(#act.token):toBe(0)
    end)

    test("アクター行の ＠自動トーク停止（proxy:word、トランスパイル後の形）でも届く（1.4, 1.5）", function()
        setup()
        local act = ACT.new({ ["ats_さくら"] = { name = "ats_さくら" } })
        local p = act:actor_proxy("ats_さくら")
        p:talk(p:word("自動トーク停止"))
        expect(STORE.auto_talk_paused):toBe(true)
        p:talk(p:word("自動トーク再開"))
        expect(STORE.auto_talk_paused):toBe(false)
        expect(#act.token):toBe(0)
    end)
end)

--- 警告を記録するログに差し替えて、コルーチンの中で fn(act) を 1 回区切るまで回す。
--- ログは pcall で包んで必ず元へ戻す（act.lua は呼び出しの時点でログを引く）。
--- @return string[] warns, boolean yielded
local function run_chain(fn)
    local warns = {}
    local noop = function() end
    local saved_log = package.loaded["@pasta_log"]
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    local ok, err = pcall(function()
        local act = ACT.new({ actors = {}, save = {} })
        local co = coroutine.create(fn)
        local resumed, e = coroutine.resume(co, act)
        assert(resumed, e)
        return coroutine.status(co)
    end)
    package.loaded["@pasta_log"] = saved_log
    assert(ok, err)
    return warns, err == "suspended"
end

local CHAIN_CALLS = {
    { "GLOBAL.yield（＞yield）", function(act) GLOBAL.yield(act) end },
    { "GLOBAL[\"チェイントーク\"]（＞チェイントーク）", function(act) GLOBAL["チェイントーク"](act) end },
    { "Lua の直接の act:yield()", function(act) act:yield() end },
}

describe("ChainTalkGuard - 止めている間のチェイントーク", function()
    for _, case in ipairs(CHAIN_CALLS) do
        test(case[1] .. " は警告 1 回・印を解いて区切る（2.8）", function()
            setup()
            GLOBAL.pause_auto_talk(nil)
            local warns, yielded = run_chain(case[2])
            expect(#warns):toBe(1)
            expect(warns[1]:find("自動トーク", 1, true) ~= nil):toBe(true)
            expect(warns[1]:find("チェイントーク", 1, true) ~= nil):toBe(true)
            expect(STORE.auto_talk_paused):toBe(false)
            expect(yielded):toBe(true)
        end)

        test(case[1] .. " は止めていなければ警告しない（5.1）", function()
            setup()
            local warns, yielded = run_chain(case[2])
            expect(#warns):toBe(0)
            expect(STORE.auto_talk_paused):toBe(false)
            expect(yielded):toBe(true)
        end)
    end
end)

local REG = require("pasta.shiori.event.register")
local CLOCK = require("pasta.shiori.clock")
local HOUR = 3600

-- 必ず thread を返す scene_executor を付けて、時計を進めた状態に整える
local function setup_threads()
    setup()
    CLOCK._reset()
    dispatcher._set_scene_executor(function(event_name)
        return coroutine.create(function() return event_name end)
    end)
end

describe("AutoTalkSwitch - 止めている間の OnSecondChange の既定ハンドラ", function()
    test("正時を過ぎていても仮想時刻の読み取りが返る（2.2, 2.6）", function()
        setup_threads()
        CLOCK.observe({ id = "OnMinuteChange", date = { unix = BASE + HOUR + 30 }, dic = {} })
        expect(CLOCK._get_state().probe_armed):toBe(true)
        GLOBAL.pause_auto_talk(nil)
        local act = act_at(BASE + HOUR + 31)
        act.req.dic = {}
        local result = REG.OnSecondChange(act)
        expect(type(result)):toBe("string")
        expect(result:find("OnPastaClock", 1, true) ~= nil):toBe(true)
    end)
end)

describe("AutoTalkSwitch - 再開の後の時計", function()
    test("停止と再開を続けて呼んでも間隔が数え直され、min 未満ではトークも続きも出ない（1.6, 3.1）", function()
        setup_threads()
        local cfg = dispatcher._get_config()
        local t = BASE + 600 -- 前回決めた次のトーク時刻（BASE + max まで）は過ぎている
        local co = coroutine.create(function() end)
        STORE.co_scene = co
        local act = act_at(t)
        GLOBAL.pause_auto_talk(act)
        GLOBAL.resume_auto_talk(act)
        expect(dispatcher.dispatch(act_at(t))):toBe(nil)
        expect(dispatcher.dispatch(act_at(t + cfg.talk_interval_min - 1))):toBe(nil)
        -- 間隔を過ぎれば、捨てずに残した続きが出る（5.1）
        expect(dispatcher.dispatch(act_at(t + cfg.talk_interval_max))):toBe(co)
    end)

    test("止めている間に仮想時刻を巻き戻してから再開しても、過ぎた正時が出ない（3.2）", function()
        setup_threads()
        GLOBAL.pause_auto_talk(nil)
        local orig = CLOCK.epoch
        local ok, err = pcall(function()
            CLOCK.epoch = function() return 7 end
            -- 巻き戻した先は正時の 30 秒後（決め直しの猶予の中）
            local rewound = BASE - HOUR + 30
            GLOBAL.resume_auto_talk(act_at(rewound))
            expect(dispatcher.dispatch(act_at(rewound + 1))):toBe(nil)
            expect(type(dispatcher.dispatch(act_at(BASE)))):toBe("thread")
        end)
        CLOCK.epoch = orig
        assert(ok, err)
    end)

    test("STORE.reset() で解け、時報が今どおり出る（4.5, 5.1）", function()
        setup_threads()
        GLOBAL.pause_auto_talk(nil)
        expect(dispatcher.dispatch(act_at(BASE + HOUR))):toBe(nil)
        STORE.reset()
        expect(STORE.auto_talk_paused):toBe(false)
        expect(type(dispatcher.dispatch(act_at(BASE + HOUR)))):toBe("thread")
    end)
end)

STORE.reset()
CLOCK._reset()
dispatcher._reset()
