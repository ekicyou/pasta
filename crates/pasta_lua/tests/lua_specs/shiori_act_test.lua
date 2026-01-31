-- SHIORI Act module tests
-- Tests for pasta.shiori.act module - sakura script builder
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock context for testing
local function create_mock_ctx()
    return {
        actors = {
            sakura = { name = "さくら", spot = "sakura" },
            kero = { name = "うにゅう", spot = "kero" },
            char2 = { name = "キャラ2", spot = "char2" },
        },
        save = {},
        yield = function() end,
        end_action = function() end,
    }
end

-- Test inheritance from pasta.act
describe("SHIORI_ACT - inheritance", function()
    test("inherits ACT.IMPL methods", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        -- sakura_script() is inherited from ACT_IMPL
        act:sakura_script("\\e")
        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("sakura_script")
    end)

    test("has IMPL field for further inheritance", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        -- Check IMPL exists and is a table (avoid deep inspection)
        expect(type(SHIORI_ACT.IMPL)):toBe("table")
        -- Check __index is set (use rawget to avoid metatable traversal)
        expect(rawget(SHIORI_ACT.IMPL, "__index") ~= nil):toBe(true)
    end)

    test("inherits word() method", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        -- word() method should be accessible (returns nil for unknown word)
        local result = act:word("unknown_word")
        expect(result):toBe(nil)
    end)
end)

-- Test talk() method override
describe("SHIORI_ACT - talk()", function()
    test("appends scope tag on first actor", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        local result = act:build()

        -- Should contain: \0 (scope) + Hello + \n + \e
        expect(result:find("\\0")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("appends scope tag on actor switch", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        act:talk(ctx.actors.kero, "Hi")
        local result = act:build()

        -- Should contain both scope tags
        expect(result:find("\\0")):toBeTruthy()
        expect(result:find("\\1")):toBeTruthy()
    end)

    test("does not append scope tag on same actor", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        act:talk(ctx.actors.sakura, "World")
        local result = act:build()

        -- \0 should appear only once
        local _, count = result:gsub("\\0", "")
        expect(count):toBe(1)
    end)

    test("uses \\p[N] for char2+ actors", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.char2, "Third character")
        local result = act:build()

        expect(result:find("\\p%[2%]")):toBeTruthy()
    end)

    test("adds newline after scope switch", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        act:talk(ctx.actors.kero, "Hi")
        local result = act:build()

        -- There should be \n before \1 (after first talk)
        expect(result:find("\\n\\1")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local returned = act:talk(ctx.actors.sakura, "Hello")
        expect(returned):toBe(act)
    end)

    test("also updates token buffer (parent behavior)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        expect(#act.token):toBeGraterThan(0)
    end)
end)

-- Test surface() method
describe("SHIORI_ACT - surface()", function()
    test("appends surface tag with number", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:surface(5)
        local result = act:build()

        expect(result):toBe("\\s[5]\\e")
    end)

    test("appends surface tag with alias string", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:surface("smile")
        local result = act:build()

        expect(result):toBe("\\s[smile]\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local returned = act:surface(5)
        expect(returned):toBe(act)
    end)
end)

-- Test wait() method
describe("SHIORI_ACT - wait()", function()
    test("appends wait tag", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:wait(500)
        local result = act:build()

        expect(result):toBe("\\w[500]\\e")
    end)

    test("handles negative values as 0", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:wait(-100)
        local result = act:build()

        expect(result):toBe("\\w[0]\\e")
    end)

    test("truncates float to integer", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:wait(500.7)
        local result = act:build()

        expect(result):toBe("\\w[500]\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local returned = act:wait(500)
        expect(returned):toBe(act)
    end)
end)

-- Test newline() method
describe("SHIORI_ACT - newline()", function()
    test("appends single newline by default", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:newline()
        local result = act:build()

        expect(result):toBe("\\n\\e")
    end)

    test("appends multiple newlines", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:newline(3)
        local result = act:build()

        expect(result):toBe("\\n\\n\\n\\e")
    end)

    test("does nothing for n < 1", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:newline(0)
        act:newline(-1)
        local result = act:build()

        expect(result):toBe("\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local returned = act:newline()
        expect(returned):toBe(act)
    end)
end)

-- Test clear() method
describe("SHIORI_ACT - clear()", function()
    test("appends clear tag", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:clear()
        local result = act:build()

        expect(result):toBe("\\c\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local returned = act:clear()
        expect(returned):toBe(act)
    end)
end)

-- Test build() method
describe("SHIORI_ACT - build()", function()
    test("returns \\e for empty buffer", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local result = act:build()

        expect(result):toBe("\\e")
    end)

    test("appends \\e to end", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:surface(5):wait(100)
        local result = act:build()

        expect(result:sub(-2)):toBe("\\e")
    end)

    test("can be called multiple times", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:surface(5)
        local result1 = act:build()
        local result2 = act:build()

        expect(result1):toBe(result2)
    end)
end)

-- Test reset() method
describe("SHIORI_ACT - reset()", function()
    test("clears buffer", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:surface(5):wait(100)
        act:reset()
        local result = act:build()

        expect(result):toBe("\\e")
    end)

    test("clears scope state", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        act:reset()
        act:talk(ctx.actors.sakura, "Hello again")
        local result = act:build()

        -- After reset, scope tag should be emitted again
        expect(result:find("\\0")):toBeTruthy()
    end)

    test("does not clear token buffer (parent behavior)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "Hello")
        local token_count = #act.token
        act:reset()

        expect(#act.token):toBe(token_count)
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        local returned = act:reset()
        expect(returned):toBe(act)
    end)
end)

-- Test escape processing
describe("SHIORI_ACT - escape", function()
    test("escapes backslash", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "path\\to\\file")
        local result = act:build()

        expect(result:find("path\\\\to\\\\file")):toBeTruthy()
    end)

    test("escapes percent", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "100%")
        local result = act:build()

        expect(result:find("100%%%%")):toBeTruthy()
    end)

    test("escapes mixed special characters", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "50% off \\ sale")
        local result = act:build()

        expect(result:find("50%%%%")):toBeTruthy()
        expect(result:find("\\\\")):toBeTruthy()
    end)
end)

-- E2E scenario test
describe("SHIORI_ACT - E2E scenario", function()
    test("complex script generation", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        act:talk(ctx.actors.sakura, "こんにちは")
            :surface(5)
            :wait(500)
            :talk(ctx.actors.kero, "やあ")
            :clear()

        local result = act:build()

        -- Verify structure
        expect(result:find("\\0")):toBeTruthy()        -- sakura scope
        expect(result:find("こんにちは")):toBeTruthy()
        expect(result:find("\\s%[5%]")):toBeTruthy()   -- surface
        expect(result:find("\\w%[500%]")):toBeTruthy() -- wait
        expect(result:find("\\1")):toBeTruthy()        -- kero scope
        expect(result:find("やあ")):toBeTruthy()
        expect(result:find("\\c")):toBeTruthy()        -- clear
        expect(result:sub(-2)):toBe("\\e")             -- end
    end)

    test("multiple rounds with reset", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx)

        -- First round
        act:talk(ctx.actors.sakura, "First")
        local result1 = act:build()
        expect(result1:find("First")):toBeTruthy()

        -- Reset and second round
        act:reset()
        act:talk(ctx.actors.kero, "Second")
        local result2 = act:build()

        expect(result2:find("First")):toBeFalsy() -- First should be cleared
        expect(result2:find("Second")):toBeTruthy()
        expect(result2:find("\\1")):toBeTruthy()  -- kero scope
    end)
end)
