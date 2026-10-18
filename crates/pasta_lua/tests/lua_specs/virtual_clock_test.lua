-- pasta.shiori.clock 仮想時刻のずれを持つ時計の単体テスト (tutorial-ssp-gaps 2.1)
-- 合図の見込み・返事からのずれ・今の時刻と日付の表・読み取りのスクリプト
-- Requirements: 2.1, 2.2, 2.6, 2.7, 2.8, 2.9, 7.1, 7.3
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local PROBE = "\\![get,property,OnPastaClock,system.year,system.month,system.day,"
    .. "system.hour,system.minute,system.second]\\e"

-- 正時ちょうどの Unix 時刻（2026-10-18 ごろ。unix % 3600 == 0）
local BASE = 497861 * 3600

--- Rust の lua_date_from と同じ形の日付の表
local function mk_date(unix)
    local t = os.date("*t", unix)
    return {
        unix = unix,
        year = t.year,
        month = t.month,
        day = t.day,
        hour = t.hour,
        min = t.min,
        sec = t.sec,
        ns = 123,
        yday = t.yday,
        ordinal = t.yday,
        wday = t.wday - 1,
        num_days_from_sunday = t.wday - 1,
    }
end

local function mk_req(id, unix, opts)
    opts = opts or {}
    return {
        id = id,
        date = unix and mk_date(unix) or nil,
        dic = opts.dic or {},
        reference = opts.reference or {},
        security_level = opts.security_level or "local",
    }
end

--- target の地方時を Reference0〜5 に入れた OnPastaClock の返事
local function mk_reply(unix, target, opts)
    opts = opts or {}
    local t = os.date("*t", target)
    opts.reference = opts.reference or {
        [0] = tostring(t.year),
        [1] = tostring(t.month),
        [2] = tostring(t.day),
        [3] = tostring(t.hour),
        [4] = tostring(t.min),
        [5] = tostring(t.sec),
    }
    return mk_req("OnPastaClock", unix, opts)
end

local XPT = { ["X-Pasta-Time"] = "2026-10-18T09:00:00+09:00" }

--- 警告・デバッグを記録するログを差し込んだ clock を新規ロードして body を実行し、元へ戻す
--- @param body fun(CLOCK: table, warns: string[])
local function with_clock(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_clock = package.loaded["pasta.shiori.clock"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["pasta.shiori.clock"] = nil
    local ok, err = pcall(function()
        local CLOCK = require("pasta.shiori.clock")
        CLOCK._reset()
        body(CLOCK, warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.shiori.clock"] = saved_clock
    if not ok then error(err, 0) end
end

describe("CLOCK.observe - 合図の見込み（Unit 1）", function()
    test("OnMinuteChange が now%60 = 0・2・58 では予約を立てない", function()
        with_clock(function(CLOCK)
            for _, s in ipairs({ 0, 2, 58 }) do
                expect(CLOCK.observe(mk_req("OnMinuteChange", BASE + s))):toBe(false)
                expect(CLOCK._get_state().probe_armed):toBe(false)
            end
        end)
    end)

    test("OnMinuteChange が now%60 = 3・30 では予約を立てる", function()
        with_clock(function(CLOCK)
            for _, s in ipairs({ 3, 30 }) do
                CLOCK._reset()
                expect(CLOCK.observe(mk_req("OnMinuteChange", BASE + s))):toBe(false)
                expect(CLOCK._get_state().probe_armed):toBe(true)
            end
        end)
    end)

    test("OnHourTimeSignal が now%3600 = 7 では立てず、1800 では立てる", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_req("OnHourTimeSignal", BASE + 7))
            expect(CLOCK._get_state().probe_armed):toBe(false)
            CLOCK.observe(mk_req("OnHourTimeSignal", BASE + 3599))
            expect(CLOCK._get_state().probe_armed):toBe(false)
            CLOCK.observe(mk_req("OnHourTimeSignal", BASE + 1800))
            expect(CLOCK._get_state().probe_armed):toBe(true)
        end)
    end)

    test("X-Pasta-Time 付きでは外れた合図でも予約を立てない", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_req("OnMinuteChange", BASE + 30, { dic = XPT }))
            CLOCK.observe(mk_req("OnHourTimeSignal", BASE + 1800, { dic = XPT }))
            expect(CLOCK._get_state().probe_armed):toBe(false)
        end)
    end)

    test("req.date が無いときは合図を見ない", function()
        with_clock(function(CLOCK)
            expect(CLOCK.observe(mk_req("OnMinuteChange", nil))):toBe(false)
            expect(CLOCK._get_state().probe_armed):toBe(false)
        end)
    end)

    test("合図はずれを足した時刻で見る（仮想の分の変わり目なら立てない）", function()
        with_clock(function(CLOCK)
            -- ずれ +30 秒: 実際の xx:xx:30 が仮想の xx:xx:00
            CLOCK.observe(mk_reply(BASE, BASE + 30))
            expect(CLOCK.offset()):toBe(30)
            CLOCK.observe(mk_req("OnMinuteChange", BASE + 30))
            expect(CLOCK._get_state().probe_armed):toBe(false)
            CLOCK.observe(mk_req("OnMinuteChange", BASE + 60))
            expect(CLOCK._get_state().probe_armed):toBe(true)
        end)
    end)

    test("他のイベントは false を返し、何も変えない", function()
        with_clock(function(CLOCK)
            expect(CLOCK.observe(mk_req("OnSecondChange", BASE + 30))):toBe(false)
            expect(CLOCK._get_state()):toEqual({ offset = 0, probe_armed = false })
        end)
    end)
end)

