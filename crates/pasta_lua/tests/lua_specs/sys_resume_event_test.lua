-- ============================================================================
-- OnSysResume スリープ明けのおしゃべりの結合テスト (onhour-talk-recount 3.2)
-- Requirements: 2.1, 2.2, 2.3, 2.7, 3.1-3.7, 4.1-4.4, 6.3-6.6
--
-- SHIORI.request（EVENT.fire）に実機の順番のリクエストを送る:
--   眠る前の秒 → スリープ → OnSysSuspend（復帰してから届く）→ 復帰後の最初の秒 → 数秒遅れて OnSysResume。
-- entry の依存閉包をテストごとに読み直し（reg_handler_chain_test と同じ）、シーンの検索は
-- @pasta_search の代役で引く。時計（pasta.shiori.clock）は閉包の外なので、テストごとに _reset する。
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local CLOCK = require("pasta.shiori.clock")

local ENTRY_CLOSURE = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.sys_resume",
    "pasta.shiori.event.mouse",
    "pasta.shiori.event.scene_play",
    "pasta.shiori.event.external_control",
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

-- 正時ちょうどの Unix 時刻。R0 はその 21 分後（「10:21」）。正時の時報マージンから遠い
local BASE = 497861 * 3600
local H = 3600
local R0 = BASE + 21 * 60
local NO_CONTENT = "SHIORI/3.0 204 No Content"
local OK = "SHIORI/3.0 200 OK"

local function mk_date(unix)
    local t = os.date("*t", unix)
    return {
        unix = unix, year = t.year, month = t.month, day = t.day,
        hour = t.hour, min = t.min, sec = t.sec, ns = 0,
        yday = t.yday, ordinal = t.yday, wday = t.wday - 1, num_days_from_sunday = t.wday - 1,
    }
end

local function hh_of(unix)
    return string.format("%02d", os.date("*t", unix).hour)
end

--- 書いた名前 → 登録名（大域）の表で引く @pasta_search の代役。unmet の名前は条件漏れを返す
local function make_search(names, unmet)
    return setmetatable({
        search_scene = function(_, name, scope)
            if scope ~= nil then return nil end
            if names[name] then return names[name], "__start__" end
            if unmet[name] then return nil, "unmet" end
            return nil
        end,
        search_scene_exact = function(_, name, scope)
            if scope == nil and names[name] then return names[name], "__start__" end
            return nil
        end,
    }, { __index = function() return function() return nil end end })
end

--- 状態を空にした閉包を読み直し、OnTalk のシーン（ONTALK）を置く
--- @param opts table|nil names: 足す 書いた名前 → 登録名、unmet: 条件漏れにする名前の集合
--- @return table env { ENTRY, REG, STORE, SCENE, GLOBAL, dispatcher, names }
local function reload_entry(opts)
    opts = opts or {}
    local names = { OnTalk = "sre_ontalk_1" }
    for k, v in pairs(opts.names or {}) do names[k] = v end
    local mocks = require("lua_test.mocks")
    mocks.reset()
    mocks.install({ search = make_search(names, opts.unmet or {}) })
    for _, name in ipairs(ENTRY_CLOSURE) do
        package.loaded[name] = nil
    end
    local env = { ENTRY = require("pasta.shiori.entry"), names = names }
    env.STORE = require("pasta.store")
    env.STORE.reset()
    require("pasta.shiori.event.callback").reset()
    require("pasta.shiori.stop").clear()
    CLOCK._reset()
    env.REG = require("pasta.shiori.event.register")
    env.SCENE = require("pasta.scene")
    env.GLOBAL = require("pasta.global")
    env.dispatcher = require("pasta.shiori.event.virtual_dispatcher")
    env.STORE.actors = { sakura = { name = "さくら", spot = 0 } }
    env.SCENE.register("sre_ontalk_1", "__start__", function(act)
        act:talk(env.STORE.actors.sakura, "ONTALK")
    end)
    return env
end

--- 閉包を読み直して fn(env) を走らせ、時計を必ず空に戻す
local function itest(name, opts, fn)
    if fn == nil then fn, opts = opts, nil end
    test(name, function()
        local ok, err = pcall(fn, reload_entry(opts))
        CLOCK._reset()
        if not ok then error(err, 0) end
    end)
end

--- SHIORI.request で 1 件送る
--- @param opts table|nil status: Status、ref: Reference（0 始まり）
local function send(env, id, unix, opts)
    opts = opts or {}
    return env.ENTRY.request({
        id = id, method = "get", version = 30, reference = opts.ref or {},
        status = opts.status or "", date = mk_date(unix), dic = {},
        security_level = "local",
    })
