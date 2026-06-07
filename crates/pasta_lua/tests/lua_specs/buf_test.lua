-- pasta.buf module tests
-- Tests for pasta.buf - string buffer abstraction (LuaJIT string.buffer or fallback)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local buf = require("pasta.buf")

-- ============================================================================
-- Requirement 1: バッファ抽象モジュール pasta.buf の提供
-- ============================================================================

describe("pasta.buf - new()", function()
    test("new() が put/tostring を持つ object を返す (1.2)", function()
        local b = buf.new()
        -- ネイティブ採用時は userdata、フォールバック時は table。
        -- いずれも put/tostring メソッドを備える（型ではなく契約を検証）。
        expect(b):toBeTruthy()
        expect(type(b.put)):toBe("function")
        expect(type(b.tostring)):toBe("function")
    end)

    test("new() の put → tostring が追記順を保持する (1.4, 1.5)", function()
        local b = buf.new()
        b:put("ab")
        b:put("c")
        expect(b:tostring()):toBe("abc")
    end)

    test("put が self を返す（チェーン可）(1.2)", function()
        local b = buf.new()
        expect(b:put("x")):toBe(b)
    end)
end)

-- ============================================================================
-- Requirement 2: 最小実装フォールバック
-- ============================================================================

describe("pasta.buf - new_fallback()", function()
    test("new_fallback() が put/tostring を持つ object を返す (2.3)", function()
        local b = buf.new_fallback()
        expect(type(b.put)):toBe("function")
        expect(type(b.tostring)):toBe("function")
    end)

    test("new_fallback() で put('ab')->put('c')->tostring() が 'abc' (2.4)", function()
        local b = buf.new_fallback()
        b:put("ab")
        b:put("c")
        expect(b:tostring()):toBe("abc")
    end)

    test("new_fallback() 空バッファの tostring() が '' (空連結)", function()
        local b = buf.new_fallback()
        expect(b:tostring()):toBe("")
    end)

    test("new_fallback() の put が self を返す（チェーン可）(2.3)", function()
        local b = buf.new_fallback()
        expect(b:put("x")):toBe(b)
    end)

    test("new_fallback() の tostring() が非破壊（複数回呼べる）(2.4)", function()
        local b = buf.new_fallback()
        b:put("a")
        b:put("b")
        expect(b:tostring()):toBe("ab")
        expect(b:tostring()):toBe("ab")
        b:put("c")
        expect(b:tostring()):toBe("abc")
    end)
end)

-- ============================================================================
-- Requirement 5: 対象ランタイムでの String Buffer 採用検証
-- ============================================================================

describe("pasta.buf - backend", function()
    test("backend == 'luajit'（テストランタイムでネイティブ採用）(5.1)", function()
        expect(buf.backend):toBe("luajit")
    end)
end)
