-- hello.lua
-- Dummy script for pasta_lua scripts layer
-- This is a sample Lua script demonstrating the basic structure

local function 挨拶(name)
    return "こんちわ、" .. (name or "World") .. "！"
end

local function main()
    print(挨拶("pasta_lua"))
    return true
end

-- Return module exports
return {
    挨拶 = 挨拶,
    main = main
}