end

local function sec(env, unix, status) return send(env, "OnSecondChange", unix, { status = status }) end
local function suspend(env, unix) return send(env, "OnSysSuspend", unix) end
local function resume(env, unix, ref0) return send(env, "OnSysResume", unix, { ref = { [0] = ref0 or "normal" } }) end

local function status_of(res) return res:match("^(.-)\r\n") end
local function has(res, part) return res:find(part, 1, true) ~= nil end
local function talk_time(env) return env.dispatcher._get_internal_state().next_talk_time end

--- 登録名 global のシーン関数を置き、書いた名前 name で検索に出す
local function add_scene(env, name, global, fn)
    env.names[name] = global
    env.SCENE.register(global, "__start__", fn)
end

--- 時報のシーン（本文あり: JIHOU・本文なし: 何も積まない）を、時刻 unix の時の名前で置く
local function add_jihou(env, unix, speak)
    add_scene(env, "時報" .. hh_of(unix), "sre_jihou_1", function(act)
        if speak then act:talk(env.STORE.actors.sakura, "JIHOU") end
    end)
end

--- 前半（ZENHAN）を出して区切り、後半（KOUHAN）を待たせる続きを作る
local function start_chain(env, unix)
    add_scene(env, "OnSreChain", "sre_chain_1", function(act)
        act:talk(env.STORE.actors.sakura, "ZENHAN")
        act:yield()
        act:talk(env.STORE.actors.sakura, "KOUHAN")
    end)
    expect(has(send(env, "OnSreChain", unix), "ZENHAN")):toBe(true)
    expect(type(env.STORE.co_scene)):toBe("thread")
end

--- 起動の秒 R0 を見せ、何も溜めずに短く眠って W = R0 + 100 に復帰し、最初の秒まで送る
--- （次のトークは R0 + 180 以降なので、最初の秒には何も出ない）
--- @return integer W 復帰の時刻
--- @return number before 復帰の前の次のトークの時刻
local function boot_and_short_sleep(env)
    expect(status_of(sec(env, R0))):toBe(NO_CONTENT)
    local before = talk_time(env)
    expect(before >= R0 + 180):toBe(true)
    local W = R0 + 100
    expect(status_of(suspend(env, W))):toBe(NO_CONTENT)
    expect(status_of(sec(env, W))):toBe(NO_CONTENT)
    expect(talk_time(env)):toBe(before)
    return W, before
end

--- 出ないことの確かめ: 次の秒も 204 で、次のトークの時刻は before のまま
local function expect_no_wake(env, W, before)
    expect(status_of(sec(env, W + 4))):toBe(NO_CONTENT)
    expect(talk_time(env)):toBe(before)
end

--- 出ることの確かめ: 次の秒が OnTalk の 200
local function expect_wake(env, W)
    local res = sec(env, W + 4)
    expect(status_of(res)):toBe(OK)
    expect(has(res, "ONTALK")):toBe(true)
end

-- ============================================================================
-- 受けていないゴースト（要件 3.1, 3.2, 3.3, 6.4）
-- ============================================================================

describe("OnSysResume - 受けていないゴーストのスリープ明け", function()
    itest("OnSysResume の応答は 204 で、次の門の開いた秒が OnTalk の 200", function(env)
        local W = boot_and_short_sleep(env)
        expect(status_of(resume(env, W + 3))):toBe(NO_CONTENT)
        expect(talk_time(env)):toBe(W + 3)
        expect_wake(env, W)
        -- 出た後はふつうの間隔に戻る
        expect(talk_time(env) >= W + 4 + 180):toBe(true)
        expect(status_of(sec(env, W + 5))):toBe(NO_CONTENT)
    end)

    itest("Status に talking がある秒は 204 で、解けた秒に出る", function(env)
        local W = boot_and_short_sleep(env)
        resume(env, W + 3)
        expect(status_of(sec(env, W + 4, "talking,balloon(0=0)"))):toBe(NO_CONTENT)
        local res = sec(env, W + 5)
        expect(status_of(res)):toBe(OK)
        expect(has(res, "ONTALK")):toBe(true)
    end)
end)

-- ============================================================================
-- 復帰の前に自動トークが出ていた（要件 1.6, 3.4, 6.3）
-- ============================================================================

