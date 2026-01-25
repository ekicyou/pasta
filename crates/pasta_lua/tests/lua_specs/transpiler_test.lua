-- Sample transpiler test
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("hello module", function()
    test("module exists", function()
        local hello = require("hello")
        expect(hello):toBeTruthy()
    end)

    test("挨拶 function", function()
        local hello = require("hello")
        local result = hello.挨拶("World")
        expect(result):toBe("こんちわ、World！")
    end)

    test("main function", function()
        local hello = require("hello")
        local result = hello.main()
        expect(result):toBe(true)
    end)
end)
