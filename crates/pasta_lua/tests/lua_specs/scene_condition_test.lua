-- ============================================================================
-- イベントのシーンを選ぶ前の自動転記のテスト (scene-condition-attributes 3.2)
-- Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 関連モジュールを読み直し、REG を空にし、時計を初期状態に戻す
local function reload()
    package.loaded["pasta.store"] = nil
    package.loaded["pasta.shiori.event"] = nil
    package.loaded["pasta.shiori.event.callback"] = nil
    package.loaded["pasta.shiori.event.register"] = nil
    package.loaded["pasta.shiori.event.boot"] = nil
    package.loaded["pasta.shiori.event.second_change"] = nil
    package.loaded["pasta.shiori.event.sys_resume"] = nil

    local STORE = require("pasta.store")
    STORE.reset()
    STORE.actors = { sakura = { name = "さくら", spot = "sakura" } }
    local REG = require("pasta.shiori.event.register")
    for k in pairs(REG) do
        REG[k] = nil
    end
    local EVENT = require("pasta.shiori.event")
    require("pasta.shiori.clock")._reset()
    return EVENT, REG, STORE
end

--- 2026-10-18（日）12:34:56 の req.date（秋・節句なし）
local function req_of(id, reference)
    local date = { year = 2026, month = 10, day = 18, hour = 12, min = 34, sec = 56, wday = 0, ns = 0 }
    date.unix = os.time(date)
    return { id = id, base_id = "OnBase", reference = reference, date = date }
end

