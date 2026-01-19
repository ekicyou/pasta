--- @module pasta.shiori.main
--- SHIORI/3.0 Protocol Entry Point with Observable Side Effects
---
--- This module provides a SHIORI implementation that demonstrates
--- actual Lua code execution through observable side effects.
--- Used for lifecycle integration testing.

-- Global SHIORI table with observable side effects
SHIORI = SHIORI or {}

-- Track load state (Requirement 1)
SHIORI.loaded_hinst = nil
SHIORI.load_dir = nil

-- Track request count (Requirement 2)
SHIORI.request_count = 0

-- File marker path for unload verification (Requirement 3)
local unload_marker_path = nil

--- SHIORI.load - Sets observable globals
--- Called when the SHIORI DLL is loaded by the baseware.
---
--- @param hinst integer DLL handle
--- @param load_dir string Load directory path
--- @return boolean success
function SHIORI.load(hinst, load_dir)
    SHIORI.loaded_hinst = hinst
    SHIORI.load_dir = load_dir
    unload_marker_path = load_dir .. "/unload_called.marker"
    return true
end

--- SHIORI.request - Increments counter, calls Pasta scene
--- Called for each SHIORI request from the baseware.
---
--- @param request_text string Raw SHIORI request text
--- @return string SHIORI response with scene output
function SHIORI.request(request_text)
    SHIORI.request_count = SHIORI.request_count + 1

    -- Call Pasta scene via @pasta_search (Requirement 4)
    local SEARCH = require "@pasta_search"
    local global_name, local_name = SEARCH:search_scene("テスト挨拶", nil)

    local scene_output = ""
    if global_name then
        local scene_module = _G[global_name]
        if scene_module and scene_module[local_name] then
            local result = scene_module[local_name]()
            if result then
                scene_output = tostring(result)
            else
                scene_output = "ライフサイクルテスト成功！"
            end
        else
            scene_output = "ライフサイクルテスト成功！"
        end
    else
        -- Scene not found - return error for test visibility
        return "SHIORI/3.0 500 Internal Server Error\r\n" ..
            "Charset: UTF-8\r\n" ..
            "Value: Scene 'テスト挨拶' not found\r\n" ..
            "\r\n"
    end

    return "SHIORI/3.0 200 OK\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Value: " .. scene_output .. "\r\n" ..
        "\r\n"
end

--- SHIORI.unload - Creates file marker for verification
--- Called when the SHIORI DLL is unloaded.
function SHIORI.unload()
    if unload_marker_path then
        local f = io.open(unload_marker_path, "w")
        if f then
            f:write("unloaded")
            f:close()
        end
    end
end

return SHIORI
