-- PASTA.str のランタイムテスト (string-concat-operator / expr-nil-coercion)
-- 連結式（＆）の生成コードが被演算子を通す変換。文字列はそのまま、数値はアクション行の表示と同じ表記、
-- nil は黙って空文字列、それ以外は警告 1 行と空文字列
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 警告を記録するログを差し込んだ pasta.act を新規ロードして body を実行し、
--- 実行後に元のモジュールを戻す（他スイートを汚さない）
--- logs には全レベル（trace〜error）のログを "<レベル>: <文言>" で記録する
--- @param body fun(ACT: table, warns: string[], logs: string[])
local function with_captured_act(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_act = package.loaded["pasta.act"]
    local warns = {}
    local logs = {}
    local function record(level)
        return function(msg)
            table.insert(logs, level .. ": " .. tostring(msg))
            if level == "warn" then table.insert(warns, msg) end
        end
    end
    package.loaded["@pasta_log"] = {
        trace = record("trace"),
        debug = record("debug"),
        info = record("info"),
        warn = record("warn"),
        error = record("error"),
    }
    package.loaded["pasta.act"] = nil
    local ok, err = pcall(function()
        body(require("pasta.act"), warns, logs)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.act"] = saved_act
    if not ok then error(err, 0) end
end

--- アクション行で値を表示したときの文字列（act:talk が積むトークンの text）
--- @param act table
--- @param v any
--- @return string
local function displayed(act, v)
    act:talk(nil, v)
    local tokens = act.token
    act.token = {}
    return tokens[#tokens].text
end

-- PASTA.str: 連結の被演算子を必ず文字列にする変換（連結式の生成コードが pasta モジュールから呼ぶ）

--- with_captured_act と同じくログを記録し、pasta（init.lua）も新規ロードして body に渡す。
--- PASTA.str の初回参照は、差し替え中の pasta.act ではなく、ここで新規ロードした pasta に載る
--- @param body fun(PASTA: table, ACT: table, warns: string[], logs: string[])
local function with_captured_pasta(body)
    local saved_pasta = package.loaded["pasta"]
    package.loaded["pasta"] = nil
    local ok, err = pcall(with_captured_act, function(ACT, warns, logs)
        body(require("pasta"), ACT, warns, logs)
    end)
    package.loaded["pasta"] = saved_pasta
    if not ok then error(err, 0) end
end

describe("PASTA.str - 文字列と数値", function()
    test("文字列はそのまま返す（空文字列・空白・数字だけの文字列も変えない）", function()
        with_captured_pasta(function(PASTA, _, _, logs)
            for _, s in ipairs({ "合計", "", " ", "a ", "01", "0x10", "１２" }) do
                expect(PASTA.str(s)):toBe(s)
                expect(PASTA.str(s, "var.s")):toBe(s)
            end
            expect(#logs):toBe(0)
        end)
    end)

    test("数値はアクション行の表示と同じ表記で文字列にする（2.5）", function()
        with_captured_pasta(function(PASTA, ACT, _, logs)
            local act = ACT.new({})
            local values = { 3, 0, -5, 3.5, 7 / 2, 1 / 3, 1e15, 1e100, 1.5e-10, 2 ^ 53,
                1 / 0, -1 / 0, 0 / 0, -0.0 }
            for _, v in ipairs(values) do
                expect(PASTA.str(v)):toBe(displayed(act, v))
                expect(PASTA.str(v, "var.n")):toBe(displayed(act, v))
            end
            expect(PASTA.str(3)):toBe("3")
            expect(PASTA.str(3.5)):toBe("3.5")
            expect(PASTA.str(1 / 3)):toBe("0.33333333333333")
            expect(PASTA.str(1e15)):toBe("1e+15")
            expect(PASTA.str(1 / 0)):toBe("inf")
            expect("合計" .. PASTA.str(3)):toBe("合計3")
            expect(#logs):toBe(0)
        end)
    end)
end)

describe("PASTA.str - nil と変換できない値", function()
    test("nil は説明の有無にかかわらず空文字列で、どのレベルのログも出さない（3.1）", function()
        with_captured_pasta(function(PASTA, _, _, logs)
            expect(PASTA.str(nil)):toBe("")
            expect(PASTA.str(nil, "var.x")):toBe("")
            expect(PASTA.str(nil, "@f()")):toBe("")
            expect(#logs):toBe(0)
        end)
    end)

    test("真偽値・関数は空文字列＋従来の文言の警告 1 行（4.2・4.3）", function()
        with_captured_pasta(function(PASTA, _, warns, logs)
            expect(PASTA.str(true)):toBe("")
            expect(PASTA.str(false, "var.f")):toBe("")
            expect(PASTA.str(function() end, "@g()")):toBe("")
            expect(#logs):toBe(3)
            expect(warns[1]):toBe("act:concat - operand is not a string or number: op='&', value=true (boolean)")
            expect(warns[2]):toBe(
                "act:concat - operand is not a string or number: op='&', operand='var.f', value=false (boolean)")
            expect(warns[3]):toBe(
                "act:concat - operand is not a string or number: op='&', operand='@g()', value=(function)")
        end)
    end)

    test("表は空文字列＋警告 1 行で、連結・文字列化のメタメソッドを呼ばない", function()
        with_captured_pasta(function(PASTA, _, warns, logs)
            local called = {}
            local mt = {}
            for _, name in ipairs({ "__concat", "__tostring", "__add", "__len", "__eq", "__call", "__index" }) do
                mt[name] = function() table.insert(called, name); return "x" end
            end
            local t = setmetatable({}, mt)
            expect(PASTA.str(t)):toBe("")
            expect(PASTA.str(t, "var.t")):toBe("")
            expect(#called):toBe(0)
            expect(#logs):toBe(2)
            expect(warns[1]):toBe("act:concat - operand is not a string or number: op='&', value=(table)")
            expect(warns[2]):toBe(
                "act:concat - operand is not a string or number: op='&', operand='var.t', value=(table)")
        end)
    end)
end)
