local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local mocks = require("lua_test.mocks")

describe("lua_test.mocks - install", function()
    test("install() registers all 5 modules in package.loaded", function()
        mocks.reset() -- ensure clean state
        mocks.install()
        expect(package.loaded["@pasta_persistence"]).not_:toBe(nil)
        expect(package.loaded["@pasta_search"]).not_:toBe(nil)
        expect(package.loaded["@pasta_sakura_script"]).not_:toBe(nil)
        expect(package.loaded["@pasta_config"]).not_:toBe(nil)
        expect(package.loaded["@pasta_log"]).not_:toBe(nil)
        mocks.reset()
    end)

    test("install() persistence stub has load and save", function()
        mocks.install()
        local p = package.loaded["@pasta_persistence"]
        expect(type(p.load)):toBe("function")
        expect(type(p.save)):toBe("function")
        -- load returns empty table
        local data = p.load()
        expect(type(data)):toBe("table")
        -- save returns true
        expect(p.save({})):toBe(true)
        mocks.reset()
    end)

    test("install() search stub uses metatable catch-all", function()
        mocks.install()
        local s = package.loaded["@pasta_search"]
        -- Any method call should return a function that returns nil
        local fn = s.search_scene
        expect(type(fn)):toBe("function")
        expect(fn("test")):toBe(nil)
        local fn2 = s.search_word
        expect(type(fn2)):toBe("function")
        expect(fn2("test")):toBe(nil)
        mocks.reset()
    end)

    test("install() log stub has all 5 log functions", function()
        mocks.install()
        local log = package.loaded["@pasta_log"]
        expect(type(log.trace)):toBe("function")
        expect(type(log.debug)):toBe("function")
        expect(type(log.info)):toBe("function")
        expect(type(log.warn)):toBe("function")
        expect(type(log.error)):toBe("function")
        -- calling them should not error
        log.trace("test")
        log.debug("test")
        log.info("test")
        log.warn("test")
        log.error("test")
        mocks.reset()
    end)

    test("install() sakura_script stub has talk_to_script and break_lines", function()
        mocks.install()
        local ss = package.loaded["@pasta_sakura_script"]
        expect(type(ss.talk_to_script)):toBe("function")
        expect(type(ss.break_lines)):toBe("function")
        expect(ss.talk_to_script("actor", "hello")):toBe("hello")
        expect(ss.break_lines("text")):toBe("text")
        mocks.reset()
    end)
end)

describe("lua_test.mocks - custom stubs", function()
    test("install({ log = custom }) replaces default log stub", function()
        local called = false
        local custom_log = {
            trace = function() called = true end,
            debug = function() end,
            info = function() end,
            warn = function() end,
            error = function() end,
        }
        mocks.install({ log = custom_log })
        local log = package.loaded["@pasta_log"]
        log.trace("test")
        expect(called):toBe(true)
        -- verify it's the custom object, not default
        expect(log):toBe(custom_log)
        mocks.reset()
    end)

    test("install({ persistence = custom }) replaces default persistence stub", function()
        local custom_persistence = {
            load = function() return { key = "value" } end,
            save = function() return false end,
        }
        mocks.install({ persistence = custom_persistence })
        local p = package.loaded["@pasta_persistence"]
        expect(p):toBe(custom_persistence)
        expect(p.load().key):toBe("value")
        expect(p.save()):toBe(false)
        mocks.reset()
    end)
end)

describe("lua_test.mocks - reset", function()
    test("reset() sets all 5 module entries to nil", function()
        mocks.install()
        -- verify installed
        expect(package.loaded["@pasta_persistence"]).not_:toBe(nil)
        -- reset
        mocks.reset()
        expect(package.loaded["@pasta_persistence"]):toBe(nil)
        expect(package.loaded["@pasta_search"]):toBe(nil)
        expect(package.loaded["@pasta_sakura_script"]):toBe(nil)
        expect(package.loaded["@pasta_config"]):toBe(nil)
        expect(package.loaded["@pasta_log"]):toBe(nil)
    end)
end)
