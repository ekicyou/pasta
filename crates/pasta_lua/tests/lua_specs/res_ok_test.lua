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