describe("OnSysResume - 復帰の後に自動トークが出ていた", function()
    itest("正時をまたいだ復帰: 最初の秒で時報、OnSysResume の後も talk_interval_min 秒より前は OnTalk が出ない", function(env)
        local W = BASE + H + 300 -- 11:05 に復帰
        add_jihou(env, W, true)
        sec(env, R0)
        suspend(env, W)
        local res = sec(env, W)
        expect(status_of(res)):toBe(OK)
        expect(has(res, "JIHOU")):toBe(true)
        local recounted = talk_time(env)
        expect(recounted >= W + 180):toBe(true)
        expect(status_of(resume(env, W + 4))):toBe(NO_CONTENT)
        expect(talk_time(env)):toBe(recounted)
        for t = W + 5, W + 179 do
            expect(status_of(sec(env, t))):toBe(NO_CONTENT)
        end
    end)

    itest("正時をまたがない長いスリープ: 最初の秒の溜まった OnTalk の後、OnSysResume でもう 1 回は出ない", function(env)
        sec(env, R0)
        local W = R0 + 1000
        suspend(env, W)
        local res = sec(env, W)
        expect(status_of(res)):toBe(OK)
        expect(has(res, "ONTALK")):toBe(true)
        local after = talk_time(env)
        expect(status_of(resume(env, W + 3))):toBe(NO_CONTENT)
        expect_no_wake(env, W, after)
    end)
end)

-- ============================================================================
-- ゴーストが受けている・出さない合図（要件 3.6, 3.7, 4.1, 4.2, 4.3, 6.5）
-- ============================================================================

describe("OnSysResume - スリープ明けのおしゃべりが出ない場合", function()
    itest("＊OnSysResume のシーンがあれば、そのシーンが動き、次のトークの時刻は変わらない", function(env)
        add_scene(env, "OnSysResume", "sre_resume_1", function(act)
            act:talk(env.STORE.actors.sakura, "RESUMESCENE")
        end)
        local W, before = boot_and_short_sleep(env)
        local res = resume(env, W + 3)
        expect(status_of(res)):toBe(OK)
        expect(has(res, "RESUMESCENE")):toBe(true)
        expect_no_wake(env, W, before)
    end)

    itest("GLOBAL.OnSysResume の関数があれば、それが動き、出ない", function(env)
        env.GLOBAL.OnSysResume = function(act)
            act:talk(env.STORE.actors.sakura, "GLOBALRESUME")
        end
        local W, before = boot_and_short_sleep(env)
        local res = resume(env, W + 3)
        expect(has(res, "GLOBALRESUME")):toBe(true)
        expect_no_wake(env, W, before)
    end)

    itest("REG.OnSysResume を代入したら、ゴーストのハンドラだけが呼ばれ、出ない", function(env)
        local called = 0
        env.REG.OnSysResume = function(_act)
            called = called + 1
            return nil
        end
        local W, before = boot_and_short_sleep(env)
        expect(status_of(resume(env, W + 3))):toBe(NO_CONTENT)
        expect(called):toBe(1)
        expect_no_wake(env, W, before)
    end)

    itest("REG:register の鎖が真の値を返した回は、出ない", function(env)
        env.REG:register("OnSysResume", function(_act) return "\\0CHAIN\\e" end)
        local W, before = boot_and_short_sleep(env)
        local res = resume(env, W + 3)
        expect(has(res, "CHAIN")):toBe(true)
        expect_no_wake(env, W, before)
    end)

    itest("Reference0 が auto なら、出ない", function(env)
        local W, before = boot_and_short_sleep(env)
        expect(status_of(resume(env, W + 3, "auto"))):toBe(NO_CONTENT)
        expect_no_wake(env, W, before)
    end)

    itest("自動トークの停止中に届いたら出ず、再開の後もすぐには出ない", function(env)
        local W = boot_and_short_sleep(env)
        env.GLOBAL.pause_auto_talk()
        expect(status_of(resume(env, W + 3))):toBe(NO_CONTENT)
        expect(status_of(sec(env, W + 4))):toBe(NO_CONTENT)
        env.GLOBAL.resume_auto_talk({ req = { date = mk_date(W + 5), dic = {} } })
        for t = W + 5, W + 10 do
            expect(status_of(sec(env, t))):toBe(NO_CONTENT)
        end
        expect(talk_time(env) >= W + 5 + 180):toBe(true)
    end)
end)

-- ============================================================================
-- 受けていないものとして出る場合（要件 3.1, 4.3, 4.4, 6.5）
-- ============================================================================