describe("EVENT.fire - 選ぶ前の自動転記", function()
    test("底のハンドラの act に Reference と日時が最初から入っている", function()
        local EVENT, REG = reload()
        local seen
        REG.OnUpdateComplete = function(act)
            seen = act.var
        end

        EVENT.fire(req_of("OnUpdateComplete", { [0] = "changed", "x" }))

        expect(seen["ｒ０"]):toBe("changed")
        expect(seen.r0):toBe("changed")
        expect(seen.r1):toBe("x")
        expect(seen.req_id):toBe("OnUpdateComplete")
        expect(seen.req_base_id):toBe("OnBase")
        expect(seen.year):toBe(2026)
        expect(seen.hour):toBe(12)
        expect(seen["時"]):toBe("12時")
        expect(seen["季節"]):toBe("秋")
        expect(seen["節句"]):toBe("なし")
    end)

    test("鎖のハンドラごとに作る act にも同じ組が入る", function()
        local EVENT, REG = reload()
        local seen = {}
        REG:register("OnBoot", function(act)
            seen[#seen + 1] = act.var
        end)
        REG:register("OnBoot", function(act)
            seen[#seen + 1] = act.var
        end)

        EVENT.fire(req_of("OnBoot", { [0] = "a" }))

        expect(#seen):toBe(2)
        for _, var in ipairs(seen) do
            expect(var.r0):toBe("a")
            expect(var.hour):toBe(12)
            expect(var["曜日"]):toBe("日曜日")
        end
    end)

    test("本文の代入は自動転記より勝つ", function()
        local EVENT, REG = reload()
        local seen
        REG.OnBoot = function(act)
            act.var["季節"] = "春"
            seen = act.var
        end

        EVENT.fire(req_of("OnBoot"))

        expect(seen["季節"]):toBe("春")
    end)

    test("本文で 2 つの転記の関数を呼んでも今までどおり転記する", function()
        local EVENT, REG = reload()
        local seen
        REG.OnBoot = function(act)
            act.var.r0 = "上書き"
            act:transfer_req_to_var()
            act:transfer_date_to_var()
            seen = act.var
        end

        EVENT.fire(req_of("OnBoot", { [0] = "a" }))

        expect(seen.r0):toBe("a")
        expect(seen.min):toBe(34)
    end)

    test("チェイントークの続きの変数は新しい act の転記で上書きされない", function()
        local EVENT, REG, STORE = reload()
        REG.OnFirst = function()
            return coroutine.create(function(act)
                local var = act.var
                var.r0 = "最初"
                var["季節"] = "自作"
                coroutine.yield("一つ目")
                return var.r0 .. "・" .. var["季節"]
            end)
        end
        REG.OnNext = function()
            return STORE.co_scene
        end

        EVENT.fire(req_of("OnFirst", { [0] = "first" }))
        local res = EVENT.fire(req_of("OnNext", { [0] = "next" }))

        expect(res:find("最初・自作", 1, true) ~= nil):toBe(true)
    end)
end)

-- ============================================================================
-- シーン検索に act を運ぶ・段の通り抜け・Call の条件漏れ (scene-condition-attributes 3.3)
-- Requirements: 2.1, 2.2, 2.3, 2.4, 2.9, 2.10, 2.11, 3.1, 3.4, 3.5
-- ============================================================================

local SCENE = require("pasta.scene")
local ACT = require("pasta.act")
local FAILURE = require("pasta.failure")
local GLOBAL = require("pasta.global")

--- SCENE.search を差し替えて body を走らせ、元に戻す
--- @param results table scope（ローカルは gsn・グローバルは "global"）ごとの戻り値 { result, miss }
local function with_search(results, body)
    local original = SCENE.search
    local calls = {}
    SCENE.search = function(key, scope, act)
        calls[#calls + 1] = { key = key, scope = scope, act = act }
        local r = results[scope or "global"] or {}
        return r[1], r[2]
    end
    local ok, err = pcall(body, calls)
    SCENE.search = original
    if not ok then error(err, 0) end
end

local function found(fn)
    return { { func = fn } }
end
local UNMET = { nil, "unmet" }

--- 放課後のシーンの中にいる act
local function act_in_scene()
    local act = ACT.new({})
    act.current_scene = { __global_name__ = "放課後_1" }
    act.scene_name = "放課後"
    return act
end

--- @pasta_search を search_scene が引数を控えて ret を返すモックに差し替えて body を走らせる
local function with_pasta_search(ret, body)
    local saved = package.loaded["@pasta_search"]
    local seen = {}
    package.loaded["@pasta_search"] = {
        search_scene = function(_, ...)
            seen.args = table.pack(...)
            return table.unpack(ret, 1, 2)
        end,
        search_word = function() return nil end,
    }
    local ok, err = pcall(body, seen)
    package.loaded["@pasta_search"] = saved
    if not ok then error(err, 0) end
end

describe("SCENE.search - act を運ぶ", function()
    local with_mock = with_pasta_search

    test("act のローカル変数・保存変数を検索の口に渡す", function()
        with_mock({ nil }, function(seen)
            local act = ACT.new({})
            SCENE.search("告白", "放課後_1", act)
            expect(seen.args[1]):toBe("告白")
            expect(seen.args[2]):toBe("放課後_1")
            expect(seen.args[3]):toBe(act.var)
            expect(seen.args[4]):toBe(act.save)
        end)
    end)

    test("act が無ければ表を渡さない", function()
        with_mock({ nil }, function(seen)
            SCENE.search("告白", nil)
            expect(seen.args[3]):toBe(nil)
            expect(seen.args[4]):toBe(nil)
        end)
    end)

    test("条件漏れは nil, \"unmet\"", function()
        with_mock({ nil, "unmet" }, function()
            local result, miss = SCENE.search("告白", nil, ACT.new({}))
            expect(result):toBe(nil)
            expect(miss):toBe("unmet")
        end)
    end)

    test("名前が無ければ 2 つ目の値は無い", function()
        with_mock({ nil }, function()
            local result, miss = SCENE.search("告白", nil, ACT.new({}))
            expect(result):toBe(nil)
            expect(miss):toBe(nil)
        end)
    end)
end)

describe("find_act_handler - 条件漏れと段の通り抜け", function()
    test("L2・L5 とも条件漏れなら nil, \"unmet\"・検索に act を渡す", function()
        with_search({ ["放課後_1"] = UNMET, global = UNMET }, function(calls)
            local act = act_in_scene()
            local handler, miss = act:find_act_handler("scene", "告白")
            expect(handler):toBe(nil)
            expect(miss):toBe("unmet")
            expect(#calls):toBe(2)
            expect(calls[1].act):toBe(act)
            expect(calls[2].act):toBe(act)
        end)
    end)

    test("L2 だけ条件漏れで L5 は名前が無くても \"unmet\"", function()
        with_search({ ["放課後_1"] = UNMET }, function()
            local _, miss = act_in_scene():find_act_handler("scene", "告白")
            expect(miss):toBe("unmet")
        end)
    end)

    test("L5 だけ条件漏れでも \"unmet\"", function()
        with_search({ global = UNMET }, function()
            local _, miss = act_in_scene():find_act_handler("scene", "告白")
            expect(miss):toBe("unmet")
        end)
    end)

    test("L2 が条件漏れでも L5 で見つかればそれを返す", function()
        local fn = function() end
        with_search({ ["放課後_1"] = UNMET, global = found(fn) }, function()
            local handler, miss = act_in_scene():find_act_handler("scene", "告白")
            expect(handler):toBe(fn)
            expect(miss):toBe(nil)
        end)
    end)

    test("L2 が条件漏れでも L4（GLOBAL）で見つかればそれを返す", function()
        local fn = function() end
        GLOBAL["告白"] = fn
        local ok, err = pcall(with_search, { ["放課後_1"] = UNMET, global = UNMET }, function()
            local handler, miss = act_in_scene():find_act_handler("scene", "告白")
            expect(handler):toBe(fn)
            expect(miss):toBe(nil)
        end)
        GLOBAL["告白"] = nil
        if not ok then error(err, 0) end
    end)

    test("どの段も名前が無ければ 2 つ目の値は無い", function()
        with_search({}, function()
            local handler, miss = act_in_scene():find_act_handler("scene", "告白")
            expect(handler):toBe(nil)
            expect(miss):toBe(nil)
        end)
    end)
end)

describe("act:call - 条件漏れは止まる・名前が無いのは今のまま", function()
    test("条件漏れは 13 種目のラベルの記録を投げる", function()
        with_search({ ["放課後_1"] = UNMET, global = UNMET }, function()
            local act = act_in_scene()
            local ok, err = pcall(act.call, act, nil, "告白", nil)
            expect(ok):toBe(false)
            expect(FAILURE.is_raised(err)):toBe(true)
            expect(err.label):toBe(FAILURE.LABEL.SCENE_UNMET)
            expect(err.subject):toBe("＞告白")
        end)
    end)

    test("名前が無ければ止まらずに今の「シーンが見つからない」を積む", function()
        with_search({}, function()
            local act = act_in_scene()
            local ok = pcall(act.call, act, nil, "告白", nil)
            expect(ok):toBe(true)
            local last = act.token[#act.token]
            expect(last.type):toBe("raw_script")
            expect(last.text):toBe("【シーンが見つからない：＞告白】")
        end)
    end)

    test("＠名前（）がシーンに当たる道は条件漏れでも今の「関数が見つからない」", function()
        with_search({ ["放課後_1"] = UNMET, global = UNMET }, function()
            local act = act_in_scene()
            local ok = pcall(act.expr_fn, act, "告白")
            expect(ok):toBe(true)
            expect(act.token[#act.token].text):toBe("【関数が見つからない：＠告白（）】")
        end)
    end)

    test("SHIORI.request: シーンの中の Call の条件漏れは止まったシーン付きの表記の応答", function()
        -- reload() で読み直した EVENT・REG を entry にも使わせる
        local _, REG = reload()
        package.loaded["pasta.shiori.entry"] = nil
        local ENTRY = require("pasta.shiori.entry")
        REG.OnScaCallUnmet = function(act)
            return coroutine.create(function()
                act:init_scene({ __global_name__ = "放課後_1" }, "放課後")
                act:call(nil, "告白", nil)
            end)
        end
        -- 検索の口を差し替える（entry の act が引く pasta.scene は他のスイートの読み直しで別物のことがある）
        local res_ok, res
        with_pasta_search({ nil, "unmet" }, function()
            res_ok, res = pcall(ENTRY.request, {
                id = "OnScaCallUnmet", method = "get", version = 30, reference = {}, security_level = "local",
            })
        end)
        REG.OnScaCallUnmet = nil
        expect(res_ok):toBe(true)
        expect(res:find("【条件に合うシーンが無い：＞告白（＊放課後）】", 1, true) ~= nil):toBe(true)
    end)
end)
