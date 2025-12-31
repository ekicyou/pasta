-- hello.lua
-- Dummy script for pasta_lua scripts layer
-- This is a sample Lua script demonstrating the basic structure

local function greet(name)
    return "Hello, " .. (name or "World") .. "!"
end

local function main()
    print(greet("pasta_lua"))
    return true
end

-- Return module exports
return {
    greet = greet,
    main = main
}