describe("CLOCK.observe - 返事からずれを決める（Unit 2）", function()
    test("6 つの値からずれが決まり、true を返す", function()
        with_clock(function(CLOCK, warns)
            expect(CLOCK.observe(mk_reply(BASE, BASE + 3600 * 5 + 1234))):toBe(true)
            expect(CLOCK.offset()):toBe(3600 * 5 + 1234)
            expect(#warns):toBe(0)
            -- 後から届いた返事のずれが残る（過去への設定も負のずれとして持つ）
            CLOCK.observe(mk_reply(BASE, BASE - 600))
            expect(CLOCK.offset()):toBe(-600)
        end)
    end)

    test("ゼロ埋めの値も読める", function()
        with_clock(function(CLOCK)
            local t = os.date("*t", BASE + 65)
            local ref = {
                [0] = tostring(t.year),
                [1] = string.format("%02d", t.month),
                [2] = string.format("%02d", t.day),
                [3] = string.format("%02d", t.hour),
                [4] = string.format("%02d", t.min),
                [5] = string.format("%02d", t.sec),
            }
            CLOCK.observe(mk_reply(BASE, BASE, { reference = ref }))
            expect(CLOCK.offset()):toBe(65)
        end)
    end)

    test("差が 2 秒以下なら 0（Reset で OS の時計に戻る）", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            expect(CLOCK.offset()):toBe(3600)
            CLOCK.observe(mk_reply(BASE, BASE + 2))
            expect(CLOCK.offset()):toBe(0)
            CLOCK.observe(mk_reply(BASE, BASE - 2))
            expect(CLOCK.offset()):toBe(0)
            CLOCK.observe(mk_reply(BASE, BASE + 3))
            expect(CLOCK.offset()):toBe(3)
        end)
    end)

    test("数でない値は 0 にして警告を 1 行出し、true を返す", function()
        with_clock(function(CLOCK, warns)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local ref = { [0] = "2026", [1] = "10", [2] = "18", [3] = "abc", [4] = "0", [5] = "0" }
            expect(CLOCK.observe(mk_reply(BASE, BASE, { reference = ref }))):toBe(true)
            expect(CLOCK.offset()):toBe(0)
            expect(#warns):toBe(1)
        end)
    end)

    test("整数でない値・足りない値でも 0 にして警告を出し、true を返す", function()
        with_clock(function(CLOCK, warns)
            local half = { [0] = "2026", [1] = "10", [2] = "18", [3] = "9.5", [4] = "0", [5] = "0" }
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            expect(CLOCK.observe(mk_reply(BASE, BASE, { reference = half }))):toBe(true)
            expect(CLOCK.offset()):toBe(0)
            local short = { [0] = "2026", [1] = "10", [2] = "18", [3] = "9", [4] = "0" }
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            expect(CLOCK.observe(mk_reply(BASE, BASE, { reference = short }))):toBe(true)
            expect(CLOCK.offset()):toBe(0)
            expect(#warns):toBe(2)
        end)
    end)

    test("req.date が無い返事は読めない値として 0 にし、true を返す", function()
        with_clock(function(CLOCK, warns)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local req = mk_reply(BASE, BASE + 3600)
            req.date = nil
            expect(CLOCK.observe(req)):toBe(true)
            expect(CLOCK.offset()):toBe(0)
            expect(#warns):toBe(1)
        end)
    end)

    test("外からの SSTP（external）の返事は受け取らず、ずれを変えない", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local req = mk_reply(BASE, BASE + 7200, { security_level = "external" })
            expect(CLOCK.observe(req)):toBe(false)
            expect(CLOCK.offset()):toBe(3600)
        end)
    end)

    test("X-Pasta-Time 付きの返事は受け取るが、ずれを変えない", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            expect(CLOCK.observe(mk_reply(BASE, BASE + 7200, { dic = XPT }))):toBe(true)
            expect(CLOCK.offset()):toBe(3600)
        end)
    end)
end)

describe("CLOCK.now / CLOCK.date - 今の時刻と日付の表（Unit 3）", function()
    test("ずれ 0 では now は req.date.unix、date は req.date そのもの", function()
        with_clock(function(CLOCK)
            local req = mk_req("OnSecondChange", BASE + 10)
            expect(CLOCK.now(req)):toBe(BASE + 10)
            expect(CLOCK.date(req)):toBe(req.date)
        end)
    end)

    test("ずれがあれば仮想の日付を新しい表で返し、req.date を書き換えない", function()
        with_clock(function(CLOCK)
            local shift = 86400 * 2 + 3600 * 5 + 61
            CLOCK.observe(mk_reply(BASE, BASE + shift))
            local req = mk_req("OnSecondChange", BASE + 10)
            local original = mk_date(BASE + 10)
            expect(CLOCK.now(req)):toBe(BASE + 10 + shift)
            local d = CLOCK.date(req)
            expect(d == req.date):toBe(false)
            local t = os.date("*t", BASE + 10 + shift)
            expect(d):toEqual({
                unix = BASE + 10 + shift,
                year = t.year,
                month = t.month,
                day = t.day,
                hour = t.hour,
                min = t.min,
                sec = t.sec,
                ns = 123,
                yday = t.yday,
                ordinal = t.yday,
                wday = t.wday - 1,
                num_days_from_sunday = t.wday - 1,
            })
            expect(d.wday == req.date.wday):toBe(false)
            expect(req.date):toEqual(original)
        end)
    end)

    test("X-Pasta-Time 付きではずれを無視する", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local req = mk_req("OnSecondChange", BASE + 10, { dic = XPT })
            expect(CLOCK.now(req)):toBe(BASE + 10)
            expect(CLOCK.date(req)):toBe(req.date)
        end)
    end)

    test("req.date が無ければ nil", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local req = mk_req("OnSecondChange", nil)
            expect(CLOCK.now(req)):toBeNil()
            expect(CLOCK.date(req)):toBeNil()
        end)
    end)
end)

describe("CLOCK.take_probe - 読み取りのスクリプト（Unit 4）", function()
    test("予約が無ければ nil", function()
        with_clock(function(CLOCK)
            expect(CLOCK.take_probe(mk_req("OnSecondChange", BASE))):toBeNil()
        end)
    end)

    test("予約があれば決まった文字列を 1 回だけ返す", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_req("OnMinuteChange", BASE + 30))
            local req = mk_req("OnSecondChange", BASE + 31)
            expect(CLOCK.take_probe(req)):toBe(PROBE)
            expect(CLOCK._get_state().probe_armed):toBe(false)
            expect(CLOCK.take_probe(req)):toBeNil()
        end)
    end)

    test("X-Pasta-Time 付きでは読み取りを出さない", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_req("OnMinuteChange", BASE + 30))
            expect(CLOCK.take_probe(mk_req("OnSecondChange", BASE + 31, { dic = XPT }))):toBeNil()
        end)
    end)
end)
