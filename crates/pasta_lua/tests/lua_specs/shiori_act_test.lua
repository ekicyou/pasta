-- SHIORI Act module tests
-- Tests for pasta.shiori.act module - sakura script builder
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock actors for testing
local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = "sakura" },
        kero = { name = "うにゅう", spot = "kero" },
        char2 = { name = "キャラ2", spot = "char2" },
    }
end

-- Create mock context (full CTX-like structure)
local function create_mock_ctx()
    local ACTOR = require("pasta.actor")
    local sakura = ACTOR.get_or_create("さくら")
    sakura.spot = "sakura"
    local kero = ACTOR.get_or_create("うにゅう")
    kero.spot = "kero"
    
    return {
        actors = {
            sakura = sakura,
            kero = kero,
        }
    }
end

-- Test inheritance from pasta.act
describe("SHIORI_ACT - inheritance", function()
    test("inherits ACT.IMPL methods", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

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
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- word() method should be accessible (returns nil for unknown word)
        local result = act:word("unknown_word")
        expect(result):toBe(nil)
    end)

    test("supports actor proxy (act.sakura:talk)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx.actors)

        -- act.sakura should create a proxy that redirects to act:talk(sakura, text)
        act.sakura:talk("Hello via proxy")
        local result = act:build()

        -- Should contain scope tag and text (same as direct call)
        expect(result:find("\\0")):toBeTruthy()
        expect(result:find("Hello via proxy")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("actor proxy supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx.actors)

        -- Proxy talk returns nil, but act methods can chain
        act.sakura:talk("First")
        act:surface(5)
        act.kero:talk("Second")
        local result = act:build()

        expect(result:find("\\0")):toBeTruthy()
        expect(result:find("First")):toBeTruthy()
        expect(result:find("\\s%[5%]")):toBeTruthy()
        expect(result:find("\\1")):toBeTruthy()
        expect(result:find("Second")):toBeTruthy()
    end)
end)

-- Test talk() method override
describe("SHIORI_ACT - talk()", function()
    test("appends scope tag on first actor", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        local result = act:build()

        -- Should contain: \0 (scope) + Hello + \n + \e
        expect(result:find("\\0")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("appends scope tag on actor switch", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.kero, "Hi")
        local result = act:build()

        -- Should contain both scope tags
        expect(result:find("\\0")):toBeTruthy()
        expect(result:find("\\1")):toBeTruthy()
    end)

    test("does not append scope tag on same actor", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.sakura, "World")
        local result = act:build()

        -- \0 should appear only once
        local _, count = result:gsub("\\0", "")
        expect(count):toBe(1)
    end)

    test("uses \\p[N] for char2+ actors", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.char2, "Third character")
        local result = act:build()

        expect(result:find("\\p%[2%]")):toBeTruthy()
    end)

    test("adds newline after scope switch", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.kero, "Hi")
        local result = act:build()

        -- There should be \n before \1 (after first talk)
        expect(result:find("\\n\\1")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:talk(actors.sakura, "Hello")
        expect(returned):toBe(act)
    end)

    test("also updates token buffer (parent behavior)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        expect(#act.token):toBeGraterThan(0)
    end)
end)

-- Test surface() method
describe("SHIORI_ACT - surface()", function()
    test("appends surface tag with number", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:surface(5)
        local result = act:build()

        expect(result):toBe("\\s[5]\\e")
    end)

    test("appends surface tag with alias string", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:surface("smile")
        local result = act:build()

        expect(result):toBe("\\s[smile]\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:surface(5)
        expect(returned):toBe(act)
    end)
end)

-- Test wait() method
describe("SHIORI_ACT - wait()", function()
    test("appends wait tag", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:wait(500)
        local result = act:build()

        expect(result):toBe("\\w[500]\\e")
    end)

    test("handles negative values as 0", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:wait(-100)
        local result = act:build()

        expect(result):toBe("\\w[0]\\e")
    end)

    test("truncates float to integer", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:wait(500.7)
        local result = act:build()

        expect(result):toBe("\\w[500]\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:wait(500)
        expect(returned):toBe(act)
    end)
end)

-- Test newline() method
describe("SHIORI_ACT - newline()", function()
    test("appends single newline by default", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:newline()
        local result = act:build()

        expect(result):toBe("\\n\\e")
    end)

    test("appends multiple newlines", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:newline(3)
        local result = act:build()

        expect(result):toBe("\\n\\n\\n\\e")
    end)

    test("does nothing for n < 1", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:newline(0)
        act:newline(-1)
        local result = act:build()

        expect(result):toBe("\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:newline()
        expect(returned):toBe(act)
    end)
end)

-- Test clear() method
describe("SHIORI_ACT - clear()", function()
    test("appends clear tag", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:clear()
        local result = act:build()

        expect(result):toBe("\\c\\e")
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:clear()
        expect(returned):toBe(act)
    end)
end)

-- Test build() method
describe("SHIORI_ACT - build()", function()
    test("returns \\e for empty buffer", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local result = act:build()

        expect(result):toBe("\\e")
    end)

    test("appends \\e to end", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:surface(5):wait(100)
        local result = act:build()

        expect(result:sub(-2)):toBe("\\e")
    end)

    test("can be called multiple times", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

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
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:surface(5):wait(100)
        act:reset()
        local result = act:build()

        expect(result):toBe("\\e")
    end)

    test("clears scope state", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:reset()
        act:talk(actors.sakura, "Hello again")
        local result = act:build()

        -- After reset, scope tag should be emitted again
        expect(result:find("\\0")):toBeTruthy()
    end)

    test("does not clear token buffer (parent behavior)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        local token_count = #act.token
        act:reset()

        expect(#act.token):toBe(token_count)
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:reset()
        expect(returned):toBe(act)
    end)
end)

-- Test escape processing
describe("SHIORI_ACT - escape", function()
    test("escapes backslash", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "path\\to\\file")
        local result = act:build()

        expect(result:find("path\\\\to\\\\file")):toBeTruthy()
    end)

    test("escapes percent", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "100%")
        local result = act:build()

        expect(result:find("100%%%%")):toBeTruthy()
    end)

    test("escapes mixed special characters", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "50% off \\ sale")
        local result = act:build()

        expect(result:find("50%%%%")):toBeTruthy()
        expect(result:find("\\\\")):toBeTruthy()
    end)
end)

-- E2E scenario test
describe("SHIORI_ACT - E2E scenario", function()
    test("complex script generation", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "こんにちは")
            :surface(5)
            :wait(500)
            :talk(actors.kero, "やあ")
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
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- First round
        act:talk(actors.sakura, "First")
        local result1 = act:build()
        expect(result1:find("First")):toBeTruthy()

        -- Reset and second round
        act:reset()
        act:talk(actors.kero, "Second")
        local result2 = act:build()

        expect(result2:find("First")):toBeFalsy() -- First should be cleared
        expect(result2:find("Second")):toBeTruthy()
        expect(result2:find("\\1")):toBeTruthy()  -- kero scope
    end)
end)
