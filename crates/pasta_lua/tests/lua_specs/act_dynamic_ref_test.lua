-- 動的参照（＠＄名前・＠＄名前（…））のランタイムテスト (dynamic-word-reference)
-- WORD.dynamic_key: 参照変数の値 → 検索キー（使えない値は警告して nil）
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 警告を記録するログを差し込んだ pasta.word を新規ロードして body を実行し、
--- 実行後に元のモジュールへ戻す（他スイートが保持する WORD/log を汚さない）
--- @param body fun(WORD: table, warns: string[])
local function with_captured_word(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_word = package.loaded["pasta.word"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["pasta.word"] = nil
    local ok, err = pcall(function()
        body(require("pasta.word"), warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.word"] = saved_word
    if not ok then error(err, 0) end
end

describe("WORD.dynamic_key - 検索キーになる値", function()
    test("空でない文字列はそのまま返し、警告しない", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key("挨拶", "var.x", "act:word")):toBe("挨拶")
            expect(#warns):toBe(0)
        end)
    end)

    test("文字列を DSL として読み直さない（＠・＄を含んでもそのままキーにする）", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key("＠＄y", "var.x", "act:word")):toBe("＠＄y")
            expect(#warns):toBe(0)
        end)
    end)

    test("数値は変数展開（tostring）と同じ表記の文字列にする", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key(1, "var.n", "act:word")):toBe("1")
            expect(WORD.dynamic_key(1.5, "var.n", "act:word")):toBe(tostring(1.5))
            expect(WORD.dynamic_key(-3, "var.n", "act:word")):toBe(tostring(-3))
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("WORD.dynamic_key - 使えない値は nil と区別できる警告", function()
    test("未代入（nil）は undefined variable 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key(nil, "var.x", "act:word")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:word - undefined variable: 'var.x'")
        end)
    end)

    test("空文字列は empty variable 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key("", "save.x", "proxy:word")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("proxy:word - empty variable: 'save.x'")
        end)
    end)

    test("真偽値は型名付きの unsupported value type 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key(true, "args[1]", "act:expr_fn")):toBeNil()
            expect(WORD.dynamic_key(false, "var.x", "act:word")):toBeNil()
            expect(warns[1]):toBe("act:expr_fn - unsupported value type: 'args[1]' (boolean)")
            expect(warns[2]):toBe("act:word - unsupported value type: 'var.x' (boolean)")
        end)
    end)

    test("テーブル・関数・thread は型名付きの unsupported value type 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key({}, "var.t", "proxy:expr_fn")):toBeNil()
            expect(WORD.dynamic_key(function() end, "var.f", "act:word")):toBeNil()
            expect(WORD.dynamic_key(coroutine.create(function() end), "var.c", "act:word")):toBeNil()
            expect(warns[1]):toBe("proxy:expr_fn - unsupported value type: 'var.t' (table)")
            expect(warns[2]):toBe("act:word - unsupported value type: 'var.f' (function)")
            expect(warns[3]):toBe("act:word - unsupported value type: 'var.c' (thread)")
        end)
    end)

    test("__tostring を持つテーブルも型不正（文字列化しない）", function()
        with_captured_word(function(WORD, warns)
            local obj = setmetatable({}, { __tostring = function() return "挨拶" end })
            expect(WORD.dynamic_key(obj, "var.o", "act:word")):toBeNil()
            expect(warns[1]):toBe("act:word - unsupported value type: 'var.o' (table)")
        end)
    end)
end)

describe("WORD.dynamic_key - 依存方向", function()
    test("pasta.word の読み込みで pasta.act・pasta.actor を読み込まない", function()
        local saved = {
            act = package.loaded["pasta.act"],
            actor = package.loaded["pasta.actor"],
        }
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.actor"] = nil
        local ok, err = pcall(function()
            with_captured_word(function(WORD)
                expect(type(WORD.dynamic_key)):toBe("function")
            end)
            expect(package.loaded["pasta.act"]):toBeNil()
            expect(package.loaded["pasta.actor"]):toBeNil()
        end)
        package.loaded["pasta.act"] = saved.act
        package.loaded["pasta.actor"] = saved.actor
        if not ok then error(err, 0) end
    end)
end)
