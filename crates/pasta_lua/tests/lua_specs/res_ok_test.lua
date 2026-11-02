-- ============================================================================
-- Task 6: RES.ok() の空文字列処理拡張テスト
-- Requirements: 9.1, 9.2, 9.3
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("RES.ok - nil and empty string handling", function()
    local RES

    local function setup()
        -- Reset module
        package.loaded["pasta.shiori.res"] = nil
        RES = require("pasta.shiori.res")
    end

    -- -------------------------------------------------------------------------
    -- Task 6.1: nil/空文字列チェック
    -- -------------------------------------------------------------------------

    test("returns no_content when value is nil", function()
        setup()

        local result = RES.ok(nil)

        -- Should return 204 No Content
        expect(result:find("204 No Content")).not_:toBe(nil)
        -- Should NOT contain Value header
        expect(result:find("Value:")):toBe(nil)
    end)

    test("returns no_content when value is empty string", function()
        setup()

        local result = RES.ok("")

        -- Should return 204 No Content
        expect(result:find("204 No Content")).not_:toBe(nil)
        -- Should NOT contain Value header
        expect(result:find("Value:")):toBe(nil)
    end)

    test("returns 200 OK with Value header when value is valid string", function()
        setup()

        local result = RES.ok("Hello World")

        -- Should return 200 OK
        expect(result:find("200 OK")).not_:toBe(nil)
        -- Should contain Value header
        expect(result:find("Value: Hello World")).not_:toBe(nil)
    end)

    test("preserves additional headers when returning no_content for nil", function()
        setup()

        local result = RES.ok(nil, { Reference0 = "test" })

        -- Should return 204 No Content
        expect(result:find("204 No Content")).not_:toBe(nil)
        -- Should contain additional header
        expect(result:find("Reference0: test")).not_:toBe(nil)
    end)

    test("preserves additional headers when returning no_content for empty string", function()
        setup()

        local result = RES.ok("", { Reference0 = "test" })

        -- Should return 204 No Content
        expect(result:find("204 No Content")).not_:toBe(nil)
        -- Should contain additional header
        expect(result:find("Reference0: test")).not_:toBe(nil)
    end)

    test("returns 200 OK with additional headers for valid string", function()
        setup()

        local result = RES.ok("Response", { Reference0 = "extra" })

        -- Should return 200 OK
        expect(result:find("200 OK")).not_:toBe(nil)
        -- Should contain Value header
        expect(result:find("Value: Response")).not_:toBe(nil)
        -- Should contain additional header
        expect(result:find("Reference0: extra")).not_:toBe(nil)
    end)

    test("whitespace-only string is treated as valid (not empty)", function()
        setup()

        local result = RES.ok("  ")

        -- Whitespace-only should be treated as valid value (not empty)
        -- This is intentional - only nil and "" are special-cased
        expect(result:find("200 OK")).not_:toBe(nil)
        expect(result:find("Value:   ")).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- ponytail-lua-scripts 3.1: 見出しの値の改行を空白に置き換え、渡された表を書き換えない
-- Requirements: 2.1, 2.2, 2.3, 2.6, 2.7
-- ============================================================================

describe("RES - 見出しの値の改行と渡された表", function()
    local RES

    local function setup()
        package.loaded["pasta.shiori.res"] = nil
        RES = require("pasta.shiori.res")
    end

    local HEAD = "Charset: UTF-8\r\nSender: Pasta\r\nSecurityLevel: local\r\n"

    test("Value の LF・CR LF・CR を 1 文字ずつ空白にし、行の数を増やさない", function()
        setup()
        local cases = {
            { "上\n下", "上 下" },
            { "上\r\n下", "上  下" },
            { "a\rb", "a b" },
        }
        for _, c in ipairs(cases) do
            expect(RES.ok(c[1])):toBe("SHIORI/3.0 200 OK\r\n" .. HEAD .. "Value: " .. c[2] .. "\r\n\r\n")
        end
    end)

    test("改行の無い値は今と同じ文字列", function()
        setup()
        expect(RES.ok([[\0\s[0]こんにちは\e]]))
            :toBe("SHIORI/3.0 200 OK\r\n" .. HEAD .. [[Value: \0\s[0]こんにちは\e]] .. "\r\n\r\n")
        expect(RES.err("理由")):toBe("SHIORI/3.0 500 Internal Server Error\r\n" .. HEAD .. "X-Error-Reason: 理由\r\n\r\n")
        expect(RES.warn("注意")):toBe("SHIORI/3.0 204 No Content\r\n" .. HEAD .. "X-Warn-Reason: 注意\r\n\r\n")
    end)

    test("err・warn の理由と追加の見出し・標準の見出しの改行も空白になる", function()
        setup()
        expect(RES.err("一\n二"))
            :toBe("SHIORI/3.0 500 Internal Server Error\r\n" .. HEAD .. "X-Error-Reason: 一 二\r\n\r\n")
        expect(RES.warn("一\r\n二")):toBe("SHIORI/3.0 204 No Content\r\n" .. HEAD .. "X-Warn-Reason: 一  二\r\n\r\n")
        expect(RES.no_content({ Reference0 = "x\ny" }))
            :toBe("SHIORI/3.0 204 No Content\r\n" .. HEAD .. "Reference0: x y\r\n\r\n")
        -- 停止とエラーの応答は RES.build を直接呼ぶ
        expect(RES.build("200 OK", { Value = "v\r" }))
            :toBe("SHIORI/3.0 200 OK\r\n" .. HEAD .. "Value: v \r\n\r\n")
        RES.env.sender = "P\na"
        expect(RES.no_content())
            :toBe("SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: P a\r\nSecurityLevel: local\r\n\r\n")
        RES.env.sender = "Pasta"
    end)

    test("RES.ok・err・warn は渡された表を書き換えない", function()
        setup()
        local H = { Reference0 = "x" }
        local r = RES.ok("v", H)
        expect(r:find("Value: v", 1, true)).not_:toBe(nil)
        expect(r:find("Reference0: x", 1, true)).not_:toBe(nil)
        expect(H.Value):toBe(nil)
        expect(RES.no_content(H):find("Value:", 1, true)):toBe(nil)

        RES.err("e", H)
        expect(H["X-Error-Reason"]):toBe(nil)
        RES.warn("w", H)
        expect(H["X-Warn-Reason"]):toBe(nil)
        expect(RES.no_content(H)):toBe("SHIORI/3.0 204 No Content\r\n" .. HEAD .. "Reference0: x\r\n\r\n")
    end)
end)
