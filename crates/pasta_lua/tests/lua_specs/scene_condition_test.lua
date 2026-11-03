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
