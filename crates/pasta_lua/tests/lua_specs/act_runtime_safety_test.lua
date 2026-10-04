-- 生成コードが呼ぶ存在確認付き act メソッドのランタイムテスト (dsl-codegen-runtime-safety)
-- act:global_fn: ＠＊名前（…）の呼び出し（未定義・関数でないときは警告して値なし）
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local GLOBAL = require("pasta.global")

--- 警告を記録するログを差し込んだ pasta.act を新規ロードして body を実行し、
--- 実行後に元のモジュールと GLOBAL のテスト用キーを戻す（他スイートを汚さない）
--- @param body fun(ACT: table, warns: string[])
local function with_captured_act(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_act = package.loaded["pasta.act"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["pasta.act"] = nil
    local ok, err = pcall(function()
        body(require("pasta.act"), warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.act"] = saved_act
    GLOBAL["rs_関数"] = nil
    GLOBAL["rs_値"] = nil
    GLOBAL["rs_失敗"] = nil
    if not ok then error(err, 0) end
end

describe("act:global_fn - 定義済みのグローバル関数", function()
    test("act を第 1 引数、続けて引数を渡して呼び、戻り値をすべて返す（警告なし）", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local got_self, got_a, got_b
            GLOBAL["rs_関数"] = function(a, x, y)
                got_self, got_a, got_b = a, x, y
                return x + y, "二つ目"
            end
            local r1, r2 = act:global_fn("rs_関数", 1, 2)
            expect(r1):toBe(3)
            expect(r2):toBe("二つ目")
            expect(got_self):toBe(act)
            expect(got_a):toBe(1)
            expect(got_b):toBe(2)
            expect(#warns):toBe(0)
        end)
    end)

    test("関数の中で起きたエラーは握りつぶさず伝わる", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            GLOBAL["rs_失敗"] = function() error("中のエラー") end
            local ok, err = pcall(act.global_fn, act, "rs_失敗")
            expect(ok):toBe(false)
            expect(tostring(err):find("中のエラー", 1, true) ~= nil):toBe(true)
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("act:global_fn - 未定義・関数でない値", function()
    test("未定義は値なしを返し、関数名を含む警告を 1 行出す", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            expect(select("#", act:global_fn("rs_関数", 1))):toBe(1)
            expect(act:global_fn("rs_関数", 1)):toBeNil()
            expect(#warns):toBe(2)
            expect(warns[1]):toBe("act:global_fn - function not found: key='rs_関数'")
        end)
    end)

    test("関数でない値は値なしを返し、関数名を含む警告を 1 行出す", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            GLOBAL["rs_値"] = "文字列"
            expect(act:global_fn("rs_値")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:global_fn - function not found: key='rs_値'")
        end)
    end)

    test("警告は act:expr_fn の未検出と同じレベル（warn）で出る", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            act:expr_fn("rs_存在しない")
            local expr_warned = false
            for _, w in ipairs(warns) do
                if w:find("act:expr_fn", 1, true) then expr_warned = true end
            end
            expect(expr_warned):toBe(true)
            local before = #warns
            act:global_fn("rs_関数")
            expect(#warns):toBe(before + 1)
            expect(warns[#warns]):toBe("act:global_fn - function not found: key='rs_関数'")
        end)
    end)
end)
