-- pasta.shiori.clock 仮想時刻のずれを持つ時計の単体テスト (tutorial-ssp-gaps 2.1)
-- 合図の見込み・返事からのずれ・今の時刻と日付の表・読み取りのスクリプト
-- Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 7.1, 7.3
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
            expect(CLOCK.observe(mk_req("OnBoot", BASE + 30))):toBe(false)
            expect(CLOCK._get_state()):toEqual({
                offset = 0, probe_armed = false, epoch = 0, jump_total = 0, resumed = false, suspended = false,
            })
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

    test("unix の無い日付の表では now は nil、date は req.date そのもので、合図も見ない", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local req = mk_req("OnMinuteChange", BASE + 30)
            req.date.unix = nil
            expect(CLOCK.now(req)):toBeNil()
            expect(CLOCK.date(req)):toBe(req.date)
            expect(CLOCK.observe(req)):toBe(false)
            expect(CLOCK._get_state().probe_armed):toBe(false)
        end)
    end)

    test("unix の無い日付の表の返事は読めない値として扱い、ずれを 0 にする", function()
        with_clock(function(CLOCK, warns)
            CLOCK.observe(mk_reply(BASE, BASE + 3600))
            local reply = mk_reply(BASE, BASE + 7200)
            reply.date.unix = nil
            expect(CLOCK.observe(reply)):toBe(true)
            expect(CLOCK.offset()):toBe(0)
            expect(#warns):toBe(1)
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


describe("CLOCK.observe - 時刻の飛び（設計の見直し 1）", function()
    --- 秒の合図を順に送る
    local function seconds(CLOCK, list, opts)
        for _, u in ipairs(list) do
            expect(CLOCK.observe(mk_req("OnSecondChange", u, opts))):toBe(false)
        end
    end

    test("差 1 秒・最初の 1 回・境目（+90・−2）は飛びにならない", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE, BASE + 1, BASE + 2, BASE + 92, BASE + 90 })
            local st = CLOCK._get_state()
            expect(st.epoch):toBe(0)
            expect(st.jump_total):toBe(0)
            expect(st.prev_now):toBe(BASE + 90)
        end)
    end)

    test("+91 秒・−3 秒で epoch が増え、jump_total が差から 1 を引いた分だけ増える", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE, BASE + 91 })
            expect(CLOCK.epoch()):toBe(1)
            expect(CLOCK._get_state().jump_total):toBe(90)
            seconds(CLOCK, { BASE + 88 })
            expect(CLOCK.epoch()):toBe(2)
            expect(CLOCK._get_state().jump_total):toBe(86)
        end)
    end)

    test("OnSysResume の後の 1 回の飛びは数えず、その次からは数える", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            expect(CLOCK.observe(mk_req("OnSysResume", BASE + 1))):toBe(false)
            expect(CLOCK._get_state().resumed):toBe(true)
            seconds(CLOCK, { BASE + 3600 })
            expect(CLOCK.epoch()):toBe(0)
            expect(CLOCK._get_state().jump_total):toBe(0)
            expect(CLOCK._get_state().resumed):toBe(false)
            seconds(CLOCK, { BASE + 7200 })
            expect(CLOCK.epoch()):toBe(1)
            expect(CLOCK._get_state().jump_total):toBe(3599)
        end)
    end)

    test("OnSysResume の後でも飛ばなかった秒の合図で印は消える", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            CLOCK.observe(mk_req("OnSysResume", BASE + 1))
            seconds(CLOCK, { BASE + 1, BASE + 3600 })
            expect(CLOCK.epoch()):toBe(1)
        end)
    end)

    -- 実機の順番（2026-10-18・SSP 2.9.18）: 最後の秒 → スリープ → OnSysSuspend（復帰してから届く）
    -- → 最初の秒（+142 秒）→ 4 秒後に OnSysResume → 次の秒
    test("実機の順番: OnSysSuspend の後の飛びは数えず、遅れて届く OnSysResume の後も印が残らない", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            expect(CLOCK.observe(mk_req("OnSysSuspend", BASE + 141))):toBe(false)
            expect(CLOCK._get_state().suspended):toBe(true)
            seconds(CLOCK, { BASE + 142, BASE + 143, BASE + 144, BASE + 145, BASE + 146 })
            expect(CLOCK.epoch()):toBe(0)
            expect(CLOCK._get_state().jump_total):toBe(0)
            expect(CLOCK._get_state().suspended):toBe(false)
            CLOCK.observe(mk_req("OnSysResume", BASE + 146))
            seconds(CLOCK, { BASE + 147 })
            expect(CLOCK._get_state().resumed):toBe(false)
            expect(CLOCK._get_state().suspended):toBe(false)
            seconds(CLOCK, { BASE + 3600 }) -- その後の設定し直しは数える
            expect(CLOCK.epoch()):toBe(1)
        end)
    end)

    test("OnSysSuspend の印は眠る前の飛ばない秒では消えず、次の飛びを 1 回だけ見逃す", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            CLOCK.observe(mk_req("OnSysSuspend", BASE))
            seconds(CLOCK, { BASE + 1, BASE + 2, BASE + 600 })
            expect(CLOCK.epoch()):toBe(0)
            seconds(CLOCK, { BASE + 4200 })
            expect(CLOCK.epoch()):toBe(1)
        end)
    end)

    test("OnSysSuspend の後にスリープせず OnSysResume が来たら、飛ばない秒で両方の印が消える", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            CLOCK.observe(mk_req("OnSysSuspend", BASE))
            CLOCK.observe(mk_req("OnSysResume", BASE + 1))
            seconds(CLOCK, { BASE + 1 })
            expect(CLOCK._get_state().suspended):toBe(false)
            expect(CLOCK._get_state().resumed):toBe(false)
            seconds(CLOCK, { BASE + 3600 })
            expect(CLOCK.epoch()):toBe(1)
        end)
    end)

    test("X-Pasta-Time 付きは飛びを見ず、prev_now も変えない", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            seconds(CLOCK, { BASE + 3600, BASE - 3600 }, { dic = XPT })
            expect(CLOCK.epoch()):toBe(0)
            expect(CLOCK._get_state().prev_now):toBe(BASE)
            seconds(CLOCK, { BASE + 1 })
            expect(CLOCK.epoch()):toBe(0)
        end)
    end)

    test("ずれが変わった次の秒の合図も飛びとして数える", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE })
            CLOCK.observe(mk_reply(BASE + 1, BASE + 1 + 3600))
            seconds(CLOCK, { BASE + 1 })
            expect(CLOCK.epoch()):toBe(1)
            expect(CLOCK._get_state().jump_total):toBe(3600)
        end)
    end)

    test("飛びは毎分・毎時の合図の予約を変えない", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE, BASE + 3630 })
            expect(CLOCK._get_state().probe_armed):toBe(false)
            CLOCK.observe(mk_req("OnMinuteChange", BASE + 3630))
            expect(CLOCK._get_state().probe_armed):toBe(true)
        end)
    end)

    test("_reset は飛びの状態も初期値に戻す", function()
        with_clock(function(CLOCK)
            seconds(CLOCK, { BASE, BASE + 3600 })
            CLOCK.observe(mk_req("OnSysResume", BASE + 3601))
            CLOCK._reset()
            expect(CLOCK._get_state()):toEqual({
                offset = 0, probe_armed = false, epoch = 0, jump_total = 0, resumed = false, suspended = false,
            })
            expect(CLOCK.epoch()):toBe(0)
        end)
    end)
