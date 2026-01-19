---@type fun(value: any): string
local toDebugString = nil

---@param value string
---@return string
local function escapeString(value)
    local result = value:gsub("\\", "\\\\")
        :gsub("\a", "\\a")
        :gsub("\b", "\\b")
        :gsub("\f", "\\f")
        :gsub("\n", "\\n")
        :gsub("\r", "\\r")
        :gsub("\t", "\\t")
        :gsub("\v", "\\v")
        :gsub("\"", "\\\"")
        :gsub("[^%g%s]", function(c)
            return string.format("\\x%02X", c:byte())
        end)
    return result
end

---@param value table
---@return string[]
local function arrayPartToStrings(value)
    local indices = {}
    for k, _ in pairs(value) do
        if type(k) == "number" then
            table.insert(indices, k)
        end
    end
    table.sort(indices)

    if #indices == 0 then
        return {}
    end

    local result = {}

    if indices[1] == 1 then
        table.insert(result, toDebugString(value[indices[1]]))
    else
        table.insert(result, string.format("[%s] = %s", tostring(indices[1]), toDebugString(value[indices[1]])))
    end

    for i = 2, #indices do
        local valueString = toDebugString(value[indices[i]])
        local diff = indices[i] - indices[i - 1]

        if diff == 1 then
            table.insert(result, valueString)
        elseif diff == math.floor(diff) and diff < 10 then
            table.insert(result, string.format("(empty x %d)", diff - 1))
            table.insert(result, valueString)
        else
            table.insert(result, string.format("[%s] = %s", tostring(indices[i]), valueString))
        end
    end

    return result
end

---@param value table
---@return string[]
local function mapPartToStrings(value)
    local keys = {}
    for k, _ in pairs(value) do
        if type(k) == "string" then
            table.insert(keys, k)
        end
    end
    table.sort(keys)

    local result = {}
    for _, key in pairs(keys) do
        table.insert(result, string.format("%s = %s", key, toDebugString(value[key])))
    end

    return result
end

toDebugString = function(value)
    if type(value) == "string" then
        return "\"" .. escapeString(value) .. "\""
    elseif type(value) == "table" then
        local arrayPart = arrayPartToStrings(value)
        local mapPart = mapPartToStrings(value)

        local items = {}
        table.move(arrayPart, 1, #arrayPart, #items + 1, items)
        table.move(mapPart, 1, #mapPart, #items + 1, items)

        if #items == 0 then
            return "{}"
        else
            return "{ " .. table.concat(items, ", ") .. " }"
        end
    end

    return tostring(value)
end

return toDebugString
