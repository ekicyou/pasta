--- Lua mock injection library for pasta backend modules.
--- Provides default stubs and install/reset API for test isolation.
local M = {}

-- Module names
local MODULES = {
    "@pasta_persistence",
    "@pasta_search",
    "@pasta_sakura_script",
    "@pasta_config",
    "@pasta_log",
}

-- Default stub factories

function M.make_persistence()
    return {
        load = function() return {} end,
        save = function(_data) return true end,
    }
end

function M.make_search()
    return setmetatable({}, {
        __index = function() return function() return nil end end,
    })
end

function M.make_sakura_script()
    return {
        talk_to_script = function(_actor, text) return text or "" end,
        break_lines = function(text, ...) return text or "" end,
    }
end

function M.make_config()
    return {}
end

function M.make_log()
    local noop = function() end
    return {
        trace = noop,
        debug = noop,
        info = noop,
        warn = noop,
        error = noop,
    }
end

-- Map from short name to module name and factory
local FACTORY_MAP = {
    persistence = { name = "@pasta_persistence", factory = M.make_persistence },
    search = { name = "@pasta_search", factory = M.make_search },
    sakura_script = { name = "@pasta_sakura_script", factory = M.make_sakura_script },
    config = { name = "@pasta_config", factory = M.make_config },
    log = { name = "@pasta_log", factory = M.make_log },
}

--- Install all module stubs into package.loaded.
--- @param opts table|nil Optional table of custom stubs keyed by short name (e.g., { persistence = {...} })
function M.install(opts)
    opts = opts or {}
    for short, entry in pairs(FACTORY_MAP) do
        if opts[short] then
            package.loaded[entry.name] = opts[short]
        else
            package.loaded[entry.name] = entry.factory()
        end
    end
end

--- Reset all module stubs (set package.loaded entries to nil).
function M.reset()
    for _, name in ipairs(MODULES) do
        package.loaded[name] = nil
    end
end

return M