describe("OnSysResume - 受けていないものとして出る場合", function()
    itest("REG:register の鎖が全部 nil を返したら、既定の動きで出る", function(env)
        local called = 0
        env.REG:register("OnSysResume", function(_act) called = called + 1 end)
        env.REG:register("OnSysResume", function(_act) called = called + 1 return false end)
        local W = boot_and_short_sleep(env)
        expect(status_of(resume(env, W + 3))):toBe(NO_CONTENT)
        expect(called):toBe(2)
        expect_wake(env, W)
    end)

    itest("OnSysResume の名前のシーンが条件漏れなら、既定の動きで出る", { unmet = { OnSysResume = true } }, function(env)
        local W = boot_and_short_sleep(env)
        expect(status_of(resume(env, W + 3))):toBe(NO_CONTENT)
        expect(talk_time(env)):toBe(W + 3)
        expect_wake(env, W)
    end)

    itest("同じ復帰で auto → normal と届いたら 2 回目で出て、出た後の 3 回目は何も変えない", function(env)
        local W, before = boot_and_short_sleep(env)
        resume(env, W + 2, "auto")
        expect(talk_time(env)):toBe(before)
        resume(env, W + 3, "normal")
        expect(talk_time(env)):toBe(W + 3)
        expect_wake(env, W)
        local after = talk_time(env)
        expect(status_of(resume(env, W + 5, "normal"))):toBe(NO_CONTENT)
        expect(talk_time(env)):toBe(after)
        expect(status_of(sec(env, W + 6))):toBe(NO_CONTENT)
    end)
end)

-- ============================================================================
-- チェイントークの続きと外から起こした OnTalk（要件 2.1, 2.7, 3.5, 6.6）
-- ============================================================================

describe("OnSysResume - 続きと外から起こした OnTalk", function()
    itest("スリープ明けのおしゃべりの機会に、待っている続きが出る", function(env)
        start_chain(env, R0)
        local W = boot_and_short_sleep(env)
        resume(env, W + 3)
        local res = sec(env, W + 4)
        expect(has(res, "KOUHAN")):toBe(true)
        expect(has(res, "ONTALK")):toBe(false)
        expect(env.STORE.co_scene):toBeNil()
    end)

    itest("本文の無い時報は続きを残し、決め直した時刻の最初の機会に続きが出る", function(env)
        local W = BASE + H + 300
        add_jihou(env, W, false)
        sec(env, R0)
        start_chain(env, R0)
        local co = env.STORE.co_scene
        suspend(env, W)
        expect(status_of(sec(env, W))):toBe(NO_CONTENT) -- 時報は出たが本文が無い
        expect(env.STORE.co_scene):toBe(co)
        local recounted = talk_time(env)
        expect(recounted >= W + 180):toBe(true)
        resume(env, W + 4) -- 時報を 1 回と数えるので予約しない
        expect(talk_time(env)):toBe(recounted)
        for t = W + 5, recounted - 1 do
            expect(status_of(sec(env, t))):toBe(NO_CONTENT)
        end
        local res = sec(env, recounted)
        expect(has(res, "KOUHAN")):toBe(true)
        expect(has(res, "ONTALK")):toBe(false)
        expect(env.STORE.co_scene):toBeNil()
    end)

    itest("台詞を出した時報は、今どおり続きを捨てる", function(env)
        local W = BASE + H + 300
        add_jihou(env, W, true)
        sec(env, R0)
        start_chain(env, R0)
        suspend(env, W)
        expect(has(sec(env, W), "JIHOU")):toBe(true)
        expect(env.STORE.co_scene):toBeNil()
    end)

    itest("外から起こした OnTalk は今どおりシーンを出し、自動トークに数えない", function(env)
        local W, before = boot_and_short_sleep(env)
        local count = env.dispatcher._get_internal_state().talk_suspend_count
        local res = send(env, "OnTalk", W + 1)
        expect(status_of(res)):toBe(OK)
        expect(has(res, "ONTALK")):toBe(true)
        expect(talk_time(env)):toBe(before)
        expect(env.dispatcher._get_internal_state().talk_suspend_count):toBe(count)
        -- 自動トークではないので、スリープ明けのおしゃべりはまだ出る
        resume(env, W + 3)
        expect_wake(env, W)
    end)
end)

-- ============================================================================
-- 後片付け
-- ============================================================================

describe("OnSysResume - 後片付け", function()
    test("最後に entry の閉包と時計を空に読み直し、後のスイートへ渡す", function()
        local env = reload_entry()
        CLOCK._reset()
        expect(type(rawget(env.REG, "OnSysResume"))):toBe("function")
        expect(#getmetatable(env.REG).handlers("OnSysResume")):toBe(0)
        expect(env.dispatcher._get_internal_state().next_talk_time):toBe(0)
    end)
end)
