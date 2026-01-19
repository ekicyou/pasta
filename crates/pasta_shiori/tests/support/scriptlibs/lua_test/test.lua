local expect = require("lua_test.expect")

---@class Test
---@field name string
---@field func function
---@field result { success: boolean, error: string } | nil

---Group of tests and test contexts.
---@class TestContext
---@field parent TestContext | nil
---@field name string
---@field children (Test | TestContext)[]
---@field result { success: boolean } | nil


---Global context to detect current test context when `describe` and `test` are called.
---@type TestContext | nil
_ENV.__testContext = nil


---Colorize text with green.
---@param text string
---@return string
local function chalkGreen(text)
    if os.getenv("NO_COLOR") then
        return text
    else
        return "\x1b[92m" .. text .. "\x1b[0m"
    end
end

---Colorize text with red.
---@param text string
---@return string
local function chalkRed(text)
    if os.getenv("NO_COLOR") then
        return text
    else
        return "\x1b[91m" .. text .. "\x1b[0m"
    end
end

---Dump test context tree. Only for debug.
---@param ctx TestContext | Test
---@param indent integer
local function dumpTest(ctx, indent)
    if ctx.children ~= nil then
        print(("  "):rep(indent) .. ctx.name .. "(" .. #ctx.children .. ")")
        for _, child in ipairs(ctx.children) do
            dumpTest(child, indent + 1)
        end
    else
        print(("  "):rep(indent) .. "*" .. ctx.name)
    end
end

---Perform test for the given test context tree.
---@param ctx TestContext | Test
---@param depth integer
---@return boolean succeeded
local function performTest(ctx, depth)
    if ctx.func ~= nil then
        -- `ctx` is the Test

        local success, err = pcall(ctx.func)
        ctx.result = {
            success = success,
            error = tostring(err),
        }
    else
        -- `ctx` is test context

        ctx.result = { success = true }
        for _, child in ipairs(ctx.children) do
            if not performTest(child, depth + 1) then
                ctx.result.success = false
            end
        end
    end

    return ctx.result.success
end

---Print test result.
---@param ctx TestContext
---@param depth integer
---@param path string[]
---@return { name: string, error: string }[]
local function printTestResult(ctx, depth, path)
    ---@type { name: string, error: string }[]
    local errors = {}

    local newPath = table.pack(table.unpack(path))
    table.insert(newPath, ctx.name)

    local resultMark = chalkGreen("✔")
    if not ctx.result.success then
        resultMark = chalkRed("✘")
    end

    if ctx.children ~= nil then
        --- `ctx` is test context

        local successCount = 0
        for _, child in ipairs(ctx.children) do
            if child.result.success then
                successCount = successCount + 1
            end
        end

        local nameAndResult = table.concat({
            ("  "):rep(depth),
            ctx.name,
            " (",
            successCount,
            "/",
            #ctx.children,
            ")",
            resultMark,
        }, "")
        print(nameAndResult)

        for _, child in ipairs(ctx.children) do
            local childErrors = printTestResult(child, depth + 1, newPath)
            for _, err in ipairs(childErrors) do
                table.insert(errors, err)
            end
        end

        print(nameAndResult)
    else
        --- `ctx` is test

        print(table.concat({
            ("  "):rep(depth),
            ctx.name,
            " ",
            resultMark,
        }, ""))

        if (not ctx.result.success) and ctx.result.error ~= nil then
            table.insert(errors, {
                name = table.concat(newPath, " > "),
                error = ctx.result.error,
            })
        end
    end

    if depth == 0 then
        if #errors == 0 then
            print(chalkGreen("All tests passed."))
        else
            print(chalkRed(string.format("%d test(s) failed.", #errors)))

            local printErrors = {}
            for _, error in ipairs(errors) do
                table.insert(printErrors, chalkRed("Error in " .. error.name) .. "\n" .. error.error)
            end

            print(table.concat(printErrors, "\n\n"))
        end
    end

    return errors
end


---Define test context
---@param name string
---@param func fun()
local function describe(name, func)
    ---@type TestContext
    local ctx = {
        parent = nil,
        name = name,
        children = {},
        result = nil,
    }

    if _ENV.__testContext ~= nil then
        -- Set up parent-child relationship
        table.insert(_ENV.__testContext.children, ctx)
        ctx.parent = _ENV.__testContext
    end

    -- Set current test context
    _ENV.__testContext = ctx

    func()

    if _ENV.__testContext.parent == nil then
        --dumpTest(_ENV.__testContext, 0)
        local succeeded = performTest(_ENV.__testContext, 0)
        printTestResult(_ENV.__testContext, 0, {})

        if not succeeded then
            os.exit(1)
        end
    else
        -- Restore parent context
        _ENV.__testContext = _ENV.__testContext.parent
    end
end

---Define test
---@param name string
---@param func fun()
local function test(name, func)
    if _ENV.__testContext == nil then
        -- Define anonymous test context
        _ENV.__testContext = {
            parent = nil,
            name = "(anonymous)",
            children = {},
            result = nil,
        }
    end

    table.insert(_ENV.__testContext.children, {
        name = name,
        func = func,
        result = nil,
    })
end

return {
    expect = expect,
    test = test,
    describe = describe,
}