end)

describe("CLOCK.steady - なめらかな時計（設計の見直し 1）", function()
    test("飛びが無ければ now と同じ、飛んだら now − jump_total", function()
        with_clock(function(CLOCK)
            expect(CLOCK.steady(mk_req("OnSecondChange", BASE))):toBe(BASE)
            CLOCK.observe(mk_req("OnSecondChange", BASE))
            CLOCK.observe(mk_req("OnSecondChange", BASE + 3 * 3600))
            expect(CLOCK.steady(mk_req("OnSecondChange", BASE + 3 * 3600))):toBe(BASE + 1)
            expect(CLOCK.steady(mk_req("OnSecondChange", BASE + 3 * 3600 + 46))):toBe(BASE + 47)
        end)
    end)

    test("ずれも足した今から飛びを引く", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_req("OnSecondChange", BASE))
            CLOCK.observe(mk_reply(BASE, BASE + 600))
            CLOCK.observe(mk_req("OnSecondChange", BASE + 1))
            expect(CLOCK.now(mk_req("OnSecondChange", BASE + 1))):toBe(BASE + 601)
            expect(CLOCK.steady(mk_req("OnSecondChange", BASE + 1))):toBe(BASE + 1)
        end)
    end)

    test("X-Pasta-Time 付きは req.date.unix をそのまま返す", function()
        with_clock(function(CLOCK)
            CLOCK.observe(mk_req("OnSecondChange", BASE))
            CLOCK.observe(mk_req("OnSecondChange", BASE + 3600))
            expect(CLOCK.steady(mk_req("OnSecondChange", BASE + 5, { dic = XPT }))):toBe(BASE + 5)
        end)
    end)

    test("req.date（か unix）が無ければ nil", function()
        with_clock(function(CLOCK)
            expect(CLOCK.steady(mk_req("OnSecondChange", nil))):toBeNil()
            local req = mk_req("OnSecondChange", BASE)
            req.date.unix = nil
            expect(CLOCK.steady(req)):toBeNil()
        end)
    end)
end)
