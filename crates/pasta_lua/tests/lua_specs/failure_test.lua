-- pasta.failure（失敗の報告の唯一の出口）の単体テスト (failure-output-unification 1.2)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 警告を記録するログを差し込んで body を実行し、元のログへ戻す
--- @param body fun(FAILURE: table, warns: string[])
local function with_log(body)
    local saved_log = package.loaded["@pasta_log"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    local ok, err = pcall(body, require("pasta.failure"), warns)
    package.loaded["@pasta_log"] = saved_log
    if not ok then error(err, 0) end
end

describe("FAILURE.report", function()
    test("警告をログへ 1 回出し、【label】を raw_script として積み、nil を返す", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            local ret = FAILURE.report(act, { label = "Call失敗：x", warning = "w1" })
            expect(ret):toBe(nil)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("w1")
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("raw_script")
            expect(act.token[1].text):toBe("【Call失敗：x】")
        end)
    end)

    test("warning が nil ならログを出さず、表記だけ積む", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            FAILURE.report(act, { label = "x" })
            expect(#warns):toBe(0)
            expect(act.token[1].text):toBe("【x】")
        end)
    end)

    test("act が nil なら警告だけ出し、エラーにしない", function()
        with_log(function(FAILURE, warns)
            expect(FAILURE.report(nil, { label = "x", warning = "w" })):toBe(nil)
            expect(#warns):toBe(1)
        end)
    end)

    test("ログはモジュールの読み込み後に差し替えても、呼んだ時点のものを使う", function()
        require("pasta.failure")
        with_log(function(FAILURE, warns)
            FAILURE.report({ token = {} }, { label = "x", warning = "late" })
            expect(warns[1]):toBe("late")
        end)
    end)
end)

describe("act:failure は FAILURE.report へ委ねる", function()
    test("act:failure(text, warning) が同じ警告とトークンを出す", function()
        with_log(function(FAILURE, warns)
            local saved = FAILURE.report
            local seen
            FAILURE.report = function(act, record)
                seen = record
                return saved(act, record)
            end
            local ok, err = pcall(function()
                local act = require("pasta.act").new({})
                expect(act:failure("t", "w")):toBe(nil)
                expect(seen.label):toBe("t")
                expect(seen.warning):toBe("w")
                expect(warns[1]):toBe("w")
                expect(act.token[#act.token].text):toBe("【t】")
            end)
            FAILURE.report = saved
            if not ok then error(err, 0) end
        end)
    end)
end)

describe("依存の向き", function()
    test("pasta.failure は act・actor・word を読み込まない", function()
        local names = { "pasta.failure", "pasta.act", "pasta.actor", "pasta.word" }
        local saved = {}
        for _, n in ipairs(names) do
            saved[n] = package.loaded[n]
            package.loaded[n] = nil
        end
        local ok, err = pcall(function()
            require("pasta.failure")
            expect(package.loaded["pasta.act"]):toBe(nil)
            expect(package.loaded["pasta.actor"]):toBe(nil)
            expect(package.loaded["pasta.word"]):toBe(nil)
        end)
        for _, n in ipairs(names) do
            package.loaded[n] = saved[n]
        end
        if not ok then error(err, 0) end
    end)
end)
